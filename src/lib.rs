#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]
#![feature(map_first_last)]
#![feature(let_chains)]
#![feature(stmt_expr_attributes)]

mod deluge;
mod deluge_ext;
mod into_deluge;
mod iter;
mod ops;

pub use self::deluge::*;
pub use deluge_ext::*;
pub use into_deluge::*;
pub use iter::*;
pub use ops::*;

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::{assert_gt, assert_lt};
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn we_can_create_iter() {
        let _del = iter([1, 2, 3]);
        assert_eq!(2, 2);
    }

    #[tokio::test]
    async fn map_can_be_created() {
        iter([1, 2, 3, 4]).map(|x| async move { x * 2 });
        assert_eq!(2, 2);
    }

    #[tokio::test]
    async fn we_can_collect() {
        let result = iter([1, 2, 3, 4]).collect::<Vec<usize>>(None).await;

        assert_eq!(vec![1, 2, 3, 4], result);
    }

    #[tokio::test]
    async fn we_can_mult() {
        let result = iter([1, 2, 3, 4])
            .map(|x| async move { x * 2 })
            .collect::<Vec<usize>>(None)
            .await;

        assert_eq!(vec![2, 4, 6, 8], result);
    }

    #[tokio::test]
    async fn we_can_go_between_values_and_deluges() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .map(|x| async move { x * 2 })
            .collect::<Vec<usize>>(None)
            .await
            .into_deluge()
            .map(|x| async move { x * 2 })
            .fold(None, 0, |acc, x| async move { acc + x })
            .await;

        assert_eq!(result, 40);
    }

    #[tokio::test]
    async fn we_wait_cuncurrently() {
        let start = Instant::now();
        let result = iter(0..100)
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                idx
            })
            .collect::<Vec<usize>>(None)
            .await;

        let iteration_took = Instant::now() - start;
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result.len(), 100);

        result
            .into_iter()
            .enumerate()
            .for_each(|(idx, elem)| assert_eq!(idx, elem));
    }

    #[tokio::test]
    async fn concurrency_limit() {
        let start = Instant::now();
        let result = iter(0..15)
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                idx
            })
            .collect::<Vec<usize>>(5)
            .await;

        let iteration_took = Instant::now() - start;
        assert_gt!(iteration_took.as_millis(), 150);
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result.len(), 15);
    }

    #[tokio::test]
    async fn concurrent_fold() {
        let start = Instant::now();
        let result = iter(0..100)
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                idx
            })
            .fold(None, 0, |acc, idx| async move { acc + idx })
            .await;

        let iteration_took = Instant::now() - start;
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result, 4950);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn parallel_test() {
        let start = Instant::now();
        let result = iter(0..150)
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                idx
            })
            .collect_par::<Vec<usize>>(10, 5)
            .await;

        let iteration_took = Instant::now() - start;
        assert_gt!(iteration_took.as_millis(), 150);
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result.len(), 150);
    }

    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn parallel_test() {
        let start = Instant::now();
        let result = iter(0..150)
            .map(|idx| async move {
                async_std::task::sleep(Duration::from_millis(50)).await;
                idx
            })
            .collect_par::<Vec<usize>>(10, 5)
            .await;

        let iteration_took = Instant::now() - start;
        assert_gt!(iteration_took.as_millis(), 150);
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result.len(), 150);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn parallel_fold() {
        let start = Instant::now();
        let result = iter(0..150)
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                idx
            })
            .fold_par(10, 5, 0, |acc, x| async move { acc + x })
            .await;

        let iteration_took = Instant::now() - start;
        assert_gt!(iteration_took.as_millis(), 150);
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result, 11175);
    }

    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn parallel_fold() {
        let start = Instant::now();
        let result = iter(0..150)
            .map(|idx| async move {
                async_std::task::sleep(Duration::from_millis(50)).await;
                idx
            })
            .fold_par(10, 5, 0, |acc, x| async move { acc + x })
            .await;

        let iteration_took = Instant::now() - start;
        assert_gt!(iteration_took.as_millis(), 150);
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result, 11175);
    }


    // Filter doesn't want to build, I have no idea why.
    // Let's move to augmenting the collector first
    /*
    #[tokio::test]
    async fn filter_works() {
        let result = iter(0..100)
            .filter(|idx| async move {
                idx % 2 == 0
            })
            .collect::<Vec<usize>>().await;

        assert_eq!(result.len(), 50);
        result.into_iter()
            .enumerate()
            .for_each(|(idx, elem)| assert_eq!(idx * 2, elem));
    }
    */
}
