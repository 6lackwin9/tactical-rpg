//! UISPLIT1-I (ENGINE-23) Stage A: the recipe pane's view-model (`recipe::RecipePane`). What the pane shows and offers,
//! as a `screen_layout::View`, for a layout file to place later (Stage B, its own row once the mock is signed off);
//! today's hand drawing stays. Its fixture is `ui/views/recipe.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the character's name); one rows field per picker, empty while the pane does not show it:
//! `members` (while a member recipe is offered), `families`, `items` and `weapons` (a row per choice, its index as
//! the id, the pick selected); `recipes` (a row per offered recipe: its name, its cost as the sub ("Free" for none),
//! why it can't run now as the detail in `foe`, the hover card's line as the note (its description while it can run,
//! else the reason), a Do button, on while it can run); `line` (the last run's refusal in `foe`, else its notes in
//! `gold`). Actions: `member`, `family`, `item`, `weapon` (a picker row's click, the index as arg), `run` (a recipe's
//! Do, the recipe as arg) and `close`.

use crate::recipe::{weapon_labels, RecipeAct, RecipePane};
use crate::screens::Cx;
use aegis_core::json::{self, s};
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_data::recipe::{TARGET_MEMBER, TARGET_WEAPON};
use emberwake_world::recipe;

pub const SCREEN: &str = "recipe";
pub const MEMBER: &str = "member";
pub const FAMILY: &str = "family";
pub const ITEM: &str = "item";
pub const WEAPON: &str = "weapon";
pub const RUN: &str = "run";
pub const CLOSE: &str = "close";

/// The view of character `npc`'s recipes with the pane's picks and last run.
pub fn view(cx: &Cx, pane: &RecipePane, npc: &str) -> View {
    let (d, gs) = (&cx.data, cx.gs());
    let name = json::s_or(d.resolve(&gs.world_id, "characters", npc), "name", npc);
    let pick = pane.pick(cx, npc);
    let offered = recipe::view(d, gs, npc, &pick);
    let members: Vec<String> = if offered.iter().any(|r| s(json::dict(r), "target") == TARGET_MEMBER) { gs.party.members.iter().map(|m| crate::hub::member_name(cx, m)).collect() } else { Vec::new() };
    let families = RecipePane::families(&offered);
    let items: Vec<String> = RecipePane::items(&offered).iter().map(|id| d.item_name(id)).collect();
    let weapons = weapon_labels(d, gs, &RecipePane::targets(&offered, TARGET_WEAPON));
    let rows: Vec<RowView> = offered.iter().map(json::dict).map(recipe_row).collect();
    let (line, tone) = if pane.errors.is_empty() { (pane.notes.join(" "), "gold") } else { (pane.errors.join(" "), "foe") };
    View::new(SCREEN)
        .field("title", Field::Text(name))
        .field("members", Field::Rows(pick_rows(&members, pane.member)))
        .field("families", Field::Rows(pick_rows(&families, pane.family)))
        .field("items", Field::Rows(pick_rows(&items, pane.item)))
        .field("weapons", Field::Rows(pick_rows(&weapons, pane.weapon)))
        .field("recipes", Field::Rows(rows))
        .field("line", Field::Text(line.clone()))
        .tone("line", &[if line.is_empty() { "" } else { tone }])
        .action(MEMBER, "For", members.len() > 1)
        .action(FAMILY, "Family", families.len() > 1)
        .action(ITEM, "Item", items.len() > 1)
        .action(WEAPON, "Weapon", weapons.len() > 1)
        .action(RUN, "Do", true)
        .action(CLOSE, "Close", true)
}

/// A picker's rows: each choice by its index, the pick (clamped to the last, as the pane draws it) selected.
fn pick_rows(list: &[String], at: usize) -> Vec<RowView> {
    let at = at.min(list.len().saturating_sub(1));
    list.iter().enumerate().map(|(i, label)| RowView { id: i.to_string(), label: label.clone(), enabled: true, selected: i == at, ..Default::default() }).collect()
}

/// An offered recipe's row: its name, its cost, why it can't run now (in `foe`), the hover card's line, Do.
fn recipe_row(r: &json::Dict) -> RowView {
    let ready = json::b(r, "ready", false);
    let reason = s(r, "reason");
    let cost = json::sget(r, "cost").join(", ");
    RowView {
        id: s(r, "id"),
        label: s(r, "name"),
        detail: reason.clone(),
        sub: if cost.is_empty() { "Free".into() } else { cost },
        enabled: ready,
        tone: if reason.is_empty() { String::new() } else { "foe".into() },
        note: if ready { s(r, "description") } else { reason },
        buttons: vec![RowButton { action: RUN.into(), label: "Do".into(), enabled: ready }],
        ..Default::default()
    }
}

/// A view click as the pane's own (`RecipeAct`); None for an unknown action or a click with no usable arg. Stage B's
/// layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn recipe_act(a: &Act) -> Option<RecipeAct> {
    let index = || a.arg.as_deref().and_then(|x| x.parse::<usize>().ok());
    match a.action.as_str() {
        MEMBER => index().map(RecipeAct::Member),
        FAMILY => index().map(RecipeAct::Family),
        ITEM => index().map(RecipeAct::Item),
        WEAPON => index().map(RecipeAct::Weapon),
        RUN => a.arg.clone().filter(|id| !id.is_empty()).map(RecipeAct::Run),
        CLOSE => Some(RecipeAct::Close),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::tests::{craft_data, data, steel_data, FROM_ACT, FULL, FUNDS};
    use crate::screens::enter_game;
    use crate::story::tests::story_cx;
    use emberwake_world::{new_game, weapons, Data};

    const NPC: &str = "ward:smith";

    /// A cx in a new game of the scratch world `ward` on `d`, with `FUNDS` gold and `stash` in the stash.
    fn cx_on(d: Data, stash: &[(&str, i64)]) -> Cx {
        let mut cx = story_cx(d);
        let mut gs = new_game::create_for_world(&cx.data.clone(), "ward", 1).expect("a new game");
        gs.party.gold = FUNDS;
        for (id, n) in stash {
            gs.party.add_to_stash(id, *n);
        }
        enter_game(&mut cx, gs, -1, Vec::new());
        cx
    }

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

    /// The pane's picks as rows fields (only those it draws), each recipe a row (cost, Do on while it can run, the
    /// reason in foe otherwise), the last run's line toned as drawn; a click maps back to the pane's own act.
    #[test]
    fn the_view_is_the_panes_picks_recipes_and_line() {
        let cx = cx_on(data(), &[("rusty", 1), ("spare", 1)]);
        let mut pane = RecipePane::default();
        let v = view(&cx, &pane, NPC);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "members", "families", "items", "weapons", "recipes", "line"]);
        assert_eq!(text(&v, "title"), "Smith");
        assert_eq!(rows_of(&v, "members").iter().map(|r| (r.label.as_str(), r.selected)).collect::<Vec<_>>(), [("Hero", true)]);
        assert!(rows_of(&v, "families").is_empty() && rows_of(&v, "weapons").is_empty(), "no family or weapon recipe here");
        let items: Vec<(&str, bool)> = rows_of(&v, "items").iter().map(|r| (r.label.as_str(), r.selected)).collect();
        assert_eq!(items.len(), 2);
        assert!(items[0].1 && !items[1].1, "the first item picked: {items:?}");
        assert!(!v.act(MEMBER).unwrap().enabled && v.act(ITEM).unwrap().enabled, "a picker of one is off");
        let train = rows_of(&v, "recipes").iter().find(|r| r.id == "train_bow").expect("the training row").clone();
        assert!(train.enabled && train.buttons[0].enabled && train.detail.is_empty() && train.sub != "Free", "{train:?}");
        let free = rows_of(&v, "recipes").iter().find(|r| r.id == "break_down").expect("the salvage row");
        assert_eq!(free.sub, "Free");
        assert_eq!((text(&v, "line"), v.tone_of("line", 0)), ("", None));
        // past the list the item pick reads the last item, as the pane draws it
        pane.item = 5;
        let v = view(&cx, &pane, NPC);
        assert!(rows_of(&v, "items")[1].selected && !rows_of(&v, "items")[0].selected);
        // the line: the notes in gold, a refusal in foe over them
        pane.notes = vec!["Hero trained.".into()];
        assert_eq!(view(&cx, &pane, NPC).tone_of("line", 0), Some("gold"));
        pane.errors = vec!["Not enough gold.".into()];
        let v = view(&cx, &pane, NPC);
        assert_eq!((text(&v, "line"), v.tone_of("line", 0)), ("Not enough gold.", Some("foe")));
        // clicks
        let act = |a: &str, arg: Option<&str>| recipe_act(&Act { action: a.into(), arg: arg.map(str::to_string) });
        assert!(matches!(act(MEMBER, Some("0")), Some(RecipeAct::Member(0))));
        assert!(matches!(act(FAMILY, Some("1")), Some(RecipeAct::Family(1))));
        assert!(matches!(act(ITEM, Some("1")), Some(RecipeAct::Item(1))));
        assert!(matches!(act(WEAPON, Some("2")), Some(RecipeAct::Weapon(2))));
        assert!(matches!(act(RUN, Some("train_bow")), Some(RecipeAct::Run(id)) if id == "train_bow"));
        assert!(matches!(act(CLOSE, None), Some(RecipeAct::Close)));
        assert!(act(ITEM, Some("x")).is_none() && act(RUN, None).is_none() && act("nope", None).is_none());
    }

    /// A blocked recipe's row is off, its reason in foe and as the note; the family and weapon pickers show for the
    /// recipes that need them; a stash recipe needs no picker.
    #[test]
    fn the_pickers_follow_the_offered_recipes() {
        let d = steel_data();
        let mut cx = cx_on(d.clone(), &[]);
        let gs = cx.gs_mut();
        gs.campaign.as_mut().expect("a campaign world").act = FROM_ACT;
        gs.party.members[0].equipment.insert("main_hand".into(), "blade".into());
        weapons::sync(&d, gs);
        let uid = weapons::all(&gs.party)[0].uid.clone();
        weapons::find_mut(&mut gs.party, &uid).expect("the blade").durability = FULL / 2;
        let v = view(&cx, &RecipePane::default(), NPC);
        assert_eq!(rows_of(&v, "families").len(), 2, "the class's two minors");
        assert!(rows_of(&v, "weapons")[0].label.contains(&format!("{}/{FULL}", FULL / 2)), "the blade's wear");
        let cx = cx_on(craft_data(), &[]);
        let v = view(&cx, &RecipePane::default(), NPC);
        for p in ["members", "families", "items", "weapons"] {
            assert!(rows_of(&v, p).is_empty(), "{p}: a stash recipe needs no pick");
        }
        let bolts = &rows_of(&v, "recipes")[0];
        assert!(!bolts.enabled && !bolts.buttons[0].enabled && !bolts.detail.is_empty() && bolts.tone == "foe" && bolts.note == bolts.detail, "no ore: {bolts:?}");
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let cx = cx_on(data(), &[("rusty", 1), ("spare", 1)]);
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&cx, &RecipePane::default(), NPC)));
    }

    /// Writes the fixture from the scratch smith (run by hand: `cargo test -p emberwake_gui write_recipe_fixture --
    /// --ignored`), so it holds the sample's values.
    #[test]
    #[ignore]
    fn write_recipe_fixture() {
        let cx = cx_on(data(), &[("rusty", 1), ("spare", 1)]);
        let v = view(&cx, &RecipePane::default(), NPC).to_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/recipe.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}
