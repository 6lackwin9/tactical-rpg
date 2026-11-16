//! PR5-62c (`TECHNIQUES-GRAFTS-DATA`) integration tests on the shipped `world:sunk_ward/*` data:
//! 1. All 15 main-hand weapon families carry 3 Mastery skills (`techniques`), 1 Cold Steel Keystone (`keystone`), and
//!    every main-hand weapon binds `weapon.technique`, with Mastery 1/2/3 (`100/300/600` XP via Berek's Hearth drill /
//!    `mastery::gain`), Masterwork `4th` (`hearth_reforge`), and Forge Rite (`hearth_rite`) opening them in the build.
//! 2. All 18 `PK1`/`PK2` grafts (6 Monster Ossuary pins, 6 Class Cranial Rituals, 6 Class Inoculated Keystones), 30
//!    Sabina Slab surgeries (`sg_*`), Strain floors `5/15/30/50`, Channel-round gating (`marrow_lash` Channel-only vs
//!    Cold Steel technique Standard-only), and Cold Steel vs Inoculated keystone mutual exclusion.
//! 3. `purge_marrow` Stage V calm rescue in `world:sunk_ward/enc_evt_06`: callable in a Standard round (`channel.none`),
//!    calms a Stage V `hostile_all` ally back to Stage IV without aborting the fight or departing the member at settle.
//! 4. Retired `c02a` skill-grant passives (`mastery_crossbow`, `mastery_bow`, `mastery_fist`, `dirty_tricks`) carry pure
//!    passive bonuses with no free M1 skill leakage.

use aegis_core::json::{self, Dict};
use emberwake_battle::mark;
use emberwake_battle::round_intent::{Pick as RoundPick, RoundTarget};
use emberwake_battle::round_intent_plan;
use emberwake_battle::session::Phase;
use emberwake_battle::{channel, meter, plan};
use emberwake_data::mastery::MASTERY_LEVEL_XP;
use emberwake_rules::damage::{self, base_armor_rating, HitInput};
use emberwake_rules::unit::skill_is_ranged;
use emberwake_rules::Unit;
use emberwake_world::{battle, mastery, new_game, recipe, roster, weapons, Data, GameState};
use recipe::Pick;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

const WORLD: &str = "sunk_ward";
const SABINA: &str = "sunk_ward:sabina_kress";
const BEREK: &str = "sunk_ward:berek_dorn";
const CRUCIBLE: &str = "sunk_ward:enc_evt_06";

const MAIN_HAND_FAMILIES: [&str; 15] = [
    "polearm", "greatsword", "sword", "hammer", "gauntlet", "fist", "crossbow", "bow",
    "knife", "dagger", "censer", "bone_saw", "scourge", "sprayer", "staff",
];

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("shipped data").0
}

/// A `sunk_ward` game advanced to Act 2 with the Hearth, Slab, Apex, Masterwork, and class rites open, plus the
/// Prologue/Tier 1-2 founders (`kaelen`, `vesper`, `maren`, `hollis`) active in the party.
fn act2_game(d: &Data) -> GameState {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("new sunk_ward game");
    for recruit in ["vesper", "maren", "hollis"] {
        let mut m = new_game::hero_from_character(d, WORLD, recruit).unwrap_or_else(|| panic!("recruit {recruit} missing"));
        m.id = recruit.into();
        if !gs.party.members.iter().any(|existing| existing.id == recruit) {
            gs.party.members.push(m);
        }
    }
    gs.party.active_ids = gs.party.members.iter().take(4).map(|m| m.id.clone()).collect();
    gs.party.member_mut("kaelen").expect("kaelen").equipment.insert("main_hand".into(), "notched_billhook".into());
    for m in &mut gs.party.members {
        m.set_class_rite(true);
    }
    gs.campaign.as_mut().expect("sunk_ward is a campaign world").act = 2;
    gs.party.gold = 2000;
    for flag in [
        "pk2.slab_open", "pk2.hearth_open", "pk2.apex_open", "pk2.masterwork_quest",
        "pk2.rite_warrior", "pk2.rite_brawler", "pk2.rite_ranger", "pk2.rite_rogue",
        "pk2.rite_priest", "pk2.rite_mage",
    ] {
        gs.set_flag(flag, Value::Bool(true));
    }
    for (item, count) in [
        ("black_iron_ingot", 10),
        ("stalker_optic_nerve", 6),
        ("sinew_braid", 4),
        ("ossified_bellows", 4),
        ("caustic_gall_sac", 4),
        ("inquisitor_marrow_valve", 4),
        ("titan_marrow_core", 6),
        ("slot_bone_spur", 6),
        ("slot_chitin_plate", 6),
        ("slot_blood_sac", 6),
        ("slot_eye_tissue", 6),
    ] {
        gs.party.add_to_stash(item, count);
    }
    weapons::sync(d, &mut gs);
    gs
}

fn built(d: &Data, gs: &GameState, member_id: &str) -> Unit {
    let m = gs.party.member(member_id).unwrap_or_else(|| panic!("member {member_id}"));
    roster::field_unit(d, gs, m, WORLD).unwrap_or_else(|| panic!("field_unit {member_id}")).unit
}

fn run_ok(d: &Data, gs: &mut GameState, npc: &str, recipe_id: &str, pick: &Pick) {
    let res = recipe::run(d, gs, npc, recipe_id, pick);
    assert!(json::b(&res, "ok", false), "{npc} {recipe_id}: {res:?}");
}

fn run_err(d: &Data, gs: &mut GameState, npc: &str, recipe_id: &str, pick: &Pick) -> String {
    let before = gs.to_value();
    let res = recipe::run(d, gs, npc, recipe_id, pick);
    assert!(!json::b(&res, "ok", true), "{npc} {recipe_id} should be refused");
    assert_eq!(gs.to_value(), before, "{npc} {recipe_id} refusal leaves state unchanged");
    json::sget(&res, "errors").join(" ")
}

#[test]
fn main_hand_families_mastery_masterwork_and_forge_rite_work_in_sunk_ward() {
    let d = data();
    assert!(d.cat.skills.iter().all(|s| !s.id.starts_with("tq_")), "no retired tq_* skills remain");

    // Verify all 15 main-hand weapon families and all main-hand weapons.
    let mut seen_skills = BTreeSet::new();
    for fam in MAIN_HAND_FAMILIES {
        let mh_items: Vec<_> = d.cat.items.values().filter(|it| it.item_type == "WEAPON" && it.slot == "MAIN_HAND" && it.weapon_type == fam).collect();
        assert!(!mh_items.is_empty(), "{fam} has main-hand weapons");
        let sample = mh_items[0];
        assert_eq!(sample.family_techniques.len(), 3, "{fam} ships 3 Mastery techniques");
        assert!(!sample.family_keystone.is_empty(), "{fam} ships a Cold Steel keystone");
        for t in &sample.family_techniques {
            let sk = d.cat.skill(&t.skill).unwrap_or_else(|| panic!("{fam} missing skill {}", t.skill));
            assert!(sk.learned_only && d.cat.family_rows(&sk.family_id()).len() == 1 && sk.weapon_types == [fam], "{fam} tech {}", t.skill);
            assert!(seen_skills.insert(t.skill.clone()), "duplicate technique {}", t.skill);
        }
        let ks = d.cat.skill(&sample.family_keystone).unwrap_or_else(|| panic!("{fam} missing keystone {}", sample.family_keystone));
        assert!(ks.learned_only && d.cat.family_rows(&ks.family_id()).len() == 1 && ks.weapon_types == [fam], "{fam} keystone {}", ks.id);
        assert!(seen_skills.insert(ks.id.clone()), "duplicate keystone {}", ks.id);
        for it in &mh_items {
            let mw = it.technique.as_ref().unwrap_or_else(|| panic!("MH weapon {} missing masterwork technique", it.id));
            let sk = d.cat.skill(&mw.skill).unwrap_or_else(|| panic!("{} missing MW skill {}", it.id, mw.skill));
            assert!(sk.learned_only && d.cat.family_rows(&sk.family_id()).len() == 1 && sk.weapon_types == [fam], "{} MW skill {}", it.id, mw.skill);
        }
    }
    assert_eq!(seen_skills.len(), 60, "15 * (3 mastery + 1 keystone) unique skills");

    // Kaelen wields notched_billhook (polearm, proficiency 3, x1.5 rate) in sunk_ward.
    let mut gs = act2_game(&d);
    let billhook = weapons::all(&gs.party).into_iter().find(|w| w.item == "notched_billhook").expect("Kaelen's billhook instance").uid.clone();
    let kaelen_pick = Pick { member: "kaelen".into(), family: "polearm".into(), ..Pick::default() };

    assert!(built(&d, &gs, "kaelen").techniques.is_empty(), "Mastery 0 holds no techniques");
    assert!(run_err(&d, &mut gs, BEREK, "hearth_rite", &kaelen_pick).contains("needs mastery 3"), "forge rite requires mastery 3");

    // Mastery 1 / 2 / 3 unlocks via Berek drills + combat mastery::gain -> 100 / 300 / 600 XP (rate x1.5 on Prof 3).
    run_ok(&d, &mut gs, BEREK, "drill_family", &kaelen_pick);
    assert!(gs.party.member("kaelen").unwrap().mastery_xp["polearm"] > 0, "drill awards mastery XP in Act 2");
    mastery::gain(&d, &mut gs, "kaelen", "polearm", MASTERY_LEVEL_XP[0]).expect("reach mastery 1");
    assert_eq!(built(&d, &gs, "kaelen").techniques.iter().map(|t| t.skill.as_str()).collect::<Vec<_>>(), ["hook_drag"]);
    mastery::gain(&d, &mut gs, "kaelen", "polearm", MASTERY_LEVEL_XP[0]).expect("reach mastery 2");
    assert_eq!(built(&d, &gs, "kaelen").techniques.iter().map(|t| t.skill.as_str()).collect::<Vec<_>>(), ["hook_drag", "pike_skew"]);
    mastery::gain(&d, &mut gs, "kaelen", "polearm", MASTERY_LEVEL_XP[1]).expect("reach mastery 3");
    assert_eq!(built(&d, &gs, "kaelen").techniques.iter().map(|t| t.skill.as_str()).collect::<Vec<_>>(), ["hook_drag", "pike_skew", "check_advance"]);

    // Masterwork 4th technique via Berek's hearth_reforge.
    run_ok(&d, &mut gs, BEREK, "hearth_reforge", &Pick { weapon: billhook.clone(), ..Pick::default() });
    assert!(weapons::find(&gs.party, &billhook).unwrap().masterwork);
    assert_eq!(
        built(&d, &gs, "kaelen").techniques.iter().map(|t| t.skill.as_str()).collect::<Vec<_>>(),
        ["hook_drag", "pike_skew", "check_advance", "halberd_sweep"]
    );

    // Forge Rite via Berek's hearth_rite opens polearm Cold Steel Keystone (`polearm_cordon`).
    run_ok(&d, &mut gs, BEREK, "hearth_rite", &Pick { member: "kaelen".into(), ..Pick::default() });
    let u = built(&d, &gs, "kaelen");
    assert_eq!(u.mastery_keystone, "polearm_cordon");
    assert!(u.keystone_skills.contains(&"polearm_cordon".to_string()));
}

#[test]
fn sabina_grafts_channel_rounds_and_keystone_exclusion_work_in_sunk_ward() {
    let d = data();
    let mut gs = act2_game(&d);

    // 30 Sabina graft surgeries exist in catalog (18 monster + 6 class rituals + 6 Inoculated keystones).
    let sabina_sg: Vec<_> = d.cat.recipes.values().filter(|r| r.npc == "sabina_kress" && r.id.starts_with("sg_")).collect();
    assert_eq!(sabina_sg.len(), 30, "18 monster + 6 class ritual + 6 Inoculated keystone surgeries");

    // 3 monster grafts raise Strain floor 0 -> 5 -> 15 -> 30 and unlock Tier III Cranial keystone.
    let kaelen = Pick { member: "kaelen".into(), ..Pick::default() };
    gs.party.member_mut("kaelen").unwrap().set_class_rite(false);
    assert!(run_err(&d, &mut gs, SABINA, "sg_ks_war_cranial", &kaelen).contains("class rite"), "cranial opens with class rite");
    gs.party.member_mut("kaelen").unwrap().set_class_rite(true);
    assert!(run_err(&d, &mut gs, SABINA, "sg_ks_war_cranial", &kaelen).contains("needs 3 grafts"), "Tier III keystone needs 3 grafts first");
    run_ok(&d, &mut gs, SABINA, "sg_optic_forearms", &kaelen);
    run_ok(&d, &mut gs, SABINA, "sg_sinew_thoracic", &kaelen);
    run_ok(&d, &mut gs, SABINA, "sg_bellows_visceral", &kaelen);
    assert_eq!(gs.party.member("kaelen").unwrap().body.len(), 3);

    // Give Kaelen Mastery 3 + Forge Rite too so we prove Inoculated Keystone preempts Cold Steel Keystone.
    mastery::gain(&d, &mut gs, "kaelen", "polearm", MASTERY_LEVEL_XP[2]).unwrap();
    run_ok(&d, &mut gs, BEREK, "hearth_rite", &kaelen);
    assert_eq!(built(&d, &gs, "kaelen").mastery_keystone, "polearm_cordon", "Cold Steel keystone active before cranial keystone");

    run_ok(&d, &mut gs, SABINA, "sg_ks_war_cranial", &kaelen);
    let m_after = gs.party.member("kaelen").unwrap();
    assert_eq!(m_after.body.len(), 4);
    assert_eq!(m_after.track_stage("calcification"), 1, "Tier III surgery adds +1 Calcification");
    let u = built(&d, &gs, "kaelen");
    assert_eq!(u.keystone_skills, ["ks_iron_marrow"], "Inoculated keystone takes the single keystone slot");
    assert!(u.mastery_keystone.is_empty(), "Cold Steel keystone excluded while Inoculated keystone is installed");
    assert_eq!(u.passive_meters.iter().find(|pm| pm.meter == "strain").and_then(|pm| pm.floor), Some(50), "4 grafts set Strain floor 50");

    // Battle check in world:sunk_ward/enc_evt_06: Channel round opens marrow_lash & closes Cold Steel hook_drag.
    let fight: Dict = json::dict(&json!({"source": "story", "encounter": CRUCIBLE})).clone();
    let mut f = battle::begin_world(&d, &mut gs, &fight).expect("Crucible starts");
    let k_idx = f.session.w.allies[0];
    let lash = d.cat.skill("marrow_lash").expect("marrow_lash");
    let hook = d.cat.skill("hook_drag").expect("hook_drag");
    assert_eq!(meter::value(&f.session.w, k_idx, "strain"), Some(50));
    assert!(channel::can_channel(&f.session.w, k_idx));
    assert_eq!(channel::problem(&f.session.w, k_idx, lash), Some(channel::ONLY_CHANNEL));
    assert_eq!(channel::problem(&f.session.w, k_idx, hook), None);
    let (res, _) = round_intent_plan::set_channel(&mut f.session.w, &mut f.session.plan, k_idx, true);
    assert!(res.is_ok(), "Channel toggles on: {res:?}");
    assert_eq!(channel::problem(&f.session.w, k_idx, lash), None, "marrow_lash opens in Channel round");
    assert_eq!(channel::problem(&f.session.w, k_idx, hook), Some(channel::CHANNEL_ROUND), "weapon technique closes in Channel round");
}

#[test]
fn purge_marrow_calms_stage_v_hostile_all_ally_in_sunk_ward_crucible() {
    let d = data();
    let mut gs = act2_game(&d);
    // Give Maren (priest) 3 monster grafts + Tier III Inoculated Keystone `graft_purge` (`purge_marrow`), and set
    // Kaelen at Calcification Stage IV (pre-Stage V threshold).
    let maren_pick = Pick { member: "maren".into(), ..Pick::default() };
    run_ok(&d, &mut gs, SABINA, "sg_optic_forearms", &maren_pick);
    run_ok(&d, &mut gs, SABINA, "sg_sinew_thoracic", &maren_pick);
    run_ok(&d, &mut gs, SABINA, "sg_bellows_visceral", &maren_pick);
    run_ok(&d, &mut gs, SABINA, "sg_purge_cranial", &maren_pick);
    gs.party.member_mut("kaelen").unwrap().tracks.insert("calcification".into(), 4);

    let fight: Dict = json::dict(&json!({"source": "story", "encounter": CRUCIBLE})).clone();
    let mut f = battle::begin_world(&d, &mut gs, &fight).expect("Crucible starts");
    let kaelen_u = f.session.w.allies[0];
    let maren_u = f.members.iter().position(|(id, _)| id == "maren").expect("Maren fielded");

    // Verify `purge_marrow` uses `channel: {none: true}` and opens in a Standard round.
    let purge_def = d.cat.skill("purge_marrow").expect("purge_marrow");
    assert!(purge_def.channel.none, "purge_marrow is channel.none");
    assert_eq!(channel::problem(&f.session.w, maren_u, purge_def), None, "purge_marrow opens in Standard round");

    // Push Kaelen into Stage V (`hostile_all`) via a Breach `calcify` mark.
    f.session.w.marks.record(kaelen_u, "calcify", 1);
    let parts = mark::sync_tracks(&mut f.session.w, kaelen_u, "calcify");
    assert!(parts.iter().any(|p| p.contains("turns on everyone")), "Stage V turns hostile_all: {parts:?}");
    assert!(!f.session.w.allies.contains(&kaelen_u), "Kaelen left allies");
    assert!(!f.session.combat_over, "fight continues rather than aborting");

    // Maren casts `purge_marrow` on turned Kaelen in a Standard round (make sure Maren is within range 0..=2).
    let kaelen_pos = f.session.w.grid.position(kaelen_u);
    let adj = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .map(|(dx, dy)| emberwake_battle::grid::V2::new(kaelen_pos.x + dx, kaelen_pos.y + dy))
        .find(|&p| {
            f.session.w.grid.is_valid(p)
                && f.session.w.grid.is_walkable(p)
                && !f.session.w.grid.is_exit(p)
                && (0..f.session.w.units.len()).all(|u| f.session.w.grid.pos(u) != Some(p))
        })
        .expect("adjacent walkable cell beside Kaelen");
    f.session.w.remove(maren_u);
    assert!(f.session.w.place(maren_u, adj));

    let holder = f.session.w.allies[0];
    f.session.stay_pick(holder).expect("non-turned ally holds the winch");
    let purge_id = f.session.w.book().find("purge_marrow").expect("purge_marrow in book");
    let (picked, _) = f.session.pick(maren_u, RoundTarget::Unit(kaelen_u), &[RoundPick::Skill { skill: purge_id, kind: None }]);
    assert!(picked.is_ok(), "purge_marrow pick on Stage V turned ally accepted under round_intent: {picked:?}");
    f.session.plan_apply_preset_all("balanced");
    assert!(f.session.confirm_plan());
    while f.session.phase != Phase::Replay && !f.session.combat_over {
        f.session.step_next();
    }
    assert!(f.session.resolver.log.iter().any(|l| l.contains("calms at the next round start")), "{:?}", f.session.resolver.log);
    assert!(f.session.can_advance_round());
    f.session.advance_round();
    assert!(f.session.w.allies.contains(&kaelen_u), "Kaelen returns to party allies after calming");

    let tally = f.tally();
    let res = battle::finish_world(&d, &mut gs, "story", &fight, "won", Some(&tally));
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.party.member("kaelen").map(|m| m.track_stage("calcification")), Some(4), "Kaelen rescued back at Stage IV without departing");
}

#[test]
fn legacy_mastery_passives_no_longer_leak_weapon_techniques() {
    let d = data();
    for pid in ["mastery_crossbow", "mastery_bow", "mastery_fist", "dirty_tricks"] {
        let p = d.cat.passives.get(pid).unwrap_or_else(|| panic!("passive {pid} missing"));
        let grants = emberwake_rules::grants::read(&p.effects);
        assert!(
            grants.iter().all(|g| !matches!(g, emberwake_rules::grants::Grant::GrantSkill(_))),
            "{pid} must not grant a skill: {:?}",
            p.effects
        );
    }
}

#[test]
fn calcification_stage_ii_and_iii_passives_apply_in_crucible() {
    let d = data();
    let track = d.cat.unit_tracks.get("calcification").expect("calcification track");
    assert_eq!(track.stages[1].passive, "calcify_2");
    assert_eq!(track.stages[2].passive, "calcify_3");
    assert_eq!(d.cat.passives["calcify_2"].slot, "");
    assert_eq!(d.cat.passives["calcify_3"].slot, "");

    let mut gs = act2_game(&d);
    let base = built(&d, &gs, "hollis");
    let (base_armor, base_move) = (base_armor_rating(&base), base.move_per_block());

    // Stage II (`calcify_2`): +2 Armor Rating, -1 Move Rate (min 1).
    gs.party.member_mut("hollis").unwrap().tracks.insert("calcification".into(), 2);
    let stage2 = built(&d, &gs, "hollis");
    assert!(stage2.passives.contains(&"calcify_1".into()) && stage2.passives.contains(&"calcify_2".into()));
    assert_eq!(base_armor_rating(&stage2), base_armor + 2);
    assert_eq!(stage2.move_per_block(), (base_move - 1).max(1));

    // Stage III (`calcify_3`): cumulative Stage II bonuses + weaken/stun resist 100 + -15% ranged beyond 3 tiles.
    gs.party.member_mut("hollis").unwrap().tracks.insert("calcification".into(), 3);
    let stage3 = built(&d, &gs, "hollis");
    assert_eq!(base_armor_rating(&stage3), base_armor + 2);
    assert_eq!(stage3.move_per_block(), (base_move - 1).max(1));
    assert_eq!(stage3.status_resist.get("weaken").copied(), Some(100));
    assert_eq!(stage3.status_resist.get("stun").copied(), Some(100));

    let fight: Dict = json::dict(&json!({"source": "story", "encounter": CRUCIBLE})).clone();
    let mut f = battle::begin_world(&d, &mut gs, &fight).expect("Crucible starts");
    let hollis_u = f.members.iter().position(|(id, _)| id == "hollis").expect("Hollis fielded");
    let foe_u = f.session.w.enemies[0];
    assert_eq!(base_armor_rating(&f.session.w.units[hollis_u]), base_armor + 2);
    assert_eq!(f.session.w.units[hollis_u].move_per_block(), (base_move - 1).max(1));
    assert_eq!(f.session.w.units[hollis_u].status_resist.get("weaken").copied(), Some(100));
    assert_eq!(f.session.w.units[hollis_u].status_resist.get("stun").copied(), Some(100));

    let ranged_sid = (0..f.session.w.cat.skills.len())
        .find(|&sid| {
            let s = &f.session.w.cat.skills[sid];
            s.kind == "ATTACK" && s.power_mult > 0.0 && skill_is_ranged(s)
        })
        .expect("a damaging ranged skill");

    // Distance 3 stays full damage; distance 4 triggers `min_distance: 4` (-15%) on both `damage::compute` and `plan::expected_with`.
    f.session.w.grid.positions[hollis_u] = Some(emberwake_battle::grid::V2::new(2, 4));
    f.session.w.grid.positions[foe_u] = Some(emberwake_battle::grid::V2::new(5, 4)); // dist = 3
    let est_d3 = plan::expected_with(&f.session.w, hollis_u, foe_u, ranged_sid, None, 0.0, 1.0, false);
    let raw_d3 = damage::compute(&f.session.w.units[hollis_u], &f.session.w.units[foe_u], HitInput { ranged: true, distance: 3, ..HitInput::default() }).final_dmg;

    f.session.w.grid.positions[foe_u] = Some(emberwake_battle::grid::V2::new(6, 4)); // dist = 4
    let est_d4 = plan::expected_with(&f.session.w, hollis_u, foe_u, ranged_sid, None, 0.0, 1.0, false);
    let raw_d4 = damage::compute(&f.session.w.units[hollis_u], &f.session.w.units[foe_u], HitInput { ranged: true, distance: 4, ..HitInput::default() }).final_dmg;

    assert!(est_d4 < est_d3, "expected damage at dist 4 ({est_d4}) < dist 3 ({est_d3})");
    assert!(raw_d4 < raw_d3, "raw ranged hit at dist 4 ({raw_d4}) < dist 3 ({raw_d3})");
}
