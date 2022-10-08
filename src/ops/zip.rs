use crate::deluge::Deluge;
use super::collect::Collect;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::PhantomData;
use pin_project::pin_project;
use std::pin::Pin;
use futures::stream::StreamExt;
use futures::join;


#[pin_project]
pub struct Zip<Del1, Del2>
where
    Del1: Deluge,
    Del2: Deluge,
{
    #[pin]
    first: Collect<Del1, ()>,

    #[pin]
    second: Collect<Del2, ()>,

    finished: bool,
}

impl<'a, Del1, Del2> Zip<Del1, Del2>
where
    Del1: Deluge,
    Del2: Deluge,
{
    pub(crate) fn new(
        first: Del1,
        second: Del2,
        concurrency: impl Into<Option<usize>>,
    ) -> Self {
        let concurrency = concurrency.into().map(|conc| conc / 2);

        // Preload the futures from each

        Self {
            first: Collect::new(first, concurrency.clone()),
            second: Collect::new(second, concurrency.clone()),
            finished: false,
        }
    }
}

struct PreloadedFutures<Del>
where Del: Deluge
{
    storage: VecDeque<Del::Output>,
}

impl<'a, Del> PreloadedFutures<Del>
where Del: Deluge
{
    fn preload(mut deluge: Del) -> Self {
        let mut storage = VecDeque::new();
        while let Some(v) = deluge.next() {
            storage.push_back(v);
        }

        Self {
            storage,
        }
    }

    fn len(&self) -> usize {
        self.storage.len()
    }
}

impl<'a, Del> Deluge for PreloadedFutures<Del>
where Del: Deluge
{
    type Item = Del::Item;
    type Output = Del::Output;

    fn next(&'a mut self) -> Option<Self::Output> {
        self.storage.pop_front()
    }
}

impl<Del1, Del2> Deluge for Zip<Del1, Del2>
where
    Del1: Deluge,
    Del2: Deluge,
{
    type Item = (Del1::Item, Del2::Item);
    type Output = impl Future<Output = Option<Self::Item>> + 'a;

    fn next(&mut self) -> Option<Self::Output> {
        println!("Next entered");
        if self.finished {
            println!("finishd");
            None
        } else {
            println!("returning a promise");
            Some(async move {
                println!("returning a future");
                let mut this = Pin::new(self).project();
                println!("About to wait");
                let (first_el, second_el) = join!(
                    this.first.next(),
                    this.second.next()
                );

                if first_el.is_none() || second_el.is_none() {
                    println!("We're finished");
                    *this.finished = true;
                    None
                } else {
                    println!("Resutning");
                    Some((first_el.unwrap(), second_el.unwrap()))
                }
            })
        }
    }
}
