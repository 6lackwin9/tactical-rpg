//! Battle metrics and the battle report (BattleMetrics, BattleReport; BREPORT1-L rules 4-7): the report's columns,
//! tips, best moment and validation, and the collector on real world battles (no self-check fails, live and sim
//! modes agree, following a fight never changes it). The Godot comparison of every recorded fight is in the parity
//! tests (trace_parity, world_traces).

use emberwake_battle::metrics::{BattleMetrics, BestFinish, BestHit, Detail, EndResult, Save};
use emberwake_battle::report::{self, Report, Row};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_rules::status;
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::path::Path;

fn row(name: &str) -> Row {
    Row {
        key: name.into(),
        name: name.into(),
        guest: false,
        state: "standing",
        dealt: 0,
        dealt_dot: 0,
        ff_dealt: 0,
        taken: 0,
        taken_dot: 0,
        healed: 0,
        regen: 0,
        effects: 0,
        combos: 0,
        kills: 0,
        status_counts: IndexMap::new(),
        combo_counts: IndexMap::new(),
    }
}

fn hit(amount: i32, skill: &str, target: &str, order: i32) -> Detail {
    Detail { best_hit: Some(BestHit { amount, skill: skill.into(), target: target.into(), round: 1, order }), ..Detail::default() }
}

#[test]
fn columns_and_tips() {
    let no_healer = [Row { dealt: 30, taken: 5, ..row("A") }, Row { effects: 2, ..row("B") }];
    let cols = report::columns_for(&no_healer);
    assert_eq!(cols, ["name", "dealt", "taken", "effects"], "no Healed, no all-zero Combos; Effects in table order");
    assert_eq!(report::columns_for(&[row("Z")]), report::ALWAYS, "Damage and Taken always, even all zero");
    assert_eq!(report::columns_for(&[Row { healed: 4, combos: 1, effects: 1, ..row("H") }]), report::COLUMNS, "all six in table order");
    let mut t = Row { dealt: 50, dealt_dot: 8, ff_dealt: 3, taken: 20, taken_dot: 6, ..row("T") };
    t.status_counts.insert("bleed".into(), 2);
    t.combo_counts = [("exploit", 1), ("break", 0), ("flow", 2)].iter().map(|(k, n)| (k.to_string(), *n)).collect();
    assert_eq!(report::dealt_tip(&t), "42 from hits, 8 from bleed, poison and burn. Hit allies for 3.");
    assert_eq!(report::taken_tip(&t), "14 from hits, 6 from effects.");
    assert_eq!(report::effects_tip(&t), format!("{} x2", status::display_name("bleed")));
    assert_eq!(report::combos_tip(&t), "Exploit x1, Flow x2", "used kinds only");
    assert_eq!(report::healed_tip(&t), "", "no regen: no Healed tip");
    t.status_counts.insert("haste".into(), 1);
    assert!(report::effects_tip(&t).ends_with(&format!("{} on allies x1", status::display_name("haste"))), "a buff is marked on allies");
}

#[test]
fn best_moment_prefers_a_save_then_a_combo_finish_then_the_biggest_hit() {
    let rows = [row("Healer"), row("Striker")];
    let big = hit(90, "Smash", "Ogre", 5);
    let save = Detail { best_save: Some(Save { ally: "Striker".into(), hp: 7, round: 2, order: 30 }), ..Detail::default() };
    assert_eq!(report::best_moment(&rows, &[save, big.clone()]), "Healer healed Striker at 7 HP, and Striker made it through.");
    let finish = BestFinish { skill: "Opportune Cut".into(), target: "Goblin".into(), combo: "exploit".into(), status: "bleed".into(), round: 2, order: 40 };
    let with_finish = Detail { best_finish: Some(finish), ..hit(10, "Cut", "Goblin", 40) };
    assert_eq!(report::best_moment(&rows, &[big.clone(), with_finish]), format!("Striker finished Goblin with Opportune Cut (exploited {}).", status::display_name("bleed")));
    let brk = Detail { best_finish: Some(BestFinish { skill: "Crush".into(), target: "Goblin".into(), combo: "break".into(), status: String::new(), round: 1, order: 3 }), ..Detail::default() };
    assert_eq!(report::best_moment(&[row("A")], &[brk]), "A finished Goblin with Crush (broke the guard).");
    let (early, late) = (hit(40, "Jab", "Wolf", 2), hit(40, "Hook", "Wolf", 9));
    assert_eq!(report::best_moment(&rows, &[late, early.clone()]), "Striker's Jab hit Wolf for 40.", "a tie picks the earlier entry");
    assert_eq!(report::best_moment(&rows, &[big, early]), "Healer's Smash hit Ogre for 90.", "the bigger hit wins");
    let save_at = |hp, order| Detail { best_save: Some(Save { ally: "B".into(), hp, round: 1, order }), ..Detail::default() };
    assert!(report::best_moment(&rows, &[save_at(9, 1), save_at(3, 50)]).starts_with("Striker healed B at 3 HP"), "the closer call wins");
    assert_eq!(report::best_moment(&rows, &[Detail::default(), Detail::default()]), "", "no hits: no line");
}

fn valid_report() -> Value {
    let r = Report { outcome: "won".into(), rounds: 2, rows: vec![Row { dealt: 10, taken: 3, ..row("A") }], columns: vec!["name", "dealt", "taken"], best: String::new() };
    r.to_json()
}

#[test]
fn report_validation() {
    assert!(report::problems(&valid_report()).is_empty(), "a built report is valid");
    assert!(!report::problems(&json!({})).is_empty());
    let mut neg = valid_report();
    neg["rows"][0]["dealt"] = json!(-1);
    assert!(!report::problems(&neg).is_empty(), "a negative number");
    let mut col = valid_report();
    col["columns"] = json!(["name", "dealt", "taken", "style"]);
    assert!(!report::problems(&col).is_empty(), "an unknown column");
    let mut typ = valid_report();
    typ["best"] = json!(3);
    assert!(!report::problems(&typ).is_empty(), "a wrong type");
    let mut missing = valid_report();
    missing["rows"][0].as_object_mut().unwrap().shift_remove("combos");
    assert!(!report::problems(&missing).is_empty(), "a missing row key");
    let mut state = valid_report();
    state["rows"][0]["state"] = json!("asleep");
    assert!(!report::problems(&state).is_empty(), "an unknown state");
}

/// Every ashlands encounter, played out with the balanced preset, a collector following it or not.
fn play_all(follow: Option<bool>) -> Vec<(Session, Value)> {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    let mut out = Vec::new();
    for e in d.own_entries("ashlands", "encounters") {
        let id = aegis_core::json::s(e, "id");
        let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", &id));
        let mut s = Session::start(d.cat.clone(), def, &WorldRoster { d: &d, world: "ashlands".into() });
        if let Some(live) = follow {
            s.attach_metrics(BattleMetrics::new(), live);
        }
        let mut ends = Vec::new();
        for _ in 0..s.def.max_rounds {
            s.plan_apply_preset_all("balanced");
            assert!(s.confirm_plan());
            for _ in 0..1000 {
                if s.phase == Phase::Replay {
                    break;
                }
                s.step_next();
                if follow == Some(false) {
                    s.metrics_step();
                }
            }
            ends.push(json!(s.snapshot().iter().map(|u| (u.hp, u.ap, u.pos.map(|p| (p.x, p.y)), u.statuses.clone())).collect::<Vec<_>>()));
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
            if follow == Some(false) {
                s.metrics_round();
            }
        }
        if follow.is_some() {
            s.metrics_end(EndResult::default());
        }
        out.push((s, Value::Array(ends)));
    }
    out
}

/// A scripted round: the hero breaks one foe's stance, then finishes a bleeding foe with an exploit and flows into
/// Defensive; its own regen ticks at the round end. The traces hold no combo and no credited regen, so these rules
/// are checked here (BREPORT1 1.2: combo counts, the best finish, regen_done).
#[test]
fn combos_and_regen_are_credited() {
    use emberwake_battle::command::{CmdKind, Command};
    use emberwake_battle::grid::V2;
    use emberwake_rules::Stance;
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    let pick = d.own_entries("ashlands", "encounters").iter().find_map(|e| {
        let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", &aegis_core::json::s(e, "id")));
        let s = Session::start(d.cat.clone(), def, &WorldRoster { d: &d, world: "ashlands".into() });
        (s.w.enemies.iter().filter(|&&e| s.w.has(e)).count() >= 2).then_some(s)
    });
    let mut s = pick.expect("an ashlands encounter with two enemies on the field");
    let h = s.w.allies[0];
    let foes: Vec<usize> = s.w.enemies.iter().copied().filter(|&e| s.w.has(e)).take(2).collect();
    let (f1, f2) = (foes[0], foes[1]);
    // Clear the field around a free tile: the hero there, the foes on either side.
    let free = |w: &emberwake_battle::world::World, c: V2| w.grid.is_valid(c) && w.grid.is_walkable(c) && !w.grid.is_exit(c) && w.occupant(c).is_none();
    let spot = (0..16).flat_map(|x| (0..12).map(move |y| V2::new(x, y))).find(|&c| free(&s.w, c) && free(&s.w, c + V2::new(1, 0)) && free(&s.w, c + V2::new(-1, 0))).expect("three free tiles in a row");
    assert!(s.w.jump_unit(h, spot) && s.w.jump_unit(f1, spot + V2::new(1, 0)) && s.w.jump_unit(f2, spot + V2::new(-1, 0)));
    let hero = s.w.units[h].name.clone();
    s.w.units[h].hp = s.w.units[h].max_hp / 2;
    emberwake_rules::status::grant_now(&mut s.w.units[h], "regen", 3, 10, &hero);
    emberwake_rules::status::grant_now(&mut s.w.units[f1], "bleed", 3, 0, "");
    s.w.units[f1].hp = 1;
    s.w.units[f2].stance = Stance::Offensive;
    s.attach_metrics(BattleMetrics::new(), false);
    assert!(s.confirm_plan());
    s.resolver.scheduled.clear();
    let attack = |target: usize, mult: f64| CmdKind::Attack {
        target: Some(target),
        skill_mult: mult,
        use_offhand: false,
        follow_tiles: 0,
        dash_tiles: 0,
        status_rounds: 0,
        min_range: 1,
        max_range: 1,
        last_damage: 0,
        last_crit: false,
        last_target: None,
    };
    let mut breaker = Command::new(h, 0, 1, attack(f2, 0.1));
    breaker.name = "Crush".into();
    breaker.disrupt_break = true;
    let mut finisher = Command::new(h, 0, 1, attack(f1, 2.0));
    finisher.name = "Opportune Cut".into();
    finisher.exploit = vec![json!({"status": "bleed", "power_mult": 1.5})];
    finisher.stance_on_end = Some(Stance::Defensive);
    assert!(s.resolver.schedule(h, breaker, 1) && s.resolver.schedule(h, finisher, 2));
    for _ in 0..200 {
        if s.phase == Phase::Replay {
            break;
        }
        s.step_next();
        s.metrics_step();
    }
    assert!(!s.w.units[f1].is_alive() && s.w.units[f2].is_alive(), "the finisher killed, the breaker did not");
    s.metrics_end(EndResult::default());
    let m = s.metrics.as_deref().unwrap();
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    let (rec, det) = (&m.units[&hero], &m.detail[&hero]);
    assert_eq!(rec.combos, 3, "a break, an exploit and a flow");
    assert_eq!(det.combo_counts.values().copied().collect::<Vec<_>>(), [1, 1, 1], "one of each kind");
    let f1_name = s.w.units[f1].name.clone();
    let finish = det.best_finish.as_ref().expect("the killing exploit is the best finish");
    assert_eq!((finish.combo.as_str(), finish.target.as_str(), finish.skill.as_str(), finish.status.as_str()), ("exploit", f1_name.as_str(), "Opportune Cut", "bleed"));
    assert_eq!(m.units[&f1_name].combos, 0, "the foe's record is untouched");
    assert!(det.regen_done > 0 && rec.heal_done >= det.regen_done && rec.regen_received == det.regen_done, "the regen tick is the hero's heal");
    let r = Report::build(m, &s, &s.outcome);
    assert_eq!(r.best, format!("{hero} finished {f1_name} with Opportune Cut (exploited {}).", status::display_name("bleed")));
    assert_eq!(report::combos_tip(&r.rows[0]), "Exploit x1, Stance break x1, Flow x1");
    assert_eq!(report::healed_tip(&r.rows[0]), format!("Includes {} from regeneration.", det.regen_done));
}

#[test]
fn following_a_fight_never_changes_it_and_live_equals_sim() {
    let plain = play_all(None);
    let live = play_all(Some(true));
    let sim = play_all(Some(false));
    assert!(!plain.is_empty());
    for ((p, l), s) in plain.iter().zip(&live).zip(&sim) {
        assert_eq!(p.1, l.1, "{}: every round ends the same with a live collector", p.0.def.id);
        assert_eq!(p.1, s.1, "{}: and with a sim collector", p.0.def.id);
        let (lm, sm) = (l.0.metrics.as_deref().unwrap(), s.0.metrics.as_deref().unwrap());
        assert!(lm.frozen, "the live collector is frozen after the end");
        assert_eq!(lm.units, sm.units, "{}: live and sim records agree", p.0.def.id);
        assert_eq!(lm.detail, sm.detail, "{}: and their details", p.0.def.id);
        for m in [lm, sm] {
            assert!(m.fail_lines.is_empty(), "{}: the HP ledger balances: {:?}", p.0.def.id, m.fail_lines);
            assert!(m.checks > 0);
            for (name, rec) in &m.units {
                let counts: i32 = m.detail[name].combo_counts.values().sum();
                assert_eq!(rec.combos, counts, "{name}: combos = the sum of its combo counts");
            }
        }
        let built = Report::build(lm, &l.0, &l.0.outcome);
        assert!(report::problems(&built.to_json()).is_empty(), "{}: the report is valid", p.0.def.id);
        assert_eq!(built.rows.len(), l.0.w.allies.len(), "one row per party unit");
        for (r, &c) in built.rows.iter().zip(&l.0.w.allies) {
            assert_eq!(r.name, l.0.w.units[c].name, "rows in roster order");
            assert_eq!(r.key, l.0.hero_key(c));
        }
    }
}
