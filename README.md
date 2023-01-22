# Deluge is (not) a Stream

<div>
 <p>
    <a href="https://crates.io/crates/deluge"><img alt="Crate Info" src="https://img.shields.io/crates/v/deluge.svg"/></a>
    <a href="https://docs.rs/deluge/"><img alt="API Docs" src="https://img.shields.io/badge/docs.rs-deluge-green"/></a>
    <a href="https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html"><img alt="Rustc Version 1.64.0+" src="https://img.shields.io/badge/rustc-1.64%2B-lightgrey.svg"/></a>
  </p>

</div>

`Deluge` builds on top of `Stream` to provide stream operations that are parallel or concurrent by default.
It allows it's user to have an ordered stream of futures that are evaluated concurrently, with all the complexity hidden inside `Deluge` itself.
We achieve this by working one level higher than a stream.
Instead of returning values that might materialize at some point in the future, we immediately return an iterator of unevaluated futures which can then be evaluated by the collector.

The animation below shows an example of mapping over a highly concurrent six element collection. &#x1F4D8; indicates the time it takes for an underlying element to become available, while &#x1F4D7; the time it takes to apply a mapped operation.

![Example of processing using Deluge and Streams](./images/process.gif)

**This library is still experimental, use at your own risk**

### Design decisions

This is an opinionated library that puts ease of use and external simplicity at the forefront.
Operations that apply to individual elements like maps and filters **do not** allocate.
They simply wrap each element in another future but they do not control the way these processed elements are evaluated.
It is the collector that controls the evaluation strategy.
At the moment there are two basic collectors supplied: a concurrent and a parallel one.
Where there is a decision between performance and ease of use to be made, we are likely to fall on the side of ease of use.

The concurrent collector accepts an optional concurrency limit.
If it is specified, at most the number of futures equal to that limit will be evaluated at once.

```rust
let result = [1, 2, 3, 4]
    .into_deluge()
    .map(|x| async move { x * 2 })
    .collect::<Vec<usize>>(None)
    .await;

assert_eq!(vec![2, 4, 6, 8], result);
```

The parallel collector spawns a number of workers.
If a number of workers is not specified, it will default to the number of logical cpus, if the concurrency limit is not specified each worker will default to `total_futures_to_evaluate / number_of_workers`.
Note that you need to enable either a `tokio` or `async-std` feature to support parallel collectors.

```rust
let result = (0..150)
    .into_deluge()
    .map(|idx| async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        idx
    })
    .collect_par::<Vec<usize>>(10, None)
    .await;

assert_eq!(result.len(), 150);
```

Please take a look at [the tests](https://github.com/mkawalec/deluge/blob/main/src/deluge_ext.rs) for more examples of using the library.

### Converting to a Stream

Both `collect` and `collect_par` implement a `Stream`.
If you want to use any of the existing `Stream` extensions or functions that are missing from `Deluge`, `collect` and then use it as any other stream.
Everything that is before collection in `Deluge` gets the benefits of running the futures in parallel.

### Questions

#### I would want to add another operation to DelugeExt. Should I?

By all means.
Please do not allocate on the heap in operations that transform individual elements.
Any operation you would find useful is fair game, contributions are welcome.

#### I found a performance improvement, should I submit a PR?

Absolutely!
As long the API exposed to the users does not get more complex, the number of allocations does not go up and intermediate memory usage does not increase.
Please open an issue first if you feel that breaking any of the above rules is absolutely neccessary and we will discuss.
