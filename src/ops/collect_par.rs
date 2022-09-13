use core::pin::Pin;
use std::future::Future;
use std::boxed::Box;
use futures::task::{Context, Poll};
use std::marker::PhantomData;
use pin_project::pin_project;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::collections::HashSet;
use std::default::Default;
use std::num::NonZeroUsize;
use crossbeam::channel;
use crate::deluge::Deluge;

#[pin_project]
pub struct CollectPar<'a, Del, C>
where Del: Deluge<'a>,
{
    deluge: Option<Del>,
    insert_idx: usize,
    worker_count: usize,
    worker_concurrency: Option<NonZeroUsize>,

    workers: Vec<Worker<Del::Output>>,
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
            insert_idx: 0,
            worker_count,
            worker_concurrency: worker_concurrency.into().and_then(NonZeroUsize::new),

            workers,
            _collection: PhantomData,
        }
    }
}

struct Worker<Item> {
    worker: Pin<Box<dyn Future<Output = ()>>>,
    work_sender: channel::Sender<Item>,
    active_jobs: AtomicUsize,
}

// TODO:
// 1. Create evaluators
// 2. Add work stealing

async fn create_worker() {
    unimplemented!()
}

impl<'a, Del, C> Future for CollectPar<'a, Del, C>
where
    Del: Deluge<'a> + 'a,
    C: Default + Extend<Del::Item>
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<C> {
        let this = self.as_mut().project();

        unimplemented!();
    }
}
