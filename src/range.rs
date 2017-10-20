// use node::Node;
// use skiplist::SkipList;

// use std;
// use std::collections::range::RangeArgument;
// use std::collections::Bound::{Unbounded, Included, Excluded};

// pub struct Range<'a, K: 'a> {
// start_: *const Node<K>,
// end_bound_: std::collections::Bound<&'a K>,
// }

// impl<'a, K: 'a + std::cmp::Ord> Range<'a, K> {
// #[inline(always)]
// fn new_from(start: *const Node<K>, end_bound: std::collections::Bound<&'a K>) -> Range<'a, K> {
// 	Range {
// 		start_: start,
// 		end_bound_: end_bound,
// 	}
// }

// #[inline(always)]
// fn empty() -> Range<'a, K> {
// 	Range::new_from(std::ptr::null(), Unbounded)
// }

// #[inline(always)]
// pub(crate) fn new<R>(list: &'a SkipList<K>, range: R) -> Range<'a, K>
// where R: 'a + RangeArgument<K> {
// 	// There are a few cases that we want to consider. Basiclaly, we want
// 	// to allow for iteration between: .., k.., ..k, a..b, and their
// 	// exclusive counterparts. This requires doing a careful matching of
// 	// each situation to make sure we give out the most efficient iterator.
// 	//
// 	// We do the matching for start in the constructor, and the matching for
// 	// the end in the next() iterator function.

// 	// The list head always exists, but the first element may not. In this
// 	// case, we are just going to get a nullptr, which represents an empty
// 	// iterator.
// 	let mut start_ptr = unsafe { (*list.head_).ptr_next(0) };
// 	match range.start() {
// 		Unbounded => {
// 			// This is always correct: since the start is unbounded, it is
// 			// either the first node or nullptr if we don't have any.
// 		},
// 		Included(start) => {
// 			start_ptr = unsafe { list.find_lower_bound(start) };
// 			if !start_ptr.is_null() {
// 				// Assuming that the we didn't reach the end of the list, it
// 				// could have gone over the start. This happens in the cases
// 				// like: [ 1 2 16 ] and the range starts at 3. The lower
// 				// bound for 3 is 2, but the next node (Which is where we
// 				// should start) is 16.
// 				let start_ref = unsafe { *start_ptr };
// 				match start_ref.next_or(0) {
// 					None => {
// 						// The next node does not exist. This means that the
// 						// lower bound is the end of the list, hence, the
// 						// iterator is empty.
// 						start_ptr = std::ptr::null();
// 					}
// 					Some(next) => {
// 						// We have a next node. Ensure it is
// 					}
// 				}
// 			}
// 		},
// 		Excluded(start) => {},
// 	}

// 	Range::new_from(start_ptr, range.end())
// }
// }

// impl<'a, K: 'a> Iterator for Range<'a, K> {
// type Item = &'a K;

// fn next(&mut self) -> Option<Self::Item> {
// 	None
// 	// if self.start_.is_null() {
// 	// 	None
// 	// } else {
// 	// 	// Check if we have reached the end.
// 	// 	let start = unsafe { &*self.start_ };
// 	// 	let output = start.key();
// 	// 	// TODO: this needs to be the next with the same type of bound
// 	// 	self.start_ = start.ptr_next(0);

// 	// 	match self.
// 	// 	Some(output)
// 	// }
// }
// }

// // impl<K: Ord> SkipList<K> {
// // 	#[inline(always)]
// // 	pub fn range<R>(&self, range: R) -> Range<K>
// // 	where R: RangeArgument<K> {
// // 		Range::new(self, range)
// // 	}
// // }
