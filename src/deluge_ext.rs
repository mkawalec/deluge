use std::future::Future;
use std::default::Default;

use crate::deluge::Deluge;
use crate::ops::*;

impl<'a, T: Sized> DelugeExt<'a> for T 
where T: Deluge<'a>,
{ }

pub trait DelugeExt<'a>: Deluge<'a>
{
    fn map<Fut, F>(self, f: F) -> Map<Self, F>
    where 
        F: FnMut(Self::Item) -> Fut + Send + 'a,
        Fut: Future + Send,
    {
        Map::new(self, f)
    }

    fn filter<Fut, F>(self, f: F) -> Filter<Self, F>
    where 
        F: FnMut(&Self::Item) -> Fut + Send + 'a,
        Fut: Future<Output = bool> + Send,
    {
        Filter::new(self, f)
    }

    fn collect<C>(self) -> Collect<'a, Self, C>
    where
        C: Default + Extend<Self::Item>
    {
        Collect::new(self)
    }
}