use crate::deluge::Deluge;
use crate::helpers::indexable_stream::*;
use crate::helpers::preloaded_futures::*;
use super::collect::Collect;

use std::future::Future;
use std::cell::RefCell;
use std::sync::Arc;
use pin_project::pin_project;
use std::pin::Pin;

#[cfg(feature = "tokio")]
type Mutex<T> = tokio::sync::Mutex<T>;
#[cfg(feature = "async-std")]
type Mutex<T> = async_std::sync::Mutex<T>;


#[pin_project]
pub struct Zip<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    streams: Mutex<Streams<'a, Del1, Del2>>,

    provided_elems: RefCell<usize>,
    elems_to_provide: usize,
}

struct Streams<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    first: Arc<IndexableStream<'a, Collect<'a, PreloadedFutures<'a, Del1>, ()>>>,
    second: Arc<IndexableStream<'a, Collect<'a, PreloadedFutures<'a, Del2>, ()>>>,
}

impl<'a, Del1, Del2> Zip<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    pub(crate) fn new(
        first: Del1,
        second: Del2,
        concurrency: impl Into<Option<usize>>,
    ) -> Self {
        let concurrency = concurrency.into();

        // Preload the futures from each
        let preloaded1 = PreloadedFutures::new(first);
        let preloaded2 = PreloadedFutures::new(second);

        let elems_to_provide = std::cmp::min(preloaded1.len(), preloaded2.len());

        Self {
            streams: Mutex::new(Streams {
                first: Arc::new(IndexableStream::new(Collect::new(preloaded1, concurrency.clone()))),
                second: Arc::new(IndexableStream::new(Collect::new(preloaded2, concurrency.clone()))),
            }),

            provided_elems: RefCell::new(0),
            elems_to_provide,
        }
    }
}


impl<'a, Del1, Del2> Deluge for Zip<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    type Item = (Del1::Item, Del2::Item);
    type Output<'x> where Self: 'x = impl Future<Output = Option<Self::Item>> + 'x;

    fn next<'x>(&'x self) -> Option<Self::Output<'x>>
    {
        let mut provided_elems = self.provided_elems.borrow_mut();
        if *provided_elems >= self.elems_to_provide {
            None
        } else {
            let current_index = *provided_elems;

            *provided_elems += 1;
            Some(async move {
                let this = Pin::new(self).project_ref();

                let (first_el, second_el) = {
                    let streams = this.streams.lock().await;
                    (streams.first.clone().get_nth(current_index).await, streams.second.clone().get_nth(current_index).await)
                };
                
                if first_el.is_none() || second_el.is_none() {
                    None
                } else {
                    Some((first_el.unwrap(), second_el.unwrap()))
                }
            })
        }
    }
}