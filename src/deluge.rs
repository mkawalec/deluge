use std::future::Future;

pub trait Deluge<'a>: Send + Sized
{
    type Item: Send;
    type Output: Future<Output = Option<Self::Item>> + 'a;

    fn next(self: &'a mut Self) -> Option<Self::Output>;
}

/*
#[derive(Debug)]
pub enum MaybeNext<T> {
    Item(T),
    Ignored,
    DelugeExhausted,
}

impl<T> MaybeNext<T> {
    pub fn map<U, F>(self, f: F) -> MaybeNext<U>
    where
        F: FnOnce(T) -> U,
    {
        use MaybeNext::*;
        match self {
            Item(x) => Item(f(x)),
            Ignored => Ignored,
            DelugeExhausted => DelugeExhausted,
        }
    }

    pub fn and_then<U, F>(self, f: F) -> MaybeNext<U>
    where
        F: FnOnce(T) -> MaybeNext<U>,
    {
        use MaybeNext::*;
        match self {
            Item(x) => f(x),
            Ignored => Ignored,
            DelugeExhausted => DelugeExhausted,
        }
    }
}

impl<T> From<Option<T>> for MaybeNext<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(el) => MaybeNext::Item(el),
            None => MaybeNext::DelugeExhausted,
        }
    }
}
*/