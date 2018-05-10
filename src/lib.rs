extern crate crossbeam_epoch;

use crossbeam_epoch::{Atomic, Owned};
use std::sync::atomic::Ordering;

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

    pub fn guard(&self) -> Guard<T> {
        Guard { parent: self, inner: crossbeam_epoch::pin() }
    }
}

pub struct Guard<'a, T: 'a> {
    parent: &'a Stm<T>,
    inner: crossbeam_epoch::Guard
}

impl<'a, T> Guard<'a, T> {
    pub fn load<'g>(&'g self) -> &'g T {
        let shared = self.parent.inner.load(Ordering::Acquire, &self.inner);
        unsafe { shared.as_ref().unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stm_test() {
        let stm = Stm::new(vec![1,2,3]);
        {
            let guard = stm.guard();
            let data = guard.load();
            println!("{:?}", data);
        }

        stm.update(|v| {
            let mut v = v.clone();
            v.push(4);
            v
        });

        {
            let guard = stm.guard();
            let data = guard.load();
            println!("{:?}", data);
        }

        stm.update(|_| vec![1]);

        {
            let guard = stm.guard();
            let data = guard.load();
            println!("{:?}", data);
        }
    }
}
