extern crate crossbeam_epoch;

use crossbeam_epoch::{Atomic, Owned};
use std::sync::atomic::Ordering;
use std::ops::Deref;
use std::fmt;

pub struct Stm<T> {
    inner: Atomic<T>
}

impl<T> Stm<T> {
    pub fn new(data: T) -> Stm<T> {
        Stm { inner: Atomic::new(data) }
    }

    pub fn update<F>(&self, f: F)
    where
        F: Fn(&T) -> T {

        let guard = crossbeam_epoch::pin();
        guard.flush();
        loop {
            let shared = self.inner.load(Ordering::Acquire, &guard);
            let data = unsafe { shared.as_ref().unwrap() };
            let t = f(data);
            let r = self.inner.compare_and_set(
                shared,
                Owned::new(t),
                Ordering::AcqRel,
                &guard
            );
            if let Ok(r) = r {
                unsafe { guard.defer(move || r.into_owned()) }
                break;
            }
        }
    }

    pub fn load(&self) -> StmGuard<T> {
        StmGuard { parent: self, inner: crossbeam_epoch::pin() }
    }
}

impl<T: fmt::Debug> fmt::Debug for Stm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StmGuard").field("data", self.load().deref()).finish()
    }
}

impl<T: fmt::Display> fmt::Display for Stm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.load().deref().fmt(f)
    }
}

pub struct StmGuard<'a, T: 'a> {
    parent: &'a Stm<T>,
    inner: crossbeam_epoch::Guard
}

impl<'a, T> Deref for StmGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        let shared = self.parent.inner.load(Ordering::Acquire, &self.inner);
        unsafe { shared.as_ref().unwrap() }
    }
}


impl<'a, T: fmt::Debug> fmt::Debug for StmGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StmGuard").field("data", &self.deref()).finish()
    }
}

impl<'a, T: fmt::Display> fmt::Display for StmGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stm_test() {
        let stm = Stm::new(vec![1,2,3]);
        {
            let data = stm.load();
            println!("{:?}", data);
        }

        stm.update(|v| {
            let mut v = v.clone();
            v.push(4);
            v
        });

        {
            let data = stm.load();
            println!("{:?}", data);
        }

        stm.update(|_| vec![1]);

        {
            let data = stm.load();
            println!("{:?}", data);
        }
    }
}
