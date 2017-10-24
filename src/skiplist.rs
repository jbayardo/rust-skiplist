use node::Node;
use height_control::HeightControl;

use std;

pub struct SkipList<K, V> {
    /// Pointer to the head of the Skip List. The first node is actually a "ghost"
    /// node: it is created within `SkipList::new`, should only be deleted in
    /// `SkipList::drop`, has the maximum possible height, and it holds dummy data
    /// that should never be touched by the algorithms.
    ///
    /// The reason we have the ghost node is because it simplifies the algorithms
    /// considerably. Searches for nodes all begin in the ghost node, which has
    /// as `next(0)` the actual first element, if any.
    pub(crate) head_: *mut Node<K, V>,

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

impl<K, V> SkipList<K, V> {
    // TODO: custom allocators??
    fn allocate_node(key: K, value: V, height: usize) -> *mut Node<K, V> {
        // Generate the node. All memory allocation is done using Box so
        // that we can actually free it using Box later
        Box::into_raw(Box::new(Node::new(key, value, height)))
    }

    fn free_node(node: *mut Node<K, V>) {
        unsafe {
            Box::from_raw(node);
        }
    }

    fn allocate_dummy_node(max_height: usize) -> *mut Node<K, V> {
        Self::allocate_node(
            // We need to produce a value of type K which will never be accessed
            unsafe { std::mem::uninitialized() },
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

    pub fn new(controller: Box<HeightControl<K>>) -> SkipList<K, V> {
        // This assertion is here because using Zero Sized Types requires
        // special handling which hasn't been implemented yet.
        assert_ne!(std::mem::size_of::<K>(), 0);
        assert_ne!(std::mem::size_of::<V>(), 0);
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

    // TODO: non-memory-releasing clear, for clearing the structure with later release (i.e. drop),
    // should be guaranteed O(1)

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

impl<K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        self.dispose();
    }
}

impl<K: std::fmt::Display, V: std::fmt::Display> std::fmt::Display for SkipList<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printed = self.len();

        write!(f, "[").unwrap();

        for (key, value) in self.iter() {
            printed -= 1;

            if likely!(printed >= 1) {
                write!(f, "{}: {}, ", key, value).unwrap();
            } else {
                write!(f, "{}: {}", key, value).unwrap();
            }
        }

        write!(f, "]").unwrap();
        std::result::Result::Ok(())
    }
}

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for SkipList<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printed = self.len();

        write!(f, "[").unwrap();

        for key in self.iter() {
            printed -= 1;

            if likely!(printed >= 1) {
                write!(f, "{:?}, ", key).unwrap();
            } else {
                write!(f, "{:?}", key).unwrap();
            }
        }

        write!(f, "]").unwrap();
        std::result::Result::Ok(())
    }
}

impl<K: Ord, V> SkipList<K, V> {
    /// Finds the node previous to the node that would have `key`, if any.
    fn find_lower_bound(&self, key: &K) -> &Node<K, V> {
        let mut current_ptr: *const Node<K, V> = self.head_;

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
    fn find_lower_bound_with_updates(&mut self, key: &K) -> (&mut Node<K, V>, Vec<&mut Node<K, V>>) {
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
    pub fn insert(&mut self, key: K, value: V) -> bool {
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

            let node = Self::allocate_node(key, value, height);
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
    pub fn get(&self, key: &K) -> Option<&V> {
        let lower_bound: &Node<K, V> = self.find_lower_bound(key);

        match lower_bound.next(0) {
            Some(node) if likely!(node.key() == key) => Some(node.value()),
            _ => None,
        }
    }

    /// Returns true if `key` is in the list.
    pub fn contains_key(&self, key: &K) -> bool {
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

    pub fn replace(&mut self, key: K) -> Option<V> {
        None
    }

    pub fn take(&mut self, key: &K) -> Option<V> {
        None
    }

//    pub fn split_off(&mut self, key: &K) -> SkipList<K, V> {
//        undefined!()
//    }
}

impl<K: Ord, V> std::ops::Index<K> for SkipList<K, V> {
    type Output = V;

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
    use quickcheck::{Arbitrary, quickcheck, TestResult, Gen};
    use height_control::GeometricalGenerator;

    // TODO: when moving into multithreaded support, ensure we protect accordingly.
    unsafe impl<K, V> Send for SkipList<K, V> {}

    /// This is only implemented in the test environment because we want to
    /// avoid accidentally deep copying a node.
    impl<K: Ord + Clone, V: Clone> Clone for SkipList<K, V> {
        fn clone(&self) -> Self {
            let mut list: SkipList<K, V> = SkipList::new(self.controller_.clone());
            for element in self.iter() {
                list.insert(element.0.clone(), element.1.clone());
            }
            list
        }
    }

    impl<K: Ord + Arbitrary, V: Arbitrary> Arbitrary for SkipList<K, V> {
        fn arbitrary<G: Gen>(gen: &mut G) -> SkipList<K, V> {
            let upgrade_probability = gen.gen_range(0.0, 1.0);
            let max_height = gen.gen_range(1, 30);

            let controller = Box::new(GeometricalGenerator::new(max_height, upgrade_probability));
            let mut list = SkipList::new(controller);

            let length: usize = Arbitrary::arbitrary(gen);
            for _i in 0..length {
                list.insert(Arbitrary::arbitrary(gen), Arbitrary::arbitrary(gen));
            }

            list
        }
    }

    #[test]
    fn new() {
        let list: SkipList<i32, i32> = Default::default();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn clear_empties() {
        fn prop(mut list: SkipList<i32, i32>) -> TestResult {
            list.clear();
            TestResult::from_bool(list.len() == 0 && list.is_empty())
        }

        quickcheck(prop as fn(SkipList<i32, i32>) -> TestResult);
    }

    #[test]
    fn clear_single() {
        let key = 34;
        let value = 9484;
        let mut list: SkipList<i32, i32> = Default::default();
        assert!(list.insert(key, value));
        assert_eq!(list.len(), 1);
        list.clear();
        assert_eq!(list.len(), 0);
        assert!(!list.contains_key(&key));
    }

    #[test]
    fn clear_does_not_invalidate() {
        let mut list: SkipList<usize, usize> = Default::default();

        for i in 0..10 {
            assert_eq!(list.len(), i);
            assert!(list.insert(i, i + 1));
            assert!(!list.insert(i, i + 1));
        }

        assert_eq!(list.len(), 10);
        list.clear();
        assert_eq!(list.len(), 0);

        for i in 0..10 {
            assert_eq!(list.len(), i);
            assert!(!list.contains_key(&i));
            assert!(list.insert(i, i + 1));
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
        let value = 433;
        let mut list: SkipList<i32, i32> = Default::default();
        assert!(list.insert(key, value));
        assert_eq!(list.len(), 1);

        {
            let fetched = list.get(&key);
            assert!(fetched.is_some());
            assert_eq!(*fetched.unwrap(), value);

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
        let value = 55555;
        let mut list: SkipList<i32, i32> = Default::default();

        {
            assert!(list.insert(key, value));
            let first_fetched = list.get(&key);
            assert!(first_fetched.is_some());
            // This is value comparison. The key should be the same as the one inserted
            assert_eq!(*first_fetched.unwrap(), value);
        }

        // The second insertion should fail, the key is already there
        assert!(!list.insert(key, value));
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
        let key_1 = 435;
        let value_1 = 938383;
        let key_2 = 555;
        let value_2 = 98484;
        let mut list: SkipList<i32, i32> = Default::default();
        assert_eq!(list.len(), 0);

        assert!(list.insert(key_1, value_1));
        assert_eq!(list.len(), 1);
        assert!(list.contains_key(&key_1));
        assert!(!list.contains_key(&key_2));

        assert!(list.insert(key_2, value_2));
        assert_eq!(list.len(), 2);
        assert!(list.contains_key(&key_1));
        assert!(list.contains_key(&key_2));

        assert!(list.remove(&key_1));
        assert_eq!(list.len(), 1);
        assert!(!list.contains_key(&key_1));
        assert!(list.contains_key(&key_2));

        assert!(list.insert(key_1, value_1));
        assert_eq!(list.len(), 2);
        assert!(list.contains_key(&key_1));
        assert!(list.contains_key(&key_2));

        assert!(list.remove(&key_2));
        assert_eq!(list.len(), 1);
        assert!(list.contains_key(&key_1));
        assert!(!list.contains_key(&key_2));

        assert!(list.remove(&key_1));
        assert_eq!(list.len(), 0);
        assert!(!list.contains_key(&key_1));
        assert!(!list.contains_key(&key_2));
    }


    #[test]
    fn insert_adds_one_to_length() {
        fn prop(mut list: SkipList<i32, i32>) -> TestResult {
            let length = list.len();
            // This just needs to produce a value that is not in the list yet...
            let sum: i32 = list.iter().map(|v| v.0.abs()).sum();
            list.insert(sum + 1, sum);
            TestResult::from_bool(list.len() == length + 1)
        }

        quickcheck(prop as fn(SkipList<i32, i32>) -> TestResult);
    }

    #[test]
    fn remove_empty() {
        let mut list: SkipList<i32, i32> = Default::default();
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
        let key = 12;
        let value = 83383;
        let mut list: SkipList<i32, i32> = Default::default();

        assert!(list.insert(key, value));
        assert_eq!(list.len(), 1);
        assert!(list.contains_key(&key));

        assert!(list.remove(&key));
        assert_eq!(list.len(), 0);
        assert!(!list.contains_key(&key));

        assert!(!list.remove(&key));
    }

    #[test]
    fn random_insert_remove() {
        use self::rand::Rng;
        let mut rng = self::rand::thread_rng();

        let mut list: SkipList<u32, u32> = Default::default();
        let mut inserted = std::collections::BTreeSet::new();

        let mut elements = 0;
        for _i in 0..1000 {
            let element = rng.next_u32();
            assert_eq!(list.len(), elements);

            assert!(list.insert(element, element + 1));
            assert!(list.contains_key(&element));

            inserted.insert(element);
            elements += 1;
        }

        for element in &inserted {
            assert_eq!(list.len(), elements);

            assert!(list.contains_key(element));
            assert!(!list.insert(*element, element + 2));

            if rng.next_u32() % 2 == 0 {
                assert!(list.remove(element));
                assert!(!list.contains_key(element));
                elements -= 1;
            }
        }
    }
// TODO:
//    #[test]
//    fn remove_takes_one_from_length() {
//        fn prop(mut list: SkipList<i32, i32>) -> TestResult {
//            let length = list.len();
//            if length == 0 {
//                return TestResult::discard();
//            }
//
//            let first = list.iter().next().unwrap().clone();
//            list.remove(first.0);
//            TestResult::from_bool(list.len() == length - 1)
//        }
//
//        quickcheck(prop as fn(SkipList<i32, i32>) -> TestResult);
//    }

    #[test]
    fn format_empty() {
        let list: SkipList<u32, u32> = Default::default();
        assert_eq!(format!("{}", list), "{}");
    }

    #[test]
    fn format_singleton() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(1, 6);
        assert_eq!(format!("{}", list), "{ 1: 6 }");
    }

    #[test]
    fn format_two() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(1, 4);
        list.insert(2, 6);
        assert_eq!(format!("{}", list), "{ 1: 4, 2: 6}");
    }

    #[test]
    fn format_multiple() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(1, 2);
        list.insert(2, 3);
        list.insert(3, 4);
        list.insert(4, 5);
        list.insert(5, 6);
        list.insert(6, 1);
        assert_eq!(format!("{}", list), "[1, 2, 3, 4, 5, 6]")
    }

    #[test]
    #[should_panic]
    fn index_empty() {
        let list: SkipList<u32, u32> = Default::default();
        list[23];
    }

    #[test]
    fn index_singleton() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(32, 12);
        assert_eq!(list[32], 12);
    }

    #[test]
    #[should_panic]
    fn index_singleton_nonexistant() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(32, 43);
        list[23];
    }

    #[test]
    fn index_multiple() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(3, 3);
        list.insert(2, 2);
        list.insert(6, 6);
        list.insert(1, 1);
        list.insert(5, 5);
        list.insert(4, 4);
        assert_eq!(list[6], 6);
    }

    #[test]
    #[should_panic]
    fn index_multiple_nonexistant() {
        let mut list: SkipList<u32, u32> = Default::default();
        list.insert(3, 6);
        list.insert(2, 7);
        list.insert(6, 10);
        list.insert(1, 231);
        list.insert(5, 154);
        list.insert(4, 6565);
        list[23];
    }

    // TODO: memory leak tests
}
