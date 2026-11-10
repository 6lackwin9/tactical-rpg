//! RULESET1-D on a scratch world (no shipped data): a party member's deck (RULESET1 9 D3-D5, RULESET1_TASKS 1.5,
//! 1.8). The member saves `deck` only when it has one, in the unreleased save v13 (no format step: a save without it
//! reads the empty deck); a damaged deck is refused, never coerced; the deck rules refuse misfiled or unowned entries;
//! `progression::build_in` slots the member's own passives and fills the active slots from its deck (else its base
//! unit's), while `build` stays classic.

use rpgkit_data::Deck;
use rpgkit_rules::ruleset::Ruleset;
use rpgkit_rules::DeckRule;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{dialogue, progression, save, Data, GameState};
use serde_json::{json, Value};

/// The class path, in path order (every family unlocks at level 1).
const PATH: [&str; 5] = ["d_one", "d_two", "d_three", "d_four", "d_ward"];
const SLOTS: (i32, i32, i32) = (3, 2, 1);
/// The STR the two slottable passives give (the dialogue check's case).
const BRACED: i64 = 7;
const SPARE: i64 = 11;

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let mut r = r.as_object().unwrap().clone();
            let id = r["id"].as_str().unwrap().to_string();
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

/// A class with a five-family path, the timeline basics, a learned-only attack nobody owns, passives of each slot kind
/// (and one source-granted), a party unit without a deck and one with a deck, the timeline ruleset with the deck rule.
fn data() -> Data {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    let path: serde_json::Map<String, Value> = PATH.iter().map(|f| (f.to_string(), json!(1))).collect();
    let catalog = json!({
        // CLASS-BASICS: the second class's basic is the learned-only attack
        "classes": [{"id": "drill", "name": "Drill", "skill_path": path}, {"id": "drill_cut", "name": "Drill Cut", "skill_path": path, "basics": ["d_far"]}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2},
            atk("d_one"), atk("d_two"), atk("d_three"), atk("d_four"),
            {"id": "d_ward", "name": "d_ward", "kind": "DEFEND", "level": 1, "blocks": 2},
            {"id": "d_far", "name": "d_far", "kind": "ATTACK", "level": 1, "blocks": 1, "learned_only": true, "target_mode": "ENEMY"}
        ],
        "passives": [
            {"id": "d_braced", "name": "Braced", "slot": "passive", "effects": [{"type": "stat", "stat": "STR", "value": BRACED}]},
            {"id": "d_spare", "name": "Spare", "slot": "passive", "effects": [{"type": "stat", "stat": "STR", "value": SPARE}]},
            {"id": "d_anvil", "name": "Anvil", "slot": "keystone", "effects": []},
            {"id": "d_innate", "name": "Innate", "effects": []}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "hero_decked", "name": "Decked", "side": "party", "class_id": "drill", "level": 1, "deck": {"active": ["d_three"]}},
            {"id": "hero_cut", "name": "Cut", "side": "party", "class_id": "drill_cut", "level": 1}
        ],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck"], "basics": ["move", "rest", "strike", "active_guard"],
            "deck_slots": {"active": SLOTS.0, "passive": SLOTS.1, "keystone": SLOTS.2}}]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_location": "camp", "start_party": ["hero"]}, "own": {
            "locations": with_ids("w", json!([{"id": "camp", "name": "Camp"}]))
        }},
        "tw": {"manifest": {"id": "tw", "start_location": "camp", "start_party": ["hero"], "ruleset": "timeline"}, "own": {
            "locations": with_ids("tw", json!([{"id": "camp", "name": "Camp"}]))
        }}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn strings(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// A member of `base` owning the three slottable passives' first two and the source-granted one.
fn member(base: &str) -> UnitState {
    let mut m = UnitState::create("hero", base);
    m.passives = strings(&["d_braced", "d_spare", "d_innate"]);
    m
}

fn state(d: &Data, m: UnitState) -> GameState {
    let mut party = PartyState::default();
    party.members.push(m);
    GameState::create(d, "w", party)
}

fn envelope(version: i64, game_state: &Value) -> String {
    json!({"format": save::FORMAT, "version": version, "saved_at": 1, "game_state": game_state}).to_string()
}

fn deck(v: Value) -> Deck {
    Deck::from_value(&v)
}

fn rule(d: &Data) -> DeckRule {
    DeckRule::of(&Ruleset::of(&d.cat, "timeline").expect("the row")).expect("the deck rule")
}

/// D5, 1.8: no deck, no key (a classic member's save is unchanged); a deck saves its non-empty keys and reads back;
/// a save of the version before reads the empty deck.
#[test]
fn a_member_deck_saves_only_when_used_and_round_trips() {
    let d = data();
    let gs = state(&d, member("hero"));
    assert!(!gs.to_value()["party"]["members"][0].as_object().unwrap().contains_key("deck"));
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_four", "d_two"], "passive": ["d_braced"], "keystone": [], "order": {"assault": ["d_two", "strike"], "guard": []}}));
    let v = state(&d, m).to_value();
    assert_eq!(v["party"]["members"][0]["deck"], json!({"active": ["d_four", "d_two"], "passive": ["d_braced"], "order": {"assault": ["d_two", "strike"]}}), "only non-empty keys");
    let loaded = save::decode(&d, &envelope(save::VERSION, &v)).expect("loads");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.party.members[0].deck, deck(v["party"]["members"][0]["deck"].clone()));
    let old = state(&d, member("hero")).to_value();
    let loaded = save::decode(&d, &envelope(save::VERSION, &old)).expect("a save without a deck loads");
    assert!(loaded.state.party.members[0].deck.is_empty() && loaded.warnings.is_empty());
}

/// "Damaged blocks refused, never coerced": a deck of the wrong shape refuses the save with its problem.
#[test]
fn a_damaged_deck_is_refused() {
    let d = data();
    let base = state(&d, member("hero")).to_value();
    let damaged: Vec<(Value, &str)> = vec![
        (json!([1]), "deck must be an object {active, passive, keystone, order, follow_ups, marrow}."),
        (json!({"follow_ups": ["d_one"]}), "deck.follow_ups must be an object {opener family: [skill families]}."),
        (json!({"follow_ups": {"d_one": "d_two"}}), "deck.follow_ups.d_one must be a list of text."),
        (json!({"active": "d_one"}), "deck.active must be a list of text."),
        (json!({"passive": [1]}), "deck.passive must be a list of text."),
        (json!({"order": ["d_one"]}), "deck.order must be an object {intent: [skill families]}."),
        (json!({"order": {"assault": "d_one"}}), "deck.order.assault must be a list of text."),
        (json!({"slots": []}), "deck: unknown key 'slots'."),
    ];
    for (block, want) in damaged {
        let mut v = base.clone();
        v["party"]["members"][0]["deck"] = block.clone();
        let errors = save::decode(&d, &envelope(save::VERSION, &v)).err().unwrap_or_else(|| panic!("{block} refused"));
        assert_eq!(errors, [format!("Unit 'hero': {want}")], "{block}");
    }
}

/// D1-D5 (the content check's unit deck rules, ownership from the member): basics and unowned families never take
/// an active slot; passives sit in their own slot kind and must be the member's; the counts fit the ruleset's slots;
/// order lists name actives or basics of their intent.
#[test]
fn the_deck_rules_refuse_misfiled_or_unowned_entries() {
    let d = data();
    let cases: Vec<(Value, &str)> = vec![
        (json!({"active": ["strike"]}), "deck active: 'strike' is a basic (open to every unit, no slot)."),
        (json!({"active": ["active_guard"]}), "deck active: 'active_guard' is a basic (open to every unit, no slot)."),
        (json!({"active": ["d_nope"]}), "deck active: unknown skill family 'd_nope'."),
        (json!({"active": ["d_far"]}), "deck active: family 'd_far' is not owned."),
        (json!({"active": ["d_one", "d_one"]}), "deck active lists 'd_one' twice."),
        (json!({"passive": ["d_anvil"]}), "deck passive: 'd_anvil' does not take the passive slot."),
        (json!({"keystone": ["d_innate"]}), "deck keystone: 'd_innate' does not take the keystone slot."),
        (json!({"passive": ["d_ghost"]}), "deck passive: unknown passive 'd_ghost'."),
        (json!({"keystone": ["d_anvil"]}), "deck keystone: passive 'd_anvil' is not owned."),
        (json!({"active": ["d_one", "d_two", "d_three", "d_four"]}), "deck: 4 active entries (the Timeline ruleset's deck slots allow 3)."),
        (json!({"order": {"dance": ["strike"]}}), "deck order: 'dance' is not an intent (one of assault, guard, support, maneuver)."),
        (json!({"active": ["d_one"], "order": {"assault": ["d_two"]}}), "deck order assault: 'd_two' is neither an active of the deck nor a basic."),
        (json!({"active": ["d_ward"], "order": {"assault": ["d_ward"]}}), "deck order assault: 'd_ward' is of intent guard."),
        (json!({"order": {"assault": ["strike", "strike"]}}), "deck order lists 'strike' twice."),
    ];
    assert_eq!(SLOTS.0, 3, "the count case above assumes three active slots");
    for (v, want) in cases {
        let mut m = member("hero");
        m.deck = deck(v.clone());
        assert_eq!(m.validate(&d), [format!("Unit 'hero': {want}")], "{v}");
    }
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_four", "d_ward", "d_two"], "passive": ["d_braced", "d_spare"], "order": {"assault": ["d_two", "strike", "d_four"], "guard": ["active_guard", "d_ward"]}}));
    assert!(m.validate(&d).is_empty(), "{:?}", m.validate(&d));
}

/// D2-D4 in the member build: under the rule only the member's slotted passives and its source-granted one apply and
/// its deck fills the active slots (an empty deck: its base unit's deck, else the class path's first families);
/// `build` (classic) applies every passive and keeps the loadout.
#[test]
fn build_in_slots_the_members_passives_and_actives() {
    let d = data();
    let rule = rule(&d);
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_four", "d_two"], "passive": ["d_braced"]}));
    let u = progression::build_in(&d, &m, "", Some(&rule)).expect("built").unit;
    assert_eq!(u.passives, strings(&["d_braced", "d_innate"]), "d_spare is not slotted");
    assert_eq!(u.active_families(), strings(&["d_four", "d_two"]));
    let classic = progression::build(&d, &m).expect("built").unit;
    assert_eq!(classic.passives, strings(&["d_braced", "d_spare", "d_innate"]));
    assert_eq!(classic.active_families(), strings(&PATH));
    assert!(classic.deck_rule.is_none());
    // auto: the first class-path families; an empty member deck takes its base unit's
    let u = progression::build_in(&d, &member("hero"), "", Some(&rule)).expect("built").unit;
    assert_eq!(u.active_families(), strings(&PATH[..SLOTS.0 as usize]));
    assert_eq!(u.passives, strings(&["d_innate"]));
    let u = progression::build_in(&d, &member("hero_decked"), "", Some(&rule)).expect("built").unit;
    assert_eq!(u.active_families(), strings(&["d_three"]));
    let mut m = member("hero_decked");
    m.deck = deck(json!({"active": ["d_one"]}));
    assert_eq!(progression::build_in(&d, &m, "", Some(&rule)).expect("built").unit.active_families(), strings(&["d_one"]));
    // None is `build`
    let (a, b) = (progression::build_in(&d, &member("hero"), "", None).unwrap().unit, progression::build(&d, &member("hero")).unwrap().unit);
    assert_eq!((&a.passives, a.active_families(), &a.known_skills), (&b.passives, b.active_families(), &b.known_skills));
}

/// The RULESET1-D review (D3 out of battle): a dialogue stat check builds the members as the played world fights, so
/// under its deck rule an unslotted passive does not count; a classic world counts every passive (`build` exactly).
#[test]
fn a_dialogue_stat_check_counts_only_slotted_passives_under_the_deck_rule() {
    let d = data();
    assert!(d.ruleset("w").is_classic() && !d.ruleset("tw").is_classic(), "w classic, tw timeline");
    let mut m = member("hero");
    m.deck = deck(json!({"passive": ["d_braced"]}));
    let str_of = |rule: Option<&DeckRule>| {
        let u = progression::build_in(&d, &m, "", rule).expect("built").unit;
        let ctx = u.weapon_context();
        u.stats.effective("STR", ctx.as_deref(), "") as i64
    };
    let (classic, decked) = (str_of(None), str_of(Some(&rule(&d))));
    assert_eq!(classic - decked, SPARE, "only the unslotted passive's STR is gone under the deck rule");
    let mut gs = state(&d, m.clone());
    assert_eq!(dialogue::best_stat(&d, &gs, "STR"), classic, "a classic world counts every passive");
    gs.world_id = "tw".into();
    assert_eq!(dialogue::best_stat(&d, &gs, "STR"), decked, "the deck world counts the slotted ones only");
}

/// INPUT1 FL4 (Q9 a): `follow_ups` sits inside the saved `deck`, written only when a list is set; a v13 save without it
/// loads with no lists and re-saves byte-identical; a set list round-trips in its written order; an empty one is no
/// list (dropped on save).
#[test]
fn follow_up_lists_save_inside_the_deck_only_when_set() {
    let d = data();
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_one", "d_two"], "order": {"assault": ["d_two", "strike"]}}));
    let v = state(&d, m.clone()).to_value();
    assert!(!v["party"]["members"][0]["deck"].as_object().unwrap().contains_key("follow_ups"), "no list, no key");
    let text = envelope(save::VERSION, &v);
    let loaded = save::decode(&d, &text).expect("a save without follow_ups loads");
    assert!(loaded.warnings.is_empty() && loaded.state.party.members[0].deck.follow_ups.is_empty());
    assert_eq!(envelope(save::VERSION, &loaded.state.to_value()), text, "re-saved byte-identical");
    m.deck.follow_ups.insert("d_one".into(), strings(&["d_two", "strike", "d_one"]));
    m.deck.follow_ups.insert("d_two".into(), Vec::new());
    let v = state(&d, m).to_value();
    assert_eq!(v["party"]["members"][0]["deck"]["follow_ups"], json!({"d_one": ["d_two", "strike", "d_one"]}), "only the set list");
    let loaded = save::decode(&d, &envelope(save::VERSION, &v)).expect("loads");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let back = &loaded.state.party.members[0].deck;
    assert_eq!(back.follow_ups_of("d_one"), Some(&strings(&["d_two", "strike", "d_one"])));
    assert!(back.follow_ups_of("d_two").is_none());
}

/// FL5 for a member's deck (the deck screen's Confirm): a list follows an opener of the deck with families of the
/// deck of the opener's intent, none twice.
#[test]
fn the_deck_rules_refuse_a_follow_up_list_of_another_intent_or_outside_the_deck() {
    let d = data();
    let cases: Vec<(Value, &str)> = vec![
        (json!({"active": ["d_one", "d_ward"], "follow_ups": {"d_one": ["d_ward"]}}), "Follow-ups after 'd_one': 'd_ward' is Guard, not Assault."),
        (json!({"active": ["d_one"], "follow_ups": {"d_one": ["d_two"]}}), "Follow-ups after 'd_one': 'd_two' is not in the deck."),
        (json!({"active": ["d_one"], "follow_ups": {"d_one": ["strike", "strike"]}}), "Follow-ups after 'd_one': 'strike' twice."),
        (json!({"active": ["d_one"], "follow_ups": {"d_two": ["strike"]}}), "Follow-ups for 'd_two': not an opener in the deck."),
        (json!({"active": ["d_one"], "follow_ups": {"move": ["strike"]}}), "Follow-ups for 'move': not an opener in the deck."),
    ];
    for (v, want) in cases {
        let mut m = member("hero");
        m.deck = deck(v.clone());
        assert_eq!(m.validate(&d), [format!("Unit 'hero': {want}")], "{v}");
    }
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_one", "d_ward"], "follow_ups": {"d_one": ["strike", "d_one"], "d_ward": ["active_guard"]}}));
    assert!(m.validate(&d).is_empty(), "{:?}", m.validate(&d));
}

/// CLASS-BASICS (R2-W2b): a member's class basics are basics of its deck: never an active, orderable, open without a
/// slot in the member build; a member of another class may not order them.
#[test]
fn a_members_class_basics_are_open_basics_of_its_deck() {
    let d = data();
    let mut m = member("hero_cut");
    m.deck = deck(json!({"active": ["d_far"]}));
    assert_eq!(m.validate(&d), ["Unit 'hero': deck active: 'd_far' is a basic (open to every unit, no slot)."]);
    m.deck = deck(json!({"order": {"assault": ["d_far", "strike"]}}));
    assert!(m.validate(&d).is_empty(), "{:?}", m.validate(&d));
    let mut other = member("hero");
    other.deck = deck(json!({"order": {"assault": ["d_far"]}}));
    assert_eq!(other.validate(&d), ["Unit 'hero': deck order assault: 'd_far' is neither an active of the deck nor a basic."]);
    let u = progression::build_in(&d, &member("hero_cut"), "", Some(&rule(&d))).expect("built").unit;
    assert_eq!(u.active_families(), strings(&PATH[..SLOTS.0 as usize]), "the class basic takes no slot");
    assert!(u.can_use(&d.cat, d.cat.skill("d_far").unwrap()), "open");
    let plain = progression::build_in(&d, &member("hero"), "", Some(&rule(&d))).expect("built").unit;
    assert!(!plain.can_use(&d.cat, d.cat.skill("d_far").unwrap()), "closed for another class");
}

/// R4-2 M1 (FB-138): a fixture save whose member decks hold entries the current data disallows (a skill that became
/// the class's basic, an unowned family, an unknown passive, more actives than the slots, order and follow-up entries
/// outside the kept deck, an unknown intent) loads: each bad entry is dropped with a load warning, the order and
/// follow-ups are read against the kept entries, a list past the slots is cut to them, and the re-save loads clean. A
/// valid deck and a damaged one (`a_damaged_deck_is_refused`) are untouched by the drop.
#[test]
fn a_saved_deck_the_data_no_longer_allows_loads_without_the_bad_entries() {
    let d = data();
    let mut hero = member("hero");
    hero.deck = deck(json!({
        "active": ["d_one", "strike", "d_far", "d_two", "d_three", "d_four"],
        "passive": ["d_braced", "d_ghost"],
        "order": {"assault": ["d_one", "d_four", "strike"], "dance": ["strike"]},
        "follow_ups": {"d_far": ["strike"], "d_one": ["d_two", "d_four"]}
    }));
    let mut cut = UnitState::create("cut", "hero_cut");
    cut.deck = deck(json!({"active": ["d_far", "d_one"], "order": {"assault": ["d_far", "d_one"]}}));
    let mut gs = state(&d, hero);
    gs.party.members.push(cut);
    let text = envelope(save::VERSION, &gs.to_value());
    let loaded = save::decode(&d, &text).unwrap_or_else(|e| panic!("the save loads: {e:?}"));
    let (hero, cut) = (&loaded.state.party.members[0], &loaded.state.party.members[1]);
    assert_eq!(SLOTS.0, 3, "the cut below assumes three active slots");
    assert_eq!(hero.deck, deck(json!({"active": ["d_one", "d_two", "d_three"], "passive": ["d_braced"], "order": {"assault": ["d_one", "strike"]}, "follow_ups": {"d_one": ["d_two"]}})));
    assert_eq!(cut.deck, deck(json!({"active": ["d_one"], "order": {"assault": ["d_far", "d_one"]}})), "the class basic leaves the actives and stays orderable");
    assert!(hero.validate(&d).is_empty() && cut.validate(&d).is_empty(), "valid after the drop");
    let dropped = |unit: &str, problem: &str| format!("Unit '{unit}': {problem} Dropped from the saved deck (the data no longer allows it).");
    for want in [
        dropped("hero", "deck active: 'strike' is a basic (open to every unit, no slot)."),
        dropped("hero", "deck active: family 'd_far' is not owned."),
        dropped("hero", "deck passive: unknown passive 'd_ghost'."),
        dropped("hero", "deck: 4 active entries (the Timeline ruleset's deck slots allow 3)."),
        dropped("hero", "deck order assault: 'd_four' is neither an active of the deck nor a basic."),
        dropped("hero", "deck order: 'dance' is not an intent (one of assault, guard, support, maneuver)."),
        dropped("hero", "Follow-ups for 'd_far': not an opener in the deck."),
        dropped("hero", "Follow-ups after 'd_one': 'd_four' is not in the deck."),
        dropped("cut", "deck active: 'd_far' is a basic (open to every unit, no slot)."),
    ] {
        assert!(loaded.warnings.contains(&want), "warned: {want}\n{:#?}", loaded.warnings);
    }
    assert!(loaded.warnings.iter().all(|w| w.ends_with("Dropped from the saved deck (the data no longer allows it).")), "{:#?}", loaded.warnings);
    let again = save::decode(&d, &envelope(save::VERSION, &loaded.state.to_value())).expect("the re-save loads");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
    assert_eq!(again.state.party.members[0].deck, hero.deck);
    // a valid deck loads as saved, without a warning
    let mut ok = member("hero");
    ok.deck = deck(json!({"active": ["d_four", "d_ward", "d_two"], "passive": ["d_braced"], "order": {"assault": ["d_two", "strike"]}}));
    let kept = save::decode(&d, &envelope(save::VERSION, &state(&d, ok.clone()).to_value())).expect("loads");
    assert!(kept.warnings.is_empty() && kept.state.party.members[0].deck == ok.deck);
}

/// PK1-S2a (PK1 4.2, rule channel): a deck's `marrow` marks save only when set and read back; a mark must be an active
/// of the deck (a damaged list refuses the save); a deck without marks saves as before.
#[test]
fn a_marrow_mark_saves_and_is_checked() {
    let d = data();
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_one", "d_two"], "marrow": ["d_one"]}));
    let v = state(&d, m).to_value();
    assert_eq!(v["party"]["members"][0]["deck"], json!({"active": ["d_one", "d_two"], "marrow": ["d_one"]}));
    let loaded = save::decode(&d, &envelope(save::VERSION, &v)).expect("loads");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.party.members[0].deck.marrow, ["d_one"]);
    let mut m = member("hero");
    m.deck = deck(json!({"active": ["d_one"], "marrow": ["d_two"]}));
    assert_eq!(m.validate(&d), ["Unit 'hero': deck marrow: 'd_two' is not an active of the deck."]);
    let mut damaged = state(&d, member("hero")).to_value();
    damaged["party"]["members"][0]["deck"] = json!({"active": ["d_one"], "marrow": [1]});
    assert_eq!(save::decode(&d, &envelope(save::VERSION, &damaged)).err().expect("refused"), ["Unit 'hero': deck.marrow must be a list of text."]);
}
