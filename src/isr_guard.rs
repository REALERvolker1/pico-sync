//! Helper types for enforcing ownership of static mutable data
//! by ISRs.
#![allow(non_camel_case_types)]

use {
    crate::{hal::pac::Interrupt, sealed_trait::Sealed},
    ::core::marker::PhantomData,
};

/// A helper trait for interrupt service routines to allow safe lockfree
/// static mutable data access
pub trait IsrToken: Sealed {
    /// The IRQ number this ISR is bound to.
    const IRQ: Interrupt;
}

macro_rules! interrupt_impl {
    ($( $irq:ident ),+$(,)?) => {
        $(
        /// A simple ZST that can be used to enforce ownership of static state
        pub struct $irq {
            _marker: PhantomData<*const ()>,
        }
        impl Sealed for $irq {}
        impl IsrToken for $irq {
            const IRQ: Interrupt = Interrupt::$irq;
        }
        impl $irq {
            /// Take ownership of this token
            /// # Safety
            /// Unlike the core tokens, there isn't really a good way to determine
            /// whether the ISR calling this function should actually own this context.
            /// This means that there will not be a runtime check in debug builds when this function
            /// is called, and bugs could arise when callers rely on this type for
            /// enforcing invariants.
            ///
            /// The main reason this is still acceptable is that one would have to go out of
            /// their way to misuse this function, since the interrupt name is part of the type name.
            pub unsafe fn steal() -> Self {
                Self {
                    _marker: PhantomData,
                }
            }
        }
        )+
    };
}
#[cfg(feature = "rp235x-hal")]
interrupt_impl! {
    TIMER0_IRQ_0,
    TIMER0_IRQ_1,
    TIMER0_IRQ_2,
    TIMER0_IRQ_3,
    TIMER1_IRQ_0,
    TIMER1_IRQ_1,
    TIMER1_IRQ_2,
    TIMER1_IRQ_3,
    PWM_IRQ_WRAP_0,
    PWM_IRQ_WRAP_1,
    DMA_IRQ_0,
    DMA_IRQ_1,
    DMA_IRQ_2,
    DMA_IRQ_3,
    USBCTRL_IRQ,
    PIO0_IRQ_0,
    PIO0_IRQ_1,
    PIO1_IRQ_0,
    PIO1_IRQ_1,
    PIO2_IRQ_0,
    PIO2_IRQ_1,
    IO_IRQ_BANK0,
    IO_IRQ_BANK0_NS,
    IO_IRQ_QSPI,
    IO_IRQ_QSPI_NS,
    SIO_IRQ_FIFO,
    SIO_IRQ_BELL,
    SIO_IRQ_FIFO_NS,
    SIO_IRQ_BELL_NS,
    SIO_IRQ_MTIMECMP,
    CLOCKS_IRQ,
    SPI0_IRQ,
    SPI1_IRQ,
    UART0_IRQ,
    UART1_IRQ,
    ADC_IRQ_FIFO,
    I2C0_IRQ,
    I2C1_IRQ,
    OTP_IRQ,
    TRNG_IRQ,
    PLL_SYS_IRQ,
    PLL_USB_IRQ,
    POWMAN_IRQ_POW,
    POWMAN_IRQ_TIMER,
}
#[cfg(feature = "rp2040-hal")]
interrupt_impl! {
    TIMER_IRQ_0,
    TIMER_IRQ_1,
    TIMER_IRQ_2,
    TIMER_IRQ_3,
    PWM_IRQ_WRAP,
    USBCTRL_IRQ,
    XIP_IRQ,
    PIO0_IRQ_0,
    PIO0_IRQ_1,
    PIO1_IRQ_0,
    PIO1_IRQ_1,
    DMA_IRQ_0,
    DMA_IRQ_1,
    IO_IRQ_BANK0,
    IO_IRQ_QSPI,
    SIO_IRQ_PROC0,
    SIO_IRQ_PROC1,
    CLOCKS_IRQ,
    SPI0_IRQ,
    SPI1_IRQ,
    UART0_IRQ,
    UART1_IRQ,
    ADC_IRQ_FIFO,
    I2C0_IRQ,
    I2C1_IRQ,
    RTC_IRQ,
    SW0_IRQ,
    SW1_IRQ,
    SW2_IRQ,
    SW3_IRQ,
    SW4_IRQ,
    SW5_IRQ,
}
