# Crossbeam-STM

Crossbeam-STM is a Software Transactional Memory implementation using crossbeam-epoch for memory reclamation.
It is meant to be as fast and consistent as possible for load speed, at the expense of having
inconsistent-timed and potentially very slow writes.

_*THIS PROJECT IS NOT READY FOR GENERAL USAGE.*_


## Example

```rust
extern crate crossbeam_stm;

use crossbeam_stm::Stm;

// Create a new STM pointer with a Vec of numbers
let stm = Stm::new(Some(vec![1,2,3,4]);

// Read from the STM
{
    let guard = stm.guard()
    let data = guard.load()
    println!("Current STM: {:?}", data);
}

// Update the STM pointer to add a new number
stm.update(|value| {
    // Since value is an Option, map over it
    value.map(|data| {
        let mut data = data.clone();
        data.push(5);
        data
    }
});

// Read the new data
{
    let guard = stm.guard()
    let data = guard.load()
    println!("Current STM: {:?}", data);
}

```