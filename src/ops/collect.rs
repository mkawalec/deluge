use crate::deluge::Deluge;
use core::pin::Pin;
use futures::stream::Stream;
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::boxed::Box;
use std::collections::{BTreeMap, HashMap};
use std::default::Default;
use std::future::Future;
use std::num::NonZeroUsize;

type DelOutput<'a, Del> = dyn Future<Output = Option<<Del as Deluge>::Item>> + 'a;

#[pin_project]
pub struct Collect<'a, Del, C>
where
    Del: Deluge,
{
    deluge: Del,
    deluge_exhausted: bool,

    insert_idx: usize,
    concurrency: Option<NonZeroUsize>,

    polled_futures: HashMap<usize, Pin<Box<DelOutput<'a, Del>>>>,
    completed_items: BTreeMap<usize, Option<Del::Item>>,

    last_provided_idx: Option<usize>,
    collection: Option<C>,
}

impl<'a, Del: Deluge, C: Default> Collect<'a, Del, C> {
    pub(crate) fn new(deluge: Del, concurrency: impl Into<Option<usize>>) -> Self {
        Self {
            deluge: deluge,
            deluge_exhausted: false,

            insert_idx: 0,
            concurrency: concurrency.into().and_then(NonZeroUsize::new),

            polled_futures: HashMap::new(),
            completed_items: BTreeMap::new(),
            last_provided_idx: None,

            collection: Some(C::default()),
        }
    }
}

impl<'a, Del, C> Stream for Collect<'a, Del, C>
where
    Del: Deluge + 'a,
{
    type Item = Del::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        loop {
            while !*this.deluge_exhausted {
                let concurrency_limit = if let Some(limit) = this.concurrency {
                    limit.get()
                } else {
                    usize::MAX
                };

                if this.polled_futures.len() < concurrency_limit {
                    // We **know** that a reference to deluge lives for 'a,
                    // so it should be safe to force the dilesystem to acknowledge that
                    let deluge: &'a Del = unsafe { std::mem::transmute(&mut *this.deluge) };
                    let next = deluge.next();
                    if let Some(future) = next {
                        this.polled_futures
                            .insert(*this.insert_idx, Box::pin(future));
                        *this.insert_idx += 1;
                    } else {
                        *this.deluge_exhausted = true;
                    }
                } else {
                    // We would exceed the concurrency limit by loading more elements
                    break;
                }
            }

            // Drive all available futures
            if !this.polled_futures.is_empty() {
                this.polled_futures.retain(|idx, fut| {
                    match Pin::new(fut).poll(cx) {
                        Poll::Ready(v) => {
                            // Drop the items that should be ignored on the floor.
                            // The indexes in the `completed_items` map don't need
                            // to be contignous, it's enough for them to be monotonic
                            this.completed_items.insert(*idx, v);
                            false
                        }
                        _ => true,
                    }
                });
            }

            // If all the polled futures were immediately evaluated
            // and we can still evaluate something more, load more items to evaluate
            //
            // Otherwise if these features need more time to evaluate
            // they will re-enter self::poll through the waker
            if !this.polled_futures.is_empty() || *this.deluge_exhausted {
                break;
            }
        }

        loop {
            let idx_to_provide = this.last_provided_idx.map(|x| x + 1).unwrap_or(0);
            if let Some(val) = this.completed_items.get_mut(&idx_to_provide) {
                *this.last_provided_idx = Some(idx_to_provide);
                // If we saw an item, always instruct the waker to try again
                cx.waker().wake_by_ref();

                if val.is_some() {
                    return Poll::Ready(Some(val.take().unwrap()));
                }
            } else if this.polled_futures.is_empty() {
                // Our input has been exhausted, nothing more to see
                return Poll::Ready(None);
            } else {
                return Poll::Pending;
            }
        }
    }
}

impl<'a, Del, C> Future for Collect<'a, Del, C>
where
    Del: Deluge + 'a,
    C: Default + Extend<Del::Item>,
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<C> {
        match self.as_mut().poll_next(cx) {
            Poll::Ready(Some(v)) => {
                self.collection.as_mut().unwrap().extend_one(v);
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(self.collection.take().unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
}
