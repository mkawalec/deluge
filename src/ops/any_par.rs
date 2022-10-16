use pin_project::pin_project;

use crate::deluge::Deluge;
use super::map::Map;
use super::collect_par::CollectPar;
use std::future::Future;
use futures::task::{Context, Poll};
use futures::Stream;
use std::pin::Pin;

#[pin_project]
pub struct AnyPar<'a, Del, Fut, F>
where
    Del: Deluge +'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    #[pin]
    stream: CollectPar<'a, Map<Del, F>, ()>,
}

impl<'a, Del, Fut, F> AnyPar<'a, Del, Fut, F> 
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    pub(crate) fn new(
        deluge: Del,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
        f: F
    ) -> Self
    {
        Self {
            stream: CollectPar::new(Map::new(deluge, f), worker_count, worker_concurrency),
        }
    }
}

impl<'a, Del, Fut, F> Future for AnyPar<'a, Del, Fut, F>
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(true)) => Poll::Ready(true),
            Poll::Ready(Some(false)) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            Poll::Ready(None) => Poll::Ready(false),
            Poll::Pending => Poll::Pending,
        }
    }
}
// Return a future, which returns if any element evaluates to true