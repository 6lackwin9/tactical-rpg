//! The term resolver with a world (TERMS1 TM3-TM6; engine-only, no Godot original): kit code with a world reads a
//! game-system word here, by its neutral id (`rpgkit_data::terms`). A `world` term (the currency, the upkeep) is the
//! world's purse block (CAMPAIGN1 P1, read through the lineage in a campaign world), else its default; a `game` term is
//! the game's `terms` row (`Catalog::term`), else its default. With no data every text is today's (TM2). The kit's
//! sentences that name the currency or the upkeep are built here (TM6: labels use `short`, sentences `word`, a sentence
//! that starts with the term `short`).

use crate::campaign::purse::purse;
use crate::data::Data;
use crate::state::{GOLD_MAX, SUPPLIES_MAX};
pub use rpgkit_data::terms::*;
use rpgkit_data::PurseCounter;

/// TM4: the world block that names `world` term `id` in world `w` (the purse counters), None for a `game` term.
fn world_block(d: &Data, w: &str, id: &str) -> Option<PurseCounter> {
    match id {
        CURRENCY => Some(purse(d, w).currency),
        UPKEEP => Some(purse(d, w).upkeep),
        _ => None,
    }
}

/// TM3, TM4, TM5: term `id` in world `w` ("" = no world: the defaults for the `world` terms).
pub fn term(d: &Data, w: &str, id: &str) -> Term {
    match world_block(d, w, id) {
        Some(c) => Term::resolve(id, &c.name, &c.short, "", &c.tip),
        None => d.cat.term(id),
    }
}

/// The title name of term `id` in world `w` ("Gold", "Coin").
pub fn name(d: &Data, w: &str, id: &str) -> String {
    term(d, w, id).name
}

/// The short form of term `id` in world `w` (HUD, labels, Results lines: "Gold +12").
pub fn short(d: &Data, w: &str, id: &str) -> String {
    term(d, w, id).short
}

/// The word of term `id` in world `w` inside a sentence ("gold", "Coin").
pub fn word(d: &Data, w: &str, id: &str) -> String {
    term(d, w, id).word
}

// ---- the kit's sentences (TU1) -----------------------------------------------------------------------------------

/// A buy or a take the party cannot pay: "Not enough gold.".
pub fn not_enough_currency(d: &Data, w: &str) -> String {
    format!("Not enough {}.", word(d, w, CURRENCY))
}

/// Gold that does not fit the purse: "Gold is capped at 9999999 (12 not added).".
pub fn currency_capped(d: &Data, w: &str, lost: i64) -> String {
    format!("{} is capped at {GOLD_MAX} ({lost} not added).", short(d, w, CURRENCY))
}

/// A dialogue option's currency lock: "Needs 10 gold".
pub fn needs_currency(d: &Data, w: &str, n: i64) -> String {
    format!("Needs {n} {}", word(d, w, CURRENCY))
}

/// A supply buy beyond the cap: "You can carry at most 99 supplies.".
pub fn upkeep_carry_max(d: &Data, w: &str) -> String {
    format!("You can carry at most {SUPPLIES_MAX} {}.", word(d, w, UPKEEP))
}

/// TM6: supplies that do not fit: "Supplies are capped at 99 (3 left behind)." while the upkeep has its default name
/// (the verb agrees with it), else "`{short}`: capped at 99 (3 left behind).".
pub fn upkeep_capped(d: &Data, w: &str, lost: i64) -> String {
    let t = term(d, w, UPKEEP);
    if t.name == default_name(UPKEEP) {
        format!("{} are capped at {SUPPLIES_MAX} ({lost} left behind).", t.short)
    } else {
        format!("{}: capped at {SUPPLIES_MAX} ({lost} left behind).", t.short)
    }
}
