//! UISPLIT1-I (ENGINE-23) Stage A: the party loadout's view-model (the Character window's Gear tab, the screen a
//! FOOTLOCKER1 footlocker opens: one member's gear and battle pouch over the shared stash). What the tab shows and
//! offers, as a `screen_layout::View`, for a layout file to place later (Stage B, its own row once the mock is signed
//! off); today's hand drawing stays. Its fixture is `ui/views/loadout.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the member's name); `heading` ("<Class>  Lv N", over the stats); `stats` (the stat sheet with the
//! draft's gear, label and value as shown); `equipped` (a row per loadout slot, id the slot: "<Slot>: <item>" with the
//! weapon instance's durability, the wield penalty badge as the detail in `foe`, on while an item is worn by choice
//! (a class default or an empty slot is off), the hover card's help line as the note, the item's icon); `off_hand`
//! (the Off-Hand button the draft would fight with, "" none; toned `gold` while it can be used, else `dim`); `pouch`
//! (a row per pouch slot, id its index: "1. <item>" or "1. Empty", on while filled; with pouch stacks the detail is
//! "in pouch/packs/limit" and the row carries `-` and `+` buttons); `stash` (a row per stash item by name, id the item:
//! "<name> xN", the badge as the detail in `foe`, on while a click does something). Actions: `take_off` (an equipped
//! row, the slot as arg), `pouch_out` (a filled pouch row, its index), `pack_less` and `pack_more` (a stack row's
//! buttons, its index), `use` (a stash row: wear it, put one in the pouch, or read a mastery book), `reset`, `confirm`
//! (both on while the draft differs from the saved state) and `close`. A drag (a slot to a slot, the stash to a slot
//! and back) stays the drawing's own until Stage B.

use crate::party::{gear_slot, party_ruleset, pouch_label, sheet, stack_help, stash_ids, stash_row, wielding, Character, PAct, POUCH_EMPTY_HELP, POUCH_ITEM_HELP, STACK_EMPTY_HELP};
use crate::screens::Cx;
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_world::party_ops::Stash;
use emberwake_world::progression;
use emberwake_world::state::{stack_limit, UnitState, LOADOUT_SLOTS, POUCH_MAX};

pub const SCREEN: &str = "loadout";
pub const EQUIPPED: &str = "equipped";
pub const POUCH: &str = "pouch";
pub const STASH: &str = "stash";
pub const TAKE_OFF: &str = "take_off";
pub const POUCH_OUT: &str = "pouch_out";
pub const PACK_LESS: &str = "pack_less";
pub const PACK_MORE: &str = "pack_more";
pub const USE: &str = "use";
pub const RESET: &str = "reset";
pub const CONFIRM: &str = "confirm";
pub const CLOSE: &str = "close";

/// The view of the Character window's Gear tab: its draft (or the saved member and stash before one opens).
pub fn view(cx: &Cx, c: &Character) -> View {
    let d = &*cx.data;
    let gs = cx.gs();
    let (work, stash, stacks) = c.gear_now(cx);
    // AUDIT-04 (PR5-40): the draft as it would wield once confirmed (its weapon instances synced)
    let wields = wielding(d, gs, &work, &stash);
    let built = progression::build_with(d, &wields, &gs.world_id, false);
    let icon = |item: &Option<String>| item.as_ref().and_then(|i| crate::tokens::item_icons(d, i).into_iter().next());
    let equipped: Vec<RowView> = LOADOUT_SLOTS
        .iter()
        .map(|slot| {
            let g = gear_slot(d, &work, &wields, built.as_ref(), slot);
            RowView {
                id: slot.to_string(),
                label: g.label,
                detail: g.badge.as_ref().map(|(b, _)| b.clone()).unwrap_or_default(),
                tone: if g.badge.is_some() { "foe".into() } else { String::new() },
                picture: icon(&g.shown),
                enabled: g.explicit.is_some(),
                note: g.help,
                ..Default::default()
            }
        })
        .collect();
    let pouch = if stacks { stack_rows(d, &work, &stash) } else { pouch_rows(d, &work) };
    let stash_rows: Vec<RowView> = stash_ids(d, &stash)
        .iter()
        .map(|id| {
            let r = stash_row(d, gs, built.as_ref(), Some(&work), id);
            RowView {
                id: id.clone(),
                label: format!("{} x{}", d.item_name(id), stash[id]),
                detail: r.badge.as_ref().map(|(b, _)| b.clone()).unwrap_or_default(),
                tone: if r.badge.is_some() { "foe".into() } else { String::new() },
                picture: icon(&Some(id.clone())),
                enabled: r.act.is_some(),
                note: r.help,
                ..Default::default()
            }
        })
        .collect();
    let stats: Vec<(String, String)> = sheet(d, &gs.world_id, &party_ruleset(cx), &wields).into_iter().map(|r| (r.label, r.shown)).collect();
    let (off_hand, off_tone) = match built.as_ref().and_then(|b| b.unit.off_hand_button(&d.cat)) {
        Some(b) => (b.name.clone(), if b.problem.is_none() { "gold" } else { "dim" }),
        None => (String::new(), ""),
    };
    let dirty = c.gear_dirty(cx);
    View::new(SCREEN)
        .field("title", Field::Text(crate::hub::member_name(cx, &work)))
        .field("heading", Field::Text(format!("{}  Lv {}", d.class_name(&gs.world_id, &work.class_id(d)), work.level(d))))
        .field("stats", Field::Stats(stats))
        .field(EQUIPPED, Field::Rows(equipped))
        .field("off_hand", Field::Text(off_hand))
        .tone("off_hand", &[off_tone])
        .field(POUCH, Field::Rows(pouch))
        .field(STASH, Field::Rows(stash_rows))
        .action(TAKE_OFF, "Take off", true)
        .action(POUCH_OUT, "Take out", true)
        .action(PACK_LESS, "-", true)
        .action(PACK_MORE, "+", true)
        .action(USE, "Use", true)
        .action(RESET, "Reset", dirty)
        .action(CONFIRM, "Confirm", dirty)
        .action(CLOSE, "Close", true)
}

/// The classic pouch: POUCH_MAX slots, filled ones on.
fn pouch_rows(d: &emberwake_world::Data, work: &UnitState) -> Vec<RowView> {
    (0..POUCH_MAX)
        .map(|i| {
            let item = work.pouch.get(i).map(String::as_str);
            RowView {
                id: i.to_string(),
                label: pouch_label(d, i, item),
                picture: item.and_then(|it| crate::tokens::item_icons(d, it).into_iter().next()),
                enabled: item.is_some(),
                note: if item.is_some() { POUCH_ITEM_HELP } else { POUCH_EMPTY_HELP }.to_string(),
                ..Default::default()
            }
        })
        .collect()
}

/// INTENT2-U (`pouch_stacks`): POUCH_MAX stack slots; a filled one reads "in pouch/packs/limit" (the pips' numbers)
/// and carries `-` (on above one packed) and `+` (on below the stack's limit).
fn stack_rows(d: &emberwake_world::Data, work: &UnitState, stash: &Stash) -> Vec<RowView> {
    let slots = work.pouch_slots();
    (0..POUCH_MAX)
        .map(|i| {
            let item = slots.get(i).map(String::as_str);
            let mut row = RowView { id: i.to_string(), label: pouch_label(d, i, item), enabled: item.is_some(), note: STACK_EMPTY_HELP.into(), ..Default::default() };
            if let Some(item) = item {
                let (cur, packed, limit) = (work.pouch_count(item), work.packed(item), stack_limit(d, item));
                row.detail = format!("{cur}/{packed}/{limit}");
                row.note = stack_help(work, stash, item);
                row.picture = crate::tokens::item_icons(d, item).into_iter().next();
                row.buttons = vec![
                    RowButton { action: PACK_LESS.into(), label: "-".into(), enabled: packed > 1 },
                    RowButton { action: PACK_MORE.into(), label: "+".into(), enabled: packed < limit },
                ];
            }
            row
        })
        .collect()
}

/// A view click as the tab's own (`PAct`), read against the tab's draft; None for an unknown action, a row that is not
/// there or one a click does nothing on. Stage B's layout draw hands its clicks here; until then only the tests call
/// it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn loadout_act(cx: &Cx, c: &Character, a: &Act) -> Option<PAct> {
    let d = &*cx.data;
    let (work, stash, stacks) = c.gear_now(cx);
    let arg = a.arg.clone().filter(|x| !x.is_empty());
    let slot_item = |i: usize| if stacks { work.pouch_slots().get(i).cloned() } else { work.pouch.get(i).cloned() };
    let index = || arg.as_ref().and_then(|x| x.parse::<usize>().ok());
    match a.action.as_str() {
        TAKE_OFF => arg.filter(|s| LOADOUT_SLOTS.contains(&s.as_str()) && gear_slot(d, &work, &work, None, s).explicit.is_some()).map(PAct::Unequip),
        POUCH_OUT => index().filter(|i| slot_item(*i).is_some()).map(PAct::PouchRemove),
        PACK_LESS | PACK_MORE if stacks => {
            let item = slot_item(index()?)?;
            let packed = work.packed(&item);
            let to = if a.action == PACK_LESS { packed - 1 } else { packed + 1 };
            (1..=stack_limit(d, &item)).contains(&to).then_some(PAct::PouchPack(item, to))
        }
        USE => arg.filter(|id| stash.contains_key(id) && d.cat.items.contains_key(id)).and_then(|id| stash_row(d, cx.gs(), None, None, &id).act),
        RESET => c.gear_dirty(cx).then_some(PAct::ResetGear),
        CONFIRM => c.gear_dirty(cx).then_some(PAct::ConfirmGear),
        CLOSE => Some(PAct::Close),
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

    /// A game whose first member's Gear tab is open, with two of a wearable item and of a battle consumable in the stash.
    fn gear_cx() -> (Cx, Character, String, String) {
        let mut cx = game_cx();
        let d = cx.data.clone();
        let member = cx.gs().party.members[0].id.clone();
        let wear = d.cat.items.values().filter(|i| i.is_equipment()).map(|i| i.id.clone()).min().expect("a wearable item");
        let drink = d.cat.items.values().filter(|i| i.offered_in_battle()).map(|i| i.id.clone()).min().expect("a battle consumable");
        let gs = cx.gs_mut();
        gs.party.stash.insert(wear.clone(), 2);
        gs.party.stash.insert(drink.clone(), 2);
        gs.party.member_mut(&member).unwrap().pouch.clear();
        let c = Character::new(&cx, &member, "gear");
        (cx, c, wear, drink)
    }

    /// The fields in order; a row per loadout slot and per pouch slot; the stash by name with its counts; Reset and
    /// Confirm off with no draft; the stats are the sheet the tab draws.
    #[test]
    fn the_view_is_the_members_gear_pouch_and_the_stash() {
        let (cx, c, wear, drink) = gear_cx();
        let v = view(&cx, &c);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "heading", "stats", EQUIPPED, "off_hand", POUCH, STASH]);
        let m = cx.gs().party.member(c.gear_member()).unwrap();
        assert_eq!(text(&v, "title"), crate::hub::member_name(&cx, m));
        assert!(text(&v, "heading").contains(&format!("Lv {}", m.level(&cx.data))));
        let eq = rows_of(&v, EQUIPPED);
        assert_eq!(eq.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), LOADOUT_SLOTS.to_vec());
        assert!(eq.iter().all(|r| !r.note.is_empty() && r.label.contains(": ")), "{eq:?}");
        let pouch = rows_of(&v, POUCH);
        assert!(pouch.len() == POUCH_MAX && pouch.iter().all(|r| !r.enabled && r.label.ends_with("Empty")), "{pouch:?}");
        let stash = rows_of(&v, STASH);
        let ids: Vec<String> = stash.iter().map(|r| r.id.clone()).collect();
        assert_eq!(ids, stash_ids(&cx.data, &cx.gs().party.stash));
        for id in [&wear, &drink] {
            let r = stash.iter().find(|r| &r.id == id).expect("in the stash");
            assert!(r.enabled && r.label.ends_with(" x2"), "{r:?}");
        }
        assert!(!v.act(RESET).unwrap().enabled && !v.act(CONFIRM).unwrap().enabled);
        match v.get("stats") {
            Some(Field::Stats(s)) => assert!(!s.is_empty()),
            _ => panic!("no stats"),
        }
    }

    /// The clicks map to the tab's own: a stash item worn or pouched, the draft then on (Reset and Confirm on), a worn
    /// item taken off, a pouch slot emptied; a missing row or unknown action maps to nothing.
    #[test]
    fn the_actions_are_the_tabs_clicks() {
        let (mut cx, mut c, wear, drink) = gear_cx();
        assert!(matches!(loadout_act(&cx, &c, &click(USE, &wear)), Some(PAct::Equip(i)) if i == wear));
        let Some(a) = loadout_act(&cx, &c, &click(USE, &drink)) else { panic!("a consumable goes in the pouch") };
        assert!(matches!(&a, PAct::PouchAdd(i) if *i == drink));
        c.act(&mut cx, a);
        let v = view(&cx, &c);
        assert!(v.act(RESET).unwrap().enabled && v.act(CONFIRM).unwrap().enabled, "a draft is open");
        let first = &rows_of(&v, POUCH)[0];
        assert!(first.enabled && first.label.contains(&cx.data.item_name(&drink)), "{first:?}");
        assert!(matches!(loadout_act(&cx, &c, &click(POUCH_OUT, "0")), Some(PAct::PouchRemove(0))));
        assert!(loadout_act(&cx, &c, &click(POUCH_OUT, "3")).is_none(), "an empty slot");
        assert!(matches!(loadout_act(&cx, &c, &click(RESET, "")), Some(PAct::ResetGear)));
        assert!(matches!(loadout_act(&cx, &c, &click(CONFIRM, "")), Some(PAct::ConfirmGear)));
        // a worn item: its row on, take_off maps; an item not in the stash, a bad slot and an unknown action do not
        let wa = loadout_act(&cx, &c, &click(USE, &wear)).unwrap();
        c.act(&mut cx, wa);
        let v = view(&cx, &c);
        let worn = rows_of(&v, EQUIPPED).iter().find(|r| r.enabled).expect("a slot worn by choice").clone();
        assert!(matches!(loadout_act(&cx, &c, &click(TAKE_OFF, &worn.id)), Some(PAct::Unequip(s)) if s == worn.id));
        assert!(loadout_act(&cx, &c, &click(TAKE_OFF, "nope")).is_none() && loadout_act(&cx, &c, &click(USE, "nope")).is_none());
        assert!(loadout_act(&cx, &c, &click("nope", "")).is_none() && loadout_act(&cx, &c, &click(PACK_MORE, "0")).is_none(), "no stacks: no pack buttons");
        assert!(matches!(loadout_act(&cx, &c, &click(CLOSE, "")), Some(PAct::Close)));
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        let (cx, c, _, _) = gear_cx();
        assert_eq!(shape(&fixture), shape(&view(&cx, &c)));
    }

    /// Writes the fixture from the debug game's first member (run by hand: `cargo test -p emberwake_gui
    /// write_loadout_fixture -- --ignored`), so it holds the sample's values.
    #[test]
    #[ignore]
    fn write_loadout_fixture() {
        let (cx, c, _, _) = gear_cx();
        let v = view(&cx, &c).to_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/loadout.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}
