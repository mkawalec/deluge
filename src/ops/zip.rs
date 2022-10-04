use crate::deluge::Deluge;
use super::collect::Collect;
use std::future::Future;
use pin_project::pin_project;
use std::pin::Pin;
use futures::stream::StreamExt;


#[pin_project]
pub struct Zip<'a, Del1, Del2>
where
    Del1: Deluge<'a>,
    Del2: Deluge<'a>,
{
    #[pin]
    first: Collect<'a, Del1, ()>,

    #[pin]
    second: Collect<'a, Del2, ()>,

    finished: bool,
}

impl<'a, Del1, Del2> Zip<'a, Del1, Del2>
where
    Del1: Deluge<'a>,
    Del2: Deluge<'a>,
{
    pub(crate) fn new(concurrency: impl Into<Option<usize>>, first: Del1, second: Del2) -> Self {
        let concurrency = concurrency.into().map(|conc| conc / 2);

        Self {
            first: Collect::new(first, concurrency.clone()),
            second: Collect::new(second, concurrency.clone()),
            finished: false,
        }
    }
}

impl<'a, Del1, Del2> Deluge<'a> for Zip<'a, Del1, Del2>
where
    Del1: Deluge<'a> + 'a,
    Del2: Deluge<'a> + 'a,
{
    type Item = (Del1::Item, Del2::Item);
    type Output = impl Future<Output = Option<Self::Item>> + 'a;

    fn next(&'a mut self) -> Option<Self::Output> {
        if self.finished {
            None
        } else {
            Some(async move {
                let this = Pin::new(self).project();
                // Get the first nonempty element from `first`, first nonempty from `second`.
                // If none can be found on either, return a filtered element and then None.
                if Some(el) = this.first.next().await {

                } else {
                    *this.finished = true;
                    return None;
                }

                todo!()
            })
        }
    }
}