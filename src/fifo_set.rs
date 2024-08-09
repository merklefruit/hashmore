use std::{
    cell::RefCell,
    hash::{BuildHasher, Hash},
    num::NonZeroUsize,
    rc::Rc,
};

use hashbrown::{hash_map::DefaultHashBuilder, HashSet};

use crate::common::{Link, Node};

/// A First-In-First-Out (FIFO) set.
///
/// This set has a fixed, pre-allocated capacity and will remove the oldest
/// entry when the capacity is reached and a new entry is inserted. This is useful
/// for implementing a cache with a fixed size to prevent it from growing indefinitely.
///
/// It is implemented with a doubly linked list that keeps track of the oldest and newest
/// entries and a hashset that maps keys to the linked list.
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
    head: Link<K>,
    tail: Link<K>,
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
        let cap = NonZeroUsize::new(capacity).expect("FIFOSet capacity must be non-zero");
        Self { set: HashSet::with_capacity(capacity), head: None, tail: None, cap }
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
        if self.set.len() == self.cap.get() {
            self.remove_first();
        }

        if self.set.contains(&key) {
            return;
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

        self.set.insert(key);
    }

    /// Removes a key from the set.
    /// Returns `true` if the key was in the set and was
    /// removed, `false` otherwise.
    #[inline]
    pub fn remove(&mut self, key: &K) -> bool {
        if !self.set.remove(key) {
            return false;
        }

        let mut current = self.head.clone();
        while let Some(node) = current {
            let next = node.borrow().next.clone();
            if node.borrow().key == *key {
                if let Some(prev) = node.borrow().prev.clone() {
                    prev.borrow_mut().next.clone_from(&next)
                } else {
                    self.head.clone_from(&next)
                }
                if let Some(next) = next.clone() {
                    next.borrow_mut().prev.clone_from(&node.borrow().prev);
                } else {
                    self.tail.clone_from(&node.borrow().prev);
                }
                return true;
            }
            current = next;
        }

        false
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

    fn remove_first(&mut self) {
        if let Some(head) = self.head.take() {
            if let Some(next) = head.borrow_mut().next.take() {
                next.borrow_mut().prev = None;
                self.head = Some(next);
            } else {
                self.tail.take();
            }
            let key = head.borrow().key.clone();
            self.set.remove(&key);
        }
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
