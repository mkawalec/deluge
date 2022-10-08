use crate::deluge::Deluge;
use std::future::{self, Future};

pub struct Iter<I> {
    iter: I,
}

impl<I> Unpin for Iter<I> {}

/// Converts any iterator into a `Deluge`
pub fn iter<I>(i: I) -> Iter<I::IntoIter>
where
    I: IntoIterator,
{
    Iter {
        iter: i.into_iter(),
    }
}

impl<I> Deluge for Iter<I>
where
    I: Iterator + Send + 'static,
    <I as Iterator>::Item: Send,
{
    type Item = I::Item;
    type Output<'a> = impl Future<Output = Option<Self::Item>> + 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Output<'a>>
    {
        let item = self.iter.next();
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
