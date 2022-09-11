use std::future::Future;

pub trait Deluge<'a>: Send + Sized
{
    type Item: Send;
    type Output: Future<Output = Self::Item> + 'a;

    fn next(self: &'a mut Self) -> Option<Self::Output>;
}
