#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]

mod ops;
mod deluge;
mod deluge_ext;
mod iter;

pub use ops::*;
pub use self::deluge::*;
pub use deluge_ext::*;
pub use iter::*;


// TODO:
// - [?] add filter
// - [ ] add filter_map
// - [ ] add fold
// - [x] rearrange files
// - [x] Control the degree of concurrency on collect
// - [ ] Add a parallel collector


#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::{assert_lt, assert_gt};
    use tokio::time::{Duration, Instant};

    #[tokio::test]
    async fn we_can_create_iter() {
        let _del = iter([1, 2, 3]);
        assert_eq!(2, 2);
    }

    #[tokio::test]
    async fn map_can_be_created() {
        iter([1, 2, 3, 4])
            .map(|x| async move { x * 2 });
        assert_eq!(2, 2);
    }

    #[tokio::test]
    async fn we_can_collect() {
        let result = iter([1, 2, 3, 4])
            .collect::<Vec<usize>>(None).await;

        assert_eq!(vec![1, 2, 3, 4], result);
    }

    #[tokio::test]
    async fn we_can_mult() {
        let result = iter([1, 2, 3, 4])
            .map(|x| async move { x * 2 })
            .collect::<Vec<usize>>(None).await;

        assert_eq!(vec![2, 4, 6, 8], result);
    }

    #[tokio::test]
    async fn we_wait_cuncurrently() {
        let start = Instant::now();
        let result = iter(0..100)
            .map(|idx| async move { 
                tokio::time::sleep(Duration::from_millis(100)).await;
                idx
            })
            .collect::<Vec<usize>>(None).await;

        let iteration_took = Instant::now() - start;
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result.len(), 100);

        result.into_iter()
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
            .collect::<Vec<usize>>(5).await;

        let iteration_took = Instant::now() - start;
        assert_gt!(iteration_took.as_millis(), 150);
        assert_lt!(iteration_took.as_millis(), 200);

        assert_eq!(result.len(), 15);
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
