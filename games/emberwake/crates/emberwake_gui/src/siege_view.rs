//! UISPLIT1-J (ENGINE-25) Stage A: the siege screen's view-model (`siege::SiegeScreen`). What the screen shows and
//! offers, as a `screen_layout::View`, for a layout file to place later (Stage B, with the Art/UI studio's siege board);
//! today's hand drawing stays. Its fixture is `ui/views/siege.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the siege's name); `rooms` (a row per room in order: "1. Name" as the label, its mark (Held, Next,
//! Ahead) as the detail toned `good`, `gold` or `dim`); `down_head` (the heading over the downed); `downed` (a row per
//! downed fighter, id the member, with a Triage button (its label names the reviving item) on while the stash holds
//! one, and its hover line as the note); `line` (the last action's refusal in `foe`, its notes in `gold`, else the
//! no-rest rule in `dim`). Actions: `triage` (a downed row's button, the member as arg), `restock` ("Re-stock (n)", on
//! while a re-stock is left), `fight` (on unless every fighter is down) and `yield` (on when every fighter is down).
//! An empty view (no rows) when no siege runs.

use crate::screens::Cx;
use crate::siege::{down_head, fight_button, restock_label, room_mark, triage_label, triage_note, SiegeAct, SiegeScreen};
use aegis_core::json::{self, s};
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_world::siege;

pub const SCREEN: &str = "siege";
pub const TRIAGE: &str = "triage";
pub const RESTOCK: &str = "restock";
pub const FIGHT: &str = "fight";
pub const YIELD: &str = "yield";

/// The stash item a Triage spends (the first that revives): (id, name).
fn reviver(view: &json::Dict) -> Option<(String, String)> {
    json::aget(view, "items").first().map(json::dict).map(|i| (s(i, "id"), s(i, "name")))
}

/// The view of screen `sc` on the running siege.
pub fn view(cx: &Cx, sc: &SiegeScreen) -> View {
    let v = siege::view(&cx.data, cx.gs()).map(|v| json::dict(&v).clone()).unwrap_or_default();
    let rooms: Vec<RowView> = json::aget(&v, "rooms")
        .iter()
        .map(json::dict)
        .enumerate()
        .map(|(i, room)| {
            let (mark, role) = room_mark(&s(room, "state"));
            RowView { id: i.to_string(), label: format!("{}. {}", i + 1, s(room, "name")), detail: mark.into(), tone: role.into(), enabled: true, ..Default::default() }
        })
        .collect();
    let item = reviver(&v);
    let downed: Vec<RowView> = json::aget(&v, "downed")
        .iter()
        .map(json::dict)
        .map(|m| {
            let who = s(m, "name");
            RowView {
                id: s(m, "member"),
                label: who.clone(),
                enabled: true,
                note: triage_note(item.is_some(), &who),
                buttons: vec![RowButton { action: TRIAGE.into(), label: triage_label(item.as_ref().map(|(_, n)| n.as_str())), enabled: item.is_some() }],
                ..Default::default()
            }
        })
        .collect();
    let (line, tone) = sc.line(&v);
    let all_down = !v.is_empty() && matches!(fight_button(&v).1, SiegeAct::Yield);
    View::new(SCREEN)
        .field("title", Field::Text(s(&v, "name")))
        .field("rooms", Field::Rows(rooms))
        .field("down_head", Field::Text(down_head(downed.is_empty()).into()))
        .field("downed", Field::Rows(downed))
        .field("line", Field::Text(line.clone()))
        .tone("line", &[if line.is_empty() { "" } else { tone }])
        .action(TRIAGE, "Triage", item.is_some())
        .action(RESTOCK, &restock_label(&v), json::b(&v, "can_restock", false))
        .action(FIGHT, "Fight", !v.is_empty() && !all_down)
        .action(YIELD, "Yield the room", all_down)
}

/// A view click as the screen's own (`SiegeAct`); None for an unknown action, one that is off, or a triage of a member
/// who is not down. Stage B's layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn siege_act(cx: &Cx, sc: &SiegeScreen, a: &Act) -> Option<SiegeAct> {
    let v = view(cx, sc);
    v.act(&a.action).filter(|x| x.enabled)?;
    match a.action.as_str() {
        TRIAGE => {
            let member = a.arg.clone().filter(|m| cx.gs().siege.as_ref().is_some_and(|sg| sg.downed.contains(m)))?;
            let kit = siege::view(&cx.data, cx.gs()).map(|v| json::dict(&v).clone()).unwrap_or_default();
            Some(SiegeAct::Triage(member, reviver(&kit)?.0))
        }
        RESTOCK => Some(SiegeAct::Restock),
        FIGHT => Some(SiegeAct::Fight),
        YIELD => Some(SiegeAct::Yield),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::siege::tests::{open, siege_cx};

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

    /// The rooms with their marks, a downed fighter with Triage (Salts), Re-stock (1) and Fight on, the no-rest line;
    /// with every fighter down Yield is on and Fight off.
    #[test]
    fn the_view_is_the_rooms_the_downed_and_the_buttons() {
        let mut cx = siege_cx();
        open(&mut cx);
        cx.gs_mut().siege.as_mut().unwrap().room = 1;
        cx.gs_mut().siege.as_mut().unwrap().downed = vec!["second".into()];
        let sc = SiegeScreen::default();
        let v = view(&cx, &sc);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "rooms", "down_head", "downed", "line"]);
        assert_eq!(text(&v, "title"), "The Hold");
        let marks: Vec<(&str, &str, &str)> = rows_of(&v, "rooms").iter().map(|r| (r.label.as_str(), r.detail.as_str(), r.tone.as_str())).collect();
        assert_eq!(marks, [("1. Grate", "Held", "good"), ("2. Lock", "Next", "gold"), ("3. Hearth", "Ahead", "dim")]);
        let down = &rows_of(&v, "downed")[0];
        assert_eq!((down.id.as_str(), down.buttons[0].label.as_str(), down.buttons[0].enabled), ("second", "Triage (Salts)", true));
        let restock = v.act(RESTOCK).unwrap();
        assert!(restock.enabled && restock.label == "Re-stock (1)");
        assert!(v.act(FIGHT).unwrap().enabled && !v.act(YIELD).unwrap().enabled);
        assert_eq!((text(&v, "line"), v.tone_of("line", 0)), ("No rest until the siege is broken.", Some("dim")));
        cx.gs_mut().siege.as_mut().unwrap().downed = vec!["lead".into(), "second".into()];
        let v = view(&cx, &sc);
        assert!(!v.act(FIGHT).unwrap().enabled && v.act(YIELD).unwrap().enabled);
    }

    /// The clicks map to the screen's own and drive it: Re-stock is spent once (then off, its note in gold), Triage
    /// revives with the first reviving item; a triage of someone not down maps to nothing.
    #[test]
    fn the_actions_are_the_screens_clicks() {
        let mut cx = siege_cx();
        open(&mut cx);
        cx.gs_mut().siege.as_mut().unwrap().room = 1;
        cx.gs_mut().siege.as_mut().unwrap().downed = vec!["second".into()];
        let mut sc = SiegeScreen::default();
        let a = siege_act(&cx, &sc, &click(RESTOCK, "")).expect("restock maps");
        sc.act(&mut cx, a);
        let v = view(&cx, &sc);
        assert!(!v.act(RESTOCK).unwrap().enabled && v.tone_of("line", 0) == Some("gold"), "{:?}", v.get("line"));
        assert!(siege_act(&cx, &sc, &click(TRIAGE, "lead")).is_none(), "not down");
        let Some(a) = siege_act(&cx, &sc, &click(TRIAGE, "second")) else { panic!("triage maps") };
        assert!(matches!(&a, SiegeAct::Triage(m, i) if m == "second" && i == "salts"));
        sc.act(&mut cx, a);
        assert!(cx.gs().siege.as_ref().unwrap().downed.is_empty());
        assert!(siege_act(&cx, &sc, &click(RESTOCK, "")).is_none() && siege_act(&cx, &sc, &click(YIELD, "")).is_none());
        assert!(matches!(siege_act(&cx, &sc, &click(FIGHT, "")), Some(SiegeAct::Fight)));
        assert!(siege_act(&cx, &sc, &click("nope", "")).is_none());
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        let mut cx = siege_cx();
        open(&mut cx);
        assert_eq!(shape(&fixture), shape(&view(&cx, &SiegeScreen::default())));
    }

    /// Writes the fixture from the scratch siege with a downed fighter (run by hand: `cargo test -p emberwake_gui
    /// write_siege_fixture -- --ignored`).
    #[test]
    #[ignore]
    fn write_siege_fixture() {
        let mut cx = siege_cx();
        open(&mut cx);
        cx.gs_mut().siege.as_mut().unwrap().room = 1;
        cx.gs_mut().siege.as_mut().unwrap().downed = vec!["second".into()];
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/siege.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&view(&cx, &SiegeScreen::default()).to_json()).unwrap() + "\n").unwrap();
    }
}
