# Crossbeam-STM

Crossbeam-STM is a Software Transactional Memory implementation using crossbeam-epoch for memory reclamation.
It is meant to be as fast and consistent as possible for load speed, at the expense of having
inconsistent-timed and potentially very slow writes.

_*THIS PROJECT IS NOT READY FOR GENERAL USAGE.*_


## Example

```rust
extern crate cb_stm_temp;

use cb_stm_temp::Stm;

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
