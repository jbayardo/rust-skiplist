#![crate_type = "lib"]
#![crate_name = "skiplist"]
#![feature(rand)]

mod height_control;
mod node;
mod skiplist;
mod iter;
mod range;

pub use skiplist::SkipList;
pub use height_control::{HeightControl, HashCoinGenerator, GeometricalGenerator, TwoPowGenerator};
