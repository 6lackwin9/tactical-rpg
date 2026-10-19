//! Godot 4's `hash()` for the value shapes the game hashes (core/templates/hashfuncs.h, Variant::recursive_hash,
//! Array::recursive_hash, String::hash), so seeds derived from hashes (GameRolls, purse rolls) match the reference.

const MURMUR3_SEED: u32 = 0x7F07C65;
/// Variant::ARRAY in Godot 4's Variant::Type enum.
const VARIANT_ARRAY: u32 = 28;

#[derive(Debug, Clone, PartialEq)]
pub enum GVar {
    Int(i64),
    Str(String),
    Arr(Vec<GVar>),
}

fn murmur3_one_32(input: u32, seed: u32) -> u32 {
    let mut k = input.wrapping_mul(0xcc9e2d51);
    k = k.rotate_left(15);
    k = k.wrapping_mul(0x1b873593);
    let mut h = seed ^ k;
    h = h.rotate_left(13);
    h.wrapping_mul(5).wrapping_add(0xe6546b64)
}

fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

/// hash_one_uint64 (Thomas Wang's 64 -> 32 bit hash).
fn one_u64(v: u64) -> u32 {
    let mut v = (!v).wrapping_add(v << 18);
    v ^= v >> 31;
    v = v.wrapping_mul(21);
    v ^= v >> 11;
    v = v.wrapping_add(v << 6);
    v ^= v >> 22;
    v as u32
}

/// String::hash (djb2 over UTF-32 code points).
pub fn string_hash(s: &str) -> u32 {
    let mut h: u32 = 5381;
    for c in s.chars() {
        h = (h << 5).wrapping_add(h).wrapping_add(c as u32);
    }
    h
}

/// GDScript `hash(value)` (a non-negative 32-bit result).
pub fn hash(v: &GVar) -> u32 {
    match v {
        GVar::Int(i) => one_u64(*i as u64),
        GVar::Str(s) => string_hash(s),
        GVar::Arr(items) => {
            let mut h = murmur3_one_32(VARIANT_ARRAY, MURMUR3_SEED);
            for it in items {
                h = murmur3_one_32(hash(it), h);
            }
            fmix32(h)
        }
    }
}
