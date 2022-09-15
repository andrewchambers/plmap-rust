# plmap

Parallel pipelined map over iterators for rust.

## Documentation

[docs.rs/plmap](https://docs.rs/plmap/)

## Example

Parallel pipelined mapping:

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

Map with your own type instead of a function:

```
use plmap::{Mapper, PipelineMap};

// The type must support clone as each worker thread gets a copy.
#[derive(Clone)]
struct CustomMapper {}

impl Mapper<i32> for CustomMapper {
    type Out = i64;
    fn apply(&mut self, x: i32) -> i64 {
        (x * 2) as i64
    }
}

fn custom_mapper() {
    for i in (0..100).plmap(5, CustomMapper{}) {
        println!("i={}", i);
    }
}
```