//extern crate skiplist;
//use skiplist::*;

//#[test]
//fn insert_and_iterate() {
//    let gn = Box::new(GeometricalGenerator::new(4, 0.5));
//    let mut sk: SkipList<i32> = SkipList::new(gn);
//
//    println!("{}", sk);
//
//    let mut k: i32 = 5;
//    while k >= 0 {
//        println!("Inserting element {}", k);
//        sk.insert(k);
//        println!("{}", sk);
//        k -= 1;
//    }
//
//    let k: i32 = 3;
//    println!("{:?}", sk.get(&k));
//    println!("{:?}", sk.remove(&k));
//    println!("{:?}", sk.get(&k));
//    println!("{}", sk);
//}
