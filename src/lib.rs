//! Hashmore: A collection of hash-based primitive data structures for Rust.
//!
//! Available data structures:
//! - FIFOMap
//! - FIFOSet
//! - More to come!

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    clippy::missing_const_for_fn,
    clippy::missing_inline_in_public_items,
    clippy::all,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![allow(
    clippy::cast_lossless,
    clippy::inline_always,
    clippy::let_unit_value,
    clippy::must_use_candidate,
    clippy::wildcard_imports,
    unsafe_op_in_unsafe_fn,
    unused_unsafe
)]

mod fifo_map;
pub use fifo_map::FIFOMap;

mod fifo_set;
pub use fifo_set::FIFOSet;
