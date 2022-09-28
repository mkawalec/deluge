pub mod collect;
#[cfg(feature = "parallel")]
pub mod collect_par;
pub mod filter;
pub mod fold;
#[cfg(feature = "parallel")]
pub mod fold_par;
pub mod map;

pub use collect::*;
#[cfg(feature = "parallel")]
pub use collect_par::*;
pub use filter::*;
pub use fold::*;
#[cfg(feature = "parallel")]
pub use fold_par::*;
pub use map::*;
