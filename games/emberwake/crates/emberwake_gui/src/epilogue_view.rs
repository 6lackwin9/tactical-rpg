//! UISPLIT1-J (ENGINE-25) Stage A: the epilogue screen's view-model (`epilogue::EpilogueScreen`). What the open page
//! shows and offers, as a `screen_layout::View`, for a layout file to place later (Stage B); today's hand drawing stays.
//! Its fixture is `ui/views/epilogue.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` (the ending's title, toned by the ending's tone: `good` calm, `gold` warn, `foe` fatal, else none);
//! `page_kind` ("paragraph", "outcomes" or "fates"); `heading` ("Outcomes" or "Fates" on those pages, else ""); `body`
//! (the open paragraph, "" on another page); `outcomes` (the world's outcome lines, a row each, on the outcomes page);
//! `fates` (the page's fate cards, a row each, id the member: the name as the label, the tag as the detail toned `dim`
//! for a departed member and `good` otherwise, the template's line as the sub, the keepsake line as the note); `count`
//! ("2 / 5"). Actions: `back` (on past the first page), `next` (on before the last), `load_save` and `main_menu` (on
//! the last page only).

use crate::epilogue::{fate_tag, keepsake_line, page_count, EpilogueAct, EpilogueScreen, FATES_HEAD, OUTCOMES_HEAD};
use aegis_ui::screen_layout::{Act, Field, RowView, View};

pub const SCREEN: &str = "epilogue";
pub const BACK: &str = "back";
pub const NEXT: &str = "next";
pub const LOAD_SAVE: &str = "load_save";
pub const MAIN_MENU: &str = "main_menu";

/// The theme role of an ending's tone (`hud::tone_colour`'s colours as roles; "" the text colour).
fn tone_role(tone: &str) -> &'static str {
    match tone {
        "calm" => "good",
        "warn" => "gold",
        "fatal" => "foe",
        _ => "",
    }
}

/// The view of screen `e` on its open page.
pub fn view(e: &EpilogueScreen) -> View {
    let ep = e.epilogue();
    let kind = e.page_kind();
    let (page, pages) = e.page_at();
    let last = e.last();
    let heading = match kind {
        "outcomes" => OUTCOMES_HEAD,
        "fates" => FATES_HEAD,
        _ => "",
    };
    let outcomes: Vec<RowView> = if kind == "outcomes" { ep.outcomes.iter().enumerate().map(|(i, o)| RowView { id: i.to_string(), label: o.clone(), enabled: true, ..Default::default() }).collect() } else { Vec::new() };
    let fates: Vec<RowView> = e
        .page_fates()
        .iter()
        .map(|f| RowView {
            id: f.member.clone(),
            label: f.name.clone(),
            detail: fate_tag(f),
            tone: if f.departed { "dim".into() } else { "good".into() },
            sub: f.text.clone(),
            note: keepsake_line(f),
            enabled: true,
            ..Default::default()
        })
        .collect();
    View::new(SCREEN)
        .field("title", Field::Text(ep.title.clone()))
        .tone("title", &[tone_role(&ep.tone)])
        .field("page_kind", Field::Text(kind.into()))
        .field("heading", Field::Text(heading.into()))
        .field("body", Field::Text(e.page_text()))
        .field("outcomes", Field::Rows(outcomes))
        .field("fates", Field::Rows(fates))
        .field("count", Field::Text(page_count(page, pages)))
        .action(BACK, crate::epilogue::BACK, page > 0)
        .action(NEXT, crate::epilogue::NEXT, !last)
        .action(LOAD_SAVE, crate::runover::LOAD_SAVE, last)
        .action(MAIN_MENU, crate::runover::MAIN_MENU, last)
}

/// A view click as the screen's own (`EpilogueAct`); None for an unknown action or one that is off on the open page.
/// Stage B's layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn epilogue_act(e: &EpilogueScreen, a: &Act) -> Option<EpilogueAct> {
    let v = view(e);
    v.act(&a.action).filter(|x| x.enabled)?;
    match a.action.as_str() {
        BACK => Some(EpilogueAct::Back),
        NEXT => Some(EpilogueAct::Next),
        LOAD_SAVE => Some(EpilogueAct::LoadSave),
        MAIN_MENU => Some(EpilogueAct::MainMenu),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epilogue::FATES_PER_PAGE;
    use crate::hud::hud_tests::game_cx;
    use emberwake_world::campaign::ending::{Epilogue, FateCard};

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

    fn click(a: &str) -> Act {
        Act { action: a.into(), arg: None }
    }

    /// A sample epilogue: two paragraphs, two outcomes and five fates (one departed), so two fate pages.
    fn sample() -> EpilogueScreen {
        let fate = |i: usize, departed: bool| FateCard {
            member: format!("m{i}"),
            name: format!("Member {i}"),
            departed,
            cause: if departed { "fell".into() } else { String::new() },
            stage: 1,
            stage_name: "Sworn".into(),
            keepsake: if departed { "A ring".into() } else { String::new() },
            text: format!("Member {i} goes on."),
        };
        let ep = Epilogue {
            id: "e".into(),
            title: "The End".into(),
            tone: "calm".into(),
            art: String::new(),
            paragraphs: vec!["First.".into(), "Second.".into()],
            outcomes: vec!["The ward stands.".into(), "The guild falls.".into()],
            fates: (0..=FATES_PER_PAGE).map(|i| fate(i, i == 1)).collect(),
        };
        EpilogueScreen::with("e".into(), ep, String::new())
    }

    /// Each page's fields: the paragraphs, then the outcomes, then the fates four to a page with their tag, line and
    /// keepsake; Back off on the first page, Next on until the last, Load a save and Main menu only on the last.
    #[test]
    fn the_view_is_the_open_page_and_its_buttons() {
        let mut cx = game_cx();
        let mut e = sample();
        let v = view(&e);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "page_kind", "heading", "body", "outcomes", "fates", "count"]);
        assert_eq!((text(&v, "title"), v.tone_of("title", 0), text(&v, "body"), text(&v, "count")), ("The End", Some("good"), "First.", "1 / 5"));
        assert!(!v.act(BACK).unwrap().enabled && v.act(NEXT).unwrap().enabled && !v.act(LOAD_SAVE).unwrap().enabled);
        for _ in 0..2 {
            e.act(&mut cx, EpilogueAct::Next);
        }
        let v = view(&e);
        assert_eq!((text(&v, "page_kind"), text(&v, "heading"), text(&v, "body")), ("outcomes", OUTCOMES_HEAD, ""));
        assert_eq!(rows_of(&v, "outcomes").len(), 2);
        e.act(&mut cx, EpilogueAct::Next);
        let v = view(&e);
        let fates = rows_of(&v, "fates");
        assert_eq!(fates.len(), FATES_PER_PAGE);
        let gone = &fates[1];
        assert!(gone.tone == "dim" && gone.detail.starts_with(crate::epilogue::DEPARTED_TAG) && gone.note == "Keepsake: A ring", "{gone:?}");
        assert!(fates[0].tone == "good" && fates[0].note.is_empty() && fates[0].sub == "Member 0 goes on.");
        e.act(&mut cx, EpilogueAct::Next);
        let v = view(&e);
        assert_eq!((rows_of(&v, "fates").len(), text(&v, "count")), (1, "5 / 5"));
        assert!(!v.act(NEXT).unwrap().enabled && v.act(LOAD_SAVE).unwrap().enabled && v.act(MAIN_MENU).unwrap().enabled);
    }

    /// The clicks map to the screen's own while on; an off button and an unknown action map to nothing.
    #[test]
    fn the_actions_are_the_screens_clicks() {
        let mut cx = game_cx();
        let mut e = sample();
        assert!(epilogue_act(&e, &click(BACK)).is_none(), "off on the first page");
        let a = epilogue_act(&e, &click(NEXT)).expect("next maps");
        e.act(&mut cx, a);
        assert_eq!(e.page_at().0, 1);
        assert!(matches!(epilogue_act(&e, &click(BACK)), Some(EpilogueAct::Back)));
        assert!(epilogue_act(&e, &click(MAIN_MENU)).is_none() && epilogue_act(&e, &click("nope")).is_none());
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&sample())));
    }

    /// Writes the fixture from the sample's first fate page (run by hand: `cargo test -p emberwake_gui
    /// write_epilogue_fixture -- --ignored`).
    #[test]
    #[ignore]
    fn write_epilogue_fixture() {
        let mut cx = game_cx();
        let mut e = sample();
        for _ in 0..3 {
            e.act(&mut cx, EpilogueAct::Next);
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/epilogue.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&view(&e).to_json()).unwrap() + "\n").unwrap();
    }
}
