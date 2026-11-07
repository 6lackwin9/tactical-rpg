//! CAMPAIGN1-U's kit side on scratch worlds (CAMPAIGN1.md W4, P1, M5, M8, F3-F5, W3): the HUD view gives today's two
//! counters outside a campaign world, and in one the purse's names, icons and tips, then the visible meters by
//! `hud.order` with their units, band tones and tips (the label replaces the name; the next rule point ahead); the
//! keyed act-start save writes and reads back but is refused while the run is over; a refused fight puts the party
//! back where it stepped from. Every expectation follows from the rule and the fixture's own numbers.

use aegis_core::json;
use rpgkit_world::campaign::view::{hud, next_point, with_unit, ChipKind};
use rpgkit_world::campaign::{self, RUN_OVER_TEXT};
use rpgkit_world::state::{PartyState, UnitState, SUPPLIES_MAX};
use rpgkit_world::{explore, save, Data, GameState};
use serde_json::{json, Value};

const SUSPICIOUS: i64 = 31;
const LOCKDOWN: i64 = 71;
const LAST: i64 = 90;
const WEEK: i64 = 7;
const CLOCK_DAYS: i64 = 37;

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

/// `alert` (a gauge in %, shown, HUD order 2), `clock` (a countdown in days, shown, order 1) and `hidden` (not shown).
fn meters() -> Value {
    json!([
        {"id": "alert", "name": "Inquisition Alert", "kind": "gauge", "min": 0, "max": 100, "unit": "%", "visible": true, "description": "How hard they look.",
         "hud": {"short": "Alert", "icon": "ui/hud/alert.png", "order": 2},
         "bands": [{"id": "guarded", "name": "Guarded", "from": 0, "tone": "calm"}, {"id": "suspicious", "name": "Suspicious", "from": SUSPICIOUS, "tone": "warn"},
                   {"id": "lockdown", "name": "Lockdown", "from": LOCKDOWN, "tone": "danger"}],
         "rules": [{"id": "a_sus", "at": "suspicious", "once": true, "warn": "Watched."}, {"id": "a_lock", "at": "lockdown", "warn": "Locked."},
                   {"id": "a_last", "at": LAST, "acts": [1], "warn": "Last."}]},
        {"id": "clock", "name": "The Red Tithing", "kind": "countdown", "min": 0, "max": 99, "unit": "days", "visible": true,
         "hud": {"short": "Tithing", "order": 1},
         "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}, {"id": "far", "name": "Far", "from": 8, "tone": "calm"}],
         "rules": [{"id": "week", "at": WEEK, "warn": "A week."}, {"id": "zero", "at": 0, "effects": [{"effect": "end", "kind": "fail", "card": "c"}]}]},
        {"id": "hidden", "name": "Hidden", "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}]}
    ])
}

/// World `classic` (no campaign) and `camp` (a campaign from act 1 with a purse and the three meters); both with a
/// two-tile map and a fight marker at (1, 0).
fn data() -> Data {
    let purse = json!({"currency": {"name": "Scrap", "icon": "ui/hud/scrap.png", "tip": "Salvaged iron."},
                       "upkeep": {"name": "Warded Tallow", "short": "Tallow", "icon": "ui/hud/tallow.png", "tip": "1 a day."}});
    let worlds = json!({
        "classic": {"manifest": {"id": "classic", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids("classic", json!([{"id": "hub", "name": "Hub"}]))}},
        "camp": {"manifest": {"id": "camp", "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}, "purse": purse}, "own": {
            "locations": with_ids("camp", json!([{"id": "hub", "name": "Hub"}])),
            "meters": with_ids("camp", meters())}}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data, w: &str) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, w, party)
}

/// Alert at `alert`, the clock running with `days` left.
fn set(gs: &mut GameState, alert: i64, days: i64) {
    let time = gs.time;
    let c = gs.campaign.as_mut().expect("a campaign");
    c.meters.get_mut("alert").unwrap().value = alert;
    let clock = c.meters.get_mut("clock").unwrap();
    clock.running = true;
    clock.due = time + days;
    c.sync(time);
}

#[test]
fn outside_a_campaign_world_the_hud_is_todays_two_counters() {
    let d = data();
    let gs = state(&d, "classic");
    let chips = hud(&d, &gs);
    let got: Vec<(ChipKind, String, String, String)> = chips.iter().map(|c| (c.kind, c.title.clone(), c.icon.clone(), c.value.clone())).collect();
    assert_eq!(
        got,
        vec![
            (ChipKind::Currency, rpgkit_data::CURRENCY_NAME.into(), rpgkit_data::CURRENCY_ICON.into(), gs.party.gold.to_string()),
            (ChipKind::Upkeep, rpgkit_data::UPKEEP_NAME.into(), rpgkit_data::UPKEEP_ICON.into(), format!("{}/{SUPPLIES_MAX}", gs.party.supplies)),
        ],
        "W4: the Ashlands' counters"
    );
    assert_eq!(chips[0].lines, vec![rpgkit_data::CURRENCY_TIP.to_string()]);
    assert_eq!(chips[1].lines, vec![rpgkit_data::UPKEEP_TIP.to_string()]);
}

#[test]
fn a_campaign_hud_is_the_purse_then_the_visible_meters_by_order() {
    let d = data();
    let mut gs = state(&d, "camp");
    set(&mut gs, 25, CLOCK_DAYS);
    let chips = hud(&d, &gs);
    let ids: Vec<&str> = chips.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["currency", "upkeep", "clock", "alert"], "P1, W4: the purse, then the meters by hud.order; `hidden` is not shown (M8)");
    assert_eq!((chips[0].title.as_str(), chips[0].icon.as_str(), chips[0].lines[0].as_str()), ("Scrap", "ui/hud/scrap.png", "Salvaged iron."));
    assert_eq!((chips[1].title.as_str(), chips[1].icon.as_str()), ("Warded Tallow", "ui/hud/tallow.png"));
    let (clock, alert) = (&chips[2], &chips[3]);
    assert_eq!((clock.short.as_str(), clock.value.as_str(), clock.tone.as_str()), ("Tithing", "37 days", "calm"), "a countdown's days left with its unit");
    assert_eq!((alert.short.as_str(), alert.value.as_str(), alert.tone.as_str(), alert.icon.as_str()), ("Alert", "25%", "calm", "ui/hud/alert.png"));
    assert_eq!(alert.title, "Inquisition Alert");
    assert_eq!(alert.lines, vec!["Band: Guarded".to_string(), format!("Next: {SUSPICIOUS}% (Suspicious)"), "How hard they look.".to_string()]);
    assert_eq!(clock.lines, vec!["Band: Far".to_string(), format!("Next: {WEEK} days")]);
    // the band moves with the value; a label replaces the name (K3)
    set(&mut gs, LOCKDOWN, 1);
    gs.campaign.as_mut().unwrap().meters.get_mut("clock").unwrap().label = "The Great Tithing".into();
    let chips = hud(&d, &gs);
    assert_eq!((chips[3].value.as_str(), chips[3].tone.as_str()), ("71%", "danger"));
    assert_eq!((chips[2].value.as_str(), chips[2].tone.as_str(), chips[2].title.as_str()), ("1 day", "danger", "The Great Tithing"));
    // a hidden meter shown (`meter_show`) joins the HUD: no `hud` row = order 0 and its name as the short text
    gs.campaign.as_mut().unwrap().meters.get_mut("hidden").unwrap().visible = true;
    let chips = hud(&d, &gs);
    let ids: Vec<&str> = chips.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["currency", "upkeep", "hidden", "clock", "alert"]);
    assert_eq!((chips[2].short.as_str(), chips[2].value.as_str()), ("Hidden", "0"));
}

#[test]
fn the_next_rule_point_skips_fired_once_rules_and_other_acts() {
    let d = data();
    let gs = state(&d, "camp");
    let c = gs.campaign.as_ref().unwrap();
    let alert = c.defs.get("alert").unwrap();
    assert_eq!(next_point(alert, 25, 1, false, &[]), Some(SUSPICIOUS));
    assert_eq!(next_point(alert, 25, 1, false, &["rule:alert/a_sus".to_string()]), Some(LOCKDOWN), "a once rule that fired is not ahead");
    assert_eq!(next_point(alert, LOCKDOWN, 1, false, &[]), Some(LAST));
    assert_eq!(next_point(alert, LOCKDOWN, 2, false, &[]), None, "a_last is an act-1 rule");
    let clock = c.defs.get("clock").unwrap();
    assert_eq!(next_point(clock, 10, 1, true, &[]), Some(WEEK));
    assert_eq!(next_point(clock, 5, 1, true, &[]), Some(0));
    assert_eq!(next_point(clock, 10, 1, false, &[]), None, "a stopped countdown has nothing ahead");
    assert_eq!((with_unit(3, "%"), with_unit(1, "days"), with_unit(2, "days"), with_unit(4, "")), ("3%".into(), "1 day".into(), "2 days".into(), "4".into()));
}

#[test]
fn the_act_start_save_reads_back_and_is_refused_while_the_run_is_over() {
    let d = data();
    let gs = state(&d, "camp");
    let dir = aegis_core::scratch::Scratch::vacant("rpgkit_campaign_view");
    let key = campaign::act_save_key(1, &gs.campaign.as_ref().unwrap().run);
    assert!(save::write_keyed(&d, &dir, &gs, &key, 1).is_empty());
    assert!(save::exists_keyed(&dir, &key));
    let back = save::read_keyed(&d, &dir, &key).expect("reads back");
    assert_eq!(back.state.campaign, gs.campaign, "F5: the same campaign block");
    for slot in save::AUTOSAVE_SLOT..=save::LAST_SLOT {
        assert!(!save::exists(&dir, slot), "F5: not a slot");
    }
    let mut over = gs.clone();
    campaign::apply_effects(&d, &mut over, &[json::dict(&json!({"effect": "end", "kind": "fail", "card": "c"})).clone()], &mut Vec::new());
    let other = campaign::act_save_key(2, &gs.campaign.as_ref().unwrap().run);
    assert_eq!(save::write_keyed(&d, &dir, &over, &other, 1), vec![RUN_OVER_TEXT.to_string()], "F3");
    assert!(!save::exists_keyed(&dir, &other));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_refused_fight_puts_the_party_back_where_it_stepped_from_and_restarts_the_counter() {
    let d = data();
    let mut gs = state(&d, "camp");
    gs.position = (1, 0);
    gs.explore.insert("steps".into(), json!(5));
    let fight = json::dict(&json!({"kind": "fight", "source": "marker", "encounter": "camp:x", "from": [0, 0]})).clone();
    // a flee's end (explore::after_fight "fled"): back to `from`, then the encounter counter starts over
    let mut fled = gs.clone();
    fled.position = (0, 0);
    explore::reset_counter(&d, &mut fled);
    explore::decline_fight(&d, &mut gs, &fight);
    assert_eq!(gs.position, (0, 0), "W3: back to `from`");
    assert_eq!(json::i(&gs.explore, "steps", -1), 0, "the counter starts over");
    assert_eq!(gs.to_value(), fled.to_value(), "as after fleeing, nothing settled");
    let mut no_from = gs.clone();
    explore::decline_fight(&d, &mut no_from, &json::dict(&json!({"kind": "fight"})).clone());
    assert_eq!(no_from.position, gs.position, "no `from`: the party stays");
}

#[test]
fn an_empty_purse_tip_adds_no_hover_line() {
    let worlds = json!({"bare": {"manifest": {"id": "bare", "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1},
        "purse": {"currency": {"name": "Scrap", "tip": ""}, "upkeep": {"name": "Tallow", "tip": ""}}}, "own": {
        "locations": with_ids("bare", json!([{"id": "hub", "name": "Hub"}]))}}});
    let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
    let d = Data::from_values(&catalog, &worlds, &json!({}));
    let chips = hud(&d, &state(&d, "bare"));
    assert_eq!((chips[0].title.as_str(), chips[1].title.as_str()), ("Scrap", "Tallow"));
    assert!(chips[0].lines.is_empty() && chips[1].lines.is_empty(), "W4: no empty tip line");
}
