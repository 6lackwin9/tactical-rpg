//! PK1-S2c (PR5-6, `docs/sessions/PK1_CORRUPTION.md` 4.3, FB-174): the Channel round on the battle screen. The toggle
//! in the skill menu's neutral row (0 AP, 0 blocks, only for a unit that can Channel; it flips the round both ways and
//! converts the plan), the placeholder Channel tint on the unit and on the strip's blocks, the marrow mark in the
//! strip's names, Feed in Rest's place, Vent's two refusals in words, the Cold Steel rows greyed with "Channel round",
//! and the planner's reason for an AP hold. A catalog encounter played by the catalog's timeline row with the rules
//! `unit_meters`, `triggers`, `hp_cost` and `channel` switched on; the Strain meter, Feed and Vent are test rows put on
//! the battle's world (shipped data holds no Strain carrier yet, PR5-7). Every expectation is a kit query
//! (`channel::problem`, `channel::version`, `meter::gate_problem`, `command::hp_cost_for`, `trigger::State::ap_held`),
//! never a shipped number.

use crate::battle::input::{skill_text, RowAct, Row};
use crate::battle::skill_menu::{Slot, CHANNEL};
use crate::battle::tips::ap_held_text;
use crate::battle::BattleScreen;
use crate::intent_text;
use crate::screens::{Cx, Screen};
use aegis_ui::art::Art;
use aegis_ui::ui::Gfx;
use emberwake_battle::channel;
use emberwake_battle::meter;
use emberwake_battle::round_intent::{Pick, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_battle::skill::{self, Kind};
use emberwake_data::{SkillDef, UnitMeterDef};
use emberwake_rules::ruleset::TIMELINE;
use emberwake_rules::unit::HeldMeter;
use serde_json::{json, Value};

const RULES: &str = "+unit_meters,+triggers,+hp_cost,+channel";
const STRAIN: &str = "t_strain";
const FEED: &str = "t_feed";
const VENT: &str = "t_vent";
/// The test rows' numbers (the expectations derive from them or from the kit).
const VENT_GATE: i32 = 50;
const VENT_PCT: i32 = 15;

fn arena(rules: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), rules, false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn sid(b: &BattleScreen, id: &str) -> usize {
    s(b).w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The selected hero holds the test Strain meter from `strain` (0-100; its row's skills Feed: +5, a self heal; Vent:
/// -20, gate 50, 15% of max HP; both marrow-only); the ruleset names it the channel meter and Feed the Channel Rest; the
/// hero marks its first active with a marrow version (`marrow_actives` 1). Returns (hero, marked family or "").
fn strain_up(b: &mut BattleScreen, strain: i32) -> (usize, String) {
    let h = b.selected.expect("a hero");
    let sess = b.host.session.as_mut().unwrap();
    let w = &mut sess.w;
    let cat = std::sync::Arc::make_mut(&mut w.cat);
    let row = json!({"id": STRAIN, "name": "Strain", "short": "St", "min": 0, "max": 100, "start": 0, "show": "bar", "skills": [FEED, VENT]});
    let def = UnitMeterDef::from_obj(row.as_object().unwrap());
    cat.unit_meters.insert(def.id.clone(), def);
    let feed = json!({"id": FEED, "name": "Feed", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "heal_base": 3, "learned_only": true,
        "description": "Heals a little; Strain rises.", "channel": {"only": true}, "meters": [{"meter": STRAIN, "delta": 5}]});
    let vent = json!({"id": VENT, "name": "Vent", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "learned_only": true,
        "channel": {"only": true}, "meters": [{"meter": STRAIN, "delta": -20}], "requires_meter": {"meter": STRAIN, "min": VENT_GATE}, "hp_cost_pct": VENT_PCT});
    cat.add_skills(vec![SkillDef::from_obj(feed.as_object().unwrap()), SkillDef::from_obj(vent.as_object().unwrap())]);
    w.ruleset.numbers.channel_meter = STRAIN.into();
    w.ruleset.numbers.channel_rest = FEED.into();
    w.ruleset.numbers.marrow_actives = 1;
    w.units[h].passive_meters.push(HeldMeter { meter: STRAIN.into(), start: strain, floor: None });
    meter::battle_start(w);
    // the first hotkey attack with a marrow version is marked marrow (whatever the shipped deck holds), else the first
    // skill with one
    let versioned: Vec<&SkillDef> = rip::hotkey_skills(w, h).into_iter().map(|k| &w.cat.skills[k]).filter(|k| channel::has_version(w, k)).collect();
    let marked = versioned.iter().find(|k| skill::kind(k) == Kind::Attack).or(versioned.first()).map(|k| k.family_id().to_string()).unwrap_or_default();
    if !marked.is_empty() {
        let u = &mut w.units[h];
        u.deck.marrow = vec![marked.clone()];
        if !u.deck.active.contains(&marked) && u.deck_rule.is_none() {
            u.deck.active.push(marked.clone());
        }
    }
    (h, marked)
}

/// The Strain value `c` holds now.
fn set_strain(b: &mut BattleScreen, c: usize, v: i32) {
    let w = &mut b.host.session.as_mut().unwrap().w;
    let g = w.meters.gauges.iter_mut().find(|g| g.unit == c && g.meter == STRAIN).expect("the hero's Strain");
    g.value = v;
}

fn rows(b: &BattleScreen, h: usize, at: RoundTarget) -> Vec<Row> {
    b.menu_rows(h, at).1
}

fn toggle(rows: &[Row]) -> Option<&Row> {
    rows.iter().find(|r| r.slot == Some(Slot::Neutral(CHANNEL)))
}

fn skill_row<'a>(rows: &'a [Row], skill: usize) -> Option<&'a Row> {
    rows.iter().find(|r| matches!(&r.act, RowAct::Pick(p) if matches!(p[..], [Pick::Skill { skill: k, .. }] if k == skill)))
}

fn tip(r: &Row) -> Vec<String> {
    r.tip.iter().map(|t| t.0.clone()).collect()
}

fn strip_labels(b: &BattleScreen, cx: &Cx) -> Vec<Value> {
    b.state(cx)["strip_labels"].as_array().cloned().unwrap_or_default()
}

/// The first foe on the field.
fn foe(b: &BattleScreen) -> usize {
    let w = &s(b).w;
    w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe")
}

/// PK1 4.3 item 1: the toggle is a neutral button only for a hero holding the channel meter (never for one without it,
/// never without the rule); it is live at once and after picks (it never locks), flips the round to Channel and back,
/// and converts the plan: a pick with a version in the new mode stays, one without is dropped (`channel::problem`), and
/// every entry left runs in the new mode (a Move, or `channel::version`: marrow versions and Feed).
#[test]
fn the_toggle_shows_for_a_holder_only_and_flips_and_converts_the_round() {
    let (mut cx, mut g, mut b) = arena(RULES);
    // no holder yet: nobody has it
    let h = b.selected.expect("a hero");
    assert!(toggle(&rows(&b, h, RoundTarget::Myself)).is_none(), "no channel meter held: no toggle");
    let (h, _) = strain_up(&mut b, 0);
    let rs = rows(&b, h, RoundTarget::Myself);
    let t = toggle(&rs).expect("the holder's toggle");
    assert!(t.valid && matches!(t.act, RowAct::Channel(true)), "live, turns Channel on: {:?}", t.detail);
    let w = &s(&b).w;
    if let Some(m) = w.allies.iter().copied().find(|&a| a != h && w.units[a].is_active()) {
        assert!(toggle(&rows(&b, m, RoundTarget::Myself)).is_none(), "a member without the meter has no toggle");
    }
    // a pick first: the first live skill row on the foe
    let f = foe(&b);
    let pick = rows(&b, h, RoundTarget::Unit(f)).into_iter().find(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(p[..], [Pick::Skill { .. }])));
    if let Some(r) = &pick {
        let RowAct::Pick(p) = &r.act else { unreachable!() };
        b.choose_pick(&cx, h, r.at.unwrap_or(RoundTarget::Unit(f)), p, &r.label);
    }
    let before = rip::picks_of(&s(&b).w, h);
    assert_eq!(before.len(), usize::from(pick.is_some()), "{}", b.message);
    // the kit's verdict for each pick in a Channel round
    let mut w2 = s(&b).w.clone();
    w2.round_states.entry(h).or_default().channel = true;
    let kept: Vec<Pick> = before.iter().filter(|p| !matches!(p, Pick::Skill { skill, .. } if channel::problem(&w2, h, &w2.cat.skills[*skill]).is_some())).cloned().collect();
    // the menu on the round's target keeps it after a pick
    let at = s(&b).w.round_states.get(&h).and_then(|r| r.target).unwrap_or(RoundTarget::Myself);
    let t = toggle(&rows(&b, h, at)).cloned().expect("still there after a pick");
    assert!(t.valid, "it never locks");
    b.set_channel(&cx, h, at, true);
    let w = &s(&b).w;
    assert!(channel::is_channel(w, h), "{}", b.message);
    assert_eq!(rip::picks_of(w, h), kept, "the picks with a marrow counterpart stay, the others drop");
    for e in s(&b).plan.entries_for(w, h) {
        let sk = &w.cat.skills[e.skill];
        assert!(skill::kind(sk) == Kind::Move || channel::version(w, h, sk).is_some(), "{} runs in the Channel mode", sk.name);
    }
    let at = s(&b).w.round_states.get(&h).and_then(|r| r.target).unwrap_or(RoundTarget::Myself);
    let hist = b.host.handle(&json!({"cmd": "history"}))["history"].as_array().cloned().unwrap_or_default();
    assert_eq!(hist.last().map(|r| r["cmd"].clone()), Some(json!("channel")), "the toggle is a recorded host request");
    let t = toggle(&rows(&b, h, at)).cloned().expect("the toggle while on");
    assert!(t.valid && matches!(t.act, RowAct::Channel(false)), "on: the next click turns it off");
    // and back to Cold Steel
    b.set_channel(&cx, h, at, false);
    let w = &s(&b).w;
    assert!(!channel::is_channel(w, h));
    for e in s(&b).plan.entries_for(w, h) {
        let sk = &w.cat.skills[e.skill];
        assert!(channel::problem(w, h, sk).is_none() && channel::version(w, h, sk).is_none(), "{} is a Cold Steel entry", sk.name);
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    // without the rule nobody has it, holder or not
    let (_cx, _g, mut b) = arena("+unit_meters,+triggers,+hp_cost");
    let (h, _) = strain_up(&mut b, 0);
    assert!(toggle(&rows(&b, h, RoundTarget::Myself)).is_none(), "no rule channel: no toggle");
}

/// PK1 4.3 item 1: a picked Cold Steel skill with no counterpart in a Channel round (not marked marrow, not the class
/// basic: `channel::problem`) is dropped when the round turns Channel, the message names the drop, and the freed
/// blocks refill in the Channel mode.
#[test]
fn a_pick_without_a_counterpart_drops_when_the_round_turns_channel() {
    let (cx, _g, mut b) = arena(RULES);
    let (h, _) = strain_up(&mut b, 0);
    let f = foe(&b);
    let mut w2 = s(&b).w.clone();
    w2.round_states.entry(h).or_default().channel = true;
    let cold = rows(&b, h, RoundTarget::Unit(f)).into_iter().find(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(p[..], [Pick::Skill { skill, .. }] if channel::problem(&w2, h, &w2.cat.skills[skill]).is_some())));
    let Some(r) = cold else { return };
    let RowAct::Pick(p) = &r.act else { unreachable!() };
    b.choose_pick(&cx, h, r.at.unwrap_or(RoundTarget::Unit(f)), p, &r.label);
    assert_eq!(rip::picks_of(&s(&b).w, h).len(), 1, "{}", b.message);
    let at = s(&b).w.round_states.get(&h).and_then(|r| r.target).unwrap_or(RoundTarget::Unit(f));
    b.set_channel(&cx, h, at, true);
    let w = &s(&b).w;
    assert!(rip::picks_of(w, h).is_empty(), "no counterpart: dropped");
    assert!(b.message.contains("dropped") && b.message.contains(channel::CHANNEL_ROUND), "{}", b.message);
    let entries = s(&b).plan.entries_for(w, h);
    assert!(!entries.is_empty(), "the auto-fill refills the round");
    for e in entries {
        let sk = &w.cat.skills[e.skill];
        assert!(skill::kind(sk) == Kind::Move || channel::version(w, h, sk).is_some(), "{} runs in the Channel mode", sk.name);
    }
}

/// PK1 4.3 item 6 (placeholder, user 2026-11-10): while the round is a Channel round the unit wears the tint
/// (`channel_units`) and every strip block in the Channel mode is tinted (`strip_labels` `channel`), a marrow
/// version's name with the marrow mark; in Cold Steel neither.
#[test]
fn a_channel_round_tints_the_unit_and_its_blocks() {
    let (mut cx, mut g, mut b) = arena(RULES);
    let (h, _) = strain_up(&mut b, 0);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.state(&cx)["channel_units"], json!([]), "Cold Steel: no tint");
    assert!(strip_labels(&b, &cx).iter().all(|l| l.get("channel").is_none()), "Cold Steel: no tinted block");
    b.set_channel(&cx, h, RoundTarget::Myself, true);
    // a marrow pick on the foe when one is live (the ghost may hold none)
    let f = foe(&b);
    if let Some(r) = rows(&b, h, RoundTarget::Unit(f)).into_iter().find(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(p[..], [Pick::Skill { .. }]))) {
        let RowAct::Pick(p) = &r.act else { unreachable!() };
        b.choose_pick(&cx, h, r.at.unwrap_or(RoundTarget::Unit(f)), p, &r.label);
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.state(&cx)["channel_units"], json!([h]), "the hero wears the tint");
    let labels = strip_labels(&b, &cx);
    let w = &s(&b).w;
    let entries = s(&b).plan.entries_for(w, h);
    assert_eq!(labels.len(), entries.len(), "{labels:?}");
    for (l, e) in labels.iter().zip(&entries) {
        let sk = &w.cat.skills[e.skill];
        let tinted = e.item.is_none() && channel::version(w, h, sk).is_some();
        assert_eq!(l.get("channel").is_some(), tinted, "{l}");
        assert_eq!(l["name"], json!(skill_text(w, h, e.skill)), "the strip's name");
        if channel::has_version(w, sk) {
            assert_eq!(l["name"], json!(channel::marrow_name(&w.cat, &sk.name)), "a marrow version carries the mark");
        }
    }
    assert!(labels.iter().any(|l| l.get("channel").is_some()), "a Channel round plans in the Channel mode: {labels:?}");
}

/// PK1 4.3 item 5: Rest's button names the Channel Rest (Feed) in a Channel round, its card says so, and no plain Rest
/// block is left in the round; back in Cold Steel it is Rest again.
#[test]
fn feed_takes_rests_place_in_a_channel_round() {
    let (cx, _g, mut b) = arena(RULES);
    let (h, _) = strain_up(&mut b, 0);
    let rest_of = |b: &BattleScreen| rows(b, h, RoundTarget::Myself).into_iter().find(|r| r.slot == Some(Slot::Neutral("Rest")));
    assert_eq!(rest_of(&b).map(|r| r.label), Some("Rest".to_string()), "Cold Steel: Rest");
    b.set_channel(&cx, h, RoundTarget::Myself, true);
    let feed = &s(&b).w.cat.skills[sid(&b, FEED)];
    assert_eq!(channel::rest_skill(&s(&b).w, h), sid(&b, FEED), "the kit's Channel Rest");
    let r = rest_of(&b).expect("the Rest button");
    assert_eq!(r.label, feed.name, "Feed in Rest's place");
    let t = tip(&r);
    assert!(t.iter().any(|l| l.contains("replaces Rest")), "{t:?}");
    let w = &s(&b).w;
    assert!(s(&b).plan.entries_for(w, h).iter().all(|e| skill::kind(&w.cat.skills[e.skill]) != Kind::Rest), "no plain Rest block in a Channel round");
    let (_, grid_rows) = b.menu_rows(h, RoundTarget::Myself);
    let grid = crate::battle::skill_menu::MenuGrid::new(&[], Vec::new(), &grid_rows, None);
    assert_eq!(grid.rest_name, feed.name, "the button is measured for its name");
    b.set_channel(&cx, h, RoundTarget::Myself, false);
    assert_eq!(rest_of(&b).map(|r| r.label), Some("Rest".to_string()), "Cold Steel again: Rest");
}

/// PK1 4.3 item 5 (Vent's two reasons): in a Channel round Vent is greyed with its meter gate in words below 50 Strain
/// (`meter::gate_problem`) and with its HP cost in words when the HP left does not exceed `hp_cost_pct` of max HP
/// (`command::hp_cost_for`, `hp_cost_problem`); with the Strain and the HP it is live.
#[test]
fn vent_shows_its_two_reasons() {
    let (cx, _g, mut b) = arena(RULES);
    let low = VENT_GATE - 20;
    let (h, _) = strain_up(&mut b, low);
    b.set_channel(&cx, h, RoundTarget::Myself, true);
    let vent = sid(&b, VENT);
    let w = &s(&b).w;
    let gate = meter::gate_problem(w, w.cat.skills[vent].requires_meter.as_ref(), |_| Some(low));
    assert!(!gate.is_empty());
    let rs = rows(&b, h, RoundTarget::Myself);
    let r = skill_row(&rs, vent).expect("Vent's cell");
    assert!(!r.valid, "below the gate");
    assert_eq!(r.detail, intent_text::menu_refusal_text(w, &gate));
    assert_eq!(r.detail, format!("Needs Strain {VENT_GATE} or more to start: {low}"));
    assert!(tip(r).contains(&r.detail), "{:?}", tip(r));
    // the Strain to pay, the HP not
    set_strain(&mut b, h, VENT_GATE + 10);
    let w = &s(&b).w;
    let cost = emberwake_battle::command::hp_cost_for(w, h, &w.cat.skills[vent]);
    assert_eq!(cost, w.units[h].max_hp * VENT_PCT / 100, "the percent HP cost");
    b.host.session.as_mut().unwrap().w.units[h].hp = cost;
    let rs = rows(&b, h, RoundTarget::Myself);
    let r = skill_row(&rs, vent).expect("Vent's cell");
    assert!(!r.valid, "not the HP to pay");
    assert_eq!(r.detail, intent_text::refusal_text(&emberwake_battle::command::hp_cost_problem(cost, cost)));
    assert!(r.detail.starts_with("Not enough HP"), "{}", r.detail);
    // both met: live
    let max = s(&b).w.units[h].max_hp;
    b.host.session.as_mut().unwrap().w.units[h].hp = max;
    let rs = rows(&b, h, RoundTarget::Myself);
    let r = skill_row(&rs, vent).expect("Vent's cell");
    assert!(r.valid, "{}", r.detail);
}

/// PK1 4.3 item 2: in a Channel round each Cold Steel skill the round does not offer (`channel::problem`) is greyed with
/// "Channel round" on its cell and card; in a Cold Steel round a marrow-only skill is greyed with "Only in a Channel
/// round".
#[test]
fn cold_steel_rows_grey_in_a_channel_round() {
    let (cx, _g, mut b) = arena(RULES);
    let (h, _) = strain_up(&mut b, 0);
    let f = foe(&b);
    // every such cell is greyed; on its own kind of target (else the kit names the target first) with the reason
    let check = |b: &BattleScreen, at: RoundTarget, want: &str| -> usize {
        let w = &s(b).w;
        let mut n = 0;
        for r in rows(b, h, at) {
            let RowAct::Pick(p) = &r.act else { continue };
            let [Pick::Skill { skill, .. }] = p[..] else { continue };
            if channel::problem(w, h, &w.cat.skills[skill]) == Some(want) {
                assert!(!r.valid, "{} greyed", r.label);
                assert_eq!(r.label.contains(&channel::marrow_name(&w.cat, "")), false, "a greyed Cold Steel cell keeps its own name: {}", r.label);
                if r.detail == want {
                    assert!(tip(&r).iter().any(|l| l == want), "{:?}", tip(&r));
                    n += 1;
                }
            }
        }
        n
    };
    // Cold Steel: Vent and Feed (marrow-only) on the self menu
    assert!(check(&b, RoundTarget::Myself, channel::ONLY_CHANNEL) > 0, "the marrow-only skills grey in Cold Steel");
    b.set_channel(&cx, h, RoundTarget::Myself, true);
    let greyed = check(&b, RoundTarget::Unit(f), channel::CHANNEL_ROUND) + check(&b, RoundTarget::Myself, channel::CHANNEL_ROUND);
    assert!(greyed > 0, "a hero with Cold Steel skills beyond its marked one sees them grey");
}

/// FB-174 (PK1-S1 `ap` set): a hero whose AP a trigger holds this round sees why in its panel (the kit's
/// `State::ap_held`); without a hold there is no such line.
#[test]
fn an_ap_hold_shows_its_reason_in_the_planner() {
    let (mut cx, mut g, mut b) = arena(RULES);
    let h = b.selected.expect("a hero");
    let has_line = |recs: &[aegis_ui::painter::TextRec]| recs.iter().any(|r| r.text.starts_with("AP held at"));
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(!has_line(&recs), "no hold: no line");
    {
        let sess = b.host.session.as_mut().unwrap();
        let cat = sess.w.cat.clone();
        let e = json!({"type": "trigger", "on": "round_end", "when": {}, "effects": [{"type": "ap", "set": 0}], "limit": "none"});
        sess.w.units[h].apply_effects(&cat, &[e], "passive:collapse", false);
        emberwake_battle::trigger::round_end(&mut sess.resolver, &mut sess.w);
        emberwake_battle::trigger::apply_ap_sets(&mut sess.resolver, &mut sess.w);
    }
    let held = s(&b).resolver.triggers.ap_held(h).map(|(ap, src)| ap_held_text(ap, src)).expect("the kit holds the hero's AP");
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let text: String = recs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(text.contains(&held) || recs.iter().any(|r| held.starts_with(&r.text) && r.text.starts_with("AP held at")), "the panel names the hold ({held}):\n{text}");
}
