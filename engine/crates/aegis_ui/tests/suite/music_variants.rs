//! MUSICVAR1 (1): music variants. A track `music/<key>.ogg` with `<key>_2.ogg`, `<key>_3.ogg`, ... beside it is a pool;
//! each music request plays one, picked from the seeded stream `VARIANT_STREAM` (its own generator, so no gameplay roll
//! moves) and never the previous request's pick. The files here are stand-ins: the pick only lists names, it decodes
//! nothing.

use aegis_ui::audio::{pick_variant, Audio};
use std::path::Path;

fn touch(dir: &Path, rel: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, b"OggS").unwrap();
}

/// A music folder with `combat` and its variants 2, 3 and 5 (4 missing), plus names that are not its variants.
fn pool_dir(name: &str) -> aegis_core::scratch::Scratch {
    let dir = aegis_core::scratch::Scratch::new(name);
    for f in ["combat.ogg", "combat_2.ogg", "combat_3.ogg", "combat_5.ogg", "combat_05.ogg", "combat_1.ogg", "combat_x.ogg", "combat_2.wav", "combat_boss.ogg", "boss.ogg"] {
        touch(&dir, &format!("music/{f}"));
    }
    dir
}

#[test]
fn the_variant_pick_is_seeded() {
    for count in 2..6 {
        let mut seen = vec![false; count];
        for seed in 0..64 {
            let first = pick_variant(count, seed, None);
            assert_eq!(first, pick_variant(count, seed, None), "the same seed picks the same variant");
            assert!(first < count);
            seen[first] = true;
            for last in 0..count {
                assert_eq!(pick_variant(count, seed, Some(last)), pick_variant(count, seed, Some(last)));
            }
        }
        assert!(seen.iter().all(|&s| s), "over many seeds every variant of {count} is picked: {seen:?}");
    }
    assert_eq!(pick_variant(1, 7, None), 0, "a single track is its own pick");
    assert_eq!(pick_variant(0, 7, Some(3)), 0);
}

#[test]
fn a_new_pick_never_repeats_the_last() {
    for count in 2..6 {
        for seed in -20..200 {
            for last in 0..count {
                let pick = pick_variant(count, seed, Some(last));
                assert!(pick < count && pick != last, "{count} variants, seed {seed}: picked the last ({last}) again");
            }
            let out_of_pool = pick_variant(count, seed, Some(count + 3));
            assert_eq!(out_of_pool, pick_variant(count, seed, None), "a last pick outside the pool is no last pick");
        }
    }
}

#[test]
fn the_pool_is_the_track_and_its_numbered_variants_skipping_missing_numbers() {
    let dir = pool_dir("music_pool");
    let a = Audio::offline(Some(dir.to_path_buf()));
    assert_eq!(a.music_variants("music/combat.ogg"), ["music/combat.ogg", "music/combat_2.ogg", "music/combat_3.ogg", "music/combat_5.ogg"]);
    assert_eq!(a.music_variants("music/boss.ogg"), ["music/boss.ogg"], "no variants: the track alone");
    assert!(a.music_variants("music/none.ogg").is_empty(), "no file: no pool");
    assert_eq!(a.music_variant("music/boss.ogg", Some(1), 9), "music/boss.ogg");
    assert_eq!(a.music_variant("music/none.ogg", Some(1), 9), "music/none.ogg", "a missing track is passed through");
    assert!(a.has("music/combat_5.ogg"), "a listed variant is known to exist");
}

#[test]
fn variants_come_from_the_folder_that_holds_the_track() {
    let base = aegis_core::scratch::Scratch::new("music_pool_layers");
    let (game, kit) = (base.join("game"), base.join("kit"));
    touch(&game, "music/combat.ogg");
    touch(&game, "music/combat_3.ogg");
    touch(&kit, "music/combat.ogg");
    touch(&kit, "music/combat_2.ogg");
    let a = Audio::offline_layered(vec![game, kit]);
    assert_eq!(a.music_variants("music/combat.ogg"), ["music/combat.ogg", "music/combat_3.ogg"], "the kit's variants do not mix into the game's track");
}

#[test]
fn a_request_keeps_its_pick_and_the_next_request_picks_another() {
    let dir = pool_dir("music_requests");
    let a = Audio::offline(Some(dir.to_path_buf()));
    let pool = a.music_variants("music/combat.ogg");
    let mut picks = Vec::new();
    for request in 1..=12u64 {
        let seed = 1337;   // every battle may share a seed: the previous pick still changes the next
        let pick = a.music_variant("music/combat.ogg", Some(request), seed);
        assert!(pool.contains(&pick));
        a.music(Some("music/menu.ogg"));   // other music between battles changes nothing for a named request
        assert_eq!(a.music_variant("music/combat.ogg", Some(request), seed), pick, "the same battle keeps its variant");
        if let Some(last) = picks.last() {
            assert_ne!(&pick, last, "request {request} repeated the last pick");
        }
        picks.push(pick);
    }
    // deterministic: a fresh player given the same requests picks the same tracks (the ids themselves do not matter)
    let b = Audio::offline(Some(dir.to_path_buf()));
    let again: Vec<String> = (0..12u64).map(|r| b.music_variant("music/combat.ogg", Some(100 + r), 1337)).collect();
    assert_eq!(again, picks);
    // the first pick is the seed's own
    let c = Audio::offline(Some(dir.to_path_buf()));
    assert_eq!(c.music_variant("music/combat.ogg", Some(1), 42), pool[pick_variant(pool.len(), 42, None)]);
}

#[test]
fn an_unnamed_request_lasts_while_its_track_plays() {
    let dir = pool_dir("music_auto");
    let a = Audio::offline(Some(dir.to_path_buf()));
    let first = a.music_variant("music/combat.ogg", None, 5);
    a.music(Some(&first));
    for _ in 0..3 {
        assert_eq!(a.music_variant("music/combat.ogg", None, 5), first, "asked every frame, the playing pick stays");
        a.music(Some(&first));
    }
    a.music(Some("music/boss.ogg"));
    let second = a.music_variant("music/combat.ogg", None, 5);
    assert_ne!(second, first, "after other music a new request picks another variant");
    assert_eq!(a.music_variant("music/combat.ogg", None, 5), second, "asked again before it plays, the pick stays");
    a.music(Some(&second));
    let third = a.music_variant("music/combat.ogg", None, 6);
    assert_ne!(third, second, "a new seed is a new request");
    assert_eq!(a.music_variant("music/combat.ogg", None, 6), third);
}

/// Review M1: once a pick has played, its unnamed request is over when anything else is wanted, even the music that was
/// wanted when it was picked (X, Y, X, Y) or silence (Y, silence, Y): the next request picks another variant.
#[test]
fn an_unnamed_request_ends_after_its_pick_played_even_back_to_the_music_before_it() {
    let dir = pool_dir("music_auto_back");
    let a = Audio::offline(Some(dir.to_path_buf()));
    // the shell's frame: the screen asks for its track, then the player wants it
    let frame = |rel: Option<&str>| match rel {
        Some("music/combat.ogg") => {
            let pick = a.music_variant("music/combat.ogg", None, 5);
            a.music(Some(&pick));
            Some(pick)
        }
        other => {
            a.music(other);
            None
        }
    };
    for between in [Some("music/boss.ogg"), None] {
        frame(between);
        let first = frame(Some("music/combat.ogg")).unwrap();
        assert_eq!(frame(Some("music/combat.ogg")).unwrap(), first, "it stays while it plays");
        frame(between);
        let second = frame(Some("music/combat.ogg")).unwrap();
        assert_ne!(second, first, "back after {between:?}: a new request, another variant");
        frame(between);
        let third = frame(Some("music/combat.ogg")).unwrap();
        assert_ne!(third, second, "and again after {between:?}");
    }
}
