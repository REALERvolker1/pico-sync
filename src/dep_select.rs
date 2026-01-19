#[cfg(feature = "rp235x-hal")]
pub(crate) use ::rp235x_hal::sio::{Spinlock, SpinlockValid};
#[cfg(feature = "rp2040-hal")]
pub(crate) use ::rp2040_hal::sio::{Spinlock, SpinlockValid};
