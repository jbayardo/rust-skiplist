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

impl<K> SkipList<K> {
    // TODO: custom allocators??
    fn allocate_node(key: K, height: usize) -> *mut Node<K> {
        // Generate the node. All memory allocation is done using Box so
        // that we can actually free it using Box later
        Box::into_raw(Box::new(Node::new(key, height)))
    }

    fn free_node(node: *mut Node<K>) {
        unsafe {
            Box::from_raw(node);
        }
    }

    fn allocate_dummy_node(max_height: usize) -> *mut Node<K> {
        Self::allocate_node(
            // We need to produce a value of type K which will never be accessed
            unsafe { std::mem::uninitialized() },
            max_height,
        )
    }

    /// Releases the memory held by the data structure. Does not initialize it again, so the state
    /// after usage is invalid. See `clear` function for reference on how to restore.
    fn dispose(&mut self) {
        unsafe {
            let mut current = self.head_;

            while let Some(next) = (*current).mut_next(0) {
                Self::free_node(current);
                current = next;
            }

            Self::free_node(current);
        }
    }

    pub fn new(controller: Box<HeightControl<K>>) -> SkipList<K> {
        // This assertion is here because using Zero Sized Types requires
        // special handling which hasn't been implemented yet.
        assert_ne!(std::mem::size_of::<K>(), 0);
        let max_height = controller.max_height();

        SkipList {
            // This is the ghost node mentioned above.
            head_: Self::allocate_dummy_node(max_height),
            length_: 0,
            height_: 0,
            // See comment on `SkipList::max_height` for reference.
            max_height_: max_height,
            // The only direct call to controller_ should be done in the
            // `SkipList::insert` function.
            controller_: controller,
        }
    }

    // TODO: non-memory-releasing clear, for clearing the structure with later release (i.e. drop)
    /// Removes all elements.
    pub fn clear(&mut self) {
        self.dispose();
        self.head_ = Self::allocate_dummy_node(self.max_height());
        self.length_ = 0;
        self.height_ = 0;
    }

    /// Returns the number of elements stored in the structure.
    pub fn len(&self) -> usize {
        self.length_
    }

    /// Returns `true` if there are no elements stored within the structure.
    pub fn is_empty(&self) -> bool {
        self.length_ == 0
    }

    /// Returns the maximum reachable height of the SkipList.
    fn max_height(&self) -> usize {
        self.max_height_
    }
}

impl<K> Drop for SkipList<K> {
    fn drop(&mut self) {
        self.dispose();
    }
}

impl<K: std::fmt::Display> std::fmt::Display for SkipList<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printed = self.len();

        write!(f, "[").unwrap();

        for key in self.iter() {
            printed -= 1;

            if likely!(printed >= 1) {
                write!(f, "{}, ", key).unwrap();
            } else {
                write!(f, "{}", key).unwrap();
            }
        }

        write!(f, "]").unwrap();
        std::result::Result::Ok(())
    }
}

impl<K: Ord> SkipList<K> {
    /// Finds the node previous to the node that would have `key`, if any.
    fn find_lower_bound(&self, key: &K) -> &Node<K> {
        let mut current_ptr: *const Node<K> = self.head_;

        for height in (0..std::cmp::max(self.height_, 1)).rev() {
            while let Some(next) = unsafe { (*current_ptr).next(height) } {
                if likely!(next.key() < key) {
                    current_ptr = next;
                } else {
                    break;
                }
            }
        }

        unsafe { &*current_ptr }
    }

    /// Finds the node previous to the node that would have `key`, if any. It
    /// also generates an `updates` vector; the vector contains for index i, the
    /// last previous node that had height greater or equal than i.
    fn find_lower_bound_with_updates(&mut self, key: &K) -> (&mut Node<K>, Vec<&mut Node<K>>) {
        let max_height = self.max_height();
        let mut updates = Vec::with_capacity(max_height);

        // Initialization for the `updates` vector starts from the back and
        // moves into the front. We set the length of the uninitialized
        // vector to the actual value we are going to use, so that we can do
        // this initialization efficiently
        unsafe {
            updates.set_len(max_height);

            for update in updates.iter_mut().take(max_height).skip(self.height_) {
                *update = &mut *self.head_;
            }

            let mut current_ptr = self.head_;
            for height in (0..std::cmp::max(self.height_, 1)).rev() {
                while let Some(next) = (*current_ptr).mut_next(height) {
                    if likely!(next.key() < key) {
                        current_ptr = next;
                    } else {
                        break;
                    }
                }

                updates[height] = &mut *current_ptr;
            }

            (&mut *current_ptr, updates)
        }
    }

    // Insert `key`. Returns false if `key` was already found.
    pub fn insert(&mut self, key: K) -> bool {
        // TODO: initialize this later. This may not ever get used if the key
        // already exists
        let height = self.controller_.get_height(&key);

        {
            let (lower_bound, mut updates) = self.find_lower_bound_with_updates(&key);

            match lower_bound.next(0) {
                // The lower bound's next node, if present, could be the same as the
                // key we are looking for, so we could abort early here
                Some(next) if unlikely!(next.key() == &key) => return false,
                _ => {}
            }

            let node = Self::allocate_node(key, height);
            for (height, update) in updates.iter_mut().enumerate().take(
                std::cmp::max(height, 1),
            )
            {
                unsafe {
                    (*node).link_to_next(height, update);
                }

                (*update).link_to(height, node);
            }
        }

        self.height_ = std::cmp::max(self.height_, height);
        self.length_ += 1;
        true
    }

    /// Returns the element with key `key`, if it exists.
    pub fn get(&self, key: &K) -> Option<&K> {
        let lower_bound: &Node<K> = self.find_lower_bound(key);

        match lower_bound.next(0) {
            Some(node) if likely!(node.key() == key) => Some(node.key()),
            _ => None,
        }
    }

    /// Returns true if `key` is in the list.
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Removes `key` from the list. Returns true if it was successfully
    /// removed; false if it was not found.
    pub fn remove(&mut self, key: &K) -> bool {
        {
            let (lower_bound, mut updates) = self.find_lower_bound_with_updates(key);

            match lower_bound.mut_next(0) {
                // `lower_bound` is the lower bound to the node, so if it doesn't have a
                // next node at level 0, it means that 'key' is not present. If it
                // does exist, then there is a possibility that it may be greater
                // than the actual key we are looking for
                None => return false,
                Some(removal) => {
                    // If the key is not the one that we are looking for, then that
                    // means we are done
                    if unlikely!(removal.key() != key) {
                        return false;
                    }

                    for (height, update) in updates.iter_mut().enumerate().take(std::cmp::max(
                        removal.height(),
                        1,
                    ))
                    {
                        (*update).link_to_next(height, removal);
                    }

                    Self::free_node(removal);
                }
            }
        }

        self.length_ -= 1;
        true
    }
}

impl<K: Ord> std::ops::Index<K> for SkipList<K> {
    type Output = K;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

// TODO: Deref which returns an iter.
// TODO: range queries

// TODO: prefetch, benchmarks
#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;

    #[test]
    fn new() {
        let list: SkipList<i32> = Default::default();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn clear_empty() {
        let mut list: SkipList<i32> = Default::default();
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn clear_single() {
        let key = 34;
        let mut list: SkipList<i32> = Default::default();
        assert!(list.insert(key));
        assert_eq!(list.len(), 1);
        list.clear();
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&key));
    }

    #[test]
    fn clear_multiple() {
        let mut list: SkipList<usize> = Default::default();

        for i in 0..10 {
            assert_eq!(list.len(), i);
            assert!(list.insert(i));
            assert!(!list.insert(i));
        }

        assert_eq!(list.len(), 10);
        list.clear();
        assert_eq!(list.len(), 0);

        for i in 0..10 {
            assert_eq!(list.len(), i);
            assert!(!list.contains(&i));
            assert!(list.insert(i));
        }

        assert_eq!(list.len(), 10);
        list.clear();
        assert_eq!(list.len(), 0);

        for i in 0..10 {
            assert!(!list.remove(&i));
            assert_eq!(list.len(), 0);
        }
    }

    #[test]
    fn insert_get_single() {
        let key = 34;
        let mut list: SkipList<i32> = Default::default();
        assert!(list.insert(key));
        assert_eq!(list.len(), 1);

        {
            let fetched = list.get(&key);
            assert!(fetched.is_some());
            assert_eq!(*fetched.unwrap(), key);

            let second_fetched = list.get(&key);
            assert!(second_fetched.is_some());
            // The keys returned in multiple get() calls should always point to the same
            // address as the first one (there should be no copies).
            assert_eq!(second_fetched.unwrap(), fetched.unwrap());
        }

        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn insert_get_duplicate() {
        let key = 55;
        let mut list: SkipList<i32> = Default::default();

        {
            assert!(list.insert(key));
            let first_fetched = list.get(&key);
            assert!(first_fetched.is_some());
            // This is value comparison. The key should be the same as the one inserted
            assert_eq!(*first_fetched.unwrap(), key);
        }

        // The second insertion should fail, the key is already there
        assert!(!list.insert(key));
        // Duplicate insertions don't change the length
        assert_eq!(list.len(), 1);
        let second_fetched = list.get(&key);
        assert!(second_fetched.is_some());

        // This is reference comparison. The reference returned should be the same
        // as the reference returned the first time (i.e. there should be no new
        // key allocations)
        // TODO: this has problems due to lifetimes.
        //assert_eq!(first_fetched.unwrap(), second_fetched.unwrap());
    }

    #[test]
    fn insert_two_remove() {
        let key_1: i32 = 435;
        let key_2: i32 = 555;
        let mut list: SkipList<i32> = Default::default();
        assert_eq!(list.len(), 0);

        assert!(list.insert(key_1));
        assert_eq!(list.len(), 1);
        assert!(list.contains(&key_1));
        assert!(!list.contains(&key_2));

        assert!(list.insert(key_2));
        assert_eq!(list.len(), 2);
        assert!(list.contains(&key_1));
        assert!(list.contains(&key_2));

        assert!(list.remove(&key_1));
        assert_eq!(list.len(), 1);
        assert!(!list.contains(&key_1));
        assert!(list.contains(&key_2));

        assert!(list.insert(key_1));
        assert_eq!(list.len(), 2);
        assert!(list.contains(&key_1));
        assert!(list.contains(&key_2));

        assert!(list.remove(&key_2));
        assert_eq!(list.len(), 1);
        assert!(list.contains(&key_1));
        assert!(!list.contains(&key_2));

        assert!(list.remove(&key_1));
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&key_1));
        assert!(!list.contains(&key_2));
    }

    #[test]
    fn remove_empty() {
        let mut list: SkipList<i32> = Default::default();
        assert!(list.is_empty());
        assert!(!list.remove(&3));
        assert_eq!(list.len(), 0);
        assert!(!list.remove(&32));
        assert_eq!(list.len(), 0);
        assert!(!list.remove(&22));
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn remove_single() {
        let key: i32 = 12;
        let mut list: SkipList<i32> = Default::default();

        assert!(list.insert(key));
        assert_eq!(list.len(), 1);
        assert!(list.contains(&key));

        assert!(list.remove(&key));
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&key));

        assert!(!list.remove(&key));
    }

    #[test]
    fn random_insert_remove() {
        use self::rand::Rng;
        let mut rng = self::rand::thread_rng();

        let mut list: SkipList<u32> = Default::default();
        let mut inserted = std::collections::BTreeSet::new();

        let mut elements = 0;
        for _i in 0..1000 {
            let element = rng.next_u32();
            assert_eq!(list.len(), elements);

            assert!(list.insert(element));
            assert!(list.contains(&element));

            inserted.insert(element);
            elements += 1;
        }

        for element in &inserted {
            assert_eq!(list.len(), elements);

            assert!(list.contains(element));
            assert!(!list.insert(*element));

            if rng.next_u32() % 2 == 0 {
                assert!(list.remove(element));
                assert!(!list.contains(element));
                elements -= 1;
            }
        }
    }

    #[test]
    fn format_empty() {
        let list: SkipList<u32> = Default::default();
        assert_eq!(format!("{}", list), "[]");
    }

    #[test]
    fn format_singleton() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(1);
        assert_eq!(format!("{}", list), "[1]");
    }

    #[test]
    fn format_two() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(1);
        list.insert(2);
        assert_eq!(format!("{}", list), "[1, 2]");
    }

    #[test]
    fn format_multiple() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(1);
        list.insert(2);
        list.insert(3);
        list.insert(4);
        list.insert(5);
        list.insert(6);
        assert_eq!(format!("{}", list), "[1, 2, 3, 4, 5, 6]")
    }

    #[test]
    #[should_panic]
    fn index_empty() {
        let list: SkipList<u32> = Default::default();
        list[23];
    }

    #[test]
    fn index_singleton() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(32);
        assert_eq!(list[32], 32);
    }

    #[test]
    #[should_panic]
    fn index_singleton_nonexistant() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(32);
        list[23];
    }

    #[test]
    fn index_multiple() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(3);
        list.insert(2);
        list.insert(6);
        list.insert(1);
        list.insert(5);
        list.insert(4);
        assert_eq!(list[6], 6);
    }

    #[test]
    #[should_panic]
    fn index_multiple_nonexistant() {
        let mut list: SkipList<u32> = Default::default();
        list.insert(3);
        list.insert(2);
        list.insert(6);
        list.insert(1);
        list.insert(5);
        list.insert(4);
        list[23];
    }

    // TODO: memory leak tests
}
