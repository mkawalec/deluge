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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_convert_to_deluge() {
        [1, 2, 3].into_deluge();
    }
}