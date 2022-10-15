use crate::deluge::Deluge;
use std::{future::Future, marker::PhantomData};

pub struct FilterMap<'a, Del, F> {
    deluge: Del,
    f: F,
    _lifetime: PhantomData<&'a Del>,
}

impl<'a, Del, F> FilterMap<'a, Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f, _lifetime: PhantomData }
    }
}

impl<'a, InputDel, Fut, F, FOutput> Deluge for FilterMap<'a, InputDel, F>
where
    InputDel: Deluge,
    F: Fn(InputDel::Item) -> Fut + Send,
    Fut: Future<Output = Option<FOutput>> + Send,
    FOutput: Send,
    F: 'a,
    Fut: 'a,
    InputDel: 'a,
{
    type Item = FOutput;
    type Output<'x> = impl Future<Output = Option<Self::Item>> + 'x where Self: 'x;

    fn next(&self) -> Option<Self::Output<'_>> {
        self.deluge.next().map(|item| async {
            let item = item.await;
            if let Some(item) = item {
                (self.f)(item).await
            } else {
                None
            }
        })
    }
}
