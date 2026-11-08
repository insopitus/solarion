//! Tiny deterministic RNG used for loot rolls and combat variance.
//!
//! Keeping this local avoids pulling an extra dependency into the game while
//! still giving us reproducible runs (important for balance testing).

use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone)]
pub struct GameRng {
    state: u64,
}

impl GameRng {
    pub const fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Uniform value in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        bits as f32 / 16_777_216.0
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    /// Uniform integer in `[min, max)`.
    pub fn range_usize(&mut self, min: usize, max: usize) -> usize {
        debug_assert!(min < max, "range_usize requires min < max");
        min + (self.next_u64() as usize) % (max - min)
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(0x5EED_1A7E_5A1A_1234)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_stay_in_range() {
        let mut rng = GameRng::new(42);
        for _ in 0..10_000 {
            let value = rng.next_f32();
            assert!((0.0..1.0).contains(&value));
        }
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = GameRng::new(7);
        let mut b = GameRng::new(7);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
