extern crate skiplist;
use skiplist::*;

extern crate rand;

#[test]
fn new() {
    let list: SkipListMap<i32, i32> = Default::default();
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

#[test]
fn clear_single() {
    let key = 34;
    let value = 9484;
    let mut list: SkipListMap<i32, i32> = Default::default();
    assert!(list.insert(key, value).is_none());
    assert_eq!(list.len(), 1);
    list.clear();
    assert_eq!(list.len(), 0);
    assert!(!list.contains_key(&key));
}

#[test]
fn clear_does_not_invalidate() {
    let mut list: SkipListMap<usize, usize> = Default::default();

    for i in 0..10 {
        assert_eq!(list.len(), i);
        assert!(list.insert(i, i + 1).is_none());
        assert!(list.insert(i, i + 1).is_some());
    }

    assert_eq!(list.len(), 10);
    list.clear();
    assert_eq!(list.len(), 0);

    for i in 0..10 {
        assert_eq!(list.len(), i);
        assert!(!list.contains_key(&i));
        assert!(list.insert(i, i + 1).is_none());
    }

    assert_eq!(list.len(), 10);
    list.clear();
    assert_eq!(list.len(), 0);

    for i in 0..10 {
        assert!(list.remove(&i).is_none());
        assert_eq!(list.len(), 0);
    }
}

#[test]
fn insert_get_single() {
    let key = 34;
    let value = 433;
    let mut list: SkipListMap<i32, i32> = Default::default();
    assert!(list.insert(key, value).is_none());
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
    let value = 555;
    let mut list: SkipListMap<i32, i32> = Default::default();

    {
        assert!(list.insert(key, value).is_none());
        let first_fetched = list.get(&key);
        assert!(first_fetched.is_some());
        // This is value comparison. The key should be the same as the one inserted
        assert_eq!(*first_fetched.unwrap(), value);
    }

    // The second insertion should fail, the key is already there
    assert!(list.insert(key, value).is_some());
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
    let value_1 = 9383;
    let key_2 = 555;
    let value_2 = 9848;
    let mut list: SkipListMap<i32, i32> = Default::default();
    assert_eq!(list.len(), 0);

    assert!(list.insert(key_1, value_1).is_none());
    assert_eq!(list.len(), 1);
    assert!(list.contains_key(&key_1));
    assert!(!list.contains_key(&key_2));

    assert!(list.insert(key_2, value_2).is_none());
    assert_eq!(list.len(), 2);
    assert!(list.contains_key(&key_1));
    assert!(list.contains_key(&key_2));

    assert!(list.remove(&key_1).is_some());
    assert_eq!(list.len(), 1);
    assert!(!list.contains_key(&key_1));
    assert!(list.contains_key(&key_2));

    assert!(list.insert(key_1, value_1).is_none());
    assert_eq!(list.len(), 2);
    assert!(list.contains_key(&key_1));
    assert!(list.contains_key(&key_2));

    assert!(list.remove(&key_2).is_some());
    assert_eq!(list.len(), 1);
    assert!(list.contains_key(&key_1));
    assert!(!list.contains_key(&key_2));

    assert!(list.remove(&key_1).is_some());
    assert_eq!(list.len(), 0);
    assert!(!list.contains_key(&key_1));
    assert!(!list.contains_key(&key_2));
}

#[test]
fn remove_empty() {
    let mut list: SkipListMap<i32, i32> = Default::default();
    assert!(list.is_empty());
    assert!(list.remove(&3).is_none());
    assert_eq!(list.len(), 0);
    assert!(list.remove(&32).is_none());
    assert_eq!(list.len(), 0);
    assert!(list.remove(&22).is_none());
    assert_eq!(list.len(), 0);
}

#[test]
fn remove_single() {
    let key = 12;
    let value = 833;
    let mut list: SkipListMap<i32, i32> = Default::default();

    assert!(list.insert(key, value).is_none());
    assert_eq!(list.len(), 1);
    assert!(list.contains_key(&key));

    assert!(list.remove(&key).is_some());
    assert_eq!(list.len(), 0);
    assert!(!list.contains_key(&key));

    assert!(list.remove(&key).is_none());
}

#[test]
fn random_insert_remove() {
    use self::rand::Rng;
    let mut rng = self::rand::thread_rng();

    let mut list: SkipListMap<u32, u32> = Default::default();
    let mut inserted = std::collections::BTreeSet::new();

    let mut elements = 0;
    for _i in 0..1000 {
        let element = rng.next_u32();
        assert_eq!(list.len(), elements);

        assert!(list.insert(element, element + 1).is_none());
        assert!(list.contains_key(&element));

        inserted.insert(element);
        elements += 1;
    }

    for element in &inserted {
        assert_eq!(list.len(), elements);

        assert!(list.contains_key(element));
        assert!(list.insert(*element, element + 2).is_some());

        if rng.next_u32() % 2 == 0 {
            assert!(list.remove(element).is_some());
            assert!(!list.contains_key(element));
            elements -= 1;
        }
    }
}

#[test]
fn format_empty() {
    let list: SkipListMap<u32, u32> = Default::default();
    assert_eq!(format!("{}", list), "[]");
}

#[test]
fn format_singleton() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(1, 6);
    assert_eq!(format!("{}", list), "[1: 6]");
}

#[test]
fn format_two() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(1, 4);
    list.insert(2, 6);
    assert_eq!(format!("{}", list), "[1: 4, 2: 6]");
}

#[test]
fn format_multiple() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(1, 2);
    list.insert(2, 3);
    list.insert(3, 4);
    list.insert(4, 5);
    list.insert(5, 6);
    list.insert(6, 1);
    assert_eq!(format!("{}", list), "[1: 2, 2: 3, 3: 4, 4: 5, 5: 6, 6: 1]")
}

#[test]
#[should_panic]
fn index_empty() {
    let list: SkipListMap<u32, u32> = Default::default();
    list[&23];
}

#[test]
fn index_singleton() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(32, 12);
    assert_eq!(list[&32], 12);
}

#[test]
#[should_panic]
fn index_singleton_nonexistant() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(32, 43);
    list[&23];
}

#[test]
fn index_multiple() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(3, 3);
    list.insert(2, 2);
    list.insert(6, 6);
    list.insert(1, 1);
    list.insert(5, 5);
    list.insert(4, 4);
    assert_eq!(list[&6], 6);
}

#[test]
#[should_panic]
fn index_multiple_nonexistant() {
    let mut list: SkipListMap<u32, u32> = Default::default();
    list.insert(3, 6);
    list.insert(2, 7);
    list.insert(6, 10);
    list.insert(1, 231);
    list.insert(5, 154);
    list.insert(4, 6565);
    list[&23];
}
