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

/// STYLE-PASS1B (`PR7-48`, FB-366): the ring on a click-to-walk's destination tile: the one amber (`accent`).
pub fn walk_ring() -> Rgba {
    accent()
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

/// BATTLE-UI2 (`SCREEN_CONTRACT_BATTLE.md` 1, frame A): the plain frame of popups, cards and the HUD: a 1 px bronze
/// outer line, a dark gap, a 1 px iron inner line and an oxblood top line, no corners, over `fill`; the contract's colours from the theme (`frame_a_outer`, `frame_a_inner`, `frame_a_top`).
pub fn frame_a(p: &mut aegis_ui::painter::Painter, r: aegis_ui::canvas::Rect, fill: Rgba) {
    let clear = rgba(0, 0, 0, 0);
    p.panel(r, fill, th().game("frame_a_outer", rgb(0x74, 0x54, 0x3c)), 0.0);
    if r.w > 2 && r.h > 2 {
        p.panel(r.inset(1), clear, th().game("frame_a_inner", rgb(0x4b, 0x55, 0x63)), 0.0);
    }
    p.line(r.x as f32 + 1.0, r.y as f32 + 1.0, (r.x + r.w - 1) as f32, r.y as f32 + 1.0, th().game("frame_a_top", rgb(0x99, 0x1b, 0x1b)), 0.0);
}

/// BATTLE-UI2 (contract 3, 5): the boss gold (a boss plate's name and rim, the crown) and the planned-AP amber.
pub fn boss_gold() -> Rgba {
    th().game("boss_gold", rgb(0xfb, 0xbf, 0x24))
}

/// BATTLE-UI2 (contract 2, 3; `PLANNED`): the amber of planned AP, on the strip's AP line and the plates' AP bars.
pub fn planned() -> Rgba {
    th().game("planned", rgb(0xd9, 0x77, 0x06))
}

/// BATTLE-UI2 (contract 3): the guests' teal (their divider, rim and "AI" tag).
pub fn guest() -> Rgba {
    th().game("guest", rgb(0x45, 0x85, 0x7c))
}

/// BATTLE-UI2 (contract 2, 4, 5): the oxblood of KO tags, impact blocks on a ruler and debuff rims.
pub fn oxblood() -> Rgba {
    th().game("oxblood", rgb(0x99, 0x1b, 0x1b))
}

/// BATTLE-UI2 (contract 6, PR7-40 part 2): the moss of a heal (its heart chip and the HP bar's gain): the heroes' HP
/// moss.
pub fn moss() -> Rgba {
    hp()
}

/// STATUSVIS1 SV15 (PR7-57): a locked block's grey (the strip's tile and the full timeline view's tint, band and rim:
/// the palette's iron) and its bar's lighter grey under the icon.
pub fn lock_block() -> Rgba {
    th().game("lock_block", rgb(0x4b, 0x55, 0x63))
}

/// SV15: the bar under a locked block's icon.
pub fn lock_bar() -> Rgba {
    th().game("lock_bar", rgb(0x9c, 0xa3, 0xaf))
}

/// SV8, SV15 (PR7-57): the crimson of the taunt badge's rim and of an enemy target's frame in the full timeline view.
pub fn crimson() -> Rgba {
    th().game("crimson", rgb(0xe1, 0x1d, 0x48))
}

/// The pale moss of a previewed heal on an HP bar.
pub fn moss_gain() -> Rgba {
    let [r, g, b, _] = moss();
    rgba(r, g, b, 140)
}
