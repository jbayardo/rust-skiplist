extern crate skiplist;
use skiplist::*;

extern crate rand;

#[test]
fn iter_empty() {
    let list: SkipListMap<i32, i32> = Default::default();
    let mut iter = list.iter();
    assert!(iter.next().is_none());
}

#[test]
fn iter_single() {
    let key = 55;
    let value = 231;
    let mut list: SkipListMap<i32, i32> = Default::default();
    list.insert(key, value);
    let mut iter = list.iter();

    let first = iter.next().unwrap();
    assert_eq!(first.0, &key);
    assert_eq!(first.1, &value);
    assert!(iter.next().is_none());
}

#[test]
fn iter_two() {
    let key_1 = 55;
    let value_1 = 112;
    let key_2 = 687;
    let value_2 = 448;

    let mut list: SkipListMap<i32, i32> = Default::default();
    list.insert(key_1, value_1);
    list.insert(key_2, value_2);
    let mut iter = list.iter();

    let first = iter.next().unwrap();
    assert_eq!(first.0, &key_1);
    assert_eq!(first.1, &value_1);

    let second = iter.next().unwrap();
    assert_eq!(second.0, &key_2);
    assert_eq!(second.1, &value_2);

    assert!(iter.next().is_none());
}

#[test]
fn iter_in_order() {
    use self::rand::Rng;
    let mut rng = self::rand::thread_rng();

    let mut list: SkipListMap<u32, u32> = Default::default();
    let mut iteration_order = std::collections::BTreeSet::new();

    for _i in 0..1000 {
        let element = rng.next_u32();
        list.insert(element, element + 1);
        iteration_order.insert(element);
    }

    assert_eq!(list.len(), iteration_order.len());
    let mut number_of_elements_iterated = 0;
    for ((key, value), set_element) in list.iter().zip(list.iter()) {
        assert_eq!(key + 1, *value);
        assert_eq!(set_element.0 + 1, *value);
        number_of_elements_iterated += 1;
    }
    assert_eq!(number_of_elements_iterated, 1000);
}
