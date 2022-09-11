use core::pin::Pin;
use std::future::Future;
use std::boxed::Box;
use futures::task::{Context, Poll};
use std::marker::PhantomData;
use pin_project::pin_project;
use std::collections::HashMap;
use std::default::Default;
use crate::deluge::Deluge;

#[pin_project]
pub struct Collect<'a, Del, C>
where Del: Deluge<'a>,
{
    deluge: Option<Del>,
    insert_idx: usize,

    polled_futures: HashMap<usize, Pin<Box<Del::Output>>>,
    completed_futures: HashMap<usize, Del::Item>,
    _collection: PhantomData<C>,
}

impl<'a, Del: Deluge<'a>, C: Default> Collect<'a, Del, C> 
{
    pub(crate) fn new(deluge: Del) -> Self {
        Self {
            deluge: Some(deluge),
            insert_idx: 0,

            polled_futures: HashMap::new(),
            completed_futures: HashMap::new(),
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
        while this.deluge.is_some() {
            // Funky stuff, extending the lifetime of the inner future
            let deluge: &'a mut Del = unsafe {
                std::mem::transmute(this.deluge.as_mut().unwrap())
            };
            if let Some(future) = deluge.next() {
                this.polled_futures.insert(*this.insert_idx, Box::pin(future));
                *this.insert_idx += 1;
            } else {
                // Nothing more to pull from the deluge, we can proceed to poll the futures
                *this.deluge = None;
            }
        }

        // Drive all the futures, but don't wait for a result
        if !this.polled_futures.is_empty() {
            this.polled_futures.retain(|idx, fut| {
                match Pin::new(fut).poll(cx) {
                    Poll::Ready(v) => {
                        this.completed_futures.insert(*idx, v);
                        false
                    },
                    _ => true
                }
            });
        } 
        
        if this.polled_futures.is_empty() {
            let mut collected: Vec<(usize, Del::Item)> = this.completed_futures.drain()
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
