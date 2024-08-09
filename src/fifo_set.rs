use hashbrown::{hash_map::DefaultHashBuilder, HashSet};
use std::{
    collections::VecDeque,
    hash::{BuildHasher, Hash},
    num::NonZeroUsize,
};

/// A First-In-First-Out (FIFO) set.
///
/// This set has a fixed, pre-allocated capacity and will remove the oldest
/// entry when the capacity is reached and a new entry is inserted. This is useful
/// for implementing a cache with a fixed size to prevent it from growing indefinitely.
///
/// # Example
///
/// ```rust
/// use fifo_set::FIFOSet;
///
/// let mut set = FIFOSet::with_capacity(3);
///
/// set.insert(1);
/// set.insert(2);
/// set.insert(3);
///
/// assert!(set.contains(&1));
/// assert!(set.contains(&2));
/// assert!(set.contains(&3));
///
/// set.insert(4);
///
/// // now 1 is removed because it was the oldest entry
/// assert!(!set.contains(&1));
/// assert!(set.contains(&2));
/// assert!(set.contains(&3));
/// assert!(set.contains(&4));
/// ```
#[derive(Debug)]
pub struct FIFOSet<K, S = DefaultHashBuilder> {
    set: HashSet<K, S>,
    order: VecDeque<K>,
    cap: NonZeroUsize,
}

impl<K> FIFOSet<K>
where
    K: Eq + Hash,
{
    /// Creates a new FIFO set with the given capacity.
    /// The capacity must be greater than zero.
    ///
    /// # Panics
    ///
    /// Panics if the capacity is zero.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            set: HashSet::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            cap: NonZeroUsize::new(capacity).expect("FIFOSet capacity must be non-zero"),
        }
    }
}

impl<K, S> FIFOSet<K, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher,
{
    /// Inserts a new key into the set.
    /// - If the set is at capacity, the oldest entry will be removed.
    /// - If the key is already in the set, it will not be inserted again.
    #[inline]
    pub fn insert(&mut self, key: K) {
        if self.set.contains(&key) {
            return;
        }

        if self.set.len() == self.cap.get() {
            // Evict the oldest item
            if let Some(old_key) = self.order.pop_front() {
                self.set.remove(&old_key);
            }
        }

        self.set.insert(key.clone());
        self.order.push_back(key);
    }

    /// Removes a key from the set.
    /// Returns `true` if the key was in the set and was removed, `false` otherwise.
    ///
    /// Note: This operation is O(n) worst case, because it needs to find the key
    /// in the VecDeque. If you need to remove many items (especially newly-inserted
    /// ones), consider using a different data structure.
    #[inline]
    pub fn remove(&mut self, key: &K) -> bool {
        if self.set.remove(key) {
            // Remove the key from the VecDeque
            if let Some(pos) = self.order.iter().position(|x| x == key) {
                self.order.remove(pos);
            }
            true
        } else {
            false
        }
    }

    /// Returns the number of unique keys currently in the set.
    #[inline]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Returns `true` if the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Returns the capacity of the set.
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.cap.get()
    }

    /// An iterator visiting all keys in insertion order.
    /// The keys are returned by reference.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &K> {
        self.set.iter()
    }

    /// Checks if the set contains the given key.
    /// - Returns `true` if the key is in the set, `false` otherwise.
    /// - The key is not removed from the set.
    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        self.set.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_set() {
        let mut set = FIFOSet::with_capacity(3);

        set.insert(1);
        set.insert(2);
        set.insert(3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));

        set.insert(4);
        assert!(!set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(set.contains(&4));

        set.insert(5);
        assert!(!set.contains(&2));
        assert!(set.contains(&3));
        assert!(set.contains(&4));
        assert!(set.contains(&5));

        assert!(!set.contains(&1));
        assert!(!set.contains(&2));
    }

    #[test]
    fn test_fifo_set_remove() {
        let mut set = FIFOSet::with_capacity(3);

        set.insert(1);
        set.insert(2);
        set.insert(3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));

        set.remove(&2);
        assert!(set.contains(&1));
        assert!(!set.contains(&2));
        assert!(set.contains(&3));

        set.remove(&1);
        assert!(!set.contains(&1));
        assert!(!set.contains(&2));
        assert!(set.contains(&3));

        set.remove(&3);
        assert!(!set.contains(&1));
        assert!(!set.contains(&2));
        assert!(!set.contains(&3));
    }
}
