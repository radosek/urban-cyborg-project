//! Replacement for libc random(): non-negative 31-bit values.

use std::cell::Cell;

thread_local! {
    static STATE: Cell<u64> = Cell::new(0x2545F4914F6CDD1D);
    static CALLS: Cell<u64> = const { Cell::new(0) };
}

/// Number of random() calls so far (replay traces compare it with the C++ build).
pub fn calls() -> u64 {
    CALLS.with(|c| c.get())
}

pub fn state() -> u64 {
    STATE.with(|c| c.get())
}

pub fn set_state(s: u64) {
    STATE.with(|c| c.set(s));
}

pub fn seed(s: u64) {
    STATE.with(|c| c.set(s | 1));
}

/// Like C `random()`: uniformly distributed in 0..=i32::MAX.
#[track_caller]
pub fn random() -> i32 {
    CALLS.with(|c| c.set(c.get() + 1));
    if std::env::var_os("URBAN_RNG_TRACE").is_some() {
        eprintln!("RNG {} {}", calls(), std::panic::Location::caller());
    }
    if std::env::var("URBAN_RNG_BT").ok().and_then(|v| v.parse::<u64>().ok()) == Some(calls()) {
        eprintln!("{}", std::backtrace::Backtrace::force_capture());
    }
    STATE.with(|c| {
        let mut x = c.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        c.set(x);
        ((x >> 33) & 0x7fff_ffff) as i32
    })
}
