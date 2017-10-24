use node::Node;
use skiplist::SkipList;

pub struct Iter<'a, K: 'a, V: 'a> {
    current_: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(crate) fn new(list: &'a SkipList<K, V>) -> Iter<'a, K, V> {
        Iter { current_: unsafe { (*list.head_).next(0) } }
    }
}

impl<K, V> SkipList<K, V> {
    pub fn iter(&self) -> Iter<K, V> {
        Iter::new(self)
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: prefetch, likely
        let key_value = self.current_.map(|node| (node.key(), node.value()));
        self.current_ = self.current_.and_then(|node| node.next(0));
        key_value
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
        let list: SkipList<i32, i32> = Default::default();
        let mut iter = list.iter();
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_single() {
        let key = 55;
        let value = 231;
        let mut list: SkipList<i32, i32> = Default::default();
        list.insert(key, value);
        let mut iter = list.iter();

        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().0, &key);
        assert_eq!(first.unwrap().1, &value);
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_two() {
        let key_1 = 55;
        let value_1 = 12312;
        let key_2 = 687;
        let value_2 = 49548;

        let mut list: SkipList<i32, i32> = Default::default();
        list.insert(key_1, value_1);
        list.insert(key_2, value_2);
        let mut iter = list.iter();

        let first = iter.next();
        assert!(first.is_some());
        let first_unwrap = first.unwrap();
        assert_eq!(first_unwrap.0, &key_1);
        assert_eq!(first_unwrap.1, &value_1);

        let second = iter.next();
        assert!(second.is_some());
        let second_unwrap = second.unwrap();
        assert_eq!(second_unwrap.0, &key_2);
        assert_eq!(second_unwrap.1, &value_2);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_in_order() {
        use self::rand::Rng;
        let mut rng = self::rand::thread_rng();

        let mut list: SkipList<u32, u32> = Default::default();
        let mut iteration_order = std::collections::BTreeSet::new();

        for _i in 0..1000 {
            let element = rng.next_u32();
            list.insert(element, element + 1);
            iteration_order.insert(element);
        }

        assert_eq!(list.len(), iteration_order.len());
        let mut number_of_elements_iterated = 0;
        for (skiplist_element, set_element) in list.iter().zip(list.iter()) {
            assert_eq!(skiplist_element.0 + 1, *skiplist_element.1);
            //TODO: assert_eq!(skiplist_element.1, set_element + 1);
            number_of_elements_iterated += 1;
        }
        assert_eq!(number_of_elements_iterated, 1000);
    }
}
