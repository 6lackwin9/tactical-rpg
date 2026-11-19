//! UISPLIT1-J (ENGINE-25) Stage A: the Content Forge and World Forge viewers' view-model (`forge::Forge`, dev only,
//! read only). What the viewer shows and offers, as a `screen_layout::View`, for a layout file to place later (Stage B);
//! today's hand drawing stays. Its fixture is `ui/views/forge.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the viewer's head); `worlds` (the World Forge's world picker, a row per world, the shown one
//! selected; empty in the Content Forge); `tabs` (the kinds, the open one); `entries` (the open kind's entries, the
//! shown one selected); `heading` (the shown entry's label and id); `fields` (its fields, key and value as text);
//! `summary_head` and `summary` (what the engine makes of it, a line an item with an empty value, toned `gold` for a heading line and `dim`
//! for the "Used by" line); `foot` (the validation line, `good`). Actions: `tab` (a tab's index), `select` (an entry's
//! id), `world` (a world's index) and `exit`.

use crate::forge::{field_lines, FAct, Forge, FOOT};
use crate::screens::Cx;
use aegis_core::json::s;
use aegis_ui::screen_layout::{Act, Field, RowView, View};
use aegis_ui::theme::th;

pub const SCREEN: &str = "forge";
pub const TAB: &str = "tab";
pub const SELECT: &str = "select";
pub const WORLD: &str = "world";
pub const EXIT: &str = "exit";

/// The view of viewer `f`.
pub fn view(cx: &Cx, f: &Forge) -> View {
    let rows = f.rows(cx);
    let shown = f.shown(&rows);
    let worlds: Vec<RowView> = if f.is_world_forge() {
        cx.data.world_ids().iter().map(|w| RowView { id: w.clone(), label: format!("{}  [{w}]", cx.world_name(w)), enabled: true, selected: w == f.world_id(), ..Default::default() }).collect()
    } else {
        Vec::new()
    };
    let tabs = f.tabs();
    let open = tabs.iter().position(|(k, _)| *k == f.kind()).unwrap_or(0);
    let entries: Vec<RowView> = rows.iter().map(|(id, label, _)| RowView { id: id.clone(), label: label.clone(), enabled: true, selected: *id == shown, ..Default::default() }).collect();
    let (heading, fields, summary, tones) = match rows.iter().find(|r| r.0 == shown) {
        Some((id, label, e)) => {
            let lines = f.summary(cx, id, e);
            let tones: Vec<&str> = lines.iter().map(|(_, c)| if *c == th().gold { "gold" } else if *c == th().dim { "dim" } else { "" }).collect();
            let summary = lines.iter().map(|(t, _)| (t.clone(), String::new())).collect();
            (format!("{label}  [{}]", s(e, "id")), field_lines(e), summary, tones)
        }
        None => (String::new(), Vec::new(), Vec::new(), Vec::new()),
    };
    View::new(SCREEN)
        .field("title", Field::Text(f.head().into()))
        .field("worlds", Field::Rows(worlds))
        .field("tabs", Field::Tabs { labels: tabs.iter().map(|(_, l)| l.to_string()).collect(), open })
        .field("entries", Field::Rows(entries))
        .field("heading", Field::Text(heading))
        .field("fields", Field::Stats(fields))
        .field("summary_head", Field::Text(f.summary_head().into()))
        .field("summary", Field::Stats(summary))
        .tone("summary", &tones)
        .field("foot", Field::Text(FOOT.into()))
        .tone("foot", &["good"])
        .action(TAB, "Tab", true)
        .action(SELECT, "Select", true)
        .action(WORLD, "World", f.is_world_forge())
        .action(EXIT, "Exit", true)
}

/// A view click as the viewer's own (`FAct`); None for an unknown action, a tab or world past the list, an entry not in
/// the open kind, or a world pick in the Content Forge. Stage B's layout draw hands its clicks here; until then only the
/// tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn forge_act(cx: &Cx, f: &Forge, a: &Act) -> Option<FAct> {
    let arg = a.arg.clone().filter(|x| !x.is_empty());
    let index = || arg.as_ref().and_then(|x| x.parse::<usize>().ok());
    match a.action.as_str() {
        TAB => index().filter(|i| *i < f.tabs().len()).map(FAct::Tab),
        SELECT => arg.clone().filter(|id| f.rows(cx).iter().any(|r| r.0 == *id)).map(FAct::Select),
        WORLD if f.is_world_forge() => index().filter(|i| *i < cx.data.world_ids().len()).map(FAct::PickWorld),
        EXIT => Some(FAct::Exit),
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

    fn click(a: &str, arg: &str) -> Act {
        Act { action: a.into(), arg: (!arg.is_empty()).then(|| arg.to_string()) }
    }

    /// The Content Forge: its kinds as tabs, the skills listed with the first shown, its fields and its summary (a gold
    /// heading line first); no world picker. The World Forge: the worlds, the shown one selected.
    #[test]
    fn the_view_is_the_viewers_tabs_entries_fields_and_summary() {
        let cx = game_cx();
        let f = Forge::content();
        let v = view(&cx, &f);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "worlds", "tabs", "entries", "heading", "fields", "summary_head", "summary", "foot"]);
        assert!(rows_of(&v, "worlds").is_empty() && !v.act(WORLD).unwrap().enabled);
        assert!(matches!(v.get("tabs"), Some(Field::Tabs { labels, open: 0 }) if labels[0] == "Skills"));
        let entries = rows_of(&v, "entries");
        assert_eq!(entries.len(), f.rows(&cx).len());
        assert!(entries[0].selected && entries.iter().filter(|r| r.selected).count() == 1);
        assert!(matches!(v.get("fields"), Some(Field::Stats(s)) if s.iter().any(|(k, _)| k == "id")));
        assert!(matches!(v.get("summary"), Some(Field::Stats(s)) if !s.is_empty()) && v.tone_of("summary", 0) == Some("gold"));
        let w = Forge::world(&cx);
        let v = view(&cx, &w);
        let worlds = rows_of(&v, "worlds");
        assert_eq!(worlds.len(), cx.data.world_ids().len());
        assert!(worlds[0].selected && v.act(WORLD).unwrap().enabled);
    }

    /// The clicks map to the viewer's own and drive it: a tab opens its kind, an entry is shown; refusals map to nothing.
    #[test]
    fn the_actions_are_the_viewers_clicks() {
        let mut cx = game_cx();
        let mut f = Forge::content();
        let Some(a) = forge_act(&cx, &f, &click(TAB, "1")) else { panic!("tab 1 maps") };
        f.act(&mut cx, a);
        assert_eq!(f.kind(), "items");
        let second = f.rows(&cx)[1].0.clone();
        let a = forge_act(&cx, &f, &click(SELECT, &second)).expect("an item entry maps");
        f.act(&mut cx, a);
        let v = view(&cx, &f);
        assert!(rows_of(&v, "entries").iter().find(|r| r.id == second).is_some_and(|r| r.selected));
        assert!(forge_act(&cx, &f, &click(TAB, "99")).is_none() && forge_act(&cx, &f, &click(SELECT, "nope")).is_none());
        assert!(forge_act(&cx, &f, &click(WORLD, "0")).is_none(), "no world pick in the Content Forge");
        assert!(matches!(forge_act(&cx, &f, &click(EXIT, "")), Some(FAct::Exit)));
        assert!(forge_act(&cx, &f, &click("nope", "")).is_none());
        let w = Forge::world(&cx);
        assert!(matches!(forge_act(&cx, &w, &click(WORLD, "0")), Some(FAct::PickWorld(0))));
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&game_cx(), &Forge::world(&game_cx()))));
    }

    /// Writes the fixture from the World Forge on the debug game (run by hand: `cargo test -p emberwake_gui
    /// write_forge_fixture -- --ignored`).
    #[test]
    #[ignore]
    fn write_forge_fixture() {
        let cx = game_cx();
        let v = view(&cx, &Forge::world(&cx)).to_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/forge.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}
