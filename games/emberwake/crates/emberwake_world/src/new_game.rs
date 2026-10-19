//! NewGame (scripts/game_state/new_game.gd): a fresh GameState from a world and its start party (catalog units or
//! world party characters), plus the playable-world rule of the new-game screen.

use crate::data::Data;
use crate::state::{GameState, PartyState, UnitState, SEED_MAX};
use crate::{dialogue, explore, travel};
use aegis_core::json;

/// A party member from a world character: id = its local id, lore_ref = its qid, passives from its lore.
pub fn hero_from_character(d: &Data, world_id: &str, character_ref: &str) -> Option<UnitState> {
    let c = d.resolve(world_id, "characters", character_ref);
    if c.is_empty() || !d.party_character_errors(world_id, character_ref).is_empty() {
        return None;
    }
    let base = json::s(c, "base_unit");
    let level = json::i(d.entry("units", &base), "level", 1);
    let mut hero = UnitState::at_level(d, &base, level, &json::s(c, "id"));
    hero.lore_ref = json::s(c, "_qid");
    hero.passives = d.character_passives(c, world_id);
    Some(hero)
}

/// Ok(state) or Err(errors). `seed` >= 0 is the save's rng_seed (clamped); -1 = from the clock.
pub fn create(d: &Data, world_id: &str, ids: &[String], seed: i64) -> Result<GameState, Vec<String>> {
    let mut errors = Vec::new();
    if !d.has_world(world_id) {
        errors.push(format!("Unknown world '{world_id}'."));
    }
    if ids.is_empty() {
        errors.push("A new game needs at least one party unit.".into());
    }
    let mut party = PartyState::default();
    for unit_id in ids {
        if !d.has("units", unit_id) && !d.resolve(world_id, "characters", unit_id).is_empty() {
            match hero_from_character(d, world_id, unit_id) {
                None => errors.push(format!("Character '{unit_id}' can't join the party.")),
                Some(h) if party.member(&h.id).is_some() => errors.push(format!("Character '{unit_id}' is listed twice.")),
                Some(h) => party.members.push(h),
            }
            continue;
        }
        if !d.has("units", unit_id) {
            errors.push(format!("Unknown unit '{unit_id}'."));
            continue;
        }
        let level = json::i(d.entry("units", unit_id), "level", 1);
        let mut member_id = unit_id.clone();
        let mut n = 2;
        while party.member(&member_id).is_some() {
            member_id = format!("{unit_id}_{n}");
            n += 1;
        }
        party.members.push(UnitState::at_level(d, unit_id, level, &member_id));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut state = GameState::create(d, world_id, party);
    state.rng_seed = if seed >= 0 { seed.min(SEED_MAX) } else { clock_seed() };
    state.party.gold = dialogue::start_gold(d, world_id);
    state.party.supplies = travel::start_supplies(d, world_id);
    let errors = state.validate(d);
    if errors.is_empty() {
        Ok(state)
    } else {
        Err(errors)
    }
}

fn clock_seed() -> i64 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    (now.as_millis() as i64 ^ now.subsec_micros() as i64) & SEED_MAX
}

/// Empty = playable (a resolved start party and start location).
pub fn playable_errors(d: &Data, world_id: &str) -> Vec<String> {
    if !d.has_world(world_id) {
        return vec![format!("Unknown world '{world_id}'.")];
    }
    let mut errors = Vec::new();
    if d.start_party(world_id).is_empty() {
        errors.push(format!("World '{world_id}' is not playable (no start party)."));
    }
    if explore::start_location(d, world_id).is_empty() {
        errors.push(format!("World '{world_id}' is not playable (no start location)."));
    }
    errors
}

pub fn playable_worlds(d: &Data) -> Vec<String> {
    d.world_ids().into_iter().filter(|w| playable_errors(d, w).is_empty()).collect()
}

pub fn create_for_world(d: &Data, world_id: &str, seed: i64) -> Result<GameState, Vec<String>> {
    let errors = playable_errors(d, world_id);
    if !errors.is_empty() {
        return Err(errors);
    }
    create(d, world_id, &d.start_party(world_id), seed)
}
