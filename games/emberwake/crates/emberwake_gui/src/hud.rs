//! The HUD bar of the region map and the world map (PLAYUI1-H, spec PLAYUI1 3.4; engine-only, no Godot file):
//! the place name at the top left; gold, supplies and the icon buttons Party, Journal, Stash, Help and Menu at the
//! right. CAMPAIGN1-U (W4, P1): the two counters take the world's purse names, icons and tips, and a campaign's
//! visible meters follow them as chips (short name, value with its unit in the band tone's colour), all read from the
//! kit's view (`campaign::hud`).

use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::screens::{show_errors, Cx, Key, Nav};
use aegis_ui::ui::{tip, Glyph, Tip, Ui, FOE, GOLD, GOOD, BODY, TEXT, TITLE, W};
use emberwake_world::campaign::view::{ChipKind, HudChip};

/// The HUD bar's height (logical px).
pub const HUD_H: i32 = 20;
/// An icon button's hit box (it holds a 14 px icon).
const HUD_BOX: i32 = 18;

/// A HUD bar icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hud {
    Party,
    Journal,
    Stash,
    Help,
    Menu,
}

/// The buttons left to right: (button, widget id, name, key, icon in the asset layers, drawn stand-in, hover text). The
/// Menu button's id and key are the screen's own (`hud_bar`'s `menu`).
const HUD_BUTTONS: [(Hud, &str, &str, &str, &str, Glyph, &str); 5] = [
    (Hud::Party, "hud.party", "Party", "P", "ui/hud/party.png", Glyph::Letter('P'), "Roster, level up, skills and gear."),
    (Hud::Journal, "hud.journal", "Journal", "J", "ui/hud/journal.png", Glyph::Letter('J'), "Quests ready, active, on hold and completed."),
    (Hud::Stash, "hud.stash", "Stash", "I", "ui/hud/stash.png", Glyph::Letter('I'), "The party's shared stash, beside the selected hero's gear."),
    (Hud::Help, "hud.help", "Help", "?", "ui/help.png", Glyph::Help, ""),
    (Hud::Menu, "", "Game menu", "", "ui/hud/menu.png", Glyph::Menu, "Party, journal, save, load, settings."),
];

/// The HUD bar's keys: P party, J journal, I stash, ? help (Esc stays each screen's own).
pub fn hud_key(k: Key) -> Option<Hud> {
    match k {
        Key::Char(c) => match c.to_ascii_lowercase() {
            'p' => Some(Hud::Party),
            'j' => Some(Hud::Journal),
            'i' => Some(Hud::Stash),
            '?' => Some(Hud::Help),
            _ => None,
        },
        _ => None,
    }
}

/// What a HUD button opens: Party the Character popup on the first active member's Stats tab, Stash the same popup's
/// Gear tab (where the party's shared stash lives), Journal the journal (a notice while there are no quests), Help the
/// controls `help`, Menu the game menu.
pub fn hud_open(cx: &mut Cx, h: Hud, help: &[&str]) {
    let first = cx.gs().party.active_members().first().map(|m| m.id.clone()).unwrap_or_default();
    match h {
        Hud::Party => cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &first, "stats")))),
        Hud::Stash => cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &first, "gear")))),
        Hud::Journal if !crate::hub::has_quests(cx) => show_errors(cx, vec!["No quests yet.".into()]),
        Hud::Journal => cx.nav.push(Nav::Push(Box::new(crate::hub::Journal::default()))),
        Hud::Help => {
            let lines = std::iter::once("Controls".to_string()).chain(help.iter().map(|l| l.to_string())).collect();
            cx.nav.push(Nav::Push(Box::new(aegis_ui::screen::Notice::new(lines))));
        }
        Hud::Menu => cx.nav.push(Nav::Push(Box::new(crate::hub::Hub::default()))),
    }
}

/// Draws the HUD bar: `name` (gold) from `left`, then at the right gold, supplies and the icon buttons Party, Journal,
/// Stash, Help and Menu (icons from the asset layers, drawn shapes while a file is missing). `menu` is the Menu
/// button's widget id and key; `why` (when not empty) disables every button but Help and says why on its hover card;
/// `help` lists the controls; `wrap` makes the screen's action of a button.
#[allow(clippy::too_many_arguments)]
pub fn hud_bar<A: Clone>(ui: &mut Ui<A>, cx: &mut Cx, left: i32, name: &str, menu: (&str, &str), why: &str, help: &[&str], wrap: impl Fn(Hud) -> A) {
    ui.p.fill(Rect::new(0, 0, W, HUD_H), rgba(10, 14, 22, 255));
    let journal = crate::hub::has_quests(cx);
    let px = ui.icon_px(Rect::new(0, 0, HUD_BOX, HUD_BOX));
    let start = W - 3 - HUD_BUTTONS.len() as i32 * (HUD_BOX + 2) + 2;
    for (i, (h, id, label, key, rel, glyph, body)) in HUD_BUTTONS.into_iter().enumerate() {
        let r = Rect::new(start + i as i32 * (HUD_BOX + 2), (HUD_H - HUD_BOX) / 2, HUD_BOX, HUD_BOX);
        let (id, key) = if h == Hud::Menu { menu } else { (id, key) };
        let title = if key.is_empty() { label.to_string() } else { format!("{label} ({key})") };
        let mut t: Tip = tip(&title, body);
        if h == Hud::Help {
            t.extend(help.iter().map(|l| (l.to_string(), TEXT)));
        }
        let off = match h {
            Hud::Help => "",
            _ if !why.is_empty() => why,
            Hud::Journal if !journal => "No quests yet.",
            _ => "",
        };
        if !off.is_empty() {
            t.push((off.to_string(), FOE));
        }
        ui.icon_button(id, r, label, cx.art.image(rel, px, px), glyph, wrap(h), off.is_empty(), false, t);
    }
    // the chips right to left (the last meter first, the currency last): an icon, a meter's short name, the amount
    let icon = Ui::<A>::icon_rect(Rect::new(0, 0, HUD_BOX, HUD_BOX)).w;
    let ty = (HUD_H as f32 - ui.p.line_h(BODY)) / 2.0;
    let mut right = start - 6;
    let chips = chips(cx);
    for chip in chips.iter().rev() {
        let id = chip_id(chip);
        let short = if chip.short.is_empty() { String::new() } else { format!("{} ", chip.short) };
        let (sw, tw) = (ui.p.text_w(&short, BODY), ui.p.text_w(&chip.value, BODY));
        let width = icon + 3 + (sw + tw).ceil() as i32;
        let r = Rect::new(right - width, 0, width, HUD_H);
        let ir = Rect::new(r.x, (HUD_H - icon) / 2, icon, icon);
        match (chip.icon.is_empty(), cx.art.image(&chip.icon, px, px)) {
            (false, Some(img)) => ui.p.image(img, ir),
            _ => counter_glyph(ui, chip, ir),
        }
        let x = (ir.x + icon + 3) as f32;
        if !short.is_empty() {
            ui.p.text(x, ty, &short, BODY, TEXT);
        }
        ui.p.text(x + sw, ty, &chip.value, BODY, if chip.kind == ChipKind::Meter { tone_colour(&chip.tone) } else { TEXT });
        let mut t = tip(&chip.title, "");
        t.extend(chip.lines.iter().map(|l| (l.clone(), TEXT)));
        ui.push(id, "label", format!("{short}{}", chip.value), r, true, false, None, t);
        right = r.x - 8;
    }
    ui.p.text_fit(left as f32, (HUD_H as f32 - ui.p.line_h(TITLE)) / 2.0, name, TITLE, (right - left - 4) as f32, GOLD);
}

/// The HUD chips of the game in play (`campaign::hud`: the purse counters, then the visible meters); none without a
/// game.
pub fn chips(cx: &Cx) -> Vec<HudChip> {
    cx.game.as_ref().map(|g| emberwake_world::campaign::hud(&cx.data, &g.gs)).unwrap_or_default()
}

/// A chip's widget id: `hud.gold` and `hud.supplies` for the purse counters (in every world), `hud.meter.<id>` for a
/// meter.
pub fn chip_id(chip: &HudChip) -> String {
    match chip.kind {
        ChipKind::Currency => "hud.gold".into(),
        ChipKind::Upkeep => "hud.supplies".into(),
        ChipKind::Meter => format!("hud.meter.{}", chip.id),
    }
}

/// CAMPAIGN1 M2, W4: the colour of a band tone (`calm`, `warn`, `danger`, `fatal`; anything else is plain text).
pub fn tone_colour(tone: &str) -> Rgba {
    match tone {
        "calm" => GOOD,
        "warn" => GOLD,
        "danger" => rgb(244, 146, 60),
        "fatal" => FOE,
        _ => TEXT,
    }
}

/// The drawn stand-in of a chip's icon while its file is missing: a coin (the currency), a tied sack (the upkeep), or a
/// lamp in the band tone's colour (a meter).
fn counter_glyph<A>(ui: &mut Ui<A>, chip: &HudChip, r: Rect) {
    let (cx_, cy_, s) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, r.w as f32);
    match chip.kind {
        ChipKind::Currency => {
            ui.p.circle(cx_, cy_, s * 0.42, rgb(150, 110, 30));
            ui.p.circle(cx_, cy_, s * 0.32, GOLD);
        }
        ChipKind::Upkeep => {
            ui.p.panel(Rect::new(r.x + r.w / 6, r.y + r.h / 3, r.w - r.w / 3, r.h - r.h / 3), rgb(150, 112, 70), rgb(96, 70, 40), 3.0);
            ui.p.line(cx_ - s * 0.18, r.y as f32 + s * 0.2, cx_ + s * 0.18, r.y as f32 + s * 0.2, rgb(196, 160, 110), 0.0);
        }
        ChipKind::Meter => {
            ui.p.circle(cx_, cy_, s * 0.38, rgb(40, 40, 48));
            ui.p.circle(cx_, cy_, s * 0.28, tone_colour(&chip.tone));
        }
    }
}

/// PLAYUI1-H: the HUD bar of the region map and the world map, drawn headless over a debug game of the first playable
/// world (nothing saved; the art layers are off, so every icon is its drawn stand-in). The region map's tests
/// (PLAYUI1-M) use the same helpers.
#[cfg(test)]
pub(crate) mod hud_tests {
    use super::*;
    use crate::region::{RegionMap, REGION_HELP};
    use crate::screens::{default_settings, Game, GameCx, Screen};
    use aegis_core::json::{self, s, s_or};
    use aegis_ui::ui::{Gfx, H};
    use emberwake_world::explore;
    use emberwake_world::state::SUPPLIES_MAX;
    use serde_json::{json, Value};
    use aegis_ui::painter::{Fonts, TextRec, View};
    use aegis_ui::screen::{Settings, TEXT_SIZES};
    use emberwake_world::Data;
    use std::path::Path;
    use std::sync::Arc;

    /// A debug game of the first playable world; its settings file is its own (tests run side by side and the region
    /// map saves the legend's state).
    pub(crate) fn game_cx() -> Cx {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tmp = std::env::temp_dir().join(format!("emberwake_hud_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let data = Arc::new(Data::open(&dir).expect("data").0);
        let world = emberwake_world::new_game::playable_worlds(&data).first().cloned().expect("a playable world");
        let gs = emberwake_world::new_game::create_for_world(&data, &world, 1).expect("a new game");
        let app = GameCx {
            data,
            manifest: Default::default(),
            game: Some(Game::new(gs, -1, true)),
            save_dir: tmp.join("saves"),
            tuning: aegis_ui::tuning::TuningFile::new(None),
            log: Vec::new(),
            encounter: String::new(),
            seed: None,
            debug: true,
            data_dir: dir,
        };
        Cx {
            settings: Settings::load(tmp.join("settings.json"), default_settings()),
            art: aegis_ui::art::Art::new(None),
            audio: aegis_ui::audio::Audio::offline(None),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: tmp,
            quit: false,
            answers: Vec::new(),
            playtest: aegis_ui::playtest::Playtests::off(),
            held: Default::default(),
            app,
        }
    }

    pub(crate) fn gfx() -> Gfx {
        let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).expect("font");
        Gfx { canvas: aegis_ui::canvas::Canvas::new(W, H), view: View::fit(W as u32, H as u32), fonts: Fonts::new(&font), kit: Default::default() }
    }

    /// One frame of `screen` at text size `scale` (the mouse off the canvas); the texts drawn.
    pub(crate) fn frame(screen: &mut dyn Screen<GameCx>, cx: &mut Cx, g: &mut Gfx, scale: f32) -> Vec<TextRec> {
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        screen.draw(cx, g, (-500, -500), true);
        g.fonts.audit.take().unwrap_or_default()
    }

    pub(crate) fn rect(w: &Value) -> (i64, i64, i64, i64) {
        let n = |i: usize| w["rect"][i].as_i64().unwrap_or(0);
        (n(0), n(1), n(2), n(3))
    }

    /// The bar's widgets, left to right: (id, kind, enabled).
    fn bar(screen: &dyn Screen<GameCx>) -> Vec<(String, String, bool)> {
        let mut w: Vec<Value> = screen.widgets().into_iter().filter(|w| rect(w).1 + rect(w).3 <= HUD_H as i64).collect();
        w.sort_by_key(|w| rect(w).0);
        w.iter().map(|w| (s(json::dict(w), "id"), s(json::dict(w), "kind"), json::b(json::dict(w), "enabled", false))).collect()
    }

    fn ids(b: &[(String, String, bool)]) -> Vec<&str> {
        b.iter().map(|w| w.0.as_str()).collect()
    }

    /// The screen the last navigation opened: (id, state); the navigation is cleared.
    fn opened(cx: &mut Cx) -> (String, Value) {
        let out = match cx.nav.last() {
            Some(Nav::Push(s)) => (s.id().to_string(), s.state(cx)),
            _ => (String::new(), Value::Null),
        };
        cx.nav.clear();
        out
    }

    #[test]
    fn the_region_map_bar_is_the_name_then_gold_supplies_and_five_icon_buttons() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        let b = bar(&map);
        assert_eq!(ids(&b), ["hud.gold", "hud.supplies", "hud.party", "hud.journal", "hud.stash", "hud.help", "region.menu"], "{b:?}");
        assert!(b[2..].iter().all(|w| w.1 == "icon_button"), "{b:?}");
        let gs = cx.gs().clone();
        let name = s_or(explore::location(&cx.data, &gs), "name", "").to_string();
        let top: Vec<&TextRec> = recs.iter().filter(|r| r.y + r.h / 2.0 < HUD_H as f32).collect();
        let at = |t: &str| top.iter().find(|r| r.text == t).map(|r| r.x);
        assert!(!name.is_empty() && at(&name).is_some_and(|x| x < 10.0), "the location name at the top left: {top:?}");
        let (gold, supplies) = (at(&gs.party.gold.to_string()), at(&format!("{}/{SUPPLIES_MAX}", gs.party.supplies)));
        assert!(gold.zip(supplies).is_some_and(|(g, s)| g < s && g > W as f32 / 2.0), "gold, then supplies at the right: {top:?}");
        assert!(!recs.iter().any(|r| r.text.contains("WASD") || r.text.starts_with("Supplies")), "the help line and the supplies text left the top left");
        let widgets = map.widgets();
        let card = |id: &str| widgets.iter().find(|w| w["id"] == id).map(|w| w["tip"].to_string()).unwrap_or_default();
        for (id, key) in [("hud.party", "(P)"), ("hud.journal", "(J)"), ("hud.stash", "(I)"), ("hud.help", "(?)"), ("region.menu", "(Esc)")] {
            assert!(card(id).contains(key), "{id}'s card names its key: {}", card(id));
        }
        assert!(card("hud.help").contains("WASD"), "Help's card lists the controls");
    }

    #[test]
    fn the_bar_buttons_open_party_journal_stash_help_and_menu() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let first = cx.gs().party.active_members().first().map(|m| m.id.clone()).expect("an active member");
        map.click_id(&mut cx, "hud.party").expect("party");
        let (id, state) = opened(&mut cx);
        assert_eq!((id.as_str(), state["member"].clone(), state["tab"].clone()), ("character", json!(first), json!("stats")));
        map.click_id(&mut cx, "hud.stash").expect("stash");
        let (id, state) = opened(&mut cx);
        assert_eq!((id.as_str(), state["member"].clone(), state["tab"].clone()), ("character", json!(first), json!("gear")), "the shared stash is on the Gear tab");
        map.click_id(&mut cx, "hud.help").expect("help");
        let (id, state) = opened(&mut cx);
        assert_eq!(id, "notice");
        assert_eq!(state["lines"].as_array().map(|l| l.len()), Some(REGION_HELP.len() + 1), "a title and the controls");
        map.click_id(&mut cx, "region.menu").expect("menu");
        assert_eq!(opened(&mut cx).0, "hub");
        // Journal: on when there are quests (the Hub's rule), else off with the reason on its card
        let journal = crate::hub::has_quests(&cx);
        let b = bar(&map);
        assert_eq!(b.iter().find(|w| w.0 == "hud.journal").map(|w| w.2), Some(journal), "{b:?}");
        if journal {
            map.click_id(&mut cx, "hud.journal").expect("journal");
            assert_eq!(opened(&mut cx).0, "journal");
        }
        for (k, want) in [('i', "character"), ('P', "character"), ('?', "notice"), ('j', if journal { "journal" } else { "notice" })] {
            map.key(&mut cx, Key::Char(k));
            assert_eq!(opened(&mut cx).0, want, "key {k}");
        }
    }

    #[test]
    fn a_popup_turns_the_bar_buttons_off_but_help_and_the_counters_stay() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        map.lore = Some(("A page".into(), "Its text.".into()));
        frame(&mut map, &mut cx, &mut g, 1.0);
        let on: Vec<(String, bool)> = bar(&map).into_iter().map(|w| (w.0, w.2)).collect();
        let want = [("hud.gold", true), ("hud.supplies", true), ("hud.party", false), ("hud.journal", false), ("hud.stash", false), ("hud.help", true), ("region.menu", false)];
        assert_eq!(on, want.map(|(id, e)| (id.to_string(), e)));
        map.key(&mut cx, Key::Char('i'));
        assert!(cx.nav.is_empty(), "the keys wait too");
    }

    #[test]
    fn the_world_map_gets_the_same_bar() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let b = bar(&wm);
        assert_eq!(ids(&b), ["world_map.back", "hud.gold", "hud.supplies", "hud.party", "hud.journal", "hud.stash", "hud.help", "world_map.menu"], "{b:?}");
        wm.click_id(&mut cx, "hud.stash").expect("stash");
        assert_eq!(opened(&mut cx).1["tab"], json!("gear"));
        wm.key(&mut cx, Key::Char('p'));
        assert_eq!(opened(&mut cx).1["tab"], json!("stats"));
        wm.click_id(&mut cx, "world_map.menu").expect("menu");
        assert_eq!(opened(&mut cx).0, "hub");
    }

    /// TEXTFIT1: the bar's texts fit (nothing cut, past the canvas, overlapping or out of its control) and stay in the
    /// bar at every text size, on both maps.
    #[test]
    fn the_bar_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let mut wm = crate::worldmap::WorldMap::new();
        let screens: [&mut dyn Screen<GameCx>; 2] = [&mut map, &mut wm];
        for screen in screens {
            for (size, scale) in TEXT_SIZES {
                let recs: Vec<TextRec> = frame(screen, &mut cx, &mut g, scale as f32).into_iter().filter(|r| r.y + r.h / 2.0 < HUD_H as f32).collect();
                let boxes: Vec<(String, f32, f32, f32, f32)> = screen
                    .widgets()
                    .iter()
                    .filter(|w| rect(w).1 < HUD_H as i64)
                    .map(|w| {
                        let (x, y, ww, hh) = rect(w);
                        (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                    })
                    .collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{} at {size}: {v}", screen.id());
                for r in &recs {
                    assert!(r.y >= 0.0 && r.y + r.h <= HUD_H as f32 + 0.5, "{} at {size}: '{}' leaves the bar ({} to {})", screen.id(), r.text, r.y, r.y + r.h);
                }
            }
        }
    }
}
