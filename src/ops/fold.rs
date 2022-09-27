use crate::deluge::Deluge;
use core::pin::Pin;
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::boxed::Box;
use std::future::Future;
use std::marker::PhantomData;

use super::collect::Collect;

#[pin_project]
pub struct Fold<'a, Del, Acc, F, Fut>
where
    Del: Deluge<'a>,
    F: FnMut(Acc, Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = Acc> + Send + 'a,
{
    deluge: Option<Del>,
    #[allow(clippy::type_complexity)]
    collect_future: Option<Pin<Box<dyn Future<Output = Vec<Del::Item>> + 'a>>>,
    collected_result: Option<std::vec::IntoIter<Del::Item>>,
    current_el_future: Option<Fut>,

    concurrency: Option<usize>,

    acc: Option<Acc>,
    f: F,
    _acc_fut: PhantomData<Fut>,
}

impl<'a, Del, Acc, F, Fut> Fold<'a, Del, Acc, F, Fut>
where
    Del: Deluge<'a>,
    F: FnMut(Acc, Del::Item) -> Fut + Send + 'a,
    Fut: Future<Output = Acc> + Send + 'a,
{
    pub(crate) fn new(deluge: Del, concurrency: impl Into<Option<usize>>, acc: Acc, f: F) -> Self {
        Self {
            deluge: Some(deluge),
            collect_future: None,
            collected_result: None,
            current_el_future: None,

            concurrency: concurrency.into(),

            acc: Some(acc),
            f,
            _acc_fut: PhantomData,
        }
    }
}

impl<'a, InputDel, Acc, F, Fut> Future for Fold<'a, InputDel, Acc, F, Fut>
where
    InputDel: Deluge<'a> + 'a,
    F: FnMut(Acc, InputDel::Item) -> Fut + Send + 'a,
    Fut: Future<Output = Acc> + Send + 'a,
{
    type Output = Acc;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        if this.deluge.is_some() && this.collect_future.is_none() {
            let collect_future = Collect::new(this.deluge.take().unwrap(), *this.concurrency);
            *this.collect_future = Some(Box::pin(collect_future));
        }

        if let Some(collect_future) = this.collect_future.as_mut() {
            match Pin::new(collect_future).poll(cx) {
                Poll::Ready(v) => {
                    *this.collected_result = Some(v.into_iter());
                    *this.collect_future = None;
                }
                _ => return Poll::Pending,
            }
        }

        loop {
            if let Some(collected_result) = this.collected_result.as_mut() && this.current_el_future.is_none() {
                if let Some(el) = collected_result.next() {
                    *this.current_el_future = Some((this.f)(this.acc.take().unwrap(), el));
                } else {
                    *this.collected_result = None;
                    break;
                }
            }

            if let Some(current_el_future) = this.current_el_future.as_mut() {
                // We're manually pin-projecting since we need to project into an Option
                match unsafe { Pin::new_unchecked(current_el_future) }.poll(cx) {
                    Poll::Ready(v) => {
                        *this.current_el_future = None;
                        *this.acc = Some(v);
                    }
                    _ => return Poll::Pending,
                }
            };
        }

        Poll::Ready(this.acc.take().unwrap())
    }
}
