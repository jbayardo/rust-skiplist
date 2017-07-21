use node::Node;
use list::SkipList;

use std;

#[derive(Debug)]
pub struct SkipListIter<'a, K: 'a> {
	current_: *const Node<K>,
	length_: usize,
	// We need this phantom data to ensure that the references returned by the
	// SkipListIter's `next` function will have the correct lifetime
	phantom_: std::marker::PhantomData<&'a Node<K>>,
}

impl<'a, K> SkipListIter<'a, K> {
	#[inline(always)]
	fn new(item: &'a SkipList<K>) -> SkipListIter<'a, K> {
		SkipListIter {
			// If `item` is an empty skip list, this will actually be a nullptr,
			// if not, then it will be a pointer to the first node.
			current_: unsafe { (*item.head_).ptr_next(0) },
			length_: item.len(),
			phantom_: std::marker::PhantomData,
		}
	}
}

impl<'a, K: 'a + std::fmt::Debug> Iterator for SkipListIter<'a, K> {
	type Item = &'a K;

	fn next(&mut self) -> Option<Self::Item> {
		if self.length_ == 0 {
			return None;
		}

		let current = unsafe { &*self.current_ };
		let output = current.key();
		self.current_ = current.ptr_next(0);
		self.length_ -= 1;
		Some(output)
	}

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		// We return a None in the second argument of the iterator because a
		// user may insert elements that end up landing after the exact place
		// we got to.
		(self.length_, Some(self.length_))
	}

	#[inline(always)]
	fn count(self) -> usize {
		self.length_
	}
}

impl<K> SkipList<K> {
	#[inline(always)]
	pub fn iter(&self) -> SkipListIter<K> {
		SkipListIter::new(self)
	}
}