//! PR5-32 (`LAYER-UP-EVENTS`, audit 2026-11-10 2.2 item 2): the battle screen's sounds, impact modifiers, breaks and
//! refusal words come from the kit's typed events (`event::EventKind`) and typed refusals (`refusal::Reason`), never
//! from the log's or the planner's words. The proof that nothing changed: every catalog encounter, under its own
//! ruleset and the timeline row, played step by step by the auto driver, cues the same sounds, modifiers and breaks
//! from the typed events as the screen's former prose reading (kept here as the reference, `prose`) did from the same
//! steps' lines; the skill menus and the classic probe word every refusal as the former text parsing did. With the
//! units renamed to the very words the prose reading looked for and the channeled mark renamed (custom `terms`), the
//! typed cues stay those of the same fight, while the prose reading would misfire. No shipped number is assumed.

use crate::battle::flow::sound_of;
use crate::battle::tips::{channel_note, channeled_word};
use crate::fx::{self, Mods};
use crate::intent_text;
use emberwake_battle::event::{self, EventKind};
use emberwake_battle::refusal;
use emberwake_battle::round_intent::{self as ri, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_host::{Battle, Host};
use emberwake_world::Data;
use serde_json::json;
use std::sync::Arc;

/// The battle screen's reading of the log's and the planner's words before PR5-32 (verbatim), the reference the typed
/// reading is compared with.
mod prose {
    use emberwake_battle::world::World;

    pub fn sound_of(line: &str) -> Option<&'static str> {
        let tag = line.split_once(']').map(|(t, _)| t.trim_start_matches('['))?;
        match tag {
            "Attack Miss" | "Area Miss" => Some("miss"),
            "Attack" if line.contains(" CRIT") => Some("crit"),
            "Attack" if line.contains(" for 0 ") => Some("mitigated"),
            "Attack" => Some("hit"),
            "Status" => {
                let id = line.split_once("(inflicts ")?.1.split(')').next()?;
                (!emberwake_rules::status::row_of(id).is_some_and(|r| r.buff)).then_some("debuff")
            }
            "Defend" if !line.contains("guard was broken") => Some("defend"),
            "Focus" => Some("buff"),
            "Support" => Some("heal"),
            "Move" | "Move Cut Short" | "Withdrawal" | "Dash" | "Chase" | "Disengage" => Some("move_step"),
            _ => None,
        }
    }

    pub fn mods(lines: &[String], hp_fell: bool) -> crate::fx::Mods {
        let attack = |l: &String| l.starts_with("[Attack]");
        let miss = lines.iter().any(|l| l.starts_with("[Attack Miss]") || l.starts_with("[Area Miss]"));
        let crit = lines.iter().any(|l| attack(l) && l.contains(" CRIT"));
        crate::fx::Mods { crit, zero: !miss && !hp_fell && lines.iter().any(attack), miss }
    }

    pub fn broke(lines: &[String]) -> bool {
        lines.iter().any(|l| l.starts_with("[Break]"))
    }

    pub fn refusal_text(reason: &str) -> String {
        let parsed = reason.strip_prefix("HP ").and_then(|r| r.split_once(" <= cost ")).and_then(|(hp, cost)| Some((hp.parse::<i32>().ok()?, cost.parse::<i32>().ok()?)));
        match parsed {
            Some((hp, cost)) if emberwake_battle::command::hp_cost_problem(hp, cost) == reason => format!("Not enough HP: {hp}, it costs {cost}"),
            _ => reason.to_string(),
        }
    }

    pub fn menu_refusal_text(w: &World, reason: &str) -> String {
        let join = emberwake_battle::round_intent_plan::REASONS_JOIN;
        if reason.contains(join) {
            return reason.split(join).map(|r| one_refusal_text(w, r)).collect::<Vec<_>>().join(join);
        }
        one_refusal_text(w, reason)
    }

    fn one_refusal_text(w: &World, reason: &str) -> String {
        let hp = refusal_text(reason);
        if hp != reason {
            return hp;
        }
        let gate = reason.rsplit_once(" < ").and_then(|(left, min)| {
            let (name, have) = left.rsplit_once(' ')?;
            Some((name, have.parse::<i32>().ok()?, min.parse::<i32>().ok()?))
        });
        if let Some((name, have, min)) = gate {
            if let Some(def) = w.cat.unit_meters.values().find(|m| m.name == name) {
                let g = emberwake_data::MeterGate { meter: def.id.clone(), min };
                if emberwake_battle::meter::gate_problem(w, Some(&g), |_| Some(have)) == reason {
                    return format!("Needs {name} {min} or more to start: {have}");
                }
            }
        }
        reason.to_string()
    }
}

/// One step of a fight: its new log lines and their typed events.
struct Step {
    lines: Vec<String>,
    events: Vec<EventKind>,
}

fn data() -> Arc<Data> {
    let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Arc::new(Data::open(&dir).expect("data").0)
}

/// A battle of `encounter` under `ruleset` ("" = its own) through the command host; None when it does not start.
fn battle(data: &Arc<Data>, encounter: &str, ruleset: &str, seed: i64) -> Option<Host> {
    let mut host = Host::new(Battle::with_data(data.clone()));
    let r = host.handle(&json!({"cmd": "new", "encounter": encounter, "seed": seed, "ruleset": ruleset}));
    (r["ok"] == json!(true)).then_some(host)
}

/// Plays the host's battle step by step (the party on the auto driver's aggressive draft) to its end; `planning` sees
/// the host at each round's planning before the draft.
fn steps(host: &mut Host, mut planning: impl FnMut(&mut Host)) -> Vec<Step> {
    let mut out = Vec::new();
    for _ in 0..4000 {
        {
            let s = host.session.as_mut().unwrap();
            if s.combat_over || s.needs_stay_pick() {
                break;
            }
            if s.phase == Phase::Replay {
                if !s.can_advance_round() {
                    break;
                }
                s.advance_round();
            }
        }
        if host.session.as_ref().unwrap().phase == Phase::Planning && !host.session.as_ref().unwrap().plan_confirmed {
            planning(host);
            let s = host.session.as_mut().unwrap();
            s.auto_draft("aggressive");
            if !s.confirm_plan() {
                break;
            }
        }
        let s = host.session.as_mut().unwrap();
        let from = s.resolver.log.len();
        s.step_next();
        out.push(Step { lines: s.resolver.log[from..].to_vec(), events: event::since(&s.resolver.events, from).cloned().collect() });
    }
    out
}

/// The screen's cues of one step from its typed events: the sounds in line order, the impact modifiers (for both a
/// step that lowered HP and one that did not) and whether it broke a unit.
fn typed_cues(st: &Step) -> (Vec<&'static str>, Mods, Mods, bool) {
    (st.events.iter().filter_map(sound_of).collect(), Mods::of(&st.events, true), Mods::of(&st.events, false), fx::broke(&st.events))
}

fn prose_cues(st: &Step) -> (Vec<&'static str>, Mods, Mods, bool) {
    (st.lines.iter().filter_map(|l| prose::sound_of(l)).collect(), prose::mods(&st.lines, true), prose::mods(&st.lines, false), prose::broke(&st.lines))
}

/// The refusal words of `s`'s party at planning: each skill menu cell's (round intent: `menu_options` on itself and on
/// the first enemy on the field) and each classic probe's (every skill the member knows on that enemy, through the
/// host's `probe` JSON), typed and as the former prose reading worded them. Returns how many refusals it compared.
fn check_refusals(host: &mut Host) -> usize {
    let mut n = 0;
    let (allies, foe, timeline) = {
        let s = host.session.as_ref().unwrap();
        let foe = (0..s.w.units.len()).find(|&u| !s.w.allies.contains(&u) && s.w.units[u].is_active() && s.w.has(u));
        (s.w.allies.iter().copied().filter(|&h| s.w.units[h].is_active()).collect::<Vec<_>>(), foe, ri::on(&s.w))
    };
    let Some(foe) = foe else { return 0 };
    for h in allies {
        if timeline {
            let s = host.session.as_ref().unwrap();
            for at in [RoundTarget::Myself, RoundTarget::Unit(foe)] {
                for o in rip::menu_options(&s.w, &s.plan, h, at) {
                    let Some(p) = &o.problem else {
                        assert!(o.reasons.is_empty(), "a live cell has no reasons: {o:?}");
                        continue;
                    };
                    assert_eq!(&refusal::text(&o.reasons), p, "the fields give the planner's text");
                    assert_eq!(intent_text::menu_refusal_text(&o.reasons), prose::menu_refusal_text(&s.w, p), "{p}");
                    n += 1;
                }
            }
        } else {
            let ids: Vec<String> = {
                let s = host.session.as_ref().unwrap();
                s.w.cat.skills.iter().filter(|k| s.w.units[h].knows_skill(&s.w.cat, &k.id)).map(|k| k.id.clone()).collect()
            };
            for id in ids {
                let p = host.handle(&json!({"cmd": "probe", "unit": h, "step": {"mode": "engage", "skill": id, "target": foe}}));
                let Some(reason) = p["reason"].as_str().filter(|_| p["valid"] == json!(false)) else { continue };
                let reasons: Vec<refusal::Reason> = p["reasons"].as_array().expect("the reasons in fields").iter().filter_map(refusal::Reason::from_json).collect();
                assert_eq!(refusal::text(&reasons), reason);
                assert_eq!(intent_text::refusal_text(&reasons), prose::refusal_text(reason), "{reason}");
                n += 1;
            }
        }
    }
    n
}

/// The typed cues equal the former prose reading on every step of every catalog encounter (its own ruleset and the
/// timeline row, two seeds), and the refusal words on the first two rounds' menus and probes.
#[test]
fn typed_cues_and_refusals_match_the_former_prose_reading_on_every_encounter() {
    let data = data();
    let (mut total, mut refusals) = (std::collections::BTreeMap::<&str, usize>::new(), 0);
    let encounters: Vec<String> = data.cat.encounters.keys().cloned().collect();
    assert!(!encounters.is_empty());
    for enc in &encounters {
        for ruleset in ["", "timeline"] {
            for seed in [1, 2] {
                let Some(mut host) = battle(&data, enc, ruleset, seed) else { continue };
                let mut rounds = 0;
                let fight = steps(&mut host, |h| {
                    rounds += 1;
                    if rounds <= 2 {
                        refusals += check_refusals(h);
                    }
                });
                for (i, st) in fight.iter().enumerate() {
                    let (typed, prose) = (typed_cues(st), prose_cues(st));
                    assert_eq!(typed, prose, "{enc} {ruleset} seed {seed} step {i}: {:?} / {:?}", st.lines, st.events);
                    for k in &typed.0 {
                        *total.entry(k).or_insert(0) += 1;
                    }
                    if typed.3 {
                        *total.entry("break").or_insert(0) += 1;
                    }
                }
            }
        }
    }
    // the fights cue the common sounds (the rest are compared wherever they occur)
    for k in ["hit", "move_step"] {
        assert!(total.get(k).copied().unwrap_or(0) > 0, "{k}: {total:?}");
    }
    assert!(refusals > 0, "some refusal was compared");
}

/// Custom marks: the units renamed to the words the prose reading looked for (" CRIT", "[Break]", " for 0 ") and the
/// channeled mark renamed (a custom `terms` row) leave the same fight's typed cues as they were, while the prose
/// reading misfires on the renamed log; the Channel note and words read the renamed term.
#[test]
fn renamed_units_and_custom_marks_leave_the_typed_cues_alone() {
    let data = data();
    let enc = data.cat.encounters.keys().next().cloned().expect("an encounter");
    let mut plain = battle(&data, &enc, "", 1).expect("the battle starts");
    let mut renamed = battle(&data, &enc, "", 1).expect("the battle starts");
    {
        let s: &mut Session = renamed.session.as_mut().unwrap();
        for (i, u) in s.w.units.iter_mut().enumerate() {
            u.name = format!("[Break] U{i} CRIT for 0 ");
        }
        let cat = Arc::make_mut(&mut s.w.cat);
        let row = emberwake_data::terms::TermRow { id: emberwake_data::terms::CHANNELED_MARK.into(), name: "CRIT".into(), word: "inoculated".into(), ..Default::default() };
        cat.terms.insert(row.id.clone(), row);
    }
    let (a, b) = (steps(&mut plain, |_| {}), steps(&mut renamed, |_| {}));
    assert_eq!(a.len(), b.len(), "the same fight");
    assert_ne!(a.iter().map(|s| s.lines.clone()).collect::<Vec<_>>(), b.iter().map(|s| s.lines.clone()).collect::<Vec<_>>(), "the log names the units");
    let cues = |f: &[Step]| f.iter().map(typed_cues).collect::<Vec<_>>();
    assert_eq!(cues(&a), cues(&b), "the typed cues do not read the names");
    assert_ne!(b.iter().map(prose_cues).collect::<Vec<_>>(), cues(&b), "the prose reading misfires on the renamed log");
    // the Channel words are the term's
    let w = &renamed.session.as_ref().unwrap().w;
    assert_eq!(channeled_word(w), "inoculated");
    let skill = &w.cat.skills[0];
    assert_eq!(channel_note(w, skill, false, Some((1.25, true, 0))), Some("CRIT version: power x1.25, armour piercing".to_string()));
    let shipped = &plain.session.as_ref().unwrap().w;
    let mark = shipped.cat.term(emberwake_data::terms::CHANNELED_MARK);
    assert_eq!(channel_note(shipped, skill, false, Some((1.25, false, 0))), Some(format!("{} version: power x1.25", mark.name)));
    assert_eq!(channeled_word(shipped), mark.word);
}
