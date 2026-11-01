//! Portraits (PORTRAITFX1-G, docs/specs/PORTRAITFX1.md 6.1 and 6.4; engine-only, Godot had no portrait effects): the
//! game's binding of the engine's portrait effects (`aegis_ui::portrait_fx`) to its units and the kit's cue rows
//! (`emberwake_battle::portrait`, the catalog kind `portrait_cues`).
//!
//! - `Who`: whose portrait it is (the map key, the portrait tier, the id and body of `face_chain`, the name for the
//!   initial-letter fallback); built from a world character, a party member, a battle unit or a portrait file.
//! - `look_of_unit`, `look_of_member`, `look_of_character`: the kit `Look` the state rows read.
//! - `Faces`: one per screen: a `PortraitFx` per key, the screen's `Budget` and its clock; `draw` holds the sidecar's
//!   `idle` overlays plus the kit's `holds` for the look, resolves the chain with the engine's expression and draws (the
//!   initial letter when no file loads); `react` fires the kit's `reactions` for step events (none under `skip_anim`).
//! - Colours: the rim key Tallow Amber and the art-bible hues the cue rows and sidecars use (`HUES`).
//!
//! Presentation only: no rule, plan, AI, save, trace or sim reads a portrait.

use crate::screens::Cx;
use aegis_core::json::{self, Dict};
use aegis_ui::art::Art;
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::fx::Clock;
use aegis_ui::painter::Painter;
use aegis_ui::portrait_fx::{idle_cues, Budget, Cue, PortraitFx, PortraitOpts};
use aegis_ui::screen::Settings;
use aegis_ui::ui::{GOLD, SMALL};
use emberwake_battle::portrait::{self, Look};
use emberwake_battle::session::Session;
use emberwake_content::art as kart;
use emberwake_data::{Catalog, PortraitCue};
use emberwake_rules::derived;
use emberwake_rules::unit::Unit;
use emberwake_world::state::UnitState;
use emberwake_world::{progression, Data, GameState};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The house rim light of every portrait (6.4): Tallow Amber `#d97706`, the rim key when a sidecar names none.
pub const RIM: Rgba = rgb(0xd9, 0x77, 0x06);
/// The art-bible hues of the cue rows and sidecars (6.4): (name, what it marks, colour).
pub const HUES: [(&str, &str, Rgba); 8] = [
    ("Tallow Amber", "the rim light", RIM),
    ("Blight Violet", "marrow glow, veins", rgb(0x93, 0x33, 0xea)),
    ("Resonant Orchid", "infused eyes", rgb(0xc0, 0x84, 0xfc)),
    ("Arterial Red", "crit, beast eyes", rgb(0xe1, 0x1d, 0x48)),
    ("Candle Gold", "guard, heal", rgb(0xfb, 0xbf, 0x24)),
    ("Salt Cyan", "cold", rgb(0x06, 0xb6, 0xd4)),
    ("Cinder Ash", "ash, smoke", rgb(0x6b, 0x72, 0x80)),
    ("Bleached Linen", "the hit flash", rgb(0xd9, 0xcb, 0xb4)),
];

/// The art-bible name of colour `#rrggbb` ("" when it is none of `HUES`).
pub fn hue_name(hex: &str) -> &'static str {
    let Some(c) = aegis_core::assets::hex_colour(hex) else { return "" };
    HUES.iter().find(|h| h.2[..3] == c[..]).map_or("", |h| h.0)
}

/// A cue row as the engine's cue (keyed `cue.<id>`); None when the engine does not know its effect or jolt preset.
pub fn cue_of(row: &PortraitCue) -> Option<Cue> {
    let color = aegis_core::assets::hex_colour(&row.color).map(|c| [c[0], c[1], c[2], 255]);
    Some(Cue::parse(&row.effect, &row.preset)?.with_intensity(row.intensity as f32).with_ms(row.ms.max(0) as u32).with_color(color).with_expression(&row.expression).with_priority(row.priority).with_key(&format!("cue.{}", row.id)))
}

/// The portrait tier of a unique folder (`heroes` -> `hero`, ...); `common` for any other.
fn tier_of_folder(folder: &str) -> &'static str {
    match folder {
        "heroes" => "hero",
        "npcs" => "npc",
        "elites" => "elite",
        "bosses" => "boss",
        _ => "common",
    }
}

/// Whose portrait is drawn: the `Faces` key (the character id, else the unit id, else the path), the portrait tier,
/// id and body `face_chain` reads, the name (its initial is the fallback) and, for a portrait file picked as it is
/// (the Gallery), that file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Who {
    pub key: String,
    pub tier: String,
    pub id: String,
    pub body: String,
    pub name: String,
    pub file: String,
}

impl Who {
    /// The portrait file `rel` (`portraits/<folder>/<id>.png`) as it is: keyed by its path; a unique folder's file
    /// still takes its expression frames (`faces/<id>/`).
    pub fn file(rel: &str) -> Who {
        let parts: Vec<&str> = rel.split('/').collect();
        let (folder, id) = match parts.as_slice() {
            ["portraits", folder, file] => (folder.to_string(), file.trim_end_matches(".png").to_string()),
            _ => (String::new(), rel.to_string()),
        };
        Who { key: rel.to_string(), tier: tier_of_folder(&folder).to_string(), name: id.clone(), id, body: String::new(), file: rel.to_string() }
    }

    /// The files to try for `expression` ("" = neutral): `face_chain`; a picked file stops at that file (its frames,
    /// then itself; a common file only itself).
    pub fn chain(&self, expression: &str) -> Vec<String> {
        let chain = kart::face_chain(&self.tier, &self.id, &self.body, expression);
        if self.file.is_empty() {
            return chain;
        }
        match chain.iter().position(|r| *r == self.file) {
            Some(at) => chain[..=at].to_vec(),
            None => vec![self.file.clone()],
        }
    }

    /// The fallback letter (the name's first character, `?` without one).
    pub fn initial(&self) -> String {
        self.name.chars().next().unwrap_or('?').to_string()
    }
}

// The builders the surfaces call (PORTRAITFX1-D: the talk box, `of_character`; -R: the Character screen and the hub
// list, `of_member`; -P: the battle panels, `of_unit`); until each lands only the tests use them.
#[cfg_attr(not(test), allow(dead_code))]
impl Who {
    /// A world character (`screens::character_art`): keyed by its id; its tier, body and name.
    pub fn of_character(d: &Data, character: &Dict) -> Who {
        let unit = d.entry("units", &json::s(character, "base_unit"));
        let class = d.entry("classes", &json::s(unit, "class_id"));
        let id = json::s(character, "id");
        Who { key: id.clone(), tier: kart::portrait_tier(character, unit), body: kart::body_of(character, unit, class), name: json::s_or(character, "name", &id), id, file: String::new() }
    }

    /// A party member: the world character it plays (its `lore_ref`), else its base unit keyed by the member id.
    pub fn of_member(d: &Data, m: &UnitState) -> Who {
        let character = progression::character_of(d, m);
        if !character.is_empty() {
            return Who::of_character(d, character);
        }
        let unit = d.entry("units", &m.base_unit);
        let class = d.entry("classes", &json::s(unit, "class_id"));
        let empty = Dict::new();
        Who { key: m.id.clone(), tier: kart::portrait_tier(&empty, unit), id: m.base_unit.clone(), body: kart::body_of(&empty, unit, class), name: progression::display_name(d, m), file: String::new() }
    }

    /// Battle unit `i` of `s`, as world character `character` (empty: none): keyed by the character id, else
    /// `<unit id>#<i>` (two units of one catalog row react apart).
    pub fn of_unit(d: &Data, s: &Session, i: usize, character: &Dict) -> Who {
        if !character.is_empty() {
            return Who::of_character(d, character);
        }
        let u = &s.w.units[i];
        let unit = d.entry("units", &u.catalog_id);
        let class = d.entry("classes", &json::s(unit, "class_id"));
        let empty = Dict::new();
        Who { key: format!("{}#{i}", u.catalog_id), tier: kart::portrait_tier(&empty, unit), id: u.catalog_id.clone(), body: kart::body_of(&empty, unit, class), name: u.name.clone(), file: String::new() }
    }
}

/// The derived stats the cue rows read (each `derived` row's `stat`, once), for `u`: (id, the value with its status
/// modifiers, the value without them); a stat the ruleset leaves out is absent.
fn derived_of(cat: &Catalog, ruleset: &str, u: &Unit) -> Vec<(String, i32, i32)> {
    let mut stats: Vec<&str> = Vec::new();
    for r in cat.portrait_cues.iter().filter(|r| r.trigger == "derived" && !r.stat.is_empty()) {
        if !stats.contains(&r.stat.as_str()) {
            stats.push(&r.stat);
        }
    }
    stats
        .into_iter()
        .filter_map(|id| Some((id.to_string(), derived::value(cat, ruleset, u, id)?, derived::value_without(cat, ruleset, u, id, derived::STATUS_SOURCE)?)))
        .collect()
}

/// Battle unit `i`'s look (5.2): its HP, down (HP 0 and still on the field), its statuses, its member's `conditions`
/// (the caller's: ROSTER1), the derived stats the cue rows read and the battle's ruleset.
// The look builders are for the surfaces (PORTRAITFX1-D talk box: member and character; -R Character screen and hub
// list, -P battle panels: unit); until -P lands only the tests call this one, hence the dead-code allowance.
#[cfg_attr(not(test), allow(dead_code))]
pub fn look_of_unit(s: &Session, i: usize, conditions: &[String]) -> Look {
    let u = &s.w.units[i];
    let ruleset = s.w.ruleset.id.clone();
    Look { hp: u.hp, max_hp: u.max_hp, down: u.hp <= 0 && !u.is_withdrawn, statuses: u.st.active.iter().cloned().collect(), conditions: conditions.to_vec(), derived: derived_of(&s.w.cat, &ruleset, u), ruleset }
}

/// A party member's look out of battle: the HP the save carries (`hp_lost` against its max in the world it plays), no
/// statuses, never down; its conditions (ROSTER1 W1: `wounded`).
pub fn look_of_member(d: &Data, gs: &GameState, m: &UnitState) -> Look {
    let world = &gs.world_id;
    let ruleset = d.ruleset_id(world);
    let built = progression::build_played(d, m, world);
    let max_hp = built.as_ref().map_or(0, |b| b.unit.max_hp);
    let hp = if max_hp > 0 { progression::start_hp(max_hp as i64, m.hp_lost) as i32 } else { 0 };
    let derived = built.map(|b| derived_of(&d.cat, &ruleset, &b.unit)).unwrap_or_default();
    Look { hp, max_hp, down: false, statuses: Vec::new(), conditions: m.conditions.clone(), derived, ruleset }
}

/// A world character's look: no HP (so no `hp` or `down` row matches), no statuses; the derived stats of its base unit
/// as built (no status modifiers out of battle) under the world's ruleset.
pub fn look_of_character(d: &Data, gs: &GameState, character: &Dict) -> Look {
    let ruleset = d.ruleset_id(&gs.world_id);
    let def = d.cat.units.get(&json::s(character, "base_unit"));
    // AP 0: the unit is built only to read its derived stats (no formula reads AP; no battle starts)
    let derived = def.map(|def| derived_of(&d.cat, &ruleset, &emberwake_rules::build_unit(&d.cat, def, 0))).unwrap_or_default();
    Look { derived, ruleset, ..Look::default() }
}

/// The face sidecar of the file a chain found (`face_sidecars`: its own, then a frame's base portrait's).
pub fn sidecar(art: &mut Art, found: &str) -> Option<aegis_core::assets::Face> {
    kart::face_sidecars(found).iter().find_map(|f| f.strip_suffix(".face.json").and_then(|stem| art.face(&format!("{stem}.png")).cloned()))
}

/// One screen's portraits (6.1): a `PortraitFx` per key, the screen's budget and clock. `begin` at the start of the
/// screen's draw (a portrait neither drawn nor reacted to since the last `begin` is dropped), then `draw` each.
#[derive(Debug, Clone, Default)]
pub struct Faces {
    fx: BTreeMap<String, PortraitFx>,
    budget: Budget,
    /// The portraits' clock: the screen advances it (`tick` by wall time; the battle copies FX1's clock in).
    pub clock: Clock,
    last_wall: Option<std::time::Instant>,
    /// Keys drawn or reacted to since the last `begin`.
    touched: BTreeSet<String>,
    /// Cues a review page holds on a portrait beside its look (the Gallery's toggles); the surfaces pin none.
    pins: BTreeMap<String, Vec<Cue>>,
    /// The Gallery's anchors check and its own reduce-motion check (None: the setting).
    pub show_anchors: bool,
    pub reduce: Option<bool>,
}

impl Faces {
    /// Starts a frame: the budget is fresh; portraits left out since the last frame are dropped.
    pub fn begin(&mut self) {
        let touched = std::mem::take(&mut self.touched);
        self.fx.retain(|k, _| touched.contains(k));
        self.pins.retain(|k, _| touched.contains(k));
        self.budget.begin();
    }

    /// The clock moves by the wall time since the last call (at most `vignette::MAX_FRAME_MS`).
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        let ms = self.last_wall.map_or(0.0, |l| (now.duration_since(l).as_secs_f64() * 1000.0).min(aegis_ui::vignette::MAX_FRAME_MS as f64));
        self.last_wall = Some(now);
        self.clock.advance(ms, 1.0);
    }

    /// The portraits' clock in ms.
    pub fn now(&self) -> u64 {
        self.clock.now()
    }

    /// How portraits draw on this screen: the reduce-motion setting (or the page's own check), the rim key, anchors.
    pub fn opts(&self, settings: &Settings) -> PortraitOpts {
        PortraitOpts { reduce_motion: self.reduce.unwrap_or_else(|| settings.reduce_motion()), rim_default: Some(RIM), show_anchors: self.show_anchors }
    }

    /// Holds `cues` on `key` beside its look until the next `pin` (the Gallery's toggles).
    pub fn pin(&mut self, key: &str, cues: Vec<Cue>) {
        self.pins.insert(key.to_string(), cues);
    }

    /// Draws `who` into `r` on `surface` with the cue rows of the data, the screen's options and its clock (6.1); false
    /// when no file of its chain loads (the initial letter is drawn instead).
    pub fn draw(&mut self, p: &mut Painter, cx: &mut Cx, who: &Who, look: &Look, surface: &str, r: Rect) -> bool {
        let data = cx.data.clone();
        let opts = self.opts(&cx.settings);
        self.draw_parts(p, &mut cx.art, &data.cat.portrait_cues, who, look, surface, r, &opts)
    }

    /// `draw` over explicit parts: the holds are the sidecar's `idle`, the state rows of `rows` that hold for `look`
    /// on `surface` and the key's pins; the file is the first of `who`'s chain for the expression that wins.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_parts(&mut self, p: &mut Painter, art: &mut Art, rows: &[PortraitCue], who: &Who, look: &Look, surface: &str, r: Rect, opts: &PortraitOpts) -> bool {
        let now = self.now();
        self.touched.insert(who.key.clone());
        let mut held: Vec<Cue> = portrait::holds(rows, look, surface).into_iter().filter_map(cue_of).collect();
        held.extend(self.pins.get(&who.key).cloned().unwrap_or_default());
        let fx = self.fx.entry(who.key.clone()).or_insert_with(|| PortraitFx::new(&who.key));
        // The expression picks the file, the file's sidecar adds idle holds that may change the expression: settle it
        // in at most two passes (holding the same set twice changes nothing).
        let mut expression = fx.expression(now).to_string();
        let mut found = None;
        let mut face = None;
        for _ in 0..2 {
            found = art.first_rel(&who.chain(&expression));
            face = found.as_deref().and_then(|f| sidecar(art, f));
            let mut cues = face.as_ref().map(idle_cues).unwrap_or_default();
            cues.extend(held.iter().cloned());
            fx.hold(&cues, now);
            if fx.expression(now) == expression {
                break;
            }
            expression = fx.expression(now).to_string();
        }
        let drawn = found.is_some_and(|rel| fx.draw(p, art, &rel, face.as_ref(), r, now, opts, &mut self.budget));
        if !drawn {
            let size = (r.h as f32 * 0.32).clamp(SMALL, 20.0);
            p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - p.line_h(size)) / 2.0, &who.initial(), size, GOLD);
        }
        drawn
    }

    /// Fires the event rows of the data for `events` on `key` (`reactions` on `surface` under `ruleset`); none under
    /// the `skip_anim` setting. How many cues fired.
    pub fn react(&mut self, cx: &Cx, key: &str, events: &[&str], surface: &str, ruleset: &str) -> usize {
        if cx.settings.v["skip_anim"].as_bool().unwrap_or(false) {
            return 0;
        }
        let rows = portrait::reactions(&cx.data.cat.portrait_cues, events, surface, ruleset);
        rows.into_iter().filter_map(cue_of).map(|c| self.fire(key, c)).count()
    }

    /// Fires `cue` on `key` now (a portrait not drawn yet keeps it for its first draw).
    pub fn fire(&mut self, key: &str, cue: Cue) {
        let now = self.now();
        self.touched.insert(key.to_string());
        self.fx.entry(key.to_string()).or_insert_with(|| PortraitFx::new(key)).fire(cue, now);
    }

    /// The portrait of `key` (None when it is not kept).
    pub fn get(&self, key: &str) -> Option<&PortraitFx> {
        self.fx.get(key)
    }

    /// The agent link's view (4.11): each kept portrait's `PortraitFx::state` at the clock, by key.
    pub fn state(&self) -> Value {
        let now = self.now();
        Value::Array(self.fx.values().map(|f| f.state(now)).collect())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::hud::hud_tests::{game_cx, gfx};
    use aegis_core::assets::Layers;
    use aegis_ui::ui::Ui;
    use emberwake_host::{Battle, Host};
    use serde_json::json;
    use std::path::PathBuf;

    /// A cue row from its non-default keys (the loader's typed row).
    pub(crate) fn row(v: Value) -> PortraitCue {
        PortraitCue::from_obj(v.as_object().expect("an object"))
    }

    /// A scratch asset layer: `rel` is a `side` px PNG of `fill` inside a 1/30 border of dark grey, with `sidecar`
    /// beside it when given.
    pub(crate) fn layer(name: &str, rel: &str, side: u32, fill: [u8; 3], sidecar: Option<Value>) -> PathBuf {
        let root = std::env::temp_dir().join(format!("emberwake_portraits_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let border = (side as f32 / 30.0).round() as u32;
        let mut px = Vec::new();
        for y in 0..side {
            for x in 0..side {
                let edge = x < border || y < border || x >= side - border || y >= side - border;
                let c = if edge { [40, 40, 40] } else { fill };
                px.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&path).unwrap()), side, side);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        if let Some(s) = sidecar {
            std::fs::write(root.join(aegis_core::assets::face_file(rel)), s.to_string()).unwrap();
        }
        root
    }

    /// PORTRAITFX1-R: `cx` with cue rows `rows` and its art a scratch layer holding each party member's portrait (the
    /// first file of its chain), 96 px of `fill`, no sidecar.
    pub(crate) fn roster_cx(mut cx: Cx, name: &str, rows: Vec<PortraitCue>, fill: [u8; 3]) -> Cx {
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows);
        let d = cx.data.clone();
        let mut rels: Vec<String> = cx.gs().party.members.iter().filter_map(|m| Who::of_member(&d, m).chain("").first().cloned()).collect();
        rels.sort();
        rels.dedup();
        let roots = rels.iter().enumerate().map(|(i, rel)| layer(&format!("{name}_{i}"), rel, 96, fill, None)).collect();
        cx.art = Art::layered(Layers::of(roots));
        cx
    }

    fn arena(cx: &Cx) -> Host {
        let mut host = Host::new(Battle::new(cx.data.cat.clone()));
        assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": 3}))["ok"], json!(true));
        host
    }

    /// G5: the kit's copy of the effect names (and jolt presets) is the engine's.
    #[test]
    fn the_kit_effect_names_are_the_engines() {
        assert_eq!(emberwake_data::PORTRAIT_EFFECTS, aegis_ui::portrait_fx::EFFECTS);
        assert_eq!(emberwake_data::PORTRAIT_JOLTS, aegis_ui::portrait_fx::PORTRAIT_JOLTS);
    }

    /// G1: a battle unit's look carries its statuses and its member's conditions; a member's look the HP its save
    /// carries and no statuses; a world character's look has no HP, so no `hp` or `down` row matches it.
    #[test]
    fn the_looks_carry_what_their_source_has() {
        let cx = game_cx();
        let mut host = arena(&cx);
        let s = host.session.as_mut().expect("a battle");
        s.w.units[0].st.active.insert("stealth".into());
        let look = look_of_unit(s, 0, &["wounded".to_string()]);
        assert!(look.statuses.contains(&"stealth".to_string()), "{look:?}");
        assert_eq!(look.conditions, ["wounded"]);
        assert_eq!((look.hp, look.max_hp, look.down), (s.w.units[0].hp, s.w.units[0].max_hp, false));
        assert_eq!(look.ruleset, s.w.ruleset.id);
        s.w.units[0].hp = 0;
        assert!(look_of_unit(s, 0, &[]).down, "HP 0 on the field is down");
        let keys: Vec<String> = (0..s.w.units.len()).map(|i| Who::of_unit(&cx.data, s, i, &Dict::new()).key).collect();
        assert_eq!(keys[0], format!("{}#0", s.w.units[0].catalog_id));
        assert!(keys.iter().enumerate().all(|(i, k)| !keys[..i].contains(k)), "every unit reacts apart: {keys:?}");

        let d = cx.data.clone();
        let mut gs = cx.gs().clone();
        let m = gs.party.members[0].clone();
        let (_, max) = progression::current_hp(&d, &m, &gs.world_id);
        gs.party.members[0].hp_lost = (max / 2).max(1);
        let m = gs.party.members[0].clone();
        let look = look_of_member(&d, &gs, &m);
        assert_eq!((look.hp as i64, look.max_hp as i64), progression::current_hp(&d, &m, &gs.world_id));
        assert!(look.hp < look.max_hp && look.statuses.is_empty() && !look.down, "{look:?}");

        let character = progression::character_of(&d, &m).clone();
        assert_eq!(Who::of_member(&d, &m), Who::of_character(&d, &character), "a member draws as the character it plays");
        let look = look_of_character(&d, &gs, &character);
        assert_eq!((look.hp, look.max_hp, look.down), (0, 0, false));
        let rows = [row(json!({"id": "any_hp", "trigger": "hp", "below": 100, "effect": "desaturate"})), row(json!({"id": "down", "trigger": "down", "effect": "still"}))];
        assert!(portrait::holds(&rows, &look, "dialogue").is_empty(), "no hp or down row matches a world character");
        assert_eq!(portrait::holds(&rows, &look_of_member(&d, &gs, &m), "dialogue").len(), 1, "the member's carried HP matches below 100");
    }

    /// G2: a portrait whose sidecar holds `eye_glow` draws it with no cue rows at all.
    #[test]
    fn a_sidecar_idle_overlay_draws_without_cue_rows() {
        let face = json!({"size": 96, "eyes": [[30, 40, 10, 6], [56, 40, 10, 6]], "idle": [{"effect": "eye_glow", "color": "#c084fc", "intensity": 0.8}]});
        let root = layer("idle", "portraits/npcs/ida.png", 96, [90, 70, 60], Some(face));
        let mut art = Art::layered(Layers::of(vec![root]));
        let mut g = gfx();
        let mut ui: Ui<()> = Ui::new(&mut g, (-500, -500), true);
        let mut faces = Faces::default();
        let who = Who { key: "ida".into(), tier: "npc".into(), id: "ida".into(), name: "Ida".into(), ..Who::default() };
        // the idle hold eases in over EASE_MS from its first draw
        for ms in [1000.0, aegis_ui::portrait_fx::EASE_MS as f64] {
            faces.clock.advance(ms, 1.0);
            faces.begin();
            assert!(faces.draw_parts(&mut ui.p, &mut art, &[], &who, &Look::default(), "dialogue", Rect::new(10, 10, 96, 96), &PortraitOpts::default()));
        }
        let live = faces.get("ida").expect("kept").live(faces.now());
        assert!(live.contains(&"eye_glow"), "{live:?}");
    }

    /// G3: a character with no portrait file draws today's initial letter.
    #[test]
    fn no_portrait_file_draws_the_initial_letter() {
        let mut art = Art::new(None);
        let mut g = gfx();
        g.fonts.audit = Some(Vec::new());
        let mut faces = Faces::default();
        let who = Who { key: "zed".into(), tier: "npc".into(), id: "zed".into(), name: "Zed".into(), ..Who::default() };
        {
            let mut ui: Ui<()> = Ui::new(&mut g, (-500, -500), true);
            faces.begin();
            assert!(!faces.draw_parts(&mut ui.p, &mut art, &[], &who, &Look::default(), "dialogue", Rect::new(10, 10, 64, 64), &PortraitOpts::default()));
        }
        let texts = g.fonts.audit.take().unwrap_or_default();
        assert!(texts.iter().any(|t| t.text == "Z"), "the initial is drawn");
    }

    /// G4: under `skip_anim` `react` fires nothing, while the state rows still hold.
    #[test]
    fn skip_anim_fires_no_reaction_and_keeps_the_holds() {
        let mut cx = game_cx();
        let rows = vec![row(json!({"id": "hit_flash", "event": "hit", "effect": "flash", "intensity": 0.6, "ms": 140})), row(json!({"id": "hurt", "trigger": "hp", "below": 50, "effect": "desaturate", "intensity": 0.5}))];
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows.clone());
        let mut faces = Faces::default();
        assert_eq!(faces.react(&cx, "x", &["hit"], "panel", "classic"), 1, "a hit fires its row");
        cx.settings.v["skip_anim"] = json!(true);
        let mut skipped = Faces::default();
        assert_eq!(skipped.react(&cx, "x", &["hit"], "panel", "classic"), 0, "skip_anim fires none");
        let root = layer("skip", "portraits/npcs/x.png", 64, [120, 90, 70], None);
        cx.art = Art::layered(Layers::of(vec![root]));
        let who = Who { key: "x".into(), tier: "npc".into(), id: "x".into(), name: "X".into(), ..Who::default() };
        let look = Look { hp: 10, max_hp: 100, ..Look::default() };
        let mut g = gfx();
        for ms in [0.0, 300.0] {
            skipped.clock.advance(ms, 1.0);
            let mut ui: Ui<()> = Ui::new(&mut g, (-500, -500), true);
            skipped.begin();
            skipped.draw(&mut ui.p, &mut cx, &who, &look, "panel", Rect::new(0, 0, 64, 64));
        }
        let fx = skipped.get("x").expect("kept");
        assert!(fx.level("flash", skipped.now()).is_none(), "no flash");
        assert!(fx.level("desaturate", skipped.now()).is_some_and(|l| (l.i - 0.5).abs() < 1e-3), "the hp row holds");
    }

    /// G7: a sidecar with `rim` "" flickers on Tallow Amber pixels (the game's rim key).
    #[test]
    fn the_rim_key_is_tallow_amber() {
        let rim = [RIM[0], RIM[1], RIM[2]];
        let root = layer("rim", "portraits/npcs/amber.png", 96, rim, Some(json!({"size": 96, "rim": ""})));
        let mut art = Art::layered(Layers::of(vec![root]));
        let mut g = gfx();
        let mut ui: Ui<()> = Ui::new(&mut g, (-500, -500), true);
        let mut faces = Faces::default();
        let who = Who { key: "amber".into(), tier: "npc".into(), id: "amber".into(), name: "Amber".into(), ..Who::default() };
        let opts = faces.opts(&Settings::load(std::env::temp_dir().join("emberwake_portraits_none.json"), crate::screens::default_settings()));
        assert_eq!(opts.rim_default, Some(RIM));
        let seed = aegis_ui::portrait_fx::seed_of("amber");
        // two clock times whose flicker factors differ
        let times: Vec<u64> = (0..200u64).map(|k| k * 100).collect();
        let a = times[0];
        let b = *times.iter().find(|&&t| (aegis_ui::portrait_fx::rim_factor(seed, t) - aegis_ui::portrait_fx::rim_factor(seed, a)).abs() > 0.05).expect("the rim flickers");
        let mut centre = Vec::new();
        for t in [a, b] {
            faces.clock = Clock { ms: t as f64, hold: 0.0 };
            faces.begin();
            assert!(faces.draw_parts(&mut ui.p, &mut art, &[], &who, &Look::default(), "dialogue", Rect::new(0, 0, 96, 96), &opts));
            let img = faces.get("amber").and_then(|f| f.composed()).expect("composed").clone();
            let i = ((img.h / 2 * img.w + img.w / 2) * 4) as usize;
            centre.push(img.px[i..i + 3].to_vec());
        }
        assert_ne!(centre[0], centre[1], "the amber pixel flickers");
        assert!(faces.get("amber").unwrap().live(b).contains(&"rim"));
    }

    /// The chain of a picked file stops at it (its frames first); a common file is only itself.
    #[test]
    fn a_picked_file_chain_ends_at_the_file() {
        let who = Who::file("portraits/npcs/tess.png");
        assert_eq!(who.chain("hurt"), ["portraits/npcs/faces/tess/hurt.png", "portraits/npcs/faces/tess/neutral.png", "portraits/npcs/tess.png"]);
        assert_eq!(Who::file("portraits/common/beast_render_beast.png").chain("hurt"), ["portraits/common/beast_render_beast.png"]);
        assert_eq!(hue_name("#d97706"), "Tallow Amber");
        assert_eq!(hue_name("#123456"), "");
    }

    /// `begin` drops a portrait neither drawn nor reacted to in the frame before (cycling members drops the old one).
    #[test]
    fn begin_drops_portraits_left_out() {
        let mut faces = Faces::default();
        faces.fire("a", Cue::parse("flash", "").unwrap());
        faces.begin();
        assert!(faces.get("a").is_some(), "reacted to: kept");
        faces.begin();
        assert!(faces.get("a").is_none(), "left out a frame: dropped");
    }
}
