use node::Node;
use list::SkipList;

#[derive(Debug)]
pub struct SkipListIter<K> {
	current: *const Node<K>,
}

impl<K> SkipListIter<K> {
	pub fn new(item: &SkipList<K>) -> SkipListIter<K> {
		if item.len() == 0 {
			return SkipListIter {
				current: ::std::ptr::null(),
			};
		}

		unsafe {
			SkipListIter {
				current: (*item.head_).mut_ptr_next(0),
			}
		}
	}
}

impl<K: Copy> Iterator for SkipListIter<K> {
	type Item = K;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current.is_null() {
			return None;
		}

		unsafe {
			let c : &Node<K> = &*self.current;
			if c.has_next(0) {
				self.current = c.ptr_next(0);
				Some(c.key().clone())
			} else {
				None
			}
		}
	}
}

impl<K: Copy> SkipList<K> {
	pub fn iter(&self) -> SkipListIter<K> {
		SkipListIter::new(self)
	}
}