use node::Node;
use height_control::HeightControl;

use std;
use std::borrow::Borrow;

pub struct SkipListMap<K, V> {
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

impl<K, V> SkipListMap<K, V> {
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
            // We need to produce a key and value that will never be accessed
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

            while let Some(next) = (*current).next_mut(0) {
                Self::free_node(current);
                current = next;
            }

            Self::free_node(current);
        }
    }

    pub fn new(controller: Box<HeightControl<K>>) -> SkipListMap<K, V> {
        // This assertion is here because using Zero Sized Types requires
        // special handling which hasn't been implemented yet.
        assert_ne!(std::mem::size_of::<K>(), 0);
        assert_ne!(std::mem::size_of::<V>(), 0);
        let max_height = controller.max_height();

        SkipListMap {
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
    // should be guaranteed O(1). Easy way: append a value that is greater than everything and not
    // equal to anything at the front!.

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

impl<K, V> Drop for SkipListMap<K, V> {
    fn drop(&mut self) {
        self.dispose();
    }
}

impl<K: std::fmt::Display, V: std::fmt::Display> std::fmt::Display for SkipListMap<K, V> {
    // TODO: rewrite
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

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for SkipListMap<K, V> {
    // TODO: rewrite
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

impl<K: Ord, V> SkipListMap<K, V> {
    /// Finds the node previous to the node that would have `key`, if any.
    pub(crate) fn find_lower_bound<Q>(&self, key: &Q) -> &Node<K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
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

    pub(crate) fn find_lower_bound_mut<Q>(&mut self, key: &Q) -> &mut Node<K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut current_ptr: *mut Node<K, V> = self.head_;

        for height in (0..std::cmp::max(self.height_, 1)).rev() {
            while let Some(next) = unsafe { (*current_ptr).next_mut(height) } {
                if likely!(next.key() < key) {
                    current_ptr = next;
                } else {
                    break;
                }
            }
        }

        unsafe { &mut *current_ptr }
    }

    /// Finds the node previous to the node that would have `key`, if any. It
    /// also generates an `updates` vector; the vector contains for index i, the
    /// last previous node that had height greater or equal than i.
    fn find_lower_bound_with_updates<Q>(
        &mut self,
        key: &Q,
    ) -> (&mut Node<K, V>, Vec<&mut Node<K, V>>)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
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
                while let Some(next) = (*current_ptr).next_mut(height) {
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // TODO: initialize this later. This may not ever get used if the key
        // already exists. Should be done right before allocating the node.
        let height = self.controller_.get_height(&key);

        {
            let (lower_bound, mut updates) = self.find_lower_bound_with_updates(&key);

            if let Some(next) = lower_bound.next_mut(0) {
                // The lower bound's next node, if present, could be the same
                // as the key we are looking for, so we could abort early here
                if unlikely!(next.key() == &key) {
                    return Some(next.replace_value(value));
                }
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
        None
    }

    /// Returns a const reference to the element with key `key`, if it exists.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let lower_bound = self.find_lower_bound(key);
        lower_bound.next(0).and_then(
            |node| if likely!(node.key() == key) {
                Some(node.value())
            } else {
                None
            },
        )
    }

    /// Returns a mutable reference to the element with key `key`, if it exists.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let lower_bound = self.find_lower_bound_mut(key);
        lower_bound.next_mut(0).and_then(|node| if likely!(
            node.key() == key
        )
        {
            Some(node.value_mut())
        } else {
            None
        })
    }

    /// Returns true if `key` is in the list.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Removes `key` from the list. Returns true if it was successfully
    /// removed; false if it was not found.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let old_value;

        {
            let (lower_bound, mut updates) = self.find_lower_bound_with_updates(key);

            match lower_bound.next_mut(0) {
                // `lower_bound` is the lower bound to the node, so if it doesn't have a
                // next node at level 0, it means that 'key' is not present. If it
                // does exist, then there is a possibility that it may be greater
                // than the actual key we are looking for
                None => return None,
                Some(removal) => {
                    // If the key is not the one that we are looking for, then that
                    // means we are done
                    if unlikely!(removal.key() != key) {
                        return None;
                    }

                    for (height, update) in updates.iter_mut().enumerate().take(std::cmp::max(
                        removal.height(),
                        1,
                    ))
                    {
                        (*update).link_to_next(height, removal);
                    }

                    old_value = removal.replace_value(unsafe { std::mem::uninitialized() });
                    Self::free_node(removal);
                }
            }
        }

        self.length_ -= 1;
        Some(old_value)
    }

    pub fn first(&self) -> Option<(&K, &V)> {
        unsafe { (*self.head_).next(0).map(|node| node.key_value()) }
    }

    pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        unsafe { (*self.head_).next_mut(0).map(|node| node.key_value_mut()) }
    }

    // TODO: The following are easier to implement with Drain
    pub fn split_off<Q>(&mut self, _key: &Q) -> SkipListMap<K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {

        unimplemented!()
    }

    pub fn append(&mut self, _other: &mut SkipListMap<K, V>) {
        unimplemented!()
    }
}

impl<'a, K, Q, V> std::ops::Index<&'a Q> for SkipListMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'a, K, Q, V> std::ops::IndexMut<&'a Q> for SkipListMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<K: Ord + Clone, V: Clone> Clone for SkipListMap<K, V> {
    fn clone(&self) -> Self {
        let mut copied: SkipListMap<K, V> = SkipListMap::new(self.controller_.clone());
        for element in self.iter() {
            copied.insert(element.0.clone(), element.1.clone());
        }

        copied
    }
}

// TODO: prefetch, benchmarks
#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use quickcheck::{Arbitrary, quickcheck, TestResult, Gen};
    use height_control::GeometricalGenerator;

    // TODO: when moving into multithreaded support, ensure we protect accordingly.
    unsafe impl<K, V> Send for SkipListMap<K, V> {}

    impl<K: Ord + Arbitrary, V: Arbitrary> Arbitrary for SkipListMap<K, V> {
        fn arbitrary<G: Gen>(gen: &mut G) -> SkipListMap<K, V> {
            let upgrade_probability = gen.gen_range(0.0, 1.0);
            let max_height = gen.gen_range(1, 30);

            let controller = Box::new(GeometricalGenerator::new(max_height, upgrade_probability));
            let mut list = SkipListMap::new(controller);

            let length: usize = Arbitrary::arbitrary(gen);
            for _i in 0..length {
                list.insert(Arbitrary::arbitrary(gen), Arbitrary::arbitrary(gen));
            }

            list
        }
    }

    #[test]
    fn clear_empties() {
        fn prop(mut list: SkipListMap<i32, i32>) -> TestResult {
            list.clear();
            TestResult::from_bool(list.len() == 0 && list.is_empty())
        }

        quickcheck(prop as fn(SkipListMap<i32, i32>) -> TestResult);
    }

    #[test]
    fn insert_adds_one_to_length() {
        fn prop(mut list: SkipListMap<i32, i32>) -> TestResult {
            let length = list.len();
            // This just needs to produce a value that is not in the list yet...
            let sum: i32 = list.iter().map(|v| v.0.abs()).sum();
            list.insert(sum + 1, sum);
            TestResult::from_bool(list.len() == length + 1)
        }

        quickcheck(prop as fn(SkipListMap<i32, i32>) -> TestResult);
    }

    //    #[test]
    //    fn remove_takes_one_from_length() {
    //        fn prop(mut list: SkipList<i32, i32>) -> TestResult {
    //            let length = list.len();
    //            if length == 0 {
    //                return TestResult::discard();
    //            }
    //
    //            let first = { list.iter().next().unwrap().clone() };
    //            {
    //                list.remove(first.0);
    //                TestResult::from_bool(list.len() == length - 1)
    //            }
    //        }
    //
    //        quickcheck(prop as fn(SkipList<i32, i32>) -> TestResult);
    //    }
    // TODO: memory leak tests
}
