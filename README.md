# plmap

Parallel pipelined map over iterators for rust.

# Example

```
// Import the iterator extension trait.
use plmap::PipelineMap;

// Map over an iterator in parallel with 5 worker threads.
fn example() {
    for i in (0..100).plmap(5, |x| x * 2) {
        println!("i={}", i);
    }
}
```