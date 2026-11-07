//! Combat hook schema and checks (HOOKS1-S, docs/specs/HOOKS1_TASKS.md 1.1-1.6 and HOOKS1.md 4.1, 8.6): the schema and
//! the code name the same fields, kinds, options and rule ids; the fields load into the typed catalog and default to
//! nothing (a row without them checks as before); the hook kinds load and drop a bad row with its reason; each rule
//! broken alone is one error naming its field; a placement and an HP cost warn; an arena hazard in a classic world
//! warns; the writer keeps the fields. Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use rpgkit_data::{HAZARDS_KIND, UNIT_METERS_KIND, ZONES_KIND};
use rpgkit_rules::ruleset::{self, RuleKind, Ruleset, REGISTER};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_hooks_schema_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn write_world(data: &Path, w: &str, manifest: Value, kinds: Value) {
    let dir = data.join("worlds").join(w);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), manifest.to_string()).unwrap();
    for (kind, entries) in kinds.as_object().unwrap() {
        fs::write(dir.join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    }
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn field_of(section: &str, kind: &str, key: &str) -> Dict {
    fields(source::schema(), section, kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("{section}.{kind}.{key}")).clone()
}

fn options(section: &str, kind: &str, key: &str) -> Vec<String> {
    json::aget(&field_of(section, kind, key), "options").iter().map(gstr).collect()
}

/// The HOOKS1 rule ids (HOOKS1 4.1), in the register's order.
const HOOK_RULES: [&str; 9] = ["forced_move", "ground_hazards", "arena_hazards", "triggers", "unit_meters", "hp_cost", "ignore_guard", "ai_tactics", "execute"];

/// The hook skill fields with the kinds HOOKS1_TASKS 1.1 gives them and whether they are per rank.
const SKILL_FIELDS: [(&str, &[&str], bool); 7] = [
    ("forced_move", &["ATTACK"], false),
    ("hazard", &["ATTACK", "DEFEND", "SUPPORT"], false),
    ("triggers", &["ATTACK", "DEFEND", "SUPPORT"], false),
    ("riders", &["ATTACK", "SUPPORT"], false),
    // U1.2-S5: Rest too (a pool's gain on rest, U1 composure)
    ("meters", &["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE", "REST"], true),
    ("hp_cost", &["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"], true),
    ("ignore_guard", &["ATTACK"], false),
];

/// Fixture files: skills of every hook (a push, an area hazard, a placement, a Guard with an intercept trigger, a rider
/// with the guard bypass, an HP cost, a meter skill, an execute exploit), two hazard rows, two unit meters (pips with a
/// meter skill; bands, decay and a rule), a data zone, passives with each hook effect, a class and units owning the
/// skills, a timeline row with the deck rule and every hook rule, a consumable that leaves a hazard, and an encounter
/// with an arena hazard and a deck phase.
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
            one("slam", "ATTACK", json!({"forced_move": {"kind": "push", "tiles": 2, "collide_mult": 0.5, "collide_stagger": 6}})),
            one("sweep", "ATTACK", json!({"area": "BURST", "hazard": {"id": "fire", "shape": "SKILL", "rounds": 2}})),
            one("snare", "ATTACK", json!({"power_mult": 0.0, "reach_min": 1, "reach_max": 3, "hazard": {"id": "wire", "shape": "NONE", "at": "target", "rounds": 3}})),
            one("hook", "DEFEND", json!({"blocks": 2, "mitigation": 0.3,
                "triggers": [{"on": "ally_hit", "when": {"range": 1, "melee": true}, "effects": [{"type": "redirect", "pct": 100}, {"type": "move", "kind": "pull", "tiles": 1, "to": "other"}]}]})),
            one("brand", "ATTACK", json!({"ignore_guard": true, "riders": [{"when": {"other_broken": true}, "effects": [{"type": "end_guard"}]}]})),
            one("rush", "ATTACK", json!({"hp_cost": 10})),
            one("coat", "SUPPORT", json!({"grant_status": "haste", "meters": [{"meter": "phial", "delta": -1}]})),
            one("carver", "ATTACK", json!({"exploit": [{"status": "expose", "power_mult": 1.5}, {"hp_below": 35, "power_mult": 2.0}]}))
        ]),
    );
    write_kind(
        data,
        HAZARDS_KIND,
        1,
        json!([
            {"id": "fire", "name": "Fire", "on": "round_end", "status": "burn", "potency": 80, "magnitude": 5, "status_rounds": 2, "damage": 8, "tint": "#e8742a", "glyph": "^"},
            {"id": "wire", "name": "Tripwire", "on": "enter", "status": "root", "potency": 90, "damage": 15, "affects": "foes", "trap": true, "hidden": true, "stop_moves": true, "duration": 3, "glyph": "t"}
        ]),
    );
    write_kind(
        data,
        UNIT_METERS_KIND,
        1,
        json!([
            {"id": "phial", "name": "Bile Phial", "short": "Phial", "max": 3, "skills": ["coat"]},
            {"id": "fury", "name": "Slag Fury", "short": "Fury", "max": 5, "decay": [{"delta": -1, "unless_gained": true}],
             "bands": [{"id": "f0", "name": "Calm", "from": 0}, {"id": "f1", "name": "Hot", "from": 1, "statuses": [{"status": "empower", "magnitude": 5}]}],
             "rules": [{"id": "full", "at": 5, "dir": "up", "once": true, "effects": [{"type": "meter_set", "meter": "fury", "value": 3}]}]}
        ]),
    );
    write_kind(data, ZONES_KIND, 1, json!([{"id": "alley", "name": "Cinder Alley", "ground": "stone"}]));
    write_kind(
        data,
        "passives",
        1,
        json!([
            {"id": "spite", "name": "Spite", "effects": [{"type": "meter", "meter": "fury"}, {"type": "trigger", "on": "hit_taken", "limit": "none", "effects": [{"type": "meter", "meter": "fury", "delta": 1}]}]},
            {"id": "braced", "name": "Braced", "effects": [{"type": "forced_move", "immune": "guarding"}]},
            {"id": "iron", "name": "Iron", "effects": [{"type": "trigger", "on": "hp_below", "when": {"pct": 30}, "limit": "battle",
                "effects": [{"type": "ap", "value": 2}, {"type": "grant", "status": "fortify", "rounds": 2, "magnitude": 8}]}]}
        ]),
    );
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2, "CONS": 2, "DEX": 1},
                "proficiency": {"sword": {"level": 1}}, "skill_path": {"slam": 1, "rush": 1, "brand": 3}}]),
    );
    write_kind(
        data,
        "units",
        1,
        json!([
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1, "extra_skills": ["hook", "sweep"], "tactics": ["brace_after_hit", "hold_line"], "target_pref": "TOUGHEST",
             "deck": {"active": ["slam"]}},
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "imp", "name": "Imp", "side": "enemy", "class_id": "drill", "stat_mode": "fixed", "hp": 30, "extra_skills": ["rush"]}
        ]),
    );
    let mut rules = strs(&["round_intent", "deck"]);
    rules.extend(strs(&HOOK_RULES));
    write_kind(data, ruleset::KIND, 1, json!([{"id": "timeline", "name": "Timeline", "rules": rules, "basics": ["move", "rest", "strike"], "deck_slots": {"active": 2, "passive": 1, "keystone": 1}}]));
    write_kind(data, "item_families", 1, json!([{"id": "bomb", "name": "Bomb", "item_type": "CONSUMABLE"}]));
    write_kind(data, "items", 2, json!([{"id": "pot", "name": "Pot", "family": "bomb", "consumable_type": "THROWABLE_BOMB", "damage": 5, "hazard": {"id": "fire", "shape": "BURST"}}]));
    write_kind(
        data,
        "encounters",
        1,
        json!([{"id": "kiln", "name": "Kiln", "party": ["hero"], "enemies": ["brute"],
            "arena_hazards": [{"id": "vents", "name": "Slag Vents", "columns": [7, 8], "blocks": [3, 6], "effect": {"damage_pct": 12, "stagger": 10, "status": "burn", "potency": 60, "magnitude": 3, "status_rounds": 1}, "line": "The vents roar."}],
            "phases": [{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute", "add": ["rush"], "order": {"assault": ["rush", "slam"]}}]}]}]),
    );
}

/// The load problems of the catalog files (the scratch folder has no progression file, which the loader reports).
fn catalog_errors(src: &source::Source) -> Vec<String> {
    src.errors.iter().filter(|e| e.contains("catalog")).cloned().collect()
}

#[test]
fn the_schema_and_the_code_name_the_same_fields_kinds_options_and_rules() {
    for (key, kinds, rank) in SKILL_FIELDS {
        let f = field_of("content", "skills", key);
        assert_eq!(json::b(&f, "rank", false), rank, "{key}: per rank");
        let users: Vec<&str> = skill::KINDS.iter().copied().filter(|k| skill::has_field(k, key)).collect();
        assert_eq!(users, kinds.to_vec(), "{key}: the kinds that use it");
        let shown: Vec<String> = json::aget(json::dget(&f, "when"), "in").iter().map(gstr).collect();
        assert_eq!(shown, strs(kinds), "{key}: the schema's `when` names the same kinds");
    }
    // the rule register: the nine ids, extensions, after RULESET1's, and the ruleset row's options list them
    let ids: Vec<&str> = REGISTER.iter().map(|r| r.id).collect();
    let at = ids.len() - HOOK_RULES.len();
    assert_eq!(ids[at..].to_vec(), HOOK_RULES.to_vec());
    assert!(REGISTER[at..].iter().all(|r| r.kind == RuleKind::Extension), "every hook rule is an extension");
    assert_eq!(options("content", ruleset::KIND, "rules"), strs(&ids));
    assert!(HOOK_RULES.iter().all(|r| !Ruleset::classic().has(r)), "classic has none");
    assert_eq!(options("content", "units", "target_pref"), strs(&rpgkit_data::TARGET_PREFS));
    assert_eq!(options("content", "units", "tactics"), strs(&rpgkit_data::TACTICS));
    assert_eq!(options("content", HAZARDS_KIND, "on"), strs(&rpgkit_data::HAZARD_ON));
    assert_eq!(options("content", HAZARDS_KIND, "affects"), strs(&rpgkit_data::HAZARD_AFFECTS));
    assert_eq!(options("content", UNIT_METERS_KIND, "show"), strs(&rpgkit_data::METER_SHOW));
    assert_eq!(source::HOOK_KINDS.to_vec(), vec![HAZARDS_KIND, UNIT_METERS_KIND, ZONES_KIND]);
    for kind in source::HOOK_KINDS {
        assert_eq!(json::s(json::dict(&fields(source::schema(), "content", kind)[0]), "key"), "id", "{kind}: id first");
    }
    for (section, kind, key) in [("content", "items", "hazard"), ("content", "encounters", "arena_hazards"), ("world", "encounters", "arena_hazards"), ("world", "world", "classes")] {
        let f = field_of(section, kind, key);
        assert_eq!(json::s(&f, "type"), "json", "{section}.{kind}.{key}");
    }
    assert_eq!(json::aget(json::dget(&field_of("content", "items", "hazard"), "when"), "in").iter().map(gstr).collect::<Vec<_>>(), ["CONSUMABLE"]);
    // the hook numbers are range-checked by the checks: no schema min / max, so normalizing never clamps them
    let unclamped = [("skills", "hp_cost"), (HAZARDS_KIND, "potency"), (HAZARDS_KIND, "magnitude"), (HAZARDS_KIND, "status_rounds"), (HAZARDS_KIND, "damage"), (HAZARDS_KIND, "cover_guard"), (HAZARDS_KIND, "move_cost"), (HAZARDS_KIND, "duration"), (UNIT_METERS_KIND, "min"), (UNIT_METERS_KIND, "max"), (UNIT_METERS_KIND, "start")];
    for (kind, key) in unclamped {
        let f = field_of("content", kind, key);
        assert!(!f.contains_key("min") && !f.contains_key("max"), "{kind}.{key}: no schema clamp");
    }
}

#[test]
fn the_fields_load_into_the_typed_catalog_and_default_to_nothing() {
    let (_scratch, data) = scratch("load");
    catalog(&data);
    let src = source::load(&data);
    assert_eq!(catalog_errors(&src), Vec::<String>::new());
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let plain = cat.skill("strike").unwrap();
    assert_eq!(
        (plain.forced_move.is_none(), plain.hazard.is_none(), plain.triggers.len(), plain.riders.len(), plain.meters.len(), plain.hp_cost, plain.ignore_guard),
        (true, true, 0, 0, 0, 0, false),
        "the schema defaults: off"
    );
    let slam = cat.skill("slam").unwrap().forced_move.clone().unwrap();
    assert_eq!((slam.kind.as_str(), slam.tiles, slam.from.as_str(), slam.when.as_str(), slam.collide_mult, slam.collide_stagger), ("push", 2, "source", "always", 0.5, 6));
    let snare = cat.skill("snare").unwrap().hazard.clone().unwrap();
    assert_eq!((snare.id.as_str(), snare.shape.as_str(), snare.at.as_str(), snare.rounds), ("wire", "NONE", "target", 3));
    let hook = cat.skill("hook").unwrap();
    assert_eq!((hook.triggers[0].on.as_str(), hook.triggers[0].limit.as_str(), hook.triggers[0].effects.len()), ("ally_hit", "round", 2));
    assert_eq!(cat.skill("brand").unwrap().riders.len(), 1);
    assert!(cat.skill("brand").unwrap().ignore_guard);
    assert_eq!(cat.skill("rush").unwrap().hp_cost, 10);
    assert_eq!(cat.skill("coat").unwrap().meters, vec![rpgkit_data::MeterDelta { meter: "phial".into(), delta: -1 }]);
    assert_eq!(cat.units["brute"].tactics, strs(&["brace_after_hit", "hold_line"]));
    assert!(cat.units["hero"].tactics.is_empty());
    assert_eq!(cat.items["pot"].hazard.as_ref().map(|h| h.shape.as_str()), Some("BURST"));
    let wire = &cat.hazards["wire"];
    assert_eq!((wire.on.as_str(), wire.affects.as_str(), wire.trap, wire.hidden, wire.duration, wire.tint.as_str()), ("enter", "foes", true, true, 3, "#808080"));
    assert_eq!((cat.unit_meters["phial"].start, cat.unit_meters["phial"].show.as_str(), cat.unit_meters["fury"].bands.len()), (0, "pips", 2));
    assert_eq!((cat.zones["alley"].rows.len(), cat.zones["alley"].backdrop.as_str(), cat.zones["alley"].ground.as_str()), (0, "", "stone"));
    let timeline = Ruleset::of(&cat, ruleset::TIMELINE).unwrap();
    assert!(HOOK_RULES.iter().all(|r| timeline.has(r)), "a row lists them");
    let mut off = timeline.clone();
    off.adjust("-execute,-forced_move").unwrap();
    assert!(!off.has(ruleset::EXECUTE) && !off.has(ruleset::FORCED_MOVE) && off.has(ruleset::TRIGGERS), "they switch one by one");
    // the description token of an execute entry
    let carver = cat.skill("carver").unwrap();
    assert_eq!(source::render("{exploit}", "ATTACK", &carver.raw, 1, true), "x1.5 vs Exposed, x2.0 at 35% HP or less");
    drop_scratch(&data);
}

#[test]
fn the_fixture_checks_clean_and_a_bad_kind_row_is_dropped_with_its_reason() {
    let (_scratch, data) = scratch("clean");
    catalog(&data);
    let v = store::view(&data);
    let mut failures = Vec::new();
    for kind in ["skills", "passives", "units", "items", "encounters"] {
        for e in v.entries(kind) {
            let errors = check::validate_entry(&v, kind, e, &json::s(e, "id"));
            if !errors.is_empty() {
                failures.push(format!("{kind} {}: {errors:?}", json::s(e, "id")));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // one bad row of each kind: dropped by the loader, named in its errors; the good rows stay
    write_kind(&data, HAZARDS_KIND, 1, json!([{"id": "fire", "name": "Fire", "status": "burn", "magnitude": 5}, {"id": "ghost", "name": "Ghost", "hidden": true}]));
    write_kind(&data, ZONES_KIND, 1, json!([{"id": "alley", "name": "Cinder Alley"}, {"id": "stone_arena", "name": "Stone Arena"}]));
    let src = source::load(&data);
    let errors = catalog_errors(&src);
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors[0].contains("hazard 'ghost': Hidden needs affects foes"), "{errors:?}");
    assert!(errors[1].contains("zone 'stone_arena': ID 'stone_arena' is a code zone"), "{errors:?}");
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert_eq!((cat.hazards.keys().cloned().collect::<Vec<_>>(), cat.zones.keys().cloned().collect::<Vec<_>>()), (strs(&["fire"]), strs(&["alley"])));
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
    let (_scratch, data) = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let trig = |on: &str, when: Value, effects: Value| json!([{"type": "trigger", "on": on, "when": when, "effects": effects}]);
    let ap = json!([{"type": "ap", "value": 1}]);
    let cases: Vec<Case> = vec![
        // what the rules allow
        ("skills", "slam", json!({"forced_move": {"kind": "pull", "tiles": 3, "when": "moved", "stop_moves": true, "collide_break": true}}), ""),
        ("skills", "sweep", json!({"forced_move": {"kind": "push", "tiles": 1, "from": "center"}}), ""),
        ("skills", "hook", json!({"hazard": {"id": "fire", "at": "self", "shape": "BURST"}}), ""),
        ("skills", "carver", json!({"exploit": [{"hp_below": 35, "power_mult": 2.0}]}), ""),
        ("passives", "iron", json!({"effects": trig("ally_downed", json!({"range": 3}), json!([{"type": "reveal", "rounds": 1}]))}), ""),
        ("passives", "iron", json!({"effects": trig("kill", json!({"skills": ["slam"], "ranged": true}), ap.clone())}), ""),
        ("passives", "iron", json!({"effects": trig("hit_dealt", json!({"other_status": "burn", "other_hp_below": 30}), json!([{"type": "inflict", "status": "expose", "potency": 100, "magnitude": 4}]))}), ""),
        ("passives", "iron", json!({"effects": trig("guard_start", json!({}), json!([{"type": "grant", "status": "steadfast", "rounds": -1, "until": "guard_end"}]))}), ""),
        ("passives", "iron", json!({"effects": trig("round_start", json!({"meter": {"id": "fury", "at_least": 3}, "intent": "assault"}), json!([{"type": "crit", "value": 20}]))}), ""),
        ("units", "brute", json!({"tactics": ["flush_cover"], "target_pref": "SOFTEST"}), ""),
        // skills (1.1)
        ("skills", "slam", json!({"forced_move": {"kind": "shove", "tiles": 2}}), "Forced move: kind 'shove' is not one of push, pull."),
        ("skills", "slam", json!({"forced_move": {"kind": "push"}}), "Forced move: tiles is required (1-3)."),
        ("skills", "slam", json!({"forced_move": {"kind": "push", "tiles": 4}}), "Forced move: tiles must be a whole number 1-3."),
        ("skills", "slam", json!({"forced_move": {"kind": "push", "tiles": 1, "from": "center"}}), "Forced move: from center needs an area attack"),
        ("skills", "slam", json!({"forced_move": {"kind": "push", "tiles": 1, "collide_mult": 3.5}}), "Forced move: collide_mult must be 0.0-3.0."),
        ("skills", "slam", json!({"forced_move": {"kind": "push", "tiles": 1, "collide_stagger": 60}}), "Forced move: collide_stagger must be a whole number 0-50."),
        ("skills", "slam", json!({"forced_move": {"kind": "push", "tiles": 1, "when": "later"}}), "Forced move: when 'later' is not one of always, moved, windup."),
        ("skills", "slam", json!({"forced_move": {"kind": "push", "tiles": 1, "distance": 2}}), "Forced move: unknown key 'distance'"),
        ("skills", "hook", json!({"forced_move": {"kind": "push", "tiles": 1}}), "Forced move belongs to ATTACK skills, not DEFEND."),
        ("skills", "sweep", json!({"hazard": {"id": "lava"}}), "Leaves hazard: unknown hazard 'lava'."),
        ("skills", "sweep", json!({"hazard": {"shape": "BURST"}}), "Leaves hazard: id is required"),
        ("skills", "slam", json!({"hazard": {"id": "fire", "shape": "SKILL"}}), "Leaves hazard: shape SKILL needs an area attack"),
        // A3.2-K2: RING and ARC are hazard shapes (as the skill shapes); CONE is not (no hazard wants it yet)
        ("skills", "sweep", json!({"hazard": {"id": "fire", "shape": "CONE"}}), "Leaves hazard: shape 'CONE' is not one of"),
        ("skills", "sweep", json!({"hazard": {"id": "fire", "shape": "RING"}}), ""),
        ("skills", "sweep", json!({"hazard": {"id": "fire", "shape": "ARC"}}), ""),
        ("skills", "sweep", json!({"hazard": {"id": "fire", "at": "near"}}), "Leaves hazard: at 'near' is not one of target, self."),
        ("skills", "sweep", json!({"hazard": {"id": "fire", "rounds": 6}}), "Leaves hazard: rounds must be a whole number 0-5."),
        ("skills", "snare", json!({"hazard": {}}), "A skill with no damage needs a status"),
        ("skills", "hook", json!({"triggers": [{"on": "kill", "effects": [{"type": "ap", "value": 1}]}]}), "Trigger 1: on 'kill' is not one of hit_taken, guarded_hit, ally_hit, evaded, broken."),
        ("skills", "hook", json!({"triggers": [{"on": "hit_taken", "effects": [{"type": "redirect", "pct": 50}]}]}), "Trigger 1 effect 1: redirect needs an ally_hit trigger."),
        ("skills", "hook", json!({"triggers": [{"on": "ally_hit", "effects": [{"type": "redirect", "pct": 50}, {"type": "redirect", "pct": 50}]}]}), "Trigger 1: one redirect per row."),
        ("skills", "hook", json!({"triggers": [{"on": "ally_hit", "effects": []}]}), "Trigger 1: effects holds 1-4 effects."),
        ("skills", "hook", json!({"triggers": [{"on": "ally_hit", "effects": [{"type": "redirect", "pct": 100}], "limit": "twice"}]}), "Trigger 1: limit 'twice' is not one of round, battle, none."),
        ("skills", "hook", json!({"triggers": [{"on": "ally_hit", "when": {"range": 7}, "effects": [{"type": "redirect", "pct": 100}]}]}), "Trigger 1 when: range must be a whole number 1-6."),
        ("skills", "hook", json!({"triggers": [{"on": "hit_taken", "when": {"range": 2}, "effects": ap.clone()}]}), "Trigger 1 when: range is for ally_hit and ally_downed only."),
        ("skills", "hook", json!({"triggers": vec![json!({"on": "evaded", "effects": [{"type": "reveal"}]}); 4]}), "Triggers holds at most 3 rows."),
        ("skills", "brand", json!({"riders": [{"effects": [{"type": "redirect", "pct": 100}]}]}), "Rider 1 effect 1: a rider cannot redirect."),
        ("skills", "brand", json!({"riders": [{"on": "kill", "effects": [{"type": "end_guard"}]}]}), "Rider 1: unknown key 'on'"),
        ("skills", "brand", json!({"riders": [{"when": {"pct": 30}, "effects": [{"type": "end_guard"}]}]}), "Rider 1 when: pct is for an hp_below trigger only."),
        ("skills", "brand", json!({"riders": [{"effects": [{"type": "end_guard", "to": "both"}]}]}), "Rider 1 effect 1: to 'both' is not one of self, other."),
        ("skills", "hook", json!({"riders": [{"effects": [{"type": "end_guard"}]}]}), "Riders belongs to ATTACK/SUPPORT skills, not DEFEND."),
        ("skills", "coat", json!({"meters": [{"meter": "strain", "delta": 1}]}), "Meter cost 1: unknown unit meter 'strain'."),
        ("skills", "coat", json!({"meters": [{"meter": "phial", "delta": 0}]}), "Meter cost 1: delta must be a whole number -100..100, not 0."),
        ("skills", "coat", json!({"meters": [{"meter": "phial", "delta": -1}, {"meter": "phial", "delta": -1}]}), "Meter costs list 'phial' twice."),
        ("skills", "hook", json!({"ignore_guard": true}), "Ignores guard belongs to ATTACK skills, not DEFEND."),
        ("skills", "hook", json!({"hp_cost": 5}), ""),
        ("skills", "carver", json!({"exploit": [{"hp_below": 0, "power_mult": 2.0}]}), "Exploit hp_below must be a whole number 1-99."),
        ("skills", "carver", json!({"exploit": [{"hp_below": 35, "power_mult": 2.5}]}), "Exploit power must be 1.05-2.0."),
        ("skills", "carver", json!({"exploit": [{"hp_below": 35, "status": "expose", "power_mult": 2.0}]}), "Exploit hp_below entry takes no status."),
        ("skills", "carver", json!({"exploit": [{"hp_below": 35, "power_mult": 2.0}, {"hp_below": 20, "power_mult": 1.5}]}), "Exploit takes at most one hp_below entry."),
        // HC1: the cost is range-checked, not clamped
        ("skills", "rush", json!({"hp_cost": 99}), ""),
        ("skills", "rush", json!({"hp_cost": 100}), "HP cost must be 0-99 (is 100)."),
        ("skills", "rush", json!({"hp_cost": -1}), "HP cost must be 0-99 (is -1)."),
        // passive and item effects (1.2, 1.3)
        ("passives", "braced", json!({"effects": [{"type": "forced_move"}]}), "Effect 1: forced_move sets immune or bonus_tiles."),
        ("passives", "braced", json!({"effects": [{"type": "forced_move", "immune": "sometimes"}]}), "Effect 1: immune 'sometimes' is not one of , always, guarding."),
        ("passives", "braced", json!({"effects": [{"type": "forced_move", "bonus_tiles": 3}]}), "Effect 1: bonus_tiles must be a whole number 0-2."),
        ("passives", "spite", json!({"effects": [{"type": "meter", "meter": "strain"}]}), "Effect 1: unknown unit meter 'strain'."),
        ("passives", "spite", json!({"effects": [{"type": "meter", "meter": "fury", "floor": 6}]}), "Effect 1: floor must be a whole number 0-5."),
        ("passives", "spite", json!({"effects": [{"type": "meter", "meter": "fury", "start": 9}]}), "Effect 1: start must be -1 (the meter's) or 0-5."),
        ("passives", "iron", json!({"effects": trig("hp_below", json!({}), ap.clone())}), "Effect 1 when: pct is required (1-99)."),
        ("passives", "iron", json!({"effects": [{"type": "trigger", "on": "hp_below", "effects": ap.clone()}]}), "Effect 1: an hp_below trigger needs when.pct (1-99)."),
        ("passives", "iron", json!({"effects": trig("hit_taken", json!({"pct": 50}), ap.clone())}), "Effect 1 when: pct is for an hp_below trigger only."),
        ("passives", "iron", json!({"effects": trig("explode", json!({}), ap.clone())}), "Effect 1: on 'explode' is not one of"),
        ("passives", "iron", json!({"effects": trig("kill", json!({"skills": ["nothing"]}), ap.clone())}), "Effect 1 when: skills: unknown skill family 'nothing'."),
        ("passives", "iron", json!({"effects": trig("kill", json!({"ranged": true, "melee": true}), ap.clone())}), "Effect 1 when: ranged and melee together never match."),
        ("passives", "iron", json!({"effects": trig("kill", json!({"other_status": "dizzy"}), ap.clone())}), "Effect 1 when: other_status: unknown status 'dizzy'."),
        ("passives", "iron", json!({"effects": trig("kill", json!({"other_hp_below": 100}), ap.clone())}), "Effect 1 when: other_hp_below must be a whole number 1-99."),
        ("passives", "iron", json!({"effects": trig("kill", json!({"intent": "dance"}), ap.clone())}), "Effect 1 when: intent 'dance' is not one of"),
        ("passives", "iron", json!({"effects": trig("kill", json!({"meter": {"id": "fury", "at_least": 9}}), ap.clone())}), "Effect 1 when meter: at_least must be a whole number 0-5."),
        ("passives", "iron", json!({"effects": trig("kill", json!({"mood": 1}), ap.clone())}), "Effect 1 when: unknown key 'mood'"),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "ap", "value": 5}]))}), "Effect 1 effect 1: value must be a whole number 1-4."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "grant", "status": "burn"}]))}), "Effect 1 effect 1: grant 'burn' is not a buff"),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "grant", "status": "haste", "rounds": 0}]))}), "Effect 1 effect 1: rounds must be -1 (the battle) or 1-5."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "grant", "status": "fortify", "magnitude": 30}]))}), "Effect 1 effect 1: fortify needs a magnitude 1-20."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "inflict", "status": "haste", "potency": 50}]))}), "Effect 1 effect 1: inflict 'haste' is not a debuff"),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "inflict", "status": "root"}]))}), "Effect 1 effect 1: potency is required (1-100)."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "guard", "mitigation": 0.95}]))}), "Effect 1 effect 1: guard needs a mitigation 0.1-0.9."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "move", "kind": "pull", "tiles": 4}]))}), "Effect 1 effect 1: tiles must be a whole number 1-3."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "meter", "meter": "fury", "delta": 11}]))}), "Effect 1 effect 1: delta must be a whole number -10..10, not 0."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "resist", "status": "haste", "value": 10}]))}), "Effect 1 effect 1: resist 'haste' is not a resistable status"),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "reveal", "rounds": 2}]))}), "Effect 1 effect 1: rounds must be a whole number 1-1."),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "heal", "value": 5}]))}), "Effect 1 effect 1: unknown effect type 'heal'"),
        ("passives", "iron", json!({"effects": trig("kill", json!({}), json!([{"type": "meter_set", "meter": "fury", "value": 1}]))}), "Effect 1 effect 1: unknown effect type 'meter_set'"),
        ("passives", "iron", json!({"effects": trig("ally_downed", json!({}), json!([{"type": "redirect", "pct": 100}]))}), "Effect 1 effect 1: redirect needs an ally_hit trigger."),
        // units and phase ai (1.2)
        ("units", "brute", json!({"tactics": ["charge"]}), "Tactics: 'charge' is not one of brace_after_hit, hold_line, flush_cover."),
        ("units", "brute", json!({"tactics": ["hold_line", "hold_line"]}), "Tactics lists 'hold_line' twice."),
        ("units", "brute", json!({"target_pref": "LOUDEST"}), "Targets: 'LOUDEST' is not one of ROLE, NEAREST, WEAKEST, SOFTEST, TOUGHEST."),
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
fn item_and_encounter_rules_each_report_one_error() {
    let (_scratch, data) = scratch("encounters");
    catalog(&data);
    let v = store::view(&data);
    let item = |hazard: Value| errors_of(&v, "items", "pot", &json!({"hazard": hazard}));
    assert_eq!(item(json!({})), Vec::<String>::new());
    assert_eq!(item(json!({"id": "fire", "shape": "SKILL"})), ["Leaves hazard: shape SKILL needs an area attack (use NONE, PLUS, BURST, LINE, RING or ARC)."]);
    assert_eq!(item(json!({"id": "smog"})), ["Leaves hazard: unknown hazard 'smog'."]);
    let vents = |patch: Value| {
        let mut h = json!({"id": "vents", "name": "Slag Vents", "columns": [7, 8], "blocks": [3, 6], "effect": {"damage_pct": 12}});
        for (k, x) in patch.as_object().unwrap() {
            if x.is_null() {
                h.as_object_mut().unwrap().remove(k);
            } else {
                h[k] = x.clone();
            }
        }
        errors_of(&v, "encounters", "kiln", &json!({"arena_hazards": [h]}))
    };
    let cases: Vec<(Value, &str)> = vec![
        (json!({}), ""),
        (json!({"columns": null, "cells": [[7, 4], [8, 4]], "rounds": [2, 4]}), ""),
        (json!({"effect": {"push": {"dir": "east", "tiles": 2, "collide_damage": 5}}}), ""),
        (json!({"columns": null}), "Arena hazard 'vents': set exactly one of cells, columns or rows."),
        (json!({"rows": [4]}), "Arena hazard 'vents': set exactly one of cells, columns or rows."),
        (json!({"columns": [12]}), "Arena hazard 'vents': columns are whole numbers 0-11."),
        (json!({"columns": null, "cells": [[7, 9]]}), "is not an [x, y] on the 12x9 board."),
        (json!({"blocks": []}), "Arena hazard 'vents': blocks lists at least one whole number 1-6."),
        (json!({"blocks": [7]}), "Arena hazard 'vents': blocks lists at least one whole number 1-6."),
        (json!({"rounds": [7]}), "Arena hazard 'vents': rounds are whole numbers 1-6 ([] = every round)."),
        (json!({"name": ""}), "Arena hazard 'vents': the name needs 1 to 24 characters."),
        (json!({"id": "Vents"}), "Arena hazard 'Vents': the id must be lowercase snake_case"),
        (json!({"effect": {}}), "Arena hazard 'vents' effect: set at least one of damage, damage_pct, stagger, status or push."),
        (json!({"effect": {"damage": 5, "damage_pct": 5}}), "Arena hazard 'vents' effect: damage and damage_pct together (one of them)."),
        (json!({"effect": {"damage_pct": 60}}), "Arena hazard 'vents' effect: damage_pct must be a whole number 0-50."),
        (json!({"effect": {"status": "haste"}}), "Arena hazard 'vents' effect: status 'haste' is not a debuff."),
        (json!({"effect": {"stagger": 5, "potency": 50}}), "Arena hazard 'vents' effect: potency, magnitude and status_rounds need a status."),
        (json!({"effect": {"push": {"dir": "up", "tiles": 1}}}), "Arena hazard 'vents' effect push: dir 'up' is not one of east, west, north, south."),
        (json!({"effect": {"push": {"dir": "east"}}}), "Arena hazard 'vents' effect push: tiles is required (1-3)."),
        (json!({"line": "x".repeat(161)}), "Arena hazard 'vents': the line is text of at most 160 characters."),
        (json!({"when": 3}), "Arena hazard 'vents': unknown key 'when'"),
    ];
    let mut failures = Vec::new();
    for (patch, want) in &cases {
        let errors = vents(patch.clone());
        let ok = if want.is_empty() { errors.is_empty() } else { errors.len() == 1 && errors[0].contains(want) };
        if !ok {
            failures.push(format!("{patch}: want {want:?}, got {errors:?}"));
        }
    }
    let five: Vec<Value> = (0..5).map(|i| json!({"id": format!("v{i}"), "name": "V", "columns": [i], "blocks": [1], "effect": {"stagger": 5}})).collect();
    let errors = errors_of(&v, "encounters", "kiln", &json!({"arena_hazards": five}));
    if errors != ["Arena hazards holds at most 4 rows."] {
        failures.push(format!("five rows: {errors:?}"));
    }
    // the deck phase effect (DU1, DU2)
    let deck = |fx: Value| errors_of(&v, "encounters", "kiln", &json!({"phases": [{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [fx]}]}));
    let deck_cases: Vec<(Value, &str)> = vec![
        (json!({"type": "deck", "unit": "brute", "order": {"guard": ["hook"]}}), "Phase 'pulverize': deck order guard: 'hook' is neither an active of the deck nor a basic."),
        (json!({"type": "deck", "unit": "brute", "add": ["brand"]}), "Phase 'pulverize': deck add: the unit does not own the skill family 'brand'."),
        (json!({"type": "deck", "unit": "brute", "add": ["nothing"]}), "Phase 'pulverize': deck add: unknown skill family 'nothing'."),
        (json!({"type": "deck", "unit": "brute", "add": ["rush", "hook"]}), "Phase 'pulverize': deck add makes 3 actives (the Timeline ruleset's deck slots allow 2)."),
        (json!({"type": "deck", "unit": "brute", "add": ["rush"], "order": {"guard": ["rush"]}}), "Phase 'pulverize': deck order guard: 'rush' is of intent assault."),
        (json!({"type": "deck", "unit": "brute"}), "Phase 'pulverize': a deck effect sets add or order."),
        (json!({"type": "deck", "unit": "ghost", "add": ["rush"]}), "Phase 'pulverize': effect unit 'ghost' is not in this encounter."),
        (json!({"type": "deck", "unit": "brute", "add": "rush"}), "Phase 'pulverize': deck add must be a list of skill families."),
        (json!({"type": "deck", "unit": "brute", "add": ["rush"], "slots": 3}), "Phase 'pulverize': unknown deck key 'slots'."),
        (json!({"type": "ai", "unit": "brute", "tactics": ["hold_line"], "target_pref": "SOFTEST"}), ""),
        (json!({"type": "ai", "unit": "brute", "tactics": ["dance"]}), "Phase 'pulverize': tactics: 'dance' is not one of brace_after_hit, hold_line, flush_cover."),
        (json!({"type": "ai", "unit": "brute", "target_pref": "ROLE"}), "Phase 'pulverize': target_pref must be one of NEAREST, WEAKEST, SOFTEST, TOUGHEST."),
    ];
    for (fx, want) in &deck_cases {
        let errors = deck(fx.clone());
        let ok = if want.is_empty() { errors.is_empty() } else { errors.len() == 1 && errors[0].contains(want) };
        if !ok {
            failures.push(format!("{fx}: want {want:?}, got {errors:?}"));
        }
    }
    let two = errors_of(&v, "encounters", "kiln", &json!({"phases": [
        {"id": "a", "name": "A", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute", "add": ["rush"]}]},
        {"id": "b", "name": "B", "trigger": {"kind": "round", "round": 3}, "effects": [{"type": "deck", "unit": "brute", "order": {"assault": ["slam"]}}]}]}));
    if two != ["Phases hold one deck effect per unit ('brute' has two)."] {
        failures.push(format!("two deck effects: {two:?}"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // without the deck rule in any ruleset row the effect is refused
    write_kind(&data, ruleset::KIND, 1, json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent"]}]));
    let v = store::view(&data);
    assert_eq!(errors_of(&v, "encounters", "kiln", &json!({})), ["A deck phase effect needs a ruleset row with the deck rule."]);
    drop_scratch(&data);
}

#[test]
fn the_hook_kinds_check_their_rows() {
    let (_scratch, data) = scratch("kinds");
    catalog(&data);
    let v = store::view(&data);
    let cases: Vec<(&str, &str, Value, &str)> = vec![
        (HAZARDS_KIND, "fire", json!({}), ""),
        (HAZARDS_KIND, "fire", json!({"status": "haste"}), "Status: 'haste' is not a debuff"),
        (HAZARDS_KIND, "fire", json!({"magnitude": 11}), "burn needs a magnitude 1-10."),
        (HAZARDS_KIND, "fire", json!({"status": "root", "magnitude": 3}), "root takes no magnitude (0)."),
        (HAZARDS_KIND, "fire", json!({"status": "", "magnitude": 3}), "Magnitude needs a status."),
        (HAZARDS_KIND, "fire", json!({"hidden": true}), "Hidden needs affects foes"),
        (HAZARDS_KIND, "fire", json!({"tint": "orange"}), "Tint 'orange' must be #rrggbb."),
        (HAZARDS_KIND, "fire", json!({"glyph": "^^"}), "Glyph '^^' must be one character."),
        (HAZARDS_KIND, "fire", json!({"on": "leave"}), "Acts on: 'leave' is not one of enter, round_end, both."),
        (HAZARDS_KIND, "fire", json!({"name": "A Very Long Hazard Name Indeed"}), "Name is 30 characters (max 24)."),
        (HAZARDS_KIND, "fire", json!({"id": "wire"}), "ID 'wire' is already used by another hazard."),
        // GH1: the numbers are range-checked, not clamped
        (HAZARDS_KIND, "fire", json!({"potency": 0, "damage": 50, "cover_guard": 0.5, "move_cost": 3, "duration": 5, "status_rounds": 5}), ""),
        (HAZARDS_KIND, "fire", json!({"potency": 101}), "Potency must be 0-100 (is 101)."),
        (HAZARDS_KIND, "fire", json!({"potency": -5}), "Potency must be 0-100 (is -5)."),
        (HAZARDS_KIND, "fire", json!({"status_rounds": 0}), "Status rounds must be 1-5 (is 0)."),
        (HAZARDS_KIND, "fire", json!({"status_rounds": 6}), "Status rounds must be 1-5 (is 6)."),
        (HAZARDS_KIND, "fire", json!({"damage": 51}), "Damage must be 0-50 (is 51)."),
        (HAZARDS_KIND, "fire", json!({"damage": -1}), "Damage must be 0-50 (is -1)."),
        (HAZARDS_KIND, "fire", json!({"cover_guard": 0.6}), "Cover must be 0.0-0.5 (is 0.6)."),
        (HAZARDS_KIND, "fire", json!({"cover_guard": -0.1}), "Cover must be 0.0-0.5 (is -0.1)."),
        (HAZARDS_KIND, "fire", json!({"move_cost": 4}), "Move cost must be 0-3 (is 4)."),
        (HAZARDS_KIND, "fire", json!({"duration": 0}), "Duration must be 1-5 (is 0)."),
        (HAZARDS_KIND, "fire", json!({"duration": 6}), "Duration must be 1-5 (is 6)."),
        (HAZARDS_KIND, "fire", json!({"magnitude": -2}), "burn needs a magnitude 1-10."),
        (UNIT_METERS_KIND, "fury", json!({}), ""),
        (UNIT_METERS_KIND, "phial", json!({"short": ""}), "Short name needs 1 to 8 characters."),
        (UNIT_METERS_KIND, "phial", json!({"min": 3, "start": 3}), "Min must be below max"),
        (UNIT_METERS_KIND, "phial", json!({"max": 201, "show": "bar"}), "max at most 200"),
        (UNIT_METERS_KIND, "phial", json!({"start": 4}), "Start must be 0-3 (is 4)."),
        (UNIT_METERS_KIND, "phial", json!({"start": -1}), "Start must be 0-3 (is -1)."),
        (UNIT_METERS_KIND, "phial", json!({"max": 0}), "Min must be below max"),
        (UNIT_METERS_KIND, "phial", json!({"min": -5}), ""),
        (UNIT_METERS_KIND, "phial", json!({"max": 12}), "Pips show at most 10 steps (max - min is 12); use bar."),
        // UM7 (HOOKS1-UI): the pip cutoff is the steps from min to max, as `meter::shown` draws them
        (UNIT_METERS_KIND, "phial", json!({"min": -8}), "Pips show at most 10 steps (max - min is 11); use bar."),
        (UNIT_METERS_KIND, "phial", json!({"min": 2, "max": 12, "start": 2}), ""),
        (UNIT_METERS_KIND, "phial", json!({"skills": ["strike"]}), "Meter skills: 'strike' does not spend phial"),
        (UNIT_METERS_KIND, "phial", json!({"skills": ["nothing"]}), "Meter skills: unknown skill 'nothing'."),
        (UNIT_METERS_KIND, "fury", json!({"decay": [{"delta": -60}]}), "Decay 1: delta must be a whole number -50..50, not 0."),
        (UNIT_METERS_KIND, "fury", json!({"decay": [{"delta": -1}, {"delta": -1}, {"delta": -1}, {"delta": -1}]}), "Decay holds at most 3 rows."),
        (UNIT_METERS_KIND, "fury", json!({"bands": [{"id": "f1", "name": "Hot", "from": 1}]}), "Band 'f1': the first band starts at min (0)."),
        (UNIT_METERS_KIND, "fury", json!({"bands": [{"id": "f0", "from": 0}, {"id": "f1", "from": 0}]}), "Band 'f1': bands run in ascending from."),
        (UNIT_METERS_KIND, "fury", json!({"bands": [{"id": "f0", "from": 0, "statuses": [{"status": "fear"}]}]}), "Band 'f0' status 1: 'fear' is not a status."),
        (UNIT_METERS_KIND, "fury", json!({"bands": [{"id": "f0", "from": 0, "statuses": [{"status": "empower", "magnitude": 30}]}]}), "Band 'f0' status 1: empower needs a magnitude 1-25."),
        (UNIT_METERS_KIND, "fury", json!({"bands": [{"id": "f0", "from": 0, "effects": [{"type": "ap_regen", "value": 1}]}]}), "Band 'f0' effect 1: 'ap_regen' is not one of stat, resist, stagger_taken, skill_potency, status_potency, crit."),
        (UNIT_METERS_KIND, "fury", json!({"bands": [{"id": "f0", "from": 0, "effects": [{"type": "resist", "status": "stun", "value": 0}]}]}), "Band 'f0': Effect 1: resistance is 1-100."),
        (UNIT_METERS_KIND, "fury", json!({"rules": [{"id": "full", "at": "f9", "effects": [{"type": "ap", "value": 1}]}]}), "Rule 'full': at must be a band id or a whole number 0-5."),
        (UNIT_METERS_KIND, "fury", json!({"rules": [{"id": "full", "at": "f1", "dir": "sideways", "effects": [{"type": "ap", "value": 1}]}]}), "Rule 'full': dir must be up or down."),
        // UM5 (HOOKS1-UI, the HOOKS1-M review): a `once` rule is remembered by its id, so every rule needs one
        (UNIT_METERS_KIND, "fury", json!({"rules": [{"at": 5, "once": true, "effects": [{"type": "ap", "value": 1}]}]}), "Rule '1': the id must be lowercase snake_case"),
        (UNIT_METERS_KIND, "fury", json!({"rules": [{"id": "a", "at": 5, "once": true, "effects": [{"type": "ap", "value": 1}]}, {"id": "a", "at": 4, "once": true, "effects": [{"type": "ap", "value": 1}]}]}), "Rule ids must be unique: 'a'."),
        (UNIT_METERS_KIND, "fury", json!({"rules": [{"id": "full", "at": 5, "effects": [{"type": "meter_set", "meter": "fury", "value": 9}]}]}), "Rule 'full' effect 1: value must be a whole number 0-5."),
        (UNIT_METERS_KIND, "fury", json!({"rules": [{"id": "full", "at": 5, "effects": [{"type": "redirect", "pct": 10}]}]}), "Rule 'full' effect 1: redirect needs an ally_hit trigger."),
        (ZONES_KIND, "alley", json!({}), ""),
        (ZONES_KIND, "alley", json!({"rows": ["...."]}), "Rows are [] (9 rows of floor) or 9 strings of 12 terrain glyphs."),
        (ZONES_KIND, "alley", json!({"ground": "carpet"}), "Ground: 'carpet' is not a terrain id"),
        (ZONES_KIND, "alley", json!({"backdrop": "../x.png"}), "Backdrop '../x.png' must be \"\" or an asset key"),
        (ZONES_KIND, "alley", json!({"backdrop": "backgrounds/battle/alley.png", "rows": vec!["............"; 9]}), ""),
        (ZONES_KIND, "alley", json!({"id": "lava_cavern"}), "ID 'lava_cavern' is a code zone"),
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
fn a_placement_and_an_hp_cost_warn() {
    let (_scratch, data) = scratch("warnings");
    catalog(&data);
    let v = store::view(&data);
    let warnings = |id: &str, change: Value| {
        let mut e = v.get("skills", id).clone();
        for (k, x) in change.as_object().unwrap() {
            e.insert(k.clone(), x.clone());
        }
        check::warnings_for(&v, "skills", &e)
    };
    assert_eq!(warnings("snare", json!({})), Vec::<String>::new());
    assert_eq!(warnings("snare", json!({"status_effect": "root", "status_potency": 50, "stagger": 5})), ["A placement (power 0 with a hazard) lands no hit, so these never apply: status, stagger."]);
    // the owners of rush (the class path, the extra skills) are the three units; the lowest max HP decides
    let cat = &v.typed;
    let lowest = ["brute", "hero", "imp"].iter().map(|u| rpgkit_rules::unit::build_unit(cat, &cat.units[*u], 0).max_hp as i64).min().unwrap();
    let cost = lowest / 2 + 1;
    let found = warnings("rush", json!({"hp_cost": cost}));
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with(&format!("An HP cost of {cost} is above half the lowest owner's max HP (")) && found[0].ends_with(&format!(": {lowest}).")), "{found:?}");
    assert_eq!(warnings("rush", json!({"hp_cost": lowest / 2})), Vec::<String>::new());
    drop_scratch(&data);
}

/// DU1 as built (user 2026-10-29 (b)): a deck `add` on an automatic deck that its class path and extra skills already
/// fill pushes out the last automatic family at run time: a warning naming it, not an error; an authored deck with room
/// warns nothing.
#[test]
fn a_deck_add_on_a_full_automatic_deck_warns() {
    let (_scratch, data) = scratch("deck_warn");
    catalog(&data);
    // brute without its authored deck: the automatic actives are slam, rush (the path's level-1 families fill the 2 slots)
    write_kind(
        &data,
        "units",
        1,
        json!([
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1, "extra_skills": ["hook", "sweep"]},
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}
        ]),
    );
    let v = store::view(&data);
    let mut e = v.get("encounters", "kiln").clone();
    e.insert("phases".into(), json!([{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute", "add": ["hook"]}]}]));
    assert_eq!(check::validate_entry(&v, "encounters", &e, "kiln"), Vec::<String>::new(), "no error");
    let found: Vec<String> = check::warnings_for(&v, "encounters", &e).into_iter().filter(|w| w.contains("deck")).collect();
    assert_eq!(found, ["Phase 'pulverize': deck add on the automatic deck of 'brute' pushes out 'rush' (the Timeline ruleset's deck slots allow 2)."]);
    // a family already among the automatic actives pushes nothing out
    e.insert("phases".into(), json!([{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute", "add": ["rush"]}]}]));
    assert!(!check::warnings_for(&v, "encounters", &e).iter().any(|w| w.contains("deck")));
    // planner 2026-10-29: an order on the automatic deck may name its automatic actives (as the run time accepts);
    // a family outside the automatic fill and the add (an extra skill past the slots, one the add pushed out) stays an error
    let order = |fx: Value| {
        let mut e = e.clone();
        e.insert("phases".into(), json!([{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [fx]}]));
        check::validate_entry(&v, "encounters", &e, "kiln")
    };
    assert_eq!(order(json!({"type": "deck", "unit": "brute", "order": {"assault": ["rush", "slam"]}})), Vec::<String>::new());
    assert_eq!(order(json!({"type": "deck", "unit": "brute", "order": {"assault": ["sweep", "slam"]}})), ["Phase 'pulverize': deck order assault: 'sweep' is neither an active of the deck nor a basic."]);
    assert_eq!(order(json!({"type": "deck", "unit": "brute", "add": ["hook"], "order": {"assault": ["rush"]}})), ["Phase 'pulverize': deck order assault: 'rush' is neither an active of the deck nor a basic."]);
    drop_scratch(&data);
    // the fixture's authored deck (slam, one short of the slots) takes the add without a warning
    let (_scratch, data) = scratch("deck_warn_authored");
    catalog(&data);
    let v = store::view(&data);
    let e = v.get("encounters", "kiln").clone();
    assert!(!check::warnings_for(&v, "encounters", &e).iter().any(|w| w.contains("deck")));
    drop_scratch(&data);
    // a loadout listing an extra before the class families: the battle fills the slots class families first
    // (`loadout_of`), so the automatic actives are slam, rush; sweep pushes out rush, and hook (listed first) is no active
    let (_scratch, data) = scratch("deck_warn_loadout");
    catalog(&data);
    write_kind(
        &data,
        "units",
        1,
        json!([
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1, "extra_skills": ["hook", "sweep"], "loadout": ["hook", "slam", "rush"]},
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}
        ]),
    );
    let v = store::view(&data);
    let mut e = v.get("encounters", "kiln").clone();
    e.insert("phases".into(), json!([{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute", "add": ["sweep"]}]}]));
    let found: Vec<String> = check::warnings_for(&v, "encounters", &e).into_iter().filter(|w| w.contains("deck")).collect();
    assert_eq!(found, ["Phase 'pulverize': deck add on the automatic deck of 'brute' pushes out 'rush' (the Timeline ruleset's deck slots allow 2)."]);
    e.insert("phases".into(), json!([{"id": "pulverize", "name": "Pulverize", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute", "order": {"assault": ["hook"]}}]}]));
    assert_eq!(check::validate_entry(&v, "encounters", &e, "kiln"), ["Phase 'pulverize': deck order assault: 'hook' is neither an active of the deck nor a basic."]);
    drop_scratch(&data);
}

#[test]
fn world_classes_and_arena_hazards_follow_the_world_and_its_ruleset() {
    let (_scratch, data) = scratch("worlds");
    catalog(&data);
    let encounter = json!({"id": "kiln", "name": "Kiln", "party": ["hero_c"], "enemies": ["brute_c"],
        "arena_hazards": [{"id": "vents", "name": "Slag Vents", "columns": [7], "blocks": [3], "effect": {"stagger": 10}}]});
    let kinds = json!({"characters": [{"id": "hero_c", "name": "Hero", "role": "party", "base_unit": "hero"}, {"id": "brute_c", "name": "Brute", "role": "enemy", "base_unit": "brute"}], "encounters": [encounter]});
    write_world(&data, "old", json!({"id": "old", "name": "Old", "version": 1}), kinds.clone());
    write_world(&data, "new", json!({"id": "new", "name": "New", "ruleset": "timeline", "version": 1, "classes": {"drill": {"name": "Vanguard", "description": "Holds the line.", "skill_path": {}}}}), kinds);
    let v = store::view(&data);
    let arena = "Arena hazards never strike here: the world's ruleset does not list the rule arena_hazards.";
    assert!(check::world::world_warnings(&v, "old").iter().any(|w| w.ends_with(arena)), "a classic world warns");
    assert!(!check::world::world_warnings(&v, "new").iter().any(|w| w.ends_with(arena)), "the timeline row lists the rule");
    assert_eq!(check::world::validate_entry(&v, "new", "encounters", &v.worlds.own_entries("new", "encounters")[0], "kiln"), Vec::<String>::new());
    assert_eq!(check::world::manifest_errors(&v, "new"), Vec::<String>::new());
    let mut failures = Vec::new();
    let cases: Vec<(Value, &str)> = vec![
        (json!({"drill": {"name": "Vanguard"}}), ""),
        (json!({"drill": {"name": "Vanguard", "skill_path": {"slam": 1, "brand": 3}}}), ""),
        (json!({"mage": {"name": "Alchemist"}}), "Class names: unknown class 'mage'."),
        (json!({"drill": {"name": ""}}), "Class names 'drill': the name needs 1 to 32 characters."),
        (json!({"drill": {"name": "Vanguard", "description": "x".repeat(241)}}), "Class names 'drill': the description is text of at most 240 characters."),
        (json!({"drill": {"name": "Vanguard", "skill_path": {"nothing": 1}}}), "Class names 'drill': skill path: unknown skill family 'nothing'."),
        (json!({"drill": {"name": "Vanguard", "skill_path": {"slam": 31}}}), "Class names 'drill': skill path slam: the level must be a whole number 1-30."),
        (json!({"drill": {"name": "Vanguard", "skill_path": ["slam"]}}), "Class names 'drill': skill_path must be an object"),
        (json!({"drill": {"name": "Vanguard", "proficiency": {}}}), "Class names 'drill': unknown key 'proficiency'"),
        (json!({"drill": "Vanguard"}), "Class names 'drill' must be an object"),
    ];
    for (classes, want) in &cases {
        let mut patch = Dict::new();
        patch.insert("classes".into(), classes.clone());
        let found = match store::world_manifest_text(&data, "new", &patch, false) {
            Ok(text) => {
                fs::write(data.join("worlds/new/world.json"), text).unwrap();
                check::world::manifest_errors(&store::view(&data), "new")
            }
            Err(e) => e,
        };
        let ok = if want.is_empty() { found.is_empty() } else { found.len() == 1 && found[0].contains(want) };
        if !ok {
            failures.push(format!("{classes}: want {want:?}, got {found:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // the deck phase effect in the classic world: refused (its ruleset has no deck rule)
    let v = store::view(&data);
    let mut e = v.worlds.own_entries("old", "encounters")[0].clone();
    e.insert("phases".into(), json!([{"id": "p", "name": "P", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute_c", "add": ["rush"]}]}]));
    assert_eq!(check::world::validate_entry(&v, "old", "encounters", &e, "kiln"), ["A deck phase effect needs the deck rule in the world's ruleset."]);
    let mut e = v.worlds.own_entries("new", "encounters")[0].clone();
    e.insert("phases".into(), json!([{"id": "p", "name": "P", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "deck", "unit": "brute_c", "add": ["rush"]}]}]));
    assert_eq!(check::world::validate_entry(&v, "new", "encounters", &e, "kiln"), Vec::<String>::new(), "a character plays as its base unit");
    drop_scratch(&data);
}

/// The writer keeps the hook fields (a skill's family fields once, `hp_cost` and `meters` as rank modifiers when they
/// differ), the kinds store every field, and a re-save writes the same text.
#[test]
fn the_writer_keeps_the_hook_fields_and_a_resave_is_identical() {
    let (_scratch, data) = scratch("writer");
    catalog(&data);
    let entry = json!({"id": "maul", "name": "Maul", "kind": "ATTACK", "hp_cost": 5, "forced_move": {"kind": "push", "tiles": 1}, "ignore_guard": true,
        "description": "{power} and a push.", "ranks": [{"hp_cost": 8}, {"meters": [{"meter": "fury", "delta": 1}]}]});
    store::upsert(&data, None, "skills", &dict(entry), None).expect("a valid ranked hook skill");
    let src = source::load(&data);
    assert_eq!(catalog_errors(&src), Vec::<String>::new());
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let ranks = cat.family_rows("maul");
    assert_eq!(ranks.iter().map(|s| s.hp_cost).collect::<Vec<_>>(), [5, 8, 5]);
    assert_eq!(ranks.iter().map(|s| s.meters.len()).collect::<Vec<_>>(), [0, 0, 1]);
    assert!(ranks.iter().all(|s| s.ignore_guard && s.forced_move.as_ref().is_some_and(|f| f.tiles == 1)), "family fields on every rank");
    for (kind, row) in [
        (HAZARDS_KIND, json!({"id": "smoke", "name": "Soot Smoke", "cover_guard": 0.3, "tint": "#6b6b73", "glyph": "s"})),
        (UNIT_METERS_KIND, json!({"id": "strain", "name": "Strain", "short": "Strain", "max": 100, "show": "bar"})),
        (ZONES_KIND, json!({"id": "canal", "name": "Canal Bridge", "ground": "bridge"})),
    ] {
        store::upsert(&data, None, kind, &dict(row), None).unwrap_or_else(|e| panic!("{kind}: {e:?}"));
    }
    for kind in ["skills", HAZARDS_KIND, UNIT_METERS_KIND, ZONES_KIND] {
        let file = store::file_of(&data, None, kind);
        assert_eq!(store::resave_text(&data, None, kind).unwrap(), fs::read_to_string(&file).unwrap(), "{kind}: a re-save is identical");
    }
    let stored: Value = serde_json::from_str(&fs::read_to_string(store::file_of(&data, None, HAZARDS_KIND)).unwrap()).unwrap();
    let smoke = stored["entries"].as_array().unwrap().iter().find(|e| e["id"] == "smoke").unwrap();
    let keys: Vec<String> = smoke.as_object().unwrap().keys().cloned().collect();
    let schema_keys: Vec<String> = fields(source::schema(), "content", HAZARDS_KIND).iter().map(|f| json::s(json::dict(f), "key")).filter(|k| k != "origin").collect();
    assert_eq!(keys, schema_keys, "a hazard row stores every field (origin system left out)");
    // a bad row is refused with its reason and the file stays as it was
    let before = fs::read_to_string(store::file_of(&data, None, ZONES_KIND)).unwrap();
    let refused = store::upsert(&data, None, ZONES_KIND, &dict(json!({"id": "swamp_bog", "name": "Bog"})), None).err().expect("a code zone id is refused");
    assert!(refused.iter().any(|m| m.contains("is a code zone")), "{refused:?}");
    assert_eq!(fs::read_to_string(store::file_of(&data, None, ZONES_KIND)).unwrap(), before);
    // an out-of-range number is refused with its reason, never clamped into range
    let refused = store::upsert(&data, None, HAZARDS_KIND, &dict(json!({"id": "slag", "name": "Slag", "damage": 60})), None).err().expect("damage 60 is refused");
    assert!(refused.iter().any(|m| m.contains("Damage must be 0-50 (is 60).")), "{refused:?}");
    let refused = store::upsert(&data, None, "skills", &dict(json!({"id": "bleeder", "name": "Bleeder", "kind": "ATTACK", "hp_cost": 120})), None).err().expect("hp_cost 120 is refused");
    assert!(refused.iter().any(|m| m.contains("HP cost must be 0-99 (is 120).")), "{refused:?}");
    drop_scratch(&data);
}
