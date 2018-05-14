#![feature(test)]

extern crate test;
extern crate crossbeam_stm;
extern crate parking_lot;

use test::{Bencher, black_box};
use crossbeam_stm::Stm;
use std::ops::Deref;

#[bench]
fn cb_stm_load(b: &mut Bencher) {
    let stm = Stm::new(vec![1,2,3]);
    b.iter(|| {
        let a = stm.load();
        black_box(a.deref());
    });
}

#[bench]
fn cb_stm_update(b: &mut Bencher) {
    let stm = Stm::new(vec![1,2,3]);
    b.iter(|| {
        stm.update(|old| old.clone())
    });
}

#[bench]
fn rwlock_load(b: &mut Bencher) {
    use std::sync::RwLock;
    let lock = RwLock::new(vec![1,2,3]);
    b.iter(|| {
        let a = lock.read().unwrap();
        black_box(a.deref());
    })
}

#[bench]
fn rwlock_update(b: &mut Bencher) {
    use std::sync::RwLock;
    let lock = RwLock::new(vec![1,2,3]);
    b.iter(|| {
        let a = lock.write().unwrap();
        black_box(a.deref());
    })
}

#[bench]
fn pl_rwlock_load(b: &mut Bencher) {
    use parking_lot::RwLock;
    let lock = RwLock::new(vec![1,2,3]);
    b.iter(|| {
        let a = lock.read();
        black_box(a.deref());
    })
}

#[bench]
fn pl_rwlock_update(b: &mut Bencher) {
    use parking_lot::RwLock;
    let lock = RwLock::new(vec![1,2,3]);
    b.iter(|| {
        let a = lock.write();
        black_box(a.deref());
    })
}

