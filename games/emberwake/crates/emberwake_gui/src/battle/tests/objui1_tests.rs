//! OBJUI1 (`PR7-31`; `OBJ1-U` with `MORALE1-U`, `docs/archive/specs/emberwake/OBJ1.md` and `MORALE1.md` section 3):
//! the board's objective and morale chips, the hover card's and the foe card's words, the goal's hover and the
//! results' lines, on shipped fights: the Sunk Ward sweep (`enc_evt_01`, morale on), the Sunk Ward bone-mill raid
//! (`enc_evt_03`, a guest), the Ashlands wagon raid (a guest who must survive) and the Ashmaw stand (slay). Every word
//! is the kit's (`emberwake_battle::obj_view`); the morale states are set on the session in memory, no data changes.

use crate::battle::BattleScreen;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::obj_view::{self as ov, FAILED_LINE, INSPECT_LINE, LEADER_LINE};
use emberwake_battle::session::Session;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

fn fight(world: &str, encounter: &str, view: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!(view);
    let mut b = BattleScreen::arena_with(&cx, Some(world.into()), encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a aegis_ui::ui::Widget<crate::battle::Action>> {
    b.w.iter().find(|w| w.id == id)
}

fn tip_lines(b: &BattleScreen, id: &str) -> Vec<String> {
    widget(b, id).unwrap_or_else(|| panic!("no widget {id}")).tip.iter().map(|t| t.0.clone()).collect()
}

fn standing_foes(b: &BattleScreen) -> Vec<usize> {
    let w = &s(b).w;
    w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.grid.pos(e).is_some()).collect()
}

/// MORALE1 3: with morale on, every standing foe wears its state chip on both boards, its title the kit's word; the
/// leader wears the pennant; the hover card says the state with its line and the leader's line; no foe has an
/// Enraged chip.
#[test]
fn each_foe_wears_its_morale_chip_and_the_leader_its_pennant() {
    for view in ["flat", "stage"] {
        let (_cx, _g, b) = fight("sunk_ward", "enc_evt_01", view);
        assert!(s(&b).morale.on, "the sweep has morale");
        let foes = standing_foes(&b);
        assert!(!foes.is_empty());
        for &f in &foes {
            let m = ov::morale_mark(s(&b), f).expect("a morale mark");
            let chip = widget(&b, &format!("board.morale.{f}")).unwrap_or_else(|| panic!("{view}: foe {f} has its chip"));
            assert_eq!(chip.text, m.title());
            assert_eq!(tip_lines(&b, &format!("board.morale.{f}")), vec![m.title(), m.line.to_string(), INSPECT_LINE.to_string()]);
            let card = tip_lines(&b, &format!("unit.{f}"));
            assert!(card.contains(&format!("{}: {}", m.title(), m.line)), "{view}: the hover card says {:?}: {card:?}", m.title());
            let leads = s(&b).morale_leader() == Some(f);
            assert_eq!(widget(&b, &format!("board.leader.{f}")).is_some(), leads, "{view}: the pennant only on the leader");
            assert_eq!(card.contains(&LEADER_LINE.to_string()), leads);
        }
        for &h in &s(&b).w.allies {
            assert!(widget(&b, &format!("board.morale.{h}")).is_none(), "no morale on the party side");
        }
        assert!(!b.w.iter().any(|w| w.id.contains("enrage")), "no Enraged chip (ENRAGE2 EN5)");
    }
}

/// MORALE1 3: the chip follows the group's state (Steady, Shaken, Wavering by the group's value against the break
/// line); a fearless unit stays Fearless; with two groups the title adds the group letter.
#[test]
fn the_chip_follows_the_groups_state_and_names_the_group() {
    let (mut cx, mut g, mut b) = fight("sunk_ward", "enc_evt_01", "flat");
    let foes = standing_foes(&b);
    let plain: Vec<usize> = foes.iter().copied().filter(|&f| !s(&b).is_fearless(f)).collect();
    assert!(!plain.is_empty(), "the sweep has a foe that can break");
    let f = plain[0];
    let brk = s(&b).morale.brk;
    for (value, word) in [(100, "Steady"), (brk + (100 - brk) / 2, "Shaken"), (brk + 1, "Wavering")] {
        let gi = s(&b).morale_group_of(f) as usize;
        b.host.session.as_mut().unwrap().morale.groups[gi].value = value;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(widget(&b, &format!("board.morale.{f}")).unwrap().text, word, "value {value}");
    }
    for &x in foes.iter().filter(|&&x| s(&b).is_fearless(x)) {
        assert_eq!(widget(&b, &format!("board.morale.{x}")).unwrap().text, "Fearless");
    }
    // a second group: the titles name their group
    let mut extra = s(&b).morale.groups[0].clone();
    extra.members.clear();
    b.host.session.as_mut().unwrap().morale.groups.push(extra);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let title = widget(&b, &format!("board.morale.{f}")).unwrap().text.clone();
    assert!(title.ends_with(", group A"), "{title}");
}

/// OBJ1 3: a guest wears the shield and its hover card says "Guest"; only a protected guest adds "Must survive" (and
/// the gold rim's words on the chip).
#[test]
fn a_guest_wears_the_shield_and_a_protected_one_must_survive() {
    let (_cx, _g, b) = fight("sunk_ward", "enc_evt_03", "flat");
    let guests = s(&b).guests.clone();
    assert!(!guests.is_empty(), "the raid fields a guest");
    for &x in &guests {
        assert_eq!(widget(&b, &format!("board.guest.{x}")).expect("the shield").text, "Guest");
        let card = tip_lines(&b, &format!("unit.{x}"));
        assert!(card.contains(&"Guest".to_string()), "{card:?}");
        assert!(!card.contains(&"Must survive".to_string()), "not protected here");
    }
    let (_cx, _g, b) = fight("ashlands", "wagon_raid", "stage");
    let protected = s(&b).protected.clone();
    assert!(!protected.is_empty(), "the wagon raid protects its guest");
    for &x in &protected {
        assert_eq!(widget(&b, &format!("board.guest.{x}")).expect("the shield").text, "Guest, must survive");
        assert!(tip_lines(&b, &format!("board.guest.{x}")).contains(&"Must survive".to_string()));
        assert!(tip_lines(&b, &format!("unit.{x}")).contains(&"Must survive".to_string()));
    }
    for &h in s(&b).w.allies.iter().filter(|h| !s(&b).guests.contains(h)) {
        assert!(widget(&b, &format!("board.guest.{h}")).is_none(), "heroes wear no shield");
    }
}

/// OBJ1 3: the slay target wears the crown chip and its hover card says so; no other unit does.
#[test]
fn the_slay_target_wears_the_crown() {
    let (_cx, _g, b) = fight("ashlands", "ashmaw_stand", "flat");
    let t = s(&b).slay_target().expect("a slay target");
    assert_eq!(widget(&b, &format!("board.slay.{t}")).expect("the crown").text, "Slay target");
    assert!(tip_lines(&b, &format!("unit.{t}")).iter().any(|l| l.starts_with("Slay target")));
    for u in (0..s(&b).w.units.len()).filter(|&u| u != t) {
        assert!(widget(&b, &format!("board.slay.{u}")).is_none());
    }
}

/// MORALE1 3, OBJ1 rule 9: the goal hovers to the full objective line and the morale line; with morale off only the
/// objective line.
#[test]
fn the_goal_hovers_to_the_objective_and_the_morale_line() {
    let (_cx, _g, b) = fight("sunk_ward", "enc_evt_01", "flat");
    let lines = tip_lines(&b, "strip.goal");
    let header = ov::header_line(s(&b));
    assert!(!header.is_empty());
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(lines[1], header);
    let (_cx, _g, b) = fight("sunk_ward", "enc_evt_03", "flat");
    assert!(!s(&b).morale.on);
    assert_eq!(tip_lines(&b, "strip.goal").len(), 1, "no morale line");
}

/// MORALE1 3: the header line by the break line, the groups and the leader.
#[test]
fn the_morale_line_reads_the_rule() {
    let (_cx, _g, mut b) = fight("sunk_ward", "enc_evt_01", "flat");
    let leader = s(&b).morale_leader().map(|c| s(&b).w.units[c].name.clone());
    let one = ov::header_line(s(&b));
    match &leader {
        Some(l) => assert_eq!(one, format!("They give up when {l} falls or half of them are down; shaken ones run sooner.")),
        None => assert_eq!(one, "They give up when half of them are down; shaken ones run sooner."),
    }
    b.host.session.as_mut().unwrap().morale.brk = 30;
    assert_eq!(ov::header_line(s(&b)), "They give up as their losses mount.");
}

/// OBJ1 3, MORALE1 3: the results' objective line: "Objective met: <goal>" on a win, "The objective failed." otherwise;
/// after a win by a break the break lines follow; the results state carries them and draws them gold first.
#[test]
fn the_results_read_the_objective_and_a_breaks_lines() {
    let (_cx, _g, mut b) = fight("sunk_ward", "enc_evt_01", "flat");
    let goal = emberwake_battle::encounter::Objectives::goal_of(&s(&b).def.raw, |r| r.to_string());
    assert_eq!(ov::result_lines(s(&b), "won"), vec![format!("Objective met: {goal}")]);
    assert_eq!(ov::result_lines(s(&b), "lost"), vec![FAILED_LINE.to_string()]);
    assert_eq!(ov::result_lines(s(&b), "fled"), vec![FAILED_LINE.to_string()]);
    let ses = b.host.session.as_mut().unwrap();
    ses.morale.groups[0].broken = true;
    ses.morale.groups[0].line = "The rest throw down their arms.".into();
    assert_eq!(ov::result_lines(s(&b), "won"), vec![format!("Objective met: {goal}"), "The rest throw down their arms.".to_string()]);
    assert_eq!(ov::result_lines(s(&b), "lost"), vec![FAILED_LINE.to_string()], "a loss shows no break line");
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut g = crate::hud::hud_tests::gfx();
    let res = aegis_core::json::obj([("ok", json!(true))]);
    let mut r = crate::hub::Results::new(&res, "won", &aegis_core::json::Dict::new()).objective(ov::result_lines(s(&b), "won"));
    r.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(r.state(&cx)["objective"], json!([format!("Objective met: {goal}"), "The rest throw down their arms."]));
    let none = crate::hub::Results::new(&res, "won", &aegis_core::json::Dict::new());
    assert!(none.state(&cx).get("objective").is_none(), "results outside a battle read as before");
}
