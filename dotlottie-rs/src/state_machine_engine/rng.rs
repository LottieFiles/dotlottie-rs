//! Inline, seedable PRNG for the `SetRandom` action.
//!
//! A standard PCG32 generator (M. E. O'Neill, pcg-random.org). It is small,
//! fast, and — crucially — fully deterministic and platform-independent: the
//! only operations are 64-bit integer arithmetic and a division by a constant,
//! so a given seed produces the same `f32` sequence on native and wasm alike.
//!
//! We deliberately avoid the `rand`/`getrandom` crates: `getrandom` needs the
//! `js` feature on wasm and a build dance, and determinism is a feature here,
//! not a limitation to work around. Entropy enters the engine only through the
//! seed the host supplies (see `StateMachineEngine::set_seed`).

/// Seed used when the host never calls `set_seed`. Keeps the no-seed path fully
/// reproducible.
pub(crate) const DEFAULT_RNG_SEED: u64 = 0x853c_49e6_748f_ea9b;

/// Fixed stream/sequence constant for the generator.
const DEFAULT_STREAM: u64 = 0xda3e_39cb_94b9_5bdb;

const MULTIPLIER: u64 = 6364136223846793005;

/// Scale factor turning a 24-bit integer into an `f32` in `[0, 1)`.
const F32_SCALE: f32 = 1.0 / (1u32 << 24) as f32;

/// A PCG32 pseudo-random number generator.
#[derive(Clone, Debug)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    /// Create a generator seeded with `seed`. The same seed always yields the
    /// same sequence.
    pub fn new(seed: u64) -> Self {
        let mut rng = Pcg32 {
            state: 0,
            inc: (DEFAULT_STREAM << 1) | 1,
        };
        // Standard PCG seeding routine: step, add seed, step again.
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    /// Advance the generator and return the next 32-bit output.
    fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Draw a uniform `f32` in the half-open interval `[0.0, 1.0)`.
    ///
    /// Uses the top 24 bits of a 32-bit output (the full `f32` mantissa), so
    /// `1.0` is never returned.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * F32_SCALE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Pcg32::new(42);
        let mut b = Pcg32::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Pcg32::new(1);
        let mut b = Pcg32::new(2);
        // Extremely unlikely to match on the first draw for distinct seeds.
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn f32_in_unit_interval() {
        let mut rng = Pcg32::new(DEFAULT_RNG_SEED);
        for _ in 0..100_000 {
            let v = rng.next_f32();
            assert!((0.0..1.0).contains(&v), "draw out of range: {v}");
        }
    }

    #[test]
    fn roughly_uniform() {
        // Sanity: a dice in [4, 9] via the canonical shaping sequence should
        // cover all six faces over many rolls.
        let mut rng = Pcg32::new(7);
        let mut seen = [false; 6];
        for _ in 0..10_000 {
            let face = 4 + (rng.next_f32() * 6.0).floor() as i32;
            assert!((4..=9).contains(&face));
            seen[(face - 4) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "not all faces appeared");
    }
}
