# Crossbeam-STM

[![crates.io](https://img.shields.io/crates/v/crossbeam-stm.svg)](https://crates.io/crates/crossbeam-stm)
[![crossbeam-stm docs](https://docs.rs/crossbeam-stm/badge.svg)](https://docs.rs/crossbeam-stm)

Crossbeam-STM is a Software Transactional Memory implementation using crossbeam-epoch for memory reclamation.
It is meant to be as fast and consistent as possible for load speed, at the expense of having
inconsistent-timed and potentially very slow writes.

The idea behind this STM is that we have an atomic pointer that is always pointing to valid data.
This data should always be an atomic pointer dereference (and a pin) away.

When the STM needs to be updated, the old value cannot be modified because it might be in use by other
threads. Instead, the existing data must be cloned to a new location and modified there. Once a new value
is ready, a compare-and-swap is performed on the atomic pointer, so all threads requesting data after
that point will receive the newly-updated data.

Loads should always be constant-time, even in the face of both load and update contention.

Updates might take a long time, and the closure passed to it might run multiple times. This is because
if the "old" value is updated before the closure finishes, the closure might overwrite up-to-date data
and must be run again with said new data passed in. Additionally, memory reclamation of old STM values
is performed at this point.

If you want to set the STM data regardless of what is currently set, you can use the `set()` method.
This call should be a lot quicker than update.

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

// Set the STM pointer
let data = vec![9,8,7,6];
stm.set(data);
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
