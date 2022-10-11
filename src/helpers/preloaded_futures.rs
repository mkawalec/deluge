use crate::deluge::Deluge;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

pub struct PreloadedFutures<'a, Del>
where
    Del: Deluge + 'a,
{
    storage: RefCell<VecDeque<Pin<Box<Del::Output<'a>>>>>,
    _del: PhantomData<Del>,
}

impl<'a, Del> PreloadedFutures<'a, Del>
where
    Del: Deluge + 'a,
{
    pub fn new(deluge: Del) -> Self {
        let mut storage = VecDeque::new();
        let deluge_borrow: &'a Del = unsafe { std::mem::transmute(&deluge) };
        while let Some(v) = deluge_borrow.next() {
            storage.push_back(Box::pin(v));
        }

        Self {
            storage: RefCell::new(storage),
            _del: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.storage.borrow().len()
    }
}

impl<'a, Del> Deluge for PreloadedFutures<'a, Del>
where
    Del: Deluge + 'a,
{
    type Item = Del::Item;
    type Output<'x> = impl Future<Output = Option<Self::Item>> + 'x where Self: 'x;

    fn next(&self) -> Option<Self::Output<'_>> {
        let next_item = { self.storage.borrow_mut().pop_front() };

        next_item.map(|el| async move { el.await })
    }
}
