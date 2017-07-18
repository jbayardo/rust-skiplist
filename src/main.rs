#![crate_type = "bin"]
#![crate_name = "skiplist"]

mod node;
mod list;
mod iter;

pub use list::SkipList;
pub use iter::SkipListIter;

#[macro_use]
extern crate log;
extern crate env_logger;

fn main() {
	::std::env::set_var("RUST_LOG", "debug");
	env_logger::init().unwrap();
	let mut sk : SkipList<u32> = SkipList::new(0.5, 16, 432112);

	let mut k = 50;
	while k > 0 {
		sk.insert(k);
		println!("{}", sk);
		k -= 1;
	}

	let k : u32 = 3;
	println!("{:?}", sk.get(&k));
	println!("{:?}", sk.remove(&k));
	println!("{:?}", sk.get(&k));
	println!("{}", sk);
}