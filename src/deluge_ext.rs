use std::default::Default;
use std::future::Future;

use crate::deluge::Deluge;
use crate::ops::*;

impl<'a, T> DelugeExt<'a> for T where T: Deluge<'a> {}

pub trait DelugeExt<'a>: Deluge<'a> {
    /// Transforms each element by applying an asynchronous function `f` to it
    /// 
    /// # Examples
    /// 
    /// ```
    /// let result = deluge::iter([1, 2, 3, 4])
    ///     .map(|x| async move { x * 2 })
    ///     .collect::<Vec<usize>>(None)
    ///     .await;
    /// 
    /// assert_eq!(vec![2, 4, 6, 8], result);
    /// ```
    fn map<Fut, F>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::Item) -> Fut + Send + 'a,
        Fut: Future + Send,
        Self: Sized,
    {
        Map::new(self, f)
    }

    /// Leaves the elements for which `f` returns a promise evaluating to `true`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let result = (0..10).into_deluge()
    ///     .filter(|x| async move { idx % 2 == 0 })
    ///     .collect::<Vec<usize>>()
    ///     .await;
    /// 
    /// assert_eq!(vec![0, 2, 4, 6, 8], result);
    /// ```
    fn filter<F>(self, f: F) -> Filter<Self, F>
    where
        for<'b> F: XFn<'b, &'b Self::Item, bool> + Send + 'b,
        Self: Sized,
    {
        Filter::new(self, f)
    }

    /// Concurrently accummulates values in the accummulator. The degree of concurrency 
    /// can either be unlimited (the default) or limited depending on the requirements.
    /// 
    /// # Examples
    /// 
    /// Unlimited concurrency:
    /// 
    /// ```
    /// let result = (0..100).into_deluge()
    ///     .fold(None, 0, |acc, x| async move { acc + x })
    ///     .await;
    /// 
    /// assert_eq!(result, 4950);
    /// ```
    /// 
    /// Concurrency limited to at most ten futures evaluated at once:
    ///
    /// ```
    /// let result = (0..100).into_deluge()
    ///     .fold(10, 0, |acc, x| async move { acc + x })
    ///     .await;
    /// 
    /// assert_eq!(result, 4950);
    /// ```
    fn fold<Acc, F, Fut>(
        self,
        concurrency: impl Into<Option<usize>>,
        acc: Acc,
        f: F,
    ) -> Fold<'a, Self, Acc, F, Fut>
    where
        F: FnMut(Acc, Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = Acc> + Send + 'a,
        Self: Sized,
    {
        Fold::new(self, concurrency, acc, f)
    }

    /// Accummulates values in an accummulator with futures evaluated in parallel.
    /// The number of workers spawned and concurrency for each worker can be controlled.
    /// By default the number of workers equals the number of logical cpus
    /// and concurrency for each worker is the total number of futures to evaluate 
    /// divided by the number of available workers.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let result = (0..100).into_deluge()
    ///     .fold_par(None, None, 0, |acc, x| async move { acc + x })
    ///     .await;
    /// 
    /// assert_eq!(result, 4950);
    /// ```
    #[cfg(feature = "parallel")]
    fn fold_par<Acc, F, Fut>(
        self,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
        acc: Acc,
        f: F,
    ) -> FoldPar<'a, Self, Acc, F, Fut>
    where
        F: FnMut(Acc, Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = Acc> + Send + 'a,
        Self: Sized,
    {
        FoldPar::new(self, worker_count, worker_concurrency, acc, f)
    }

    /// Consumes at most `how_many` elements from the Deluge, ignoring the rest.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let result = (0..100).into_deluge()
    ///     .take(1)
    ///     .fold(None, 0, |acc, x| async move { acc + x })
    ///     .await;
    /// 
    /// assert_eq!(0, result);
    /// ```
    fn take(self, how_many: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, how_many)
    }

    fn collect<C>(self, concurrency: impl Into<Option<usize>>) -> Collect<'a, Self, C>
    where
        C: Default + Extend<Self::Item>,
        Self: Sized,
    {
        Collect::new(self, concurrency)
    }

    #[cfg(feature = "parallel")]
    fn collect_par<C>(
        self,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
    ) -> CollectPar<'a, Self, C>
    where
        C: Default + Extend<Self::Item>,
        Self: Sized,
    {
        CollectPar::new(self, worker_count, worker_concurrency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::into_deluge::IntoDeluge;
    use crate::iter::iter;
    use more_asserts::{assert_gt, assert_lt};
    use std::time::{Duration, Instant};

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
    async fn take_until_a_limit() {
        let result = (0..100)
            .into_deluge()
            .take(10)
            .fold(None, 0, |acc, idx| async move { acc + idx })
            .await;

        assert_eq!(result, 45);
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
        let result = (0..150)
            .into_deluge()
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
        let result = (0..150)
            .into_deluge()
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
