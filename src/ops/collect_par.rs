use core::pin::Pin;
use std::future::Future;
use std::boxed::Box;
use futures::task::{Context, Poll};
use futures::poll;
use std::marker::PhantomData;
use pin_project::pin_project;
use std::collections::{HashMap, BTreeMap};
use std::sync::atomic::AtomicUsize;
use std::collections::HashSet;
use std::default::Default;
use std::num::NonZeroUsize;
use std::sync::Arc;
use crate::deluge::Deluge;

#[cfg(feature = "tokio")]
type Mutex<T> = tokio::sync::Mutex<T>;
#[cfg(feature = "tokio")]
use tokio::sync::mpsc;
#[cfg(feature = "tokio")]
type SendError<T> = tokio::sync::mpsc::error::SendError<T>;

#[cfg(feature = "async-std")]
type Mutex<T> = async_std::sync::Mutex<T>;
#[cfg(feature = "async-std")]
use async_std::sync as mpsc;
#[cfg(feature = "async-std")]
type SendError<T> = PhantomData<T>;


type OutstandingFutures<'a, Del> = Arc<Mutex<BTreeMap<usize, Pin<Box<<Del as Deluge<'a>>::Output>>>>>;
type CompletedItem<'a, Del> = (usize, Option<<Del as Deluge<'a>>::Item>);
type Worker<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

#[pin_project]
pub struct CollectPar<'a, Del, C>
where Del: Deluge<'a>,
{
    deluge: Option<Del>,
    worker_count: usize,
    worker_concurrency: Option<NonZeroUsize>,

    workers: Vec<Worker<'a>>,
    outstanding_futures: Option<OutstandingFutures<'a, Del>>,
    completed_items: BTreeMap<usize, Del::Item>,
    completed_channel: (mpsc::Sender<CompletedItem<'a, Del>>, mpsc::Receiver<CompletedItem<'a, Del>>),
    _collection: PhantomData<C>,
}

impl<'a, Del: Deluge<'a>, C: Default> CollectPar<'a, Del, C> 
{
    pub(crate) fn new(
        deluge: Del,
        worker_count: impl Into<Option<usize>>,
        worker_concurrency: impl Into<Option<usize>>
    ) -> Self
    {
        let worker_count = worker_count.into().unwrap_or_else(|| num_cpus::get());
        let mut workers = Vec::new();
        workers.reserve_exact(worker_count);

        Self {
            deluge: Some(deluge),
            worker_count,
            worker_concurrency: worker_concurrency.into().and_then(NonZeroUsize::new),

            workers,
            outstanding_futures: None,
            completed_items: BTreeMap::new(),
            // TODO: Only spawn the channel after we know how many items we have to eval
            completed_channel: mpsc::channel(12345),
            _collection: PhantomData,
        }
    }
}

// Approach 
// 1. Central mutexed container for jobs to be stolen from
// 2. Each worker starts with worker_concurrency futures 
//    and steals from the central place as needed


// We need it to proove to the compilter that our worker body is a `FnOnce`
fn make_fn_once<'a, T, F: FnOnce() -> T>(f: F) -> F {
    f
}

// TODO:
// 1. Create evaluators
// 2. Add work stealing

// No need to provide initial work, the worker should pull
// it from `outstanding_futures` by itself
fn create_worker<'a, Del: Deluge<'a> + 'a>(
    outstanding_futures: OutstandingFutures<'a, Del>,
    completed_channel: mpsc::Sender<CompletedItem<'a, Del>>,
    concurrency: NonZeroUsize,
) -> Pin<Box<dyn Future<Output = ()> + 'a>>
{
    Box::pin(async move {
        let mut evaluated_futures: BTreeMap<usize, Pin<Box<Del::Output>>> = BTreeMap::new();

        let run_worker = make_fn_once(|| async {
            let mut more_work_available = true;
            loop {
                // Load up on work if we aren't full and we expect more work to show up
                if evaluated_futures.len() < concurrency.get() && more_work_available {
                    let mut outstanding_futures = outstanding_futures.lock().await;
                    while evaluated_futures.len() < concurrency.get() && !outstanding_futures.is_empty() {
                        if let Some((idx, fut)) = outstanding_futures.pop_first() {
                            evaluated_futures.insert(idx, fut);
                        }
                    }

                    if outstanding_futures.is_empty() {
                        more_work_available = false;
                    }
                }

                // We failed to find any work to perform, shut down gracefully
                if evaluated_futures.is_empty() {
                    break;
                }

                for (idx, fut) in evaluated_futures.iter_mut() {
                    #[cfg(feature = "tokio")]
                    match poll!(fut) {
                        Poll::Ready(v) => completed_channel.send((*idx, v)).await?,
                        _ => (),
                    }

                    #[cfg(feature = "async-std")]
                    match poll!(fut) {
                        Poll::Ready(v) => completed_channel.send((idx, fut)).await,
                        _ => (),
                    }
                }
            }

            Ok::<(), SendError<CompletedItem<'a, Del>>>(())
        });

        if let Err(_e) = run_worker().await {
            // Return unevaluated futures into the pile
            let mut outstanding_futures = outstanding_futures.lock().await;
            evaluated_futures.into_iter()
                .for_each(|(idx, fut)| {
                    outstanding_futures.insert(idx, fut);
            });
        }
    })
}

impl<'a, Del, C> Future for CollectPar<'a, Del, C>
where
    Del: Deluge<'a> + 'a,
    C: Default + Extend<Del::Item>
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<C> {
        let this = self.as_mut().project();

        if this.deluge.is_some() {
            let mut outstanding_futures = BTreeMap::new();
            let mut insert_idx = 0;

            // Load up all the futures
            loop {
                // Funky stuff, extending the lifetime of the inner future
                let deluge: &'a mut Del = unsafe {
                    std::mem::transmute(this.deluge.as_mut().unwrap())
                };
                let next = deluge.next();
                if let Some(future) = next {
                    outstanding_futures.insert(insert_idx, Box::pin(future));
                    insert_idx += 1;
                } else {
                    *this.deluge = None;
                    break;
                }
            }

            let total_futures = outstanding_futures.len();
            *this.outstanding_futures = Some(Arc::new(Mutex::new(outstanding_futures)));

            // Spawn the workers
            if this.workers.is_empty() {
                let worker_concurrency = this.worker_concurrency
                    .or_else(|| NonZeroUsize::new(total_futures / *this.worker_count))
                    .unwrap_or_else(|| unsafe {
                        NonZeroUsize::new_unchecked(1)
                    });

                for _ in 0..(*this.worker_count) {
                    this.workers.push(create_worker::<'a, Del>(
                        this.outstanding_futures.as_ref().unwrap().clone(),
                        this.completed_channel.0.clone(),
                        worker_concurrency
                    ));
                }
            }
        }

        // Wait until the workers finish up
        todo!();
    }
}
