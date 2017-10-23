use node::Node;
use skiplist::SkipList;

pub struct Iter<'a, K: 'a> {
    current_: Option<&'a Node<K>>,
}

impl<'a, K> Iter<'a, K> {
    pub(crate) fn new(list: &'a SkipList<K>) -> Iter<'a, K> {
        Iter { current_: unsafe { (*list.head_).next(0) } }
    }
}

impl<K> SkipList<K> {
    pub fn iter(&self) -> Iter<K> {
        Iter::new(self)
    }
}

impl<'a, K: 'a> Iterator for Iter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: prefetch, likely
        let key = self.current_.map(|node| node.key());
        self.current_ = self.current_.and_then(|node| node.next(0));
        key
    }
}

// TODO: size hint
// TODO: first, last, binary_search

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use std;

    #[test]
    fn iter_empty() {
        let list: SkipList<i32> = Default::default();
        let mut iter = list.iter();
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_single() {
        let key = 55;
        let mut list: SkipList<i32> = Default::default();
        list.insert(key);
        let mut iter = list.iter();

        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap(), &key);
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_two() {
        let key_1 = 55;
        let key_2 = 687;
        let mut list: SkipList<i32> = Default::default();
        list.insert(key_1);
        list.insert(key_2);
        let mut iter = list.iter();

        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap(), &key_1);

        let second = iter.next();
        assert!(second.is_some());
        assert_eq!(second.unwrap(), &key_2);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_in_order() {
        use self::rand::Rng;
        let mut rng = self::rand::thread_rng();

        let mut list: SkipList<u32> = Default::default();
        let mut iteration_order = std::collections::BTreeSet::new();

        for _i in 0..1000 {
            let element = rng.next_u32();
            list.insert(element);
            iteration_order.insert(element);
        }

        assert_eq!(list.len(), iteration_order.len());
        let mut number_of_elements_iterated = 0;
        for (skiplist_element, set_element) in list.iter().zip(list.iter()) {
            assert_eq!(skiplist_element, set_element);
            number_of_elements_iterated += 1;
        }
        assert_eq!(number_of_elements_iterated, 1000);
    }
}
