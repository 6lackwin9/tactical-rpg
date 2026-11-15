//! GEAR-REWORK1 (PR5-51a pass 2, `docs/sessions/U3_GEAR_REWORK.md` 4): the battle skill menu's Off-Hand button. A
//! catalog encounter under the catalog's timeline row with the rule `unit_meters` switched on; the off-hand items, their
//! skills and the ammo meter are test rows put on the battle's world (shipped data holds no item with the hand fields
//! yet, PR5-51b), worn by the selected hero. The button is a neutral-row button only for a hero whose off hand has an
//! action (`offhand::action`): it names the action (Fade-Step, Reload, the jab), takes its skill's cell out of the
//! intent rows, picks it like any cell, and greys with the kit's reason (a Reload above 0). With today's data no hero
//! has one. Every expectation is a kit query or a test row, never a shipped number.

use crate::battle::input::{Row, RowAct};
use crate::battle::skill_menu::{Slot, OFF_HAND};
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::art::Art;
use aegis_ui::ui::Gfx;
use emberwake_battle::meter;
use emberwake_battle::offhand::RELOAD_NOT_EMPTY;
use emberwake_battle::round_intent::{Pick, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_data::{ItemDef, SkillDef, UnitMeterDef};
use emberwake_rules::gear::{Equipment, Slot as GearSlot};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

const AMMO: &str = "t_ammo";
const AMMO_MAX: i32 = 3;

fn arena() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "+unit_meters", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn foe(b: &BattleScreen) -> usize {
    let w = &s(b).w;
    w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe")
}

fn off_item(id: &str, family: &str, extra: Value) -> Value {
    let mut row = json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": family, "base_power": 0, "family_slot_rule": "off_only", "off_hand_category": family});
    row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    row
}

/// The selected hero wears test item `item` in the off hand (the test rows are added to the battle's catalog first: a
/// cloak with Fade-Step, a quiver with Reload and the ammo meter, a light knife reaching 2); its gear syncs and its
/// meters start again. Returns the hero.
fn wear(b: &mut BattleScreen, item: &str) -> usize {
    let h = b.selected.expect("a hero");
    let w = &mut b.host.session.as_mut().unwrap().w;
    let cat = std::sync::Arc::make_mut(&mut w.cat);
    if !cat.items.contains_key("t_mantle") {
        let skill = |id: &str, name: &str| SkillDef::from_obj(json!({"id": id, "name": name, "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "heal_base": 1, "learned_only": true}).as_object().unwrap());
        cat.add_skills(vec![skill("t_fade", "Fade-Step"), skill("t_reload", "Reload")]);
        let meter = UnitMeterDef::from_obj(json!({"id": AMMO, "name": "Ammo", "short": "Am", "min": 0, "max": AMMO_MAX, "start": 0, "start_full": true, "show": "pips"}).as_object().unwrap());
        cat.unit_meters.insert(meter.id.clone(), meter);
        for row in [
            off_item("t_mantle", "cloak", json!({"name": "Mantle", "family_off_hand_skill": "t_fade"})),
            off_item("t_quiver", "ammo", json!({"name": "Quiver", "family_off_hand_skill": "t_reload", "family_empower_next": {"meter": AMMO, "cost": 1, "power_mult": 1.5}})),
            json!({"id": "t_knife", "name": "Knife", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "knife", "base_power": 8, "family_slot_rule": "1h_light", "family_reach_max": 2}),
        ] {
            let def = ItemDef::from_obj(row.as_object().unwrap());
            cat.items.insert(def.id.clone(), def);
        }
    }
    let cat = w.cat.clone();
    let u = &mut w.units[h];
    u.inventory.unequip(GearSlot::OffHand);
    u.inventory.equip(Equipment::new(&cat.items[item]), GearSlot::OffHand);
    u.sync_from_stats_and_gear(&cat);
    meter::restart_unit(w, h);
    h
}

fn rows(b: &BattleScreen, h: usize, at: RoundTarget) -> Vec<Row> {
    b.menu_rows(h, at).1
}

fn button(rows: &[Row]) -> Option<&Row> {
    rows.iter().find(|r| r.slot == Some(Slot::Neutral(OFF_HAND)))
}

fn picks_skill(r: &Row, skill: usize) -> bool {
    matches!(&r.act, RowAct::Pick(p) if matches!(p[..], [Pick::Skill { skill: k, .. }] if k == skill))
}

fn sid(b: &BattleScreen, id: &str) -> usize {
    s(b).w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The shipped data (GEAR-REWORK1, PR5-51b: the off-hand families carry the hand fields): a hero's menu has the button
/// only when its off hand gives an action (`offhand::action`), on some target when it does, and some hero has one.
#[test]
fn a_hero_has_the_off_hand_button_when_its_off_hand_gives_an_action() {
    let (_cx, _g, b) = arena();
    let f = foe(&b);
    let w = &s(&b).w;
    let mut with = 0;
    for h in w.allies.iter().copied().filter(|&a| w.units[a].is_active() && w.has(a)) {
        let shown: Vec<bool> = [RoundTarget::Myself, RoundTarget::Unit(f)].into_iter().map(|at| button(&rows(&b, h, at)).is_some()).collect();
        if emberwake_battle::offhand::action(w, h).is_some() {
            assert!(shown.contains(&true), "{}: the button on some target", w.units[h].name);
            with += 1;
        } else {
            assert!(!shown.contains(&true), "{}: no off-hand action, no button", w.units[h].name);
        }
    }
    assert!(with > 0, "a shipped hero holds an off-hand item with an action");
}

/// A cloak's Fade-Step is the button (named so, its card naming the item), its cell leaves the intent rows, and the
/// button picks it like a cell.
#[test]
fn a_cloaks_fade_step_is_the_off_hand_button_and_picks_like_a_cell() {
    let (cx, _g, mut b) = arena();
    let h = wear(&mut b, "t_mantle");
    let fade = sid(&b, "t_fade");
    let rs = rows(&b, h, RoundTarget::Myself);
    let r = button(&rs).expect("the Off-Hand button");
    assert_eq!(r.label, "Fade-Step");
    assert!(picks_skill(r, fade), "it picks the cloak's skill");
    assert_eq!(rs.iter().filter(|x| picks_skill(x, fade)).count(), 1, "no second cell of it in an intent row");
    assert_eq!(r.tip.first().map(|t| t.0.as_str()), Some("Off-Hand: Fade-Step"));
    assert!(r.tip.iter().any(|t| t.0.contains("Mantle")), "the card names the item: {:?}", r.tip);
    assert!(r.valid, "live on its own target: {}", r.detail);
    let RowAct::Pick(p) = &r.act else { unreachable!() };
    b.choose_pick(&cx, h, r.at.unwrap_or(RoundTarget::Myself), p, &r.label);
    assert!(rip::picks_of(&s(&b).w, h).iter().any(|x| matches!(x, Pick::Skill { skill, .. } if *skill == fade)), "picked: {}", b.message);
}

/// A quiver's Reload greys with the kit's reason while ammo is left and opens at 0.
#[test]
fn a_quivers_reload_greys_until_the_ammo_is_out() {
    let (_cx, _g, mut b) = arena();
    let h = wear(&mut b, "t_quiver");
    assert_eq!(meter::value(&s(&b).w, h, AMMO), Some(AMMO_MAX), "a full quiver");
    let rs = rows(&b, h, RoundTarget::Myself);
    let r = button(&rs).expect("the Reload button");
    assert_eq!((r.label.as_str(), r.valid, r.detail.as_str()), ("Reload", false, RELOAD_NOT_EMPTY));
    assert!(r.tip.iter().any(|t| t.0.contains(&format!("{AMMO_MAX}/{AMMO_MAX}"))), "the card shows the ammo: {:?}", r.tip);
    let w = &mut b.host.session.as_mut().unwrap().w;
    w.meters.gauges.iter_mut().find(|g| g.unit == h && g.meter == AMMO).expect("the gauge").value = 0;
    let r = button(&rows(&b, h, RoundTarget::Myself)).cloned().expect("the Reload button");
    assert!(r.valid, "open at 0: {}", r.detail);
}

/// A light off-hand knife's button is the catalog's off-hand jab (the kit's `off_hand_jab`) on the foe; the open menu
/// draws it. With `AEGIS_REVIEW_DIR` set the open menu on the foe is written at 1280 x 720 as `battle_offhand.png` there.
#[test]
fn a_light_off_hand_weapons_button_is_the_jab_and_the_menu_draws_it() {
    let (mut cx, mut g, mut b) = arena();
    let h = wear(&mut b, "t_knife");
    let f = foe(&b);
    let jab = emberwake_rules::unit::off_hand_jab(&s(&b).w.cat).map(|j| j.id.clone()).expect("the catalog's off-hand jab");
    let r = button(&rows(&b, h, RoundTarget::Unit(f))).cloned().expect("the jab button");
    assert!(picks_skill(&r, sid(&b, &jab)), "the jab");
    assert_eq!(r.label, s(&b).w.cat.skills[sid(&b, &jab)].name);
    // the open menu on the foe draws the button among the neutral ones
    b.act(&mut cx, Action::UnitClick(f));
    let p = b.popup.clone().expect("the menu on the foe");
    let k = p.rows.iter().position(|r| r.slot == Some(Slot::Neutral(OFF_HAND))).expect("the button in the open menu");
    if let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") {
        let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).expect("font");
        g = Gfx { canvas: aegis_ui::canvas::Canvas::new(1280, 720), view: aegis_ui::painter::View::fit(1280, 720), fonts: aegis_ui::painter::Fonts::new(&font), kit: Default::default() };
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let file = std::fs::File::create(std::path::Path::new(&out).join("battle_offhand.png")).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
    } else {
        b.draw(&mut cx, &mut g, (-500, -500), true);
    }
    let drawn = b.w.iter().find(|x| x.id == format!("popup.{k}")).expect("the button's widget");
    assert!(drawn.text.starts_with(&r.label), "{}", drawn.text);
}

#[test]
fn wpn_orb_exposes_catalyst_pulse_in_off_hand_button() {
    let (mut cx, mut g, mut b) = arena();
    let h = {
        let w = &mut b.host.session.as_mut().unwrap().w;
        let h = w.allies.iter().copied().find(|&u| w.units[u].stats.proficiency.entries.contains_key("orb")).unwrap_or(w.allies[0]);
        let cat = std::sync::Arc::clone(&w.cat);
        let orb = cat.items.get("wpn_orb").expect("shipped wpn_orb");
        let u = &mut w.units[h];
        u.inventory.unequip(GearSlot::OffHand);
        u.inventory.equip(Equipment::new(orb), GearSlot::OffHand);
        u.sync_from_stats_and_gear(&cat);
        meter::restart_unit(w, h);
        h
    };
    b.selected = Some(h);
    let f = foe(&b);
    let pulse = sid(&b, "catalyst_pulse");
    let r = button(&rows(&b, h, RoundTarget::Unit(f))).cloned().expect("Catalyst Pulse button");
    assert!(picks_skill(&r, pulse), "picks catalyst_pulse");
    assert_eq!(r.label, s(&b).w.cat.skills[pulse].name);
    b.act(&mut cx, Action::UnitClick(f));
    let p = b.popup.clone().expect("menu on foe");
    let k = p.rows.iter().position(|row| row.slot == Some(Slot::Neutral(OFF_HAND))).expect("button in menu");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let drawn = b.w.iter().find(|x| x.id == format!("popup.{k}")).expect("button widget");
    assert!(drawn.text.starts_with(&r.label), "{}", drawn.text);
}
