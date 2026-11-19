//! PR7-36 (STYLE-PASS1, ART-01, ART-17): the game's named colour tokens that are not engine roles, read from the
//! theme's `game_colors` (`ui/theme.json`). Every fallback is the colour the screens drew before the token, so the
//! default theme (no file) draws as it did; the shipped theme sets each one in the style bible's palette.

use aegis_ui::canvas::{rgb, rgba, Rgba};
use aegis_ui::theme::th;

/// A hero's name, ring and timeline colour (was light blue; parchment in the shipped theme).
pub fn ally() -> Rgba {
    th().game("ally", rgb(110, 170, 255))
}

/// HP bars of heroes (a muted moss: saturation at most 0.55).
pub fn hp() -> Rgba {
    th().game("hp", rgb(96, 210, 110))
}

/// AP pips and AP numbers (a muted cold iron).
pub fn ap() -> Rgba {
    th().game("ap", rgb(110, 160, 235))
}

/// A tile a hero can move to: its fill (low alpha) and its edge.
pub fn move_tile() -> (Rgba, Rgba) {
    (th().game("move_fill", rgba(110, 170, 255, 22)), th().game("move_edge", rgba(110, 170, 255, 70)))
}

/// An exit marker (map exits and the battle exit tile): amber, a locked or shut one oxblood.
pub fn exit(open: bool) -> Rgba {
    if open {
        th().game("exit_open", rgb(70, 190, 100))
    } else {
        th().game("exit_shut", rgb(180, 70, 60))
    }
}

/// The one orange for chevrons and the active chip (ART-07): the theme's `focus`.
pub fn accent() -> Rgba {
    th().focus
}

/// PR7-36 (ART-05): the icon files to try for item `id` (its family's folder, then the type's and any base).
pub fn item_icons(d: &emberwake_world::Data, id: &str) -> Vec<String> {
    let item = d.entry("items", id);
    let family = d.entry("item_families", &aegis_core::json::s(item, "family"));
    emberwake_content::art::item_icon_chain(item, family)
}

/// PR7-36 (ART-05): the icon files to try for skill `id` (its family's, then its kind's).
pub fn skill_icons(d: &emberwake_world::Data, id: &str) -> Vec<String> {
    let sk = d.entry("skills", id);
    let fam = aegis_core::json::s_or(sk, "family", id);
    emberwake_content::art::skill_icon_chain(&fam, &aegis_core::json::s(sk, "kind"))
}
