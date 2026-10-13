//! A tiny deterministic PRNG for random ticks.

/// A 32-bit xorshift generator.
///
/// It is intentionally small and dependency-free; do not use it for anything
/// security related.
#[derive(Clone, Copy, Debug)]
pub struct Rng32 {
    state: u32,
}

impl Rng32 {
    /// Creates a generator. A zero seed is replaced by a fixed non-zero seed.
    pub const fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9 } else { seed },
        }
    }

    /// Returns the next pseudo-random `u32`.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Returns a pseudo-random value in `0..upper`.
    ///
    /// Returns `0` when `upper` is `0`.
    pub fn next_below(&mut self, upper: u32) -> u32 {
        if upper == 0 {
            0
        } else {
            self.next_u32() % upper
        }
    }
}

impl Default for Rng32 {
    fn default() -> Self {
        Self::new(0x1234_5678)
    }
}
