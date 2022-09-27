use std::future::Future;
use core::pin::Pin;
use crate::deluge::Deluge;
use std::boxed::Box;
use futures::task::{Context, Poll};
use std::marker::PhantomData;
use pin_project::pin_project;
use std::collections::{HashMap, BTreeMap};
use std::default::Default;
use std::num::NonZeroUsize;

pub struct Fold<Del, Acc, F> {
    deluge: Del,
    accummulator: Acc,
    f: F,
}


impl<Del, Acc, F> Fold<Del, Acc, F> 
{
    pub(crate) fn new(deluge: Del, accummulator: Acc, f: F) -> Self {
        Self { deluge, accummulator, f }
    }
}

impl<'a, InputDel, Acc, Fut, F> Future for Fold<InputDel, Acc, F>
where 
    InputDel: Deluge<'a> + 'a,
    F: FnMut(Acc, InputDel::Item) -> Fut + Send + 'a,
    Fut: Future<Output = Acc> + Send + 'a,
{
    type Output = Acc;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}