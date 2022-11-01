use crate::deluge::Deluge;
use std::sync::Mutex;

pub struct First<Del> {
    deluge: Del,
    item_provided: Mutex<bool>,
}

impl<Del> First<Del> {
    pub(crate) fn new(deluge: Del) -> Self {
        Self {
            deluge,
            item_provided: Mutex::new(false),
        }
    }
}

impl<Del> Deluge for First<Del>
where
    Del: Deluge,
{
    type Item = Del::Item;
    type Output<'a> = Del::Output<'a> where Self: 'a;

    fn next(&self) -> Option<Self::Output<'_>> {
        let mut item_provided = self.item_provided.lock().unwrap();

        if !*item_provided {
            *item_provided = true;
            self.deluge.next()
        } else {
            None
        }
    }
}
