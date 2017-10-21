use node::Node;
use skiplist::SkipList;

use std;

pub struct SkipListHeightIter<'a, K: 'a> {
	height_: usize,
	current_: *const Node<K>,

	// We need this phantom data to ensure that the references returned by the
	// SkipListIter's `next` function will have the correct lifetime
	phantom_: std::marker::PhantomData<&'a Node<K>>,
}

impl<'a, K> SkipListHeightIter<'a, K> {
	pub(crate) fn new(list: &'a SkipList<K>, height: usize) -> SkipListHeightIter<'a, K> {
		SkipListHeightIter {
			// If `list` is an empty skip list, this will actually be a nullptr,
			// if not, then it will be a pointer to the first node.
			current_: unsafe { (*list.head_).next(0).unwrap() },
			height_: height,
			phantom_: std::marker::PhantomData,
		}
	}
}

pub struct SkipListIter<'a, K: 'a> {
    current_: *const Node<K>,
    length_: usize,
    // We need this phantom data to ensure that the references returned by the
    // SkipListIter's `next` function will have the correct lifetime
    phantom_: std::marker::PhantomData<&'a Node<K>>,
}

impl<'a, K> SkipListIter<'a, K> {
    pub(crate) fn new(list: &'a SkipList<K>) -> SkipListIter<'a, K> {
        SkipListIter {
            // If `list` is an empty skip list, this will actually be a nullptr,
            // if not, then it will be a pointer to the first node.
            current_: unsafe { (*list.head_).next(0).unwrap() },
            length_: list.len(),
            phantom_: std::marker::PhantomData,
        }
    }
}

impl<K> SkipList<K> {
    #[inline(always)]
    pub fn iter(&self) -> SkipListIter<K> {
        SkipListIter::new(self)
    }
}

impl<'a, K: 'a> Iterator for SkipListIter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length_ == 0 {
            return None;
        }

        let current = unsafe { &*self.current_ };
		match current.next(0) {
			None => self.current_ = std::ptr::null(),
			Some(next_ptr) => self.current_ = next_ptr
		}

        self.length_ -= 1;
        Some(current.key())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We return a None in the second argument of the iterator because a
        // user may insert elements that end up landing after the exact place
        // we got to.
        // TODO: test if the user can indeed insert while an iterator is ongoing
        (self.length_, Some(self.length_))
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.length_
    }

    fn min(self) -> Option<Self::Item> {
        if self.length_ == 0 {
            None
        } else {
            Some(unsafe { (*self.current_).key() })
        }
    }
}
