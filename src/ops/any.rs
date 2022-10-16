use pin_project::pin_project;

use super::collect::Collect;
use super::map::Map;
use crate::deluge::Deluge;
use futures::task::{Context, Poll};
use futures::Stream;
use std::future::Future;
use std::pin::Pin;

#[pin_project]
pub struct Any<'a, Del, Fut, F>
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    #[pin]
    stream: Collect<'a, Map<Del, F>, ()>,
}

impl<'a, Del, Fut, F> Any<'a, Del, Fut, F>
where
    Del: Deluge + 'a,
    F: Fn(Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = bool> + Send,
{
    pub(crate) fn new(deluge: Del, concurrency: impl Into<Option<usize>>, f: F) -> Self {
        Self {
            stream: Collect::new(Map::new(deluge, f), concurrency),
        }
    }
}

impl<'a, Del, Fut, F> Future for Any<'a, Del, Fut, F>
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
            }
            Poll::Ready(None) => Poll::Ready(false),
            Poll::Pending => Poll::Pending,
        }
    }
}
