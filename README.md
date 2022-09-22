# Deluge, not a stream

Rust's streams drive and evaluate items sequentially.
While this is a simple analog of Iterators, it causes asynchronous operations to take much more time than expected because each future is only driven after the prior one returns a result.
This library aims to invert that pattern by driving the underlying features concurrently or in parallel, to the desired level of concurrency and parallelism.
At the same time all the complexity is hidden away from the users behind well known Iterator-like operations.


### TODO:

- [x] Don't require `collect` to construct an intermediate vector
- [ ] Add `fold`
- [ ] Figure out why `filter` doesn't want to compile in tests
- [ ] Add `filter_map`
- [ ] Document
- [ ] Create proper, not timeing-based, correctness tests
- [ ] Benchmark