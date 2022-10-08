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
pub struct Zip<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    #[pin]
    first: Collect<'a, PreloadedFutures<'a, Del1>, ()>,

    #[pin]
    second: Collect<'a, PreloadedFutures<'a, Del2>, ()>,

    provided_elems: usize,
    elems_to_provide: usize,
}

impl<'a, Del1, Del2> Zip<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    pub(crate) fn new(
        first: Del1,
        second: Del2,
        concurrency: impl Into<Option<usize>>,
    ) -> Self {
        let concurrency = concurrency.into().map(|conc| conc / 2);

        // Preload the futures from each
        let preloaded1 = PreloadedFutures::preload(first);
        let preloaded2 = PreloadedFutures::preload(second);

        let elems_to_provide = std::cmp::min(preloaded1.len(), preloaded2.len());

        Self {
            first: Collect::new(preloaded1, concurrency.clone()),
            second: Collect::new(preloaded2, concurrency.clone()),

            provided_elems: 0,
            elems_to_provide,
        }
    }
}


struct PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    storage: VecDeque<Pin<Box<Del::Output<'a>>>>,
    _del: PhantomData<Del>
}

impl<'a, Del> PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    fn preload(mut deluge: Del) -> Self {
        let mut storage = VecDeque::new();
        loop {
            if let Some(v) = deluge.next() {
                storage.push_back(Box::pin(v));
            } else {
                break;
            }
        }

        Self {
            storage,
            _del: PhantomData,
        }
    }

    fn len(&self) -> usize {
        self.storage.len()
    }
}

impl<'a, Del> Deluge for PreloadedFutures<'a, Del>
where Del: Deluge + 'a
{
    type Item = Del::Item;
    type Output<'x> where Self: 'x = impl Future<Output = Option<Self::Item>> + 'x;

    fn next<'x>(&'x mut self) -> Option<Self::Output<'x>>
    {
        self.storage.pop_front().map(|el| async move {
            el.await
        })
    }
}

impl<'a, Del1, Del2> Deluge for Zip<'a, Del1, Del2>
where
    Del1: Deluge + 'a,
    Del2: Deluge + 'a,
{
    type Item = (Del1::Item, Del2::Item);
    type Output<'x> where Self: 'x = impl Future<Output = Option<Self::Item>> + 'x;

    fn next<'x>(&'x mut self) -> Option<Self::Output<'x>>
    {
        println!("Next entered");
        if self.provided_elems >= self.elems_to_provide {
            println!("finishd");
            None
        } else {
            self.provided_elems += 1;
            println!("returning a promise");
            Some(async move {
                println!("returning a future");
                let mut this = Pin::new(self).project();
                let (first_el, second_el) = join!(
                    this.first.next(),
                    this.second.next()
                );
                println!("About to wait");
                
                if first_el.is_none() || second_el.is_none() {
                    println!("We're finished");
                    None
                } else {
                    println!("Resutning");
                    Some((first_el.unwrap(), second_el.unwrap()))
                }
            })
        }
    }
}
