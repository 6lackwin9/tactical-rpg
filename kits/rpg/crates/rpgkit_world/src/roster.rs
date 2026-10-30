//! The roster (ROSTER1, kit hook; engine-only, no Godot original): a party member's life across battles. How a member
//! takes the field (`field_unit`) and how a battle's end settles the party (`settle_party`) live here, in the world
//! crate, so every screen and runner applies one rule. ROSTER1-L slice 2 moved both from the battle screen with no
//! rule change: the build is the played world's (its class path and deck rule, decision "HOOKS1-UI slice 1 answers")
//! at the member's stored HP, and the settle is HEAL1's classic result (HealRules.hp_after) after ITEMUSE1 rule 7's
//! pouch write-back. The rule rows, wounded and departures (ROSTER1 3.2, 4) extend these two calls.

use crate::data::Data;
use crate::progression;
use crate::state::{GameState, UnitState};
use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::session::Session;
use rpgkit_battle::world::AiProfile;
use rpgkit_data::UnitDef;
use rpgkit_rules::Unit;

/// A member as it takes the field: the battle unit, the saved pouch ids that found no battle slot, its AI profile.
pub struct Fielded {
    pub unit: Unit,
    pub pouch_unloaded: Vec<String>,
    pub profile: AiProfile,
}

/// W2 (classic: no condition changes the build yet): member `m` as it fights in world `world`, the world whose
/// encounter it fights: its class path and deck rule there (`progression::build_played`), at its stored HP
/// (HealRules.start_hp), with its base unit's AI profile. None when the saved member does not build.
pub fn field_unit(d: &Data, m: &UnitState, world: &str) -> Option<Fielded> {
    let built = progression::build_played(d, m, world)?;
    let mut unit = built.unit;
    unit.hp = progression::start_hp(unit.max_hp as i64, m.hp_lost) as i32;
    let base = d.cat.units.get(&m.base_unit).cloned().unwrap_or_else(|| UnitDef::from_obj(&Default::default()));
    Some(Fielded { unit, pouch_unloaded: built.pouch_unloaded, profile: AiProfile::for_unit(&d.cat, &base) })
}

/// How a battle left one fielded member (ROSTER1 3.2's input).
#[derive(Debug, Clone, PartialEq)]
pub struct HeroEnd {
    pub member: String,
    pub hp: i64,
    pub max_hp: i64,
    pub alive: bool,
    /// The battle pouch's unused item ids.
    pub pouch: Vec<String>,
    /// The saved pouch ids that found no battle slot (`Fielded::pouch_unloaded`).
    pub unloaded: Vec<String>,
    /// ENCOUNTER2 S3's fate (None for a unit `party_fates` does not list).
    pub fate: Option<PartyFate>,
}

/// The ends of session `s`'s fielded members: `members` (member id, unloaded pouch ids) in the order they were fielded,
/// paired with the session's allies in order (the heroes come first; guests follow and have no member).
pub fn hero_ends(s: &Session, members: &[(String, Vec<String>)]) -> Vec<HeroEnd> {
    let fates = s.party_fates();
    members
        .iter()
        .zip(&s.w.allies)
        .map(|((member, unloaded), &a)| {
            let u = &s.w.units[a];
            HeroEnd {
                member: member.clone(),
                hp: u.hp as i64,
                max_hp: u.max_hp as i64,
                alive: u.is_alive(),
                pouch: u.inventory.pouch.iter().map(|p| p.id.clone()).collect(),
                unloaded: unloaded.clone(),
                fate: fates.iter().find(|(c, _)| *c == a).map(|(_, f)| *f),
            }
        })
        .collect()
}

/// 3.2, classic (L3; no rule row yet): settles the fielded members after a battle with `outcome`. Every outcome: the
/// unused pouch items go back to the saved pouch, then the ids that found no slot (ITEMUSE1 rule 7, GameEncounter
/// .write_back_pouches). Won and fled: HP is written back against the battle max (HealRules.hp_after: a downed hero
/// rallies on a win, keeps 1 HP on a flee); a lost battle leaves HP to the runner's defeat rule. Call it before the
/// runner's `after_fight` (ROSTER1 L5: the settle comes before CAMPAIGN1's battle tick).
pub fn settle_party(_d: &Data, gs: &mut GameState, outcome: &str, ends: &[HeroEnd]) {
    for e in ends {
        if let Some(m) = gs.party.member_mut(&e.member) {
            m.pouch = e.pouch.iter().chain(&e.unloaded).cloned().collect();
        }
    }
    if outcome != "won" && outcome != "fled" {
        return;
    }
    for e in ends {
        if let Some(m) = gs.party.member_mut(&e.member) {
            let hp = progression::hp_after(if e.alive { e.hp.min(e.max_hp) } else { 0 }, e.max_hp, outcome);
            m.hp_lost = (e.max_hp - hp).max(0);
        }
    }
}
