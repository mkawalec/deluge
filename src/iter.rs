use crate::deluge::Deluge;
use std::cell::RefCell;
use std::future::{self, Future};

pub struct Iter<I> {
    iter: RefCell<I>,
}

impl<I> Unpin for Iter<I> {}

/// Converts any iterator into a `Deluge`
pub fn iter<I>(i: I) -> Iter<I::IntoIter>
where
    I: IntoIterator,
{
    Iter {
        iter: RefCell::new(i.into_iter()),
    }
}

impl<I> Deluge for Iter<I>
where
    I: Iterator + Send + 'static,
    <I as Iterator>::Item: Send,
{
    type Item = I::Item;
    type Output<'a> = impl Future<Output = Option<Self::Item>> + 'a;

    fn next(&self) -> Option<Self::Output<'_>> {
        let item = { self.iter.borrow_mut().next() };
        item.map(|item| future::ready(Some(item)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn we_can_create_iter() {
        let _del = iter([1, 2, 3]);
        assert_eq!(2, 2);
    }
}
