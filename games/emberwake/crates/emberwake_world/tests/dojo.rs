//! The gear dojo's sandbox (emberwake_battle::dojo, DojoSession) on the shipped catalog: expectations come from the
//! battle rules it reuses (damage::compute, skill costs, statuses, gear rules), not from recorded numbers.

use emberwake_battle::dojo::{DojoSession, DUMMY_MAX_HP, HERO_AP, PRESETS, ROUND_AP_REGEN};
use emberwake_rules::damage::{self, HitInput};
use emberwake_rules::gear::Slot;
use emberwake_rules::unit::Stance;
use emberwake_world::Data;
use std::path::Path;

fn session() -> DojoSession {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    DojoSession::new(d.cat.clone())
}

#[test]
fn a_new_dojo_has_two_heroes_at_battle_ap_a_fresh_dummy_and_the_starter_stash() {
    let s = session();
    assert_eq!(s.heroes.len(), 2);
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

#[test]
fn every_preset_names_catalog_items_and_sword_and_board_dresses_the_warrior() {
    let mut s = session();
    for p in PRESETS {
        for (slot, item) in p.gear {
            assert!(s.cat.items.contains_key(*item), "{}: {item}", p.id);
            assert!(emberwake_rules::gear::item_fits_slot(&s.cat.items[*item], slot), "{}: {item} in {slot}", p.id);
        }
        assert!(p.pouch.iter().all(|i| s.cat.items.contains_key(*i)), "{}", p.id);
    }
    s.apply_preset("sword_board");
    let inv = &s.hero().inventory;
    assert_eq!(inv.main_weapon().map(|e| e.item.id.as_str()), Some("wpn_broadsword"));
    assert_eq!(inv.get(Slot::OffHand).map(|e| e.item.id.as_str()), Some("shd_buckler"));
    assert_eq!(inv.pouch.len(), 2);
    assert_eq!(s.log.last().unwrap().outcome, "LOADOUT");
}

#[test]
fn gear_moves_between_the_stash_and_the_hero_and_follows_the_two_handed_rule() {
    let mut s = session();
    s.clear_gear();
    let total = s.stash.len();
    let pre = s.stash.clone();
    s.apply_preset("sword_board");
    assert!(s.hero().inventory.get(Slot::OffHand).is_some());
    // A two-handed weapon stows the shield to the stash.
    let mut added = 0;
    if !s.stash.iter().any(|i| i == "wpn_claymore") {
        s.stash.push("wpn_claymore".into());
        added += 1;
    }
    assert!(s.equip("wpn_claymore", "main_hand"));
    assert!(s.hero().inventory.get(Slot::OffHand).is_none(), "the shield went back");
    assert!(s.stash.iter().any(|i| i == "shd_buckler"));
    assert!(!s.equip("shd_buckler", "off_hand"), "no shield under a 2H weapon");
    assert!(!s.eligible("off_hand").iter().any(|i| i == "shd_buckler"));
    s.unequip("main_hand");
    assert!(s.hero().inventory.main_weapon().is_none_or(|m| m.item.id != "wpn_claymore"));
    s.clear_gear();
    // Everything is back: the starter stash, the preset's made pieces, the claymore the test added.
    let made = PRESETS[0].gear.iter().map(|(_, i)| *i).chain(PRESETS[0].pouch.iter().copied()).filter(|i| !pre.iter().any(|x| x == i)).count();
    assert_eq!(s.stash.len(), total + added + made, "nothing is lost or invented");
}

#[test]
fn the_ready_weapon_returns_to_the_main_hand_when_the_pair_changes() {
    let mut s = session();
    s.apply_preset("claymore_backup");
    s.switch_weapon();
    assert!(s.hero().inventory.is_offhand_active());
    s.apply_preset("claymore_backup");
    assert!(!s.hero().inventory.is_offhand_active(), "a fresh pair starts on the main hand");
    s.switch_weapon();
    s.unequip("off_hand");
    assert!(!s.hero().inventory.off_active, "no pair: the main hand is ready (_normalize_active_weapon)");
}

/// U3.2-S3 (FB-34): a preset piece the hero has no training for is worn (soft wield), under the wield penalty.
#[test]
fn a_preset_piece_the_hero_has_no_training_for_is_worn_untrained() {
    let mut s = session();
    let preset = PRESETS.iter().find(|p| p.gear.iter().any(|(_, i)| !s.trained_for(&s.cat.items[*i]))).expect("a preset with a piece the warrior has no training for");
    s.apply_preset(preset.id);
    for (_, item) in preset.gear.iter().filter(|(_, i)| !s.trained_for(&s.cat.items[*i])) {
        let worn = s.hero().inventory.gear().find(|e| e.id() == *item).unwrap_or_else(|| panic!("{item} is worn"));
        assert!(worn.wield_penalty(&s.hero().stats).untrained, "{item} is worn untrained");
    }
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
fn a_2h_weapon_with_an_off_hand_weapon_switches_and_gates_the_strikes() {
    let mut s = session();
    s.apply_preset("claymore_backup");
    assert!(s.hero().inventory.needs_weapon_switch(), "claymore + shortsword needs the switch");
    {
        assert!(s.availability()["switch"].enabled);
        assert!(!s.availability()["offhand"].enabled, "the off hand waits for the switch");
        s.switch_weapon();
        assert!(s.hero().inventory.is_offhand_active());
        let hp = s.dummy.hp;
        s.strike("main");
        assert_eq!(s.dummy.hp, hp, "the main hand is sheathed");
        assert_eq!(s.log.last().unwrap().outcome, "FAILED");
    }
    s.apply_preset("sword_board");
    s.switch_weapon();
    assert_eq!(s.log.last().unwrap().outcome, "FAILED", "nothing to switch with a 1H pair");
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
