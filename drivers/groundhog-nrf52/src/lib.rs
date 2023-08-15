//! Contains a nrf52-compatible implementation of the `RollingTimer` trait.
//!
//! The `GlobalRollingTimer` is especially helpful if you are running RTIC on
//! a nrf52 board. The built-in cycle counter (`CYCCNT`) which is commonly used
//! as a monotonic counter will not work when the debugger is not attached, which
//! in turn will make scheduling operations not work as expected.
//!
//! # Usage
//!
//! To use the the `GlobalRollingTimer` with RTIC, it first needs to be selected
//! as the monotonic timer (here on top of the nrf52840 hal):
//!
//! ```ignore
//! #[rtic::app(device = nrf52840_hal::pac, peripherals = true, monotonic = groundhog_nrf52::GlobalRollingTimer)]
//! ```
//!
//! During the init phase it needs to be initialized with a concrete timer implementation:
//!
//! ```ignore
//! #[init]
//! fn init(ctx: init::Context) -> init::LateResources {
//!     // using TIMER0 here
//!     GlobalRollingTimer::init(ctx.device.TIMER0);
//!     // ...
//! }
//! ```
//!
//! Then, you can specify the schedule interval in microseconds as part of your task:
//!
//! ```ignore
//! #[task]
//! fn my_task(ctx: my_task::Context) {
//!     ctx.schedule
//!         .my_task(ctx.scheduled + 1_000_000)
//!         .unwrap();
//! }
//! ```
//! In this case the task will be scheduled again one second later.
//!
#![no_std]

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use groundhog::RollingTimer;
use nrf52840_hal::{pac::timer0::RegisterBlock as RegBlock0, timer::Instance};
use rtic::Monotonic;

use core::sync::atomic::{AtomicPtr, Ordering};

static TIMER_PTR: AtomicPtr<RegBlock0> = AtomicPtr::new(core::ptr::null_mut());

/// A global rolling timer
///
/// This must be initialized with a timer (like `TIMER0`) once,
/// on startup, before valid timer values will be returned. Until then,
/// a timer value of 0 ticks will always be returned.
///
/// At the moment, this is limited to a 32-bit 1MHz timer, which has a
/// maximum observable time delta of 71m34s.
pub struct GlobalRollingTimer;

impl GlobalRollingTimer {
    pub const fn new() -> Self {
        Self
    }

    pub fn init<T: Instance>(timer: T) {
        timer.set_periodic();
        timer.timer_start(0xFFFF_FFFFu32);
        let t0 = timer.as_timer0();

        let old_ptr = TIMER_PTR.swap(t0 as *const _ as *mut _, Ordering::SeqCst);

        debug_assert!(old_ptr == core::ptr::null_mut());
    }
}

impl Monotonic for GlobalRollingTimer {
    type Instant = i32;
    type Duration = i32;

    fn now(&mut self) -> Self::Instant {
        self.get_ticks() as i32
    }

    fn zero() -> Self::Instant {
        0
    }

    unsafe fn reset(&mut self) {
        if let Some(t0) = TIMER_PTR.load(Ordering::SeqCst).as_ref() {
            t0.tasks_clear.write(|w| w.bits(1));
        }
    }

    fn set_compare(&mut self, _instant: Self::Instant) {
        todo!()
    }

    fn clear_compare_flag(&mut self) {
        todo!()
    }
}

impl RollingTimer for GlobalRollingTimer {
    type Tick = u32;
    const TICKS_PER_SECOND: u32 = 1_000_000;

    fn get_ticks(&self) -> u32 {
        if let Some(t0) = unsafe { TIMER_PTR.load(Ordering::SeqCst).as_ref() } {
            t0.tasks_capture[1].write(|w| unsafe { w.bits(1) });
            t0.cc[1].read().bits()
        } else {
            0
        }
    }

    fn is_initialized(&self) -> bool {
        TIMER_PTR.load(Ordering::SeqCst) != core::ptr::null_mut()
    }
}

impl DelayUs<u32> for GlobalRollingTimer {
    fn delay_us(&mut self, us: u32) {
        let start = self.get_ticks();
        while self.ticks_since(start) < us {}
    }
}

impl DelayMs<u32> for GlobalRollingTimer {
    fn delay_ms(&mut self, ms: u32) {
        for _ in 0..ms {
            self.delay_us(1000)
        }
    }
}
