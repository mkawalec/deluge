use core::pin::Pin;

pub trait Deluge {
    type Item; 
}

pub struct Iter<I> {
    iter: I
}

impl<I> Unpin for Iter<I> {}

pub fn iter<I>(i: I) -> Iter<I::IntoIter>
where I: IntoIterator
{
    unimplemented!();
}

impl<I> Deluge for Iter<I>
where I: Iterator
{
    type Item = I::Item;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
