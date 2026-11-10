//! Which music plays (scripts/audio/music_director.gd's choice, MUSIC1 rule 7 and MUSIC2): a screen names a context
//! (menu, field, travel, town, shop, danger, combat, encounter, boss) and its world and location; the context resolves
//! to the first track in its fallback chain that has a file under assets/audio/music, so a new track needs a file, no
//! code. Screens return the track from `Screen::music`; the engine's shell plays the topmost screen's choice.

use crate::screens::Cx;
use aegis_core::json::s;
use aegis_core::json::Dict;
use emberwake_world::{Data, GameState};
use emberwake_data::MusicAsk;

// STORYMUSIC1: game.json's `music_keys` lists these contexts and the story keys (the kit's content check of a story's
// music keys); a story test keeps it complete.
pub const MENU: &str = "menu";
pub const FIELD: &str = "field";
pub const TRAVEL: &str = "travel";
pub const TOWN: &str = "town";
pub const SHOP: &str = "shop";
pub const DANGER: &str = "danger";
pub const COMBAT: &str = "combat";
pub const ENCOUNTER: &str = "encounter";
pub const BOSS: &str = "boss";
/// The menu context's last step.
const MENU_KEY: &str = "menu_theme";

/// Rule 7: the context tried next when none of a context's keys has a track.
fn parent(ctx: &str) -> Option<&'static str> {
    match ctx {
        TRAVEL | TOWN | DANGER => Some(FIELD),
        SHOP => Some(TOWN),
        ENCOUNTER | BOSS => Some(COMBAT),
        _ => None,
    }
}

/// MUSIC2: a location kind's region map context; any other kind (gate, wild, none) plays field.
pub fn kind_context(kind: &str) -> &'static str {
    match kind {
        "town" | "fortress" => TOWN,
        "ruin" | "camp" | "dungeon" => DANGER,
        _ => FIELD,
    }
}

/// The region map's context: shop while its shop is open, else the location's kind.
pub fn region_context(d: &Data, gs: &GameState, shop_open: bool) -> &'static str {
    if shop_open {
        return SHOP;
    }
    if gs.location_id.is_empty() {
        return FIELD;
    }
    kind_context(&s(d.resolve(&gs.world_id, "locations", &gs.location_id), "kind"))
}

/// Rule 7: the keys a context tries, most specific first. Per context step: "<scope>_<context>" for every scope,
/// (field only) the plain scope keys, "<context>" alone, (menu only) menu_theme; then the parent context.
pub fn candidates(d: &Data, ctx: &str, world: &str, location: &str) -> Vec<String> {
    let sc = scopes(d, world, location);
    let mut out: Vec<String> = Vec::new();
    let mut add = |k: String| {
        if !k.is_empty() && !out.contains(&k) {
            out.push(k);
        }
    };
    let mut step = Some(ctx);
    let mut seen: Vec<&str> = Vec::new();
    while let Some(c) = step.filter(|c| !c.is_empty() && !seen.contains(c)) {
        seen.push(c);
        for x in &sc {
            add(format!("{x}_{c}"));
        }
        if c == FIELD {
            for x in &sc {
                add(x.clone());
            }
        }
        add(c.to_string());
        if c == MENU {
            add(MENU_KEY.to_string());
        }
        step = parent(c);
    }
    out
}

/// Rule 3 scopes, most specific first: the location's region, its parent chain, the world, then the world's
/// lineage (nearest ancestor first). No world: none.
pub fn scopes(d: &Data, world: &str, location: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if world.is_empty() {
        return out;
    }
    let loc = if location.is_empty() { None } else { Some(d.resolve(world, "locations", location)) };
    let home = loc.map(|l| s(l, "_world")).filter(|h| !h.is_empty()).unwrap_or_else(|| world.to_string());
    let mut region = find_region(d, world, &home, &loc.map(|l| s(l, "region")).unwrap_or_default());
    for _ in 0..32 {
        let Some(r) = region else { break };
        let id = s(r, "id");
        if id.is_empty() || out.contains(&id) {
            break;
        }
        out.push(id);
        let home = s(r, "_world");
        region = find_region(d, world, if home.is_empty() { world } else { &home }, &s(r, "parent"));
    }
    for w in std::iter::once(world.to_string()).chain(d.lineage(world).into_iter().rev()) {
        if !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

/// A region ref seen from the game world, else from the world that wrote it (a connected world's entry).
fn find_region<'a>(d: &'a Data, world: &str, home: &str, r: &str) -> Option<&'a Dict> {
    if r.is_empty() {
        return None;
    }
    let hit = d.resolve(world, "regions", r);
    let hit = if hit.is_empty() && home != world { d.resolve(home, "regions", r) } else { hit };
    (!hit.is_empty()).then_some(hit)
}

/// The music file of a key.
pub fn path(key: &str) -> String {
    format!("music/{key}.ogg")
}

/// The first track of the context's chain that has a file (None: silence), before its variant is picked.
fn chain_track(cx: &Cx, ctx: &str, world: &str, location: &str) -> Option<String> {
    candidates(&cx.data, ctx, world, location).into_iter().map(|k| path(&k)).find(|p| cx.audio.has(p))
}

/// The track of the context's chain (`chain_track`), as one of its variants when `<key>_2.ogg`, ... ship (MUSICVAR1):
/// a request lasts while the track plays, seeded by the game's seed (0 without a game).
pub fn track(cx: &Cx, ctx: &str, world: &str, location: &str) -> Option<String> {
    let seed = cx.game.as_ref().map_or(0, |g| g.gs.rng_seed);
    chain_track(cx, ctx, world, location).map(|p| cx.audio.music_variant(&p, None, seed))
}

/// MUSICVAR1: the track of one battle (`request`, from `new_request` when it starts), its variant seeded by the
/// battle's seed: the battle keeps its pick, the next battle picks again and never the same one.
pub fn battle_track(cx: &Cx, ctx: &str, world: &str, location: &str, request: u64, seed: i64) -> Option<String> {
    chain_track(cx, ctx, world, location).map(|p| cx.audio.music_variant(&p, Some(request), seed))
}

/// A new music request id (one per battle start; ids only need to differ).
pub fn new_request() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// STORYMUSIC1: what a vignette's or a sequence step's `music` asks the story stage for (V8's wish): None keeps the
/// music playing (""; a music key with no track yet), `Some("")` is silence (`none`), else the track: a track path as
/// written, a music key (a context, a story key such as `story_opening`, or a track's key) through `game_track`, so it
/// takes the game's scopes (`<region>_story_opening` first) and its MUSICVAR1 variants.
pub fn story_wish(cx: &Cx, music: &str) -> Option<String> {
    match emberwake_data::music_ask(music) {
        MusicAsk::Keep => None,
        MusicAsk::Silence => Some(String::new()),
        MusicAsk::Track(p) => Some(p.to_string()),
        MusicAsk::Key(k) => game_track(cx, k),
    }
}

/// STORYMUSIC1: a reel's `music` as the engine's player reads it ("", `none` or a track): a music key becomes its
/// track (`story_wish`), or "" (the music goes on) while it has none.
pub fn reel_music(cx: &Cx, music: &str) -> String {
    match emberwake_data::music_ask(music) {
        MusicAsk::Key(_) => story_wish(cx, music).unwrap_or_default(),
        _ => music.to_string(),
    }
}

/// `ctx` for the game in play (its world and location); without a game, the context alone.
pub fn game_track(cx: &Cx, ctx: &str) -> Option<String> {
    match cx.game.as_ref() {
        Some(g) => track(cx, ctx, &g.gs.world_id, &g.gs.location_id),
        None => track(cx, ctx, "", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_ui::audio::Audio;

    fn data() -> Data {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        Data::open(&dir).expect("data").0
    }

    #[test]
    fn candidates_without_a_world_walk_the_context_chain() {
        let d = data();
        assert_eq!(candidates(&d, MENU, "", ""), ["menu", "menu_theme"]);
        assert_eq!(candidates(&d, SHOP, "", ""), ["shop", "town", "field"]);
        assert_eq!(candidates(&d, BOSS, "", ""), ["boss", "combat"]);
        assert_eq!(candidates(&d, TRAVEL, "", ""), ["travel", "field"]);
    }

    #[test]
    fn a_world_field_plays_the_world_track_before_field() {
        let d = data();
        let c = candidates(&d, FIELD, "ashlands", "");
        let at = |k: &str| c.iter().position(|x| x == k).unwrap();
        assert!(at("ashlands_field") < at("ashlands") && at("ashlands") < at("field"));
        assert_eq!(candidates(&d, TOWN, "ashlands", "").last().map(String::as_str), Some("field"));
    }

    #[test]
    fn every_shipped_sound_decodes_and_every_context_has_a_track() {
        // ASSETORG2: the game's, the kit's and the engine's audio folders
        let game = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let roots: Vec<std::path::PathBuf> = aegis_core::assets::Layers::for_game(&game).roots.iter().map(|r| r.join("audio")).collect();
        let audio = Audio::offline_layered(roots.clone());
        let mut n = 0;
        for root in &roots {
            for folder in ["music", "sfx"] {
                for e in std::fs::read_dir(root.join(folder)).into_iter().flatten().flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".ogg") || name.ends_with(".wav") {
                        audio.check(&format!("{folder}/{name}")).unwrap();
                        n += 1;
                    }
                }
            }
        }
        assert!(n > 0, "no sound files");
        let d = data();
        for ctx in [MENU, FIELD, TRAVEL, TOWN, SHOP, DANGER, COMBAT, BOSS] {
            assert!(candidates(&d, ctx, "ashlands", "").iter().any(|k| audio.has(&path(k))), "{ctx} has no track");
        }
    }

    #[test]
    fn the_region_map_plays_its_location_kind_or_the_shop() {
        let d = data();
        let gs = GameState { world_id: "ashlands".into(), location_id: "warden_keep".into(), ..GameState::default() };
        assert_eq!(region_context(&d, &gs, false), TOWN, "warden_keep is a fortress");
        assert_eq!(region_context(&d, &gs, true), SHOP);
        assert_eq!(region_context(&d, &GameState { location_id: String::new(), ..gs }, false), FIELD);
    }

    /// MUSICVAR1 outside battle: a context's track with variants plays one while it lasts; after other music the next
    /// request picks another. A plain track (no variants) is unchanged.
    #[test]
    fn a_context_track_with_variants_picks_one_per_request() {
        let dir = aegis_core::scratch::Scratch::new("emberwake_music_variants");
        std::fs::create_dir_all(dir.join("music")).unwrap();
        for f in ["menu_theme.ogg", "menu_theme_2.ogg", "travel.ogg"] {
            std::fs::write(dir.join("music").join(f), b"OggS").unwrap();
        }
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.audio = Audio::offline(Some(dir.to_path_buf()));
        assert_eq!(track(&cx, TRAVEL, "", "").as_deref(), Some("music/travel.ogg"));
        let first = track(&cx, MENU, "", "").expect("a menu track");
        cx.audio.music(Some(&first));
        assert_eq!(track(&cx, MENU, "", "").as_ref(), Some(&first), "the playing pick stays");
        cx.audio.music(Some("music/travel.ogg"));
        let second = track(&cx, MENU, "", "").expect("a menu track");
        assert_ne!(second, first, "back from other music, the other variant");
        assert!(["music/menu_theme.ogg", "music/menu_theme_2.ogg"].contains(&second.as_str()));
    }

    #[test]
    fn kinds_pick_the_region_context() {
        assert_eq!(kind_context("fortress"), TOWN);
        assert_eq!(kind_context("dungeon"), DANGER);
        assert_eq!(kind_context("gate"), FIELD);
    }
}
