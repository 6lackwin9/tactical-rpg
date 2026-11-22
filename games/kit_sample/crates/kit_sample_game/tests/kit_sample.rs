//! ENGINE-42 (KIT-SAMPLE1): the kit sample game's folder, data and fight. The screens are walked end to end by the
//! scenario `kit_sample_fight` (`tools/fc.sh gate`, step `scenarios_all`).

use kit_sample_game::battle::{Fight, ENCOUNTER, STAGE_SET};
use kit_sample_game::GAME_ID;
use serde_json::json;
use std::path::Path;

fn game_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn data() -> rpgkit_world::Data {
    rpgkit_world::Data::open(&game_dir().join("data")).expect("the data folder opens").0
}

#[test]
fn the_game_folder_holds_its_manifest_and_valid_scenarios() {
    let g = game_dir();
    let m = aegis_content::manifest::Manifest::read(&g.join("game.json")).unwrap();
    assert_eq!(m.id, GAME_ID);
    let files = aegis_cli::scenario::files_in(&g.join("scenarios"));
    assert!(!files.is_empty());
    let problems = aegis_cli::scenario::check_files(&files);
    assert!(problems.is_empty(), "{problems:#?}");
    let layers = aegis_core::assets::Layers::for_game(&g);
    for rel in [kit_sample_game::sound::PRESS, kit_sample_game::sound::BACK] {
        assert!(layers.has(&format!("audio/{rel}")), "{rel}");
    }
}

/// The data folder passes the kit's whole content check (what `kit_sample_cli content check` runs), and every unit's
/// body, the decor's prop and the stage set's floor sheet are found over the layers: the game's, then the kit's
/// library, then the engine's.
#[test]
fn the_data_passes_the_kits_check_and_its_art_is_found() {
    use aegis_content::store::Hooks;
    let g = game_dir();
    let problems = rpgkit_content::store::RpgContent.check(&g.join("data"));
    assert!(problems.is_empty(), "{problems:#?}");
    let d = data();
    let layers = aegis_core::assets::Layers::for_game(&g);
    assert!(layers.roots.iter().any(|r| r.ends_with("kits/rpg/assets")), "the manifest's kit adds its library");
    for id in ["ks_guard", "ks_archer", "ks_raider", "ks_brute"] {
        let art = &d.cat.units[id].art;
        assert!(layers.has(&format!("units/{art}/sheet.png")), "{id}: {art}");
    }
    assert!(layers.has("props/common/fire_pit/v0.png"), "the decor's prop, from the kit library");
    assert!(layers.has(&format!("tilesets/{STAGE_SET}/floor/sheet.png")) && layers.has(&format!("tilesets/{STAGE_SET}/stage.fx.json")));
}

/// The fight is the kit's session: two a side, every round drafted by the auto plan, and it ends inside its round
/// limit with an outcome; the same seed plays the same fight.
#[test]
fn the_two_a_side_fight_runs_to_an_outcome_the_same_each_time() {
    let d = data();
    let play = || {
        let mut f = Fight::start(&d).expect(ENCOUNTER);
        assert_eq!((f.s.w.allies.len(), f.s.w.enemies.len()), (2, 2));
        assert!(!f.over(), "a new fight is not over");
        let rounds = f.to_end();
        assert!(f.over() && rounds >= 1 && rounds <= f.s.def.max_rounds, "{rounds}");
        assert!(!f.s.outcome.is_empty());
        f.state()
    };
    let a = play();
    assert_eq!(a, play(), "one seed, one fight");
    assert_eq!(a["over"], json!(true));
}

/// The stage set's look: one zone's tint and light spot, and the fire pit casting light.
#[test]
fn the_stage_set_names_one_zone_and_a_lit_fire_pit() {
    let mut art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::for_game(&game_dir()));
    let look = rpgkit_ui::stage::zone_look(&mut art, STAGE_SET, "stone_arena");
    assert!(look.tint.is_some(), "the zone's tint");
    assert!(look.lights.iter().any(|l| l.key == "fire_pit"), "{:?}", look.lights);
    assert_eq!(rpgkit_ui::stage::light_spots(&mut art, STAGE_SET, "stone_arena").len(), 1);
}

#[test]
fn the_command_host_starts_a_game_and_replays_it() {
    use aegis_host::{Driver, Host};
    let mut h = Host::new(kit_sample_game::Commands::default());
    assert_eq!(h.send(&json!({"cmd": "state"}))["started"], json!(false));
    assert_eq!(h.send(&json!({"cmd": "new"}))["started"], json!(true));
    assert_eq!(h.history().len(), 1);
    assert_eq!(h.send(&json!({"cmd": "fly"}))["ok"], json!(false));
}
