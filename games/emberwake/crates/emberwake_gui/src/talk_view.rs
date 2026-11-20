//! UISPLIT1-J (ENGINE-25) Stage A: the hub talk box's view-model (`talk::Services`, the conversation over the region
//! map, in a hub, or on the plain backdrop of `ServiceScreen`). What the talk box shows and offers, as a
//! `screen_layout::View`, for a layout file to place later (Stage B); today's hand drawing (`talk::draw_box`) stays.
//! Its fixture is `ui/views/talk.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `pane` (what the conversation shows: "talk", or the pane over it: "shop", "recipe", "facility", "board",
//! with "turn_in" over all; "" when no conversation is open); `speaker` (the speaker's name); `band` (the faction line
//! under it, "" none); `expression` (the node's expression, "" none); `line` (the speaker's line); `replies` (a row per
//! reply as the box draws it, id the widget to click (`choice.<i>`, `dialogue.continue`): the text shown (a locked
//! reply with its reason) as the label, its number 1-9 as the detail, off while locked or while the turn-in picker is
//! open). Actions: `choose` (arg the reply's index, on while a reply is open to pick), `advance` (Continue, on while
//! the node goes on) and `farewell` (on while the conversation can be left: a farewell reply, or a talk that need not
//! be answered). The shop, board, recipe and facility panes have their own views; the shop's is `trade`.

use crate::screens::Cx;
use crate::talk::{band_line, reply_rows, Services, SvcAct, TalkAct};
use aegis_core::json::{self, s};
use aegis_ui::screen_layout::{Act, Field, RowView, View};

pub const SCREEN: &str = "talk";
pub const CHOOSE: &str = "choose";
pub const ADVANCE: &str = "advance";
pub const FAREWELL: &str = "farewell";

/// What the conversation shows (see the module doc).
fn pane(svc: &Services, open: bool) -> &'static str {
    if !open {
        ""
    } else if svc.turning_in() {
        "turn_in"
    } else if !svc.shop_qid().is_empty() {
        "shop"
    } else if !svc.recipe_npc().is_empty() {
        "recipe"
    } else if !svc.facility_npc().is_empty() {
        "facility"
    } else if !svc.board_qid().is_empty() {
        "board"
    } else {
        "talk"
    }
}

/// The view of the services `svc`'s open conversation (an empty one with every action off when none is open).
pub fn view(cx: &Cx, svc: &Services) -> View {
    let v = svc.talk_view(cx);
    let open = v.is_some();
    let v = v.unwrap_or_default();
    let shown = pane(svc, open);
    let talk = shown == "talk";
    let rows = reply_rows(&v);
    let replies: Vec<RowView> = rows
        .iter()
        .enumerate()
        .map(|(i, (id, text, locked, act))| {
            let num = if matches!(act, TalkAct::Choose(_)) && i < 9 { (i + 1).to_string() } else { String::new() };
            RowView { id: id.clone(), label: text.clone(), detail: num, enabled: talk && !locked, ..Default::default() }
        })
        .collect();
    let can_choose = talk && rows.iter().any(|(_, _, locked, act)| !locked && matches!(act, TalkAct::Choose(_)));
    let can_advance = talk && rows.iter().any(|(_, _, _, act)| *act == TalkAct::Advance);
    let has_farewell = json::aget(&v, "choices").iter().any(|c| s(json::dict(c), "service") == "farewell");
    View::new(SCREEN)
        .field("pane", Field::Text(shown.into()))
        .field("speaker", Field::Text(s(&v, "speaker_name")))
        .field("band", Field::Text(band_line(&v)))
        .field("expression", Field::Text(s(&v, "expression")))
        .field("line", Field::Text(s(&v, "text")))
        .field("replies", Field::Rows(replies))
        .action(CHOOSE, "Choose", can_choose)
        .action(ADVANCE, "Continue", can_advance)
        .action(FAREWELL, "Farewell", talk && (has_farewell || !svc.must_choose(cx)))
}

/// A view click as the services' own (`SvcAct`); None for an unknown action, one that is off, or a reply index that is
/// not an open reply. Stage B's layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn talk_act(cx: &Cx, svc: &Services, a: &Act) -> Option<SvcAct> {
    view(cx, svc).act(&a.action).filter(|x| x.enabled)?;
    match a.action.as_str() {
        CHOOSE => {
            let i = a.arg.as_ref().and_then(|x| x.parse::<usize>().ok())?;
            let rows = reply_rows(&svc.talk_view(cx)?);
            let (_, _, locked, act) = rows.get(i)?;
            match act {
                TalkAct::Choose(n) if !locked => Some(SvcAct::Choose(*n)),
                _ => None,
            }
        }
        ADVANCE => Some(SvcAct::Advance),
        FAREWELL => Some(SvcAct::Farewell),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::game_cx;
    use crate::talk::ServiceScreen;

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

    /// The debug game's first person with a farewell reply, talking on the plain backdrop: (cx, host, the farewell's
    /// reply index).
    fn talking() -> (Cx, ServiceScreen, usize) {
        let mut cx = game_cx();
        let d = cx.data.clone();
        let _ = emberwake_world::explore::resume(&d, cx.gs_mut());
        let start = cx.gs().clone();
        let mut host = ServiceScreen::default();
        for c in d.merged_entries(&start.world_id, "characters") {
            *cx.gs_mut() = start.clone();
            host.talk(&mut cx, &s(&c, "id"));
            let Some(v) = host.svc.talk_view(&cx) else { continue };
            if let Some(i) = json::aget(&v, "choices").iter().position(|c| s(json::dict(c), "service") == "farewell") {
                return (cx, host, i);
            }
            host.svc.end(&mut cx);
        }
        panic!("nobody in the first playable world has a farewell reply");
    }

    /// Nothing open: an empty view with every action off. A conversation: the speaker, the line and the replies as the
    /// box draws them (the agent link's `talk_state` rows), numbered, with Farewell on.
    #[test]
    fn the_view_is_the_speaker_the_line_and_the_replies() {
        let cx = game_cx();
        let v = view(&cx, &Services::default());
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["pane", "speaker", "band", "expression", "line", "replies"]);
        assert!(text(&v, "pane").is_empty() && rows_of(&v, "replies").is_empty());
        assert!(v.actions.iter().all(|a| !a.enabled), "nothing open: every action off");
        let (cx, host, _) = talking();
        let v = view(&cx, &host.svc);
        let tv = host.svc.talk_view(&cx).unwrap();
        assert_eq!((text(&v, "pane"), text(&v, "speaker"), text(&v, "line")), ("talk", s(&tv, "speaker_name").as_str(), s(&tv, "text").as_str()));
        let want: Vec<String> = reply_rows(&tv).into_iter().map(|r| r.0).collect();
        let got: Vec<String> = rows_of(&v, "replies").iter().map(|r| r.id.clone()).collect();
        assert_eq!(got, want);
        assert_eq!(rows_of(&v, "replies")[0].detail, "1");
        assert!(v.act(FAREWELL).unwrap().enabled && v.act(CHOOSE).unwrap().enabled);
    }

    /// The clicks map to the services' own and drive them: the farewell reply (by index) and Farewell end the talk; an
    /// index past the replies and an unknown action map to nothing.
    #[test]
    fn the_actions_are_the_talk_boxs_clicks() {
        let (mut cx, mut host, i) = talking();
        assert!(talk_act(&cx, &host.svc, &click(CHOOSE, "99")).is_none() && talk_act(&cx, &host.svc, &click("nope", "")).is_none());
        let Some(a) = talk_act(&cx, &host.svc, &click(CHOOSE, &i.to_string())) else { panic!("the farewell reply maps") };
        assert!(matches!(a, SvcAct::Choose(n) if n == i as i64));
        host.svc.act(&mut cx, a);
        assert!(!host.svc.talking());
        assert!(talk_act(&cx, &host.svc, &click(FAREWELL, "")).is_none(), "nothing open");
        let (mut cx, mut host, _) = talking();
        let a = talk_act(&cx, &host.svc, &click(FAREWELL, "")).expect("Farewell maps");
        host.svc.act(&mut cx, a);
        assert!(!host.svc.talking() && !cx.busy());
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        let (cx, host, _) = talking();
        assert_eq!(shape(&fixture), shape(&view(&cx, &host.svc)));
    }

    /// Writes the fixture from the debug game's first person with a farewell (run by hand: `cargo test -p emberwake_gui
    /// write_talk_fixture -- --ignored`).
    #[test]
    #[ignore]
    fn write_talk_fixture() {
        let (cx, host, _) = talking();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/talk.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&view(&cx, &host.svc).to_json()).unwrap() + "\n").unwrap();
    }
}
