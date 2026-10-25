//! Engine core: game-agnostic building blocks every crate uses. Godot's RNG, `hash()` and sort, bit for bit, SHA-256,
//! the Variant helpers and lenient field readers over serde_json values, a data formula evaluator. Knows nothing about any game's rules or
//! content (docs/specs/SPLIT1.md section 2).

pub mod assets;
pub mod fmt;
pub mod formula;
pub mod ghash;
pub mod json;
pub mod rng;
pub mod sha256;
pub mod sort;
pub mod value;
