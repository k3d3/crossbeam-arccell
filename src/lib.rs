extern crate crossbeam_epoch;

use crossbeam_epoch::Atomic;
use crossbeam_epoch::Shared;
use std::sync::atomic::Ordering;
use crossbeam_epoch::Owned;

struct Stm<T> {
    inner: Atomic<T>
}

impl<T> Stm<T> {
    fn new(data: Option<T>) -> Stm<T> {
        let inner = match data {
            Some(t) => Atomic::new(t),
            None => Atomic::null()
        };
        Stm { inner }
    }

    fn update<F>(&self, f: F)
    where
        F: Fn(Option<&T>) -> Option<T> {

        let guard = crossbeam_epoch::pin();
        loop {
            let shared = self.inner.load_consume(&guard);
            let data = unsafe { shared.as_ref() };
            match f(data) {
                Some(t) => {
                    let r = self.inner.compare_and_set(
                        shared,
                        Owned::new(t),
                        Ordering::AcqRel,
                        &guard
                    );
                    if r.is_ok() { break }
                },
                None => {
                    let r = self.inner.compare_and_set(
                        shared,
                        Shared::null(),
                        Ordering::AcqRel,
                        &guard
                    );
                    if r.is_ok() { break }
                }
            };
        }
    }

    fn guard(&self) -> Guard<T> {
        Guard { parent: self, inner: crossbeam_epoch::pin() }
    }
}

struct Guard<'a, T: 'a> {
    parent: &'a Stm<T>,
    inner: crossbeam_epoch::Guard
}

impl<'a, T> Guard<'a, T> {
    fn load<'g>(&'g self) -> Option<&'g T> {
        let shared = self.parent.inner.load_consume(&self.inner);
        unsafe { shared.as_ref() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stm_test() {
        let stm = Stm::new(Some(vec![1,2,3]));
        {
            let guard = stm.guard();
            let data = guard.load();
            println!("{:?}", data);
        }

        stm.update(|v| {
            v.map(|d| {
                let mut d = d.clone();
                d.push(4);
                d
            })
        });

        {
            let guard = stm.guard();
            let data = guard.load();
            println!("{:?}", data);
        }

        stm.update(|_| None);

        {
            let guard = stm.guard();
            let data = guard.load();
            println!("{:?}", data);
        }
    }
}
