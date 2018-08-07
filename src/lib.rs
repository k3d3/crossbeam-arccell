#![warn(missing_docs)]
//! This library provides a Software Transactional Memory structure that
//! can be used for sharing data among multiple threads in a way that is
//! safe and can be loaded quickly.
//!
//! For more information, look at the documentation for the `Stm` struct.
//!
//! # Example
//! ```
//! use crossbeam_stm::Stm;
//!
//! // Create a new STM pointer with a Vec of numbers
//! let stm = Stm::new(vec![1,2,3,4]);
//!
//! // Read from the STM
//! {
//!     let data = stm.load();
//!     println!("Current STM: {:?}", data);
//! }
//!
//! // Update the STM pointer to add a new number
//! stm.update(|old| {
//!     let mut new = old.clone();
//!     new.push(5);
//!     new
//! });
//!
//! // Read the new data
//! {
//!     let data = stm.load();
//!     println!("Current STM: {:?}", data);
//! }
//!
//! // Set the STM pointer
//! let data = vec![9,8,7,6];
//! stm.set(data);
//!
//! // Read the new data, again
//! {
//!     let data = stm.load();
//!     println!("Current STM: {:?}", data);
//! }
//! ```

extern crate crossbeam_epoch;

use crossbeam_epoch::{Atomic, Owned};
use std::sync::atomic::Ordering;
use std::ops::Deref;
use std::fmt;

/// A Software Transactional Memory pointer.
///
/// Loads should always be constant-time, even in the face of both load
/// and update contention.
///
/// Updates might take a long time, and the closure passed to it might
/// run multiple times. This is because if the "old" value is updated
/// before the closure finishes, the closure might overwrite up-to-date
/// data and must be run again with said new data passed in. Additionally,
/// memory reclamation of old STM values is performed at this point.
///
/// Sets take much longer than loads as well, but they should be approximately
/// constant-time as they don't need to be re-run if a different thread
/// sets the STM before it can finish.
pub struct Stm<T: 'static + Send + Sync> {
    inner: Atomic<T>,
}

impl<T: 'static + Send + Sync> Stm<T> {
    /// Create a new STM pointer pointing to `data`.
    ///
    /// # Example
    /// ```
    /// # use crossbeam_stm::Stm;
    /// let stm = Stm::new(vec![1,2,3,4]);
    /// ```
    pub fn new(data: T) -> Stm<T> {
        Stm {
            inner: Atomic::new(data),
        }
    }

    /// Update the STM in a fallible fashion.
    pub fn update_fallible<F, E>(&self, f: F) -> Result<(), E>
    where
        F: Fn(&T) -> Result<T, E>,
    {
        let guard = crossbeam_epoch::pin();
        guard.flush();
        loop {
            let shared = self.inner.load(Ordering::Acquire, &guard);
            let data = unsafe { shared.as_ref().unwrap() };
            let t = f(data)?;
            let r = self.inner
                .compare_and_set(shared, Owned::new(t), Ordering::AcqRel, &guard);
            if let Ok(r) = r {
                unsafe { guard.defer(move || r.into_owned()) }
                break;
            }
        }
        Ok(())
    }

    /// Update the STM.
    ///
    /// This is done by passing the current STM value to a closure and
    /// setting the STM to the closure's return value, provided no other
    /// threads have changed the STM in the meantime.
    ///
    /// If you don't care about any other threads setting the STM during
    /// processing, use the `set()` method.
    ///
    /// # Example
    /// ```
    /// # use crossbeam_stm::Stm;
    /// let stm = Stm::new(vec![1,2,3,4]);
    /// stm.update(|old| {
    ///     let mut new = old.clone();
    ///     new.push(5);
    ///     new
    /// })
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: Fn(&T) -> T,
    {
        self.update_fallible(|t| Ok::<T, ()>(f(t))).unwrap()
    }

    /// Update the STM, ignoring the current value.
    ///
    /// # Example
    /// ```
    /// # use crossbeam_stm::Stm;
    /// let stm = Stm::new(vec![1,2,3,4]);
    /// stm.set(vec![9,8,7,6]);
    /// ```
    pub fn set(&self, data: T) {
        let guard = crossbeam_epoch::pin();
        guard.flush();
        let r = self.inner.swap(Owned::new(data), Ordering::Release, &guard);
        unsafe { guard.defer(move || r.into_owned()) }
    }

    /// Load the current value from the STM.
    ///
    /// This returns an STM guard, rather than returning the
    /// internal value directly. In order to access the value explicitly,
    /// it must be dereferenced.
    ///
    /// # Example
    /// ```
    /// # use crossbeam_stm::Stm;
    /// let stm = Stm::new(vec![1,2,3,4]);
    /// let stm_guard = stm.load();
    /// assert_eq!(*stm_guard, vec![1,2,3,4]);
    /// ```
    ///
    /// # Warning
    /// This method returns a guard that will pin the current thread, but
    /// won't directly hold on to a particular value. This means that even
    /// though `load()` has been called, it's not a guarantee that the data
    /// won't change between dereferences. As an example,
    ///
    /// ```
    /// # use crossbeam_stm::Stm;
    /// let stm = Stm::new(vec![1,2,3,4]);
    /// let guard = stm.load();
    /// assert_eq!(*guard, vec![1,2,3,4]);
    /// stm.set(vec![9,8,7,6]);
    /// assert_eq!(*guard, vec![9,8,7,6]);
    /// ```
    pub fn load(&self) -> StmGuard<T> {
        StmGuard {
            parent: self,
            inner: crossbeam_epoch::pin(),
        }
    }
}

impl<T: 'static + Send + Sync + fmt::Debug> fmt::Debug for Stm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StmGuard")
            .field("data", self.load().deref())
            .finish()
    }
}

impl<T: 'static + Send + Sync + fmt::Display> fmt::Display for Stm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.load().deref().fmt(f)
    }
}

impl<T: 'static + Send + Sync> Drop for Stm<T> {
    fn drop(&mut self) {
        let guard = crossbeam_epoch::pin();
        let shared = self.inner.load(Ordering::Acquire, &guard);
        unsafe {
            shared.into_owned();
        }
    }
}

impl<T: 'static + Send + Sync> Clone for Stm<T> {
    fn clone(&self) -> Stm<T> {
        Stm {
            inner: self.inner.clone()
        }
    }
}

/// Structure that ensures any loaded data won't be freed by a future update.
///
/// Once this structure is dropped, the memory it dereferenced to can be
/// reclaimed.
pub struct StmGuard<'a, T: 'static + Send + Sync> {
    parent: &'a Stm<T>,
    inner: crossbeam_epoch::Guard,
}

impl<'a, T: 'static + Send + Sync> Deref for StmGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        let shared = self.parent.inner.load(Ordering::Acquire, &self.inner);
        unsafe { shared.as_ref().unwrap() }
    }
}

impl<'a, T: 'static + Send + Sync + fmt::Debug> fmt::Debug for StmGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StmGuard")
            .field("data", &self.deref())
            .finish()
    }
}

impl<'a, T: 'static + Send + Sync + fmt::Display> fmt::Display for StmGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};
    static DROPCOUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn stm_test() {
        let stm = Stm::new(vec![1, 2, 3]);
        {
            let data = stm.load();
            assert_eq!(*data, vec![1, 2, 3]);
        }

        stm.update(|v| {
            let mut v = v.clone();
            v.push(4);
            v
        });

        {
            let data = stm.load();
            assert_eq!(*data, vec![1, 2, 3, 4]);
        }

        stm.update(|_| vec![1]);

        {
            let data = stm.load();
            assert_eq!(*data, vec![1]);
        }
    }

    #[test]
    fn test_no_leaks() {
        DROPCOUNTER.store(0, Ordering::SeqCst);

        struct DropCounter<'a> {
            r: &'a AtomicUsize,
        }

        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                self.r.fetch_add(1, Ordering::SeqCst);
            }
        }

        drop(Stm::new(DropCounter { r: &DROPCOUNTER }));

        // We expect the value to have been dropped exactly once.
        assert_eq!(DROPCOUNTER.load(Ordering::SeqCst), 1);
    }
}
