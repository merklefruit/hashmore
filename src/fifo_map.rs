use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use std::{cell::RefCell, hash::Hash, num::NonZeroUsize, rc::Rc};

use crate::common::{Link, Node, NodeRef};

/// A First-In-First-Out (FIFO) map.
///
/// This hashmap has a fixed, pre-allocated capacity and will remove the oldest
/// entry when the capacity is reached and a new entry is inserted. This is useful
/// for implementing a cache with a fixed size to prevent it from growing indefinitely.
///
/// It is implemented with a doubly linked list that keeps track of the oldest and newest
/// entries and a hashmap that maps keys to values and the corresponding linked list pointer.
///
/// # Example
///
/// ```rust
/// use fifo_map::FIFOMap;
///
/// let mut map = FIFOMap::with_capacity(3);
///
/// map.insert("a", 1);
/// map.insert("b", 2);
/// map.insert("c", 3);
///
/// assert_eq!(map.get(&"a"), Some(&1));
/// assert_eq!(map.get(&"b"), Some(&2));
/// assert_eq!(map.get(&"c"), Some(&3));
///
/// map.insert("d", 4);
///
/// // now "a" is removed because it was the oldest entry
/// assert_eq!(map.get(&"a"), None);
/// assert_eq!(map.get(&"b"), Some(&2));
/// ```
#[derive(Debug)]
pub struct FIFOMap<K, V, S = DefaultHashBuilder> {
    map: HashMap<K, (V, NodeRef<K>), S>,
    head: Link<K>,
    tail: Link<K>,
    cap: NonZeroUsize,
}

impl<K, V> FIFOMap<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new FIFO map with the given capacity.
    /// The capacity must be greater than zero.
    ///
    /// # Panics
    ///
    /// Panics if the capacity is zero.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).expect("FIFOMap capacity must be non-zero");
        Self { map: HashMap::with_capacity(capacity), head: None, tail: None, cap }
    }
}

impl<K, V> FIFOMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Inserts a new key-value pair into the map.
    /// - If the map is at capacity, the oldest entry will be removed.
    /// - If the key is already in the map, the value will be updated.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        if self.map.len() == self.cap.get() {
            self.remove_first();
        }

        let new_node = Node { key: key.clone(), next: None, prev: self.tail.clone() };
        let new_node_ref = Rc::new(RefCell::new(new_node));

        if let Some(tail) = self.tail.take() {
            tail.borrow_mut().next = Some(new_node_ref.clone());
        }
        self.tail = Some(new_node_ref.clone());

        if self.head.is_none() {
            self.head = Some(new_node_ref.clone());
        }

        self.map.insert(key, (value, new_node_ref));
    }

    /// Removes a key-value pair from the map and returns the value.
    /// If the key is not in the map, `None` is returned.
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key).map(|(v, node)| {
            if let Some(prev) = node.borrow().prev.clone() {
                prev.borrow_mut().next.clone_from(&node.borrow().next)
            } else {
                self.head.clone_from(&node.borrow().next)
            }

            if let Some(next) = node.borrow().next.clone() {
                next.borrow_mut().prev.clone_from(&node.borrow().prev)
            } else {
                self.tail.clone_from(&node.borrow().prev);
            }

            v
        })
    }

    /// Returns the number of key-value pairs currently in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the capacity of the map.
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.cap.get()
    }

    /// An iterator visiting all keys in insertion order.
    /// The keys are returned by reference.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.keys()
    }

    /// An iterator visiting all values in insertion order.
    /// The values are returned by reference.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.values().map(|(v, _)| v)
    }

    /// An iterator visiting all key-value pairs in insertion order.
    /// The key-value pairs are returned by reference.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter().map(|(k, (v, _))| (k, v))
    }

    /// Returns a reference to the value corresponding to the key.
    /// - If the key is not in the map, `None` is returned.
    /// - The key-value pair is not removed from the map.
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|(v, _)| v)
    }

    /// Checks if the map contains the given key.
    /// - Returns `true` if the key is in the map, `false` otherwise.
    /// - The key is not removed from the map.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Removes the oldest entry from the map.
    /// If the map is empty, this is a no-op.
    fn remove_first(&mut self) {
        if let Some(head) = self.head.take() {
            if let Some(next) = head.borrow_mut().next.take() {
                next.borrow_mut().prev = None;
                self.head = Some(next);
            } else {
                self.tail.take();
            }
            let key = &head.borrow().key;
            self.map.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fifo_map::FIFOMap;

    #[test]
    fn test_fifo_map_reach_cap() {
        let mut map = FIFOMap::with_capacity(3);

        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        assert_eq!(map.get(&"a"), Some(&1));
        assert_eq!(map.get(&"b"), Some(&2));
        assert_eq!(map.get(&"c"), Some(&3));

        map.insert("d", 4);

        assert_eq!(map.get(&"a"), None);
        assert_eq!(map.get(&"b"), Some(&2));
    }

    #[test]
    fn test_fifo_map_replace_many() {
        let mut map = FIFOMap::with_capacity(3);

        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        assert_eq!(map.get(&"a"), Some(&1));
        assert_eq!(map.get(&"b"), Some(&2));
        assert_eq!(map.get(&"c"), Some(&3));

        map.insert("d", 4);

        assert_eq!(map.get(&"a"), None);
        assert_eq!(map.get(&"b"), Some(&2));

        map.insert("e", 5);
        map.insert("f", 6);

        assert_eq!(map.get(&"b"), None);
        assert_eq!(map.get(&"c"), None);
        assert_eq!(map.get(&"d"), Some(&4));
        assert_eq!(map.get(&"e"), Some(&5));
        assert_eq!(map.get(&"f"), Some(&6));
    }

    #[test]
    #[should_panic]
    fn test_fifo_map_zero_capacity() {
        FIFOMap::<u64, u64>::with_capacity(0);
    }

    #[test]
    fn test_fifo_map_remove() {
        let mut map = FIFOMap::with_capacity(3);

        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        assert_eq!(map.remove(&"a"), Some(1));
        assert_eq!(map.remove(&"b"), Some(2));
        assert_eq!(map.remove(&"c"), Some(3));
        assert_eq!(map.remove(&"d"), None);
        assert_eq!(map.get(&"a"), None);
        assert_eq!(map.get(&"b"), None);
        assert_eq!(map.get(&"c"), None);
    }
}
