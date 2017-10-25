use node::Node;
use map::SkipListMap;

use std;
use std::borrow::Borrow;
use std::collections::range::RangeArgument;
use std::collections::Bound;

pub struct Iter<'a, K: 'a, V: 'a>(Option<&'a Node<K, V>>);

impl<'a, K, V> Iter<'a, K, V> {
    pub fn new(list: &'a SkipListMap<K, V>) -> Iter<'a, K, V> {
        Iter(unsafe { (*list.head_).next(0) })
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: prefetch, likely
        let key_value = self.0.map(|node| node.key_value());
        self.0 = self.0.and_then(|node| node.next(0));
        key_value
    }
}

pub struct IterMut<'a, K: 'a, V: 'a>(Option<&'a mut Node<K, V>>);

impl<'a, K, V> IterMut<'a, K, V> {
    pub fn new(list: &'a mut SkipListMap<K, V>) -> IterMut<'a, K, V> {
        IterMut(unsafe { (*list.head_).next_mut(0) })
    }
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: do this the right way...
        let current = std::mem::replace(&mut self.0, None);
        if let Some(node) = current {
            let node_ptr: *mut Node<K, V> = node;
            std::mem::replace(&mut self.0, node.next_mut(0));
            Some(unsafe { (*node_ptr).key_value_mut() })
        } else {
            None
        }
    }
}

pub struct Keys<'a, K: 'a, V: 'a>(Iter<'a, K, V>);

impl<'a, K, V> Keys<'a, K, V> {
    pub fn new(list: &'a SkipListMap<K, V>) -> Keys<'a, K, V> {
        Keys(Iter::new(list))
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next();
        next.map(|kv| kv.0)
    }
}

pub struct Values<'a, K: 'a, V: 'a>(Iter<'a, K, V>);

impl<'a, K, V> Values<'a, K, V> {
    pub fn new(list: &'a SkipListMap<K, V>) -> Values<'a, K, V> {
        Values(Iter::new(list))
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next();
        next.map(|kv| kv.1)
    }
}

pub struct ValuesMut<'a, K: 'a, V: 'a>(IterMut<'a, K, V>);

impl<'a, K, V> ValuesMut<'a, K, V> {
    pub fn new(list: &'a mut SkipListMap<K, V>) -> ValuesMut<'a, K, V> {
        ValuesMut(IterMut::new(list))
    }
}

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next();
        next.map(|kv| kv.1)
    }
}

pub struct Range<'a, K: 'a, V: 'a> {
    /// `current_` is inclusive. We will keep on iterating until `current_` is `None`.
    current_: Option<&'a Node<K, V>>,
    /// `end_` is inclusive. If `None`, the end is considered to be unbounded. Otherwise, the key
    /// in the contained `Node<K, V>` is the maximum.
    end_: Option<&'a Node<K, V>>,
}

impl<'a, K: 'a + Ord, V: 'a> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let key_value = self.current_.map(|node| node.key_value());

        self.current_ = self.current_.and_then(
            |node|
                node.next(0).and_then(|next|
                    match self.end_ {
                        None => Some(next),
                        Some(end) => {
                            if likely!(next.key() <= end.key()) {
                                Some(next)
                            } else {
                                None
                            }
                        }
                    }
                ));

        key_value
    }
}

impl<'a, K: 'a + Ord, V: 'a> Range<'a, K, V> {
    pub fn new<T, R>(list: &SkipListMap<K, V>, range: R) -> Range<K, V>
    where
        K: Borrow<T>,
        R: RangeArgument<T>,
        T: Ord + ?Sized,
    {
        let lower_bound = match range.start() {
            Bound::Included(key) => list.find_lower_bound(key).next(0),
            Bound::Excluded(key) => {
                list.find_lower_bound(key).next(0).and_then(
                    |next|
                    if next.key() == key {
                        next.next(0)
                    } else {
                        Some(next)
                    },
                )
            }
            Bound::Unbounded => unsafe { (*list.head_).next(0) },
        };

        let upper_bound = match range.end() {
            Bound::Included(key) => list.find_lower_bound(key).next(0),
            Bound::Excluded(key) => Some(list.find_lower_bound(key)),
            Bound::Unbounded => None,
        };

        Range {
            current_: lower_bound,
            end_: upper_bound,
        }
    }
}

impl<K, V> SkipListMap<K, V> {
    pub fn iter(&self) -> Iter<K, V> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut::new(self)
    }

    pub fn keys(&self) -> Keys<K, V> {
        Keys::new(self)
    }

    pub fn values(&self) -> Values<K, V> {
        Values::new(self)
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut::new(self)
    }
}

impl<K: Ord, V> SkipListMap<K, V> {
    pub fn range<T, R>(&self, range: R) -> Range<K, V>
    where
        K: Borrow<T>,
        R: RangeArgument<T>,
        T: Ord + ?Sized,
    {
        Range::new(self, range)
    }

    pub fn range_mut<T, R>(&mut self, _range: R) -> std::collections::btree_map::RangeMut<K, V>
    where
        K: Borrow<T>,
        R: RangeArgument<T>,
        T: Ord + ?Sized,
    {
        unimplemented!()
    }
}

// TODO: size hint
// TODO: first, last, binary_search

#[cfg(test)]
mod tests {}
