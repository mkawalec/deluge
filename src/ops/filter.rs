use crate::{deluge::{Deluge}};
use std::{future::Future, marker::PhantomData};

pub struct Filter<'x, Del, F> {
    deluge: Del,
    f: F,
    _del: PhantomData<&'x Del>
}

impl<'x, Del, F> Filter<'x, Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f, _del: PhantomData }
    }
}

/// An internal helper trait allowing us to bind the lifetime
/// of an output future with a lifetime of a parameter to a callback function
pub trait XFn<'a, I: 'a, O> {
    type Output: Future<Output = O> + 'a;
    fn call(&self, x: I) -> Self::Output;
}

impl<'a, I: 'a, O, F, Fut> XFn<'a, I, O> for F
where
    F: Fn(I) -> Fut,
    Fut: Future<Output = O> + 'a,
{
    type Output = Fut;
    fn call(&self, x: I) -> Fut {
        self(x)
    }
}

impl<'x, InputDel, F> Deluge for Filter<'x, InputDel, F>
where
    InputDel: Deluge + 'x,
    for<'b> F: XFn<'b, &'b InputDel::Item, bool> + Send + 'b,
{
    type Item = InputDel::Item;
    type Output<'a> where Self: 'a = impl Future<Output = Option<Self::Item>> + 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Output<'a>>
    {
        self.deluge.next().map(|item| async {
            let item = item.await;
            if let Some(item) = item {
                if self.f.call(&item).await {
                    Some(item)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
