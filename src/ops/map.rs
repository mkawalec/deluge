use crate::deluge::Deluge;
use std::future::Future;

pub(crate) struct Map<Del, F> {
    deluge: Del,
    f: F,
}

impl<Del, F> Map<Del, F> {
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f }
    }
}

impl<'a, InputDel, Fut, F> Deluge<'a> for Map<InputDel, F>
where
    InputDel: Deluge<'a> + 'a,
    F: FnMut(InputDel::Item) -> Fut + Send + 'a,
    Fut: Future + Send + 'a,
    <Fut as Future>::Output: Send,
{
    type Item = Fut::Output;
    type Output = impl Future<Output = Option<Self::Item>> + 'a;

    fn next(&'a mut self) -> Option<Self::Output> {
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
