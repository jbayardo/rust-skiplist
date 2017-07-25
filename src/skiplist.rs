use node::Node;
use height_control::HeightControl;

use std;

pub struct SkipList<K> {
    pub(crate) head_: *mut Node<K>,

    length_: usize,
    height_: usize,
    controller_: Box<HeightControl<K>>,
}

impl<K: Default> SkipList<K> {
    pub fn new(controller: Box<HeightControl<K>>) -> SkipList<K> {
        // This assertion is here because using Zero Sized Types requires
        // special handling which hasn't been implemented yet.
        assert!(std::mem::size_of::<K>() != 0, "We're not ready to handle ZSTs");

        SkipList {
            head_: Box::into_raw(Box::new(Node::new(Default::default(), controller.max_height()))),
            length_: 0,
            height_: 0,
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
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.length_
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.length_ == 0
    }
}

impl<K: Ord> SkipList<K> {
    // Finds the node previous to the node that would have `key`, if any.
    //
    // This function breaks the mutability correctness, because it takes a const
    // reference to self and returns mutable nodes.
    pub(crate) unsafe fn find_lower_bound(&self, key: &K) -> &mut Node<K> {
        let mut current = self.head_;
        for height in (0..self.height_).rev() {
            while (*current).has_next(height) &&
                  (*current).next(height).key() < key {
                current = (*current).mut_ptr_next(height);
            }
        }

        &mut *current
    }

    // Finds the node previous to the node that would have `key`, if any. It
    // also generates an `updates` vector; the vector contains for index i, the
    // last previous node that had height greater or equal than i.
    //
    // This function breaks the mutability correctness, because it takes a const
    // reference to self and returns mutable nodes.
    pub(crate) unsafe fn find_lower_bound_with_updates(&self, key: &K)
      -> (&mut Node<K>, Vec<&mut Node<K>>) {
        let max_height = self.controller_.max_height();
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
            while (*current).has_next(height) &&
                  (*current).next(height).key() < key {
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
            let (current, mut updates)
                = self.find_lower_bound_with_updates(&key);

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
        let node : &Node<K> = unsafe { self.find_lower_bound(key) };
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
            let (current, mut updates)
                = self.find_lower_bound_with_updates(&key);

            // 'current' is the lower bound to the node, so if it doesn't have a
            // next node at level 0, it means that 'key' is not present. If it
            // does exist, then there is a possibility that it may be greater
            // than the actual key we are looking for
            if !current.has_next(0) {
                return false
            }

            let next = current.mut_ptr_next(0);
            // If the key is not the one that we are looking for, then that
            // means we are done too
            if (*next).key() != key {
                return false
            }

            for h in 0..std::cmp::max((*next).height(), 1) {
                updates[h].set_next(h, (*next).mut_ptr_next(h));
            }

            // Free the memory for the 'next' pointer
            Box::from_raw(next);
        }

        // Update length
        self.length_ -= 1;
        return true
    }

    pub fn replace(&mut self, key: K) -> Option<K> {
        let current = unsafe { self.find_lower_bound(&key) };

        // 'current' is the lower bound to the node, so if it doesn't have a
        // next node at level 0, it means that 'key' is not present. If it
        // does exist, then there is a possibility that it may be greater
        // than the actual key we are looking for
        if !current.has_next(0) {
            return None
        }

        let next = current.mut_next(0);
        // If the key is not the one that we are looking for, then that
        // means we are done too
        if next.key() != &key {
            return None
        }

        return Some(next.replace_key(key))
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
    fn new_empty() {
        // let k_upgrade_probability = 0.5;
        // let k_max_height = 3;
        // let list : SkipList<u32> =
        //     SkipList::new(k_upgrade_probability, k_max_height);
        // assert_eq!(list.len(), 0);
        // assert_eq!(list.upgrade_probability(), k_upgrade_probability);
        // assert_eq!(list.max_height(), k_max_height);
        // assert!(list.is_empty());
    }

    #[test]
    fn insert_single_clear() {
        // let k_upgrade_probability = 0.7;
        // let k_max_height = 8;
        // let mut list : SkipList<i32> =
        //     SkipList::new(k_upgrade_probability, k_max_height);
        // assert!(list.insert(34));
        // assert_eq!(list.len(), 1);
        // assert!(!list.is_empty());

        // list.clear();
        // assert_eq!(list.len(), 0);
        // assert_eq!(list.upgrade_probability(), k_upgrade_probability);
        // assert_eq!(list.max_height(), k_max_height);
        // assert!(list.is_empty());
    }

    #[test]
    fn insert_already_exists() {

    }

    #[test]
    fn insert_multiple() {

    }

    #[test]
    fn get_single() {

    }

    #[test]
    fn get_multiple() {

    }

    #[test]
    fn get_not_found() {

    }

    #[test]
    fn remove_from_empty() {

    }

    #[test]
    fn remove_not_found() {

    }

    #[test]
    fn remove_from_single() {

    }

    #[test]
    fn remove_from_multiple() {

    }

    #[test]
    fn replace_from_empty() {

    }

    #[test]
    fn replace_not_found() {

    }

    #[test]
    fn replace_from_single() {

    }

    #[test]
    fn replace_from_multiple() {

    }
}