//! Godot 4's RandomNumberGenerator (core/math/random_pcg.h, thirdparty/misc/pcg.c), bit for bit, so a seeded session
//! replays the same rolls as the Godot reference. Parity is judged statistically; exact rolls make the first
//! divergence between the two engines easy to find.

const DEFAULT_SEED: u64 = 12047754176567800795;
const DEFAULT_INC: u64 = 1442695040888963407;
const MULT: u64 = 6364136223846793005;

#[derive(Debug, Clone)]
pub struct GodotRng {
    state: u64,
    inc: u64,
    /// Rolls drawn so far (32-bit outputs), for traces.
    pub draws: u64,
}

impl Default for GodotRng {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

impl GodotRng {
    /// `RandomNumberGenerator.seed = seed` (GDScript ints are i64; the seed is their u64 bit pattern).
    pub fn new(seed: u64) -> Self {
        let mut r = Self { state: 0, inc: (DEFAULT_INC << 1) | 1, draws: 0 };
        r.next_u32();
        r.state = r.state.wrapping_add(seed);
        r.next_u32();
        r.draws = 0;
        r
    }

    pub fn from_i64(seed: i64) -> Self {
        Self::new(seed as u64)
    }

    /// pcg32_random_r.
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULT).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        self.draws += 1;
        xorshifted.rotate_right(rot)
    }

    /// `randi()`: a full 32-bit draw (GDScript sees it as a non-negative int).
    pub fn randi(&mut self) -> i64 {
        self.next_u32() as i64
    }

    /// pcg32_boundedrand_r: uniform in [0, bound).
    fn bounded(&mut self, bound: u32) -> u32 {
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// `randi_range(from, to)`, both ends inclusive.
    pub fn randi_range(&mut self, from: i32, to: i32) -> i32 {
        if from == to {
            return from;
        }
        let span = (from - to).unsigned_abs().wrapping_add(1);
        self.bounded(span) as i32 + from.min(to)
    }

    /// `randf()`: Godot's float in [0, 1] built from two draws (exponent from the first, mantissa from the second),
    /// computed in f32 as the engine does, then widened like a GDScript float.
    pub fn randf(&mut self) -> f64 {
        let proto_exp_offset = self.next_u32();
        if proto_exp_offset == 0 {
            return 0.0;
        }
        let mantissa = (self.next_u32() | 0x8000_0001) as f32;
        ldexpf(mantissa, -32 - proto_exp_offset.leading_zeros() as i32) as f64
    }

    /// `randf_range(from, to)` in f32 arithmetic.
    pub fn randf_range(&mut self, from: f64, to: f64) -> f64 {
        let r = self.randf() as f32;
        (r * (to as f32 - from as f32) + from as f32) as f64
    }
}

fn ldexpf(x: f32, exp: i32) -> f32 {
    // Exact for the ranges used here (results are normal f32 values >= 2^-64).
    (x as f64 * 2f64.powi(exp)) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_first_draws() {
        let mut r = GodotRng::new(0);
        assert_eq!(r.randi(), 881477183);
    }
}
