//! PR5-25 (`FB-187` (5), `docs/sessions/PK2_GRAFTS.md` 2, PK1 3.7) on a scratch catalog built here (no shipped data):
//! a SUPPORT skill's `unmark` (the priest's Inoculated keystone) reaches a party member a unit track's Stage V
//! departure turned `hostile_all`, as Quenched Cinder Salt's item `unmark` does. Expectations come from the rules: only
//! an `unmark` use of a party-side unit reaches the turned member (a heal still needs a friend, an enemy-side user never
//! reaches it); the planner, the round intent target check (a Support round on it while an `unmark` skill or item of
//! the unit reaches it) and the pick menu offer it; the unmark moves the stage back below the departure stage
//! (`sync_tracks`' formula), so the member calms at the next round start and rejoins at its battle-start place; the AI
//! of a party-side unit plans the unmark when it would calm the member (classic and round intent), never when the
//! stage stays at the departure.

use rpgkit_battle::ai;
use rpgkit_battle::command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::mark;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::round_intent::{self, Opener, Pick, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Roster, Session};
use rpgkit_battle::world::{Tactic, UnitId, World};
use rpgkit_data::{Catalog, ItemDef, UnitDef};
use rpgkit_rules::ruleset::{Ruleset, ROUND_INTENT, TIMELINE, TRIGGERS};
use rpgkit_rules::Overrides;
use serde_json::json;
use indexmap::IndexMap;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const TRACK: &str = "t_track";
const MARK: &str = "t_mark";
/// The track's five stages; the last (Stage V) departs.
const TOP: i32 = 5;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_inoculate", "name": "Inoculate", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY", "reach_min": 0, "reach_max": 3,
               "unmark": {"mark": MARK}, "uses_per_battle": 1}),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY", "reach_min": 0, "reach_max": 3, "heal_base": 20}),
    ];
    let stages: Vec<_> = (1..=TOP).map(|n| if n == TOP { json!({"name": format!("Stage {n}"), "departure": "lost"}) } else { json!({"name": format!("Stage {n}")}) }).collect();
    let tracks = vec![json!({"id": TRACK, "name": "Track", "mark": MARK, "cause": "t_cause", "stages": stages})];
    // a light power: nobody falls in the rounds a test plays
    let unit = |id: &str, side: &str, speed: i32, role: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 500, "power": 10, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5, "ai_role": role, "flee_ratio": 0.0})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_tracks": tracks,
        "units": [unit("p_hero", "party", 14, "AGGRESSOR"), unit("p_ally", "party", 12, "SUPPORT"), unit("p_foe", "enemy", 10, "AGGRESSOR")],
    }))
}

struct Plain;

impl Roster for Plain {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, _r: &str) -> Overrides {
        Overrides::default()
    }
}

/// Hero (4,4), the ally below it (4,5), the foe two tiles right of the hero (6,4); the hero holds the test track at
/// `stage`; the ally knows `ally_skills` (plus the basics), the others the basics only. `rules` beside triggers.
fn session(rules: &[&str], stage: i32, ally_skills: &[&str]) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    let rules: Vec<&str> = [TRIGGERS].into_iter().chain(rules.iter().copied()).collect();
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": ["move", "rest", "strike", "guard"]}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &Plain);
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(6, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    let basics = ["strike", "guard", "rest", "move"];
    for c in [HERO, ALLY, FOE] {
        let own: &[&str] = if c == ALLY { ally_skills } else { &[] };
        s.w.units[c].known_skills = basics.iter().chain(own).map(|s| s.to_string()).collect();
    }
    let mut tracks = IndexMap::new();
    tracks.insert(TRACK.to_string(), stage);
    s.w.units[HERO].tracks = Some(tracks);
    s
}

/// A breach of `value` on the hero (the mark recorded as the trigger effect records it, the tracks re-read).
fn breach(w: &mut World, value: i32) -> Vec<String> {
    w.marks.record(HERO, MARK, value);
    mark::sync_tracks(w, HERO, MARK)
}

/// The hero one stage below Stage V, breached to it: turned on everyone.
fn turned(rules: &[&str], ally_skills: &[&str]) -> Session {
    let mut s = session(rules, TOP - 1, ally_skills);
    assert!(breach(&mut s.w, 1).iter().any(|p| p == "p_hero turns on everyone"));
    assert!(mark::turned_ally(&s.w, HERO) && !s.w.allies.contains(&HERO), "Stage V: hostile to all");
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn stage(w: &World) -> i32 {
    w.units[HERO].tracks.as_ref().and_then(|t| t.get(TRACK).copied()).unwrap_or(0)
}

fn salt() -> ItemDef {
    let row = json!({"id": "t_salt", "name": "Salt", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "use_blocks": 1, "use_reach": 1, "unmark": {"mark": MARK}});
    ItemDef::from_obj(row.as_object().unwrap())
}

/// Only an `unmark` use of a party-side unit reaches the turned member: the skill's target check and the planner take
/// it, a heal skill is still for friends, the foe (enemy side) holding the same skill never reaches it, and a plain foe
/// is no target for it.
#[test]
fn only_a_party_side_unmark_reaches_the_turned_member() {
    let mut s = turned(&[], &["t_inoculate", "t_mend"]);
    let (ino, mend) = (sid(&s.w, "t_inoculate"), sid(&s.w, "t_mend"));
    assert_eq!(command::target_problem(&s.w, ALLY, ino, Some(HERO)), "");
    assert_eq!(command::target_problem(&s.w, ALLY, mend, Some(HERO)), "not an ally");
    assert_eq!(command::target_problem(&s.w, FOE, ino, Some(HERO)), "not an ally", "an enemy-side user");
    assert_eq!(command::target_problem(&s.w, ALLY, ino, Some(FOE)), "not an ally", "a foe that never was an ally");
    assert!(mark::unmark_reaches(&s.w, ALLY, MARK, HERO) && !mark::unmark_reaches(&s.w, ALLY, "", HERO));
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&s.w, ALLY, PlanStep::ally(mend, HERO), true), "not an ally");
    assert_eq!(plan.add_step(&s.w, ALLY, PlanStep::ally(ino, HERO), true), "", "the planner takes the unmark");
    // the item rule is the same rule
    s.w.units[ALLY].inventory.pouch.push(salt());
    assert_eq!(command::item_side_problem(&s.w, ALLY, &salt(), Some(HERO)), "");
    assert_ne!(command::item_side_problem(&s.w, FOE, &salt(), Some(HERO)), "");
}

/// The acceptance (classic battle): the ally's planned Inoculate on the Stage V member takes the breach back (the
/// stage drops to the one below the departure, the settle's formula), the member calms at the next round start,
/// rejoins at its battle-start place and is not hostile at the end (so the settle keeps it).
#[test]
fn a_stage_v_member_calms_at_the_next_round_start() {
    let mut s = turned(&[], &["t_inoculate"]);
    let ino = sid(&s.w, "t_inoculate");
    assert_eq!(s.plan.add_step(&s.w, ALLY, PlanStep::ally(ino, HERO), true), "");
    assert!(s.confirm_plan());
    while s.phase != Phase::Replay && !s.combat_over {
        s.step_next();
    }
    let line = format!("p_hero's {MARK} undone; p_hero's Track is at stage {} (Stage {}); p_hero calms at the next round start", TOP - 1, TOP - 1);
    assert!(s.resolver.log.iter().any(|l| l.contains(&line)), "{:?}", s.resolver.log);
    assert_eq!(stage(&s.w), TOP - 1, "below the departure stage");
    assert!(s.w.sides.is_rogue(HERO) && s.w.marks.calming(HERO), "still hostile this round");
    assert_eq!(mark::hostile_at_end(&s.w, HERO), "", "calming: not hostile at the end");
    assert!(s.can_advance_round());
    s.advance_round();
    assert!(!s.w.sides.is_rogue(HERO), "calmed: {:?}", s.resolver.log);
    assert_eq!(s.w.allies, vec![HERO, ALLY], "back at its start place");
    assert_eq!(mark::hostile_at_end(&s.w, HERO), "");
    assert_eq!(mark::uses_of(&s.w, ALLY, "t_inoculate"), 1);
}

/// Rule round_intent: a Support round may take the turned member as its target while an `unmark` of the unit reaches
/// it (the keystone skill, or the Salt in its pouch), never otherwise (no unmark, or the skill used up); the pick menu
/// on it lists the unmark (not the heal) and the pick plans it.
#[test]
fn a_support_round_takes_the_turned_member_for_its_unmark() {
    let mut s = turned(&[ROUND_INTENT], &["t_inoculate", "t_mend"]);
    let (ino, mend) = (sid(&s.w, "t_inoculate"), sid(&s.w, "t_mend"));
    assert_eq!(round_intent::target_problem(&s.w, ALLY, RoundIntent::Support, RoundTarget::Unit(HERO)), "");
    let rows = rip::pick_options(&s.w, &s.plan, ALLY, RoundTarget::Unit(HERO));
    let row = rows.iter().find(|o| o.picks == vec![Pick::Skill { skill: ino, kind: None }]).expect("the unmark row");
    assert_eq!((row.intent, row.problem.clone()), (Some(RoundIntent::Support), None));
    assert!(!rows.iter().any(|o| o.picks == vec![Pick::Skill { skill: mend, kind: None }]), "no heal row on it");
    let (r, _) = rip::pick(&mut s.w, &mut s.plan, ALLY, RoundTarget::Unit(HERO), &[Pick::Skill { skill: mend, kind: None }]);
    assert!(r.is_err(), "the heal is refused");
    let (r, _) = rip::pick(&mut s.w, &mut s.plan, ALLY, RoundTarget::Unit(HERO), &[Pick::Skill { skill: ino, kind: None }]);
    assert_eq!(r, Ok(()));
    assert!(s.plan.entries_for(&s.w, ALLY).iter().any(|e| e.skill == ino && e.target == Some(HERO)), "planned on the member");
    // used up: no unmark reaches it any more
    s.w.skill_uses.insert((ALLY, "t_inoculate".into()), 1);
    assert_eq!(round_intent::target_problem(&s.w, ALLY, RoundIntent::Support, RoundTarget::Unit(HERO)), "Support targets yourself or an ally.");
    // without the skill: refused, the Salt in the pouch reaches it (the item's rule), and the menu lists it
    let mut s = turned(&[ROUND_INTENT], &["t_mend"]);
    assert_eq!(round_intent::target_problem(&s.w, ALLY, RoundIntent::Support, RoundTarget::Unit(HERO)), "Support targets yourself or an ally.");
    s.w.units[ALLY].inventory.pouch.push(salt());
    s.w.add_item_skills();
    assert_eq!(round_intent::target_problem(&s.w, ALLY, RoundIntent::Support, RoundTarget::Unit(HERO)), "");
    let rows = rip::pick_options(&s.w, &s.plan, ALLY, RoundTarget::Unit(HERO));
    assert!(rows.iter().any(|o| o.picks == vec![Pick::Item("t_salt".into())] && o.problem.is_none()), "the Salt row");
    let (r, _) = rip::pick(&mut s.w, &mut s.plan, ALLY, RoundTarget::Unit(HERO), &[Pick::Item("t_salt".into())]);
    assert_eq!(r, Ok(()));
}

/// The classic AI of a party-side unit plans its unmark on the turned member first when it would calm it; not when
/// the stage stays at the departure (two breaches held at the top), not without a mark to undo, never for the foe.
#[test]
fn the_ai_calms_the_turned_member_when_the_unmark_would() {
    let s = turned(&[], &["t_inoculate"]);
    let ino = sid(&s.w, "t_inoculate");
    assert!(mark::would_calm(&s.w, HERO, MARK));
    let mut plan = Plan::new();
    let p = s.w.profile_of(ALLY);
    ai::build_steps(&s.w, &mut plan, ALLY, &p);
    let first = plan.entries_for(&s.w, ALLY).into_iter().next().expect("a planned entry");
    assert_eq!((first.skill, first.target), (ino, Some(HERO)), "the unmark first");
    // the foe knows the unmark too (FB-194 review): an enemy-side user never reaches the turned member
    let mut s = s;
    s.w.units[FOE].known_skills.push("t_inoculate".to_string());
    let mut plan = Plan::new();
    let p = s.w.profile_of(FOE);
    ai::build_steps(&s.w, &mut plan, FOE, &p);
    assert!(!plan.entries_for(&s.w, FOE).iter().any(|e| e.skill == ino), "never the foe's");

    // an overshoot: the unmark leaves Stage V, so the AI does not spend it
    let mut s = turned(&[], &["t_inoculate"]);
    assert!(breach(&mut s.w, 1).is_empty(), "held at the top");
    assert!(!mark::would_calm(&s.w, HERO, MARK));
    let mut plan = Plan::new();
    let p = s.w.profile_of(ALLY);
    ai::build_steps(&s.w, &mut plan, ALLY, &p);
    assert!(!plan.entries_for(&s.w, ALLY).iter().any(|e| e.skill == ino));
    // a member a phase turned without a mark: nothing to undo
    let mut s = session(&[], 0, &["t_inoculate"]);
    assert!(s.w.set_side(HERO, rpgkit_battle::encounter::HOSTILE_ALL));
    assert!(mark::turned_ally(&s.w, HERO) && !mark::would_calm(&s.w, HERO, MARK));
}

/// Rule round_intent: the intent AI opens a Support round on the turned member with its unmark before any tactic
/// (All Out too), and the round plans it.
#[test]
fn the_intent_ai_opens_a_support_round_on_the_turned_member() {
    for tactic in [Tactic::Balanced, Tactic::AllOut] {
        let mut s = turned(&[ROUND_INTENT], &["t_inoculate", "t_mend"]);
        let ino = sid(&s.w, "t_inoculate");
        let mut plan = Plan::new();
        let p = s.w.profile_of(ALLY);
        let r = ai::plan_round_intent(&mut s.w, &mut plan, ALLY, &p, tactic);
        assert_eq!(r.row, ai::ROW_SUPPORT, "{tactic:?}");
        let pick = r.pick.expect("a pick");
        assert_eq!((pick.intent, pick.target, pick.opener), (RoundIntent::Support, RoundTarget::Unit(HERO), Some(Opener::Skill(ino))));
        assert!(plan.entries_for(&s.w, ALLY).iter().any(|e| e.skill == ino && e.target == Some(HERO)), "{tactic:?}");
    }
}
