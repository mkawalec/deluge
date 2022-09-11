use std::future::{self, Future};
use crate::deluge::Deluge;

pub struct Iter<I> {
    iter: I
}

impl<I> Unpin for Iter<I> {}

pub fn iter<I>(i: I) -> Iter<I::IntoIter>
where I: IntoIterator
{
    Iter {
        iter: i.into_iter()
    }
}

impl<'a, I> Deluge<'a> for Iter<I>
where I: Iterator + Send + 'a,
      <I as Iterator>::Item: Send + 'a,
{
    type Item = I::Item;
    type Output = impl Future<Output = Self::Item> + 'a;

    fn next(self: &mut Self) -> Option<Self::Output> {
        let item = self.iter.next();
        // TODO: Why is this cast neccessary?
        item.map(|item| future::ready(item))
    }
}