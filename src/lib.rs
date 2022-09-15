//! Parallel pipelined map over iterators.
//!
//! This crate adds the plmap and scoped_plmap functions to iterators
//! allowing easy pipelined parallelism. Because the implementation
//! uses pipelining, it preserves order, but also suffers from head of line
//! blocking.
//!
//! # Examples
//!
//! Parallel pipelined mapping:
//! ```
//! // Import the iterator extension trait.
//! use plmap::PipelineMap;
//!
//! // Map over an iterator in parallel with 5 worker threads.
//! fn example() {
//!     for i in (0..100).plmap(5, |x| x * 2) {
//!         println!("i={}", i);
//!     }
//! }
//! ```
//!
//! Scoped and parallel pipelined mapping:
//! ```
//! #[rustversion::since(1.63)]
//! use {std::thread, plmap::ScopedPipelineMap};
//!
//! #[rustversion::since(1.63)]
//! fn example() {
//!     thread::scope(|s| {
//!        // Using a thread scope let's you use non 'static lifetimes.
//!        for (i, v) in (0..100).scoped_plmap(s, 5, |x| x * 2).enumerate() {
//!             println!("i={}", i);
//!        }
//!     })
//! }
//! ```
//!
//! Map with your own type instead of a function:
//! ```
//! use plmap::{Mapper, PipelineMap};
//!
//! // The type must support clone as each worker thread gets a copy.
//! #[derive(Clone)]
//! struct CustomMapper {}
//!
//! impl Mapper<i32> for CustomMapper {
//!     type Out = i64;
//!     fn apply(&mut self, x: i32) -> i64 {
//!         (x * 2) as i64
//!     }
//! }
//!
//! fn custom_mapper() {
//!     for i in (0..100).plmap(5, CustomMapper{}) {
//!         println!("i={}", i);
//!     }
//! }
//! ```

mod mapper;
mod pipeline;
mod scoped_pipeline;

pub use mapper::*;
pub use pipeline::*;
pub use scoped_pipeline::*;
