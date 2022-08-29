use core::pin::Pin;
use std::future::{self, Future};
use std::boxed::Box;
use futures::future::BoxFuture;
use std::marker::PhantomData;
use pin_project::pin_project;

pub trait Deluge
{
    type Item;

    fn next(self: &mut Self) -> Option<BoxFuture<Self::Item>>;
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
impl<I> Deluge for Iter<I>
where I: Iterator,
      <I as Iterator>::Item: Send,
{
    type Item = I::Item;

    fn next(self: &mut Self) -> Option<BoxFuture<Self::Item>> {
        let item = self.iter.next();
        // TODO: Why is this cast neccessary?
        item.map(|item| Box::pin(future::ready(item)) as BoxFuture<Self::Item>)
    }
}

// TODO: Concurrent Map and filter
pub struct Map<Del, F> {
    deluge: Del,
    f: F,
}


impl<Del, F> Map<Del, F> 
{
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self { deluge, f }
    }
}

impl<Del, F, Fut> Deluge for Map<Del, F>
where Del: Deluge,
      F: FnMut(Del::Item) -> Fut,
      Fut: Future,
{
    type Item = Fut::Output;

    fn next(self: &mut Self) -> Option<BoxFuture<Self::Item>> {
        self.deluge.next().map(|item| Box::pin(async {
            let item = item.await;
            (self.f)(item).await
        }))
    }
}

// The idea is that we allocate new futures and the collect itself drives their evaluation
// What about folds? Folds need to evaluate all the futures first...
// Let's take a first approach in which we're just concurrent and the behavior is not configurable

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn we_can_create_iter() {
        let del = iter([1, 2, 3]);
        assert_eq!(2, 2);
    }
}
