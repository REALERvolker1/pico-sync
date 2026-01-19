#![no_std]

#[cfg(not(any(feature = "rp2040-hal", feature = "rp235x-hal")))]
compile_error!("You must choose a HAL implementation!");

mod dep_select;
pub mod mutex;

pub use mutex::SpinlockMutex;
