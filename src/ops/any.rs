use crate::deluge::Deluge;
use super::map::Map;
use std::future::Future;

pub struct Any<Del, F> {
    deluge: Map<Del, F>,
}

impl<Del, Fut, F> Any<Del, F> 
where
    Del: Deluge,
    F: Fn(Del::Item) -> Fut + Send,
    Fut: Future<Output = bool> + Send,
{
    pub(crate) fn new(deluge: Del, f: F) -> Self {
        Self {
            deluge: Map::new(deluge, f)
        }
    }
}

// Return a future, which returns if any element evaluates to true