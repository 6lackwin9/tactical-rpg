//! SKILLUI1-L: the rank chosen per family on the Skills tab (games/emberwake/docs/specs/SKILLUI1.md sections 2, 3 and
//! 5, with the user's answers of 2026-10-20). Expectations come from the rules R1-R8 and V1-V2, on the shipped catalog:
//! a priest member and its Mend family (three ranks: mend, mend_2, mend_3).

use emberwake_battle::session::EncounterDef;
use emberwake_battle::plan::PlanStep;
use emberwake_battle::session::{Prebuilt, Session};
use emberwake_battle::skill::Kind;
use emberwake_battle::world::AiProfile;
use emberwake_world::battle::WorldRoster;
use emberwake_world::progression;
use emberwake_world::skill_text;
use emberwake_world::state::UnitState;
use emberwake_world::Data;
use std::path::Path;

const FAM: &str = "mend";
const TOP_LEVEL: i64 = 12;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// A priest member at `level` with Mend slotted (once the priest owns it) and `ranks` chosen.
fn priest(d: &Data, level: i64, ranks: &[(&str, i64)]) -> UnitState {
    let mut m = UnitState::at_level(d, "priest", level, "alda");
    if m.owned_families(d).iter().any(|f| f == FAM) {
        m.loadout_class = vec![FAM.into()];
    }
    m.loadout_ranks = ranks.iter().map(|(f, r)| (f.to_string(), *r)).collect();
    m
}

fn built(d: &Data, m: &UnitState) -> emberwake_rules::Unit {
    progression::build_with(d, m, false).expect("a valid member").unit
}

fn usable(d: &Data, u: &emberwake_rules::Unit, fam: &str) -> Vec<i32> {
    d.cat.family_rows(fam).into_iter().filter(|s| u.can_use(&d.cat, s)).map(|s| s.tier).collect()
}

fn top(d: &Data) -> i32 {
    built(d, &priest(d, TOP_LEVEL, &[])).selectable_ranks(&d.cat, FAM).into_iter().max().unwrap()
}

fn sid(d: &Data, id: &str) -> usize {
    d.cat.skills.iter().position(|s| s.id == id).unwrap_or_else(|| panic!("no skill {id}"))
}

/// A battle with the member as the only party unit (GameEncounter's path: a Prebuilt hero).
fn battle(d: &Data, m: &UnitState) -> (Session, usize) {
    let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", "courtyard_skirmish"));
    let roster = WorldRoster { d, world: "ashlands".into() };
    let base = d.cat.units.get("priest").unwrap().clone();
    let hero = Prebuilt { unit: built(d, m), art: String::new(), profile: AiProfile::for_unit(&d.cat, &base), key: m.id.clone() };
    let s = Session::start_with(d.cat.clone(), def, &roster, vec![hero]);
    let u = s.w.allies[0];
    (s, u)
}

#[test]
fn the_mend_family_has_three_ranks_and_a_level_12_priest_has_them_all() {
    let d = data();
    assert_eq!(d.cat.family_rows(FAM).iter().map(|s| s.tier).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(top(&d), 3);
}

#[test]
fn catalog_unit_keeps_every_unlocked_tier() {
    let d = data();
    let mut def = d.cat.units.get("priest").unwrap().clone();
    def.level = TOP_LEVEL as i32;
    let u = emberwake_rules::unit::build_unit(&d.cat, &def, 0);
    assert!(u.rank_choice.is_none(), "R7: the rule is off for catalog units");
    assert_eq!(usable(&d, &u, FAM), u.selectable_ranks(&d.cat, FAM));
    assert_eq!(usable(&d, &u, FAM).len(), 3);
}

#[test]
fn member_without_ranks_uses_the_top_rank() {
    let d = data();
    let u = built(&d, &priest(&d, TOP_LEVEL, &[]));
    assert_eq!(u.effective_rank(&d.cat, FAM), Some(top(&d)));
    assert_eq!(usable(&d, &u, FAM), vec![top(&d)], "R2, R4: one usable rank, the highest unlocked");
}

#[test]
fn chosen_rank_is_the_only_usable_tier() {
    let d = data();
    let u = built(&d, &priest(&d, TOP_LEVEL, &[(FAM, 1)]));
    assert_eq!(usable(&d, &u, FAM), vec![1]);
    let (s, hero) = battle(&d, &priest(&d, TOP_LEVEL, &[(FAM, 1)]));
    let menu: Vec<String> = s.w.book().available(Kind::Support, s.w.skill_cap_of(hero), Some(&s.w.units[hero])).into_iter().map(|i| d.cat.skills[i].id.clone()).collect();
    assert!(menu.contains(&"mend".to_string()) && !menu.iter().any(|m| m == "mend_2" || m == "mend_3"), "the battle menu lists Mend once: {menu:?}");
}

#[test]
fn planner_refuses_another_rank() {
    let d = data();
    let (mut s, hero) = battle(&d, &priest(&d, TOP_LEVEL, &[(FAM, 1)]));
    let problem = s.plan.add_step(&s.w, hero, PlanStep::ally(sid(&d, "mend_2"), hero), true);
    assert_eq!(problem, format!("{} uses Mend at Rank I (set on the Skills tab).", s.w.units[hero].name));
    assert_eq!(s.plan.add_step(&s.w, hero, PlanStep::ally(sid(&d, "mend"), hero), true), "", "the chosen rank plans");
}

#[test]
fn pin_above_unlocked_falls_to_highest_below() {
    let d = data();
    // a level where Mend II is open and Mend III is not
    let level = (1..TOP_LEVEL).find(|&l| built(&d, &priest(&d, l, &[])).selectable_ranks(&d.cat, FAM) == vec![1, 2]).expect("a level with ranks 1-2");
    let u = built(&d, &priest(&d, level, &[(FAM, 3)]));
    assert_eq!(u.effective_rank(&d.cat, FAM), Some(2));
}

#[test]
fn learned_row_selects_only_its_tier() {
    let d = data();
    // a hand-built unit that knows only mend_2 (no class path): the family's selectable ranks are that row alone
    let mut def = d.cat.units.get("priest").unwrap().clone();
    def.class_id.clear();
    def.loadout = vec!["mend_2".into()];
    let mut u = emberwake_rules::unit::build_unit(&d.cat, &def, 0);
    u.known_skills = vec!["mend_2".into()];
    u.path_levels.clear();
    assert_eq!(u.selectable_ranks(&d.cat, FAM), vec![2]);
}

#[test]
fn basics_ignore_the_rank_rule() {
    let d = data();
    let u = built(&d, &priest(&d, TOP_LEVEL, &[(FAM, 1)]));
    let strike = d.cat.skill("strike").unwrap();
    assert!(strike.basic && u.rank_allowed(&d.cat, strike) && u.can_use(&d.cat, strike));
}

#[test]
fn family_learned_mid_battle_uses_its_top_rank() {
    let d = data();
    let mut u = built(&d, &priest(&d, TOP_LEVEL, &[(FAM, 1)]));
    let fam = "hex";
    u.learn_skill(&d.cat, fam);
    assert!(!u.rank_choice.as_ref().unwrap().contains_key(fam));
    let ranks = u.selectable_ranks(&d.cat, fam);
    assert!(!ranks.is_empty(), "the learned family is usable");
    assert_eq!(u.effective_rank(&d.cat, fam), ranks.last().copied(), "R2: no entry, the highest rank");
}

#[test]
fn slot_keeps_its_rank_on_level_up() {
    let d = data();
    // the user's answer to Q2: a slot never moves up by itself; an entry stays where it was set
    let mut m = priest(&d, 3, &[]);
    progression::pin_ranks(&d, &mut m);
    let pinned = m.loadout_ranks.get(FAM).copied().unwrap();
    m.total_xp = progression::xp_for_level(&d, TOP_LEVEL);
    assert_eq!(built(&d, &m).effective_rank(&d.cat, FAM), Some(pinned as i32), "still at the rank it was slotted with");
    assert!(top(&d) > pinned as i32);
}

#[test]
fn slotting_stores_the_highest_unlocked_rank() {
    let d = data();
    let mut m = priest(&d, TOP_LEVEL, &[]);
    progression::pin_ranks(&d, &mut m);
    assert_eq!(m.loadout_ranks.get(FAM).copied(), Some(top(&d) as i64));
    // an existing entry is kept
    let mut m = priest(&d, TOP_LEVEL, &[(FAM, 1)]);
    progression::pin_ranks(&d, &mut m);
    assert_eq!(m.loadout_ranks.get(FAM).copied(), Some(1));
}

#[test]
fn rank_entry_for_unowned_family_is_an_error() {
    let d = data();
    let m = priest(&d, TOP_LEVEL, &[("volley", 1)]);
    assert!(m.validate(&d).iter().any(|e| e.contains("rank set for family 'volley' that is not owned")), "{:?}", m.validate(&d));
    let m = priest(&d, TOP_LEVEL, &[("strike", 1)]);
    assert!(m.validate(&d).iter().any(|e| e.contains("is a basic family (basics have one rank)")), "{:?}", m.validate(&d));
}

#[test]
fn rank_without_a_tier_row_is_an_error() {
    let d = data();
    let m = priest(&d, TOP_LEVEL, &[(FAM, 7)]);
    assert!(m.validate(&d).iter().any(|e| e.contains("family 'mend' has no rank 7")), "{:?}", m.validate(&d));
}

#[test]
fn ranks_round_trip_through_a_save() {
    let d = data();
    let m = priest(&d, TOP_LEVEL, &[(FAM, 2)]);
    let v = m.to_value();
    assert_eq!(v["loadout"]["ranks"]["mend"], serde_json::json!(2));
    let back = UnitState::from_dict(v.as_object().unwrap());
    assert_eq!(back, m);
}

#[test]
fn member_without_ranks_saves_no_ranks_key() {
    let d = data();
    let v = priest(&d, TOP_LEVEL, &[]).to_value();
    assert!(v["loadout"].get("ranks").is_none(), "a member without ranks saves exactly as before: {}", v["loadout"]);
}

#[test]
fn version_10_save_loads_unchanged() {
    let d = data();
    let gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 7).expect("a new game");
    let member = gs.party.members[0].clone();
    let env = serde_json::json!({"format": emberwake_world::save::FORMAT, "version": 10, "saved_at": 0, "game_state": gs.to_value()});
    let (out, warnings) = emberwake_world::save::migrate(&d, env.as_object().unwrap()).expect("a version 10 save migrates");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(out["version"], serde_json::json!(emberwake_world::save::VERSION));
    let back = emberwake_world::state::GameState::from_dict(&d, out["game_state"].as_object().unwrap());
    assert_eq!(back.party.members[0], member, "the no-op step keeps the member as it was");
    assert!(member.loadout_ranks.is_empty());
}

#[test]
fn hero_ai_uses_only_chosen_ranks() {
    let d = data();
    let (mut s, hero) = battle(&d, &priest(&d, TOP_LEVEL, &[(FAM, 2)]));
    s.w.units[hero].hp = 1;
    s.plan_apply_preset(hero, "balanced");
    let used: Vec<String> = s.plan.steps_of(hero).iter().filter_map(|st| st.skill.map(|i| d.cat.skills[i].id.clone())).collect();
    assert!(!used.iter().any(|id| id == "mend" || id == "mend_3"), "only Mend II may be planned: {used:?}");
    assert!(used.iter().any(|id| id == "mend_2"), "a hero at 1 HP heals with the chosen rank: {used:?}");
}

#[test]
fn pin_below_every_selectable_rank_uses_the_lowest() {
    let d = data();
    // R2's last branch: an entry below every selectable rank falls to the lowest one
    let mut def = d.cat.units.get("priest").unwrap().clone();
    def.class_id.clear();
    let mut u = emberwake_rules::unit::build_unit(&d.cat, &def, 0);
    u.known_skills = vec!["mend_2".into(), "mend_3".into()];
    u.path_levels.clear();
    u.rank_choice = Some([(FAM.to_string(), 1)].into_iter().collect());
    assert_eq!(u.selectable_ranks(&d.cat, FAM), vec![2, 3]);
    assert_eq!(u.effective_rank(&d.cat, FAM), Some(2));
}

#[test]
fn empty_stored_loadout_still_uses_only_the_chosen_rank() {
    let d = data();
    // with no stored loadout every owned family is active, and the rank rule still holds (R3)
    let mut m = priest(&d, TOP_LEVEL, &[(FAM, 1)]);
    m.loadout_class.clear();
    let u = built(&d, &m);
    assert!(u.active_families().iter().any(|f| f == FAM) || u.active_families().is_empty());
    assert_eq!(usable(&d, &u, FAM), vec![1]);
}

#[test]
fn confirming_the_default_loadout_pins_its_families() {
    let d = data();
    // the class default loadout is stored empty; pinning still covers every family the hero takes into battle
    let mut m = priest(&d, TOP_LEVEL, &[]);
    m.loadout_class.clear();
    progression::pin_ranks(&d, &mut m);
    let u = built(&d, &m);
    assert!(!u.active_families().is_empty());
    for fam in u.active_families() {
        assert_eq!(m.loadout_ranks.get(&fam).copied(), u.selectable_ranks(&d.cat, &fam).last().map(|&r| r as i64), "{fam}");
    }
}

#[test]
fn ranks_read_as_roman_numerals() {
    use emberwake_rules::unit::rank_numeral;
    assert_eq!([1, 2, 3, 4, 5].map(rank_numeral), ["I", "II", "III", "IV", "V"]);
    assert_eq!((rank_numeral(0), rank_numeral(9)), ("-", "V+"));
}

#[test]
fn rank_lines_list_only_changing_fields() {
    let d = data();
    let u = built(&d, &priest(&d, TOP_LEVEL, &[]));
    let lines = skill_text::rank_lines(&d.cat, &u, FAM);
    assert_eq!(lines.iter().map(|l| l.tier).collect::<Vec<_>>(), vec![1, 2, 3]);
    let rows = d.cat.family_rows(FAM);
    let reach_same = rows.iter().all(|s| (s.reach_min, s.reach_max) == (rows[0].reach_min, rows[0].reach_max));
    for l in &lines {
        assert!(l.text.contains("heal "), "the heal changes between ranks: {}", l.text);
        assert_eq!(l.text.contains("reach "), !reach_same, "reach shows only when it changes: {}", l.text);
        assert_eq!(l.ap, emberwake_battle::skill::ap_cost(rows[(l.tier - 1) as usize], Some(&u)));
    }
    // the first level that owns Mend: rank I open, rank III still locked
    let first = (1..TOP_LEVEL).find(|&l| priest(&d, l, &[]).owned_families(&d).iter().any(|f| f == FAM)).expect("the priest learns Mend");
    let low = built(&d, &priest(&d, first, &[]));
    let locked = skill_text::rank_lines(&d.cat, &low, FAM);
    let l3 = locked.iter().find(|l| l.tier == 3).unwrap();
    assert!(!l3.unlocked && l3.text == format!("unlocks at Lv {}", l3.unlock_level), "{l3:?}");
}
