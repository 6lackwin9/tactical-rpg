//! LOOK1-FT: the frame tuner on the shipped frames, read only: what the tuner finds on `ornate` and `ornate_full` matches
//! what `tools/gen_art.py` builds them from (a tip on a stub and a plain line; an arm tiled into the kit's edge, no tip), and
//! the writer's text of each shipped sidecar is that file, byte for byte, so a Save only ever touches the keys it changes.

use aegis_core::assets::Layers;
use aegis_ui::frame_tuner::{find_tip, sidecar_text};
use aegis_ui::painter::nine_art_line;
use aegis_ui::theme::{self, FRAME_ROLES};
use serde_json::Value;
use std::path::Path;

fn layers() -> Layers {
    Layers::for_game(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap())
}

#[test]
fn the_tuner_reads_the_shipped_ornate_frames() {
    let t = theme::load(&layers()).unwrap();
    let ornate = &t.frames["ornate"];
    let tip = find_tip(&ornate.img, &ornate.nine);
    let [from, half] = tip.expect("the ornate corner has a tip");
    let [ax, _] = ornate.nine.corner_anchor.unwrap();
    assert!(from > ax && from < ornate.nine.slice[0], "the head is on the arm, inside the slice");
    assert!(half >= 1);
    let (width, colour) = nine_art_line(&ornate.img, &ornate.nine).expect("the ornate frame has a line");
    assert_eq!(width, 1.0, "1 logical px");
    assert_eq!(colour, t.frame_line);
    let full = &t.frames["ornate_full"];
    assert_eq!(find_tip(&full.img, &full.nine), None, "its arms run into the kit's edge");
    assert_eq!(nine_art_line(&full.img, &full.nine), None, "its edge is tiled art");
    for f in ["panel", "button", "card"] {
        assert_eq!(nine_art_line(&t.frames[f].img, &t.frames[f].nine), None, "{f} draws inside its rect");
    }
}

#[test]
fn the_writers_text_of_each_shipped_sidecar_is_that_file() {
    let layers = layers();
    for (role, _) in FRAME_ROLES {
        let Some(file) = layers.find(&format!("{}/{role}.json", theme::FRAMES_DIR)) else { continue };
        let text = std::fs::read_to_string(&file).unwrap();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(sidecar_text(&value) + "\n", text, "{role}");
    }
}
