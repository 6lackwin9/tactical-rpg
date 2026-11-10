//! MUSICVAR1: a battle plays one variant of its track (`music/combat.ogg` with `combat_2.ogg`, ... beside it), picked
//! from the battle's seed on the music variant stream; it keeps that pick while it lasts, and the next battle picks
//! again, never the one before. Stand-in files in a scratch audio folder (the pick lists names, it decodes nothing).

use crate::battle::BattleScreen;
use crate::screens::Screen;
use aegis_ui::audio::{pick_variant, Audio};

#[test]
fn a_battle_keeps_its_music_variant_and_the_next_battle_picks_another() {
    let dir = aegis_core::scratch::Scratch::new("emberwake_battle_music");
    std::fs::create_dir_all(dir.join("music")).unwrap();
    for f in ["combat.ogg", "combat_2.ogg", "combat_3.ogg"] {
        std::fs::write(dir.join("music").join(f), b"OggS").unwrap();
    }
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.audio = Audio::offline(Some(dir.to_path_buf()));
    cx.app.encounter = "goblin_ambush".into();
    cx.app.seed = Some(1);
    let mut b = BattleScreen::sandbox(&cx);
    assert!(b.host.session.is_some(), "the arena battle starts: {}", b.message);
    let pool = cx.audio.music_variants("music/combat.ogg");
    assert_eq!(pool.len(), 3);
    let seed = b.host.session.as_ref().unwrap().def.rng_seed;
    let first = b.music(&cx).expect("the battle has music");
    assert_eq!(first, pool[pick_variant(pool.len(), seed, None)], "the first pick is the battle seed's");
    let mut last = first;
    for _ in 0..6 {
        cx.audio.music(Some(&last));
        assert_eq!(b.music(&cx).as_ref(), Some(&last), "the battle keeps its variant frame after frame");
        b.start_battle();
        let next = b.music(&cx).expect("the battle has music");
        assert!(pool.contains(&next));
        assert_ne!(next, last, "a new battle (same seed) never repeats the last pick");
        last = next;
    }
}
