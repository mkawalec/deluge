use crate::deluge::Deluge;
use std::future::Future;

pub struct FilterMap<Del, F> {
    deluge: Del,
    f: F,
}

impl<Del, F> FilterMap<Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f }
    }
}

impl<InputDel, Fut, F, FOutput> Deluge for FilterMap<InputDel, F>
where
    InputDel: Deluge,
    F: Fn(InputDel::Item) -> Fut + Send,
    Fut: Future<Output = Option<FOutput>> + Send,
    FOutput: Send,
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
