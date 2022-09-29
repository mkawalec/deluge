#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]
#![feature(map_first_last)]
#![feature(let_chains)]
#![feature(stmt_expr_attributes)]

mod deluge;
mod deluge_ext;
mod into_deluge;
mod iter;
mod ops;

pub use self::deluge::*;
pub use deluge_ext::*;
pub use into_deluge::*;
pub use iter::*;
pub use ops::*;
