//! ROBUST1 (ENGINE-10): string tables over the asset layers: the merge, the fallback, the fill, the check.

use aegis_core::assets::Layers;
use aegis_core::scratch::Scratch;
use aegis_ui::strings::{self, Strings};

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
fn a_game_table_sets_its_keys_over_the_engine_and_a_missing_key_falls_back() {
    let engine = layer("strings_engine", &[("strings/en.json", r#"{"ok": "OK", "quit": "Quit", "hp": "{n} HP"}"#)]);
    let game = layer("strings_game", &[("strings/en.json", r#"{"quit": "Leave", "title": "A Game"}"#), ("strings/fr.json", r#"{"quit": "Partir", "title": "Un Jeu", "hp": "{n} PV"}"#)]);
    let layers = Layers::of(vec![game.path_buf(), engine.path_buf()]);
    let (en, p) = Strings::load(&layers, "en");
    assert!(p.is_empty(), "{p:?}");
    assert_eq!((en.get("ok"), en.get("quit"), en.get("title")), ("OK", "Leave", "A Game"));
    assert_eq!(en.get("nothing.here"), "nothing.here", "a missing key shows as itself");
    let (fr, _) = Strings::load(&layers, "fr");
    assert_eq!((fr.get("quit"), fr.get("ok")), ("Partir", "OK"), "a key the language lacks falls back to en");
    assert_eq!(fr.fmt("hp", &[("n", "12")]), "12 PV");
    let (de, p) = Strings::load(&layers, "de");
    assert_eq!(de.get("quit"), "Leave");
    assert!(p[0].contains("no string table for the language 'de'"));
    assert_eq!(strings::languages(&layers), ["en", "fr"]);
    // The check: fr lacks "ok" (only the game layer's fr counts with the engine's en), and every key is held to en.
    let problems = strings::check_layers(&layers).join(" | ");
    assert!(problems.contains("fr.json: no 'ok'"), "{problems}");
    strings::install(fr);
    assert_eq!(strings::tr("quit"), "Partir");
    assert_eq!(strings::tr_with("hp", &[("n", "3")]), "3 PV");
}

#[test]
fn the_check_wants_en_and_the_same_placeholders_and_no_extra_keys() {
    let t = layer("strings_check", &[("strings/en.json", r#"{"a": "{x} and {y}", "b": "B"}"#), ("strings/es.json", r#"{"a": "{x} y {z}", "b": "B", "c": "C"}"#), ("strings/it.json", "[1]")]);
    let got = strings::check_layers(&Layers::of(vec![t.path_buf()])).join(" | ");
    for want in ["es.json: 'a' fills [\"x\", \"z\"], en fills [\"x\", \"y\"]", "es.json: 'c' is not in en", "it.json: not a JSON object"] {
        assert!(got.contains(want), "{want} in {got}");
    }
    let only_fr = layer("strings_no_en", &[("strings/fr.json", r#"{"a": "A"}"#)]);
    assert!(strings::check_layers(&Layers::of(vec![only_fr.path_buf()]))[0].contains("no strings/en.json"));
    assert!(strings::check_layers(&Layers::of(vec![Scratch::new("strings_none").path_buf()])).is_empty(), "no tables, nothing to check");
    assert_eq!(strings::placeholders("{a} {b_2} {} {not one} {a}"), ["a", "b_2"]);
}
