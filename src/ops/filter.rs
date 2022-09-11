use std::future::Future;
use crate::deluge::Deluge;

pub struct Filter<Del, F> {
    deluge: Del,
    f: F,
}


impl<Del, F> Filter<Del, F> 
{
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f }
    }
}

impl<'a, InputDel, Fut, F> Deluge<'a> for Filter<InputDel, F>
where
    InputDel: Deluge<'a> + 'a,
    for<'b> F: FnMut(&'b InputDel::Item) -> Fut + Send + 'b,
    Fut: Future<Output = bool> + Send + 'a,
{
    type Item = InputDel::Item;
    type Output = impl Future<Output = Option<Self::Item>> + 'a;

    fn next(self: &'a mut Self) -> Option<Self::Output> {
        self.deluge.next().map(|item| async {
            let item = item.await;
            if let Some(item) = item {
                if (self.f)(&item).await {
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