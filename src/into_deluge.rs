use crate::iter::{iter, Iter};

pub trait IntoDeluge: IntoIterator {
    fn into_deluge(self) -> Iter<<Self as IntoIterator>::IntoIter>
        where Self: Sized + IntoIterator;
}

impl<T> IntoDeluge for T
where T: IntoIterator,
{
    fn into_deluge(self) -> Iter<T::IntoIter> {
        iter(self)        
    }
}