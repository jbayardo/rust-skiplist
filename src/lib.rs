#![crate_type = "lib"]
#![crate_name = "skiplist"]

// used in macros
#![feature(core_intrinsics)]
#![feature(allow_internal_unsafe)]
#![feature(stmt_expr_attributes)]

// used for ranges
#![feature(collections_range)]

// test framework
#![cfg_attr(test, feature(plugin))]
#[cfg(test)]
extern crate quickcheck;

#[macro_use]
mod macros;

mod height_control;
mod node;
mod map;
mod iter;

pub use map::SkipListMap;
pub use height_control::{HeightControl, HashCoinGenerator, GeometricalGenerator, TwoPowGenerator};
pub use iter::Iter;
