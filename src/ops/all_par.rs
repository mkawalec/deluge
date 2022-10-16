use pin_project::pin_project;

use super::collect_par::CollectPar;
use super::map::Map;
use crate::deluge::Deluge;
use futures::task::{Context, Poll};
use futures::Stream;
use std::future::Future;
use std::pin::Pin;

#[pin_project]
pub struct AllPar<'a, Del, Fut, F>
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    #[pin]
    stream: CollectPar<'a, Map<Del, F>, ()>,
}

impl<'a, Del, Fut, F> AllPar<'a, Del, Fut, F>
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    pub(crate) fn new(
        deluge: Del,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
        f: F,
    ) -> Self {
        Self {
            stream: CollectPar::new(Map::new(deluge, f), worker_count, worker_concurrency),
        }
    }
}

impl<'a, Del, Fut, F> Future for AllPar<'a, Del, Fut, F>
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(true)) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            Poll::Ready(Some(false)) => {
                Poll::Ready(false)
            }
            Poll::Ready(None) => Poll::Ready(true),
            Poll::Pending => Poll::Pending,
        }
    }
}
