//! PR7-41 (PT-23): pouch items and ally-target skills are pickable in the plan without a modifier key. The hero panel's
//! Ally button lists the other members the selected hero can use a skill or pouch item on (the kit's `menu_options`);
//! a row opens the skill menu on that member (what Ctrl+click on its token does), where the cell picks as any other.
//! A refused cell that needs yourself or an ally names the control in its hover card. A catalog encounter on the
//! catalog's timeline row; the item is a catalog healing consumable put in the hero's pouch (no shipped numbers).

use crate::battle::input::RowAct;
use crate::battle::{Action, BattleScreen};
use crate::intent_text;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::round_intent::{self as ri, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_battle::world::World;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn w(b: &BattleScreen) -> &World {
    &s(b).w
}

fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

/// A timeline arena, planning, with the selected hero.
fn planning() -> (Cx, Gfx, BattleScreen, usize) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    let mut g = crate::hud::hud_tests::gfx();
    drawn(&mut cx, &mut g, &mut b);
    let h = b.selected.expect("a selected hero");
    (cx, g, b, h)
}

/// The other members on the field.
fn others(b: &BattleScreen, h: usize) -> Vec<usize> {
    w(b).allies.iter().copied().filter(|&a| a != h && w(b).units[a].is_active() && w(b).has(a)).collect()
}

/// Empties every member's pouch (so only what a test gives counts) and gives `h` two copies of a catalog healing
/// consumable offered in battle; its id.
fn give_salve(cx: &Cx, b: &mut BattleScreen, h: usize) -> String {
    let item = cx.data.cat.items.values().filter(|i| i.offered_in_battle() && !i.aims_at_foes() && i.heal > 0 && !i.is_area()).min_by(|a, b| a.id.cmp(&b.id)).cloned().expect("a catalog healing consumable");
    let sess = b.host.session.as_mut().unwrap();
    for u in sess.w.units.iter_mut() {
        u.inventory.pouch.clear();
    }
    sess.w.units[h].inventory.pouch.push(item.clone());
    sess.w.units[h].inventory.pouch.push(item.clone());
    sess.w.add_item_skills();
    b.after_change(cx);
    item.id
}

/// Clicks the hero panel's Ally button through its drawn widget.
fn click_ally(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    drawn(cx, g, b);
    let wd = b.w.iter().find(|x| x.id == "hero.ally").expect("the Ally button");
    assert!(wd.enabled, "the Ally button is live while planning with another member on the field");
    assert!(matches!(wd.action, Some(Action::AllyTarget)));
    b.run(cx, Action::AllyTarget);
}

#[test]
fn the_ally_button_lists_the_members_and_a_row_opens_the_skill_menu_where_the_pouch_item_picks() {
    let (mut cx, mut g, mut b, h) = planning();
    let mates = others(&b, h);
    assert!(!mates.is_empty(), "the arena fields a second member");
    let id = give_salve(&cx, &mut b, h);
    click_ally(&mut cx, &mut g, &mut b);
    let list = b.popup.clone().expect("the ally list");
    assert!(list.menu.is_none() && list.title.contains("use on an ally"), "{}", list.title);
    let listed: Vec<usize> = list.rows.iter().filter_map(|r| if let RowAct::AllyMenu(a) = r.act { Some(a) } else { None }).collect();
    // the kit's menu agrees: a member is listed exactly when a cell on it is open
    for &a in &mates {
        let open = rip::menu_options(w(&b), &s(&b).plan, h, RoundTarget::Unit(a)).iter().any(|o| o.problem.is_none() && o.at == RoundTarget::Unit(a) && o.picks.iter().any(|p| p.holds_target()));
        assert_eq!(listed.contains(&a), open, "member {a} listed only when the hero can use something on it");
    }
    let (k, o) = list.rows.iter().enumerate().find_map(|(k, r)| if let RowAct::AllyMenu(a) = r.act { Some((k, a)) } else { None }).expect("the salve reaches a member");
    b.run(&mut cx, Action::Choose(k));
    assert_eq!(b.selected, Some(h), "the hero stays selected");
    let menu = b.popup.clone().expect("the skill menu on the member");
    assert_eq!(menu.menu, Some((h, RoundTarget::Unit(o))));
    let cell = menu.rows.iter().position(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(&p[..], [rip::Pick::Item(i)] if *i == id))).expect("a live cell of the pouch item on the member");
    b.run(&mut cx, Action::Choose(cell));
    assert!(rip::picks_of(w(&b), h).iter().any(|p| matches!(p, rip::Pick::Item(i) if *i == id)), "the item is picked: {}", b.message);
    assert_eq!(w(&b).round_states.get(&h).and_then(|r| r.target), Some(RoundTarget::Unit(o)), "on the member");
}

#[test]
fn an_ally_target_skill_picks_through_the_ally_button() {
    let (mut cx, mut g, mut b, h) = planning();
    let mates = others(&b, h);
    assert!(!mates.is_empty(), "the arena fields a second member");
    // a catalog support skill aimed at an ally that the hero can use on a member (learned in memory when it has none)
    let reaches = |b: &BattleScreen| -> Option<(usize, usize)> {
        others(b, h).into_iter().find_map(|a| {
            rip::menu_options(w(b), &s(b).plan, h, RoundTarget::Unit(a)).iter().find_map(|o| match &o.picks[..] {
                [rip::Pick::Skill { skill, .. }] if o.problem.is_none() && o.at == RoundTarget::Unit(a) => Some((a, *skill)),
                _ => None,
            })
        })
    };
    if reaches(&b).is_none() {
        let support: Vec<String> = w(&b).cat.skills.iter().filter(|s| s.kind == "SUPPORT" && !s.system && s.target_mode == "ALLY" && ri::skill_intent(s) == "support").map(|s| s.id.clone()).collect();
        for sid in support {
            let u = &mut b.host.session.as_mut().unwrap().w.units[h];
            u.meter_skills.push(sid.clone());
            u.known_skills.push(sid);
            b.after_change(&cx);
            if reaches(&b).is_some() {
                break;
            }
            let u = &mut b.host.session.as_mut().unwrap().w.units[h];
            u.meter_skills.pop();
            u.known_skills.pop();
        }
    }
    let (o, skill) = reaches(&b).expect("an ally-target skill the hero can use on a member");
    click_ally(&mut cx, &mut g, &mut b);
    let k = b.popup.as_ref().unwrap().rows.iter().position(|r| matches!(r.act, RowAct::AllyMenu(a) if a == o)).expect("the member is listed");
    b.run(&mut cx, Action::Choose(k));
    let cell = b.popup.as_ref().expect("the skill menu").rows.iter().position(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(&p[..], [rip::Pick::Skill { skill: x, .. }] if *x == skill))).expect("the skill's live cell");
    b.run(&mut cx, Action::Choose(cell));
    assert!(rip::picks_of(w(&b), h).iter().any(|p| matches!(p, rip::Pick::Skill { skill: x, .. } if *x == skill)), "the skill is picked: {}", b.message);
}

#[test]
fn with_nothing_for_an_ally_the_button_says_so() {
    let (mut cx, mut g, mut b, h) = planning();
    let sess = b.host.session.as_mut().unwrap();
    for u in sess.w.units.iter_mut() {
        u.inventory.pouch.clear();
    }
    b.after_change(&cx);
    let open = others(&b, h).into_iter().any(|a| rip::menu_options(w(&b), &s(&b).plan, h, RoundTarget::Unit(a)).iter().any(|o| o.problem.is_none() && o.at == RoundTarget::Unit(a) && o.picks.iter().any(|p| p.holds_target())));
    click_ally(&mut cx, &mut g, &mut b);
    if open {
        assert!(b.popup.is_some(), "a hero with an ally pick lists its members");
    } else {
        assert!(b.popup.is_none(), "no list");
        assert!(b.message.contains("no skill or pouch item to use on an ally"), "{}", b.message);
    }
}

#[test]
fn a_cell_that_needs_an_ally_or_yourself_names_the_control() {
    let (mut cx, _g, mut b, h) = planning();
    let id = give_salve(&cx, &mut b, h);
    let foe = w(&b).enemies.iter().copied().find(|&e| w(&b).units[e].is_active() && w(&b).has(e)).expect("a foe");
    // the menu on a foe: the salve's cells there are refused for their target, and each card says how to give it one
    b.run(&mut cx, Action::UnitClick(foe));
    let menu = b.popup.clone().expect("the skill menu on the foe");
    let cells: Vec<_> = menu.rows.iter().filter(|r| !r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(&p[..], [rip::Pick::Item(i)] if *i == id))).collect();
    assert!(!cells.is_empty(), "the salve has refused cells on a foe");
    for r in cells {
        assert!(r.tip.iter().any(|t| t.0.contains("Ally button") || t.0.contains("Self button")), "the card names the control: {:?}", r.tip);
    }
    // the words, read from the kit's refusal field (HITCALC1: `Reason::NeedsTarget`, never its text)
    use emberwake_battle::refusal::{Reason, TargetNeed};
    let reason = |n: TargetNeed| vec![Reason::NeedsTarget(n)];
    assert!(intent_text::target_control(&reason(TargetNeed::Ally)).is_some_and(|h| h.contains("Ally button") && h.contains("Ctrl+click")));
    assert!(intent_text::target_control(&reason(TargetNeed::Yourself)).is_some_and(|h| h.contains("Self button")));
    assert!(intent_text::target_control(&reason(TargetNeed::YourselfOrAlly)).is_some_and(|h| h.contains("Self") && h.contains("Ally")));
    assert_eq!(intent_text::target_control(&reason(TargetNeed::Enemy)), None, "an enemy is clicked on the board");
    // the kit's text alone names no control
    assert_eq!(intent_text::target_control(&[Reason::Text("Target an ally to use this".into())]), None);
}
