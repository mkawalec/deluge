use crate::deluge::Deluge;

use std::collections::VecDeque;
use std::future::Future;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::pin::Pin;

pub struct PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    storage: RefCell<VecDeque<Pin<Box<Del::Output<'a>>>>>,
    _del: PhantomData<Del>,
}

impl<'a, Del> PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    pub fn new(deluge: Del) -> Self {
        let mut storage = VecDeque::new();
        let deluge_borrow: &'a Del = unsafe {
            std::mem::transmute(&deluge)
        };
        loop {
            if let Some(v) = deluge_borrow.next() {
                storage.push_back(Box::pin(v));
            } else {
                break;
            }
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
where Del: Deluge + 'a
{
    type Item = Del::Item;
    type Output<'x> where Self: 'x = impl Future<Output = Option<Self::Item>> + 'x;

    fn next<'x>(&'x self) -> Option<Self::Output<'x>>
    {
        let next_item = {
            self.storage.borrow_mut().pop_front()
        };

        next_item.map(|el| async move {
            el.await
        })
    }
}