use std::future::Future;

/// A stream of unevaluated futures eventually returning an element of the stream
///
/// An executor such as `collect` or `collect_par` controls how these futures are evaluated.
/// If a `None` is returned for a given element of a collection, it means that
/// element was filtered out earlier in the processing chain and should be omitted.
///
/// If `None` is returned from the call to `next`, the Deluge has ran out of items to provide.
/// Calling `next` again will be unsafe and may lead to panics.
pub trait Deluge<'a> {
    type Item: Send;
    type Output: Future<Output = Option<Self::Item>> + 'a;

    fn next(&'a mut self) -> Option<Self::Output>;
}
