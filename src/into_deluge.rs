use crate::iter::{iter, Iter};
use crate::Deluge;

/// Allows converting any type that implements `IntoDeluge` into a `Deluge`.
/// Specifically anything that implements `IntoIterator` or is a `Deluge` itself
/// can be converted into `Deluge`.
pub trait IntoDeluge<T>
where
    T: Deluge
{
    fn into_deluge(self) -> T
    where
        Self: Sized;
}

impl<T> IntoDeluge<Iter<T::IntoIter>> for T
where
    T: IntoIterator,
    <T as IntoIterator>::IntoIter: Send + 'static,
    <T as IntoIterator>::Item: Send,
{
    fn into_deluge(self) -> Iter<T::IntoIter> {
        iter(self)
    }
}

impl<T> IntoDeluge<T> for T
where
    T: Deluge
{
    fn into_deluge(self) -> Self {
        self
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
