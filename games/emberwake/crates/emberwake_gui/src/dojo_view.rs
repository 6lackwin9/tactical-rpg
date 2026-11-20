//! UISPLIT1-J (ENGINE-25) Stage A: the dojo's view-model (`dojo::Dojo`, the training ground). What the dojo shows and
//! offers, as a `screen_layout::View`, for a layout file to place later (Stage B); today's hand drawing (its windows)
//! stays. Its fixture is `ui/views/dojo.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title`; `heroes` (a row per hero, id its index: the name, "HP a/b  AP c/d  Stance" as the sub, the active
//! one selected); `carry` (the carry line, `foe` when overloaded); `stage` (the hero's and the dummy's name with "HP
//! a/b" as the value and their second stage line as the next item, toned `gold` and `foe` on the names); `round`
//! ("Round N"); `gear` (a row per slot, id the slot key, on while worn: a click takes it off); `pouch` (a row per pouch
//! slot, id its index, on while filled, with an Unload button); `stash_tabs` and `stash` (the filter, and a row per
//! shown stash item, id the item: "Name xN", its hover line as the note, dim when it can be neither worn nor pouched);
//! `log_tabs` and `log` (the log filter, and a row per shown entry, id its number: the head, the detail line as the sub,
//! the category's tone, the picked one selected); `breakdown` (the picked entry's breakdown, "" none). Actions:
//! `hero`, `strike` (arg main, offhand, ranged or dual; on while one is), `use_skill`, `pouch` (use a pouch item),
//! `unpouch`, `unequip`, `stash_use`, `stash_filter`, `log_filter`, `log_pick`, `log_clear` (on while the log holds an
//! entry), `next_round`, `stance` (on under a ruleset with stances), `heal_hero`, `heal_dummy` and `exit`.

use crate::dojo::{DAct, Dojo, LOG_FILTERS, STASH_FILTERS};
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_battle::dojo::SLOTS;

pub const SCREEN: &str = "dojo";
pub const HERO: &str = "hero";
pub const STRIKE: &str = "strike";
pub const USE_SKILL: &str = "use_skill";
pub const POUCH: &str = "pouch";
pub const UNPOUCH: &str = "unpouch";
pub const UNEQUIP: &str = "unequip";
pub const STASH_USE: &str = "stash_use";
pub const STASH_FILTER: &str = "stash_filter";
pub const LOG_FILTER: &str = "log_filter";
pub const LOG_PICK: &str = "log_pick";
pub const LOG_CLEAR: &str = "log_clear";
pub const NEXT_ROUND: &str = "next_round";
pub const STANCE: &str = "stance";
pub const HEAL_HERO: &str = "heal_hero";
pub const HEAL_DUMMY: &str = "heal_dummy";
pub const EXIT: &str = "exit";

/// The strikes, in the console's order.
const STRIKES: [&str; 4] = ["main", "offhand", "ranged", "dual"];

/// The theme role of a log category (`dojo::category_colour`'s colours as roles).
fn category_role(c: &str) -> &'static str {
    match c {
        "ATTACK" => "foe",
        "DEFEND" => "info",
        "BUFF" => "good",
        _ => "dim",
    }
}

/// The view of dojo `d`.
pub fn view(d: &Dojo) -> View {
    let s = d.session();
    let (stash_filter, log_filter) = d.filters();
    let heroes: Vec<RowView> = s.heroes.iter().enumerate().map(|(i, h)| RowView { id: i.to_string(), label: h.name.clone(), sub: Dojo::hero_line(h), enabled: true, selected: i == s.active, ..Default::default() }).collect();
    let (carry, over) = d.carry_line();
    let (hero, dummy) = (s.hero(), &s.dummy);
    let stage = vec![
        (hero.name.clone(), format!("HP {}/{}", hero.hp, hero.max_hp)),
        (d.stage_line(0), String::new()),
        (dummy.name.clone(), format!("HP {}/{}", dummy.hp, dummy.max_hp)),
        (d.stage_line(1), String::new()),
    ];
    let inv = &hero.inventory;
    let gear: Vec<RowView> = SLOTS
        .iter()
        .map(|(key, slot)| {
            let worn = inv.get(*slot).map(|e| e.item.clone());
            RowView { id: (*key).into(), label: Dojo::slot_label(key, worn.as_ref()), enabled: worn.is_some(), ..Default::default() }
        })
        .collect();
    let pouch: Vec<RowView> = (0..inv.max_pouch_slots)
        .map(|i| {
            let item = inv.pouch.get(i);
            let buttons = if item.is_some() { vec![RowButton { action: UNPOUCH.into(), label: "x".into(), enabled: true }] } else { Vec::new() };
            RowView { id: i.to_string(), label: Dojo::pouch_label(i, item), enabled: item.is_some(), buttons, ..Default::default() }
        })
        .collect();
    let stash: Vec<RowView> = d
        .stash_rows()
        .into_iter()
        .filter_map(|(id, n)| {
            let item = s.cat.items.get(&id)?;
            Some(RowView { label: format!("{} x{n}", item.name), note: d.stash_help(item), enabled: Dojo::usable(item), id, ..Default::default() })
        })
        .collect();
    let log: Vec<RowView> = d
        .log_entries()
        .into_iter()
        .map(|e| {
            let (head, sub) = Dojo::log_lines(e);
            RowView { id: e.n.to_string(), label: head, sub, tone: category_role(e.category).into(), enabled: true, selected: d.picked() == Some(e.n), ..Default::default() }
        })
        .collect();
    let breakdown = d.picked().and_then(|n| s.log.iter().find(|e| e.n == n)).map(Dojo::breakdown).unwrap_or_default();
    let avail = s.availability();
    View::new(SCREEN)
        .field("title", Field::Text("Dojo".into()))
        .field("heroes", Field::Rows(heroes))
        .field("carry", Field::Text(carry))
        .tone("carry", &[if over { "foe" } else { "" }])
        .field("stage", Field::Stats(stage))
        .tone("stage", &["gold", "dim", "foe", "dim"])
        .field("round", Field::Text(format!("Round {}", s.round)))
        .field("gear", Field::Rows(gear))
        .field("pouch", Field::Rows(pouch))
        .field("stash_tabs", Field::Tabs { labels: STASH_FILTERS.iter().map(|(_, l)| l.to_string()).collect(), open: STASH_FILTERS.iter().position(|(k, _)| *k == stash_filter).unwrap_or(0) })
        .field("stash", Field::Rows(stash))
        .field("log_tabs", Field::Tabs { labels: LOG_FILTERS.iter().map(|f| f.to_string()).collect(), open: LOG_FILTERS.iter().position(|f| *f == log_filter).unwrap_or(0) })
        .field("log", Field::Rows(log))
        .field("breakdown", Field::Text(breakdown))
        .action(HERO, "Hero", true)
        .action(STRIKE, "Strike", STRIKES.iter().any(|k| avail.get(k).is_some_and(|a| a.enabled)))
        .action(USE_SKILL, "Use", true)
        .action(POUCH, "Use item", !inv.pouch.is_empty())
        .action(UNPOUCH, "Unload", !inv.pouch.is_empty())
        .action(UNEQUIP, "Take off", true)
        .action(STASH_USE, "Wear", true)
        .action(STASH_FILTER, "Filter", true)
        .action(LOG_FILTER, "Log filter", true)
        .action(LOG_PICK, "Explain", true)
        .action(LOG_CLEAR, "CLEAR", !s.log.is_empty())
        .action(NEXT_ROUND, "Next round", true)
        .action(STANCE, "Stance", s.ruleset.stances())
        .action(HEAL_HERO, "Heal hero", true)
        .action(HEAL_DUMMY, "Heal dummy", true)
        .action(EXIT, "Exit", true)
}

/// A view click as the dojo's own (`DAct`); None for an unknown action, one that is off, an arg that names nothing on
/// the dojo (a hero, slot, pouch slot, stash item, filter or log entry it does not hold), or a strike that is off.
/// Stage B's layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn dojo_act(d: &Dojo, a: &Act) -> Option<DAct> {
    view(d).act(&a.action).filter(|x| x.enabled)?;
    let s = d.session();
    let arg = a.arg.clone().filter(|x| !x.is_empty());
    let index = || arg.as_ref().and_then(|x| x.parse::<usize>().ok());
    match a.action.as_str() {
        HERO => index().filter(|i| *i < s.heroes.len()).map(DAct::Hero),
        STRIKE => {
            let key = STRIKES.iter().find(|k| arg.as_deref() == Some(**k))?;
            s.availability().get(key).filter(|x| x.enabled)?;
            Some(DAct::Strike(key))
        }
        USE_SKILL => Some(DAct::UseSkill),
        POUCH => index().filter(|i| *i < s.hero().inventory.pouch.len()).map(DAct::Pouch),
        UNPOUCH => index().filter(|i| *i < s.hero().inventory.pouch.len()).map(DAct::Unpouch),
        UNEQUIP => SLOTS.iter().find(|(k, slot)| arg.as_deref() == Some(*k) && s.hero().inventory.get(*slot).is_some()).map(|(k, _)| DAct::Unequip(k)),
        STASH_USE => arg.filter(|id| s.stash.contains(id)).map(DAct::StashClick),
        STASH_FILTER => STASH_FILTERS.iter().find(|(k, _)| arg.as_deref() == Some(*k)).map(|(k, _)| DAct::StashFilter(k)),
        LOG_FILTER => LOG_FILTERS.iter().find(|f| arg.as_deref() == Some(**f)).map(|f| DAct::LogFilter(f)),
        LOG_PICK => index().filter(|n| s.log.iter().any(|e| e.n == *n)).map(DAct::LogPick),
        LOG_CLEAR => Some(DAct::LogClear),
        NEXT_ROUND => Some(DAct::NextRound),
        STANCE => Some(DAct::Stance),
        HEAL_HERO => Some(DAct::HealHero),
        HEAL_DUMMY => Some(DAct::HealDummy),
        EXIT => Some(DAct::Exit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::game_cx;

    fn rows_of<'a>(v: &'a View, name: &str) -> &'a [RowView] {
        match v.get(name) {
            Some(Field::Rows(r)) => r,
            _ => panic!("no rows {name}"),
        }
    }

    fn text<'a>(v: &'a View, name: &str) -> &'a str {
        match v.get(name) {
            Some(Field::Text(t)) => t,
            _ => panic!("no text {name}"),
        }
    }

    fn click(a: &str, arg: &str) -> Act {
        Act { action: a.into(), arg: (!arg.is_empty()).then(|| arg.to_string()) }
    }

    /// The fields in order; the heroes with the active one selected; the stage's four lines toned; a slot per gear key;
    /// the stash filter open on All; an empty log with CLEAR off.
    #[test]
    fn the_view_is_the_dojos_party_stage_gear_stash_and_log() {
        let cx = game_cx();
        let d = Dojo::new(&cx);
        let v = view(&d);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "heroes", "carry", "stage", "round", "gear", "pouch", "stash_tabs", "stash", "log_tabs", "log", "breakdown"]);
        let heroes = rows_of(&v, "heroes");
        assert_eq!(heroes.len(), d.session().heroes.len());
        assert!(heroes[d.session().active].selected && heroes[0].sub.starts_with("HP "));
        assert!(matches!(v.get("stage"), Some(Field::Stats(s)) if s.len() == 4 && s[0].1.starts_with("HP ")));
        assert_eq!((v.tone_of("stage", 0), v.tone_of("stage", 2)), (Some("gold"), Some("foe")));
        assert_eq!(text(&v, "round"), format!("Round {}", d.session().round));
        let gear: Vec<&str> = rows_of(&v, "gear").iter().map(|r| r.id.as_str()).collect();
        assert_eq!(gear, SLOTS.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        assert!(matches!(v.get("stash_tabs"), Some(Field::Tabs { open: 0, .. })));
        assert_eq!(rows_of(&v, "stash").len(), d.stash_rows().len());
        assert!(rows_of(&v, "log").is_empty() && !v.act(LOG_CLEAR).unwrap().enabled && text(&v, "breakdown").is_empty());
    }

    /// The clicks map to the dojo's own and drive it: a strike logs an entry (then CLEAR is on and the entry can be
    /// picked, its breakdown shown), a filter opens its tab; refusals (an unknown strike, a missing hero, a log entry or
    /// stash item the dojo does not hold, an unknown action) map to nothing.
    #[test]
    fn the_actions_are_the_dojos_clicks() {
        let mut cx = game_cx();
        let mut d = Dojo::new(&cx);
        let key = STRIKES.iter().find(|k| d.session().availability().get(*k).is_some_and(|a| a.enabled)).expect("a strike is on");
        let a = dojo_act(&d, &click(STRIKE, key)).expect("the strike maps");
        d.act(&mut cx, a);
        let v = view(&d);
        let log = rows_of(&v, "log");
        assert!(!log.is_empty() && v.act(LOG_CLEAR).unwrap().enabled, "{log:?}");
        let n = log[0].id.clone();
        let Some(a) = dojo_act(&d, &click(LOG_PICK, &n)) else { panic!("the pick maps") };
        d.act(&mut cx, a);
        let v = view(&d);
        assert!(rows_of(&v, "log")[0].selected && !text(&v, "breakdown").is_empty());
        let a = dojo_act(&d, &click(STASH_FILTER, "consumable")).expect("a filter maps");
        d.act(&mut cx, a);
        assert!(matches!(view(&d).get("stash_tabs"), Some(Field::Tabs { open: 4, .. })));
        assert!(dojo_act(&d, &click(STRIKE, "kick")).is_none() && dojo_act(&d, &click(HERO, "99")).is_none());
        assert!(dojo_act(&d, &click(LOG_PICK, "9999")).is_none() && dojo_act(&d, &click(STASH_USE, "nope")).is_none());
        assert!(matches!(dojo_act(&d, &click(EXIT, "")), Some(DAct::Exit)) && dojo_act(&d, &click("nope", "")).is_none());
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&Dojo::new(&game_cx()))));
    }

    /// Writes the fixture from a fresh dojo after one strike (run by hand: `cargo test -p emberwake_gui
    /// write_dojo_fixture -- --ignored`).
    #[test]
    #[ignore]
    fn write_dojo_fixture() {
        let mut cx = game_cx();
        let mut d = Dojo::new(&cx);
        if let Some(a) = STRIKES.iter().find_map(|k| dojo_act(&d, &click(STRIKE, k))) {
            d.act(&mut cx, a);
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/dojo.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&view(&d).to_json()).unwrap() + "\n").unwrap();
    }
}
