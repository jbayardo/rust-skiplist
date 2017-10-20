use node::Node;
use height_control::HeightControl;

use std;

pub struct SkipList<K> {
    /// Pointer to the head of the Skip List. The first node is actually a "ghost"
    /// node: it is created within `SkipList::new`, should only be deleted in
    /// `SkipList::drop`, has the maximum possible height, and it holds dummy data
    /// that should never be touched by the algorithms.
    ///
    /// The reason we have the ghost node is because it simplifies the algorithms
    /// considerably. Searches for nodes all begin in the ghost node, which has
    /// as `next(0)` the actual first element, if any.
    pub(crate) head_: *mut Node<K>,

    /// Number of elements in the SkipList
    length_: usize,

    /// Maximum reached height
    height_: usize,

    /// Maximum height the `controller_` can generate. This is stored here instead
    /// of calling `controller_` because all calls to `controller_` are virtually
    /// dispatched, which is more expensive than just holding an usize.
    max_height_: usize,

    /// Used to generate the height for any given node when inserting data.
    controller_: Box<HeightControl<K>>,
}

impl<K: Default> SkipList<K> {
    pub fn new(controller: Box<HeightControl<K>>) -> SkipList<K> {
        // This assertion is here because using Zero Sized Types requires
        // special handling which hasn't been implemented yet.
        assert!(
            std::mem::size_of::<K>() != 0,
            "We're not ready to handle ZSTs"
        );

        SkipList {
            // This is the ghost node mentioned above. The fact that we need a dummy
            // variable here is the reason we have a Default constraint on K.
            head_: Box::into_raw(Box::new(
                Node::new(Default::default(), controller.max_height()),
            )),
            length_: 0,
            height_: 0,
            // See comment on `SkipList::max_height` for reference.
            max_height_: controller.max_height(),
            // The only direct call to controller_ should be done in the
            // `SkipList::insert` function.
            controller_: controller,
        }
    }


    #[inline(always)]
    pub fn clear(&mut self) {
        // TODO: reimplement...
        //*self = Self::new(self.controller_);
    }
}

impl<K> SkipList<K> {
    /// Returns the number of values stored in the structure.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.length_
    }

    /// Returns `true` if there are no values stored within the structure.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.length_ == 0
    }

    /// Returns the maximum reachable height of the SkipList.
    #[inline(always)]
    fn max_height(&self) -> usize {
        self.max_height_
    }
}

impl<K: Ord> SkipList<K> {
    /// Finds the node previous to the node that would have `key`, if any.
    ///
    /// This function breaks the mutability correctness, because it takes a const
    /// reference to self and returns mutable nodes.
    pub(crate) unsafe fn find_lower_bound(&self, key: &K) -> &mut Node<K> {
        let mut current = self.head_;
        for height in (0..self.height_).rev() {
            while (*current).has_next(height) && (*current).next(height).key() < key {
                current = (*current).mut_ptr_next(height);
            }
        }

        &mut *current
    }

    /// Finds the node previous to the node that would have `key`, if any. It
    /// also generates an `updates` vector; the vector contains for index i, the
    /// last previous node that had height greater or equal than i.
    ///
    /// This function breaks the mutability correctness, because it takes a const
    /// reference to self and returns mutable nodes.
    pub(crate) unsafe fn find_lower_bound_with_updates(
        &self,
        key: &K,
    ) -> (&mut Node<K>, Vec<&mut Node<K>>) {
        let max_height = self.max_height();
        let mut updates = Vec::with_capacity(max_height);
        // Initialization for the `updates` vector starts from the back and
        // moves into the front. We set the length of the uninitialized
        // vector to the actual value we are going to use, so that we can do
        // this initialization efficiently
        updates.set_len(max_height);
        for height in self.height_..max_height {
            updates[height] = &mut *self.head_;
        }

        let mut current = self.head_;
        for height in (0..self.height_).rev() {
            while (*current).has_next(height) && (*current).next(height).key() < key {
                current = (*current).mut_ptr_next(height);
            }

            updates[height] = &mut *current;
        }

        (&mut *current, updates)
    }

    pub fn insert(&mut self, key: K) -> bool {
        // TODO: initialize this later. This may not ever get used if the key
        // already exists
        let height = self.controller_.get_height(&key);

        unsafe {
            let (current, mut updates) = self.find_lower_bound_with_updates(&key);

            // The lower bound's next node, if present, could be the same as the
            // key we are looking for, so we could abort early here
            if current.has_next(0) {
                if current.next(0).key() == &key {
                    return false;
                }
            }

            // Generate the node. All memory allocation is done using Box so
            // that we can actually free it using Box later
            let node = Box::into_raw(Box::new(Node::new(key, height)));
            for h in 0..std::cmp::max(height, 1) {
                (*node).set_next(h, updates[h].mut_ptr_next(h));
                updates[h].set_next(h, node);
            }
        }

        self.height_ = std::cmp::max(self.height_, height);
        self.length_ += 1;
        true
    }

    pub fn get(&self, key: &K) -> Option<&K> {
        let node: &Node<K> = unsafe { self.find_lower_bound(key) };
        match node.next_or(0) {
            None => None,
            Some(next) => {
                if next.key() == key {
                    Some(next.key())
                } else {
                    None
                }
            }
        }
    }

    #[inline(always)]
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn remove(&mut self, key: &K) -> bool {
        unsafe {
            let (current, mut updates) = self.find_lower_bound_with_updates(&key);

            // 'current' is the lower bound to the node, so if it doesn't have a
            // next node at level 0, it means that 'key' is not present. If it
            // does exist, then there is a possibility that it may be greater
            // than the actual key we are looking for
            if !current.has_next(0) {
                return false;
            }

            let next = current.mut_ptr_next(0);
            // If the key is not the one that we are looking for, then that
            // means we are done too
            if (*next).key() != key {
                return false;
            }

            for h in 0..std::cmp::max((*next).height(), 1) {
                updates[h].set_next(h, (*next).mut_ptr_next(h));
            }

            // Free the memory for the 'next' pointer
            Box::from_raw(next);
        }

        // Update length
        self.length_ -= 1;
        return true;
    }

    pub fn replace(&mut self, key: K) -> Option<K> {
        let current = unsafe { self.find_lower_bound(&key) };

        // 'current' is the lower bound to the node, so if it doesn't have a
        // next node at level 0, it means that 'key' is not present. If it
        // does exist, then there is a possibility that it may be greater
        // than the actual key we are looking for
        if !current.has_next(0) {
            return None;
        }

        let next = current.mut_next(0);
        // If the key is not the one that we are looking for, then that
        // means we are done too
        if next.key() != &key {
            return None;
        }

        return Some(next.replace_key(key));
    }
}

impl<K: std::fmt::Display> std::fmt::Display for SkipList<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[ ").unwrap();

        for key in self.iter() {
            write!(f, "{} ", key).unwrap();
        }

        write!(f, "]").unwrap();
        std::result::Result::Ok(())
    }
}

impl<K> Drop for SkipList<K> {
    fn drop(&mut self) {
        unsafe {
            let mut current = self.head_;
            while (*current).has_next(0) {
                let next = (*current).mut_ptr_next(0);
                Box::from_raw(current);
                current = next;
            }
            Box::from_raw(current);
        }
    }
}

impl<K: Ord> std::ops::Index<K> for SkipList<K> {
    type Output = K;

    #[inline(always)]
    fn index(&self, index: K) -> &Self::Output {
        return self.get(&index).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let list: SkipList<i32> = Default::default();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn insert_get_single() {
        let k_key = 34;
        let mut list: SkipList<i32> = Default::default();
        assert!(list.insert(k_key));

        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        let fetched = list.get(&k_key);
        assert!(fetched.is_some());
        assert_eq!(*fetched.unwrap(), k_key);

        let second_fetched = list.get(&k_key);
        assert!(second_fetched.is_some());
        // The keys returned in multiple get() calls should always point to the same
        // address as the first one (there should be no copies).
        assert_eq!(second_fetched.unwrap(), fetched.unwrap());
    }

    #[test]
    fn insert_get_duplicate() {
        let k_key = 55;
        let mut list: SkipList<i32> = Default::default();

        assert!(list.insert(k_key));

        {
            let first_fetched = list.get(&k_key);
            assert!(first_fetched.is_some());
            // This is value comparison. The key should be the same as the one inserted
            assert_eq!(*first_fetched.unwrap(), k_key);
        }

        // The second insertion should fail, the key is already there
        assert!(!list.insert(k_key));
        // Duplicate insertions don't change the length
        assert_eq!(list.len(), 1);
        let second_fetched = list.get(&k_key);
        assert!(second_fetched.is_some());
        // This is reference comparison. The reference returned should be the same
        // as the reference returned the first time (i.e. there should be no new
        // key allocations)
        // TODO: this has problems due to lifetimes.
        // assert_eq!(first_fetched.unwrap(), second_fetched.unwrap());
    }
}
