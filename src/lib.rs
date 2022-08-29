use core::pin::Pin;
use std::future::{self, Future};
use std::boxed::Box;
use std::marker::PhantomData;
use pin_project::pin_project;

pub trait Deluge<Fut: Future<Output = Self::Item>>
{
    type Item;

    fn next(self: &mut Self) -> Option<Fut>;
}

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

// TODO: This should also accept an iter to futures!
impl<I, Fut> Deluge<Fut> for Iter<I>
where I: Iterator,
      <I as Iterator>::Item: 'static,
      Fut: Future<Output = I::Item>
{
    type Item = I::Item;

    fn next(self: &mut Self) -> Option<Fut> {
        let item = self.iter.next();
        // TODO: Why is this cast neccessary?
        item.map(|item| future::ready(item))
    }
}

// TODO: Concurrent Map and filter
pub struct Map<Del, Fut, F> {
    deluge: Del,
    _fut: PhantomData<Fut>,
    f: F,
}


impl<Del, Fut, F> Map<Del, Fut, F> 
where Del: Deluge<Fut>,
      Fut: Future<Output = Del::Item>
{
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, _fut: PhantomData, f }
    }
}

impl<InputDel, InputFut, F, Fut> Deluge<Fut> for Map<InputDel, InputFut, F>
where InputDel: Deluge<InputFut>,
      InputFut: Future<Output = InputDel::Item>,
      F: FnMut(InputDel::Item) -> Fut,
      Fut: Future,
{
    type Item = Fut::Output;

    fn next(self: &mut Self) -> Option<Fut> {
        self.deluge.next().map(|item| async {
            let item = item.await;
            (self.f)(item).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn we_can_create_iter() {
        let del = iter([1, 2, 3]);
        assert_eq!(2, 2);
    }
}
