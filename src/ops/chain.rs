use std::marker::PhantomData;
use std::sync::Mutex;

use crate::deluge::Deluge;
use futures::Future;

pub struct Chain<'a, Del1, Del2> {
    deluge1: Del1,
    deluge2: Del2,
    first_exhausted: Mutex<bool>,
    _lifetime: PhantomData<&'a Del1>,
}

impl<'a, Del1, Del2> Chain<'a, Del1, Del2>
where
    Del1: Deluge,
    Del2: for<'x> Deluge<Item = Del1::Item, Output<'x> = Del1::Output<'x>>,
{
    pub(crate) fn new(deluge1: Del1, deluge2: Del2) -> Self {
        Self {
            deluge1,
            deluge2,
            first_exhausted: Mutex::new(false),
            _lifetime: PhantomData,
        }
    }
}

impl<'a, Del1, Del2> Deluge for Chain<'a, Del1, Del2>
where
    Del1: Deluge + 'static,
    Del2: for <'x> Deluge<Item = Del1::Item, Output<'x> = Del1::Output<'x>> + 'static,
{
    type Item = Del1::Item;
    type Output<'x> = impl Future<Output = Option<Self::Item>> + 'x where Self: 'x;

    fn next(&self) -> Option<Self::Output<'_>> {
        let mut first_exhausted = self.first_exhausted.lock().unwrap();

        let result = if *first_exhausted {
            self.deluge2.next()
        } else {
            match self.deluge1.next() {
                None => {
                    *first_exhausted = true;
                    self.deluge2.next()
                },
                otherwise => otherwise,
            }
        };

        result.map(|v| async move { v.await })
    }
}

