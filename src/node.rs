use std;
use std::ops::{Index};

#[derive(Debug)]
pub struct Node<K> {
	forward_: std::vec::Vec<*mut Node<K>>,
	key_: K,
}

impl<K> Node<K> {
	// Node of height 0 means it has only one pointer to the next node, node of
	// height 1 means it keeps a pointer to the next node, and to the next
	// height 1 node, and so on and so forth.
	#[inline(always)]
	pub fn new(key: K, height: usize) -> Node<K> {
		Node {
			forward_: vec![std::ptr::null_mut(); height + 1],
			key_: key,
		}
	}

	#[inline(always)]
	pub fn has_next(&self, height: usize) -> bool {
		height < self.forward_.len() && !self.forward_.index(height).is_null()
	}

	// Returns a reference to the underlying node at the given height
	#[inline(always)]
	pub fn next(&self, height: usize) -> &Node<K> {
		debug_assert!(self.has_next(height));
		unsafe { &*self.forward_[height] }
	}

	// Returns a mutable reference to the underlying node at the given height
	#[inline(always)]
	pub fn mut_next(&mut self, height: usize) -> &mut Node<K> {
		debug_assert!(self.has_next(height));
		unsafe { &mut *self.forward_[height] }
	}

	#[inline(always)]
	pub fn ptr_next(&self, height: usize) -> *const Node<K> {
		debug_assert!(self.has_next(height));
		self.forward_[height].clone()
	}

	#[inline(always)]
	pub fn mut_ptr_next(&mut self, height: usize) -> *mut Node<K> {
		debug_assert!(self.has_next(height));
		self.forward_[height].clone()
	}

	#[inline(always)]
	pub fn set_next(&mut self, height: usize, destination: *mut Node<K>) {
		debug_assert!(height < self.forward_.len());
		self.forward_[height] = destination;
	}

	#[inline(always)]
	pub fn key(&self) -> &K {
		&self.key_
	}
}