use futures::stream::{StreamExt, StreamFuture};
use futures::Stream;
use pin_project::pin_project;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

#[cfg(feature = "tokio")]
type Mutex<T> = tokio::sync::Mutex<T>;
#[cfg(feature = "async-std")]
type Mutex<T> = async_std::sync::Mutex<T>;

#[cfg(feature = "tokio")]
type OwnedMutexGuard<T> = tokio::sync::OwnedMutexGuard<T>;
#[cfg(feature = "async-std")]
type OwnedMutexGuard<T> = async_std::sync::MutexGuardArc<T>;

/// Stream wrapper allowing it's users to subscribe to an element at a specific index
/// through `IndexableStream::get_nth`.
pub struct IndexableStream<'a, S: Stream + 'a> {
    wakers: std::sync::Mutex<BTreeMap<usize, Waker>>,
    inner: Arc<Mutex<InnerIndexableStream<S>>>,
    _lifetime: PhantomData<&'a ()>,
}

struct InnerIndexableStream<S: Stream> {
    stream: Option<Pin<Box<S>>>,
    items: HashMap<usize, S::Item>,
    next_promise: Option<StreamFuture<Pin<Box<S>>>>,
    current_index: usize,
    exhausted: bool,
}

impl<'a, S: Stream + 'a> IndexableStream<'a, S> {
    pub fn new(stream: S) -> Self {
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

    pub fn get_nth(self: Arc<Self>, idx: usize) -> GetNthElement<'a, S> {
        GetNthElement {
            indexable: self,
            idx,
            mutex_guard: None,
        }
    }
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[pin_project]
pub struct GetNthElement<'a, S: Stream + 'a> {
    indexable: Arc<IndexableStream<'a, S>>,
    idx: usize,
    mutex_guard: Option<BoxFuture<'a, OwnedMutexGuard<InnerIndexableStream<S>>>>,
}

impl<'a, S: Stream + 'a> Future for GetNthElement<'a, S> {
    type Output = Option<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        {
            let mut wakers = this.indexable.wakers.lock().unwrap();
            wakers.insert(*this.idx, cx.waker().clone());
        }

        if this.mutex_guard.is_none() {
            #[cfg(feature = "tokio")]
            {
                *this.mutex_guard = Some(Box::pin(this.indexable.inner.clone().lock_owned()));
            }
            #[cfg(feature = "async-std")]
            {
                *this.mutex_guard = Some(Box::pin(this.indexable.inner.lock_arc()));
            }
        }

        let mut mutex_guard = this.mutex_guard.take().unwrap();
        match Pin::new(&mut mutex_guard).poll(cx) {
            Poll::Pending => {
                *this.mutex_guard = Some(mutex_guard);
                Poll::Pending
            }
            Poll::Ready(mut inner) => {
                if inner.exhausted {
                    let mut wakers = this.indexable.wakers.lock().unwrap();
                    wakers.remove(&*this.idx);
                    if let Some(v) = inner.items.remove(this.idx) {
                        return Poll::Ready(Some(v));
                    } else {
                        return Poll::Ready(None);
                    }
                }

                if let Some(el) = inner.items.remove(this.idx) {
                    let mut wakers = this.indexable.wakers.lock().unwrap();
                    wakers.remove(&*this.idx);
                    return Poll::Ready(Some(el));
                }

                let is_first_entry = {
                    let mut wakers = this.indexable.wakers.lock().unwrap();
                    if let Some(first_entry) = wakers.first_entry() {
                        *first_entry.key() == *this.idx
                    } else {
                        false
                    }
                };

                if is_first_entry {
                    if inner.next_promise.is_none() {
                        inner.next_promise = Some(inner.stream.take().unwrap().into_future());
                    }
                    let mut next_promise = inner.next_promise.take().unwrap();

                    match Pin::new(&mut next_promise).poll(cx) {
                        Poll::Pending => {
                            inner.next_promise = Some(next_promise);
                            Poll::Pending
                        }
                        Poll::Ready((None, _)) => {
                            inner.exhausted = true;
                            let wakers = this.indexable.wakers.lock().unwrap();
                            wakers.values().for_each(|waker| waker.wake_by_ref());

                            Poll::Ready(None)
                        }
                        Poll::Ready((Some(v), stream)) => {
                            inner.stream = Some(stream);

                            if *this.idx == inner.current_index {
                                let mut wakers = this.indexable.wakers.lock().unwrap();
                                wakers.remove(this.idx);

                                inner.current_index += 1;
                                Poll::Ready(Some(v))
                            } else {
                                let current_index = inner.current_index;
                                inner.items.insert(current_index, v);

                                let mut wakers = this.indexable.wakers.lock().unwrap();
                                if let Some(entry) = wakers.first_entry() {
                                    entry.get().wake_by_ref();
                                }

                                inner.current_index += 1;
                                Poll::Pending
                            }
                        }
                    }
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
