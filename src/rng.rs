//! Xorshift64* generator, exactly as specified in SPEC section 5.2.
//!
//! The generator is deterministic and has no clock. A zero seed is replaced by
//! the documented constant so that a caller can never lock the state at zero.

/// The seed substituted for zero, per SPEC 5.2.
pub const ZERO_SEED_REPLACEMENT: u64 = 0x9E37_79B9_7F4A_7C15;

/// The Xorshift64* multiplier.
const MULTIPLIER: u64 = 0x2545_F491_4F6C_DD1D;

/// A deterministic Xorshift64* random number generator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    /// Create a generator. A seed of zero is replaced by the documented value.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                ZERO_SEED_REPLACEMENT
            } else {
                seed
            },
        }
    }

    /// Return the current internal state. Used by tests to prove that blocked
    /// moves do not advance the RNG.
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Draw the next `u64` in the sequence.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(MULTIPLIER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_seed_uses_the_documented_replacement() {
        assert_eq!(Xorshift64::new(0).state(), ZERO_SEED_REPLACEMENT);
        assert_ne!(Xorshift64::new(0).state(), Xorshift64::new(1).state());
    }

    #[test]
    fn known_seed_42_prefix_is_stable() {
        let mut rng = Xorshift64::new(42);
        let draws: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();
        assert_eq!(
            draws,
            vec![
                6_255_019_084_209_693_600,
                14_430_073_426_741_505_498,
                14_575_455_857_230_217_846,
                17_414_512_882_241_728_735,
            ]
        );
    }
}
