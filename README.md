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
