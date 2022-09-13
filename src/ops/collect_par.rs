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
pub struct CollectPar<'a, Del, C>
where Del: Deluge<'a>,
{
    deluge: Option<Del>,
    insert_idx: usize,
    worker_count: usize,
    worker_concurrency: Option<NonZeroUsize>,

    polled_futures: HashMap<usize, Pin<Box<Del::Output>>>,
    completed_futures: HashMap<usize, Del::Item>,
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
        Self {
            deluge: Some(deluge),
            insert_idx: 0,
            worker_count: worker_count.into().unwrap_or_else(|| num_cpus::get()),
            worker_concurrency: worker_concurrency.into().and_then(NonZeroUsize::new),

            polled_futures: HashMap::new(),
            completed_futures: HashMap::new(),
            _collection: PhantomData,
        }
    }
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
