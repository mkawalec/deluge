use core::pin::Pin;
use std::future::{self, Future};
use std::boxed::Box;
use futures::future::BoxFuture;
use futures::task::{Context, Poll};
use std::marker::PhantomData;
use pin_project::pin_project;
use std::collections::HashMap;

pub trait Deluge: Send + Sized
{
    type Item: Send;

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
where I: Iterator + Send,
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
where 
    Del: Deluge,
    F: FnMut(Del::Item) -> Fut + Send,
    Fut: Future + Send,
    <Fut as Future>::Output: Send,
{
    type Item = Fut::Output;

    fn next(self: &mut Self) -> Option<BoxFuture<Self::Item>> {
        self.deluge.next().map(|item| Box::pin(async {
            let item = item.await;
            (self.f)(item).await
        }) as Pin<Box<dyn Future<Output = Self::Item> + Send>>)
    }
}

#[pin_project]
pub struct Collect<'a, Del, C> {
    deluge: Option<Del>,
    current_index: usize,
    polled_futures: HashMap<usize, BoxFuture<'a, C>>,
    completed_futures: HashMap<usize, BoxFuture<'a, C>>,
}

impl<'a, Del: Deluge, C: Default> Collect<'a, Del, C> 
{
    pub(crate) fn new(deluge: Del) -> Self {
        Self {
            deluge: Some(deluge),
            current_index: 0,
            polled_futures: HashMap::new(),
            completed_futures: HashMap::new(),
        }
    }
}

impl<'a, Del, C> Future for Collect<'a, Del, C>
where
    Del: Deluge + Deluge<Item = C>,
    C: Default + Extend<Del::Item>,
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<C> {
        let mut this = self.as_mut().project();
        while this.deluge.is_some() {
            let deluge = this.deluge.as_mut().unwrap();
            if let Some(future) = deluge.next() {
                self.polled_futures.insert(*this.current_index, future);
                *this.current_index += 1;
            } else {
                *this.deluge = None;
            }
        }
        // Need to iterate through all the promises. If a given promise is not ready yet,
        // let's poll it and continue with other promises
        unimplemented!()
    }
}


impl<T: Sized> DelugeExt for T where T: Deluge { }

trait DelugeExt: Deluge {
    fn map<Fut, F>(self, f: F) -> Map<Self, F>
    where 
        F: FnMut(Self::Item) -> Fut + Send,
        Fut: Future + Send,
    {
        Map::new(self, f)
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

    #[tokio::test]
    async fn map_can_be_created() {
        iter([1, 2, 3, 4])
            .map(|x| async move { x * 2 });
        assert_eq!(2, 2);
    }
}
