//! UISPLIT1-I (ENGINE-23) Stage A: the facility pane's view-model (`facility::FacilityPane`). What the pane shows and
//! offers, as a `screen_layout::View`, for a layout file to place later (Stage B, its own row once the mock is signed
//! off); today's hand drawing stays. Its fixture is `ui/views/facility.view.json`
//! (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the keeper's name); `facilities` (a row per facility the keeper keeps: its name, "Level N of M" as
//! the detail (in `good` at the top level), the effects in force as the sub, the next level's cost as the second
//! detail ("" at the top), the hover card's line as the note (the next level while it can be bought, else why not, or
//! "Highest level reached."), an Upgrade button while a level is left, on while it can be bought); `line` (the last
//! upgrade's refusal in `foe`, else its notes in `gold`). Actions: `upgrade` (a row's Upgrade, the facility as arg)
//! and `close`.

use crate::facility::{cost_line, level_line, next_line, now_line, FacilityAct, FacilityPane};
use crate::screens::Cx;
use aegis_core::json::{self, s};
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_world::facility;

pub const SCREEN: &str = "facility";
pub const UPGRADE: &str = "upgrade";
pub const CLOSE: &str = "close";

/// The view of character `npc`'s facilities with the pane's last upgrade.
pub fn view(cx: &Cx, pane: &FacilityPane, npc: &str) -> View {
    let (d, gs) = (&cx.data, cx.gs());
    let name = json::s_or(d.resolve(&gs.world_id, "characters", npc), "name", npc);
    let rows: Vec<RowView> = facility::view(d, gs, npc).iter().map(json::dict).map(facility_row).collect();
    let (line, tone) = if pane.errors.is_empty() { (pane.notes.join(" "), "gold") } else { (pane.errors.join(" "), "foe") };
    View::new(SCREEN)
        .field("title", Field::Text(name))
        .field("facilities", Field::Rows(rows))
        .field("line", Field::Text(line.clone()))
        .tone("line", &[if line.is_empty() { "" } else { tone }])
        .action(UPGRADE, "Upgrade", true)
        .action(CLOSE, "Close", true)
}

/// A facility's row: name, level of top, effects in force, the next level's cost, the hover card's line, Upgrade.
fn facility_row(r: &json::Dict) -> RowView {
    let top = json::dget(r, "next").is_empty();
    let ready = json::b(r, "ready", false);
    let reason = s(r, "reason");
    RowView {
        id: s(r, "id"),
        label: s(r, "name"),
        detail: level_line(r),
        detail2: cost_line(r),
        sub: now_line(r),
        enabled: ready,
        tone: if top { "good".into() } else { String::new() },
        tone2: if !top && !reason.is_empty() { "foe".into() } else { String::new() },
        note: if top { "Highest level reached.".into() } else if ready { next_line(r) } else { reason },
        buttons: if top { Vec::new() } else { vec![RowButton { action: UPGRADE.into(), label: "Upgrade".into(), enabled: ready }] },
        ..Default::default()
    }
}

/// A view click as the pane's own (`FacilityAct`); None for an unknown action or an upgrade with no facility. Stage B's
/// layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn facility_act(a: &Act) -> Option<FacilityAct> {
    match a.action.as_str() {
        UPGRADE => a.arg.clone().filter(|id| !id.is_empty()).map(FacilityAct::Upgrade),
        CLOSE => Some(FacilityAct::Close),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facility::tests::{data, FUNDS, GATE};
    use crate::screens::enter_game;
    use crate::story::tests::story_cx;
    use emberwake_world::new_game;

    const NPC: &str = "ward:smith";

    /// A cx in a new game of the scratch world `ward` with `FUNDS` gold.
    fn cx_on() -> Cx {
        let mut cx = story_cx(data());
        let mut gs = new_game::create_for_world(&cx.data.clone(), "ward", 1).expect("a new game");
        gs.party.gold = FUNDS;
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

    /// Each facility a row (level of top, effects in force, the next level's cost and line, Upgrade on while it can be
    /// bought, its reason in foe otherwise, no Upgrade at the top); the line toned as drawn; a click maps back.
    #[test]
    fn the_view_is_the_keepers_facilities_and_line() {
        let mut cx = cx_on();
        let mut pane = FacilityPane::default();
        let v = view(&cx, &pane, NPC);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "facilities", "line"]);
        assert_eq!(text(&v, "title"), "Smith");
        let rows = rows_of(&v, "facilities");
        let forge = rows.iter().find(|r| r.id == "ward:forge").expect("the forge");
        assert_eq!((forge.label.as_str(), forge.detail.as_str(), forge.sub.as_str()), ("The Forge", "Level 1 of 3", "No effects yet."));
        assert!(forge.enabled && forge.buttons[0].enabled && forge.detail2.starts_with("Cost: ") && forge.note.starts_with("Next: level 2 Warm Hearth"), "{forge:?}");
        let shed = rows.iter().find(|r| r.id == "ward:shed").expect("the shed");
        assert!(shed.buttons.is_empty() && shed.tone == "good" && shed.note == "Highest level reached." && shed.detail2.is_empty(), "{shed:?}");
        // after an upgrade: level 2 in force, level 3 gated (its reason in foe and as the note), the line in gold
        assert!(!pane.act(&mut cx, FacilityAct::Upgrade("ward:forge".into())));
        let v = view(&cx, &pane, NPC);
        let forge = rows_of(&v, "facilities").iter().find(|r| r.id == "ward:forge").expect("the forge").clone();
        assert!(forge.sub.starts_with("Now: ") && !forge.enabled && !forge.buttons[0].enabled, "{forge:?}");
        assert_eq!((forge.note.as_str(), forge.tone2.as_str()), (facility::NOT_YET_TEXT, "foe"));
        assert!(text(&v, "line").contains("level 2") && v.tone_of("line", 0) == Some("gold"));
        // the gate open: the next level can be bought once paid for; a refusal shows in foe
        cx.gs_mut().flags.insert(GATE.into(), true.into());
        pane.act(&mut cx, FacilityAct::Upgrade("ward:forge".into()));
        let v = view(&cx, &pane, NPC);
        assert!(!pane.errors.is_empty() && v.tone_of("line", 0) == Some("foe"), "{:?}", pane.errors);
        // clicks
        let act = |a: &str, arg: Option<&str>| facility_act(&Act { action: a.into(), arg: arg.map(str::to_string) });
        assert!(matches!(act(UPGRADE, Some("ward:forge")), Some(FacilityAct::Upgrade(id)) if id == "ward:forge"));
        assert!(matches!(act(CLOSE, None), Some(FacilityAct::Close)));
        assert!(act(UPGRADE, None).is_none() && act("nope", None).is_none());
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&cx_on(), &FacilityPane::default(), NPC)));
    }

    /// Writes the fixture from the scratch smith (run by hand: `cargo test -p emberwake_gui write_facility_fixture --
    /// --ignored`), so it holds the sample's values.
    #[test]
    #[ignore]
    fn write_facility_fixture() {
        let v = view(&cx_on(), &FacilityPane::default(), NPC).to_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/facility.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}
