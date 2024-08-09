# `hashmore` | More useful hash-based primitive data structures for Rust

This crate contains some simple primitives for working with hash-based
data structures that come up often in practice.

> [!IMPORTANT]
> Status: **Experimental**

## Primitives

### `FIFOMap`

A `FIFOMap` is a map that evicts the oldest key-value pair when it reaches a
certain size. This is useful for implementing caches and other data structures
where you want to limit the size of the data structure.

The implementation is based on a `HashMap` wrapped with a ring buffer (`VecDeque`)
to keep track of the order of insertion of the keys. When the map reaches its
capacity, the oldest key-value pair is removed from the map and the ring buffer.

```rust
use hashmore::FIFOMap;

let mut map = FIFOMap::new(2);

map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

assert_eq!(map.get("a"), None);
assert_eq!(map.get("b"), 2);
assert_eq!(map.get("c"), 3);
```

There is an equivalent set data structure called `FIFOSet` which has the
same API as `FIFOMap` but without the values.

### `LRUMap`

TODO

## License

This project is licensed under the MIT license.
