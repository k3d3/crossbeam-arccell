#![feature(test)]

extern crate crossbeam_stm;
extern crate parking_lot;
extern crate test;

use test::{black_box, Bencher};
use crossbeam_stm::Stm;
use std::ops::Deref;

#[bench]
fn cb_stm_load(b: &mut Bencher) {
    let stm = Stm::new(vec![1, 2, 3]);
    b.iter(|| {
        let a = stm.load();
        black_box(a.deref());
    });
}

#[bench]
fn cb_stm_update(b: &mut Bencher) {
    let stm = Stm::new(vec![1, 2, 3]);
    b.iter(|| stm.update(|old| old.clone()));
}

#[bench]
fn cb_stm_update_no_reclaim(b: &mut Bencher) {
    let stm = Stm::new(vec![1, 2, 3]);
    b.iter(|| stm.update_no_reclaim(|old| old.clone()));
}

#[bench]
fn cb_stm_set(b: &mut Bencher) {
    let stm = Stm::new(vec![1, 2, 3]);
    b.iter(|| stm.set(vec![1, 2, 4]));
}

#[bench]
fn cb_stm_set_no_reclaim(b: &mut Bencher) {
    let stm = Stm::new(vec![1, 2, 3]);
    b.iter(|| stm.set_no_reclaim(vec![1, 2, 4]));
}

#[bench]
fn rwlock_load(b: &mut Bencher) {
    use std::sync::RwLock;
    let lock = RwLock::new(vec![1, 2, 3]);
    b.iter(|| {
        let a = lock.read().unwrap();
        black_box(a.deref());
    })
}

#[bench]
fn rwlock_update(b: &mut Bencher) {
    use std::sync::RwLock;
    let lock = RwLock::new(vec![1, 2, 3]);
    b.iter(|| {
        let a = lock.write().unwrap();
        black_box(a.deref());
    })
}

#[bench]
fn pl_rwlock_load(b: &mut Bencher) {
    use parking_lot::RwLock;
    let lock = RwLock::new(vec![1, 2, 3]);
    b.iter(|| {
        let a = lock.read();
        black_box(a.deref());
    })
}

#[bench]
fn pl_rwlock_update(b: &mut Bencher) {
    use parking_lot::RwLock;
    let lock = RwLock::new(vec![1, 2, 3]);
    b.iter(|| {
        let a = lock.write();
        black_box(a.deref());
    })
}

#[bench]
fn arc_load(b: &mut Bencher) {
    use std::sync::Arc;
    let arc = Arc::new(vec![1, 2, 3]);
    b.iter(|| {
        let a = arc.clone();
        black_box(a.deref());
    })
}
