use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

/// Deterministic pseudo-random number generator for reproducible battle simulations.
/// Equivalent to Godot's `RandomNumberGenerator`.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    seed: u64,
    rng: Pcg64,
    roll_count: u64,
}

impl Default for DeterministicRng {
    fn default() -> Self {
        Self::new(1337)
    }
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: Pcg64::seed_from_u64(seed),
            roll_count: 0,
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        self.rng = Pcg64::seed_from_u64(seed);
        self.roll_count = 0;
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn roll_count(&self) -> u64 {
        self.roll_count
    }

    /// Generates a non-negative random integer.
    pub fn randi(&mut self) -> i64 {
        self.roll_count += 1;
        self.rng.r#gen::<i64>().abs()
    }

    /// Generates a random integer in [from, to] inclusive.
    pub fn randi_range(&mut self, from: i64, to: i64) -> i64 {
        self.roll_count += 1;
        if from >= to {
            return from;
        }
        self.rng.gen_range(from..=to)
    }

    /// Generates a random float in [0.0, 1.0).
    pub fn randf(&mut self) -> f64 {
        self.roll_count += 1;
        self.rng.r#gen::<f64>()
    }

    /// Generates a random float in [from, to).
    pub fn randf_range(&mut self, from: f64, to: f64) -> f64 {
        self.roll_count += 1;
        if from >= to {
            return from;
        }
        self.rng.gen_range(from..to)
    }

    /// Roll 1..100 inclusive (standard RPG percentage roll).
    pub fn roll_d100(&mut self) -> i64 {
        self.randi_range(1, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_replays() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);

        let rolls1: Vec<i64> = (0..20).map(|_| rng1.randi_range(1, 100)).collect();
        let rolls2: Vec<i64> = (0..20).map(|_| rng2.randi_range(1, 100)).collect();

        assert_eq!(rolls1, rolls2);
        assert_eq!(rng1.roll_count(), 20);
    }
}
