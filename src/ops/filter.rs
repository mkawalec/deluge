use crate::deluge::Deluge;
use std::{future::Future, marker::PhantomData};

pub struct Filter<'x, Del, F> {
    deluge: Del,
    f: F,
    _del: PhantomData<&'x Del>,
}

impl<'x, Del, F> Filter<'x, Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self {
            deluge,
            f,
            _del: PhantomData,
        }
    }
}

/// An internal helper trait allowing us to bind the lifetime
/// of an output future with a lifetime of a parameter to a callback function
pub trait XFn<'a, I: 'a, O> {
    type Output: Future<Output = O> + 'a;
    fn call(&'a self, x: &'a I) -> Self::Output;
}

impl<'a, I: 'a, O, F, Fut> XFn<'a, I, O> for F
where
    F: Fn(&'a I) -> Fut,
    Fut: Future<Output = O> + 'a,
{
    type Output = Fut;
    fn call(&'a self, x: &'a I) -> Fut {
        self(x)
    }
}

impl<'a, InputDel, F> Deluge for Filter<'a, InputDel, F>
where
    InputDel: Deluge + 'a,
    for<'b> F: XFn<'b, InputDel::Item, bool> + Send + 'b,
{
    type Item = InputDel::Item;
    type Output<'x> = impl Future<Output = Option<Self::Item>> + 'x where Self: 'x;

    fn next(&self) -> Option<Self::Output<'_>> {
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
