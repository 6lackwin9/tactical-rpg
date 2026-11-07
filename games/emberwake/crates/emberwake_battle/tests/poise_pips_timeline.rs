//! S2.2-S3 (`docs/sessions/S2_POISE.md` 4 item 3, 5; FB-71): the shipped `timeline` ruleset row plays poise by pips
//! (seed `s22s3`); the tests below build the row with `poise_mode: pips` in memory from the shipped one, so they read the
//! pip rule whatever the row says. Section 5's acceptance tests 1 and 2 on a battle under that row (every timeline rule
//! on): chip hits under the threshold remove no pip and a heavy hit removes `floor(D / T)`; a guard halves the stagger
//! before the threshold. Test 3 (S2.2-S4) on fights: a break queues `stagger` and the next round refills the pips. A whole fight of each catalog encounter keeps the value the mirror of the pips (`value = pips x T`) and
//! never takes a pip for a hit under the threshold. Expectations come from the rules, never from shipped numbers: the
//! tank's Max Poise is set here (60: 5 pips of T 12 at the default `poise_pips`), its stagger passives and statuses are
//! cleared in memory, and the hits are planned through the kit's `poise::forecast_hits`.

use emberwake_battle::poise::{self, PlannedHit, PoiseState, Verdict};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_battle::world::{UnitId, World};
use emberwake_rules::ruleset::{PoiseMode, Ruleset, TIMELINE};
use emberwake_world::Data;
use indexmap::IndexMap;
use serde_json::json;
use std::path::Path;

/// S2 section 5: the tank's Max Poise.
const TANK: i32 = 60;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("shipped data").0
}

/// The shipped timeline row with the seed's one change (`poise_mode: pips`).
fn pips_timeline(d: &Data) -> Ruleset {
    let mut row = d.cat.rulesets.get(TIMELINE).expect("the shipped timeline row").clone();
    row.insert("poise_mode".into(), json!("pips"));
    Ruleset::from_row(&row)
}

/// The first catalog encounter with a party and enemies, started under `rules`.
fn battle(d: &Data, rules: &Ruleset, id: &str, seed: i64) -> Session {
    let entry = d.cat.encounters.get(id).expect("encounter").clone();
    let mut def = EncounterDef::from_obj(&entry);
    def.rng_seed = seed;
    def.ruleset = rules.clone();
    Session::start(d.cat.clone(), def, &IndexMap::new())
}

fn fightable(d: &Data) -> Vec<String> {
    let has = |e: &serde_json::Map<String, serde_json::Value>, k: &str| e.get(k).and_then(|v| v.as_array()).is_some_and(|a| !a.is_empty());
    d.cat.encounters.iter().filter(|(_, e)| has(e, "party") && has(e, "enemies")).map(|(id, _)| id.clone()).collect()
}

/// A tank on the first enemy of `s`: Max Poise `TANK`, full pips, no stagger passives or statuses (in memory only).
fn tank(s: &mut Session) -> UnitId {
    let foe = s.w.enemies[0];
    s.w.units[foe].stagger_taken.clear();
    s.w.units[foe].st.active.clear();
    let n = s.w.ruleset.numbers.clone();
    let mut st = PoiseState::default();
    poise::refill(&n, &mut st, TANK);
    s.w.poise.insert(foe, st);
    foe
}

/// `n` planned hits of stagger `stagger` on `t` from block 1 (no windup, no flank, nothing scheduled).
fn planned(w: &World, t: UnitId, stagger: i32, n: i32) -> Vec<PlannedHit> {
    assert!(w.ruleset.has(emberwake_rules::ruleset::POISE), "the timeline row plays poise");
    let impact = poise::Impact { stagger, ..poise::Impact::default() };
    (1..=n).map(|b| PlannedHit { target: t, block: b, impact, blocks: 1, flank: false, ignore_guard: false }).collect()
}

/// The seed's row: pips with the default pip numbers; every other number is the shipped row's.
#[test]
fn the_seeded_timeline_row_reads_pips() {
    let d = data();
    let shipped = Ruleset::of(&d.cat, TIMELINE).expect("timeline");
    let pips = pips_timeline(&d);
    assert_eq!(pips.numbers.poise_mode, PoiseMode::Pips);
    assert!(poise::pips_mode(&pips.numbers));
    let mut want = shipped.numbers.clone();
    want.poise_mode = PoiseMode::Pips;
    assert_eq!(pips.numbers, want, "the seed changes poise_mode only");
    let notes = pips.notes();
    assert!(notes.errors.iter().chain(&notes.warnings).all(|n| !n.contains("poise") && !n.contains("pip")), "the register has nothing to say on poise: {notes:?}");
}

/// S2.2-S3: the shipped timeline row itself plays pips (seed `s22s3`).
#[test]
fn the_shipped_timeline_row_reads_pips() {
    let d = data();
    assert_eq!(Ruleset::of(&d.cat, TIMELINE).expect("timeline").numbers.poise_mode, PoiseMode::Pips);
}

/// S2 section 5 test 1 on a timeline battle: five D = 10 hits on the tank (T 12) remove no pip and the value stays the
/// mirror; one D = 26 hit removes 2 pips.
#[test]
fn chip_hits_leave_the_tank_s_pips_on_a_timeline_battle() {
    let d = data();
    let rules = pips_timeline(&d);
    let id = fightable(&d).into_iter().next().expect("a fightable encounter");
    let mut s = battle(&d, &rules, &id, 1);
    let foe = tank(&mut s);
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    assert_eq!(t, (TANK as f64 / n.poise_pips as f64).round() as i32, "T = roundi(max / poise_pips)");
    let fcs = poise::forecast_hits(&s.w, &[], &planned(&s.w, foe, 10, 5));
    for (k, f) in fcs.iter().enumerate() {
        let f = f.expect("the tank has a bar");
        assert_eq!(f.taken, 10, "hit {k}: no factor applies");
        assert!(f.taken < t, "a chip under T");
        assert_eq!((f.pips_before, f.pips_after, f.after, f.verdict), (n.poise_pips, n.poise_pips, n.poise_pips * t, Verdict::Holds), "hit {k}");
    }
    let f = poise::forecast_hits(&s.w, &[], &planned(&s.w, foe, 26, 1))[0].unwrap();
    assert_eq!((f.pips_before - f.pips_after, f.after), (26 / t, (n.poise_pips - 26 / t) * t), "floor(26 / T) pips");
}

/// S2 section 5 test 2 on a timeline battle: a held guard (Guard Option 1C) applies `stagger_guard_mult` before the
/// threshold: a stagger 20 hit takes 10, no pip against T 12; unguarded it removes 1.
#[test]
fn a_guard_halves_the_stagger_before_the_threshold_on_a_timeline_battle() {
    let d = data();
    let rules = pips_timeline(&d);
    let id = fightable(&d).into_iter().next().expect("a fightable encounter");
    let mut s = battle(&d, &rules, &id, 1);
    let foe = tank(&mut s);
    let n = s.w.ruleset.numbers.clone();
    let t = poise::threshold(&n, TANK);
    let open = poise::forecast_hits(&s.w, &[], &planned(&s.w, foe, 20, 1))[0].unwrap();
    assert_eq!((open.taken, open.pips_before - open.pips_after), (20, 20 / t), "unguarded");
    emberwake_battle::trigger::hold_defend(&mut s.w, foe, 0.5, 1);
    let held = poise::forecast_hits(&s.w, &[], &planned(&s.w, foe, 20, 1))[0].unwrap();
    assert_eq!(held.factors.guard, n.stagger_guard_mult);
    let taken = (20.0 * n.stagger_guard_mult).round() as i32;
    assert_eq!((held.taken, held.pips_before - held.pips_after), (taken, taken / t), "guarded");
    assert_eq!(held.pips_after, held.pips_before, "10 < 12: no pip");
}

/// Every catalog encounter fought under the seeded row: after every step each bar's value is its pips x T, and no
/// landed hit under the threshold took a pip (unless it broke by `sure_break`).
#[test]
fn a_timeline_fight_keeps_the_value_the_mirror_of_the_pips() {
    let d = data();
    let rules = pips_timeline(&d);
    let mut hits = 0;
    for id in fightable(&d) {
        let mut s = battle(&d, &rules, &id, 7);
        for round in 0..s.def.max_rounds {
            s.auto_draft("balanced");
            if !s.confirm_plan() {
                break;
            }
            for _ in 0..600 {
                if s.phase == Phase::Replay {
                    break;
                }
                s.step_next();
                for (c, st) in &s.w.poise {
                    if st.max > 0 {
                        assert_eq!(st.value, st.pips * poise::threshold(&s.w.ruleset.numbers, st.max), "{id} round {round}: unit {c} {st:?}");
                    }
                }
            }
            for a in &s.resolver.scheduled {
                for h in &a.command.staggers {
                    hits += 1;
                    let t = poise::threshold(&s.w.ruleset.numbers, s.w.poise[&h.target].max);
                    if h.taken < t && h.broke.is_none() {
                        assert_eq!(h.pips_after, h.pips_before, "{id}: a chip under T takes no pip: {h:?}");
                    }
                }
            }
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
    }
    assert!(hits > 0, "the fights landed stagger hits");
}

/// S2.2-S4 (S2 section 5 test 3) on timeline fights: whenever a unit breaks (0 pips) it has no wind-up left and has
/// `stagger` queued unless it holds `steadfast`; at the next round its pips are full again (not broken, broken last
/// round), `stagger` is active and the planner locks the stagger row's blocks.
#[test]
fn a_break_queues_stagger_on_a_timeline_fight() {
    let d = data();
    let rules = pips_timeline(&d);
    let row = emberwake_rules::status::row_of(poise::BREAK_STATUS).expect("the stagger row");
    let mut checked = 0;
    for id in fightable(&d) {
        for seed in [1337, 7, 42] {
            let mut s = battle(&d, &rules, &id, seed);
            for round in 0..s.def.max_rounds {
                s.auto_draft("balanced");
                if !s.confirm_plan() {
                    break;
                }
                let mut queued: Vec<UnitId> = Vec::new();
                for _ in 0..600 {
                    if s.phase == Phase::Replay {
                        break;
                    }
                    let before: Vec<UnitId> = s.w.poise.iter().filter(|(_, st)| st.broken).map(|(c, _)| *c).collect();
                    s.step_next();
                    for (&c, st) in s.w.poise.iter().filter(|(c, st)| st.broken && !before.contains(c)) {
                        let u = &s.w.units[c];
                        assert_eq!(st.pips, 0, "{id} seed {seed} round {round}: a break is 0 pips");
                        assert!(!s.resolver.scheduled.iter().any(|a| a.unit == c && poise::action_in_windup(a, s.current_block)), "{id}: 1. no wind-up left");
                        let steadfast = u.has_status(poise::STEADFAST);
                        assert_eq!(u.st.pending.contains(poise::BREAK_STATUS), !steadfast, "{id} seed {seed} round {round}: 2. stagger queued unless steadfast");
                        if !steadfast && u.is_alive() {
                            queued.push(c);
                        }
                    }
                }
                if s.combat_over || !s.can_advance_round() {
                    break;
                }
                s.advance_round();
                for c in queued.into_iter().filter(|&c| s.w.units[c].is_active()) {
                    let st = s.w.poise[&c];
                    assert_eq!((st.pips, st.broken, st.broken_last), (s.w.ruleset.numbers.poise_pips, false, true), "{id}: 3. refilled at the roll-over");
                    assert!(s.w.units[c].has_status(poise::BREAK_STATUS), "{id}: staggered this round");
                    let locked = s.plan.compile(&s.w, c).blocked_blocks;
                    for b in row.lock_blocks.iter().filter(|&&b| b <= s.w.total_blocks) {
                        assert!(locked.contains_key(b), "{id}: block {b} locked");
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0, "some timeline fight broke a unit");
}
