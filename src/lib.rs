//! Synchronization primitives for Raspberry Pi Silicon

#![no_std]
#![forbid(missing_docs)]

#[cfg(feature = "rp235x-hal")]
pub(crate) use rp235x_hal as hal;
#[cfg(feature = "rp2040-hal")]
pub(crate) use rp2040_hal as hal;

#[cfg(not(any(feature = "rp2040-hal", feature = "rp235x-hal")))]
compile_error!("You must choose a HAL implementation!");

#[cfg(feature = "core-guards")]
pub mod cores;

pub mod mutex;

pub(crate) mod sealed_trait {
    pub trait Sealed {}
}

pub use crate::mutex::SpinlockMutex;
