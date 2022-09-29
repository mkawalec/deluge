use crate::deluge::Deluge;
use std::future::Future;

pub struct Filter<Del, F> {
    deluge: Del,
    f: F,
}

impl<Del, F> Filter<Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f }
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

impl<'a, InputDel, F> Deluge<'a> for Filter<InputDel, F>
where
    InputDel: Deluge<'a> + 'a,
    for<'b> F: XFn<'b, &'b InputDel::Item, bool> + Send + 'b,
{
    type Item = InputDel::Item;
    type Output = impl Future<Output = Option<Self::Item>> + 'a;

    fn next(&'a mut self) -> Option<Self::Output> {
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
