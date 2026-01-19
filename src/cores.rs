//! A simple zero-sized type that can only be "owned" by the context running on its own core.
//! It can be used for enforcing safety invariants.

use {
    crate::{
        hal::sio::{CoreId, Sio},
        sealed_trait::Sealed,
    },
    ::core::{cell::OnceCell, marker::PhantomData},
};

#[cfg(feature = "single-core-guard-access")]
use ::portable_atomic::{AtomicBool, Ordering};

/// A lock-free data structure used for static mutable state on a single core.
/// # Safety
/// This is unsound to access from a non-owning core.
pub struct LocalCell<T, C>
where
    C: SingleCore,
{
    data: OnceCell<T>,
    _marker: PhantomData<C>,
}
// SAFETY: It is impossible to access this cell from other cores without unsafe code.
unsafe impl<T, C> Send for LocalCell<T, C> where C: SingleCore {}
unsafe impl<T, C> Sync for LocalCell<T, C> where C: SingleCore {}
impl<T, C> LocalCell<T, C>
where
    C: SingleCore,
{
    /// Create a new uninitialized cell
    pub const fn new() -> Self {
        Self {
            data: OnceCell::new(),
            _marker: PhantomData,
        }
    }
    #[inline(always)]
    fn initialize_inner(&self, init: impl FnOnce() -> T) -> &T {
        self.data.get_or_init(init)
    }
    /// Initialize the cell's data, providing proof of ownership.
    #[cfg(feature = "single-core-guard-access")]
    pub fn get_or_init<'c, 's: 'c>(&'s self, _ctx: &'c C, init: impl FnOnce() -> T) -> &'s T {
        self.initialize_inner(init)
    }
    /// Initialize the cell's data, providing proof of ownership.
    /// # Safety
    /// Without `single-core-guard-access`, the core guard could potentially be
    /// claimed by multiple contexts, including interrupts!
    /// The caller must ensure this does not get preempted by others who wish to
    /// access the inner value.
    #[cfg(not(feature = "single-core-guard-access"))]
    pub unsafe fn get_or_init<'c, 's: 'c>(
        &'s self,
        _ctx: &'c C,
        init: impl FnOnce() -> T,
    ) -> &'s T {
        self.initialize_inner(init)
    }
    /// Get the inner value without checking if it is initialized.
    /// # Safety
    /// Calling this function on an uninitialized cell is Undefined Behavior.
    pub unsafe fn get_unchecked(&self, _ctx: &C) -> &T {
        // SAFETY: Users of this type assert they initialized this before the runtime start
        unsafe { self.data.get().unwrap_unchecked() }
    }
    /// Get the inner value, or `None` if it has not been initialized yet.
    #[cfg(feature = "single-core-guard-access")]
    pub fn get<'c, 's: 'c>(&'s self, _ctx: &'c C) -> Option<&'s T> {
        self.data.get()
    }
    /// Get the inner value, or `None` if it has not been initialized yet.
    /// # Safety
    /// Without `single-core-guard-access`, the core guard could potentially be
    /// claimed by multiple contexts, including interrupts!
    /// The caller must ensure this does not get preempted by others who wish to
    /// access the inner value.
    #[cfg(not(feature = "single-core-guard-access"))]
    pub unsafe fn get<'c, 's: 'c>(&'s self, _ctx: &'c C) -> Option<&'s T> {
        self.data.get()
    }
}

macro_rules! singlecore {
    ($struct:ident, $id:expr, $statevar:ident) => {
        #[cfg(feature = "single-core-guard-access")]
        static $statevar: AtomicBool = AtomicBool::new(false);

        /// A token that can be used to enforce ownership
        /// and safety invariants through the type system.
        pub struct $struct {
            _marker: PhantomData<*const ()>,
        }

        impl Sealed for $struct {}
        impl SingleCore for $struct {
            const ID: CoreId = $id;
            unsafe fn steal() -> Self {
                debug_assert_eq!(Sio::core(), Self::ID);
                Self {
                    _marker: PhantomData,
                }
            }
        }
        impl $struct {
            /// Try to take the ownership token.
            /// If it is already claimed, this function will return `None`.
            #[cfg(feature = "single-core-guard-access")]
            #[inline(always)]
            pub fn try_claim() -> Option<Self> {
                try_claim_inner(&$statevar)
            }
            // implementation note: This is called something different because
            // user code safety assumptions should be entirely different when toggling this feature on/off.

            /// Try to create a new ownership token. Succeeds if called on the right core.
            #[cfg(not(feature = "single-core-guard-access"))]
            pub fn try_new() -> Option<Self> {
                try_new_inner()
            }
        }
    };
}
/// helper function so we avoid messing with macros
#[cfg(feature = "single-core-guard-access")]
fn try_claim_inner<T: SingleCore>(b: &AtomicBool) -> Option<T> {
    if b.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        // SAFETY: This is guarded with a cmpxchg
        Some(unsafe { T::steal() })
    } else {
        None
    }
}
#[cfg(not(feature = "single-core-guard-access"))]
fn try_new_inner<T: SingleCore>() -> Option<T> {
    if Sio::core() == T::ID {
        // SAFETY: This branch can only occur when run on
        // the correct core.
        Some(unsafe { T::steal() })
    } else {
        None
    }
}

/// A marker trait that allows type-generic ownership tokens.
pub trait SingleCore: Sealed {
    /// The ID of the current core
    const ID: CoreId;
    /// Acquire the current core's token.
    /// # Safety
    /// As these token types are relied upon for sync-safety,
    /// calling this from a different core can lead to memory safety
    /// violations.
    /// # Panics
    /// When debug assertions are enabled, this will panic if
    /// called from a different core.
    unsafe fn steal() -> Self;
}

singlecore!(Core0Token, CoreId::Core0, IS_CORE0_TAKEN);
singlecore!(Core1Token, CoreId::Core1, IS_CORE1_TAKEN);
