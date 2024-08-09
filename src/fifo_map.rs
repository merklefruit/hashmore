use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use std::{
    collections::VecDeque,
    hash::{BuildHasher, Hash},
    num::NonZeroUsize,
};

/// A First-In-First-Out (FIFO) map.
///
/// This hashmap has a fixed, pre-allocated capacity and will remove the oldest
/// entry when the capacity is reached and a new entry is inserted. This is useful
/// for implementing a cache with a fixed size to prevent it from growing indefinitely.
///
/// # Example
///
/// ```rust
/// use hashmore::FIFOMap;
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
    map: HashMap<K, V, S>,
    order: VecDeque<K>,
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
        Self {
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            cap: NonZeroUsize::new(capacity).expect("FIFOMap capacity must be non-zero"),
        }
    }
}

impl<K, V, S> FIFOMap<K, V, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher,
{
    /// Inserts a new key-value pair into the map.
    /// - If the map is at capacity, the oldest entry will be removed.
    /// - If the key is already in the map, the value will be updated.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            // If the key already exists, update the value and do not change order.
            self.map.insert(key.clone(), value);
        } else {
            if self.map.len() == self.cap.get() {
                // Evict the oldest item
                if let Some(old_key) = self.order.pop_front() {
                    self.map.remove(&old_key);
                }
            }
            self.map.insert(key.clone(), value);
            self.order.push_back(key);
        }
    }

    /// Removes a key-value pair from the map and returns the value.
    /// If the key is not in the map, `None` is returned.
    ///
    /// Note: This operation is O(n) worst case, because it needs to find the key
    /// in the VecDeque. If you need to remove many items (especially newly-inserted
    /// ones), consider using a different data structure.
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Remove the key from the HashMap and get the value
        if let Some(value) = self.map.remove(key) {
            // Remove the key from the VecDeque
            if let Some(pos) = self.order.iter().position(|x| x == key) {
                self.order.remove(pos);
            }
            Some(value)
        } else {
            None
        }
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
        self.map.values()
    }

    /// An iterator visiting all key-value pairs in insertion order.
    /// The key-value pairs are returned by reference.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }

    /// Returns a reference to the value corresponding to the key.
    /// - If the key is not in the map, `None` is returned.
    /// - The key-value pair is not removed from the map.
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    /// Checks if the map contains the given key.
    /// - Returns `true` if the key is in the map, `false` otherwise.
    /// - The key is not removed from the map.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
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
