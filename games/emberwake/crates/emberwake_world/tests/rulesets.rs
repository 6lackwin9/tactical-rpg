//! RULESET1-R on the shipped data: every shipped world resolves to the ruleset its manifest (or nearest ancestor) names, classic when unset, and the three classic worlds stay classic (3.1.1); a battle
//! plays by the ruleset its caller gives it (3.1.3); under `round_intent` the classic stance machinery is inert (intent
//! IN3: no stance change, no switch AP, no flow stance, no stance break), in battles and in the Dojo, while classic keeps
//! it; the register's extensions change nothing in a classic battle until their rules land.

use emberwake_battle::dojo::DojoSession;
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_battle::skill;
use emberwake_rules::ruleset::{Ruleset, CLASSIC, REGISTER, ROUND_INTENT, TIMELINE};
use emberwake_rules::Stance;
use emberwake_world::Data;
use indexmap::IndexMap;
use serde_json::json;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// A timeline ruleset with `round_intent` only (no data row needed).
fn round_intent() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT]}).as_object().unwrap())
}

/// The first catalog encounter with a party and enemies, by rule (no shipped id named).
fn encounter(d: &Data) -> EncounterDef {
    let e = d.cat.encounters.values().map(EncounterDef::from_obj).find(|e| !e.party.is_empty() && !e.enemies.is_empty()).expect("a catalog encounter with both sides");
    assert_eq!(e.ruleset, Ruleset::classic(), "a row alone is classic");
    e
}

/// Autoplays up to `rounds` rounds; per round, every scheduled action as (stance switch AP, enters a stance, flow
/// stance, breaks stances, attack) and every unit's stance after the round; plus the whole log.
#[allow(clippy::type_complexity)]
fn play(d: &Data, ruleset: Ruleset, rounds: i32) -> (Vec<(i32, Option<Stance>, Option<Stance>, bool, bool)>, Vec<Stance>, Vec<String>) {
    let mut def = encounter(d);
    def.ruleset = ruleset;
    let mut s = Session::start(d.cat.clone(), def, &IndexMap::new());
    let (mut actions, mut stances) = (Vec::new(), Vec::new());
    for _ in 0..rounds {
        s.plan_apply_preset_all("balanced");
        if !s.confirm_plan() {
            break;
        }
        while s.phase != Phase::Replay {
            s.step_next();
        }
        for a in &s.resolver.scheduled {
            let attack = matches!(a.command.kind, emberwake_battle::command::CmdKind::Attack { .. });
            actions.push((a.command.stance_switch_cost, a.command.stance_on_start, a.command.stance_on_end, a.command.disrupt_break, attack));
        }
        stances.extend(s.w.units.iter().map(|u| u.stance));
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    (actions, stances, s.resolver.log.clone())
}

#[test]
fn every_shipped_world_plays_its_manifest_ruleset() {
    let d = data();
    for w in d.world_ids() {
        // the nearest manifest in the world's lineage (itself first) that names a ruleset; classic when none does
        let want = d.lineage(&w).iter().rev().find_map(|lw| d.manifest(lw).get("ruleset").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string)).unwrap_or_else(|| CLASSIC.to_string());
        assert_eq!(d.ruleset_id(&w), want, "{w}");
        let r = d.ruleset(&w);
        if want == CLASSIC {
            assert_eq!(r, Ruleset::classic(), "{w}");
        } else {
            assert_eq!(r.id, want, "{w}: the ruleset row is found, not the classic fallback");
        }
    }
}

#[test]
fn the_classic_worlds_stay_classic() {
    let d = data();
    for w in ["ashlands", "iron_reaches", "tidewrack"] {
        assert!(d.has_world(w), "{w} ships");
        assert_eq!(d.ruleset_id(w), CLASSIC, "{w}");
        assert_eq!(d.ruleset(w), Ruleset::classic(), "{w}");
    }
}

#[test]
fn round_intent_leaves_the_classic_stances_inert_in_battle() {
    let d = data();
    let (actions, stances, _) = play(&d, round_intent(), 3);
    assert!(actions.iter().any(|a| a.4), "the fight has attacks");
    assert!(actions.iter().all(|a| a.0 == 0 && a.1.is_none() && a.2.is_none() && !a.3), "no switch AP, no stance entered, no flow, no break");
    assert!(stances.iter().all(|s| *s == Stance::Neutral), "every unit stays Neutral");
    // classic keeps the machinery: an attack enters the Offensive stance (SkillKinds stance rule)
    let (classic, _, _) = play(&d, Ruleset::classic(), 3);
    assert!(classic.iter().any(|a| a.4 && a.1 == Some(Stance::Offensive)), "classic attacks enter Offensive");
}

#[test]
fn extensions_change_nothing_in_a_classic_battle_yet() {
    let d = data();
    let mut all = Ruleset::classic();
    let on: Vec<String> = REGISTER.iter().filter(|r| r.kind == emberwake_rules::ruleset::RuleKind::Extension).map(|r| format!("+{}", r.id)).collect();
    all.adjust(&on.join(",")).unwrap();
    assert_eq!(all.rules.len(), on.len());
    assert_eq!(play(&d, all, 4), play(&d, Ruleset::classic(), 4), "the same actions, stances and log");
}

#[test]
fn the_dojo_follows_its_ruleset() {
    let d = data();
    let mut s = DojoSession::new(d.cat.clone());
    let attack = s.cat.skills.iter().find(|x| x.kind == "ATTACK").cloned().expect("an attack");
    assert_eq!(s.skill_ap(&attack), skill::ap_cost(&attack, Some(s.hero())) + skill::switch_cost(&attack, Stance::Neutral), "classic: the switch AP");
    s.set_ruleset(round_intent());
    assert_eq!(s.log.last().map(|e| e.title.as_str()), Some("Ruleset"));
    assert_eq!(s.skill_ap(&attack), skill::ap_cost(&attack, Some(s.hero())), "timeline: no switch AP");
    s.select_skill(&attack.id);
    s.use_skill(None);
    assert_eq!(s.hero().stance, Stance::Neutral, "using an attack keeps Neutral");
    s.shift_stance();
    assert_eq!((s.hero().stance, s.log.last().map(|e| e.outcome.as_str())), (Stance::Neutral, Some("FAILED")), "no stance shift");
    s.reset();
    assert_eq!(s.ruleset, round_intent(), "a reset keeps the ruleset");
    s.set_ruleset(Ruleset::classic());
    s.shift_stance();
    assert_eq!(s.hero().stance, Stance::Offensive, "classic shifts Neutral -> Offensive");
}
