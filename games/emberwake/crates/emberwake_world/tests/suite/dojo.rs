//! The gear dojo's sandbox (emberwake_battle::dojo, DojoSession) on the shipped catalog: expectations come from the
//! battle rules it reuses (damage::compute, skill costs, statuses, gear rules), not from recorded numbers.

use emberwake_battle::dojo::{DojoSession, DUMMY_MAX_HP, HERO_AP, ROUND_AP_REGEN};
use emberwake_rules::damage::{self, HitInput};
use emberwake_rules::gear::{off_hand_ok, wear_lock, Slot};
use emberwake_rules::unit::Stance;
use emberwake_world::Data;
use std::path::Path;

fn session() -> DojoSession {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    DojoSession::new(d.cat.clone())
}

#[test]
fn a_new_dojo_has_the_dojo_heroes_at_battle_ap_a_fresh_dummy_and_the_starter_stash() {
    let s = session();
    assert_eq!(s.heroes.len(), 4);
    assert!(s.heroes.iter().all(|h| h.ap == HERO_AP && h.max_ap == HERO_AP));
    assert_eq!((s.dummy.hp, s.dummy.max_hp, s.dummy.guard), (DUMMY_MAX_HP, DUMMY_MAX_HP, 0.0));
    let starters = s.cat.items.values().filter(|i| i.raw.get("starter_stash").and_then(|v| v.as_bool()) == Some(true)).count();
    assert_eq!(s.stash.len(), starters);
    assert_eq!(s.carry.capacity, emberwake_rules::carry::evaluate(&s.heroes, s.carry.stash_weight).capacity);
}

#[test]
fn a_main_strike_deals_what_the_damage_rules_say_and_force_crit_crits() {
    let mut s = session();
    let hero = s.hero().clone();
    s.strike("main");
    let e = s.log.last().unwrap().clone();
    // The seeded roll may crit: the expectation follows the outcome the log names.
    let expected = damage::compute(&hero, &session().dummy, HitInput { force_crit: e.outcome == "CRIT", ..HitInput::default() }).final_dmg;
    assert_eq!((e.category, e.value), ("ATTACK", expected));
    assert_eq!(s.dummy.hp, DUMMY_MAX_HP - expected);
    assert!(e.breakdown.contains("RESULT:") && e.breakdown.contains(&format!("{expected} damage")), "{}", e.breakdown);
    s.force_crit = true;
    s.strike("main");
    assert_eq!(s.log.last().unwrap().outcome, "CRIT");
}

#[test]
fn a_skill_pays_its_ap_and_stance_switch_and_defend_focus_rest_do_their_part() {
    let mut s = session();
    let attack = s.attack_skills().into_iter().find(|k| k.id != "strike" && emberwake_battle::skill::switch_cost(k, s.hero().stance) > 0).cloned();
    let a = attack.expect("an attack that switches stance");
    {
        let cost = s.skill_ap(&a);
        s.use_skill(Some(&a.id));
        assert_eq!(s.hero().ap, HERO_AP - cost);
        assert_eq!(s.hero().stance, emberwake_battle::skill::stance_after(&a, Stance::Neutral));
    }
    {
        let d = s.usable_skills().into_iter().find(|k| k.kind == "DEFEND" && k.mitigation > 0.0).cloned().expect("a guard skill");
        s.use_skill(Some(&d.id));
        assert_eq!(s.hero().guard, d.mitigation);
    }
    {
        let f = s.usable_skills().into_iter().find(|k| k.kind == "FOCUS").cloned().expect("a focus skill");
        s.use_skill(Some(&f.id));
        assert_eq!((s.hero().attack_boost, s.hero().attack_boost_rounds), (f.boost, f.boost_rounds));
    }
    {
        let r = s.usable_skills().into_iter().find(|k| k.kind == "REST").cloned().expect("a rest skill");
        let before = s.hero().ap;
        s.use_skill(Some(&r.id));
        assert_eq!(s.hero().ap, s.hero().max_ap.min(before + r.recover_ap));
    }
}

#[test]
fn the_dummy_strikes_back_after_a_skill_when_countering() {
    let mut s = session();
    s.dummy_counter = true;
    let hp = s.hero().hp;
    s.use_skill(Some("strike"));
    assert!(s.hero().hp < hp);
    assert!(s.log.last().unwrap().by_dummy);
}

#[test]
fn a_round_regains_ap_drops_the_guard_and_counts_up() {
    let mut s = session();
    s.use_skill(Some("strike"));
    s.heroes[s.active].guard = 0.4;
    let ap = s.hero().ap;
    s.next_round();
    assert_eq!(s.round, 2);
    assert_eq!(s.hero().guard, 0.0);
    assert_eq!(s.hero().ap, (ap + ROUND_AP_REGEN).min(s.hero().max_ap));
}

#[test]
fn stances_cycle_and_the_dummy_guard_toggles() {
    let mut s = session();
    let seen: Vec<Stance> = (0..4).map(|_| {
        s.shift_stance();
        s.hero().stance
    }).collect();
    assert_eq!(seen, [Stance::Offensive, Stance::Tactical, Stance::Defensive, Stance::Neutral]);
    s.toggle_dummy_defense();
    assert_eq!((s.dummy.guard, s.dummy.stance), (0.5, Stance::Defensive));
    s.toggle_dummy_defense();
    assert_eq!((s.dummy.guard, s.dummy.stance), (0.0, Stance::Neutral));
}

#[test]
fn a_benchmark_leaves_the_dummy_alone_and_repeats_with_the_seed() {
    let (mut a, mut b) = (session(), session());
    a.run_benchmark(50);
    b.run_benchmark(50);
    assert_eq!(a.dummy.hp, DUMMY_MAX_HP);
    let (ea, eb) = (a.log.last().unwrap(), b.log.last().unwrap());
    assert_eq!((ea.outcome.as_str(), &ea.detail), ("SIM", &eb.detail));
}

/// Whether `item` sits in the active hero's `slot` (a slot key).
fn wears(s: &DojoSession, slot: &str, item: &str) -> bool {
    emberwake_battle::dojo::SLOTS.iter().find(|(k, _)| *k == slot).and_then(|(_, sl)| s.hero().inventory.get(*sl)).is_some_and(|e| e.id() == item)
}

/// GEAR-REWORK1 (PR5-51b): every shipped class is under the strict gear rule, so a preset piece is worn exactly when
/// the hero may wear it: the hard lock (`gear::wear_lock`: proficiency 0 or raw `required_stats` unmet) refuses it,
/// and an off-hand piece also needs the worn main hand's leave (`gear::off_hand_ok`); a refused piece is named as
/// skipped. `sword_board` (the warrior's preset) gives the warrior its one-handed sword.
#[test]
fn every_preset_names_catalog_items_and_sword_and_board_dresses_the_warrior() {
    let mut s = session();
    // LAYER-DOWN-2: the presets are the catalog's dojo_presets rows
    assert!(!s.presets().is_empty(), "the game ships its presets");
    for p in s.presets() {
        for (slot, item) in &p.gear {
            assert!(s.cat.items.contains_key(item), "{}: {item}", p.id);
            assert!(emberwake_rules::gear::item_fits_slot(&s.cat.items[item], slot), "{}: {item} in {slot}", p.id);
        }
        assert!(p.pouch.iter().all(|i| s.cat.items.contains_key(i)), "{}", p.id);
        for hero in 0..s.heroes.len() {
            s.set_active(hero);
            s.apply_preset(&p.id);
            let main = s.hero().inventory.main_weapon().map(|e| e.item.clone());
            for (slot, item) in &p.gear {
                let def = &s.cat.items[item];
                let mut may = wear_lock(&s.hero().stats, def).is_none();
                if slot == "off_hand" {
                    may &= off_hand_ok(main.as_ref(), def);
                }
                assert_eq!(wears(&s, slot, item), may, "{} on {}: {item} in {slot}", p.id, s.hero().name);
                if !may {
                    assert!(s.log.last().unwrap().breakdown.contains(&def.name), "{}: {item} named as skipped", p.id);
                }
            }
        }
    }
    s.set_active(0);
    assert_eq!(s.cat.classes.get(&s.classes[s.active]).map(|c| c.id.as_str()), Some("warrior"), "the first Dojo hero is the warrior");
    s.apply_preset("sword_board");
    let inv = &s.hero().inventory;
    assert_eq!(inv.main_weapon().map(|e| e.item.id.as_str()), Some("wpn_broadsword"));
    assert_eq!(inv.pouch.len(), 2);
    assert_eq!(s.log.last().unwrap().outcome, "LOADOUT");
}

/// A preset whose one-handed main hand and off-hand piece the hero wears, then a two-handed weapon the hero may wear
/// that leaves no room for that piece (found by rule): the piece goes back to the stash and stays out.
#[test]
fn gear_moves_between_the_stash_and_the_hero_and_follows_the_two_handed_rule() {
    // the preset, found on a scratch Dojo (applying presets makes pieces)
    let mut probe = session();
    let mut chosen = None;
    for p in probe.presets() {
        let Some(off) = p.gear.iter().find(|(slot, _)| slot == "off_hand").map(|(_, i)| i.clone()) else { continue };
        probe.apply_preset(&p.id);
        if wears(&probe, "off_hand", &off) && probe.hero().inventory.main_weapon().is_some_and(|m| !m.item.two_handed) {
            chosen = Some((p.id.clone(), off));
            break;
        }
    }
    let (first, off) = chosen.expect("a preset whose one-handed main and off-hand piece the warrior wears");
    let mut s = session();
    s.clear_gear();
    let total = s.stash.len();
    let pre = s.stash.clone();
    s.apply_preset(&first);
    assert!(wears(&s, "off_hand", &off));
    let off_def = s.cat.items[&off].clone();
    let two = s.cat.items.values().find(|i| i.two_handed && i.item_type == "WEAPON" && wear_lock(&s.hero().stats, i).is_none() && !off_hand_ok(Some(i), &off_def)).map(|i| i.id.clone()).expect("a two-handed weapon the warrior may wear");
    // A two-handed weapon stows the off-hand piece to the stash.
    let mut added = 0;
    if !s.stash.iter().any(|i| *i == two) {
        s.stash.push(two.clone());
        added += 1;
    }
    assert!(s.equip(&two, "main_hand"));
    assert!(s.hero().inventory.get(Slot::OffHand).is_none(), "the off-hand piece went back");
    assert!(s.stash.iter().any(|i| *i == off));
    assert!(!s.equip(&off, "off_hand"), "no {off} under a 2H weapon");
    assert!(!s.eligible("off_hand").iter().any(|i| *i == off));
    s.unequip("main_hand");
    assert!(s.hero().inventory.main_weapon().is_none_or(|m| m.item.id != two));
    s.clear_gear();
    // Everything is back: the starter stash, the preset's made pieces (the skipped ones are dropped), the 2H weapon
    let p = s.presets().into_iter().find(|p| p.id == first).expect("the preset");
    let worn = |slot: &str, i: &str| wear_lock(&s.hero().stats, &s.cat.items[i]).is_none() && (slot != "off_hand" || i == off);
    let made = p.gear.iter().filter(|(slot, i)| worn(slot, i)).map(|(_, i)| i).chain(p.pouch.iter()).filter(|i| !pre.contains(i)).count();
    assert_eq!(s.stash.len(), total + added + made, "nothing is lost or invented");
}

/// GEAR-REWORK1 (PR5-51a, U3.3 1): `Switch Weapon` is retired: the Dojo offers no switch, and a preset with a
/// two-handed main weapon ends with the main weapon worn and its off-hand piece worn only when the weapon's family
/// pairs with it (`gear::off_hand_ok`: a `2h_rig` bow and its ammo), the main hand live. Found by rule over the Dojo's
/// heroes (the first that may wear the weapon), not by id.
#[test]
fn there_is_no_weapon_switch_and_a_two_handed_preset_leaves_the_off_hand_empty() {
    let mut s = session();
    assert!(!s.availability().contains_key("switch"), "no switch action");
    let mut seen = 0;
    for p in s.presets() {
        let Some(main) = p.gear.iter().find(|(slot, i)| slot == "main_hand" && s.cat.items[i].two_handed).map(|(_, i)| s.cat.items[i].clone()) else { continue };
        let Some(hero) = (0..s.heroes.len()).find(|h| wear_lock(&s.heroes[*h].stats, &main).is_none()) else { continue };
        s.set_active(hero);
        s.apply_preset(&p.id);
        assert_eq!(s.hero().inventory.main_weapon().map(|m| m.id().to_string()), Some(main.id.clone()), "{}: the 2H weapon is worn", p.id);
        match p.gear.iter().find(|(slot, _)| slot == "off_hand").map(|(_, i)| s.cat.items[i].clone()) {
            Some(off) => {
                let may = off_hand_ok(Some(&main), &off) && wear_lock(&s.hero().stats, &off).is_none();
                assert_eq!(wears(&s, "off_hand", &off.id), may, "{}: {} beside the 2H weapon only when it pairs", p.id, off.id);
            }
            None => assert!(s.hero().inventory.get(Slot::OffHand).is_none(), "{}: the off hand stays empty", p.id),
        }
        s.strike(if s.hero().inventory.main_weapon().unwrap().is_ranged_weapon() { "ranged" } else { "main" });
        assert_ne!(s.log.last().unwrap().outcome, "FAILED", "{}: the main hand strikes, nothing to switch first", p.id);
        seen += 1;
    }
    assert!(seen > 0, "a two-handed preset a Dojo hero may wear");
}

/// GEAR-REWORK1 (PR5-51b, U3.3 1): every shipped class is strict, so a preset piece the hero has no training for
/// (proficiency 0) is not worn: the hard lock skips it and the log names it (U3.2-S3's soft wield is gone for them).
#[test]
fn a_preset_piece_the_hero_has_no_training_for_is_skipped() {
    let mut s = session();
    let mut seen = 0;
    for hero in 0..s.heroes.len() {
        s.set_active(hero);
        assert!(s.hero().stats.proficiency.is_strict(), "{}: a strict class", s.hero().name);
        for preset in s.presets() {
            let untrained: Vec<String> = preset.gear.iter().filter(|(_, i)| !s.trained_for(&s.cat.items[i])).map(|(_, i)| i.clone()).collect();
            if untrained.is_empty() {
                continue;
            }
            s.apply_preset(&preset.id);
            for item in &untrained {
                assert!(s.hero().inventory.gear().all(|e| e.id() != item.as_str()), "{item} is not worn untrained");
                assert!(s.log.last().unwrap().breakdown.contains(&s.cat.items[item].name), "{item} is named as skipped");
                seen += 1;
            }
        }
    }
    assert!(seen > 0, "a preset holds a piece a Dojo hero has no training for");
}

#[test]
fn the_pouch_loads_from_the_stash_heals_and_bombs() {
    let mut s = session();
    s.clear_gear();
    for id in ["con_heal_potion", "con_fire_bomb"] {
        if !s.stash.iter().any(|i| i == id) {
            s.stash.push(id.into());
        }
        assert!(s.add_to_pouch(id));
    }
    s.heroes[s.active].hp -= 30;
    let hp = s.hero().hp;
    s.use_pouch(0);
    assert!(s.hero().hp > hp);
    s.use_pouch(0);
    assert!(s.dummy.hp < DUMMY_MAX_HP);
    assert!(s.hero().inventory.pouch.is_empty());
}

#[test]
fn attribute_points_are_spent_and_confirmed_or_cancelled() {
    let mut s = session();
    if s.hero().stats.unallocated == 0 {
        s.begin_allocation();
        assert!(s.alloc.is_none() && s.log.last().unwrap().outcome == "NONE", "nothing to spend");
    }
    let a = s.active;
    s.heroes[a].stats.unallocated = 3;
    let free = 3;
    s.begin_allocation();
    let str_before = s.hero().stats.base("STR");
    s.allocate("STR");
    s.allocate("STR");
    s.deallocate("STR");
    assert_eq!(s.alloc.as_ref().unwrap().free, free - 1);
    s.confirm_allocation();
    assert_eq!(s.hero().stats.base("STR"), str_before + 1);
    assert_eq!(s.hero().stats.unallocated, free - 1);
}
