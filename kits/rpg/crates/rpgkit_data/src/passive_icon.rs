//! PASSIVE-ICON1 (ENGINE-24, STATUSVIS1 SV10 and SV12): a passive's icon archetype. A passive row's `icon` names one of
//! the kit's archetypes (`ARCHETYPES`, one file each at `passives/<key>.png`, `kits/rpg/assets/passives/README.md`);
//! empty means the archetype is derived from the passive's effects (`derived`): the first effect that is not a setup
//! step (a meter the passive holds, a skill it grants) decides it. The kit names no game.

use crate::PassiveDef;
use aegis_core::json;
use serde_json::Value;

/// The archetype keys, in the README's order.
pub const ARCHETYPES: [&str; 16] = [
    "weapon_drill",
    "armour_drill",
    "might",
    "intellect",
    "perception",
    "agility",
    "poise_up",
    "resist",
    "ap_gain",
    "crit_up",
    "heal_up",
    "status_potency",
    "plan_sight",
    "immovable",
    "cover_ally",
    "last_stand",
];

/// The archetype a passive with no effect to read shows.
pub const FALLBACK: &str = "might";

/// The kit's armour weights: a proficiency in one of them is an armour drill, any other gear a weapon drill.
const ARMOUR_WEIGHTS: [&str; 3] = ["heavy", "medium", "light"];

/// Archetype `key`'s icon file (an asset key in the game, kit or engine layers).
pub fn path(key: &str) -> String {
    format!("passives/{key}.png")
}

/// Whether `key` is one of the kit's archetypes.
pub fn known(key: &str) -> bool {
    ARCHETYPES.contains(&key)
}

/// The archetype passive `p` shows: its `icon` when set, else the one derived from its effects.
pub fn of(p: &PassiveDef) -> String {
    if p.icon.is_empty() {
        derived(&p.effects)
    } else {
        p.icon.clone()
    }
}

/// The archetype of an effect list: the first effect that is not a setup step decides; none: `FALLBACK`.
pub fn derived(effects: &[Value]) -> String {
    effects.iter().map(json::dict).filter(|e| !matches!(json::s(e, "type").as_str(), "meter" | "skill")).find_map(of_effect).unwrap_or(FALLBACK).to_string()
}

/// One effect's archetype (None for a setup step).
fn of_effect(e: &json::Dict) -> Option<&'static str> {
    let group = json::s(e, "group");
    Some(match json::s(e, "type").as_str() {
        "meter" | "skill" => return None,
        "proficiency" if ARMOUR_WEIGHTS.contains(&json::s(e, "gear").as_str()) => "armour_drill",
        "proficiency" | "forbid_hands" | "pierce_melee" => "weapon_drill",
        "stat" => match json::s(e, "stat").as_str() {
            "INT" | "KNOW" => "intellect",
            "PERC" => "perception",
            "DEX" | "move_rate" => "agility",
            "max_poise" => "poise_up",
            "armor" | "composure" => "resist",
            _ => "might",
        },
        "resist" => "resist",
        "stagger_taken" => "poise_up",
        "forced_move" if !json::s(e, "immune").is_empty() => "immovable",
        "forced_move" => "might",
        "foresight" => "plan_sight",
        "crit" => "crit_up",
        "ap_regen" => "ap_gain",
        "status_potency" => "status_potency",
        "skill_reach" => "perception",
        "skill_potency" | "unlock" if group == "heal" => "heal_up",
        "skill_potency" | "unlock" if group == "spell" => "intellect",
        "skill_potency" => "might",
        "unlock" => "weapon_drill",
        "trigger" => match json::s(e, "on").as_str() {
            "hp_below" => "last_stand",
            "ally_hit" | "guard_start" => "cover_ally",
            "heal_given" | "cleanse_given" => "heal_up",
            "evaded" => "agility",
            _ if json::aget(e, "effects").iter().any(|x| json::s(json::dict(x), "type") == "ap") => "ap_gain",
            _ => "status_potency",
        },
        _ => FALLBACK,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn passive(icon: &str, effects: Value) -> PassiveDef {
        PassiveDef { id: "p".into(), name: "P".into(), effects: json::arr(&effects).to_vec(), slot: String::new(), icon: icon.into() }
    }

    #[test]
    fn the_icon_field_wins_and_an_empty_one_is_derived() {
        assert_eq!(of(&passive("last_stand", json!([{"type": "crit", "value": 5}]))), "last_stand");
        assert_eq!(of(&passive("", json!([{"type": "crit", "value": 5}]))), "crit_up");
        assert_eq!(of(&passive("", json!([]))), FALLBACK);
        assert!(ARCHETYPES.iter().all(|k| known(k)) && !known("nope"));
        assert_eq!(path("resist"), "passives/resist.png");
    }

    #[test]
    fn the_first_effect_past_the_setup_steps_decides() {
        let cases = [
            (json!([{"type": "proficiency", "gear": "sword", "levels": 1}]), "weapon_drill"),
            (json!([{"type": "proficiency", "gear": "heavy", "levels": 2}]), "armour_drill"),
            (json!([{"type": "stat", "stat": "CONS", "value": 2}]), "might"),
            (json!([{"type": "stat", "stat": "KNOW", "value": 1}]), "intellect"),
            (json!([{"type": "stat", "stat": "PERC", "value": 1}]), "perception"),
            (json!([{"type": "stat", "stat": "DEX", "value": 1}]), "agility"),
            (json!([{"type": "stat", "stat": "max_poise", "value": 6}]), "poise_up"),
            (json!([{"type": "resist", "status": "burn", "value": 50}]), "resist"),
            (json!([{"type": "stagger_taken", "mult": 0}]), "poise_up"),
            (json!([{"type": "forced_move", "immune": "guarding"}]), "immovable"),
            (json!([{"type": "forced_move", "bonus_tiles": 1}]), "might"),
            (json!([{"type": "foresight"}]), "plan_sight"),
            (json!([{"type": "ap_regen", "value": 1}]), "ap_gain"),
            (json!([{"type": "status_potency", "value": 5}]), "status_potency"),
            (json!([{"type": "skill_potency", "group": "heal", "value": 0.1}]), "heal_up"),
            (json!([{"type": "unlock", "group": "spell", "levels": 1}]), "intellect"),
            (json!([{"type": "trigger", "on": "hp_below", "effects": []}]), "last_stand"),
            (json!([{"type": "trigger", "on": "ally_hit", "effects": []}]), "cover_ally"),
            (json!([{"type": "trigger", "on": "kill", "effects": [{"type": "ap", "value": 2}]}]), "ap_gain"),
            (json!([{"type": "meter", "meter": "m"}, {"type": "trigger", "on": "cleanse_given", "effects": []}]), "heal_up"),
            (json!([{"type": "meter", "meter": "m"}, {"type": "skill", "skill": "s"}]), FALLBACK),
            (json!([{"type": "something_new"}]), FALLBACK),
        ];
        for (effects, want) in cases {
            assert_eq!(derived(json::arr(&effects)), want, "{effects}");
        }
    }
}
