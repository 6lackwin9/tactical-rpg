//! Presentation-only per-map atmosphere and zone grading (`map_fx/<map_id>.fx.json` over the asset layers).
//!
//! Game-agnostic: names no game, world, or rule. When no sidecar exists for a map, nothing runs and rendering stays
//! byte-for-byte unchanged. When present, a map may tune per-zone floor/wall brightness and RGB tint plus data-driven
//! light-pool parameters for props, and `ENGINE-64` (`LIVE-RELOAD1`) reloads edits automatically in dev runs.

use crate::canvas::{Canvas, Rect};
use crate::theme::parse_colour;
use aegis_core::assets::Layers;
use aegis_core::vfs;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Folder under each asset layer holding per-map presentation sidecars (`map_fx/<map_id>.fx.json`).
pub const DIR: &str = "map_fx";

/// Safe clamp range for multiplicative RGB channel grades (keeps silhouettes readable and avoids NaN/inf).
pub const MUL_MIN: f32 = 0.25;
pub const MUL_MAX: f32 = 2.50;
/// Safe clamp range for additive RGB channel lifts.
pub const ADD_MIN: f32 = -64.0;
pub const ADD_MAX: f32 = 64.0;
/// Safe clamp range for ambient darkness `[0.0, 0.85]`.
pub const DARK_MAX: f32 = 0.85;
/// Default ambient darkness strength (`96 / 255`) when `dim_ambience` is active.
pub const DEFAULT_DARK: f32 = 96.0 / 255.0;
/// Default ambient shadow colour.
pub const DEFAULT_AMBIENCE_RGB: [u8; 3] = [6, 8, 16];
/// Default light glow blend strength.
pub const DEFAULT_LIGHT_STRENGTH: f32 = 0.28;

/// Multiplicative and additive RGB grading for one map zone.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneGrade {
    /// Multiplicative RGB grade on floor pixels in this zone (default `[1.0, 1.0, 1.0]`).
    pub floor_mul: [f32; 3],
    /// Optional separate multiplicative RGB grade on wall pixels bordering this zone (defaults to `floor_mul`).
    pub wall_mul: Option<[f32; 3]>,
    /// Additive RGB lift applied after multiplication on floor pixels (default `[0.0, 0.0, 0.0]`).
    pub floor_add: [f32; 3],
}

impl Default for ZoneGrade {
    fn default() -> Self {
        Self { floor_mul: [1.0, 1.0, 1.0], wall_mul: None, floor_add: [0.0, 0.0, 0.0] }
    }
}

impl ZoneGrade {
    pub fn is_identity(&self) -> bool {
        let eq = |a: [f32; 3], b: [f32; 3]| (0..3).all(|i| (a[i] - b[i]).abs() < 1e-4);
        eq(self.floor_mul, [1.0, 1.0, 1.0])
            && eq(self.floor_add, [0.0, 0.0, 0.0])
            && self.wall_mul.map_or(true, |w| eq(w, [1.0, 1.0, 1.0]))
    }

    pub fn for_cell(&self, wall: bool) -> ([f32; 3], [f32; 3]) {
        if wall {
            (self.wall_mul.unwrap_or(self.floor_mul), [0.0, 0.0, 0.0])
        } else {
            (self.floor_mul, self.floor_add)
        }
    }
}

/// Data-driven light pool specification for a prop art key or prop id.
#[derive(Debug, Clone, PartialEq)]
pub struct LightSpec {
    /// Pool radius in 32-px art pixels.
    pub radius: f32,
    /// Warm or cool RGB colour of the light pool.
    pub color: [u8; 3],
    /// Peak additive/blend weight in `[0.0, 0.85]` (default `0.28`).
    pub strength: f32,
}

/// Per-map presentation sidecar (`map_fx/<map_id>.fx.json`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MapFx {
    /// Optional override for map-wide dim ambience (falls back to map/location setting when `None`).
    pub dim_ambience: Option<bool>,
    /// Optional ambient darkness strength in `[0.0, 0.85]` (default `96 / 255`).
    pub ambience_dark: Option<f32>,
    /// Optional ambient shadow tint RGB (default `[6, 8, 16]`).
    pub ambience_rgb: Option<[u8; 3]>,
    /// Per-zone floor/wall grading keyed by zone id.
    pub zones: BTreeMap<String, ZoneGrade>,
    /// Per-prop-art or per-prop-id light pool overrides.
    pub lights: BTreeMap<String, LightSpec>,
}

impl MapFx {
    /// True when this sidecar carries no active grading, ambience override, or light override.
    pub fn is_empty(&self) -> bool {
        self.dim_ambience.is_none()
            && self.ambience_dark.is_none()
            && self.ambience_rgb.is_none()
            && self.zones.values().all(ZoneGrade::is_identity)
            && self.lights.is_empty()
    }

    /// Relative asset path for `map_id`.
    pub fn rel_path(map_id: &str) -> String {
        format!("{DIR}/{map_id}.fx.json")
    }

    /// Loads `map_fx/<map_id>.fx.json` from the layered asset roots; `None` when absent or invalid.
    pub fn load_layered(layers: &Layers, map_id: &str) -> Option<Self> {
        let rel = Self::rel_path(map_id);
        let path = layers.find(&rel)?;
        Self::load_file(&path)
    }

    /// Loads a `MapFx` sidecar directly from `path` through the VFS.
    pub fn load_file(path: &Path) -> Option<Self> {
        let text = vfs::read_to_string(path).ok()?;
        let val: Value = serde_json::from_str(&text).ok()?;
        Self::from_json(&val)
    }

    /// Parses a `MapFx` JSON object, clamping numeric fields to safe presentation bounds.
    pub fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let dim_ambience = obj.get("dim_ambience").and_then(Value::as_bool);
        let ambience_dark = obj
            .get("ambience_dark")
            .and_then(Value::as_f64)
            .filter(|x| x.is_finite())
            .map(|x| (x as f32).clamp(0.0, DARK_MAX));
        let ambience_rgb = obj.get("ambience_rgb").and_then(parse_rgb);
        let mut zones = BTreeMap::new();
        if let Some(zm) = obj.get("zones").and_then(Value::as_object) {
            for (zid, zv) in zm {
                let Some(zo) = zv.as_object() else { continue };
                let floor_mul = zo.get("floor_mul").and_then(|x| parse_vec3(x, MUL_MIN, MUL_MAX)).unwrap_or([1.0, 1.0, 1.0]);
                let wall_mul = zo.get("wall_mul").and_then(|x| parse_vec3(x, MUL_MIN, MUL_MAX));
                let floor_add = zo.get("floor_add").and_then(|x| parse_vec3(x, ADD_MIN, ADD_MAX)).unwrap_or([0.0, 0.0, 0.0]);
                zones.insert(zid.clone(), ZoneGrade { floor_mul, wall_mul, floor_add });
            }
        }
        let mut lights = BTreeMap::new();
        if let Some(lm) = obj.get("lights").and_then(Value::as_object) {
            for (key, lv) in lm {
                let Some(lo) = lv.as_object() else { continue };
                let Some(radius) = lo.get("radius").and_then(Value::as_f64).filter(|x| x.is_finite() && *x > 0.0) else {
                    continue;
                };
                let Some(color) = lo.get("color").and_then(parse_rgb) else { continue };
                let strength = lo
                    .get("strength")
                    .and_then(Value::as_f64)
                    .filter(|x| x.is_finite())
                    .map_or(DEFAULT_LIGHT_STRENGTH, |x| (x as f32).clamp(0.0, 0.85));
                lights.insert(key.clone(), LightSpec { radius: (radius as f32).clamp(4.0, 640.0), color, strength });
            }
        }
        Some(Self { dim_ambience, ambience_dark, ambience_rgb, zones, lights })
    }

    /// Serializes this `MapFx` to clean deterministic JSON.
    pub fn to_json(&self) -> Value {
        let r3 = |x: f32| (x as f64 * 1000.0).round() / 1000.0;
        let mut root = Map::new();
        if let Some(dim) = self.dim_ambience {
            root.insert("dim_ambience".into(), json!(dim));
        }
        if let Some(dark) = self.ambience_dark {
            root.insert("ambience_dark".into(), json!(r3(dark)));
        }
        if let Some(rgb) = self.ambience_rgb {
            root.insert("ambience_rgb".into(), json!(rgb));
        }
        if !self.zones.is_empty() {
            let mut zm = Map::new();
            for (k, z) in &self.zones {
                let mut zo = Map::new();
                zo.insert("floor_mul".into(), json!([r3(z.floor_mul[0]), r3(z.floor_mul[1]), r3(z.floor_mul[2])]));
                if let Some(wm) = z.wall_mul {
                    zo.insert("wall_mul".into(), json!([r3(wm[0]), r3(wm[1]), r3(wm[2])]));
                }
                if z.floor_add.iter().any(|v| v.abs() > 1e-3) {
                    zo.insert("floor_add".into(), json!([r3(z.floor_add[0]), r3(z.floor_add[1]), r3(z.floor_add[2])]));
                }
                zm.insert(k.clone(), Value::Object(zo));
            }
            root.insert("zones".into(), Value::Object(zm));
        }
        if !self.lights.is_empty() {
            let mut lm = Map::new();
            for (k, l) in &self.lights {
                lm.insert(
                    k.clone(),
                    json!({
                        "radius": r3(l.radius),
                        "color": l.color,
                        "strength": r3(l.strength),
                    }),
                );
            }
            root.insert("lights".into(), Value::Object(lm));
        }
        Value::Object(root)
    }

    /// Looks up a light override first by prop id, then by prop art key.
    pub fn light_for(&self, prop_id: &str, art: &str) -> Option<&LightSpec> {
        self.lights.get(prop_id).or_else(|| self.lights.get(art))
    }
}

/// Generation-keyed cache for `map_fx/<map_id>.fx.json` over `Art` so `ENGINE-64` live reload picks up edits
/// automatically without per-frame disk I/O.
#[derive(Debug, Default)]
pub struct MapFxCache {
    cached: Option<(String, u64, Option<MapFx>)>,
}

impl MapFxCache {
    pub fn new() -> Self {
        Self { cached: None }
    }

    /// Returns the cached `MapFx` for `map_id` at `art.generation()`, reloading from the asset layers on miss.
    pub fn get(&mut self, art: &crate::art::Art, map_id: &str) -> Option<&MapFx> {
        let gen = art.generation();
        let hit = self.cached.as_ref().is_some_and(|(id, g, _)| id == map_id && *g == gen);
        if !hit {
            let loaded = MapFx::load_layered(art.layers(), map_id);
            self.cached = Some((map_id.to_string(), gen, loaded));
        }
        self.cached.as_ref().and_then(|(_, _, fx)| fx.as_ref())
    }

    /// Cloned convenience wrapper for call sites that pass `&mut` context alongside `fx`.
    pub fn get_cloned(&mut self, art: &crate::art::Art, map_id: &str) -> Option<MapFx> {
        self.get(art, map_id).cloned()
    }

    pub fn clear(&mut self) {
        self.cached = None;
    }
}

/// How an offline static map capture selects its `MapFx` sidecar.
#[derive(Debug, Clone, PartialEq)]
pub enum StaticFx {
    /// Load `map_fx/<map_id>.fx.json` across the layered asset roots if present.
    Layered,
    /// Disable `map_fx` (baseline render without per-map presentation grading).
    Disabled,
    /// Use an explicit `MapFx` (e.g., candidate generated by `aegis align-map`).
    Override(MapFx),
}

impl StaticFx {
    /// Parses a `fx=<path|none|off>` CLI value; `None` selects `StaticFx::Layered`.
    pub fn from_arg(arg: Option<&str>) -> anyhow::Result<Self> {
        match arg {
            None => Ok(Self::Layered),
            Some("none") | Some("off") => Ok(Self::Disabled),
            Some(path) => {
                let fx = MapFx::load_file(Path::new(path))
                    .ok_or_else(|| anyhow::anyhow!("reading MapFx '{path}'"))?;
                Ok(Self::Override(fx))
            }
        }
    }

    /// Resolves the effective `MapFx` for `map_id` against `layers`.
    pub fn resolve(&self, layers: &Layers, map_id: &str) -> Option<MapFx> {
        match self {
            Self::Layered => MapFx::load_layered(layers, map_id),
            Self::Disabled => None,
            Self::Override(fx) => Some(fx.clone()),
        }
    }
}

/// Render layer isolation mode for offline static map capture and layer-wise alignment validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticLayers {
    /// Base tiles, walls, borders, and depth shadows only (no composed ground overlay, no props, no lighting/FX).
    Shell,
    /// Composed ground overlay + walls, borders, and depth shadows (no props, no lighting/FX).
    Ground,
    /// Ground + walls + all props (no `MapFx` zone grading and no dim ambience/light pools).
    Unlit,
    /// Full composite: ground + walls + props + `MapFx` + dim ambience/light pools.
    Full,
}

impl StaticLayers {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s {
            "shell" => Ok(Self::Shell),
            "ground" => Ok(Self::Ground),
            "unlit" => Ok(Self::Unlit),
            "full" => Ok(Self::Full),
            other => anyhow::bail!("render_map: unknown layers '{other}' (expected shell, ground, unlit, full)"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shell => "shell",
            Self::Ground => "ground",
            Self::Unlit => "unlit",
            Self::Full => "full",
        }
    }

    pub fn includes_ground(self) -> bool {
        self != Self::Shell
    }

    pub fn includes_props(self) -> bool {
        matches!(self, Self::Unlit | Self::Full)
    }

    pub fn includes_lighting(self) -> bool {
        self == Self::Full
    }
}

/// Parsed offline `render_map=<id> out=<file.png> [fx=<path|none>] [layers=shell|ground|unlit|full] [map_override=<json>] [view=<mode>]` request.
#[derive(Debug, Clone)]
pub struct RenderMapRequest {
    pub map_id: String,
    pub out: std::path::PathBuf,
    pub view: String,
    pub fx: StaticFx,
    pub layers: StaticLayers,
    pub map_override: Option<std::path::PathBuf>,
}

impl RenderMapRequest {
    /// Parses `render_map=` from `(key, value)` launch args. Returns `Ok(None)` when `render_map=` is absent.
    pub fn from_args(args: &[(String, String)]) -> anyhow::Result<Option<Self>> {
        let get = |k: &str| args.iter().rev().find(|(ak, _)| ak == k).map(|(_, v)| v.as_str());
        let Some(map_id) = get("render_map") else { return Ok(None) };
        let out = get("out").ok_or_else(|| anyhow::anyhow!("render_map requires out=<path.png>"))?;
        let view = get("view").unwrap_or("square").to_string();
        let fx = StaticFx::from_arg(get("fx"))?;
        let layers = StaticLayers::parse(get("layers").unwrap_or("full"))?;
        let map_override = get("map_override").map(std::path::PathBuf::from);
        Ok(Some(Self {
            map_id: map_id.to_string(),
            out: std::path::PathBuf::from(out),
            view,
            fx,
            layers,
            map_override,
        }))
    }
}

/// Engine-owned offline static map capture hook implemented by any Aegis game that supports `aegis align-map`.
pub trait StaticMapRenderer {
    fn render_static_map(&mut self, req: &RenderMapRequest) -> anyhow::Result<Canvas>;
}

/// Runs `renderer` for `req`, writes the captured PNG to `req.out`, and prints the standard capture summary line.
#[cfg(not(target_arch = "wasm32"))]
pub fn write_capture<R: StaticMapRenderer>(renderer: &mut R, req: &RenderMapRequest) -> anyhow::Result<(i32, i32)> {
    let canvas = renderer.render_static_map(req)?;
    if let Some(parent) = req.out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    crate::shell::save_png(&canvas, &req.out, canvas.w)?;
    println!("rendered {} ({}x{}) -> {}", req.map_id, canvas.w, canvas.h, req.out.display());
    Ok((canvas.w, canvas.h))
}

fn parse_vec3(v: &Value, lo: f32, hi: f32) -> Option<[f32; 3]> {
    let a = v.as_array()?;
    if a.len() != 3 {
        return None;
    }
    let mut out = [0.0f32; 3];
    for (i, item) in a.iter().enumerate() {
        let f = item.as_f64().filter(|x| x.is_finite())?;
        out[i] = (f as f32).clamp(lo, hi);
    }
    Some(out)
}

fn parse_rgb(v: &Value) -> Option<[u8; 3]> {
    if let Some(s) = v.as_str() {
        let c = parse_colour(s)?;
        return Some([c[0], c[1], c[2]]);
    }
    let a = v.as_array()?;
    if a.len() != 3 {
        return None;
    }
    let mut out = [0u8; 3];
    for (i, item) in a.iter().enumerate() {
        let n = item.as_i64()?;
        out[i] = n.clamp(0, 255) as u8;
    }
    Some(out)
}

/// Grid-space transform mapping physical canvas pixels onto map cell coordinates.
#[derive(Debug, Clone, Copy)]
pub struct GradeGrid {
    /// Physical canvas x, y of cell `(0, 0)` top-left corner.
    pub ox: f32,
    pub oy: f32,
    /// Physical pixels per cell along x and y (must be > 0).
    pub cw: f32,
    pub ch: f32,
    pub cols: i32,
    pub rows: i32,
}

/// Resolves per-cell grades from `fx` using `is_wall` and `zone_at` (with south-facing wall inheritance so room
/// north walls share the room's grade) and applies feathered zone grading over `area`.
pub fn apply_zone_grades_with(
    canvas: &mut Canvas,
    area: Rect,
    grid: GradeGrid,
    fx: &MapFx,
    is_wall: impl Fn(i32, i32) -> bool,
    zone_at: impl Fn(i32, i32) -> String,
) {
    if grid.cols <= 0 || grid.rows <= 0 || fx.zones.is_empty() {
        return;
    }
    let n = (grid.cols * grid.rows) as usize;
    let mut cell_grades = Vec::with_capacity(n);
    let mut cell_is_wall = Vec::with_capacity(n);
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            let wall = is_wall(x, y);
            cell_is_wall.push(wall);
            let zone_id = if !wall {
                zone_at(x, y)
            } else {
                // South-facing wall cells belong visually to the room they face, then own zone, then neighbours.
                let mut picked = String::new();
                for (nx, ny) in [(x, y + 1), (x, y), (x, y - 1), (x - 1, y), (x + 1, y)] {
                    if nx >= 0 && ny >= 0 && nx < grid.cols && ny < grid.rows && (!is_wall(nx, ny) || (nx == x && ny == y)) {
                        let z = zone_at(nx, ny);
                        if !z.is_empty() {
                            picked = z;
                            break;
                        }
                    }
                }
                picked
            };
            let grade = fx.zones.get(&zone_id).filter(|g| !g.is_identity()).map(|g| g.for_cell(wall));
            cell_grades.push(grade);
        }
    }
    apply_zone_grades(canvas, area, grid, &cell_grades, &cell_is_wall);
}

fn visible_rect(canvas: &Canvas, area: Rect) -> Option<Rect> {
    let mut out = area.intersect(Rect::new(0, 0, canvas.w, canvas.h));
    if let Some(clip) = canvas.clip {
        out = out.intersect(clip);
    }
    if let Some(bounds) = canvas.bounds {
        out = out.intersect(bounds);
    }
    (out.w > 0 && out.h > 0).then_some(out)
}

/// Applies per-cell `(mul, add)` RGB grades across `area` of `canvas` (clipped to the canvas bounds).
///
/// `cell_grades` is row-major (`rows * cols`); `None` cells are left untouched. Adjacent non-wall cells with different
/// grades softly blend across a thin feather band (25% of a cell around the boundary) so doorways transition smoothly
/// without leaking across walls.
pub fn apply_zone_grades(
    canvas: &mut Canvas,
    area: Rect,
    grid: GradeGrid,
    cell_grades: &[Option<([f32; 3], [f32; 3])>],
    cell_is_wall: &[bool],
) {
    let n = (grid.cols.max(0) as usize) * (grid.rows.max(0) as usize);
    if n == 0 || cell_grades.len() != n || cell_is_wall.len() != n || grid.cw <= 0.0 || grid.ch <= 0.0 {
        return;
    }
    if cell_grades.iter().all(|g| g.is_none()) {
        return;
    }
    let Some(vis) = visible_rect(canvas, area) else {
        return;
    };
    let cols = grid.cols;
    let rows = grid.rows;
    let idx = |cx: i32, cy: i32| (cy * cols + cx) as usize;
    let inv_cw = 1.0 / grid.cw;
    let inv_ch = 1.0 / grid.ch;
    let w = canvas.w as usize;

    for py in vis.y..(vis.y + vis.h) {
        let gy = (py as f32 + 0.5 - grid.oy) * inv_ch;
        let cy = gy.floor() as i32;
        if cy < 0 || cy >= rows {
            continue;
        }
        let fy = gy - cy as f32;
        let row_off = py as usize * w;
        for px in vis.x..(vis.x + vis.w) {
            let gx = (px as f32 + 0.5 - grid.ox) * inv_cw;
            let cx = gx.floor() as i32;
            if cx < 0 || cx >= cols {
                continue;
            }
            let ci = idx(cx, cy);
            let Some((mut mul, mut add)) = cell_grades[ci] else {
                continue;
            };
            // Feather across doorways between two graded floor zones (never across walls).
            if !cell_is_wall[ci] {
                let fx = gx - cx as f32;
                let blend_axis = |frac: f32, nx: i32, ny: i32, mul: &mut [f32; 3], add: &mut [f32; 3]| {
                    if nx < 0 || nx >= cols || ny < 0 || ny >= rows {
                        return;
                    }
                    let ni = idx(nx, ny);
                    if cell_is_wall[ni] {
                        return;
                    }
                    let (nm, na) = cell_grades[ni].unwrap_or(([1.0, 1.0, 1.0], [0.0, 0.0, 0.0]));
                    let t = ((0.25 - frac) / 0.25).clamp(0.0, 1.0) * 0.5;
                    if t > 0.0 {
                        for c in 0..3 {
                            mul[c] = mul[c] * (1.0 - t) + nm[c] * t;
                            add[c] = add[c] * (1.0 - t) + na[c] * t;
                        }
                    }
                };
                if fx < 0.25 {
                    blend_axis(fx, cx - 1, cy, &mut mul, &mut add);
                } else if fx > 0.75 {
                    blend_axis(1.0 - fx, cx + 1, cy, &mut mul, &mut add);
                }
                if fy < 0.25 {
                    blend_axis(fy, cx, cy - 1, &mut mul, &mut add);
                } else if fy > 0.75 {
                    blend_axis(1.0 - fy, cx, cy + 1, &mut mul, &mut add);
                }
            }
            let i = (row_off + px as usize) * 4;
            for c in 0..3 {
                let v = canvas.px[i + c] as f32 * mul[c] + add[c];
                canvas.px[i + c] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Physical-pixel elliptical light pool rendered by `apply_ambience_and_lights`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightPool {
    /// Center in physical canvas pixels.
    pub x: f32,
    pub y: f32,
    /// Horizontal and vertical radii in physical canvas pixels.
    pub rx: f32,
    pub ry: f32,
    /// Light RGB colour.
    pub color: [u8; 3],
    /// Peak glow blend weight in `[0.0, 0.85]`.
    pub strength: f32,
}

/// Evening/atmosphere darkening parameters for `apply_ambience_and_lights`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbienceConfig {
    /// Ambient darkness strength in `[0.0, 0.85]`.
    pub dark: f32,
    /// Ambient shadow tint RGB.
    pub rgb: [u8; 3],
    /// Maximum combined light-pool glow blend alpha.
    pub glow_cap: f32,
}

impl AmbienceConfig {
    pub fn from_fx(fx: Option<&MapFx>) -> Self {
        Self {
            dark: fx.and_then(|f| f.ambience_dark).unwrap_or(DEFAULT_DARK),
            rgb: fx.and_then(|f| f.ambience_rgb).unwrap_or(DEFAULT_AMBIENCE_RGB),
            glow_cap: if fx.is_some() { 0.45 } else { 0.32 },
        }
    }
}

/// Applies evening ambient darkening and elliptical coloured light pools across `area` of `canvas`
/// (intersected with canvas bounds and active clip rect).
pub fn apply_ambience_and_lights(canvas: &mut Canvas, area: Rect, ambience: AmbienceConfig, pools: &[LightPool]) {
    let Some(vis) = visible_rect(canvas, area) else {
        return;
    };
    struct PreparedLight {
        lx: f32,
        ly: f32,
        inv_rx2: f32,
        inv_ry2: f32,
        bx0: i32,
        by0: i32,
        bx1: i32,
        by1: i32,
        col: [f32; 3],
        strength: f32,
    }
    let mut lights = Vec::with_capacity(pools.len());
    for p in pools {
        if p.rx < 1.0 || p.ry < 1.0 {
            continue;
        }
        let bx0 = (p.x - p.rx).floor().max(vis.x as f32) as i32;
        let by0 = (p.y - p.ry).floor().max(vis.y as f32) as i32;
        let bx1 = (p.x + p.rx).ceil().min((vis.x + vis.w) as f32) as i32;
        let by1 = (p.y + p.ry).ceil().min((vis.y + vis.h) as f32) as i32;
        if bx1 > bx0 && by1 > by0 {
            lights.push(PreparedLight {
                lx: p.x,
                ly: p.y,
                inv_rx2: 1.0 / (p.rx * p.rx),
                inv_ry2: 1.0 / (p.ry * p.ry),
                bx0,
                by0,
                bx1,
                by1,
                col: [p.color[0] as f32, p.color[1] as f32, p.color[2] as f32],
                strength: p.strength,
            });
        }
    }
    let dark_max = ambience.dark.clamp(0.0, DARK_MAX) * 255.0;
    let (ar, ag, ab) = (ambience.rgb[0] as f32, ambience.rgb[1] as f32, ambience.rgb[2] as f32);
    let w = canvas.w;
    for py in vis.y..(vis.y + vis.h) {
        let fy = py as f32 + 0.5;
        let row = (py * w) as usize;
        for px in vis.x..(vis.x + vis.w) {
            let fx_px = px as f32 + 0.5;
            let mut min_dark = dark_max;
            let (mut gr, mut gg, mut gb, mut gw) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
            for l in &lights {
                if px < l.bx0 || px >= l.bx1 || py < l.by0 || py >= l.by1 {
                    continue;
                }
                let dx = fx_px - l.lx;
                let dy = fy - l.ly;
                let d2 = dx * dx * l.inv_rx2 + dy * dy * l.inv_ry2;
                if d2 < 1.0 {
                    let d = d2.sqrt();
                    min_dark = min_dark.min(dark_max * d.powf(1.6));
                    let wt = (1.0 - d).powf(1.3) * l.strength;
                    gr += l.col[0] * wt;
                    gg += l.col[1] * wt;
                    gb += l.col[2] * wt;
                    gw += wt;
                }
            }
            let i = (row + px as usize) * 4;
            let da = min_dark / 255.0;
            let inv = 1.0 - da;
            let mut r = canvas.px[i] as f32 * inv + ar * da;
            let mut g = canvas.px[i + 1] as f32 * inv + ag * da;
            let mut b = canvas.px[i + 2] as f32 * inv + ab * da;
            if gw > 0.0 {
                let ga = gw.min(ambience.glow_cap);
                let ig = 1.0 - ga;
                r = r * ig + (gr / gw) * ga;
                g = g * ig + (gg / gw) * ga;
                b = b * ig + (gb / gw) * ga;
            }
            canvas.px[i] = r.round().clamp(0.0, 255.0) as u8;
            canvas.px[i + 1] = g.round().clamp(0.0, 255.0) as u8;
            canvas.px[i + 2] = b.round().clamp(0.0, 255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::rgb;

    #[test]
    fn json_round_trip_and_clamping() {
        let raw = json!({
            "dim_ambience": true,
            "ambience_dark": 0.42,
            "ambience_rgb": "#0a0c14",
            "zones": {
                "bright_room": {
                    "floor_mul": [1.8, 1.85, 1.9],
                    "wall_mul": [1.4, 1.4, 1.45],
                    "floor_add": [8.0, 8.0, 10.0]
                },
                "clamped_room": {
                    "floor_mul": [99.0, 0.01, 1.0],
                    "floor_add": [-200.0, 200.0, 0.0]
                }
            },
            "lights": {
                "brazier": { "radius": 160.0, "color": "#ffaa50", "strength": 0.35 }
            }
        });
        let fx = MapFx::from_json(&raw).expect("valid fx");
        assert_eq!(fx.dim_ambience, Some(true));
        assert_eq!(fx.ambience_rgb, Some([10, 12, 20]));
        assert_eq!(fx.zones["clamped_room"].floor_mul, [MUL_MAX, MUL_MIN, 1.0]);
        assert_eq!(fx.zones["clamped_room"].floor_add, [ADD_MIN, ADD_MAX, 0.0]);
        assert_eq!(fx.light_for("x", "brazier").unwrap().color, [255, 170, 80]);

        let back = MapFx::from_json(&fx.to_json()).expect("round-trip");
        assert_eq!(back.dim_ambience, fx.dim_ambience);
        assert_eq!(back.ambience_rgb, fx.ambience_rgb);
        assert_eq!(back.zones.len(), fx.zones.len());
        assert_eq!(back.lights.len(), fx.lights.len());
    }

    #[test]
    fn zone_grades_adjust_floor_and_respect_walls() {
        let mut c = Canvas::new(16, 8);
        c.fill(Rect::new(0, 0, 16, 8), rgb(100, 100, 100));
        let grid = GradeGrid { ox: 0.0, oy: 0.0, cw: 8.0, ch: 8.0, cols: 2, rows: 1 };
        let grades = vec![
            Some(([1.5, 1.0, 0.5], [10.0, 0.0, 0.0])),
            Some(([0.5, 0.5, 0.5], [0.0, 0.0, 0.0])),
        ];
        // Second cell is a wall, so no feathering leaks across the border.
        let walls = vec![false, true];
        apply_zone_grades(&mut c, Rect::new(0, 0, 16, 8), grid, &grades, &walls);

        // Centre of cell 0: (100 * 1.5 + 10 = 160, 100, 50)
        let i0 = ((4 * 16 + 4) * 4) as usize;
        assert_eq!(&c.px[i0..i0 + 3], &[160, 100, 50]);
        // Centre of cell 1 (wall): (50, 50, 50)
        let i1 = ((4 * 16 + 12) * 4) as usize;
        assert_eq!(&c.px[i1..i1 + 3], &[50, 50, 50]);
    }

    #[test]
    fn zone_grades_with_inherits_south_room_wall_grade() {
        let mut c = Canvas::new(8, 16);
        c.fill(Rect::new(0, 0, 8, 16), rgb(100, 100, 100));
        let mut fx = MapFx::default();
        fx.zones.insert(
            "hall".into(),
            ZoneGrade { floor_mul: [1.4, 1.4, 1.4], wall_mul: Some([0.6, 0.6, 0.6]), floor_add: [0.0; 3] },
        );
        let grid = GradeGrid { ox: 0.0, oy: 0.0, cw: 8.0, ch: 8.0, cols: 1, rows: 2 };
        // Row 0 is a wall with empty zone; row 1 is open floor in "hall".
        apply_zone_grades_with(
            &mut c,
            Rect::new(0, 0, 8, 16),
            grid,
            &fx,
            |_x, y| y == 0,
            |_x, y| if y == 1 { "hall".into() } else { String::new() },
        );
        let wall_i = ((4 * 8 + 4) * 4) as usize;
        let floor_i = ((12 * 8 + 4) * 4) as usize;
        assert_eq!(&c.px[wall_i..wall_i + 3], &[60, 60, 60]);
        assert_eq!(&c.px[floor_i..floor_i + 3], &[140, 140, 140]);
    }

    #[test]
    fn ambience_and_lights_darkens_outer_and_warms_pool_center() {
        let mut c = Canvas::new(32, 32);
        c.fill(Rect::new(0, 0, 32, 32), rgb(100, 100, 100));
        let amb = AmbienceConfig { dark: 0.5, rgb: [0, 0, 0], glow_cap: 0.45 };
        let pools = [LightPool { x: 16.0, y: 16.0, rx: 10.0, ry: 10.0, color: [255, 160, 60], strength: 0.4 }];
        apply_ambience_and_lights(&mut c, Rect::new(0, 0, 32, 32), amb, &pools);
        let corner = 0usize;
        let center = ((16 * 32 + 16) * 4) as usize;
        // Corner is outside the pool: darkened by 50% -> ~50
        assert_eq!(&c.px[corner..corner + 3], &[50, 50, 50]);
        // Center is lifted and warm-tinted: R > G > B and brighter than corner
        assert!(c.px[center] > c.px[center + 2] + 30);
        assert!(c.px[center] > 120);
    }

    #[test]
    fn static_layers_and_render_map_request_parse_cleanly() {
        assert_eq!(StaticLayers::parse("shell").unwrap(), StaticLayers::Shell);
        assert!(!StaticLayers::Shell.includes_ground());
        assert!(StaticLayers::Ground.includes_ground());
        assert!(!StaticLayers::Ground.includes_props());
        assert!(StaticLayers::Unlit.includes_props());
        assert!(!StaticLayers::Unlit.includes_lighting());
        assert!(StaticLayers::Full.includes_lighting());

        let args = vec![
            ("render_map".into(), "demo_hall".into()),
            ("out".into(), "/tmp/demo.png".into()),
            ("fx".into(), "none".into()),
            ("layers".into(), "unlit".into()),
            ("view".into(), "tilted".into()),
        ];
        let req = RenderMapRequest::from_args(&args).unwrap().expect("parsed request");
        assert_eq!(req.map_id, "demo_hall");
        assert_eq!(req.fx, StaticFx::Disabled);
        assert_eq!(req.layers, StaticLayers::Unlit);
        assert_eq!(req.view, "tilted");
    }
}
