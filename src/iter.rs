use node::Node;
use map::SkipListMap;

pub struct Iter<'a, K: 'a, V: 'a> {
    current_: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub fn new(list: &'a SkipListMap<K, V>) -> Iter<'a, K, V> {
        Iter { current_: unsafe { (*list.head_).next(0) } }
    }
}

impl<K, V> SkipListMap<K, V> {
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
mod tests {}
