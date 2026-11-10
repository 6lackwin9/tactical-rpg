//! PK1-S2a (`docs/sessions/PK1_CORRUPTION.md` 4, FB-167 Q1-Q3; rule `channel`) on a scratch catalog built here (no
//! shipped data). Expectations come from the rules and the rows below: only a holder of the channel meter can Channel,
//! the round flag starts every round off; a marrow version exists for a single-intent skill (never a dual-intent one,
//! a neutral kind or `channel.none`); a Channel round offers only the marrow-only skills, the class basic and the
//! actives marked marrow (at most `marrow_actives`), a Cold Steel round no marrow-only skill; the derived cost is
//! `channel_cost` by AP (or `channel.cost`), paid at the start, with power x `channel_power` and armour piercing; the
//! converting toggle keeps Moves and Rest picks, keeps each skill that has a version in the new mode, drops the others
//! and refills the freed blocks (Feed as the Channel Rest); the first skill fixes the round's one target; Vent's gate
//! (Strain 50 at its start) and its percent HP cost; a Channel round skips decay; a battle with the rule on and no
//! holder plays exactly as one without it.

use rpgkit_battle::channel;
use rpgkit_battle::command::{self, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, Pick, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, UnitDef};
use rpgkit_rules::ruleset::{self, Ruleset, AP_COST, CHANNEL, DECK, FIELD_INTENT, HP_COST, PIERCE, ROUND_INTENT, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE2: UnitId = 3;
/// The test rows' numbers (the expectations derive from them).
const FEED_GAIN: i32 = 5;
const VENT_DROP: i32 = 20;
const VENT_GATE: i32 = 50;
const VENT_PCT: i32 = 15;
const DECAY: i32 = 10;
const OWN_COST: i32 = 7;

fn attack(id: &str, ap: i32, extra: Value) -> Value {
    let mut o = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": ap, "power_mult": 0.5, "target_mode": "ENEMY"});
    for (k, v) in extra.as_object().unwrap() {
        o[k] = v.clone();
    }
    o
}

/// Engine skills, the universal basics (Strike, Guard), the class basic Bash, the actives (Jab 1 AP, Heavy 2 AP, Huge 3 AP,
/// Vast 4 AP, Cleave an ARC: Field and Assault, Plain with `channel.none`, Spike marrow-only, Own with its own cost),
/// and the strain row's skills Feed (+5, self heal) and Vent (-20, gate 50, 15% HP), both marrow-only.
fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let path = json!({"t_jab": 1, "t_heavy": 1, "t_huge": 1, "t_vast": 1, "t_cleave": 1, "t_plain": 1, "t_spike": 1, "t_own": 1, "t_brace": 1});
    let unit = |id: &str, side: &str, deck: Value| json!({"id": id, "name": id, "class_id": "marrowkin", "level": 3, "side": side, "skill_cap": 5, "default_intent": "assault", "deck": deck});
    let hero_deck = json!({"active": ["t_jab", "t_heavy", "t_cleave", "t_plain", "t_spike", "t_brace"], "marrow": ["t_jab", "t_heavy"]});
    Catalog::from_value(&json!({
        "classes": [{"id": "marrowkin", "name": "Marrowkin", "base_stats": stats, "growth": {}, "proficiency": {}, "skill_path": path, "basics": ["t_bash"]}],
        "units": [unit("p_hero", "party", hero_deck), unit("p_ally", "party", json!({})), unit("p_foe", "enemy", json!({})), unit("p_foe2", "enemy", json!({}))],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "ap_cost": 1, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "ap_cost": 1, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            attack("t_bash", 1, json!({"learned_only": true})),
            attack("t_jab", 1, json!({})),
            attack("t_heavy", 2, json!({})),
            attack("t_huge", 3, json!({})),
            attack("t_vast", 4, json!({})),
            attack("t_cleave", 1, json!({"area": "ARC"})),
            attack("t_plain", 1, json!({"channel": {"none": true}})),
            attack("t_spike", 1, json!({"channel": {"only": true}})),
            attack("t_own", 1, json!({"channel": {"cost": OWN_COST, "power": 2.0}})),
            {"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 1, "ap_cost": 1, "mitigation": 0.3},
            {"id": "t_feed", "name": "Feed", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "heal_base": 3, "learned_only": true,
                "channel": {"only": true}, "meters": [{"meter": "strain", "delta": FEED_GAIN}]},
            {"id": "t_vent", "name": "Vent", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "learned_only": true,
                "channel": {"only": true}, "meters": [{"meter": "strain", "delta": -VENT_DROP}], "requires_meter": {"meter": "strain", "min": VENT_GATE}, "hp_cost_pct": VENT_PCT}
        ],
        "unit_meters": [{"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "start": 0, "show": "bar",
            "decay": [{"delta": -DECAY, "unless_gained": true}, {"delta": -DECAY, "if_guarded": true}], "skills": ["t_feed", "t_vent"]}],
    }))
}

/// The hero holds the strain meter (a passive's `meter` effect); nobody else does (`holder` false: no one).
struct Holders(bool);

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if self.0 && r == "p_hero" { vec!["inoculated".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn rules(channel_on: bool, extra: Value) -> Ruleset {
    let mut list = vec![ROUND_INTENT, DECK, AP_COST, PIERCE, FIELD_INTENT, TRIGGERS, UNIT_METERS, HP_COST];
    if channel_on {
        list.push(CHANNEL);
    }
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": list, "basics": ["move", "rest", "strike", "guard"], "deck_slots": {"active": 6, "passive": 0, "keystone": 0},
        "channel_meter": "strain", "channel_rest": "t_feed"});
    if !channel_on {
        // a row without the rule carries no Channel fields either
        let o = row.as_object_mut().unwrap();
        o.remove("channel_meter");
        o.remove("channel_rest");
    }
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    Ruleset::from_row(row.as_object().unwrap())
}

/// Hero (4,4) faces the foe (5,4); the ally beside the hero at (4,5), the second foe at (5,6).
fn session_with(ruleset: Ruleset, holder: bool) -> Session {
    let row = json!({"id": "ch_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe2"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 4242});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut cat = catalog();
    cat.passives.insert("inoculated".into(), PassiveDef { id: "inoculated".into(), name: "Inoculated".into(), effects: vec![json!({"type": "meter", "meter": "strain"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Holders(holder));
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(5, 4)), (FOE2, V2::new(5, 6))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn session() -> Session {
    session_with(rules(true, json!({})), true)
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn skill<'a>(w: &'a World, id: &str) -> &'a rpgkit_data::SkillDef {
    &w.cat.skills[sid(w, id)]
}

fn set_strain(w: &mut World, c: UnitId, v: i32) {
    w.meters.gauges.iter_mut().find(|g| g.unit == c && g.meter == "strain").expect("a gauge").value = v;
}

fn flag(w: &mut World, c: UnitId, on: bool) {
    w.round_states.entry(c).or_default().channel = on;
}

fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

/// The skill ids of `c`'s planned steps, in order.
fn planned(s: &Session, c: UnitId) -> Vec<String> {
    s.plan.steps_of(c).iter().filter_map(|st| st.skill).map(|k| s.w.cat.skills[k].id.clone()).collect()
}

/// `c`'s menu cell of skill `id` on `at`: its problem (None: it can be picked).
fn cell(s: &Session, c: UnitId, at: RoundTarget, id: &str) -> Option<String> {
    let k = sid(&s.w, id);
    rip::menu_options(&s.w, &s.plan, c, at).into_iter().find(|o| matches!(o.picks.last(), Some(Pick::Skill { skill, .. }) if *skill == k)).unwrap_or_else(|| panic!("a cell for {id}")).problem
}

/// PK1 4.2, 4.3 item 1: only a unit holding the channel meter can Channel (the toggle refuses another unit and changes
/// nothing); the toggle is free to flip both ways; the flag resets at the round start.
#[test]
fn only_a_holder_channels_and_every_round_starts_in_cold_steel() {
    let mut s = session();
    assert!(channel::can_channel(&s.w, HERO), "the hero holds strain");
    assert!(!channel::can_channel(&s.w, ALLY), "the ally holds none");
    let (r, _) = s.set_channel(ALLY, true);
    assert_eq!(r, Err("p_ally cannot Channel.".to_string()));
    assert!(!channel::is_channel(&s.w, ALLY));
    assert_eq!(s.set_channel(HERO, true).0, Ok(Vec::new()));
    assert!(channel::is_channel(&s.w, HERO));
    assert_eq!(s.set_channel(HERO, false).0, Ok(Vec::new()), "never locked");
    assert!(!channel::is_channel(&s.w, HERO));
    let _ = s.set_channel(HERO, true);
    round_intent::round_start(&mut s.w);
    assert!(!channel::is_channel(&s.w, HERO), "cleared at the round start");
    // without the rule nobody can Channel
    let off = session_with(rules(false, json!({})), true);
    assert!(!channel::can_channel(&off.w, HERO));
}

/// PK1 4.2: a marrow version exists for a single-intent skill only: never a dual-intent one (Cleave's ARC: Field and
/// Assault), a neutral kind, `channel.none` or a marrow-only skill (it has no Cold Steel version). In a Channel round
/// the marrow-only skills, the class basic and the first `marrow_actives` marked actives open; the universal basic and
/// an unmarked active do not.
#[test]
fn eligibility_is_single_intent_and_the_round_opens_its_marrow_set() {
    let mut s = session();
    let w = &s.w;
    assert!(channel::has_version(w, skill(w, "t_jab")));
    assert_eq!(round_intent::skill_rows(w, skill(w, "t_cleave")).len(), 2, "Cleave is dual intent");
    for id in ["t_cleave", "t_plain", "t_spike", "move", "rest"] {
        assert!(!channel::has_version(w, skill(w, id)), "{id} has no marrow version");
    }
    assert_eq!(channel::marked(w, HERO), ["t_jab"], "marrow_actives 1: the first mark");
    flag(&mut s.w, HERO, true);
    let w = &s.w;
    for id in ["t_jab", "t_bash", "t_spike", "t_feed", "t_vent", "move", "rest"] {
        assert!(channel::opens(w, HERO, skill(w, id)), "{id} opens");
        assert_eq!(channel::problem(w, HERO, skill(w, id)), None);
    }
    for id in ["t_heavy", "strike", "guard", "t_cleave", "t_plain"] {
        assert_eq!(channel::problem(w, HERO, skill(w, id)), Some(channel::CHANNEL_ROUND), "{id}");
    }
    // a second mark opens with marrow_actives 2
    let mut two = session_with(rules(true, json!({"marrow_actives": 2})), true);
    assert_eq!(channel::marked(&two.w, HERO), ["t_jab", "t_heavy"]);
    flag(&mut two.w, HERO, true);
    assert_eq!(channel::problem(&two.w, HERO, skill(&two.w, "t_heavy")), None);
    // a Cold Steel round greys the marrow-only skills
    let cold = session();
    assert_eq!(channel::problem(&cold.w, HERO, skill(&cold.w, "t_spike")), Some(channel::ONLY_CHANNEL));
    assert_eq!(channel::problem(&cold.w, HERO, skill(&cold.w, "t_jab")), None);
}

/// PK1 4.1, 4.2: the derived cost is `channel_cost` by the skill's AP (1, 2, 3+; 0 AP none), `channel.cost` replaces it;
/// power x `channel_power` (or `channel.power`); a marrow-only skill pays only its own `channel.cost`, at power 1.
#[test]
fn the_marrow_cost_is_by_ap_and_its_overrides_hold() {
    let mut s = session();
    flag(&mut s.w, HERO, true);
    let w = &s.w;
    let n = &w.ruleset.numbers;
    assert_eq!(n.channel_cost, ruleset::CHANNEL_COST);
    for (id, cost) in [("t_jab", n.channel_cost[0]), ("t_heavy", n.channel_cost[1]), ("t_huge", n.channel_cost[2]), ("t_vast", n.channel_cost[2]), ("t_own", OWN_COST)] {
        let v = channel::version(w, HERO, skill(w, id)).unwrap_or_else(|| panic!("{id}: a version"));
        assert_eq!(v.cost, cost, "{id}");
        assert!(v.pierce, "{id}: an attack pierces");
    }
    assert_eq!(channel::version(w, HERO, skill(w, "t_jab")).unwrap().power, n.channel_power);
    assert_eq!(channel::version(w, HERO, skill(w, "t_own")).unwrap().power, 2.0);
    let spike = channel::version(w, HERO, skill(w, "t_spike")).expect("marrow-only");
    assert_eq!((spike.cost, spike.power, spike.pierce), (0, 1.0, false), "as authored");
    assert_eq!(n.channel_cost_for(0), 0, "0 AP pays none");
    let feed = channel::meters_for(w, HERO, skill(w, "t_feed"));
    assert_eq!(feed.iter().map(|d| d.delta).collect::<Vec<_>>(), [FEED_GAIN], "Feed pays its own meters only");
    // Cold Steel: no version, no cost
    let cold = session();
    assert!(channel::version(&cold.w, HERO, skill(&cold.w, "t_jab")).is_none());
    assert!(channel::meters_for(&cold.w, HERO, skill(&cold.w, "t_jab")).is_empty());
}

/// PK1 4.2 in resolve: a marrow Jab's command pays the cost on the channel meter at its start (marking it gained), hits
/// at power x `channel_power` and pierces; the Cold Steel Jab does none of it.
#[test]
fn a_marrow_version_resolves_with_its_cost_power_and_piercing() {
    let mut s = session();
    let cold = command(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_jab"), FOE), "t_jab");
    flag(&mut s.w, HERO, true);
    let marrow = command(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_jab"), FOE), "t_jab");
    let cost = s.w.ruleset.numbers.channel_cost[0];
    assert!(cold.meters.is_empty() && !cold.impact.armor_piercing);
    assert_eq!(marrow.meters.iter().map(|d| (d.meter.as_str(), d.delta)).collect::<Vec<_>>(), [("strain", cost)]);
    assert!(marrow.impact.armor_piercing && marrow.pierces(&s.w));
    let mult = |c: &Command| match c.kind {
        command::CmdKind::Attack { skill_mult, .. } => skill_mult,
        _ => panic!("an attack"),
    };
    assert!((mult(&marrow) - mult(&cold) * s.w.ruleset.numbers.channel_power).abs() < 1e-9);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, marrow, 1));
    run(&mut w, &mut res, 1, 1);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(cost), "paid at the start");
    assert!(w.meters.gauges.iter().any(|g| g.unit == HERO && g.gained));
}

/// PK1 4.3 item 2, 7: in a Channel round every row offers only marrow versions (Cold Steel versions greyed "Channel
/// round"); in a Cold Steel round the marrow-only skills are greyed and no marrow version is offered.
#[test]
fn rows_offer_only_marrow_versions_in_a_channel_round() {
    let mut s = session();
    let at = RoundTarget::Unit(FOE);
    assert_eq!(cell(&s, HERO, at, "t_spike"), Some(channel::ONLY_CHANNEL.to_string()));
    assert_eq!(cell(&s, HERO, at, "t_jab"), None);
    assert_eq!(cell(&s, HERO, at, "strike"), None);
    let _ = s.set_channel(HERO, true);
    assert_eq!(cell(&s, HERO, at, "t_jab"), None, "the marked active's version");
    assert_eq!(cell(&s, HERO, at, "t_spike"), None, "marrow-only");
    assert_eq!(cell(&s, HERO, at, "t_bash"), None, "the class basic's version");
    for id in ["t_heavy", "strike", "t_plain"] {
        assert_eq!(cell(&s, HERO, at, id), Some(channel::CHANNEL_ROUND.to_string()), "{id}");
    }
    assert_eq!(cell(&s, HERO, RoundTarget::Myself, "guard"), Some(channel::CHANNEL_ROUND.to_string()), "Stand row too");
}

/// PK1 4.3 item 1 (user 2026-11-10): the converting toggle. Picks Move here, Jab, Heavy, Rest: flipped to Channel, the
/// Move, Jab (now its marrow version) and Rest (now Feed) stay, Heavy (no version: not marked within the limit) is
/// dropped and named, and the fill refills the freed blocks with marrow versions and Feed, never a Cold Steel skill or
/// Rest. Flipped back, the Move, Jab and Rest stay and the Cold Steel fill and Rest return (no marrow-only skill, no
/// Feed); the dropped Heavy is not restored.
#[test]
fn the_toggle_converts_the_plan_keeping_moves() {
    let mut s = session();
    s.w.grid.positions[FOE] = Some(V2::new(6, 4));
    let jab = sid(&s.w, "t_jab");
    let heavy = sid(&s.w, "t_heavy");
    let here = V2::new(5, 4);
    let foe = RoundTarget::Unit(FOE);
    let (r, _) = s.pick(HERO, RoundTarget::Cell(here), &[Pick::MoveHere(here)]);
    assert_eq!(r, Ok(()));
    for p in [Pick::Skill { skill: jab, kind: None }, Pick::Skill { skill: heavy, kind: None }, Pick::Rest] {
        let (r, _) = s.pick(HERO, foe, &[p]);
        assert_eq!(r, Ok(()));
    }
    let before = planned(&s, HERO);
    assert_eq!(before[..4], ["move", "t_jab", "t_heavy", "rest"]);
    let (lines, _) = s.set_channel(HERO, true);
    assert_eq!(lines, Ok(vec!["p_hero: t_heavy dropped: Channel round".to_string()]));
    assert_eq!(rip::picks_of(&s.w, HERO), [Pick::MoveHere(here), Pick::Skill { skill: jab, kind: None }, Pick::Rest]);
    let after = planned(&s, HERO);
    assert_eq!(after[..3], ["move", "t_jab", "t_feed"], "the Move stays, Rest became Feed");
    assert_eq!(s.plan.compile(&s.w, HERO).blocks_left(), 0, "the freed blocks are refilled");
    for id in &after {
        assert!(channel::problem(&s.w, HERO, skill(&s.w, id)).is_none(), "{id}: only marrow versions and neutral steps");
        assert_ne!(id, "rest", "Feed is the Channel Rest");
    }
    let (lines, _) = s.set_channel(HERO, false);
    assert_eq!(lines, Ok(Vec::new()));
    let back = planned(&s, HERO);
    assert_eq!(back[..3], ["move", "t_jab", "rest"]);
    assert!(!back.iter().any(|id| id == "t_feed" || id == "t_spike" || id == "t_heavy"), "{back:?}");
}

/// PK1 4.3 items 4-5: a Channel round planned without picks fills in the Channel order (marrow-only first) and every
/// block it cannot fill is Feed; a Cold Steel round fills with Rest and never Feed.
#[test]
fn feed_is_the_fill_of_a_channel_round() {
    let mut s = session();
    s.w.units[HERO].ap = 1;
    let (r, _) = s.set_channel(HERO, true);
    assert!(r.is_ok());
    let steps = planned(&s, HERO);
    assert_eq!(steps[0], "t_spike", "the Channel order opens with the marrow-only skill");
    assert!(steps[1..].iter().all(|id| id == "t_feed"), "AP 1 spent: Feed on every block left: {steps:?}");
    assert_eq!(steps.len() as i32, BLOCKS);
    let (_, _) = s.set_channel(HERO, false);
    let cold = planned(&s, HERO);
    assert!(cold.iter().any(|id| id == "rest") && !cold.iter().any(|id| id == "t_feed"), "{cold:?}");
}

/// PK1 4.3 item 3 (TG1): the first marrow skill fixes the round's one target; a later pick on another unit is refused.
#[test]
fn the_first_skill_fixes_the_target_in_a_channel_round() {
    let mut s = session();
    s.w.grid.positions[FOE2] = Some(V2::new(3, 4));
    let _ = s.set_channel(HERO, true);
    let jab = sid(&s.w, "t_jab");
    assert_eq!(s.pick(HERO, RoundTarget::Unit(FOE), &[Pick::Skill { skill: jab, kind: None }]).0, Ok(()));
    let r = s.pick(HERO, RoundTarget::Unit(FOE2), &[Pick::Skill { skill: jab, kind: None }]).0;
    assert_eq!(r, Err("One target per round: p_foe".to_string()));
    assert_eq!(s.pick(HERO, RoundTarget::Unit(FOE), &[Pick::Skill { skill: sid(&s.w, "t_spike"), kind: None }]).0, Ok(()));
}

/// PK1 4.3 item 5: Vent starts only at Strain 50 or more (planned at its block; at its start a forced one fails unpaid)
/// and may take Strain below 50 (55 -> 35, then no second Vent); its HP cost is 15% of max HP through HC1 (refused at or
/// below the cost).
#[test]
fn vent_needs_fifty_strain_and_the_hp() {
    let mut s = session();
    flag(&mut s.w, HERO, true);
    let vent = sid(&s.w, "t_vent");
    let mut w = s.w.clone();
    // Vent is a Stand (self) skill
    round_intent::set_intent(&mut w, HERO, rpgkit_battle::round_intent::RoundIntent::Guard).expect("Stand");
    set_strain(&mut w, HERO, VENT_GATE - 5);
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, HERO, PlanStep::self_skill(vent), true), format!("Strain {} < {VENT_GATE}", VENT_GATE - 5));
    set_strain(&mut w, HERO, VENT_GATE + 5);
    assert_eq!(plan.add_step(&w, HERO, PlanStep::self_skill(vent), true), "");
    assert_eq!(plan.compile(&w, HERO).meter("strain"), Some(VENT_GATE + 5 - VENT_DROP));
    assert_eq!(plan.add_step(&w, HERO, PlanStep::self_skill(vent), true), format!("Strain {} < {VENT_GATE}", VENT_GATE + 5 - VENT_DROP), "no second Vent below 50");
    // the HP cost: 15% of max HP
    let cost = (w.units[HERO].max_hp * VENT_PCT / 100).max(1);
    assert_eq!(command::hp_cost_for(&w, HERO, skill(&w, "t_vent")), cost);
    let mut low = w.clone();
    low.units[HERO].hp = cost;
    assert_eq!(Plan::new().add_step(&low, HERO, PlanStep::self_skill(vent), true), format!("HP {cost} <= cost {cost}"));
    // resolved: 55 -> 35, the HP paid
    let cmd = command(&w, HERO, PlanStep::self_skill(vent), "t_vent");
    assert_eq!(cmd.hp_cost, cost);
    let hp = w.units[HERO].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, cmd.clone(), 1));
    let mut run_w = w.clone();
    run(&mut run_w, &mut res, 1, 1);
    assert_eq!(meter::value(&run_w, HERO, "strain"), Some(VENT_GATE + 5 - VENT_DROP));
    assert_eq!(run_w.units[HERO].hp, hp - cost);
    // forced below the gate at its start: it fails unpaid
    let mut gated = w.clone();
    set_strain(&mut gated, HERO, VENT_GATE - 1);
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, cmd, 1));
    run(&mut gated, &mut res, 1, 1);
    assert!(res.log.iter().any(|l| l == &format!("[Meter] p_hero's Vent: fails unpaid (Strain {} < {VENT_GATE})", VENT_GATE - 1)), "{:?}", res.log);
    assert_eq!(gated.units[HERO].hp, hp, "nothing paid");
}

/// PK1 4.3 items 5, 8: any Channel round skips the strain decay, every row of it: the `unless_gained` row with no gain
/// and the `if_guarded` row while the unit guarded; a Cold Steel round decays (-10, and -10 more after a guard).
#[test]
fn a_channel_round_skips_decay() {
    let mut s = session();
    set_strain(&mut s.w, HERO, 30);
    for guarded in [false, true] {
        let mut w = s.w.clone();
        if guarded {
            w.units[HERO].held_guard = Some(rpgkit_rules::unit::HeldGuard { mitigation: 0.3, last_block: BLOCKS });
        }
        let mut channel_w = w.clone();
        flag(&mut channel_w, HERO, true);
        let mut res = Resolver::new(BLOCKS, 1);
        res.apply_round_end_effects(&mut channel_w);
        assert_eq!(meter::value(&channel_w, HERO, "strain"), Some(30), "a Channel round (guarded {guarded})");
        let mut res = Resolver::new(BLOCKS, 1);
        res.apply_round_end_effects(&mut w);
        let decay = if guarded { 2 * DECAY } else { DECAY };
        assert_eq!(meter::value(&w, HERO, "strain"), Some(30 - decay), "a Cold Steel round (guarded {guarded})");
    }
}

/// PK1 4.3 item 4 (review H1): the Channel order leaves the meter rows' skills (Feed, Vent) and the Channel Rest out:
/// a Guard Channel round with a marked DEFEND active plans the active first, then Feed fills what is left.
#[test]
fn a_marked_active_comes_before_feed() {
    let mut s = session();
    s.w.units[HERO].deck.marrow = vec!["t_brace".to_string()];
    s.w.units[HERO].ap = 1;
    round_intent::set_intent(&mut s.w, HERO, rpgkit_battle::round_intent::RoundIntent::Guard).expect("Stand");
    let (r, _) = s.set_channel(HERO, true);
    assert!(r.is_ok());
    let steps = planned(&s, HERO);
    assert_eq!(steps[0], "t_brace", "{steps:?}");
    assert!(steps[1..].iter().all(|id| id == "t_feed"), "{steps:?}");
    assert_eq!(steps.len() as i32, BLOCKS);
    let ids: Vec<String> = rip::order(&s.w, HERO, rpgkit_battle::round_intent::RoundIntent::Guard).iter().map(|&k| s.w.cat.skills[k].id.clone()).collect();
    assert_eq!(ids, ["t_brace"], "no Channel basic in the order");
}

/// Review M3: the battle reads a marrow version by IR21 as the checks do: without `field_intent` the ARC Cleave plans in
/// one legacy row, yet it is dual intent and has no version.
#[test]
fn a_dual_intent_skill_has_no_version_without_field_intent() {
    let mut list = rules(true, json!({}));
    list.rules.retain(|r| *r != FIELD_INTENT);
    let s = session_with(list, true);
    let cleave = skill(&s.w, "t_cleave");
    assert_eq!(round_intent::skill_rows(&s.w, cleave).len(), 1, "one legacy row");
    assert!(!channel::has_version(&s.w, cleave));
    assert!(channel::has_version(&s.w, skill(&s.w, "t_jab")));
}

/// PK1 4.3 item 1 (review L7): flipping back to Cold Steel drops a marrow-only pick; when the toggle drops the opener,
/// the round keeps its target and the next skill pick opens it.
#[test]
fn flipping_back_drops_marrow_only_picks_and_the_target_stays() {
    let mut s = session();
    let (jab, heavy, spike) = (sid(&s.w, "t_jab"), sid(&s.w, "t_heavy"), sid(&s.w, "t_spike"));
    let _ = s.set_channel(HERO, true);
    for k in [jab, spike] {
        assert_eq!(s.pick(HERO, RoundTarget::Unit(FOE), &[Pick::Skill { skill: k, kind: None }]).0, Ok(()));
    }
    let (lines, _) = s.set_channel(HERO, false);
    assert_eq!(lines, Ok(vec!["p_hero: t_spike dropped: Only in a Channel round".to_string()]));
    assert_eq!(rip::picks_of(&s.w, HERO), [Pick::Skill { skill: jab, kind: None }]);

    let mut s = session();
    s.w.grid.positions[FOE2] = Some(V2::new(3, 4));
    for k in [heavy, jab] {
        assert_eq!(s.pick(HERO, RoundTarget::Unit(FOE), &[Pick::Skill { skill: k, kind: None }]).0, Ok(()));
    }
    let (lines, _) = s.set_channel(HERO, true);
    assert_eq!(lines, Ok(vec!["p_hero: t_heavy dropped: Channel round".to_string()]));
    let rs = s.w.round_states.get(&HERO).expect("a round state");
    assert_eq!(rs.target, Some(RoundTarget::Unit(FOE)), "the round keeps its target");
    assert_eq!(rs.opener, Some(rpgkit_battle::round_intent::Opener::Skill(jab)), "the next pick opens it");
    assert_eq!(s.pick(HERO, RoundTarget::Unit(FOE2), &[Pick::Skill { skill: jab, kind: None }]).0, Err("One target per round: p_foe".to_string()));
}

/// Inert without holders: with the rule on and no unit holding the channel meter, a whole battle plays exactly as with
/// the rule off (the same plans, log and HP).
#[test]
fn the_rule_is_inert_without_a_holder() {
    let fight = |on: bool| {
        let mut s = session_with(rules(on, json!({})), false);
        let mut log = Vec::new();
        let mut plans = Vec::new();
        for _ in 0..4 {
            if !s.plan_confirmed {
                s.auto_draft("balanced");
                plans.push((0..s.w.units.len()).map(|c| planned(&s, c)).collect::<Vec<_>>());
                assert!(s.confirm_plan());
            }
            s.resolve_round();
            log.extend(s.log_lines(0, 0));
            if !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
        let hp: Vec<i32> = s.w.units.iter().map(|u| u.hp).collect();
        // the same draws (no roll of its own) and the same trace
        let rng = format!("{:?}", s.resolver.rng);
        let trace = rpgkit_battle::sim::metrics_trace(&s).to_string();
        (plans, log, hp, rng, trace)
    };
    let (on, off) = (fight(true), fight(false));
    assert!(!on.1.is_empty());
    assert_eq!(on, off);
    let s = session_with(rules(true, json!({})), false);
    assert!((0..s.w.units.len()).all(|c| !channel::can_channel(&s.w, c)));
}
