extern crate crossbeam_epoch;

use crossbeam_epoch::{Atomic, Owned};
use std::sync::atomic::Ordering;
use std::ops::Deref;
use std::fmt;

pub struct Stm<T: 'static + Send> {
    inner: Atomic<T>,
}

impl<T: 'static + Send> Stm<T> {
    pub fn new(data: T) -> Stm<T> {
        Stm {
            inner: Atomic::new(data),
        }
    }

    pub fn update<F>(&self, f: F)
    where
        F: Fn(&T) -> T,
    {
        let guard = crossbeam_epoch::pin();
        guard.flush();
        loop {
            let shared = self.inner.load(Ordering::Acquire, &guard);
            let data = unsafe { shared.as_ref().unwrap() };
            let t = f(data);
            let r = self.inner
                .compare_and_set(shared, Owned::new(t), Ordering::AcqRel, &guard);
            if let Ok(r) = r {
                unsafe { guard.defer(move || r.into_owned()) }
                break;
            }
        }
    }

    pub fn load(&self) -> StmGuard<T> {
        StmGuard {
            parent: self,
            inner: crossbeam_epoch::pin(),
        }
    }
}

impl<T: 'static + Send + fmt::Debug> fmt::Debug for Stm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StmGuard")
            .field("data", self.load().deref())
            .finish()
    }
}

impl<T: 'static + Send + fmt::Display> fmt::Display for Stm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.load().deref().fmt(f)
    }
}

impl<T: 'static + Send> Drop for Stm<T> {
    fn drop(&mut self) {
            let guard = crossbeam_epoch::pin();
            let shared = self.inner.load(Ordering::Acquire, &guard);
            unsafe { shared.into_owned(); }
    }
}

pub struct StmGuard<'a, T: 'static + Send> {
    parent: &'a Stm<T>,
    inner: crossbeam_epoch::Guard,
}

impl<'a, T: 'static + Send> Deref for StmGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        let shared = self.parent.inner.load(Ordering::Acquire, &self.inner);
        unsafe { shared.as_ref().unwrap() }
    }
}

impl<'a, T: 'static + Send + fmt::Debug> fmt::Debug for StmGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StmGuard")
            .field("data", &self.deref())
            .finish()
    }
}

impl<'a, T: 'static + Send + fmt::Display> fmt::Display for StmGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stm_test() {
        let stm = Stm::new(vec![1, 2, 3]);
        {
            let data = stm.load();
            assert_eq!(*data, vec![1,2,3]);
        }

        stm.update(|v| {
            let mut v = v.clone();
            v.push(4);
            v
        });

        {
            let data = stm.load();
            assert_eq!(*data, vec![1,2,3,4]);
        }

        stm.update(|_| vec![1]);

        {
            let data = stm.load();
            assert_eq!(*data, vec![1]);
        }
    }

    // This test should not at all compile.
    #[test]
    fn test_nonstatic() {
        use crossbeam_epoch::*;

        struct StrRef<'a> {
            r: &'a str,
        }

        impl<'a> Drop for StrRef<'a> {
            fn drop(&mut self) {
                println!("{}", self.r);
            }
        }

        impl<'a> Clone for StrRef<'a> {
            fn clone(&self) -> StrRef<'a> {
                StrRef { r: self.r }
            }
        }

        {
            // We make a string on a local stack frame.
            let a = String::from("Local string..");
            let s = StrRef { r: &a };
            let stm = Stm::new(s);

            // We defer the string reference destruction.
            stm.update(|curr: &StrRef| curr.clone() );
        }
        println!("Frame left!");

        // On my compiler, the memory for "Local String.." will be overwritten with "PRINTMEINSTEAD".
        String::from("PRINTMEINSTEAD");

        // String reference is actually destroyed, but now it points to "PRINTMEINSTEAD"!
        pin().flush();
        pin().flush();

        // Output:
        //   Frame left!
        //   PRINTMEINSTEAD
    }

    #[test]
    fn test_no_leaks() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let count = AtomicUsize::new(0);

        struct DropCounter<'a> {
            r: &'a AtomicUsize
        }

        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                self.r.fetch_add(1, Ordering::SeqCst);
            }
        }

        drop(Stm::new(DropCounter { r: &count }));

        // We expect the value to have been dropped exactly once.
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
