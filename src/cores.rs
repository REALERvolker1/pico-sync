use {
    crate::{
        hal::sio::{CoreId, Sio},
        sealed_trait::Sealed,
    },
    ::core::marker::PhantomData,
    ::portable_atomic::{AtomicBool, Ordering},
};

macro_rules! singlecore {
    ($struct:ident, $id:expr, $statevar:ident) => {
        #[cfg(feature = "safe-core-guard-access")]
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
            #[cfg(feature = "safe-core-guard-access")]
            #[inline(always)]
            pub fn take() -> Option<Self> {
                take_inner(&$statevar)
            }
        }
    };
}
/// helper function so we avoid messing with macros
#[cfg(feature = "safe-core-guard-access")]
fn take_inner<T: SingleCore>(b: &AtomicBool) -> Option<T> {
    if b.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        // SAFETY: This is guarded with a cmpxchg
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
