//! GameRolls (scripts/game_state/game_rolls.gd): per-save deterministic streams. Roll n of stream s is one randi()
//! of a RandomNumberGenerator seeded with hash([rng_seed, s, n]); every roll counts gs.rolls[s] up by one.

use crate::state::GameState;
use aegis_core::ghash::{hash, GVar};
use aegis_core::json::{self, int};
use aegis_core::rng::GodotRng;
use serde_json::Value;

pub const EXPLORE: &str = "explore";
pub const TRAVEL: &str = "travel";
/// REWARD1: one draw 0..99 per defeated unit whose base unit has a loot table.
pub const LOOT: &str = "loot";
/// T1.2-S2 (FB-54b): one draw 1..dialogue_roll_die per dialogue `roll` effect, at choice time only (never on a redraw).
pub const DIALOGUE: &str = "dialogue";

pub fn roll(gs: &mut GameState, stream: &str) -> i64 {
    let n = gs.rolls.get(stream).map(json::to_int).unwrap_or(0);
    let seed = hash(&GVar::Arr(vec![GVar::Int(gs.rng_seed), GVar::Str(stream.into()), GVar::Int(n)]));
    let mut rng = GodotRng::from_i64(seed as i64);
    let value = rng.randi();
    gs.rolls.insert(stream.into(), int(n + 1));
    value
}

/// An int in lo..hi inclusive (bounds in either order). One roll.
pub fn range_i(gs: &mut GameState, stream: &str, lo: i64, hi: i64) -> i64 {
    let (a, b) = (lo.min(hi), lo.max(hi));
    a + roll(gs, stream).rem_euclid(b - a + 1)
}

pub fn chance(gs: &mut GameState, stream: &str, pct: i64) -> bool {
    range_i(gs, stream, 0, 99) < pct
}

fn weight(p: &Value) -> i64 {
    let pair = json::arr(p);
    if pair.len() == 2 {
        json::to_int(&pair[1])
    } else {
        0
    }
}

/// One roll over [[id, weight], ...] in authored order; "" and no roll when nothing is pickable.
pub fn pick(gs: &mut GameState, stream: &str, pairs: &[Value]) -> String {
    let total: i64 = pairs.iter().map(|p| weight(p).max(0)).sum();
    if total <= 0 {
        return String::new();
    }
    let mut r = range_i(gs, stream, 0, total - 1);
    for p in pairs {
        let w = weight(p).max(0);
        if r < w {
            return json::gstr(&json::arr(p)[0]);
        }
        r -= w;
    }
    String::new()
}
