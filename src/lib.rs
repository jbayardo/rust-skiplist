#![crate_type = "lib"]
#![crate_name = "skiplist"]

// used in macros
#![feature(core_intrinsics)]
#![feature(allow_internal_unsafe)]
#![feature(stmt_expr_attributes)]

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

pub use crate::map::SkipListMap;
pub use crate::height_control::{HeightControl, HashCoinGenerator, GeometricalGenerator, TwoPowGenerator};
pub use crate::iter::Iter;
