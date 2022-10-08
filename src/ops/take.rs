use crate::{deluge::Deluge};
use futures::Future;

pub struct Take<Del> {
    deluge: Del,
    how_many: usize,
    how_many_provided: usize,
}

impl<Del> Take<Del> {
    pub(crate) fn new(deluge: Del, how_many: usize) -> Self {
        Self {
            deluge,
            how_many,
            how_many_provided: 0,
        }
    }
}
impl<Del> Deluge for Take<Del>
where
    Del: Deluge + 'static,
{
    type Item = Del::Item;
    type Output<'a> = impl Future<Output = Option<Self::Item>> + 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Output<'a>>
    {
        if self.how_many_provided < self.how_many {
            self.how_many_provided += 1;
            self.deluge.next()
        } else {
            None
        }
    }
}
