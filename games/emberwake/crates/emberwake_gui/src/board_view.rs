//! UISPLIT1-H (ENGINE-21) Stage A: the notice board's view-model. What the board pane (`board::BoardPane`) shows and
//! offers, as a `screen_layout::View`, for a layout file to place later (Stage B, its own row once the mock is signed
//! off); today's hand drawing stays. Its fixture is `ui/views/board.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the board's name), `keeper`, `intro`; one rows field per section of `board::SECTIONS` (`main`,
//! `jobs`, `under_way`, `locked`, `closed`; a row per quest: its name, "Lv N", the deadline in `foe`, the node under
//! it, the section's line as its note, the pick selected); `detail_title` and `detail` (the picked quest's paragraphs,
//! one per line, each toned as drawn); `accept_why` (why Accept is off) and `errors` (the last refusal). Actions:
//! `pick` (a row's select, the quest as arg), `accept` (on for an offered quest) and `close`.

use crate::board::{rows, BoardAct, BoardPane};
use crate::screens::Cx;
use aegis_core::json::{self, s, s_or, Dict};
use aegis_ui::screen_layout::{Act, Field, RowView, View};
use emberwake_world::board;

pub const SCREEN: &str = "board";
pub const PICK: &str = "pick";
pub const ACCEPT: &str = "accept";
pub const CLOSE: &str = "close";

/// The view of board `view` (`board::view`, with `board::with_unlock_words`) with the pane's pick and the picked
/// quest's detail `det` (`board::detail`).
pub fn view(cx: &Cx, pane: &BoardPane, view: &Dict, det: &Dict) -> View {
    let all = rows(view);
    let mut v = View::new(SCREEN)
        .field("title", Field::Text(s_or(view, "name", "Board")))
        .field("keeper", Field::Text(s(view, "keeper_name")))
        .field("intro", Field::Text(s(view, "intro")));
    for (id, _) in board::SECTIONS {
        let list: Vec<RowView> = all.iter().filter(|r| s(r, "_section") == id).map(|r| quest_row(r, &pane.selected)).collect();
        v = v.field(id, Field::Rows(list));
    }
    let row = all.iter().find(|r| s(r, "quest") == pane.selected).cloned().unwrap_or_default();
    let paras = if det.is_empty() && row.is_empty() { Vec::new() } else { BoardPane::detail_paras(cx, det, &row) };
    // the deadline's Arterial Red is the theme's foe here; the body text keeps the default colour
    let tones: Vec<&str> = paras.iter().map(|p| if p.2 == crate::board::DEADLINE_ROLE { "foe" } else if p.2 == "text" { "" } else { p.2 }).collect();
    let why = crate::board::accept_why(&s(&row, "status"), &s(&row, "line"));
    v = v
        .field("detail_title", Field::Text(if row.is_empty() && det.is_empty() { String::new() } else { s_or(det, "name", &s(&row, "name")) }))
        .field("detail", Field::Text(paras.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join("\n")))
        .tone("detail", &tones)
        .field("accept_why", Field::Text(why.clone()))
        .field("errors", Field::Text(pane.errors.join(" ")))
        .action(PICK, "Pick", true)
        .action(ACCEPT, "Accept", why.is_empty())
        .action(CLOSE, "Close", true);
    v
}

/// A quest's row: the name, "Lv N", the deadline (in `foe`), the node under it, the section's line as the note.
fn quest_row(r: &Dict, selected: &str) -> RowView {
    let q = s(r, "quest");
    let lv = json::i(r, "rec_level", 0);
    let due = r.get("deadline").map(|v| s(json::dict(v), "text")).unwrap_or_default();
    RowView {
        id: q.clone(),
        label: s(r, "name"),
        detail: if lv > 0 { format!("Lv {lv}") } else { String::new() },
        detail2: due.clone(),
        sub: s(r, "node_name"),
        enabled: true,
        selected: q == selected,
        tone: if lv > 0 { "info".into() } else { String::new() },
        tone2: if due.is_empty() { String::new() } else { "foe".into() },
        note: s(r, "line"),
        ..Default::default()
    }
}

/// A view click as the pane's own (`BoardAct`); None for an unknown action or a pick with no quest. Stage B's layout
/// draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn board_act(pane: &BoardPane, a: &Act) -> Option<BoardAct> {
    match a.action.as_str() {
        PICK => a.arg.clone().filter(|q| !q.is_empty()).map(BoardAct::Pick),
        ACCEPT => Some(BoardAct::Accept(pane.selected.clone())),
        CLOSE => Some(BoardAct::Close),
        _ => None,
    }
}

/// The view on the UI Gallery's fixture board (`board::sample`) and on a real board of the debug game.
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

    /// Every section is a rows field in the kit's order, each quest a row (name, level, deadline in foe, node, line);
    /// the pick is selected and its detail shows, toned as the pane draws it; Accept is on only for an offered quest,
    /// with the pane's reason otherwise; a click maps back to the pane's own act.
    #[test]
    fn the_view_is_the_boards_sections_detail_and_buttons() {
        let cx = game_cx();
        let (bv, details) = crate::board::sample();
        let mut pane = BoardPane::default();
        let det_of = |q: &str| details.iter().find(|d| s(d, "quest") == q).cloned().unwrap_or_default();
        let pick = pane.select(&bv);
        let v = view(&cx, &pane, &bv, &det_of(&pick));
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "keeper", "intro", "main", "jobs", "under_way", "locked", "closed", "detail_title", "detail", "accept_why", "errors"]);
        assert_eq!((text(&v, "title"), text(&v, "keeper")), ("Notice board", "The Clerk"));
        let main = &rows_of(&v, "main")[0];
        assert_eq!((main.id.as_str(), main.label.as_str(), main.sub.as_str(), main.selected), ("sample:m1", "Clear the Old Mill", "Old Mill", true));
        assert!(!main.detail2.is_empty() && main.tone2 == "foe", "the deadline in foe: {main:?}");
        assert_eq!(rows_of(&v, "jobs").len(), 2);
        assert_eq!(rows_of(&v, "locked")[0].note, "Needs Clear the Old Mill (completed)");
        assert_eq!(text(&v, "detail_title"), "Clear the Old Mill");
        let detail = text(&v, "detail");
        let lines: Vec<&str> = detail.split('\n').collect();
        assert_eq!(lines[0], "On offer");
        assert_eq!(v.tone_of("detail", 0), Some("dim"), "the status line dim");
        let due = lines.iter().position(|l| l.contains("Day 14") || l.contains("day 14") || l.contains("14")).expect("the deadline line");
        assert_eq!(v.tone_of("detail", due), Some("foe"), "the deadline line in foe");
        assert!(v.act(ACCEPT).unwrap().enabled && text(&v, "accept_why").is_empty(), "an offered quest");
        // a locked pick: Accept off with the row's reason
        pane.pick("sample:l1".into());
        let v = view(&cx, &pane, &bv, &det_of("sample:l1"));
        assert!(!v.act(ACCEPT).unwrap().enabled);
        assert_eq!(text(&v, "accept_why"), "Needs Clear the Old Mill (completed)");
        assert!(rows_of(&v, "locked")[0].selected && !rows_of(&v, "main")[0].selected);
        // clicks
        let act = |a: &str, arg: Option<&str>| board_act(&pane, &Act { action: a.into(), arg: arg.map(str::to_string) });
        assert!(matches!(act(PICK, Some("sample:j1")), Some(BoardAct::Pick(q)) if q == "sample:j1"));
        assert!(matches!(act(ACCEPT, None), Some(BoardAct::Accept(q)) if q == "sample:l1"));
        assert!(matches!(act(CLOSE, None), Some(BoardAct::Close)));
        assert!(act(PICK, None).is_none() && act("nope", None).is_none());
        // the refusal line
        pane.errors = vec!["Not now.".into()];
        assert_eq!(text(&view(&cx, &pane, &bv, &Dict::new()), "errors"), "Not now.");
        // an empty board: no rows, no detail, Accept off
        let empty = view(&cx, &BoardPane::default(), &Dict::new(), &Dict::new());
        assert!(board::SECTIONS.iter().all(|(id, _)| rows_of(&empty, id).is_empty()));
        assert_eq!((text(&empty, "detail"), text(&empty, "detail_title")), ("", ""));
        assert!(!empty.act(ACCEPT).unwrap().enabled);
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let cx = game_cx();
        let (bv, details) = crate::board::sample();
        let mut pane = BoardPane::default();
        let pick = pane.select(&bv);
        let det = details.iter().find(|d| s(d, "quest") == pick).cloned().unwrap_or_default();
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&cx, &pane, &bv, &det)));
    }

    /// Writes the fixture from the gallery's board (run by hand: `cargo test -p emberwake_gui write_board_fixture --
    /// --ignored`), so it holds the sample's values.
    #[test]
    #[ignore]
    fn write_board_fixture() {
        let cx = game_cx();
        let (bv, details) = crate::board::sample();
        let mut pane = BoardPane::default();
        let pick = pane.select(&bv);
        let det = details.iter().find(|d| s(d, "quest") == pick).cloned().unwrap_or_default();
        let v = view(&cx, &pane, &bv, &det).to_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/board.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}
