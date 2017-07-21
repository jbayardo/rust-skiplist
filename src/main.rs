#![crate_type = "bin"]
#![crate_name = "skiplist"]

mod node;
mod list;
mod iter;

pub use list::SkipList;
pub use iter::SkipListIter;

fn main() {
	let mut sk : SkipList<i32> = SkipList::new(0.5, 4);

	println!("{}", sk);

	let mut k : i32 = 5;
	while k >= 0 {
		println!("Inserting element {}", k);
		sk.insert(k);
		println!("{}", sk);
		k -= 1;
	}

	let k : i32 = 3;
	println!("{:?}", sk.get(&k));
	println!("{:?}", sk.remove(&k));
	println!("{:?}", sk.get(&k));
	println!("{}", sk);
}