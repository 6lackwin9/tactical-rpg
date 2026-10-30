//! ENCOUNTER2-HA on a scratch catalog built here (no shipped data): the party AI of a hold escape (docs/specs/
//! ENCOUNTER2.md 3.1 H6). Expectations come from the rules: the keeper (the holder, or with it down and `fallback` any
//! the hero on a hold tile or nearest to one) walks toward the nearest free hold tile and defends on it; the other heroes
//! draft Withdraw once the gate is open or the keeper reaches a hold tile with this round's move, and draft by their
//! preset before that; sims and autoplay (the default pick, then the Balanced draft every round) win the escape with
//! the holder staying, under the step AI and under the rule `round_intent`. Without a hold escape (or before the pick)
//! no hero has a part and the drafts are the plain fight's.

use indexmap::IndexMap;
use rpgkit_battle::ai::{self, HoldPart};
use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::Mode;
use rpgkit_battle::round_intent::{self, RoundIntent};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

/// The heroes in party order, then the foe (unit ids follow the roster: party, then enemies).
const ASH: UnitId = 0;
const BRYN: UnitId = 1;
const COLE: UnitId = 2;
const DARA: UnitId = 3;
const FOE: UnitId = 4;
const HEROES: [UnitId; 4] = [ASH, BRYN, COLE, DARA];

/// The winch (the one hold tile) in the top right, the exits below it on the right edge.
const WINCH: V2 = V2 { x: 10, y: 1 };

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
    ];
    let unit = |id: &str, name: &str, side: &str| json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": 12, "attack_range": 1, "skill_cap": 5, "flee_ratio": 0.0});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("s_ash", "Ash", "party"), unit("s_bryn", "Bryn", "party"), unit("s_cole", "Cole", "party"), unit("s_dara", "Dara", "party"), unit("s_brute", "Brute", "enemy")],
    }))
}

/// The escape `hold` of the winch (three of four heroes out, `fallback` from `stay`), on an open 12 x 9 floor.
fn row(stay: Value) -> Value {
    let mut sb = json!({"mode": "hold", "tiles": [[WINCH.x, WINCH.y]], "tiles_label": "the winch"});
    for (k, v) in stay.as_object().unwrap() {
        sb[k] = v.clone();
    }
    json!({"id": "hold", "name": "Hold", "rows": vec![".".repeat(12); 9], "party": ["s_ash", "s_bryn", "s_cole", "s_dara"], "enemies": ["s_brute"],
        "exits": [[11, 3], [11, 4], [11, 5]], "objective": "escape", "objective_count": 3, "stay_behind": sb,
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 12, "rng_seed": 1337})
}

/// The round intent ruleset of these tests (Guard is offered through `active_guard`).
fn timeline() -> Ruleset {
    let row = json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT], "basics": ["move", "rest", "strike", "active_guard"]});
    Ruleset::from_row(row.as_object().unwrap())
}

/// The battle of `row` (under `rules` when given) with the units on `cells` (heroes in order, then the foe).
fn session(row: Value, rules: Option<Ruleset>, cells: [V2; 5]) -> Session {
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    if let Some(r) = rules {
        def.ruleset = r;
    }
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in cells.into_iter().enumerate() {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    s
}

/// The heroes in a column on the left, the foe in the bottom left corner.
fn on_the_left() -> [V2; 5] {
    [V2::new(1, 2), V2::new(1, 4), V2::new(1, 5), V2::new(1, 6), V2::new(0, 8)]
}

/// Ash one tile below the winch, the others on the left, the foe in the bottom left corner.
fn ash_by_the_winch() -> [V2; 5] {
    [V2::new(10, 2), V2::new(1, 4), V2::new(1, 5), V2::new(1, 6), V2::new(0, 8)]
}

fn has_mode(s: &Session, c: UnitId, mode: Mode) -> bool {
    s.plan.steps_of(c).iter().any(|st| st.mode == mode)
}

fn defends(s: &Session, c: UnitId) -> bool {
    s.plan.entries_for(&s.w, c).iter().any(|e| skill::kind(&s.w.cat.skills[e.skill]) == Kind::Defend)
}

/// Autoplay to the end: the default pick, the `preset` draft for the whole party each round, confirm, resolve.
fn autoplay(s: &mut Session, preset: &str) {
    for _ in 0..s.def.max_rounds {
        if s.phase == Phase::Replay {
            if !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
        s.stay_pick_default();
        s.plan_apply_preset_all(preset);
        assert!(s.confirm_plan(), "round {}: the party plan is confirmed", s.current_round);
        while s.phase != Phase::Replay {
            s.step_next();
        }
    }
}

fn assert_held_and_escaped(s: &Session) {
    assert_eq!(s.outcome, "won", "the escape is won");
    assert_eq!(s.objective.stayed, vec![ASH]);
    assert_eq!(s.party_fates(), vec![(ASH, PartyFate::Stayed), (BRYN, PartyFate::Escaped), (COLE, PartyFate::Escaped), (DARA, PartyFate::Escaped)]);
}

#[test]
fn autoplay_holds_the_winch_and_gets_the_others_out() {
    let mut s = session(row(json!({})), None, on_the_left());
    autoplay(&mut s, "balanced");
    assert_held_and_escaped(&s);
}

#[test]
fn autoplay_holds_and_escapes_under_round_intent() {
    let mut s = session(row(json!({})), Some(timeline()), on_the_left());
    assert!(round_intent::on(&s.w), "fixture: the rule is on");
    s.stay_pick_default();
    s.plan_apply_preset_all("balanced");
    assert_eq!(round_intent::intent_of(&s.w, ASH), Some(RoundIntent::Guard), "the keeper's round is Guard");
    autoplay(&mut s, "balanced");
    assert_held_and_escaped(&s);
}

#[test]
fn the_keeper_walks_toward_the_winch_then_defends_on_it() {
    let mut s = session(row(json!({})), None, on_the_left());
    assert_eq!(s.stay_pick_default(), Some(ASH));
    let far = s.w.pos(ASH).dist(WINCH);
    assert!(far > s.plan.max_move_tiles(&s.w, ASH), "fixture: one move does not reach the winch");
    s.plan_apply_preset_all("balanced");
    assert_eq!(ai::hold_part(&s.w, &s.plan, ASH), Some(HoldPart::Hold(WINCH)));
    let end = s.plan.compile(&s.w, ASH).pos;
    assert!(end.dist(WINCH) < far, "the keeper closes in on the winch");
    assert!(!has_mode(&s, ASH, Mode::Withdraw) && !has_mode(&s, ASH, Mode::Engage), "the keeper neither leaves nor attacks");
    // on the winch: no move, a Defend
    let mut s = session(row(json!({})), None, [WINCH, V2::new(1, 4), V2::new(1, 5), V2::new(1, 6), V2::new(0, 8)]);
    s.stay_pick_default();
    s.plan_apply_preset_all("balanced");
    assert!(defends(&s, ASH), "the keeper defends on the winch");
    assert_eq!(s.plan.compile(&s.w, ASH).pos, WINCH, "and stays on it");
}

#[test]
fn the_others_leave_once_the_keeper_reaches_the_winch_this_round() {
    // the keeper out of one move's reach: the others draft by their preset (no Withdraw)
    let mut s = session(row(json!({})), None, on_the_left());
    s.stay_pick_default();
    s.plan_apply_preset_all("balanced");
    for c in [BRYN, COLE, DARA] {
        assert_eq!(ai::hold_part(&s.w, &s.plan, c), None, "{} waits", s.w.units[c].name);
        assert!(!has_mode(&s, c, Mode::Withdraw));
    }
    // the keeper one tile from the winch: the gate is still shut, the others draft Withdraw
    let mut s = session(row(json!({})), None, ash_by_the_winch());
    s.stay_pick_default();
    assert!(!s.w.hold.as_ref().unwrap().is_open(&s.w), "fixture: the gate is shut");
    s.plan_apply_preset_all("balanced");
    for c in [BRYN, COLE, DARA] {
        assert_eq!(ai::hold_part(&s.w, &s.plan, c), Some(HoldPart::Out));
        assert!(has_mode(&s, c, Mode::Withdraw), "{} withdraws", s.w.units[c].name);
    }
    assert_eq!(s.plan.compile(&s.w, ASH).pos, WINCH, "the keeper steps onto the winch");
}

#[test]
fn with_the_holder_down_the_nearest_hero_keeps_the_winch() {
    let cells = [V2::new(1, 2), V2::new(8, 2), V2::new(1, 5), V2::new(1, 6), V2::new(0, 8)];
    for (fallback, keeper) in [("any", Some(BRYN)), ("none", None)] {
        let mut s = session(row(json!({"fallback": fallback})), None, cells);
        s.stay_pick(ASH).unwrap();
        let hp = s.w.units[ASH].hp;
        s.w.damage(ASH, hp + 100, true);
        assert!(!s.w.units[ASH].is_alive(), "fixture: the holder is down");
        assert_eq!(ai::hold_keeper(&s.w), keeper, "fallback {fallback}");
        s.plan_apply_preset_all("balanced");
        match keeper {
            Some(k) => {
                assert_eq!(ai::hold_part(&s.w, &s.plan, k), Some(HoldPart::Hold(WINCH)));
                assert_eq!(s.plan.compile(&s.w, k).pos, WINCH, "Bryn steps onto the winch");
                for c in [COLE, DARA] {
                    assert!(has_mode(&s, c, Mode::Withdraw), "{} withdraws", s.w.units[c].name);
                }
            }
            None => {
                for c in HEROES {
                    assert_eq!(ai::hold_part(&s.w, &s.plan, c), None, "nobody can open the gate");
                }
            }
        }
    }
}

#[test]
fn without_a_hold_or_before_the_pick_the_drafts_are_the_plain_fights() {
    let plain = |stay: Value| {
        let mut r = row(json!({}));
        r["stay_behind"] = stay;
        r
    };
    let mut flat = session(plain(json!({})), None, ash_by_the_winch());
    assert!(flat.w.hold.is_none());
    flat.plan_apply_preset_all("balanced");
    let mut held = session(row(json!({})), None, ash_by_the_winch());
    held.plan_apply_preset_all("balanced");
    for c in HEROES.into_iter().chain([FOE]) {
        assert_eq!(ai::hold_part(&flat.w, &flat.plan, c), None);
        assert_eq!(ai::hold_part(&held.w, &held.plan, c), None, "no part before the pick");
        assert_eq!(format!("{:?}", flat.plan.steps_of(c)), format!("{:?}", held.plan.steps_of(c)), "{} drafts as in the plain fight", flat.w.units[c].name);
    }
}
