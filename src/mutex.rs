//! A hardware spinlock-backed mutex

use {
    crate::hal::sio::{Spinlock, SpinlockValid},
    ::core::{
        cell::UnsafeCell,
        marker::PhantomData,
        mem::MaybeUninit,
        ops::{Deref, DerefMut},
    },
};

/// A mutex backed by a hardware spinlock.
pub struct SpinlockMutex<T, const N: usize>
where
    Spinlock<N>: SpinlockValid,
{
    data: UnsafeCell<T>,
    _marker: PhantomData<Spinlock<N>>,
}

impl<T, const N: usize> SpinlockMutex<T, N>
where
    Spinlock<N>: SpinlockValid,
{
    /// Create a new (locked) spinlock.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            _marker: PhantomData,
        }
    }
    /// Take a mutable reference to the data inside the spinlock, with
    /// a lock you already claimed.
    pub fn lock_with(&self, lock: Spinlock<N>) -> RefMut<'_, T, N> {
        // SAFETY: The user provided us with a lock, and
        // the only way to get a structure of type `Spinlock<N>`
        // is by owning the lock. This is guaranteed by the HALs. as long as they
        // used safe code to acquire it.
        RefMut::new(unsafe { &mut *self.data.get() }, lock)
    }
    /// Try to claim the spinlock. If successful, returns a mutable reference.
    /// If unsuccessful, returns `None`. Does not block.
    pub fn try_lock(&self) -> Option<RefMut<'_, T, N>> {
        Spinlock::try_claim().map(|l| self.lock_with(l))
    }
    /// Wait for the spinlock to be unlocked, blocking the
    /// current core.
    /// # Safety
    /// This will cause a deadlock if the lock has already been claimed.
    /// Since deadlocks are considered safe in rust, this is not an inherent issue,
    /// but neither the RP2040 nor the RP2350A/B unlock all spinlocks on boot (in hardware).
    ///
    /// If you wish to provide your own lock, you may use the
    /// [`lock_with`](Self::lock_with)
    /// method.
    pub fn lock_blocking(&self) -> RefMut<'_, T, N> {
        self.lock_with(Spinlock::claim())
    }
    /// Consume the mutex, returning the inner data. This neither
    /// claims the spinlock, nor blocks the current core.
    pub fn into_inner(self) -> T {
        // SAFETY: We have exclusive access,
        // since the caller relinquishes ownership.
        self.data.into_inner()
    }
    /// Acquire a temporary mutable reference to the inner data with
    /// [`lock_blocking`](Self::lock_blocking)
    /// and immediately calls the user-provided function.
    pub fn call_with_lock_blocking<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut T) -> R,
    {
        let mut l = self.lock_blocking();
        f(l.as_mut())
    }
    /// Attempts to acquire a temporary mutable reference to the inner data with
    /// [`try_lock`](Self::try_lock)
    /// and immediately calls the user-provided function if the lock was successfully acquired.
    pub fn try_call_with_lock<F, R>(&self, mut f: F) -> Option<R>
    where
        F: FnMut(&mut T) -> R,
    {
        let mut l = self.try_lock()?;
        let r = f(l.as_mut());
        Some(r)
    }
}
impl<T, const N: usize> SpinlockMutex<MaybeUninit<T>, N>
where
    Spinlock<N>: SpinlockValid,
{
    /// Construct an uninitialized value
    pub const fn uninit() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }
    /// Same safety warnings as [`MaybeUninit::assume_init`]. Basically a transmute.
    pub unsafe fn assume_init(self) -> SpinlockMutex<T, N> {
        let data = self.data.into_inner();
        SpinlockMutex {
            // SAFETY: Caller asserts it is initialized
            data: UnsafeCell::new(unsafe { data.assume_init() }),
            _marker: PhantomData,
        }
    }
    /// Same safety warnings as [`MaybeUninit::assume_init_ref`]. Basically a pointer cast.
    pub unsafe fn assume_init_ref(&self) -> &SpinlockMutex<T, N> {
        // SAFETY: The memory layout of MaybeUninit<T> is equivalent to bare T
        // Additionally, the memory layout of UnsafeCell<T> is equivalent to bare T.
        // The only real types that are changing here are from
        // Mtx<UnsafeCell<MaybeUninit<T>>> to Mtx<UnsafeCell<T>>
        // since the target type also happens to be the exact same struct type as this,
        // but without an extra level of type and optimizer indirection.
        // Also, the target keeps the same synchronization properties as this,
        // since no `SpinlockMutex`es actually hold a lock.
        unsafe { core::mem::transmute(self) }
        // TODO: Find a better function to do this
    }
    /// Same safety warnings as [`MaybeUninit::assume_init_mut`]
    pub unsafe fn try_assume_init_lock(&self) -> Option<RefMut<'_, T, N>> {
        self.try_lock().map(|r| unsafe { r.assume_init() })
    }
    /// Same safety warnings as [`MaybeUninit::assume_init_mut`]
    pub unsafe fn assume_init_lock_blocking(&self) -> RefMut<'_, T, N> {
        let l = self.lock_blocking();
        unsafe { l.assume_init() }
    }
    /// Same safety warnings as [`MaybeUninit::assume_init_mut`]
    pub unsafe fn assume_init_lock_with(&self, lock: Spinlock<N>) -> RefMut<'_, T, N> {
        let l = self.lock_with(lock);
        unsafe { l.assume_init() }
    }
}

// SAFETY: Spinlocks provide hardware-level synchronization
unsafe impl<T, const N: usize> Send for SpinlockMutex<T, N> where Spinlock<N>: SpinlockValid {}
unsafe impl<T, const N: usize> Sync for SpinlockMutex<T, N> where Spinlock<N>: SpinlockValid {}

/// A mutable borrow to the data inside of a spinlock
pub struct RefMut<'l, T, const N: usize>
where
    Spinlock<N>: SpinlockValid,
{
    spinlock: Spinlock<N>,
    data: &'l mut T,
    /// Removes any Send/Sync auto-impls
    _marker: PhantomData<*const ()>,
}
impl<'l, T, const N: usize> RefMut<'l, T, N>
where
    Spinlock<N>: SpinlockValid,
{
    fn new(data: &'l mut T, lock: Spinlock<N>) -> Self {
        Self {
            spinlock: lock,
            data,
            _marker: PhantomData,
        }
    }
}
impl<'l, T, const N: usize> AsMut<T> for RefMut<'l, T, N>
where
    Spinlock<N>: SpinlockValid,
{
    fn as_mut(&mut self) -> &mut T {
        self.data
    }
}
impl<'l, T, const N: usize> AsRef<T> for RefMut<'l, T, N>
where
    Spinlock<N>: SpinlockValid,
{
    fn as_ref(&self) -> &T {
        self.data
    }
}
impl<'l, T, const N: usize> Deref for RefMut<'l, T, N>
where
    Spinlock<N>: SpinlockValid,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data
    }
}
impl<'l, T, const N: usize> DerefMut for RefMut<'l, T, N>
where
    Spinlock<N>: SpinlockValid,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl<'l, T, const N: usize> RefMut<'l, MaybeUninit<T>, N>
where
    Spinlock<N>: SpinlockValid,
{
    /// Same safety warnings as [`MaybeUninit::assume_init_mut`]
    pub unsafe fn assume_init(self) -> RefMut<'l, T, N> {
        RefMut {
            spinlock: self.spinlock,
            // SAFETY: Caller asserts this is initialized
            data: unsafe { self.data.assume_init_mut() },
            _marker: PhantomData,
        }
    }
}
