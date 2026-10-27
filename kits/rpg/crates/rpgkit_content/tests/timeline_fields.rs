//! Timeline fields and their checks (RULESET1-S, docs/specs/RULESET1_TASKS.md 1.3-1.6): the schema and the code name
//! the same fields, kinds and options; the fields load into the typed catalog with defaults that change nothing; each
//! rule broken alone is one error naming its field (a DEFEND skill set to assault, channel no ruleset lists, an order
//! entry of another intent, ...); a Maneuver attack without dash tiles is one warning; ranked timeline fields fold and
//! render through the writer. Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use rpgkit_rules::ruleset::{self, INTENTS, NEUTRAL};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_timeline_fields_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// The deck of `drillman` that passes every rule (two actives it owns, one passive and one keystone of their slots, an
/// order list per intent from its actives and the ruleset's basics).
fn good_deck() -> Value {
    json!({"active": ["thrust", "brace"], "passive": ["p_slot"], "keystone": ["k_slot"], "order": {"assault": ["thrust", "strike"], "guard": ["brace"], "maneuver": ["flank_cut"]}})
}

/// Fixture files: skills of every kind (flank_cut a Maneuver basic of the ruleset), a class whose path opens jab at
/// level 3, passives of each slot, the derived Max Poise, a timeline row with the deck rule (2 actives, 1 passive,
/// 1 keystone), a level 1 unit with that deck, and a reach family.
fn catalog(data: &Path) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(
        data,
        "skills",
        2,
        json!([
            one("move", "MOVE", json!({"system": true})),
            one("rest", "REST", json!({"system": true})),
            one("strike", "ATTACK", json!({"basic": true})),
            one("thrust", "ATTACK", json!({"blocks": 2})),
            one("jab", "ATTACK", json!({})),
            one("brace", "DEFEND", json!({"grant_status": "riposte"})),
            one("mend", "SUPPORT", json!({"heal_base": 5})),
            one("focus", "FOCUS", json!({})),
            one("leap", "DISENGAGE", json!({"jump_tiles": 2})),
            one("flank_cut", "ATTACK", json!({"round_intent": "maneuver", "dash_tiles": 2}))
        ]),
    );
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1},
                "proficiency": {"sword": {"level": 1}, "polearm": {"level": 1}}, "skill_path": {"thrust": 1, "jab": 3}}]),
    );
    let stat = json!([{"type": "stat", "stat": "STR", "value": 1}]);
    write_kind(
        data,
        "passives",
        1,
        json!([{"id": "p_slot", "name": "P", "effects": stat, "slot": "passive"}, {"id": "k_slot", "name": "K", "effects": stat, "slot": "keystone"}, {"id": "plain", "name": "Plain", "effects": stat}]),
    );
    write_kind(data, "derived_stats", 1, json!([{"id": "max_poise", "name": "Max Poise", "formula": "CONS * 2"}]));
    write_kind(
        data,
        ruleset::KIND,
        1,
        json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck", "poise"], "basics": ["move", "rest", "strike", "flank_cut"], "poise_stat": "max_poise",
                "deck_slots": {"active": 2, "passive": 1, "keystone": 1}}]),
    );
    write_kind(data, "units", 1, json!([{"id": "drillman", "name": "Drillman", "side": "enemy", "class_id": "drill", "level": 1, "extra_skills": ["brace", "mend"], "deck": good_deck()}]));
    write_kind(
        data,
        "item_families",
        1,
        json!([{"id": "polearm", "name": "Polearm", "item_type": "WEAPON", "hands": ["main_2h"], "default_hand": "main_2h", "scaling_stats": ["STR"],
                "base": {"main_2h": {"power": 26, "range": 1, "block_cost": 1, "burden": 2, "mitigation": 0}}, "engine_rules": ["reach"]}]),
    );
}

/// The load problems of the catalog files (the scratch folder has no progression file, which the loader reports).
fn catalog_errors(src: &source::Source) -> Vec<String> {
    src.errors.iter().filter(|e| e.contains("catalog")).cloned().collect()
}

fn field_of(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("content.{kind}.{key}")).clone()
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// The timeline skill fields with the kinds RULESET1_TASKS 1.3 gives them (round_intent: every kind) and whether
/// they are per rank.
const SKILL_FIELDS: [(&str, &[&str], bool); 9] = [
    ("round_intent", &[], false),
    ("intent_buff", &["DEFEND", "FOCUS", "SUPPORT"], false),
    ("ap_cost", &["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"], true),
    ("recovery_blocks", &["ATTACK", "FOCUS", "SUPPORT"], true),
    ("stagger", &["ATTACK"], true),
    ("windup_poise", &["ATTACK", "FOCUS", "SUPPORT"], true),
    ("unbreakable", &["ATTACK", "FOCUS", "SUPPORT"], false),
    ("sure_break", &["ATTACK"], false),
    ("armor_piercing", &["ATTACK"], false),
];

#[test]
fn the_schema_and_the_checks_name_the_same_fields_kinds_and_options() {
    for (key, kinds, rank) in SKILL_FIELDS {
        let f = field_of("skills", key);
        assert_eq!(json::b(&f, "rank", false), rank, "{key}: per rank");
        let users: Vec<&str> = skill::KINDS.iter().copied().filter(|k| skill::has_field(k, key)).collect();
        if kinds.is_empty() {
            assert_eq!(users, skill::KINDS.to_vec(), "{key}: every kind");
            assert!(!f.contains_key("when"), "{key}: shown for every kind");
        } else {
            let mut want: Vec<&str> = kinds.to_vec();
            want.sort_by_key(|k| skill::KINDS.iter().position(|x| x == k));
            assert_eq!(users, want, "{key}: the kinds that use it");
            let shown: Vec<String> = json::aget(json::dget(&f, "when"), "in").iter().map(gstr).collect();
            let mut shown_sorted = shown.clone();
            shown_sorted.sort_by_key(|k| skill::KINDS.iter().position(|x| x == k));
            assert_eq!(shown_sorted, strs(&want), "{key}: the schema's `when` names the same kinds");
        }
    }
    let mut intents = vec![String::new()];
    intents.extend(strs(&INTENTS));
    intents.push(NEUTRAL.to_string());
    assert_eq!(json::aget(&field_of("skills", "round_intent"), "options").iter().map(gstr).collect::<Vec<_>>(), intents, "\"\", the ruleset intents, neutral");
    assert_eq!(json::aget(&field_of("passives", "slot"), "options").iter().map(gstr).collect::<Vec<_>>(), strs(&["", "passive", "keystone"]));
    let weapon_types: Vec<String> = json::aget(&field_of("items", "weapon_type"), "options").iter().map(gstr).collect();
    for g in skill::OPTIONAL_GEAR_TYPES {
        assert!(skill::GEAR_TYPES.contains(&g) && weapon_types.contains(&g.to_string()), "{g}: a gear type and a weapon type option");
    }
    assert!(json::aget(&field_of("item_families", "engine_rules"), "options").iter().any(|o| o == "reach"));
    // every IN4 intent a kind allows is a schema option, and each authored kind's default is among its allowed ones
    for kind in skill::KINDS {
        for i in ruleset::kind_intents(kind) {
            assert!(intents.contains(&i.to_string()), "{kind}: {i}");
        }
        assert!(ruleset::kind_intents(kind).contains(&ruleset::kind_intent(kind)), "{kind}: its default intent is allowed");
    }
}

#[test]
fn the_fields_load_into_the_typed_catalog_and_default_to_nothing() {
    let data = scratch("load");
    catalog(&data);
    let src = source::load(&data);
    assert_eq!(catalog_errors(&src), Vec::<String>::new());
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let plain = cat.skill("thrust").unwrap();
    assert_eq!(
        (plain.round_intent.as_str(), plain.intent_buff, plain.ap_cost, plain.recovery_blocks, plain.stagger, plain.windup_poise, plain.unbreakable, plain.sure_break, plain.armor_piercing),
        ("", false, -1, 0, 0, 0, false, false, false),
        "the schema defaults: unset"
    );
    assert_eq!(cat.skill("flank_cut").unwrap().round_intent, "maneuver");
    let u = &cat.units["drillman"];
    assert_eq!(u.deck, rpgkit_data::Deck::from_value(&good_deck()));
    assert_eq!((u.deck.active.len(), u.deck.order.len(), u.poise), (2, 3, -1));
    assert_eq!(u.default_intent, "", "no default intent unless set");
    assert!(rpgkit_data::Deck::from_value(&json!({})).is_empty());
    assert_eq!((cat.passives["p_slot"].slot.as_str(), cat.passives["plain"].slot.as_str()), ("passive", ""));
    // the bounds are the schema's (normalizing clamps, as for every kind)
    let row = normalize(fields(source::schema(), "content", "skills"), &dict(json!({"id": "x", "ap_cost": 15, "recovery_blocks": -3, "stagger": 400})), false);
    assert_eq!((json::i(&row, "ap_cost", 0), json::i(&row, "recovery_blocks", 9), json::i(&row, "stagger", 0)), (10, 0, 100));
    let unit = normalize(fields(source::schema(), "content", "units"), &dict(json!({"id": "x", "poise": 500})), false);
    assert_eq!(json::i(&unit, "poise", 0), 200);
    drop_scratch(&data);
}

/// One entry checked as it would be written: (kind, entry id, field changes, expected message part or "" = clean).
type Case = (&'static str, &'static str, Value, &'static str);

fn errors_of(v: &check::View, kind: &str, id: &str, change: &Value) -> Vec<String> {
    let mut e = v.get(kind, id).clone();
    assert!(!e.is_empty(), "fixture {kind} {id}");
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, kind, &e, id)
}

#[test]
fn each_broken_rule_is_one_error_naming_its_field() {
    let data = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let cases: Vec<Case> = vec![
        // the fixtures themselves, and what the rules allow
        ("skills", "thrust", json!({}), ""),
        ("skills", "move", json!({}), ""),
        ("skills", "brace", json!({"intent_buff": true}), ""),
        ("skills", "focus", json!({"intent_buff": true}), ""),
        ("skills", "mend", json!({"round_intent": "guard", "target_mode": "SELF"}), ""),
        ("skills", "thrust", json!({"exploit": [{"status": "flanked", "power_mult": 1.3}]}), ""),
        ("skills", "thrust", json!({"recovery_blocks": 2, "blocks": 4, "stagger": 12, "sure_break": true, "ap_cost": 3, "armor_piercing": true}), ""),
        ("units", "drillman", json!({}), ""),
        ("units", "drillman", json!({"deck": {}}), ""),
        ("units", "drillman", json!({"default_intent": "guard"}), ""),
        ("units", "drillman", json!({"default_intent": "channel"}), "Default intent: 'channel' is not one of , assault, guard, support, maneuver."),
        ("passives", "plain", json!({"effects": [{"type": "stagger_taken", "mult": 0.0, "max_blocks": 1}, {"type": "foresight", "range": 4}]}), ""),
        ("item_families", "polearm", json!({}), ""),
        // skills (1.3)
        ("skills", "brace", json!({"round_intent": "assault"}), "Round intent: DEFEND skills take guard (not assault)."),
        ("skills", "move", json!({"round_intent": "assault"}), "Round intent: MOVE skills take neutral (not assault)."),
        ("skills", "thrust", json!({"round_intent": "channel"}), "Round intent: channel needs a ruleset whose intents list it."),
        ("skills", "thrust", json!({"round_intent": "bogus"}), "Round intent: 'bogus' is not one of"),
        ("skills", "mend", json!({"round_intent": "guard"}), "Round intent guard on a SUPPORT skill needs Targets SELF"),
        ("skills", "mend", json!({"intent_buff": true}), "Intent-bound buff needs a granted status"),
        ("skills", "leap", json!({"intent_buff": true}), "Intent-bound buff belongs to DEFEND/FOCUS/SUPPORT skills, not DISENGAGE."),
        ("skills", "thrust", json!({"blocks": 5, "recovery_blocks": 2}), "Blocks plus recovery blocks is 7 (at most 6"),
        ("skills", "thrust", json!({"sure_break": true}), "Sure break needs a stagger of at least 1"),
        ("skills", "brace", json!({"stagger": 10}), "Stagger belongs to ATTACK skills, not DEFEND."),
        ("skills", "leap", json!({"recovery_blocks": 1}), "Recovery blocks belongs to ATTACK/FOCUS/SUPPORT skills, not DISENGAGE."),
        ("skills", "thrust", json!({"exploit": [{"status": "flanked", "power_mult": 1.3}, {"status": "flanked", "power_mult": 1.2}]}), "Exploit lists 'flanked' twice."),
        // passives (1.4)
        ("passives", "plain", json!({"slot": "relic"}), "Deck slot: 'relic' is not one of , passive, keystone."),
        ("passives", "plain", json!({"effects": [{"type": "stagger_taken", "mult": 2.5}]}), "Effect 1: stagger taken needs a mult 0.0-2.0"),
        ("passives", "plain", json!({"effects": [{"type": "stagger_taken", "mult": 0.5, "max_blocks": 7}]}), "Effect 1: max_blocks must be a whole number 0-6"),
        ("passives", "plain", json!({"effects": [{"type": "foresight", "range": 9}]}), "Effect 1: foresight range must be a whole number 0-8"),
        // units (1.5)
        ("units", "drillman", json!({"deck": {"active": ["jab"]}}), "Deck active: the unit does not own the skill family 'jab'."),
        ("units", "drillman", json!({"deck": {"active": ["strike"]}}), "Deck active: 'strike' is a basic"),
        ("units", "drillman", json!({"deck": {"active": ["flank_cut"]}}), "Deck active: 'flank_cut' is a basic"),
        ("units", "drillman", json!({"deck": {"active": ["nothing"]}}), "Deck active: unknown skill family 'nothing'."),
        ("units", "drillman", json!({"deck": {"active": ["thrust", "brace", "mend"]}}), "Deck: 3 active entries (the Timeline ruleset's deck slots allow 2)."),
        ("units", "drillman", json!({"deck": {"active": ["thrust", "thrust"]}}), "Deck active lists 'thrust' twice."),
        ("units", "drillman", json!({"deck": {"passive": ["k_slot"]}}), "Deck passive: 'k_slot' has the slot keystone, not the slot passive."),
        ("units", "drillman", json!({"deck": {"keystone": ["plain"]}}), "Deck keystone: 'plain' has no slot (granted by its source, always on), not the slot keystone."),
        ("units", "drillman", json!({"deck": {"active": ["thrust", "brace"], "order": {"assault": ["brace"]}}}), "Deck order assault: 'brace' is of intent guard."),
        ("units", "drillman", json!({"deck": {"active": ["thrust"], "order": {"support": ["mend"]}}}), "Deck order support: 'mend' is neither an active of the deck nor a basic."),
        ("units", "drillman", json!({"deck": {"order": {"dance": ["strike"]}}}), "Deck order: 'dance' is not an intent"),
        ("units", "drillman", json!({"deck": {"order": {"assault": ["strike", "strike"]}}}), "Deck order lists 'strike' twice."),
        ("units", "drillman", json!({"deck": {"relic": []}}), "Deck: 'relic' is not one of active, passive, keystone, order."),
        ("units", "drillman", json!({"deck": []}), "Deck must be an object"),
        // item families (1.6)
        ("item_families", "polearm", json!({"engine_rules": ["reach", "always_ranged"]}), "reach is for melee families; it can't go with always_ranged."),
        ("item_families", "polearm", json!({"base": {"main_2h": {"power": 26, "range": 2, "block_cost": 1, "burden": 2, "mitigation": 0}}}), "reach needs base range 1 on every hand (main_2h has 2)."),
    ];
    let mut failures = Vec::new();
    for (kind, id, change, want) in &cases {
        let errors = errors_of(&v, kind, id, change);
        let ok = if want.is_empty() { errors.is_empty() } else { errors.len() == 1 && errors[0].contains(want) };
        if !ok {
            failures.push(format!("{kind} {id} {change}: want {want:?}, got {errors:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    drop_scratch(&data);
}

#[test]
fn channel_is_taken_once_a_ruleset_lists_it() {
    let data = scratch("channel");
    catalog(&data);
    write_kind(
        &data,
        ruleset::KIND,
        1,
        json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent"], "intents": ["assault", "guard", "support", "maneuver", "channel"]}]),
    );
    let v = store::view(&data);
    assert_eq!(errors_of(&v, "skills", "thrust", &json!({"round_intent": "channel"})), Vec::<String>::new());
    assert_eq!(errors_of(&v, "skills", "rest", &json!({"round_intent": "channel"})), ["Round intent: REST skills are neutral; channel is for authored skills."]);
    drop_scratch(&data);
}

#[test]
fn a_maneuver_attack_without_dash_tiles_and_an_unbreakable_windup_with_poise_warn() {
    let data = scratch("warnings");
    catalog(&data);
    let v = store::view(&data);
    let warnings = |id: &str, change: Value| {
        let mut e = v.get("skills", id).clone();
        for (k, x) in change.as_object().unwrap() {
            e.insert(k.clone(), x.clone());
        }
        check::warnings_for(&v, "skills", &e)
    };
    assert_eq!(warnings("flank_cut", json!({})), Vec::<String>::new(), "a carried move of 2 tiles");
    assert_eq!(warnings("flank_cut", json!({"dash_tiles": 0})), ["A Maneuver attack with dash tiles 0 carries no move; its range is its reach band."]);
    assert_eq!(warnings("thrust", json!({"dash_tiles": 0})), Vec::<String>::new(), "an Assault attack needs no carried move");
    assert_eq!(warnings("thrust", json!({"unbreakable": true})), Vec::<String>::new());
    assert_eq!(warnings("thrust", json!({"unbreakable": true, "windup_poise": 20})), ["Unbreakable with windup poise: the poise bonus does nothing while the unit cannot break."]);
    // a ranked entry: the rank whose modifier drops the dash
    let ranked = dict(json!({"id": "hook", "name": "Hook", "kind": "ATTACK", "round_intent": "maneuver", "dash_tiles": 2, "ranks": [{"dash_tiles": 0}]}));
    assert_eq!(check::warnings_for(&v, "skills", &ranked), ["Rank 2: a Maneuver attack with dash tiles 0 carries no move; its range is its reach band."]);
    drop_scratch(&data);
}

/// The writer stores the timeline fields as every skill field (family fields once, rank fields as rank modifiers when
/// they differ), the loader expands them per rank, the tokens render each rank's numbers, and a re-save is identical.
#[test]
fn ranked_timeline_fields_fold_expand_and_render() {
    let data = scratch("writer");
    catalog(&data);
    let entry = json!({"id": "cleave", "name": "Cleave", "kind": "ATTACK", "blocks": 3, "round_intent": "assault", "ap_cost": 3, "recovery_blocks": 1, "stagger": 15,
        "windup_poise": 10, "unbreakable": true, "description": "{ap}, {stagger} stagger, then {recovery} ({ap:n}/{stagger:n}/{recovery:n}).",
        "ranks": [{"ap_cost": 4, "stagger": 20}, {"recovery_blocks": 2, "windup_poise": 0}]});
    store::upsert(&data, None, "skills", &dict(entry), None).expect("a valid ranked skill");
    let src = source::load(&data);
    assert_eq!(catalog_errors(&src), Vec::<String>::new());
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let ranks: Vec<&rpgkit_data::SkillDef> = cat.family_rows("cleave");
    assert_eq!(ranks.len(), 3);
    // each rank: rank 1's value unless its modifier sets one (modifiers do not carry over to later ranks)
    let want = [(3, 1, 15, 10), (4, 1, 20, 10), (3, 2, 15, 0)];
    for (s, (ap, rec, stg, wp)) in ranks.iter().zip(want) {
        assert_eq!((s.ap_cost, s.recovery_blocks, s.stagger, s.windup_poise, s.unbreakable, s.round_intent.as_str()), (ap, rec, stg, wp, true, "assault"), "{}", s.id);
        let blocks = if rec == 1 { "1 block".to_string() } else { format!("{rec} blocks") };
        assert_eq!(json::s(&s.raw, "description"), format!("{ap} AP, {stg} stagger, then {blocks} ({ap}/{stg}/{rec})."), "{}", s.id);
    }
    let stored: Value = serde_json::from_str(&fs::read_to_string(store::file_of(&data, None, "skills")).unwrap()).unwrap();
    let cleave = stored["entries"].as_array().unwrap().iter().find(|e| e["id"] == "cleave").unwrap();
    assert_eq!(cleave["ranks"], json!([{"ap_cost": 4, "stagger": 20}, {"recovery_blocks": 2, "windup_poise": 0}]), "rank modifiers keep only what differs from rank 1");
    assert_eq!((cleave["unbreakable"].clone(), cleave["round_intent"].clone()), (json!(true), json!("assault")));
    let thrust = stored["entries"].as_array().unwrap().iter().find(|e| e["id"] == "thrust").unwrap();
    for (key, ..) in SKILL_FIELDS {
        assert!(thrust.get(key).is_none(), "an unset {key} is not written");
    }
    let text = fs::read_to_string(store::file_of(&data, None, "skills")).unwrap();
    assert_eq!(store::resave_text(&data, None, "skills").unwrap(), text, "a re-save without changes is byte-identical");
    drop_scratch(&data);
}

/// TQ23 (a): a unit's default intent is one a ruleset lists (the fixture row lists the four, this one two).
#[test]
fn a_default_intent_no_ruleset_lists_is_refused() {
    let data = scratch("default_intent");
    catalog(&data);
    write_kind(&data, ruleset::KIND, 1, json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent"], "intents": ["assault", "guard"]}]));
    let v = store::view(&data);
    assert_eq!(errors_of(&v, "units", "drillman", &json!({"deck": {}, "default_intent": "guard"})), Vec::<String>::new());
    assert_eq!(
        errors_of(&v, "units", "drillman", &json!({"deck": {}, "default_intent": "maneuver"})),
        ["Default intent: no ruleset lists 'maneuver' among its intents (timeline: assault, guard)."]
    );
    drop_scratch(&data);
}

/// Planner pick (RULESET1-S): an intent-bound buff is no start buff (no unit has a round intent at deployment); the
/// other new buff is.
#[test]
fn an_intent_bound_buff_is_no_start_buff() {
    let start = |st: &str| check::encounter::start_status_problems(&json!([{"status": st, "text": "A line."}]));
    assert_eq!(start("steadfast"), Vec::<String>::new());
    assert_eq!(start("stealth"), ["Start buff 'stealth' must be a buff other than taunt or an intent-bound one (riposte, stealth)."]);
    assert_eq!(start("riposte"), ["Start buff 'riposte' must be a buff other than taunt or riposte."], "the Godot text, unchanged");
}
