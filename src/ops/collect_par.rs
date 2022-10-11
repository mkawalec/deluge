use crate::deluge::Deluge;
use core::pin::Pin;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::task::{Context, Poll};
use futures::Stream;
use pin_project::pin_project;
use std::boxed::Box;
use std::collections::BTreeMap;
use std::default::Default;
use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[cfg(feature = "tokio")]
type Mutex<T> = tokio::sync::Mutex<T>;
#[cfg(feature = "tokio")]
use tokio::sync::mpsc;
#[cfg(feature = "tokio")]
type SendError<T> = tokio::sync::mpsc::error::SendError<T>;

#[cfg(feature = "async-std")]
type Mutex<T> = async_std::sync::Mutex<T>;
#[cfg(feature = "async-std")]
use async_std::channel as mpsc;
#[cfg(feature = "async-std")]
type SendError<T> = mpsc::SendError<T>;

type OutstandingFutures<'a, Del> =
    Arc<Mutex<BTreeMap<usize, Pin<Box<<Del as Deluge>::Output<'a>>>>>>;
type CompletedItem<Del> = (usize, Option<<Del as Deluge>::Item>);
type Worker<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

#[pin_project]
pub struct CollectPar<'a, Del, C>
where
    Del: Deluge + 'a,
{
    deluge: Del,
    deluge_exhausted: bool,
    worker_count: usize,
    worker_concurrency: Option<NonZeroUsize>,

    workers: Vec<Worker<'a>>,
    outstanding_futures: Option<OutstandingFutures<'a, Del>>,
    completed_items: BTreeMap<usize, Option<Del::Item>>,
    #[allow(clippy::type_complexity)]
    completed_channel: Option<(
        mpsc::Sender<CompletedItem<Del>>,
        mpsc::Receiver<CompletedItem<Del>>,
    )>,

    last_provided_idx: Option<usize>,
    collection: Option<C>,
}

impl<'a, Del: Deluge, C: Default> CollectPar<'a, Del, C> {
    pub(crate) fn new(
        deluge: Del,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>,
    ) -> Self {
        let worker_count = worker_count.into().unwrap_or_else(num_cpus::get);
        let mut workers = Vec::new();
        workers.reserve_exact(worker_count);

        Self {
            deluge: deluge,
            deluge_exhausted: false,
            worker_count,
            worker_concurrency: worker_concurrency.into().and_then(NonZeroUsize::new),

            workers,
            outstanding_futures: None,
            completed_items: BTreeMap::new(),
            // Only spawn the channel after we know how many items we have to eval
            completed_channel: None,

            last_provided_idx: None,
            collection: Some(C::default()),
        }
    }
}

// Approach
// 1. Central mutexed container for jobs to be stolen from
// 2. Each worker starts with worker_concurrency futures
//    and steals from the central place as needed

fn create_worker<'a, Del: Deluge + 'a>(
    outstanding_futures: OutstandingFutures<'a, Del>,
    completed_channel: mpsc::Sender<CompletedItem<Del>>,
    concurrency: NonZeroUsize,
) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
    Box::pin(async move {
        let mut evaluated_futures = FuturesUnordered::new();

        let run_worker = make_fn_once(|| async {
            let mut more_work_available = true;
            loop {
                // Load up on work if we aren't full and we expect more work to show up
                if evaluated_futures.len() < concurrency.get() && more_work_available {
                    let mut outstanding_futures = outstanding_futures.lock().await;
                    while evaluated_futures.len() < concurrency.get()
                        && !outstanding_futures.is_empty()
                    {
                        if let Some((idx, fut)) = outstanding_futures.pop_first() {
                            evaluated_futures.push(IndexedFuture::new(idx, fut));
                        }
                    }

                    if outstanding_futures.is_empty() {
                        more_work_available = false;
                    }
                }

                if let Some(result) = evaluated_futures.next().await {
                    completed_channel.send(result).await?;
                } else {
                    // If there is no more results to fetch, double check if nothing
                    // was returned into `outstanding_futures` by another crashing worker.
                    // If it is still empty, terminate.
                    let outstanding_futures = outstanding_futures.lock().await;
                    if !outstanding_futures.is_empty() {
                        more_work_available = true;
                        continue;
                    } else {
                        break;
                    }
                }
            }

            Ok::<(), SendError<CompletedItem<Del>>>(())
        });

        if let Err(_e) = run_worker().await {
            // Return unevaluated futures into the pile
            let mut outstanding_futures = outstanding_futures.lock().await;
            evaluated_futures.into_iter().for_each(|fut| {
                outstanding_futures.insert(fut.index(), fut.into_future());
            });
        }
    })
}

impl<'a, Del, C> Stream for CollectPar<'a, Del, C>
where
    Del: Deluge + 'a,
    C: Default + Extend<Del::Item>,
{
    type Item = Del::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        if !*this.deluge_exhausted {
            let mut outstanding_futures = BTreeMap::new();
            let mut insert_idx = 0;

            // Load up all the futures
            loop {
                // We **know** that a reference to deluge lives for 'a,
                // so it should be safe to force the dilesystem to acknowledge that
                let deluge: &'a Del = unsafe { std::mem::transmute(&mut *this.deluge) };

                let next = deluge.next();
                if let Some(future) = next {
                    outstanding_futures.insert(insert_idx, Box::pin(future));
                    insert_idx += 1;
                } else {
                    *this.deluge_exhausted = true;
                    break;
                }
            }

            let total_futures = outstanding_futures.len();

            #[cfg(feature = "tokio")]
            {
                *this.completed_channel = Some(mpsc::channel(total_futures));
            }
            #[cfg(feature = "async-std")]
            {
                *this.completed_channel = Some(mpsc::bounded(total_futures));
            }

            *this.outstanding_futures = Some(Arc::new(Mutex::new(outstanding_futures)));

            // Spawn workers
            if this.workers.is_empty() {
                let worker_concurrency = this
                    .worker_concurrency
                    .or_else(|| NonZeroUsize::new(total_futures / *this.worker_count))
                    .unwrap_or_else(|| unsafe { NonZeroUsize::new_unchecked(1) });

                for _ in 0..(*this.worker_count) {
                    this.workers.push(create_worker::<'a, Del>(
                        this.outstanding_futures.as_ref().unwrap().clone(),
                        this.completed_channel.as_ref().unwrap().0.clone(),
                        worker_concurrency,
                    ));
                }
            }
        }

        // Drive the workers
        this.workers
            .retain_mut(|worker| !matches!(Pin::new(worker).poll(cx), Poll::Ready(_)));

        // Drain the compelted channel
        while let Ok((idx, v)) = this.completed_channel.as_mut().unwrap().1.try_recv() {
            this.completed_items.insert(idx, v);
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
            } else if this.workers.is_empty() {
                // Our input has been exhausted, nothing more to see
                return Poll::Ready(None);
            } else {
                return Poll::Pending;
            }
        }
    }
}

impl<'a, Del, C> Future for CollectPar<'a, Del, C>
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

// A helper type allowing us to have a future with a synchronously available index on it
#[pin_project]
pub struct IndexedFuture<Fut> {
    future: Pin<Box<Fut>>,
    index: usize,
}

impl<Fut: Future> IndexedFuture<Fut> {
    fn new(index: usize, future: Pin<Box<Fut>>) -> Self {
        IndexedFuture { future, index }
    }

    fn index(&self) -> usize {
        self.index
    }

    fn into_future(self) -> Pin<Box<Fut>> {
        self.future
    }
}

impl<Fut> Future for IndexedFuture<Fut>
where
    Fut: Future,
{
    type Output = (usize, Fut::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        match Pin::new(this.future).poll(cx) {
            Poll::Ready(result) => Poll::Ready((*this.index, result)),
            _ => Poll::Pending,
        }
    }
}

// Useful for when we need to prove to the compiler that our worker body is a `FnOnce`
fn make_fn_once<T, F: FnOnce() -> T>(f: F) -> F {
    f
}
