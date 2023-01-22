use std::default::Default;
use std::future::Future;

use crate::deluge::Deluge;
use crate::ops::*;

impl<T> DelugeExt for T where T: Deluge {}

/// Exposes easy to use Deluge operations. **This should be your first step**
pub trait DelugeExt: Deluge {
    /// Resolves to true if all of the calls to `F` return true
    /// for each element of the deluge.
    /// Evaluates the elements concurrently and short circuits evaluation
    /// on a first failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .all(None, |x| async move { x < 5 })
    ///     .await;
    ///
    /// assert!(result);
    ///
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .all(None, |x| async move { x < 3 })
    ///     .await;
    ///
    /// assert!(!result);
    /// # });
    /// ```
    fn all<'a, Fut, F>(self, concurrency: impl Into<Option<usize>>, f: F) -> All<'a, Self, Fut, F>
    where
        F: Fn(Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = bool> + Send,
        Self: Sized,
    {
        All::new(self, concurrency, f)
    }

    /// A parallel version of `DelugeExt::all`.
    /// Resolves to true if all of the calls to `F` return true
    /// for each element of the deluge.
    /// Evaluates the elements in parallel and short circuits evaluation
    /// on a first failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .all_par(None, None, |x| async move { x < 5 })
    ///     .await;
    ///
    /// assert!(result);
    ///
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .all_par(None, None, |x| async move { x < 3 })
    ///     .await;
    ///
    /// assert!(!result);
    /// # });
    /// ```
    fn all_par<'a, Fut, F>(
        self,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
        f: F,
    ) -> AllPar<'a, Self, Fut, F>
    where
        F: Fn(Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = bool> + Send,
        Self: Sized,
    {
        AllPar::new(self, worker_count, worker_concurrency, f)
    }

    /// Resolves to true if any of the calls to `F` return true
    /// for any element of the deluge.
    /// Evaluates the elements concurrently and short circuits evaluation
    /// on a successful result.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .any(None, |x| async move { x == 4 })
    ///     .await;
    ///
    /// assert!(result);
    ///
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .any(None, |x| async move { x > 10 })
    ///     .await;
    ///
    /// assert!(!result);
    /// # });
    /// ```
    fn any<'a, Fut, F>(self, concurrency: impl Into<Option<usize>>, f: F) -> Any<'a, Self, Fut, F>
    where
        F: Fn(Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = bool> + Send,
        Self: Sized,
    {
        Any::new(self, concurrency, f)
    }

    /// A parallel version of `DelugeExt::any`.
    /// Resolves to true if any of the calls to `F` return true
    /// for any element of the deluge.
    /// Evaluates the elements in parallel and short circuits evaluation
    /// on a successful result.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .any_par(None, None, |x| async move { x == 4 })
    ///     .await;
    ///
    /// assert!(result);
    ///
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .any_par(None, None, |x| async move { x > 10 })
    ///     .await;
    ///
    /// assert!(!result);
    /// # });
    /// ```
    fn any_par<'a, Fut, F>(
        self,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
        f: F,
    ) -> AnyPar<'a, Self, Fut, F>
    where
        F: Fn(Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = bool> + Send,
        Self: Sized,
    {
        AnyPar::new(self, worker_count, worker_concurrency, f)
    }

    /// Chains two deluges together
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .chain([5, 6, 7, 8].into_deluge())
    ///     .collect::<Vec<usize>>(None)
    ///     .await;
    ///
    /// assert_eq!(result.len(), 8);
    /// assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    /// # })
    /// ```
    fn chain<'a, Del2>(self, deluge2: Del2) -> Chain<'a, Self, Del2>
    where
        Del2: for<'x> Deluge<Item = Self::Item, Output<'x> = Self::Output<'x>> + 'static,
        Self: Sized,
    {
        Chain::new(self, deluge2)
    }

    /// Consumes all the items in a deluge and returns
    /// the number of elements that were observed.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .count();
    ///
    /// assert_eq!(result, 4);
    /// # })
    /// ```
    fn count(self) -> usize
    where
        Self: Sized,
    {
        count(self)
    }

    /// Transforms each element by applying an asynchronous function `f` to it
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = [1, 2, 3, 4]
    ///     .into_deluge()
    ///     .map(|x| async move { x * 2 })
    ///     .collect::<Vec<usize>>(None)
    ///     .await;
    ///
    /// assert_eq!(vec![2, 4, 6, 8], result);
    /// # });
    /// ```
    fn map<Fut, F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> Fut + Send,
        Fut: Future + Send,
        Self: Sized,
    {
        Map::new(self, f)
    }

    /// Leaves the elements for which `f` returns a promise evaluating to `true`.
    ///
    /// # WARNING
    ///
    /// Currently has [a breaking bug](https://github.com/mkawalec/deluge/issues/1).
    ///
    /// # Examples
    ///
    // ```
    // use deluge::*;
    //
    // # futures::executor::block_on(async {
    // let result = (0..10).into_deluge()
    //     .filter(|x| async move { x % 2 == 0 })
    //     .collect::<Vec<usize>>(None)
    //     .await;
    //
    // assert_eq!(vec![0, 2, 4, 6, 8], result);
    // # });
    //
    // ```
    // 27.10.2022: Commented out filter until the issue#1 is solved
    /*fn filter<'a, F>(self, f: F) -> Filter<'a, Self, F>
    where
        for<'b> F: XFn<'b, Self::Item, bool> + Send + 'b,
        Self: Sized,
    {
        Filter::new(self, f)
    }*/

    /// Filters out elements for which a function returns `None`,
    /// substitutes the elements for the ones there it returns `Some(new_value)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..10).into_deluge()
    ///     .filter_map(|x| async move { if x % 2 == 0 {
    ///             Some(x.to_string())
    ///         } else {
    ///             None
    ///         }
    ///     })
    ///     .collect::<Vec<String>>(None)
    ///     .await;
    ///
    /// assert_eq!(vec![0.to_string(), 2.to_string(), 4.to_string(), 6.to_string(), 8.to_string()], result);
    /// # });
    ///
    /// ```
    fn filter_map<Fut, F>(self, f: F) -> FilterMap<Self, F>
    where
        F: Fn(Self::Item) -> Fut + Send,
        Fut: Future + Send,
        Self: Sized,
    {
        FilterMap::new(self, f)
    }

    /// Returns the first element of the input deluge and then finishes
    ///
    /// # Examples
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..10).into_deluge()
    ///     .first()
    ///     .collect::<Vec<usize>>(None)
    ///     .await;
    ///
    /// assert_eq!(vec![0], result);
    /// # });
    ///
    /// ```
    fn first(self) -> First<Self>
    where
        Self: Sized,
    {
        First::new(self)
    }

    /// Concurrently accummulates values in the accummulator. The degree of concurrency
    /// can either be unlimited (the default) or limited depending on the requirements.
    ///
    /// # Examples
    ///
    /// Unlimited concurrency:
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).into_deluge()
    ///     .fold(None, 0, |acc, x| async move { acc + x })
    ///     .await;
    ///
    /// assert_eq!(result, 4950);
    /// # });
    /// ```
    ///
    /// Concurrency limited to at most ten futures evaluated at once:
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).into_deluge()
    ///     .fold(10, 0, |acc, x| async move { acc + x })
    ///     .await;
    ///
    /// assert_eq!(result, 4950);
    /// # });
    /// ```
    fn fold<Acc, F, Fut>(
        self,
        concurrency: impl Into<Option<usize>>,
        acc: Acc,
        f: F,
    ) -> Fold<Self, Acc, F, Fut>
    where
        F: FnMut(Acc, Self::Item) -> Fut + Send,
        Fut: Future<Output = Acc> + Send,
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
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).into_deluge()
    ///     .fold_par(None, None, 0, |acc, x| async move { acc + x })
    ///     .await;
    ///
    /// assert_eq!(result, 4950);
    /// # });
    /// ```
    #[cfg(feature = "async-runtime")]
    fn fold_par<'a, Acc, F, Fut>(
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

    /// Returns the last element of the input deluge and then finishes
    ///
    /// # Examples
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..10).into_deluge()
    ///     .last()
    ///     .collect::<Vec<usize>>(None)
    ///     .await;
    ///
    /// assert_eq!(vec![9], result);
    /// # });
    ///
    /// ```
    fn last(self) -> Last<Self>
    where
        Self: Sized,
    {
        Last::new(self)
    }

    /// Consumes at most `how_many` elements from the Deluge, ignoring the rest.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).into_deluge()
    ///     .take(1)
    ///     .fold(None, 0, |acc, x| async move { acc + x })
    ///     .await;
    ///
    /// assert_eq!(0, result);
    /// # });
    /// ```
    fn take(self, how_many: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, how_many)
    }

    /// Combines two Deluges into one with elements being
    /// tuples of subsequent elements from each
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).rev()
    ///     .into_deluge()
    ///     .zip((0..90).into_deluge(), None)
    ///     .collect::<Vec<(u64, u64)>>(None)
    ///     .await;
    ///
    /// assert_eq!(result.len(), 90);
    /// assert_eq!(result[0], (99, 0));
    /// assert_eq!(result[1], (98, 1));
    /// # });
    /// ```
    fn zip<'a, Del2>(
        self,
        other: Del2,
        concurrency: impl Into<Option<usize>>,
    ) -> Zip<'a, Self, Del2>
    where
        Del2: Deluge + 'a,
        Self: Sized,
    {
        Zip::new(self, other, concurrency)
    }

    /// Collects elements in the current `Deluge` into a collection with a desired concurrency
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).into_deluge()
    ///     .collect::<Vec<usize>>(None)
    ///     .await;
    ///
    /// assert_eq!(result.len(), 100);
    /// # });
    /// ```
    fn collect<'a, C>(self, concurrency: impl Into<Option<usize>>) -> Collect<'a, Self, C>
    where
        C: Default + Extend<Self::Item>,
        Self: Sized,
    {
        Collect::new(self, concurrency)
    }

    /// Collects elements in the current `Deluge` into a collection
    /// in parallel. Optionally accepts a degree of parallelism
    /// and concurrency for each worker.
    ///
    /// If the number of workers is not specified, we will default to the number of logical cpus.
    /// If concurrency per worker is not specified, we will default to the total number of
    /// items in a current deluge divided by the number of workers.
    ///
    /// # Examples
    ///
    /// ```
    /// use deluge::*;
    ///
    /// # futures::executor::block_on(async {
    /// let result = (0..100).into_deluge()
    ///     .collect_par::<Vec<usize>>(None, None)
    ///     .await;
    ///
    /// assert_eq!(result.len(), 100);
    /// # });
    /// ```
    #[cfg(feature = "async-runtime")]
    fn collect_par<'a, C>(
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
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn map_can_be_created() {
        [1, 2, 3, 4].into_deluge().map(|x| async move { x * 2 });
        assert_eq!(2, 2);
    }

    #[tokio::test]
    async fn we_can_collect() {
        let result = [1, 2, 3, 4].into_deluge().collect::<Vec<usize>>(None).await;

        assert_eq!(vec![1, 2, 3, 4], result);
    }

    #[tokio::test]
    async fn any_works() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .any(None, |x| async move { x == 4 })
            .await;

        assert!(result);
    }

    #[tokio::test]
    async fn any_short_circuits() {
        let evaluated = Arc::new(Mutex::new(Vec::new()));

        let result = [1, 2, 3, 4, 5, 6, 7]
            .into_deluge()
            .any(1, |x| {
                let evaluated = evaluated.clone();
                async move {
                    {
                        let mut evaluated = evaluated.lock().await;
                        evaluated.push(x);
                    }
                    tokio::time::sleep(Duration::from_millis(100 - 10 * x)).await;
                    x == 2
                }
            })
            .await;

        assert!(result);
        // We might evaluate a little bit more than we should have, but not much more
        assert_lt!(Arc::try_unwrap(evaluated).unwrap().into_inner().len(), 4);
    }

    #[tokio::test]
    async fn any_par_works() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .any_par(None, None, |x| async move { x == 4 })
            .await;

        assert!(result);
    }

    #[tokio::test]
    async fn any_par_short_circuits() {
        let evaluated = Arc::new(Mutex::new(Vec::new()));

        let result = [1, 2, 3, 4, 5, 6, 7]
            .into_deluge()
            .any_par(2, 1, |x| {
                let evaluated = evaluated.clone();
                async move {
                    {
                        let mut evaluated = evaluated.lock().await;
                        evaluated.push(x);
                    }
                    tokio::time::sleep(Duration::from_millis(100 - 10 * x)).await;
                    x == 2
                }
            })
            .await;

        assert!(result);
        // We might evaluate a little bit more than we should have, but not much more
        assert_lt!(Arc::try_unwrap(evaluated).unwrap().into_inner().len(), 5);
    }

    #[tokio::test]
    async fn all_works() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .all(None, |x| async move { x < 5 }).await;

        assert!(result);
    }

    #[tokio::test]
    async fn all_short_circuits() {
        let evaluated = Arc::new(Mutex::new(Vec::new()));

        let result = [1, 2, 3, 4, 5, 6, 7]
            .into_deluge()
            .all(1, |x| {
                let evaluated = evaluated.clone();
                async move {
                    {
                        let mut evaluated = evaluated.lock().await;
                        evaluated.push(x);
                    }
                    tokio::time::sleep(Duration::from_millis(100 - 10 * x)).await;
                    x < 3
                }
            })
            .await;

        assert!(!result);
        // We might evaluate a little bit more than we should have, but not much more
        assert_lt!(Arc::try_unwrap(evaluated).unwrap().into_inner().len(), 5);
    }

    #[tokio::test]
    async fn all_par_works() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .all_par(None, None, |x| async move { x < 5 })
            .await;

        assert!(result);
    }

    #[tokio::test]
    async fn all_par_short_circuits() {
        let evaluated = Arc::new(Mutex::new(Vec::new()));

        let result = [1, 2, 3, 4, 5, 6, 7]
            .into_deluge()
            .all_par(2, 1, |x| {
                let evaluated = evaluated.clone();
                async move {
                    {
                        let mut evaluated = evaluated.lock().await;
                        evaluated.push(x);
                    }
                    tokio::time::sleep(Duration::from_millis(100 - 10 * x)).await;
                    x < 3
                }
            })
            .await;

        assert!(!result);
        // We might evaluate a little bit more than we should have, but not much more
        assert_lt!(Arc::try_unwrap(evaluated).unwrap().into_inner().len(), 7);
    }

    #[tokio::test]
    async fn chain_works() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .chain([5, 6, 7, 8].into_deluge())
            .collect::<Vec<usize>>(None)
            .await;

        assert_eq!(result.len(), 8);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[tokio::test]
    async fn count_works() {
        let result = [1, 2, 3, 4].into_deluge().count();

        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn we_can_mult() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .map(|x| async move { x * 2 })
            .collect::<Vec<usize>>(None)
            .await;

        assert_eq!(vec![2, 4, 6, 8], result);
    }

    #[tokio::test]
    async fn filter_map_works() {
        let result = [1, 2, 3, 4]
            .into_deluge()
            .filter_map(|x| async move {
                if x % 2 == 0 {
                    Some("yes")
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>(None)
            .await;

        assert_eq!(vec!["yes", "yes"], result);
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
        let result = (0..100)
            .into_deluge()
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(100 - (idx as u64))).await;
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
        let result = (0..15)
            .into_deluge()
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
    async fn first_works() {
        let result = (0..100)
            .into_deluge()
            .first()
            .collect::<Vec<usize>>(None)
            .await;

        assert_eq!(result, vec![0]);
    }

    #[tokio::test]
    async fn last_works() {
        let result = (0..100)
            .into_deluge()
            .last()
            .collect::<Vec<usize>>(None)
            .await;

        assert_eq!(result, vec![99]);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn concurrent_fold() {
        let start = Instant::now();
        let result = (0..100)
            .into_deluge()
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
        let result = (0..150)
            .into_deluge()
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
        let result = (0..150)
            .into_deluge()
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

    #[cfg(feature = "async-runtime")]
    #[tokio::test]
    async fn zips_work() {
        let result = (0..100)
            .into_deluge()
            .zip((10..90).into_deluge(), None)
            .collect::<Vec<(usize, usize)>>(None)
            .await;

        assert_eq!(result.len(), 80);
    }

    #[cfg(feature = "async-runtime")]
    #[tokio::test]
    async fn zips_inverted_waits() {
        let other_deluge = (0..90).into_deluge().map(|idx| async move {
            // We sleep here so first element from this Deluge
            // only becomes available with the last element from the next one
            tokio::time::sleep(Duration::from_millis(idx)).await;
            idx
        });

        let result = (0..100).rev()
            .into_deluge()
            .map(|idx| async move {
                tokio::time::sleep(Duration::from_millis(idx)).await;
                idx
            })
            .zip(other_deluge, None)
            .collect::<Vec<(u64, u64)>>(None)
            .await;

        assert_eq!(result.len(), 90);
        for (idx, (fst, snd)) in result.into_iter().enumerate() {
            assert_eq!(idx as u64, 99 - fst);
            assert_eq!(idx as u64, snd);
        }
    }

    // Filter doesn't want to build, I have no idea why.
    // Let's move to augmenting the collector first
    /*
    #[tokio::test]
    async fn filter_works() {
        let result = (0..100)
            .into_deluge()
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
