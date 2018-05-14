# Crossbeam-STM

Crossbeam-STM is a Software Transactional Memory implementation using crossbeam-epoch for memory reclamation.
It is meant to be as fast and consistent as possible for load speed, at the expense of having
inconsistent-timed and potentially very slow writes.

## Example

```rust
extern crate crossbeam_stm;

use crossbeam_stm::Stm;

// Create a new STM pointer with a Vec of numbers
let stm = Stm::new(vec![1,2,3,4]);

// Read from the STM
{
    let data = stm.load();
    println!("Current STM: {:?}", data);
}

// Update the STM pointer to add a new number
stm.update(|old| {
    let mut new = old.clone();
    new.push(5);
    new
});

// Read the new data
{
    let data = stm.load();
    println!("Current STM: {:?}", data);
}

```

## Benchmarks

Note that these benchmarks exist without any contention.

Under contention: 
- Crossbeam-STM's load will always be constant-time.
- Crossbeam-STM's update will slow down if multiple threads attempt to write at the same time.

The following benchmarks are available under the `benches/` directory:

```
// Crossbeam-STM
test cb_stm_load      ... bench:          12 ns/iter (+/- 0)
test cb_stm_update    ... bench:         686 ns/iter (+/- 7)

// RwLock in stdlib
test rwlock_load      ... bench:          48 ns/iter (+/- 0)
test rwlock_update    ... bench:          36 ns/iter (+/- 0)

// RwLock in parking_lot
test pl_rwlock_load   ... bench:          21 ns/iter (+/- 0)
test pl_rwlock_update ... bench:          13 ns/iter (+/- 0)
```

## License

Licensed under the terms of the MIT license and the Apache License (Version 2.0)

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
