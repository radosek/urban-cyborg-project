//! Replacement for libc random(): non-negative 31-bit values.

use std::cell::Cell;

thread_local! {
    static STATE: Cell<u64> = Cell::new(0x2545F4914F6CDD1D);
}

pub fn seed(s: u64) {
    STATE.with(|c| c.set(s | 1));
}

/// Like C `random()`: uniformly distributed in 0..=i32::MAX.
pub fn random() -> i32 {
    STATE.with(|c| {
        let mut x = c.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        c.set(x);
        ((x >> 33) & 0x7fff_ffff) as i32
    })
}
