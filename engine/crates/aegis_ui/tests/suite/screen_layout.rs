//! UISPLIT1 (PR7-38): the view and layout formats, the check, the default, the loader and a fixture drawn to a canvas.

use aegis_core::assets::Layers;
use aegis_core::scratch::Scratch;
use aegis_ui::screen_layout::{self, Field, Layout, Layouts, View};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn engine_assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

fn sample_view() -> View {
    screen_layout::load_fixture(&Layers::of(vec![engine_assets()]), "layout_sample").expect("the engine's sample fixture").expect("it parses")
}

fn problems_of(v: Value) -> String {
    Layout::from_json(&v).err().unwrap_or_default().join(" | ")
}

fn region(extra: Value) -> Value {
    let mut r = json!({"id": "a", "component": "text_block", "bind": {"text": "t"}});
    for (k, v) in extra.as_object().unwrap() {
        r[k] = v.clone();
    }
    json!({"screen": "s", "regions": [r]})
}

#[test]
fn a_view_reads_every_field_kind_and_writes_back_the_same() {
    let v = sample_view();
    let kinds: Vec<&str> = v.fields.iter().map(|(_, f)| f.kind()).collect();
    assert_eq!(kinds, ["text", "text", "tabs", "rows", "picture", "text", "stats", "meter"]);
    assert_eq!(View::from_json(&v.to_json()).unwrap(), v, "to_json and from_json round trip");
    let more = View::new("x").field("f", Field::Flag(true)).field("n", Field::Number(3.0)).field("i", Field::Icons(vec!["a.png".into()])).field("s", Field::Slots(vec![Default::default()]));
    assert_eq!(View::from_json(&more.to_json()).unwrap(), more);
    assert_eq!(Field::Number(3.0).as_text().as_deref(), Some("3"));
    let bad = View::from_json(&json!({"screen": "x", "fields": {"a": {"meter": [1, 0]}, "b": {"text": 1, "number": 2}, "c": {"nope": 1}}, "actions": [{"id": "has space"}], "extra": 1})).unwrap_err().join(" | ");
    for wanted in ["field 'a': 'meter'", "field 'b': a field is an object with one key", "field 'c': unknown field kind", "action 1", "unknown key 'extra'"] {
        assert!(bad.contains(wanted), "{wanted} in {bad}");
    }
}

#[test]
fn a_layout_file_names_each_problem_in_it() {
    assert!(Layout::from_json(&region(json!({}))).is_ok());
    let cases = [
        (region(json!({"component": "carousel"})), "unknown component 'carousel'"),
        (region(json!({"anchor": "middle"})), "'anchor' is one of"),
        (region(json!({"w": "150%"})), "a share is 1% to 100%"),
        (region(json!({"h": 0})), "whole number from 1 to 1000"),
        (region(json!({"x": 5000})), "'x' is an offset"),
        (region(json!({"bind": {}})), "text_block needs bind 'text'"),
        (region(json!({"bind": {"text": "t", "rows": "r"}})), "takes no bind 'rows'"),
        (region(json!({"in": "nowhere"})), "no earlier region"),
        (region(json!({"style": "popup"})), "a text_block's 'style' is muted or right, not 'popup'"),
        (region(json!({"colour": "#fff"})), "unknown key 'colour'"),
        (region(json!({"when": [{"h": 3}]})), "'text_scale_min' is a text scale"),
        (region(json!({"component": "button_bar", "bind": {"actions": "some"}})), "list of action ids or \"all\""),
        (json!({"screen": "s", "regions": []}), "not empty"),
        (json!({"screen": "s", "regions": [{"id": "a", "component": "custom", "bind": {"slot": "x"}}, {"id": "a", "component": "custom", "bind": {"slot": "y"}}]}), "another region has this id"),
    ];
    for (v, wanted) in cases {
        let got = problems_of(v.clone());
        assert!(got.contains(wanted), "{v}: wanted '{wanted}', got '{got}'");
    }
}

#[test]
fn the_check_holds_a_layout_to_its_view() {
    let v = sample_view();
    let layers = Layers::of(vec![engine_assets()]);
    let l = screen_layout::load_layout(&layers, "layout_sample").unwrap().unwrap();
    assert_eq!(l.check(&v), Vec::<String>::new(), "the engine's sample fits its fixture");
    let mut thin = v.clone();
    thin.fields.retain(|(n, _)| n != "goods");
    thin.actions.retain(|a| a.id != "leave");
    if let Some((_, f)) = thin.fields.iter_mut().find(|(n, _)| n == "stats") {
        *f = Field::Text("3".into());
    }
    let got = l.check(&thin).join(" | ");
    for wanted in ["region 'goods': the view has no field 'goods'", "region 'foot': the view offers no action 'leave'", "field 'stats' is text, not stats"] {
        assert!(got.contains(wanted), "{wanted} in {got}");
    }
    assert!(l.check(&View::new("other")).iter().any(|p| p.contains("the layout is for 'layout_sample'")));
}

/// A scratch asset layer with `files` (path under the layer, text).
fn layer(tag: &str, files: &[(&str, &str)]) -> Scratch {
    let t = Scratch::new(tag);
    for (rel, text) in files {
        let p = t.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    t
}

#[test]
fn the_gate_check_wants_a_fixture_for_each_layout_and_a_fit() {
    assert_eq!(screen_layout::check_layers(&Layers::of(vec![engine_assets()])), Vec::<String>::new());
    let lay = r#"{"screen": "shop", "regions": [{"id": "t", "component": "text_block", "bind": {"text": "gold"}}]}"#;
    let t = layer("layout_check", &[("ui/layouts/shop.json", lay), ("ui/layouts/menu.json", r#"{"screen": "menu", "regions": [{"id": "t", "component": "text_block", "bind": {"text": "x"}}]}"#), ("ui/views/menu.view.json", r#"{"screen": "menu", "fields": {"y": {"text": "a"}}}"#), ("ui/views/broken.view.json", "{")]);
    let got = screen_layout::check_layers(&Layers::of(vec![t.path_buf()])).join(" | ");
    for wanted in ["shop.json: no fixture ui/views/shop.view.json", "menu.json: region 't': the view has no field 'x'", "broken.view.json"] {
        assert!(got.contains(wanted), "{wanted} in {got}");
    }
    assert_eq!(screen_layout::screens(&Layers::of(vec![t.path_buf()])), ["broken", "menu", "shop"]);
    // A file whose `screen` is another's is refused by its name.
    let t = layer("layout_name", &[("ui/layouts/a.json", r#"{"screen": "b", "regions": [{"id": "t", "component": "custom", "bind": {"slot": "x"}}]}"#)]);
    assert!(screen_layout::load_layout(&Layers::of(vec![t.path_buf()]), "a").unwrap().unwrap_err()[0].contains("'screen' is 'b'"));
    // The first layer holding a file wins: a game's layout over the engine's.
    let game = layer("layout_over", &[("ui/layouts/layout_sample.json", r#"{"screen": "layout_sample", "regions": [{"id": "only", "component": "text_block", "bind": {"text": "title"}}]}"#)]);
    let l = screen_layout::load_layout(&Layers::of(vec![game.path_buf(), engine_assets()]), "layout_sample").unwrap().unwrap();
    assert_eq!(l.regions.len(), 1);
}

#[test]
fn the_loader_falls_back_to_the_default_once_per_change_and_says_why() {
    let mut l = Layouts::new(Layers::of(vec![engine_assets()]));
    let v = sample_view();
    assert!(l.get(&v).is_some());
    assert!(l.take_problems().is_empty());
    let mut thin = v.clone();
    thin.fields.retain(|(n, _)| n != "about");
    assert!(l.get(&thin).is_none(), "a layout that does not fit draws the default");
    let ps = l.take_problems();
    assert!(ps.len() == 1 && ps[0].contains("'about'") && ps[0].contains("drawn with the default"), "{ps:?}");
    assert!(l.get(&thin).is_none() && l.take_problems().is_empty(), "the same view says it once");
    assert!(l.get(&View::new("no_file")).is_none() && l.take_problems().is_empty(), "no file is the default, not a problem");
    assert!(l.get(&v).is_some(), "a view that fits again draws the layout again");
    assert!(l.take_problems().is_empty());
}

#[test]
fn a_fixture_draws_through_its_layout_and_through_the_default() {
    let layers = Layers::of(vec![engine_assets()]);
    let r = screen_layout::render(&layers, "layout_sample", (1280, 720), false).unwrap();
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    assert!(aegis_ui::shell::blank_reason(&r.canvas).is_none(), "the shot is not blank");
    for id in ["layout_sample.buy", "layout_sample.leave", "layout_sample.goods.0", "layout_sample.goods.2", "layout_sample.tabs.0", "layout_sample.load"] {
        assert!(r.widgets.iter().any(|w| w == id), "{id} in {:?}", r.widgets);
    }
    assert!(!r.widgets.iter().any(|w| w == "layout_sample.pick"), "the layout's footer names two actions only");
    let d = screen_layout::render(&layers, "layout_sample", (1280, 720), true).unwrap();
    for id in ["layout_sample.buy", "layout_sample.pick", "layout_sample.tab", "layout_sample.leave"] {
        assert!(d.widgets.iter().any(|w| w == id), "the default offers every action: {id} in {:?}", d.widgets);
    }
    assert_ne!(r.canvas.px, d.canvas.px, "the layout and the default differ");
    // Another size draws the same widgets.
    let big = screen_layout::render(&layers, "layout_sample", (1920, 1080), false).unwrap();
    assert_eq!(big.widgets, r.widgets);
    assert!(screen_layout::render(&layers, "nothing", (1280, 720), false).is_err());
}

#[test]
fn a_custom_region_hands_its_rect_back() {
    let t = layer("layout_custom", &[
        ("ui/layouts/board.json", r#"{"screen": "board", "regions": [{"id": "stage", "component": "custom", "bind": {"slot": "map"}, "anchor": "center", "w": "50%", "h": "50%"}]}"#),
        ("ui/views/board.view.json", r#"{"screen": "board"}"#),
    ]);
    let r = screen_layout::render(&Layers::of(vec![t.path_buf(), engine_assets()]), "board", (1280, 720), false).unwrap();
    assert_eq!(r.drawn.custom.get("map").map(|x| (x.w, x.h)), Some((640, 360)));
}

#[test]
fn disabled_or_missing_select_actions_draw_dead_items_and_region_ids_stay_apart_from_actions() {
    let mut v = sample_view();
    for a in &mut v.actions {
        if a.id == "pick" || a.id == "tab" {
            a.enabled = false;
        }
    }
    let t = layer("layout_dead", &[("ui/views/layout_sample.view.json", &v.to_json().to_string())]);
    let r = screen_layout::render(&Layers::of(vec![t.path_buf(), engine_assets()]), "layout_sample", (1280, 720), false).unwrap();
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    for id in ["layout_sample.goods.0", "layout_sample.tabs.0"] {
        assert!(r.inert.iter().any(|w| w == id), "{id} does nothing: {:?}", r.inert);
    }
    assert!(!r.inert.iter().any(|w| w == "layout_sample.buy"));
    let mut clash = v.clone();
    clash.actions.push(screen_layout::ActionView { id: "load".into(), label: "Load".into(), enabled: true });
    let l = screen_layout::load_layout(&Layers::of(vec![engine_assets()]), "layout_sample").unwrap().unwrap();
    assert!(l.check(&clash).iter().any(|p| p.contains("region 'load': an action has this id too")));
}

#[test]
fn a_when_override_takes_the_place_from_its_text_scale_up() {
    let l = Layout::from_json(&region(json!({"w": 10, "when": [{"text_scale_min": 1.25, "w": 20}, {"text_scale_min": 1.5, "w": 30}]}))).unwrap();
    let r = &l.regions[0];
    let w_at = |scale: f32| r.place_at(scale).w;
    assert_eq!(w_at(1.0), screen_layout::Size::Tokens(10));
    assert_eq!(w_at(1.3), screen_layout::Size::Tokens(20));
    assert_eq!(w_at(2.0), screen_layout::Size::Tokens(30), "the last matching override wins");
}
