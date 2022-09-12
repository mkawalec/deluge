use std::future::Future;

pub trait Deluge<'a>
{
    type Item: Send;
    type Output: Future<Output = Option<Self::Item>> + 'a;

    fn next(self: &'a mut Self) -> Option<Self::Output>;
}