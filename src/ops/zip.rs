use crate::deluge::Deluge;
use super::collect::Collect;
use std::collections::{VecDeque, HashMap, BTreeMap};
use std::future::Future;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::sync::Arc;
use std::task::{Waker, Context, Poll};
use pin_project::pin_project;
use tokio::sync::OwnedMutexGuard;
use std::pin::Pin;
use futures::stream::{StreamExt, Next, StreamFuture};
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

struct IndexableStream<'a, S: Stream + 'a> {
    wakers: std::sync::Mutex<BTreeMap<usize, Waker>>,
    inner: Arc<Mutex<InnerIndexableStream<S>>>,
    _lifetime: PhantomData<&'a ()>
}

struct InnerIndexableStream<S: Stream> {
    stream: Option<Pin<Box<S>>>,
    items: HashMap<usize, S::Item>,
    next_promise: Option<StreamFuture<Pin<Box<S>>>>,
    current_index: usize,
    exhausted: bool,
}

impl<'a, S: Stream + 'a> IndexableStream<'a, S> {
    fn new(stream: S) -> Self {
        Self {
            wakers: std::sync::Mutex::new(BTreeMap::new()),
            inner: Arc::new(Mutex::new(InnerIndexableStream {
                stream: Some(Box::pin(stream)),
                items: HashMap::new(),
                next_promise: None,
                current_index: 0,
                exhausted: false,
            })),
            _lifetime: PhantomData,
        }
    }

    fn get_nth(self: Arc<Self>, idx: usize) -> GetNthElement<'a, S> {
        GetNthElement {
            indexable: self,
            idx,
            mutex_guard: None,
        }
    }
}

#[pin_project]
struct GetNthElement<'a, S: Stream + 'a> {
    indexable: Arc<IndexableStream<'a, S>>,
    idx: usize,
    mutex_guard: Option<Pin<Box<dyn Future<Output = OwnedMutexGuard<InnerIndexableStream<S>>> +'a>>>,
}

impl<'a, S: Stream + 'a> Future for GetNthElement<'a, S> {
    type Output = Option<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        let mut wakers = this.indexable.wakers.lock().unwrap();
        wakers.insert(*this.idx, cx.waker().clone());

        if this.mutex_guard.is_none() {
            *this.mutex_guard = Some(Box::pin(this.indexable.inner.clone().lock_owned()));
        }

        let mut mutex_guard = this.mutex_guard.take().unwrap();
        match Pin::new(&mut mutex_guard).poll(cx) {
            Poll::Pending => {
                *this.mutex_guard = Some(mutex_guard);
                Poll::Pending
            },
            Poll::Ready(mut inner) => {
                if inner.exhausted {
                    wakers.remove(&*this.idx);
                    if let Some(v) = inner.items.remove(this.idx) {
                        return Poll::Ready(Some(v));
                    } else {
                        return Poll::Ready(None);
                    }
                }

                if let Some(el) = inner.items.remove(this.idx) {
                    wakers.remove(&*this.idx);
                    return Poll::Ready(Some(el));
                }

                let is_first_entry = if let Some(first_entry) = wakers.first_entry() {
                    *first_entry.key() == *this.idx
                } else {
                    false
                };

                if is_first_entry {
                    if inner.next_promise.is_none() {
                        inner.next_promise = Some(inner.stream.take().unwrap().into_future());
                    }
                    let mut next_promise = inner.next_promise.take().unwrap();

                    match Pin::new(&mut next_promise).poll(cx) {
                        Poll::Pending => {
                            inner.next_promise = Some(next_promise);
                            return Poll::Pending;
                        },
                        Poll::Ready((None, _)) => {
                            inner.exhausted = true;
                            wakers.values().for_each(|waker| waker.wake_by_ref());
                            
                            return Poll::Ready(None);
                        },
                        Poll::Ready((Some(v), stream)) => {
                            inner.stream = Some(stream);

                            if *this.idx == inner.current_index {
                                wakers.remove(this.idx);

                                inner.current_index += 1;
                                return Poll::Ready(Some(v));
                            } else {
                                let current_index = inner.current_index;
                                inner.items.insert(current_index, v);
                                if let Some(entry) = wakers.first_entry() {
                                    entry.get().wake_by_ref();
                                }

                                inner.current_index += 1;
                                return Poll::Pending;
                            }
                        }
                    }
                } else {
                    return Poll::Pending;
                }
            }
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
                let mut this = Pin::new(self).project_ref();

                let (first_el, second_el) = {
                    let mut streams = this.streams.lock().await;
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