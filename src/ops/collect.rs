use core::pin::Pin;
use std::future::Future;
use std::boxed::Box;
use futures::task::{Context, Poll};
use std::marker::PhantomData;
use pin_project::pin_project;
use std::collections::HashMap;
use std::default::Default;
use std::num::NonZeroUsize;
use crate::deluge::Deluge;

#[pin_project]
pub struct Collect<'a, Del, C>
where Del: Deluge<'a>,
{
    deluge: Option<Del>,
    insert_idx: usize,
    concurrency: Option<NonZeroUsize>,

    polled_futures: HashMap<usize, Pin<Box<Del::Output>>>,
    completed_items: HashMap<usize, Del::Item>,
    _collection: PhantomData<C>,
}

impl<'a, Del: Deluge<'a>, C: Default> Collect<'a, Del, C> 
{
    pub(crate) fn new(deluge: Del, concurrency: impl Into<Option<usize>>) -> Self {
        Self {
            deluge: Some(deluge),
            insert_idx: 0,
            concurrency: concurrency.into().and_then(NonZeroUsize::new),

            polled_futures: HashMap::new(),
            completed_items: HashMap::new(),
            _collection: PhantomData,
        }
    }
}

impl<'a, Del, C> Future for Collect<'a, Del, C>
where
    Del: Deluge<'a> + 'a,
    C: Default + Extend<Del::Item>
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<C> {
        let this = self.as_mut().project();

        loop {
            while this.deluge.is_some() {
                let concurrency_limit = if let Some(limit) = this.concurrency {
                    limit.get()
                } else {
                    usize::MAX
                };

                // Funky stuff, extending the lifetime of the inner future
                let deluge: &'a mut Del = unsafe {
                    std::mem::transmute(this.deluge.as_mut().unwrap())
                };

                if this.polled_futures.len() < concurrency_limit {
                    let next = deluge.next();
                    if let Some(future) = next {
                        this.polled_futures.insert(*this.insert_idx, Box::pin(future));
                        *this.insert_idx += 1;
                    } else {
                        // Nothing more to pull from the deluge, we can proceed to poll the futures
                        *this.deluge = None;
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
                            if let Some(v) = v {
                                this.completed_items.insert(*idx, v);
                            }
                            false
                        },
                        _ => true
                    }
                });
            } 

            // If all the polled futures were immediately evaluated
            // and we can still evaluate something more, load more items to evaluate
            //
            // Otherwise if these features need more time to evaluate
            // they will re-enter self::poll through the waker
            if !this.polled_futures.is_empty() || this.deluge.is_none() {
                break;
            }
        }

        // If all futures have finished, collect the results into the output
        // TODO: Use an already ordered map to not require the sorting step here
        if this.polled_futures.is_empty() && this.deluge.is_none() {
            let mut collected: Vec<(usize, Del::Item)> = this.completed_items.drain()
                .collect();
            collected.sort_by_key(|(idx, _)| *idx);

            let items = collected
                .into_iter()
                .map(|(_, el)| el);

            let mut collection: C = Default::default();
            collection.extend(items);
            Poll::Ready(collection)
        } else {
            Poll::Pending
        }
    }
}
