#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]
#![feature(map_first_last)]
#![feature(let_chains)]
#![feature(stmt_expr_attributes)]

//! # Deluge is (not) a Stream
//!
//! Deluge implements parallel and concurrent stream operations while driving the underlying futures concurrently.
//! This is in contrast to standard streams which evaluate each future sequentially, leading to large delays on highly concurrent operations.
//! 
//! ```toml
//! deluge = "0.1"
//! ```
//!
//! **This library is still experimental, use at your own risk**
//! 
//! ### Available features
//! 
//! By default the library does not build the parallel collectors and folds.
//! In order to enable them, please enable either the `tokio` or `async-std` feature.
//! 
//! ```toml
//! deluge = { version = "0.1", features = ["tokio"] }
//! ```
//! 
//! or
//! 
//! ```toml
//! deluge = { version = "0.1", features = ["async-std"] }
//! ```
//!
//! ### Design decisions
//!
//! This is an opinionated library that puts ease of use and external simplicity at the forefront.
//! Operations that apply to individual elements like maps and filters **do not** allocate.
//! They simply wrap each element in another future but they do not control the way these processed elements are evaluated.
//! It is the collector that controls the evaluation strategy.
//! At the moment there are two basic collectors supplied: a concurrent and a parallel one.
//!
//! The concurrent collector accepts an optional concurrency limit.
//! If it is specified, at most the number of futures equal to that limit will be evaluated at once.
//!
//! ```
//! use deluge::*;
//! 
//! # futures::executor::block_on(async {
//! let result = deluge::iter([1, 2, 3, 4])
//!    .map(|x| async move { x * 2 })
//!    .collect::<Vec<usize>>(None)
//!    .await;
//!
//! assert_eq!(vec![2, 4, 6, 8], result);
//! # });
//! ```
//!
//! The parallel collector spawns a number of workers.
//! If a number of workers is not specified, it will default to the number of cpus, if the concurrency limit is not specified each worker will default to `total_futures_to_evaluate / number_of_workers`.
//! Note that you need to enable either a `tokio` or `async-std` feature to support parallel collectors.
//!
//! ```
//! use deluge::*;
//! # use std::time::Duration;
//! 
//! # let rt = tokio::runtime::Runtime::new().unwrap();
//! # rt.handle().block_on(async {
//! let result = (0..150)
//!    .into_deluge()
//!    .map(|idx| async move {
//!        tokio::time::sleep(Duration::from_millis(50)).await;
//!        idx
//!    })
//!    .collect_par::<Vec<usize>>(10, None)
//!    .await;
//!
//! assert_eq!(result.len(), 150);
//! # });
//! ```

mod deluge;
mod deluge_ext;
mod into_deluge;
mod iter;
mod ops;

pub use self::deluge::*;
pub use deluge_ext::*;
pub use into_deluge::*;
pub use iter::*;