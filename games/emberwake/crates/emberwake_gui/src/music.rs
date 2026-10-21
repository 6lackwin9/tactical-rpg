//! Which music plays (scripts/audio/music_director.gd's choice, MUSIC1 rule 7 and MUSIC2): a screen names a context
//! (menu, field, travel, town, shop, danger, combat, encounter, boss) and its world and location; the context resolves
//! to the first track in its fallback chain that has a file under assets/audio/music, so a new track needs a file, no
//! code. Screens return the track from `Screen::music`; the engine's shell plays the topmost screen's choice.

use crate::screens::Cx;
use aegis_core::json::s;
use aegis_core::json::Dict;
use emberwake_world::{Data, GameState};

pub const MENU: &str = "menu";
pub const FIELD: &str = "field";
pub const TRAVEL: &str = "travel";
pub const TOWN: &str = "town";
pub const SHOP: &str = "shop";
pub const DANGER: &str = "danger";
pub const COMBAT: &str = "combat";
pub const BOSS: &str = "boss";
/// The menu context's last step.
const MENU_KEY: &str = "menu_theme";

/// Rule 7: the context tried next when none of a context's keys has a track.
fn parent(ctx: &str) -> Option<&'static str> {
    match ctx {
        TRAVEL | TOWN | DANGER => Some(FIELD),
        SHOP => Some(TOWN),
        "encounter" | BOSS => Some(COMBAT),
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

/// The first track of the context's chain that has a file (None: silence).
pub fn track(cx: &Cx, ctx: &str, world: &str, location: &str) -> Option<String> {
    candidates(&cx.data, ctx, world, location).into_iter().map(|k| path(&k)).find(|p| cx.audio.has(p))
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

    #[test]
    fn kinds_pick_the_region_context() {
        assert_eq!(kind_context("fortress"), TOWN);
        assert_eq!(kind_context("dungeon"), DANGER);
        assert_eq!(kind_context("gate"), FIELD);
    }
}
