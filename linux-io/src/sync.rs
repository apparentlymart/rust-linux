use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::null,
    sync::atomic::{
        AtomicU32,
        Ordering::{Acquire, Relaxed, Release},
    },
};

/// A mutex implemented in terms of the Linux "futex" system call.
pub struct Mutex<T: ?Sized> {
    futex: Futex<true>,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(v: T) -> Self {
        Self {
            futex: Futex::new(),
            data: UnsafeCell::new(v),
        }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        self.futex.lock();
        MutexGuard::new(self)
    }

    pub fn try_lock<'a>(&'a self) -> core::result::Result<MutexGuard<'a, T>, ()> {
        if self.futex.try_lock() {
            Ok(MutexGuard::new(self))
        } else {
            Err(())
        }
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'mutex, T: ?Sized + 'mutex> {
    lock: &'mutex Mutex<T>,
    _not_send: PhantomData<*mut ()>,
}

impl<'mutex, T: ?Sized + 'mutex> MutexGuard<'mutex, T> {
    #[inline]
    fn new(lock: &'mutex Mutex<T>) -> Self {
        Self {
            lock,
            _not_send: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { self.lock.futex.unlock() }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

#[repr(transparent)]
struct Futex<const SINGLE_PROCESS: bool = false> {
    futex_word: AtomicU32,
}

impl<const SINGLE_PROCESS: bool> Futex<SINGLE_PROCESS> {
    // The following is essentially just the futex implementation for std, but lightly
    // adapted to use linux-unsafe to call the kernel's futex ops.

    const UNLOCKED: u32 = 0;
    const LOCKED: u32 = 1; // locked, no other threads waiting
    const CONTENDED: u32 = 2; // locked, and other threads waiting (contended)

    const FUTEX_WAIT: linux_unsafe::int = linux_unsafe::FUTEX_WAIT | Self::FUTEX_OP_FLAGS;
    const FUTEX_WAKE: linux_unsafe::int = linux_unsafe::FUTEX_WAKE | Self::FUTEX_OP_FLAGS;
    const FUTEX_OP_FLAGS: linux_unsafe::int = if SINGLE_PROCESS {
        linux_unsafe::FUTEX_PRIVATE
    } else {
        0
    };

    #[inline]
    pub const fn new() -> Self {
        Self {
            futex_word: AtomicU32::new(Self::UNLOCKED),
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        self.futex_word
            .compare_exchange(Self::UNLOCKED, Self::LOCKED, Acquire, Relaxed)
            .is_ok()
    }

    #[inline]
    pub fn lock(&self) {
        if self
            .futex_word
            .compare_exchange(Self::UNLOCKED, Self::LOCKED, Acquire, Relaxed)
            .is_err()
        {
            self.lock_contended();
        }
    }

    #[cold]
    fn lock_contended(&self) {
        // Spin first to speed things up if the lock is released quickly.
        let mut state = self.spin();

        // If it's unlocked now, attempt to take the lock
        // without marking it as contended.
        if state == Self::UNLOCKED {
            match self
                .futex_word
                .compare_exchange(Self::UNLOCKED, Self::LOCKED, Acquire, Relaxed)
            {
                Ok(_) => return, // Locked!
                Err(s) => state = s,
            }
        }

        loop {
            // Put the lock in contended state.
            // We avoid an unnecessary write if it as already set to CONTENDED,
            // to be friendlier for the caches.
            if state != Self::CONTENDED
                && self.futex_word.swap(Self::CONTENDED, Acquire) == Self::UNLOCKED
            {
                // We changed it from UNLOCKED to CONTENDED, so we just successfully locked it.
                return;
            }

            // Wait for the futex to change state, assuming it is still CONTENDED.
            let _ = self.futex_wait(Self::CONTENDED);

            // Spin again after waking up.
            state = self.spin();
        }
    }

    fn spin(&self) -> u32 {
        let mut spin = 100;
        loop {
            // We only use `load` (and not `swap` or `compare_exchange`)
            // while spinning, to be easier on the caches.
            let state = self.futex_word.load(Relaxed);

            // We stop spinning when the mutex is UNLOCKED,
            // but also when it's CONTENDED.
            if state != Self::LOCKED || spin == 0 {
                return state;
            }

            core::hint::spin_loop();
            spin -= 1;
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        if self.futex_word.swap(Self::UNLOCKED, Release) == Self::CONTENDED {
            // We only wake up one thread. When that thread locks the mutex, it
            // will mark the mutex as CONTENDED (see lock_contended above),
            // which makes sure that any other waiting threads will also be
            // woken up eventually.
            self.wake();
        }
    }

    #[cold]
    fn wake(&self) {
        let _ = self.futex_wake();
    }

    #[inline]
    fn futex_wait(&self, want: u32) -> linux_unsafe::result::Result<linux_unsafe::int> {
        unsafe {
            linux_unsafe::futex(
                self.futex_word.as_ptr(),
                Self::FUTEX_WAIT,
                want,
                0,
                null(),
                0,
            )
        }
    }

    #[inline]
    fn futex_wake(&self) -> linux_unsafe::result::Result<linux_unsafe::int> {
        unsafe { linux_unsafe::futex(self.futex_word.as_ptr(), Self::FUTEX_WAKE, 1, 0, null(), 0) }
    }
}
