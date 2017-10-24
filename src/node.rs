use std;
use std::borrow::{Borrow, BorrowMut};

#[derive(Debug)]
pub(crate) struct Node<K, V> {
    forward_: std::vec::Vec<*mut Node<K, V>>,
    key_: K,
    value_: V,
}

impl<K, V> Node<K, V> {
    // Node of height 0 means it has only one pointer to the next node, node of
    // height 1 means it keeps a pointer to the next node, and to the next
    // height 1 node, and so on and so forth.
    pub fn new(key: K, value: V, height: usize) -> Node<K, V> {
        Node {
            forward_: vec![std::ptr::null_mut(); height + 1],
            key_: key,
            value_: value,
        }
    }

    pub fn height(&self) -> usize {
        self.forward_.len() - 1
    }

    // Returns a reference to the underlying node at the given height
    pub fn next(&self, height: usize) -> Option<&Node<K, V>> {
        match self.forward_.get(height) {
            None => None,
            Some(ptr) => {
                if unlikely!(ptr.is_null()) {
                    None
                } else {
                    Some(unsafe { &**ptr })
                }
            }
        }
    }

    pub fn next_mut(&mut self, height: usize) -> Option<&mut Node<K, V>> {
        match self.forward_.get(height) {
            None => None,
            Some(ptr) => {
                if unlikely!(ptr.is_null()) {
                    None
                } else {
                    Some(unsafe { &mut **ptr })
                }
            }
        }
    }

    pub fn link_to(&mut self, height: usize, destination: *mut Node<K, V>) {
        debug_assert!(height <= self.height());
        unsafe {
            *(self.forward_.get_unchecked_mut(height)) = destination;
        }
    }

    pub fn link_to_next(&mut self, height: usize, node: &Node<K, V>) {
        debug_assert!(height <= self.height());
        debug_assert!(height <= node.height());
        unsafe {
            *(self.forward_.get_unchecked_mut(height)) = *(node.forward_.get_unchecked(height));
        }
    }

    pub fn key<Q>(&self) -> &Q
    where
        K: Borrow<Q>,
        Q: ?Sized,
    {
        (&self.key_).borrow()
    }

    pub fn value<Q>(&self) -> &Q
    where
        V: Borrow<Q>,
        Q: ?Sized,
    {
        (&self.value_).borrow()
    }

    pub fn value_mut<Q>(&mut self) -> &mut Q
    where
        V: BorrowMut<Q>,
        Q: ?Sized,
    {
        (&mut self.value_).borrow_mut()
    }

    pub fn replace_value(&mut self, value: V) -> V {
        std::mem::replace(&mut self.value_, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let key = 3;
        let value = 12;
        let height = 5;
        let node = Node::new(key, value, height);
        assert_eq!(*node.key(), key);
        assert_eq!(*node.value(), value);
        assert_eq!(node.height(), height);
    }

    #[test]
    fn next_out_of_bounds() {
        let key = 3;
        let value = 12;
        let height = 5;
        let mut node = Node::new(key, value, height);
        assert!(node.next(10).is_none());
        assert!(node.next_mut(10).is_none());
    }

    #[test]
    fn next_empty() {
        let key = 3;
        let value = 42;
        let height = 5;
        let mut node = Node::new(key, value, height);
        for height in 0..height {
            assert!(node.next(height).is_none());
            assert!(node.next_mut(height).is_none());
        }
    }

    #[test]
    fn link_singleton() {
        let key = 4;
        let value = 12312;
        let height = 5;

        let k_node_set_height = 0;

        let mut node = Node::new(key, value, height);
        let next_node = Box::into_raw(Box::new(Node::new(key, value, height)));
        node.link_to(k_node_set_height, next_node);

        for h in 0..node.height() {
            let next = node.next_mut(h);

            if h == k_node_set_height {
                let next_ptr = next.unwrap();
                assert_eq!(next_ptr.key(), unsafe { (*next_node).key() });
                assert_eq!(next_ptr.value(), unsafe { (*next_node).value() });
            } else {
                assert!(next.is_none());
            }
        }

        unsafe {
            Box::from_raw(next_node);
        }
    }

    #[test]
    fn link_out_of_range() {}

    #[test]
    fn link_next_singleton() {}

    #[test]
    fn link_next_out_of_range() {}
}
