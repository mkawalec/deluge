use crate::deluge::Deluge;
use std::future::Future;

pub struct Map<Del, F> {
    deluge: Del,
    f: F,
}

impl<Del, F> Map<Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f }
    }
}

impl<InputDel, Fut, F> Deluge for Map<InputDel, F>
where
    InputDel: Deluge,
    F: Fn(InputDel::Item) -> Fut + Send,
    Fut: Future + Send,
    <Fut as Future>::Output: Send,
    F: 'static,
    Fut: 'static,
    InputDel: 'static,
{
    type Item = Fut::Output;
    type Output<'a> = impl Future<Output = Option<Self::Item>> + 'a;

    fn next<'a>(&'a self) -> Option<Self::Output<'a>>
    {
        self.deluge.next().map(|item| async {
            let item = item.await;
            if let Some(item) = item {
                Some((self.f)(item).await)
            } else {
                None
            }
        })
    }
}
