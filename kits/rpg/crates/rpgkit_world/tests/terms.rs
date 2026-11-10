//! TERMS1-L's resolver on scratch worlds (TERMS1.md TM2-TM6, TU1): with no purse block and no `terms` row every term is
//! the register's default (today's texts); a campaign world's purse names the currency and the upkeep (the short form
//! as the sentence word unless the name is the default); a `terms` row renames an intent; the kit's shop, quest and
//! dialogue texts read the world's words. Every expectation follows from the register, the rule and the fixture.

use aegis_core::json::{self, Dict};
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_world::state::{PartyState, UnitState, GOLD_MAX, SUPPLIES_MAX};
use rpgkit_world::terms::{self, Term, TermScope, CURRENCY, REGISTER, UPKEEP};
use rpgkit_world::{dialogue, progression, quest, shop, Data, GameState};
use serde_json::{json, Value};

const SUPPLY_PRICE: i64 = 5;

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = json::arr(&rows)
        .iter()
        .map(|r| {
            let mut r = json::dict(r).clone();
            let id = json::s(&r, "id");
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

/// One world's manifest and own rows: a hub and a shop selling supplies.
fn world(id: &str, extra: Value) -> Value {
    let mut manifest = json!({"id": id, "start_location": "hub", "start_party": ["hero"]});
    for (k, v) in json::dict(&extra) {
        manifest[k] = v.clone();
    }
    json!({"manifest": manifest, "own": {
        "locations": with_ids(id, json!([{"id": "hub", "name": "Hub"}])),
        "shops": with_ids(id, json!([{"id": "store", "name": "Store", "supply_price": SUPPLY_PRICE}]))}})
}

/// World `plain` (no campaign), `camp` (a campaign whose purse names Scrap and Lamp Oil / Oil) and `goldish` (a
/// campaign whose purse repeats the default names); the catalog's `terms` rows are `terms`.
fn data_with(terms: Value) -> Data {
    let campaign = json!({"start_act": 1});
    let worlds = json!({
        "plain": world("plain", json!({})),
        "camp": world("camp", json!({"campaign": campaign, "purse": {"currency": {"name": "Scrap", "tip": "Salvaged iron."}, "upkeep": {"name": "Lamp Oil", "short": "Oil"}}})),
        "goldish": world("goldish", json!({"campaign": campaign, "purse": {"currency": {"name": "Gold"}, "upkeep": {"name": "Supplies"}}})),
    });
    let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}], "terms": terms});
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn data() -> Data {
    data_with(json!([]))
}

fn state(d: &Data, w: &str) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, w, party);
    gs.party.gold = 0;
    gs
}

fn errors(r: &Dict) -> Vec<String> {
    json::sget(r, "errors")
}

#[test]
fn with_no_data_every_term_is_its_default() {
    let d = data();
    for t in REGISTER {
        let want = Term { name: t.name.into(), short: t.short.into(), word: t.word.into(), tip: t.tip.into() };
        assert_eq!(terms::term(&d, "plain", t.id), want, "TM2: '{}' outside a campaign world", t.id);
        assert_eq!(d.cat.term(t.id), want, "TM2: '{}' through the battle catalog", t.id);
        assert_eq!(t.short, t.name, "TM5: a default short is its name");
        assert_eq!(t.word, t.name.to_lowercase(), "today's in-sentence words are the names in lower case");
    }
    assert_eq!(terms::term(&d, "plain", CURRENCY).name, rpgkit_data::CURRENCY_NAME, "the register points at the purse defaults");
    assert_eq!(terms::term(&d, "plain", UPKEEP).tip, rpgkit_data::UPKEEP_TIP);
    assert_eq!(terms::ids(TermScope::World), [CURRENCY, UPKEEP], "Q-T6: the spec's seven terms, two named per world");
    // PK1-S2b: one game term per intent, then the marrow mark; PK1-S2c-2: then the Cold Steel mark
    assert_eq!(terms::ids(TermScope::Game).len(), RoundIntent::ALL.len() + 2, "one game term per intent, the marrow mark and the Cold Steel mark");
    let game = terms::ids(TermScope::Game);
    assert_eq!(game[game.len() - 2..], [rpgkit_data::terms::MARROW_MARK, rpgkit_data::terms::COLD_STEEL_MARK]);
    for i in RoundIntent::ALL {
        assert_eq!(i.name(), terms::default_name(i.term_id()), "RoundIntent::name() is the register's default");
        assert_eq!(i.label(&d.cat), i.name(), "no terms row: the label is the default name");
    }
}

#[test]
fn a_purse_world_names_the_currency_and_the_upkeep() {
    let d = data();
    let cur = terms::term(&d, "camp", CURRENCY);
    assert_eq!((cur.name.as_str(), cur.short.as_str(), cur.word.as_str(), cur.tip.as_str()), ("Scrap", "Scrap", "Scrap", "Salvaged iron."), "TM4, TM5: a lore name keeps its case");
    let up = terms::term(&d, "camp", UPKEEP);
    assert_eq!((up.name.as_str(), up.short.as_str(), up.word.as_str()), ("Lamp Oil", "Oil", "Oil"), "TM5: the word of a renamed term is its short form");
    assert_eq!(up.tip, rpgkit_data::UPKEEP_TIP, "TM4: an unset tip is the default");
    let g = terms::term(&d, "goldish", CURRENCY);
    assert_eq!((g.short.as_str(), g.word.as_str()), ("Gold", "gold"), "TM5: the default name keeps today's lower-case word");
    assert_eq!(terms::word(&d, "goldish", UPKEEP), "supplies");
    assert_eq!(terms::term(&d, "", CURRENCY), Term::default_of(CURRENCY), "no world: the default");
}

#[test]
fn a_terms_row_renames_an_intent() {
    let d = data_with(json!([{"id": "intent_guard", "name": "Bulwark"}, {"id": "intent_assault", "name": "Onslaught", "short": "Push", "word": "onslaught", "tip": "Hit hard."}]));
    let guard = terms::term(&d, "plain", terms::INTENT_GUARD);
    assert_eq!((guard.name.as_str(), guard.short.as_str(), guard.word.as_str()), ("Bulwark", "Bulwark", "Bulwark"), "TM5: short = name, word = short");
    assert_eq!(terms::term(&d, "camp", terms::INTENT_GUARD), guard, "a game term is the same in every world");
    let a = d.cat.term(terms::INTENT_ASSAULT);
    assert_eq!((a.name.as_str(), a.short.as_str(), a.word.as_str(), a.tip.as_str()), ("Onslaught", "Push", "onslaught", "Hit hard."));
    assert_eq!(RoundIntent::Guard.label(&d.cat), "Bulwark", "TM3: the battle reads Catalog::term");
    assert_eq!(RoundIntent::Guard.name(), "Guard", "name() stays the default");
    assert_eq!(RoundIntent::Support.label(&d.cat), "Support", "an intent without a row keeps its default");
    assert_eq!(terms::term(&d, "plain", CURRENCY).name, rpgkit_data::CURRENCY_NAME, "a world term never reads the table");
}

#[test]
fn the_sentences_take_the_forms_of_tm6() {
    let d = data();
    assert_eq!(terms::currency_capped(&d, "plain", 5), format!("Gold is capped at {GOLD_MAX} (5 not added)."), "today's text");
    assert_eq!(terms::currency_capped(&d, "camp", 5), format!("Scrap is capped at {GOLD_MAX} (5 not added)."), "a sentence that starts with the term: short");
    assert_eq!(terms::upkeep_capped(&d, "plain", 3), format!("Supplies are capped at {SUPPLIES_MAX} (3 left behind)."), "today's text while the name is the default");
    assert_eq!(terms::upkeep_capped(&d, "goldish", 3), terms::upkeep_capped(&d, "plain", 3));
    assert_eq!(terms::upkeep_capped(&d, "camp", 3), format!("Oil: capped at {SUPPLIES_MAX} (3 left behind)."), "TM6: no verb to agree with a renamed upkeep");
    assert_eq!(terms::not_enough_currency(&d, "plain"), "Not enough gold.");
    assert_eq!(terms::upkeep_carry_max(&d, "camp"), format!("You can carry at most {SUPPLIES_MAX} Oil."));
    assert_eq!(progression::REFUSE_SUPPLIES, format!("Making camp needs {} {}.", progression::CAMP_COST, terms::word(&d, "plain", UPKEEP)), "the Camp refusal (no campaign) is the default word");
}

#[test]
fn shop_quest_and_dialogue_texts_read_the_worlds_words() {
    let d = data();
    for (w, word, up) in [("plain", "gold", "supplies"), ("camp", "Scrap", "Oil")] {
        let mut gs = state(&d, w);
        assert!(shop::is_open(&d, &gs, "store"), "the fixture's shop is open in {w}");
        let r = shop::buy(&d, &mut gs, "store", shop::SUPPLIES, shop::SUPPLIES, 1);
        assert_eq!(errors(&r), [format!("Not enough {word}.")], "TU1: shop buy in {w}");
        let r = shop::trade(&d, &mut gs, "store", &[(shop::SUPPLIES.into(), shop::SUPPLIES.into(), 1)], &[]);
        assert_eq!(errors(&r), [format!("Not enough {word}.")], "TU1: shop trade in {w}");
        gs.party.supplies = SUPPLIES_MAX;
        gs.party.gold = SUPPLY_PRICE;
        let r = shop::buy(&d, &mut gs, "store", shop::SUPPLIES, shop::SUPPLIES, 1);
        assert_eq!(errors(&r), [format!("You can carry at most {SUPPLIES_MAX} {up}.")], "TU1: the supply cap in {w}");
        assert_eq!(quest::take(&d, &mut gs, &json!({}), SUPPLY_PRICE + 1), [format!("Not enough {word}.")], "TU1: quest take in {w}");
        let choice = json::obj([("requires", json!([{"kind": "gold", "min": 10}]))]);
        assert_eq!(dialogue::lock_reason(&d, &gs, &choice), format!("Needs 10 {word}"), "TU1: a dialogue lock in {w}");
    }
}
