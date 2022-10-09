use crate::deluge::Deluge;
use super::collect::Collect;
use std::collections::{VecDeque, HashMap};
use std::future::Future;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::sync::Arc;
use std::task::{Waker, Context, Poll};
use pin_project::pin_project;
use std::pin::Pin;
use futures::stream::StreamExt;
use futures::{join, Stream};

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
    first: Collect<'a, PreloadedFutures<'a, Del1>, ()>,
    second: Collect<'a, PreloadedFutures<'a, Del2>, ()>,
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
                first: Collect::new(preloaded1, concurrency.clone()),
                second: Collect::new(preloaded2, concurrency.clone()),
            }),

            provided_elems: RefCell::new(0),
            elems_to_provide,
        }
    }
}

struct IndexableStream<S: Stream> {
    stream: S,
    items: HashMap<usize, Option<S::Item>>,
    wakers: std::sync::Mutex<HashMap<usize, Waker>>,
}

impl<S: Stream> IndexableStream<S> {
    fn new(stream: S) -> Self {
        Self {
            stream,
            items: HashMap::new(),
            wakers: std::sync::Mutex::new(HashMap::new())
        }
    }

    fn get_nth(self: Arc<Self>, element: usize) -> GetNthElement<S> {
        GetNthElement {
            indexable: self,
            element,
        }
    }
}

#[pin_project]
struct GetNthElement<S: Stream> {
    indexable: Arc<IndexableStream<S>>,
    element: usize,
}

impl<S: Stream> Future for GetNthElement<S> {
    type Output = Option<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        {
            let mut wakers = this.indexable.wakers.lock().unwrap();
            wakers.insert(*this.element, cx.waker().clone());
        }

        // Figure out which element we are inserting,
        // insert it and wake the correct waker.
        // When the stream is exhausted, wake all the remaining wakers

        todo!()
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
            *provided_elems += 1;
            Some(async move {
                let mut this = Pin::new(self).project_ref();

                let (first_el, second_el) = {
                    let mut streams = this.streams.lock().await;
                    (streams.first.next().await, streams.second.next().await)
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

struct PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    storage: RefCell<VecDeque<Pin<Box<Del::Output<'a>>>>>,
    deluge: Del,
}

impl<'a, Del> PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    fn new(deluge: Del) -> Self {
        let mut storage = VecDeque::new();
        let deluge_borrow: &'a Del = unsafe {
            std::mem::transmute(&deluge)
        };
        loop {
            if let Some(v) = deluge_borrow.next() {
                storage.push_back(Box::pin(v));
            } else {
                break;
            }
        }
        Self {
            storage: RefCell::new(storage),
            deluge,
        }
    }

    fn len(&self) -> usize {
        self.storage.borrow().len()
    }
}

impl<'a, Del> Deluge for PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    type Item = Del::Item;
    type Output<'x> where Self: 'x = impl Future<Output = Option<Self::Item>> + 'x;

    fn next<'x>(&'x self) -> Option<Self::Output<'x>>
    {
        let next_item = {
            self.storage.borrow_mut().pop_front()
        };

        next_item.map(|el| async move {
            el.await
        })
    }
}