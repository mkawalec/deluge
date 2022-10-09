use crate::{deluge::Deluge};
use futures::Future;
use std::cell::RefCell;

pub struct Take<Del> {
    deluge: Del,
    how_many: usize,
    how_many_provided: RefCell<usize>,
}

impl<Del> Take<Del> {
    pub(crate) fn new(deluge: Del, how_many: usize) -> Self {
        Self {
            deluge,
            how_many,
            how_many_provided: RefCell::new(0),
        }
    }
}
impl<Del> Deluge for Take<Del>
where
    Del: Deluge + 'static,
{
    type Item = Del::Item;
    type Output<'a> = impl Future<Output = Option<Self::Item>> + 'a;

    fn next<'a>(&'a self) -> Option<Self::Output<'a>>
    {
        let mut how_many_provided = self.how_many_provided.borrow_mut();
        if *how_many_provided < self.how_many {
            *how_many_provided += 1;
            self.deluge.next()
        } else {
            None
        }
    }
}
