use std::future::Future;
use std::default::Default;

use crate::deluge::Deluge;
use crate::ops::*;

impl<'a, T> DelugeExt<'a> for T 
where T: Deluge<'a>,
{ }

pub trait DelugeExt<'a>: Deluge<'a>
{
    fn map<Fut, F>(self, f: F) -> Map<Self, F>
    where 
        F: FnMut(Self::Item) -> Fut + Send + 'a,
        Fut: Future + Send,
        Self: Sized,
    {
        Map::new(self, f)
    }

    fn filter<F>(self, f: F) -> Filter<Self, F>
    where 
        for<'b> F: XFn<'b, &'b Self::Item, bool> + Send + 'b,
        Self: Sized,
    {
        Filter::new(self, f)
    }

    fn collect<C>(self, concurrency: impl Into<Option<usize>>) -> Collect<'a, Self, C>
    where
        C: Default + Extend<Self::Item>,
        Self: Sized,
    {
        Collect::new(self, concurrency)
    }
}