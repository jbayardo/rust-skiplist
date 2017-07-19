use node::Node;
use list::SkipList;

use std;

pub struct SkipListIter<'a, K: 'a> {
	current_: *const Node<K>,
	phantom_: std::marker::PhantomData<&'a K>,
}

// TODO: attempt to convert all of this into safe rust??
impl<'a, K> SkipListIter<'a, K> {
	pub fn new(item: &'a SkipList<K>) -> SkipListIter<'a, K> {
		unsafe {
			SkipListIter {
				// 'current_' will be null if the list is empty
				current_: (*item.head_).ptr_next(0),
				phantom_: std::marker::PhantomData,
			}
		}
	}
}

impl<'a, K: 'a> Iterator for SkipListIter<'a, K> {
	type Item = &'a K;

	fn next(&mut self) -> Option<Self::Item> {
		// We have reached the end of the list
		if self.current_.is_null() {
			return None;
		}

		unsafe {
			let c : &Node<K> = &*self.current_;
			if c.has_next(0) {
				self.current_ = c.ptr_next(0);
				Some(c.key())
			} else {
				None
			}
		}
	}
}

impl<K> SkipList<K> {
	pub fn iter(&self) -> SkipListIter<K> {
		SkipListIter::new(self)
	}
}