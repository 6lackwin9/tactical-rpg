//! The runtime theme (LOOK1-E1, spec LOOK1 section 3; engine-only, no Godot source):
//! colour roles, three font faces, layout values and nine-slice frames, read from `ui/theme.json` and `ui/frames/`
//! over the asset layers (engine, kit, game, bottom-up: each file sets only the keys it names). A key no file sets
//! keeps `Theme::default()`, which is the look before the theme (every colour constant of `ui`), so with no theme
//! file every widget draws pixel for pixel as before. The engine ships no theme file. Widgets draw through `th()`, the
//! theme in force on this thread (the shell's event loop); `load` checks everything and returns every problem, which
//! the shell shows through `show_errors` while the previous theme stays.

use crate::canvas::{rgb, rgba, Image, Rgba};
use crate::painter::{Typeface, View};
use crate::panes::TAB_H;
use crate::screen::{show_errors, Cx};
use crate::ui;
use aegis_core::assets::Layers;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

/// The theme file's path in an asset layer.
pub const FILE: &str = "ui/theme.json";
/// The frames' folder in an asset layer (`<role>.png` with its `<role>.json` sidecar).
pub const FRAMES_DIR: &str = "ui/frames";
/// The body face's file when no theme names one (the shell's UI font).
pub const BODY_FONT: &str = "fonts/OpenSans-SemiBold.ttf";
const TOP_KEYS: [&str; 7] = ["version", "colors", "game_colors", "fonts", "layout", "metrics", "frames"];

/// The colour roles: the theme's fields, `ROLES` (names in table order), `ROLE_DOCS` (what each is drawn for) and the
/// lookups by name. The first 17 are the spec's table (LOOK1 3.2); the rest are values the engine widgets drew that
/// the table lacked, each defaulting to that value (named in E1's report).
macro_rules! theme_roles {
    ($($name:ident = $def:expr, $doc:literal;)*) => {
        /// The theme in force (`th()`): colour roles, game colours, faces, layout values and frames.
        #[derive(Debug, Clone)]
        pub struct Theme {
            $(#[doc = $doc] pub $name: Rgba,)*
            /// Colours only game code reads (`Theme::game`), by name.
            pub game_colors: BTreeMap<String, Rgba>,
            /// The faces (`Typeface` order: body, display, mono).
            pub faces: [FontSpec; 3],
            pub layout: Layout,
            /// The `metrics` keys the theme files name: tokens of a screen on the layout grid (`Metrics::of`).
            pub metrics: MetricKeys,
            /// `frames: false` ignores every frame file (vector panels in the colours).
            pub frames_on: bool,
            /// The frame files found, by role (the game's over the kit's over the engine's).
            pub frames: BTreeMap<String, Frame>,
            /// The theme files that applied, bottom-up (`ui/theme.json (kit)`).
            pub sources: Vec<String>,
        }

        impl Default for Theme {
            fn default() -> Self {
                let body = FontSpec { file: BODY_FONT.into(), scale: 1.0, bytes: None };
                Self { $($name: $def,)* game_colors: BTreeMap::new(), faces: [body.clone(), body.clone(), body], layout: Layout::default(), metrics: MetricKeys::default(), frames_on: true, frames: BTreeMap::new(), sources: Vec::new() }
            }
        }

        /// Every colour role, in table order.
        pub const ROLES: &[&str] = &[$(stringify!($name),)*];
        /// What each role in `ROLES` is drawn for.
        pub const ROLE_DOCS: &[&str] = &[$($doc,)*];

        impl Theme {
            /// The colour of role `name`.
            pub fn role(&self, name: &str) -> Option<Rgba> {
                match name {
                    $(stringify!($name) => Some(self.$name),)*
                    _ => None,
                }
            }

            fn role_mut(&mut self, name: &str) -> Option<&mut Rgba> {
                match name {
                    $(stringify!($name) => Some(&mut self.$name),)*
                    _ => None,
                }
            }
        }
    };
}

theme_roles! {
    bg = ui::BG, "Screen ground.";
    panel = ui::MODAL, "`Ui::frame` and popup fill.";
    panel_edge = ui::FRAME, "Frame edge.";
    frame_line = rgb(146, 64, 14), "A frame's thin accent line: the one shade every frame file is generated with.";
    card = ui::CARD, "Button, card and row fill.";
    card_hot = ui::PANEL_HI, "Hovered fill.";
    selected = ui::SELECTED, "Active or selected fill.";
    edge = ui::BORDER, "Idle and disabled border.";
    edge_live = rgb(100, 130, 186), "Enabled button border.";
    edge_hot = rgb(110, 140, 196), "Hovered card border.";
    focus = ui::GOLD, "Hover and active button border, the active label, primary action.";
    text = ui::TEXT, "Body text.";
    dim = ui::DIM, "Secondary and disabled text.";
    gold = ui::GOLD, "Headers, titles, tip titles.";
    info = ui::INFO, "Neutral highlight.";
    foe = ui::FOE, "Foes, damage.";
    good = ui::GOOD, "Good changes, HP.";
    ink = rgb(16, 16, 20), "Text on light panes (parchment).";
    input = rgb(22, 30, 48), "Text field fill.";
    field = rgb(30, 44, 74), "Dropdown and select fill.";
    section = rgb(26, 38, 64), "Section header fill.";
    check_on = rgb(40, 62, 110), "Checked box fill.";
    toggled_edge = rgb(150, 124, 56), "Toggled icon button border.";
    selection = rgba(74, 110, 180, 170), "Selected text in a field.";
    track = rgb(40, 46, 60), "Slider and stepper track.";
    track_fill = rgb(200, 204, 214), "Slider and stepper filled part.";
    knob = rgb(240, 242, 248), "Slider and stepper knob.";
    scroll_track = rgba(40, 46, 60, 200), "Scroll bar.";
    scroll_knob = rgb(150, 160, 180), "Scroll bar knob.";
    tip = rgba(12, 18, 30, 250), "Hover card fill.";
    scrim = rgba(0, 0, 0, 120), "The dim under a question or notice.";
    drag_shade = rgba(10, 14, 24, 150), "Over the widget being dragged.";
    ghost = rgba(18, 26, 44, 220), "The dragged widget's card under the cursor.";
    drop_hot = rgba(244, 198, 84, 40), "The drop target under the cursor.";
    ring = rgb(255, 255, 255), "The keyboard focus ring.";
    parchment = ui::TEXT, "Parchment inset fill (no frame file).";
    title_idle = rgb(26, 36, 58), "Title bar of a window not in front.";
    ruler = rgb(16, 22, 36), "A timeline's ruler strip.";
    playing = rgb(52, 96, 150), "The block being played: its fill.";
    playing_edge = rgb(128, 220, 255), "The block being played: its border and, faint, its column over the lanes.";
    lane = rgb(24, 30, 44), "A timeline lane's cells.";
    lane_stripe = rgba(60, 66, 84, 140), "The stripes over a masked lane.";
    lane_hatch = rgba(120, 128, 150, 90), "The hatching over a hatched lane.";
    impact_edge = rgba(240, 244, 255, 220), "The light edge where a timeline part lands.";
    socket = rgb(6, 9, 16), "A pip gauge's free socket.";
    socket_edge = rgba(200, 210, 228, 220), "A pip gauge's socket outline.";
    tool_bg = rgb(12, 16, 26), "A tool screen's ground (the recordings).";
    tool_pane = rgb(18, 24, 38), "A tool screen's pane fill (the recordings).";
    stage = rgb(10, 11, 16), "A vignette reel's frame and backdrop when it names none.";
    mote = rgb(206, 204, 200), "The drifting specks over a vignette card (their alpha is the card's).";
    grip = rgb(110, 140, 196), "A window's resize grip with the mouse off it (`focus` on it).";
}

/// A face: its font file (layer-relative) and size factor; `bytes` None is the shell's UI font.
#[derive(Debug, Clone, PartialEq)]
pub struct FontSpec {
    pub file: String,
    pub scale: f32,
    pub bytes: Option<Arc<Vec<u8>>>,
}

/// Layout values in logical px (LOOK1 3.4); the old layouts keep their own numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub radius: f32,
    pub frame_radius: f32,
    pub frame_title_h: i32,
    pub grid: i32,
    pub margin: i32,
    pub gutter: i32,
    pub padding: i32,
    /// A ring this many px wide in `focus` outside the hovered or active button (0: none).
    pub focus_glow: i32,
}

impl Default for Layout {
    fn default() -> Self {
        Self { radius: ui::RADIUS, frame_radius: 4.0, frame_title_h: 26, grid: 4, margin: 8, gutter: 6, padding: 6, focus_glow: 0 }
    }
}

/// The layout keys with their ranges and whether they take whole numbers.
pub const LAYOUT_KEYS: [(&str, f32, f32, bool); 8] =
    [("radius", 0.0, 8.0, false), ("frame_radius", 0.0, 8.0, false), ("frame_title_h", 12.0, 48.0, true), ("grid", 1.0, 8.0, true), ("margin", 0.0, 32.0, true), ("gutter", 0.0, 24.0, true), ("padding", 0.0, 24.0, true), ("focus_glow", 0.0, 3.0, true)];

impl Layout {
    /// The value of key `k` as a number.
    pub fn get(&self, k: &str) -> f32 {
        match k {
            "radius" => self.radius,
            "frame_radius" => self.frame_radius,
            "frame_title_h" => self.frame_title_h as f32,
            "grid" => self.grid as f32,
            "margin" => self.margin as f32,
            "gutter" => self.gutter as f32,
            "padding" => self.padding as f32,
            "focus_glow" => self.focus_glow as f32,
            _ => 0.0,
        }
    }

    fn set(&mut self, k: &str, v: f32) {
        match k {
            "radius" => self.radius = v,
            "frame_radius" => self.frame_radius = v,
            "frame_title_h" => self.frame_title_h = v as i32,
            "grid" => self.grid = v as i32,
            "margin" => self.margin = v as i32,
            "gutter" => self.gutter = v as i32,
            "padding" => self.padding = v as i32,
            "focus_glow" => self.focus_glow = v as i32,
            _ => {}
        }
    }
}

/// The theme files' `metrics` block (LAYOUT720 section 14): only the keys a file names, each in grid units. A key no file
/// names is today's value on that grid (`Metrics::of`); on `Grid::Old` and `Grid::Quadrant` none of them counts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MetricKeys {
    pub body: Option<f32>,
    pub small: Option<f32>,
    pub title: Option<f32>,
    pub head: Option<f32>,
    pub line: Option<f32>,
    pub radius: Option<f32>,
    pub margin: Option<i32>,
    pub gutter: Option<i32>,
    pub padding: Option<i32>,
    pub grid: Option<i32>,
    pub frame_title_h: Option<i32>,
    pub tab_h: Option<i32>,
    pub row_h: Option<i32>,
    pub focus_glow: Option<i32>,
    pub breakpoints_width: Option<[i32; 2]>,
    pub breakpoints_height: Option<[i32; 2]>,
}

/// The `metrics` keys that take one number: their ranges and whether they take whole numbers. Each range keeps the key
/// above 0 unless the spec says otherwise (`radius`, `focus_glow`), and what `layout` allows, doubled, where it has the key.
pub const METRIC_KEYS: [(&str, f32, f32, bool); 14] = [
    ("body", 1.0, 64.0, false),
    ("small", 1.0, 64.0, false),
    ("title", 1.0, 64.0, false),
    ("head", 1.0, 64.0, false),
    ("line", 1.0, 96.0, false),
    ("radius", 0.0, 16.0, false),
    ("margin", 1.0, 64.0, true),
    ("gutter", 1.0, 48.0, true),
    ("padding", 1.0, 48.0, true),
    ("grid", 1.0, 16.0, true),
    ("frame_title_h", 24.0, 96.0, true),
    ("tab_h", 1.0, 96.0, true),
    ("row_h", 1.0, 96.0, true),
    ("focus_glow", 0.0, 6.0, true),
];

/// The most a size-class limit can be, in units.
pub const BREAKPOINT_MAX: i64 = 100_000;

impl MetricKeys {
    fn set(&mut self, k: &str, v: f32) {
        match k {
            "body" => self.body = Some(v),
            "small" => self.small = Some(v),
            "title" => self.title = Some(v),
            "head" => self.head = Some(v),
            "line" => self.line = Some(v),
            "radius" => self.radius = Some(v),
            "margin" => self.margin = Some(v as i32),
            "gutter" => self.gutter = Some(v as i32),
            "padding" => self.padding = Some(v as i32),
            "grid" => self.grid = Some(v as i32),
            "frame_title_h" => self.frame_title_h = Some(v as i32),
            "tab_h" => self.tab_h = Some(v as i32),
            "row_h" => self.row_h = Some(v as i32),
            "focus_glow" => self.focus_glow = Some(v as i32),
            _ => {}
        }
    }
}

/// The size-class limits, in grid units: `[compact, medium]` per axis. Under `compact` an axis is Compact, under `medium`
/// Medium, else Expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Breakpoints {
    pub width: [i32; 2],
    pub height: [i32; 2],
}

/// The limits with no `metrics.breakpoints` (assumed, to tune with the first real target; LAYOUT720 6.4).
pub const BREAKPOINTS: Breakpoints = Breakpoints { width: [800, 1200], height: [480, 700] };
/// A row's height in old units (the shop's rows).
const ROW_H: i32 = 14;

/// What a screen's layout reads (LAYOUT720 6.2, 14): text sizes, gaps and small control heights, in the units of the view
/// it was made for. `Ui::m` is the one read path; `Metrics::of` is for what draws without a `Ui`. On `Grid::Old` and
/// `Grid::Quadrant` every value is today's constant or `Theme::layout`, so `metrics` changes nothing there. On a grid of
/// more units per old unit a value is the `metrics` key when a file names it, else today's value times the units, so a
/// screen moved to the grid draws its old picture under any theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub body: f32,
    pub small: f32,
    pub title: f32,
    pub head: f32,
    pub line: f32,
    pub radius: f32,
    /// `Layout::frame_radius` in these units: a frame's vector panel. No `metrics` key.
    pub frame_radius: f32,
    pub margin: i32,
    pub gutter: i32,
    pub padding: i32,
    pub grid: i32,
    pub frame_title_h: i32,
    pub tab_h: i32,
    pub row_h: i32,
    pub focus_glow: i32,
    /// The thinnest fill that still shows, in units: one physical px at least, at any scale.
    pub hair: i32,
    pub breakpoints: Breakpoints,
    /// Units per old unit (`View::g`).
    pub g: i32,
}

impl Metrics {
    /// The metrics of a screen drawn under `v` in theme `t`.
    pub fn of(t: &Theme, v: View) -> Metrics {
        let (l, k, g) = (&t.layout, &t.metrics, v.g.max(1));
        let f = g as f32;
        let hair = ((1.0 / v.s).ceil() as i32).max(1);
        let breakpoints = Breakpoints { width: k.breakpoints_width.unwrap_or(BREAKPOINTS.width), height: k.breakpoints_height.unwrap_or(BREAKPOINTS.height) };
        if g == 1 {
            return Metrics { body: ui::BODY, small: ui::SMALL, title: ui::TITLE, head: ui::HEAD, line: ui::LINE, radius: l.radius, frame_radius: l.frame_radius, margin: l.margin, gutter: l.gutter, padding: l.padding, grid: l.grid, frame_title_h: l.frame_title_h, tab_h: TAB_H, row_h: ROW_H, focus_glow: l.focus_glow, hair, breakpoints, g };
        }
        Metrics {
            body: k.body.unwrap_or(ui::BODY * f),
            small: k.small.unwrap_or(ui::SMALL * f),
            title: k.title.unwrap_or(ui::TITLE * f),
            head: k.head.unwrap_or(ui::HEAD * f),
            line: k.line.unwrap_or(ui::LINE * f),
            radius: k.radius.unwrap_or(l.radius * f),
            frame_radius: l.frame_radius * f,
            margin: k.margin.unwrap_or(l.margin * g),
            gutter: k.gutter.unwrap_or(l.gutter * g),
            padding: k.padding.unwrap_or(l.padding * g),
            grid: k.grid.unwrap_or(l.grid * g),
            frame_title_h: k.frame_title_h.unwrap_or(l.frame_title_h * g),
            tab_h: k.tab_h.unwrap_or(TAB_H * g),
            row_h: k.row_h.unwrap_or(ROW_H * g),
            focus_glow: k.focus_glow.unwrap_or(l.focus_glow * g),
            hair,
            breakpoints,
            g,
        }
    }

    /// `n` old units in these units: what a widget's inner layout writes where no token names the number.
    pub fn u(&self, n: i32) -> i32 {
        n * self.g
    }

    /// `u` for a position or a size that is not whole.
    pub fn uf(&self, n: f32) -> f32 {
        n * self.g as f32
    }
}

/// How a nine-slice part fills its span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NineFill {
    Stretch,
    Tile,
    /// The centre only: not drawn (the role's fill colour shows instead).
    None,
}

impl NineFill {
    pub fn name(self) -> &'static str {
        match self {
            NineFill::Stretch => "stretch",
            NineFill::Tile => "tile",
            NineFill::None => "none",
        }
    }
}

/// A frame's sidecar (LOOK1 3.5): the slices in art px, art px per logical px, how edges and centre fill, the
/// content inset in logical px and the role of the text drawn on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Nine {
    pub slice: [i32; 4],
    pub scale: i32,
    pub edges: NineFill,
    pub center: NineFill,
    pub pad: [i32; 4],
    pub ink: String,
    /// Sidecar `"reduce": "average"`: a high-resolution frame (`scale` above the view's) is averaged down to the window's
    /// physical size instead of sampled (`Painter::nine`). Off for the pixel frames, which keep their exact pixels. A
    /// `straddle` frame always averages when it reduces, with or without this key.
    pub reduce: bool,
    /// Sidecar `"corner_inner": [x, y]`: art px from the anchor that the corner's art reaches into the panel;
    /// `painter::nine_content_rect` keeps a panel's content past them at whatever size the corners draw.
    pub corner_inner: Option<[i32; 2]>,
    /// Sidecar `"straddle": true`: the frame sits half in and half out of the panel. The edge bands and corners are
    /// centred on its boundary and corner points, the centre fills exactly the panel and the rest of the file's corners
    /// is see-through. The panel's rect, layout, widget and hit area stay as they are; only the drawing overhangs. Its
    /// cells are always averaged when reduced, whatever `reduce` says. Its corners follow the window at `scale` and keep
    /// one size on any panel; a panel too small for two draws them as large as fits (`Painter::nine`).
    pub straddle: bool,
    /// Sidecar `"sharpen"`: the strength of an unsharp mask on the corners once reduced (0 off), which holds thin lines
    /// at the window's size where a mask on the file would have to guess it.
    pub sharpen: f32,
    /// Sidecar `"corner_anchor": [x, y]`, with `straddle`: the art px of the top-left corner that sits on the panel's
    /// corner point (mirrored for the others), so the boundary runs along the art's own arms, whatever its image's
    /// empty space. Without it the corner's middle.
    pub corner_anchor: Option<[i32; 2]>,
    /// Sidecar `"corner_size"`: the corners' drawn size against `scale` (1 as the art says, `CORNER_SIZE` the limits).
    /// Inside the rect it takes the edge bands with it, as they share the corners' rows and columns.
    pub corner_size: f32,
    /// Sidecar `"tip": [from, half]`, with `straddle`: the head at each arm's end of the top-left corner: from art px
    /// `from` to the slice's end, `half` art px either side of the anchor (mirrored for the other corners).
    pub tip: Option<[i32; 2]>,
    /// Sidecar `"tip_size"`, with `tip`: the head's size against the art, from its base, the arm staying put.
    pub tip_size: f32,
    /// Sidecar `"line_width"`, with `straddle` and stretched edges: the edge drawn as a solid line this many logical px
    /// wide (`LINE_WIDTH` the limits) instead of the art's band; without it the art's own width.
    pub line_width: Option<f32>,
    /// Sidecar `"line_color"`, as `line_width`: that line's colour role or `#rrggbb`; without it the art's own colour.
    pub line_color: Option<LineColour>,
}

/// A frame line's colour: what the sidecar says (a colour role or a hex colour) and the colour that is, a role's taken
/// from the finished theme (`load`), so a cache keyed on the `Nine` tells two colours apart.
#[derive(Debug, Clone, PartialEq)]
pub struct LineColour {
    pub name: String,
    pub rgba: Rgba,
}

/// The limits of the sidecar's `corner_size` and `tip_size`.
pub const CORNER_SIZE: (f32, f32) = (0.25, 3.0);
/// The limits of the sidecar's `line_width`, in logical px.
pub const LINE_WIDTH: (f32, f32) = (0.5, 8.0);

impl Nine {
    /// Slices `slice` at scale 2, stretched, padded by the slices, text in `text`.
    pub fn new(slice: [i32; 4]) -> Self {
        Self { slice, scale: 2, edges: NineFill::Stretch, center: NineFill::Stretch, pad: slice.map(|s| (s + 1) / 2), ink: "text".into(), reduce: false, corner_inner: None, straddle: false, sharpen: 0.0, corner_anchor: None, corner_size: 1.0, tip: None, tip_size: 1.0, line_width: None, line_color: None }
    }

    /// The sidecar `v` of a `w` x `h` PNG, or every problem with it (`at` names the file).
    pub fn parse(v: &Value, w: i32, h: i32, at: &str) -> Result<Nine, Vec<String>> {
        let mut errs = Vec::new();
        let Some(o) = v.as_object() else { return Err(vec![format!("{at}: a sidecar is a JSON object")]) };
        for k in o.keys().filter(|k| !["slice", "scale", "edges", "center", "pad", "ink", "reduce", "corner_inner", "straddle", "sharpen", "corner_anchor", "corner_size", "tip", "tip_size", "line_width", "line_color"].contains(&k.as_str())) {
            errs.push(format!("{at}: unknown key '{k}' (allowed: slice, scale, edges, center, pad, ink, reduce, corner_inner, straddle, sharpen, corner_anchor, corner_size, tip, tip_size, line_width, line_color)"));
        }
        let four = |key: &str, lo: i64, hi: i64, errs: &mut Vec<String>| -> Option<[i32; 4]> {
            let a = o.get(key)?;
            let vals: Vec<i64> = a.as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
            if vals.len() != 4 || a.as_array().is_none_or(|a| a.len() != 4) {
                errs.push(format!("{at}: {key} must be [left, top, right, bottom] whole numbers"));
                return None;
            }
            if let Some(bad) = vals.iter().find(|n| **n < lo || **n > hi) {
                errs.push(format!("{at}: {key} value {bad} is outside {lo}-{hi}"));
                return None;
            }
            Some([vals[0] as i32, vals[1] as i32, vals[2] as i32, vals[3] as i32])
        };
        let slice = match o.get("slice") {
            None => {
                errs.push(format!("{at}: slice is required"));
                None
            }
            // A slice is below the PNG's size when valid, so this bound keeps every sum and rounding on it in range.
            Some(_) => four("slice", 1, w.max(h).max(1) as i64, &mut errs),
        };
        if let Some([l, t, r, b]) = slice {
            if l + r >= w {
                errs.push(format!("{at}: slice left + right ({}) must be below the PNG width {w}", l + r));
            }
            if t + b >= h {
                errs.push(format!("{at}: slice top + bottom ({}) must be below the PNG height {h}", t + b));
            }
        }
        let mut nine = Nine::new(slice.unwrap_or([1; 4]));
        if let Some(s) = o.get("scale") {
            match s.as_i64().filter(|n| (1..=32).contains(n)) {
                Some(n) => nine.scale = n as i32,
                None => errs.push(format!("{at}: scale must be a whole number 1-32")),
            }
        }
        if let Some(m) = o.get("reduce") {
            match m.as_str() {
                Some("average") => nine.reduce = true,
                _ => errs.push(format!("{at}: reduce must be average")),
            }
        }
        if let Some(m) = o.get("straddle") {
            match m.as_bool() {
                Some(b) => nine.straddle = b,
                None => errs.push(format!("{at}: straddle must be true or false")),
            }
        }
        if let Some(m) = o.get("sharpen") {
            match m.as_f64().filter(|n| (0.0..=2.0).contains(n)) {
                Some(n) => nine.sharpen = n as f32,
                None => errs.push(format!("{at}: sharpen must be a number 0-2")),
            }
        }
        if let Some(m) = o.get("corner_inner") {
            let vals: Vec<i64> = m.as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
            match vals[..] {
                [x, y] if x >= 0 && y >= 0 && x < nine.slice[0] as i64 && y < nine.slice[1] as i64 => nine.corner_inner = Some([x as i32, y as i32]),
                _ => errs.push(format!("{at}: corner_inner must be [x, y] art px inside the top-left slice")),
            }
        }
        if let Some(m) = o.get("corner_anchor") {
            let vals: Vec<i64> = m.as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
            match vals[..] {
                [x, y] if x >= 0 && y >= 0 && x < nine.slice[0] as i64 && y < nine.slice[1] as i64 => nine.corner_anchor = Some([x as i32, y as i32]),
                _ => errs.push(format!("{at}: corner_anchor must be [x, y] inside the top-left slice")),
            }
        }
        let fill = |key: &str, allowed: &[NineFill], errs: &mut Vec<String>| -> Option<NineFill> {
            let s = o.get(key)?.as_str().unwrap_or("");
            let hit = allowed.iter().find(|f| f.name() == s).copied();
            if hit.is_none() {
                errs.push(format!("{at}: {key} must be {}", allowed.iter().map(|f| f.name()).collect::<Vec<_>>().join(" or ")));
            }
            hit
        };
        if let Some(f) = fill("edges", &[NineFill::Stretch, NineFill::Tile], &mut errs) {
            nine.edges = f;
        }
        if let Some(f) = fill("center", &[NineFill::Stretch, NineFill::Tile, NineFill::None], &mut errs) {
            nine.center = f;
        }
        let k = nine.scale;
        nine.pad = nine.slice.map(|s| (s + k - 1) / k);
        if let Some(p) = four("pad", 0, 32, &mut errs) {
            nine.pad = p;
        }
        if let Some(ink) = o.get("ink") {
            match ink.as_str().filter(|s| ROLES.contains(s)) {
                Some(s) => nine.ink = s.into(),
                None => errs.push(format!("{at}: unknown ink '{}' (a colour role: {})", ink.as_str().unwrap_or(&ink.to_string()), ROLES.join(", "))),
            }
        }
        // The tuner's keys: each stays inside its limits, and says when the frame cannot draw it.
        let size = |key: &str, errs: &mut Vec<String>| -> Option<f32> {
            let m = o.get(key)?;
            let n = m.as_f64().filter(|n| (CORNER_SIZE.0 as f64..=CORNER_SIZE.1 as f64).contains(n)).map(|n| n as f32);
            if n.is_none() {
                errs.push(format!("{at}: {key} must be a number {}-{}", CORNER_SIZE.0, CORNER_SIZE.1));
            }
            n
        };
        if let Some(n) = size("corner_size", &mut errs) {
            nine.corner_size = n;
        }
        if let Some(n) = size("tip_size", &mut errs) {
            nine.tip_size = n;
        }
        if let Some(m) = o.get("tip") {
            let vals: Vec<i64> = m.as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
            let room = nine.slice.into_iter().min().unwrap_or(1) as i64;
            match vals[..] {
                [from, half] if from >= 1 && from < room && half >= 1 && half <= room => nine.tip = Some([from as i32, half as i32]),
                _ => errs.push(format!("{at}: tip must be [from, half] art px inside the slices")),
            }
        }
        if nine.tip.is_some() && !nine.straddle {
            errs.push(format!("{at}: tip needs straddle"));
        }
        if o.contains_key("tip_size") && !o.contains_key("tip") {
            errs.push(format!("{at}: tip_size needs tip"));
        }
        if let Some(m) = o.get("line_width") {
            match m.as_f64().filter(|n| (LINE_WIDTH.0 as f64..=LINE_WIDTH.1 as f64).contains(n)) {
                Some(n) => nine.line_width = Some(n as f32),
                None => errs.push(format!("{at}: line_width must be a number {}-{}", LINE_WIDTH.0, LINE_WIDTH.1)),
            }
        }
        if let Some(m) = o.get("line_color") {
            match m.as_str().and_then(line_colour) {
                Some(c) => nine.line_color = Some(c),
                None => errs.push(format!("{at}: line_color must be a colour role or #rrggbb")),
            }
        }
        if (o.contains_key("line_width") || o.contains_key("line_color")) && (!nine.straddle || nine.edges != NineFill::Stretch) {
            errs.push(format!("{at}: line_width and line_color need straddle and stretched edges"));
        }
        if errs.is_empty() {
            Ok(nine)
        } else {
            Err(errs)
        }
    }

    /// The content rect inside `r` (the pad).
    pub fn content(&self, r: crate::canvas::Rect) -> crate::canvas::Rect {
        let [l, t, rr, b] = self.pad;
        crate::canvas::Rect::new(r.x + l, r.y + t, (r.w - l - rr).max(0), (r.h - t - b).max(0))
    }
}

/// The sidecar's `line_color` value `s`: a hex colour or a role (coloured by `load`, or by the caller, from its theme).
pub fn line_colour(s: &str) -> Option<LineColour> {
    match parse_colour(s) {
        Some(rgba) => Some(LineColour { name: s.to_lowercase(), rgba }),
        None => ROLES.contains(&s).then(|| LineColour { name: s.into(), rgba: [0, 0, 0, 0] }),
    }
}

/// A frame file: its image, sidecar and where it came from (`ui/frames/panel.png (game)`).
#[derive(Debug, Clone)]
pub struct Frame {
    pub img: Arc<Image>,
    pub nine: Nine,
    pub source: String,
}

/// The frame roles and the role each falls back to (None: the vector panel in the role's colours).
pub const FRAME_ROLES: [(&str, Option<&str>); 15] = [
    ("panel", None),
    ("inset", None),
    ("parchment", Some("inset")),
    ("button", None),
    ("button_hot", Some("button")),
    ("button_down", Some("button_hot")),
    ("button_off", Some("button")),
    ("tab", Some("button")),
    ("tab_on", Some("button_down")),
    ("card", None),
    ("card_hot", Some("card")),
    ("card_on", Some("card_hot")),
    ("tip", Some("panel")),
    ("ornate", Some("panel")),
    ("ornate_full", Some("ornate")),
];

/// A hover card's vector corner radius, in old units.
const TIP_RADIUS: f32 = 3.0;

fn fallback(role: &str) -> Option<&'static str> {
    FRAME_ROLES.iter().find(|r| r.0 == role).and_then(|r| r.1)
}

impl Theme {
    /// A colour only game code reads (`game_colors`), else `fallback`.
    pub fn game(&self, name: &str, fallback: Rgba) -> Rgba {
        self.game_colors.get(name).copied().unwrap_or(fallback)
    }

    pub fn face(&self, face: Typeface) -> &FontSpec {
        &self.faces[face.index()]
    }

    /// The frame that draws `role`: its own file, else the first file along its fallbacks; None draws the vector
    /// panel (also for every role while `frames_on` is false).
    pub fn frame(&self, role: &str) -> Option<&Frame> {
        if !self.frames_on {
            return None;
        }
        let mut at = Some(role);
        while let Some(r) = at {
            if let Some(f) = self.frames.get(r) {
                return Some(f);
            }
            at = fallback(r);
        }
        None
    }

    /// What draws `role`: the frame file's source, or "vector".
    pub fn frame_source(&self, role: &str) -> String {
        self.frame(role).map_or_else(|| "vector".into(), |f| f.source.clone())
    }

    /// The vector panel of a frame role (no frame file): fill, edge and corner radius, as the widgets drew it before.
    pub fn vector(&self, role: &str) -> (Rgba, Rgba, f32) {
        let r = self.layout.radius;
        match role {
            "panel" | "ornate" | "ornate_full" => (self.panel, self.panel_edge, self.layout.frame_radius),
            "inset" => (self.card, self.edge, r),
            "parchment" => (self.parchment, self.edge, r),
            "button" | "tab" => (self.card, self.edge_live, r),
            "button_hot" => (self.card_hot, self.focus, r),
            "button_down" | "tab_on" => (self.selected, self.focus, r),
            "button_off" => (self.card, self.edge, r),
            "card" => (self.card, self.edge, r),
            "card_hot" => (self.card_hot, self.edge_hot, r),
            "card_on" => (self.selected, self.focus, r),
            "tip" => (self.tip, self.edge_hot, TIP_RADIUS),
            _ => (self.card, self.edge, r),
        }
    }

    /// `vector` on the grid of `m`: the same colours, the corner radius in its units.
    pub fn vector_in(&self, role: &str, m: &Metrics) -> (Rgba, Rgba, f32) {
        let (fill, edge, _) = self.vector(role);
        let radius = match role {
            "panel" | "ornate" | "ornate_full" => m.frame_radius,
            "tip" => m.uf(TIP_RADIUS),
            _ => m.radius,
        };
        (fill, edge, radius)
    }

    /// The text colour on frame role `role`: its frame's `ink`, else `ink` on vector parchment and `text` elsewhere.
    pub fn ink_on(&self, role: &str) -> Rgba {
        match self.frame(role) {
            Some(f) => self.role(&f.nine.ink).unwrap_or(self.text),
            None if role == "parchment" => self.ink,
            None => self.text,
        }
    }

    /// `contrast` with a translucent colour composited over this theme's `bg` first.
    pub fn contrast(&self, fg: Rgba, bg: Rgba) -> f32 {
        let back = over(bg, self.bg);
        ratio(over(fg, back), back)
    }
}

/// `c` composited over the opaque `under`.
fn over(c: Rgba, under: Rgba) -> Rgba {
    let a = c[3] as f32 / 255.0;
    let m = |i: usize| (c[i] as f32 * a + under[i] as f32 * (1.0 - a)).round() as u8;
    [m(0), m(1), m(2), 255]
}

/// WCAG 2 relative luminance of an opaque colour.
fn luminance(c: Rgba) -> f32 {
    let lin = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

fn ratio(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The WCAG 2 contrast ratio of `fg` on `bg` (1 to 21); a translucent fg or bg is composited over the current
/// theme's `bg` first.
pub fn contrast(fg: Rgba, bg: Rgba) -> f32 {
    th().contrast(fg, bg)
}

/// The contrast pairs LOOK1 8.2 checks: (text role, ground role, the least ratio): text at 4.5:1, `dim` at 3:1, a check
/// mark (not text) at 3:1. `ink` is checked on `parchment`. The later grounds are the widgets' own (LOOK1-G2), each with
/// the roles the widget draws on it; a theme file is held to all of them, the code defaults show their ratios only.
pub fn contrast_pairs() -> Vec<(&'static str, &'static str, f32)> {
    let mut out = Vec::new();
    for ground in ["panel", "card", "selected"] {
        for (fg, least) in [("text", 4.5), ("gold", 4.5), ("info", 4.5), ("dim", 3.0)] {
            out.push((fg, ground, least));
        }
    }
    out.push(("ink", "parchment", 4.5));
    for (ground, fgs) in [
        ("card_hot", &["text", "gold", "dim"][..]),
        ("input", &["text", "dim"]),
        ("field", &["text"]),
        ("section", &["text", "dim"]),
        ("selection", &["text"]),
        ("tip", &["text", "gold", "dim"]),
        ("ghost", &["text"]),
        ("title_idle", &["text", "dim"]),
        ("ruler", &["text", "dim"]),
        ("playing", &["focus"]),
        ("tool_bg", &["text", "dim"]),
        ("tool_pane", &["text", "gold", "dim"]),
        ("stage", &["text", "gold", "dim"]),
    ] {
        for fg in fgs {
            out.push((*fg, ground, if *fg == "dim" { 3.0 } else { 4.5 }));
        }
    }
    out.push(("text", "check_on", 3.0));
    out
}

/// `#rrggbb` or `#rrggbbaa` (either case).
pub fn parse_colour(s: &str) -> Option<Rgba> {
    let hex = s.strip_prefix('#')?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let ch = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([ch(0)?, ch(2)?, ch(4)?, if hex.len() == 8 { ch(6)? } else { 255 }])
}

/// A colour as `#rrggbb`, or `#rrggbbaa` when translucent.
pub fn colour_hex(c: Rgba) -> String {
    if c[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
    }
}

fn game_colour_name(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=32).contains(&b.len()) && b[0].is_ascii_lowercase() && b[1..].iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

/// The names of `n` asset roots in lookup order (game, kit, engine; `Layers::for_game`).
pub fn layer_names(n: usize) -> Vec<String> {
    match n {
        1 => vec!["game".into()],
        2 => vec!["game".into(), "engine".into()],
        3 => vec!["game".into(), "kit".into(), "engine".into()],
        _ => (0..n).map(|i| format!("layer {}", i + 1)).collect(),
    }
}

/// The theme of `layers`: `Theme::default()` with each layer's `ui/theme.json` applied bottom-up (engine, kit,
/// game) and the frames of `ui/frames/` (the game's over the kit's over the engine's); every problem found, or the
/// theme when there is none.
pub fn load(layers: &Layers) -> Result<Theme, Vec<String>> {
    let mut t = Theme::default();
    let mut errs = Vec::new();
    let names = layer_names(layers.roots.len());
    let mut face_set = [false; 3];
    for (i, root) in layers.roots.iter().enumerate().rev() {
        let file = root.join(FILE);
        if file.is_file() {
            let at = format!("{FILE} ({})", names[i]);
            apply_file(&mut t, &file, &at, layers, &mut face_set, &mut errs);
            t.sources.push(at);
        }
    }
    // A face no file names is the body face.
    for face in [Typeface::Display, Typeface::Mono] {
        if !face_set[face.index()] {
            t.faces[face.index()] = t.faces[Typeface::Body.index()].clone();
        }
    }
    for (i, root) in layers.roots.iter().enumerate().rev() {
        load_frames(&mut t, &root.join(FRAMES_DIR), &names[i], &mut errs);
    }
    // A line colour's role, from the finished colours.
    let roles: Vec<Option<Rgba>> = t.frames.values().map(|f| f.nine.line_color.as_ref().and_then(|c| t.role(&c.name))).collect();
    for (f, rgba) in t.frames.values_mut().zip(roles) {
        if let (Some(c), Some(rgba)) = (f.nine.line_color.as_mut(), rgba) {
            c.rgba = rgba;
        }
    }
    if errs.is_empty() {
        Ok(t)
    } else {
        Err(errs)
    }
}

fn apply_file(t: &mut Theme, file: &Path, at: &str, layers: &Layers, face_set: &mut [bool; 3], errs: &mut Vec<String>) {
    let v: Value = match std::fs::read_to_string(file).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string())) {
        Ok(v) => v,
        Err(e) => return errs.push(format!("{at}: {e}")),
    };
    let Some(o) = v.as_object() else { return errs.push(format!("{at}: the theme is a JSON object")) };
    if o.get("version").and_then(Value::as_i64) != Some(1) {
        errs.push(format!("{at}: version must be 1"));
    }
    for k in o.keys().filter(|k| !TOP_KEYS.contains(&k.as_str())) {
        errs.push(format!("{at}: unknown key '{k}' (allowed: {})", TOP_KEYS.join(", ")));
    }
    let object = |key: &str, errs: &mut Vec<String>| -> Vec<(String, Value)> {
        match o.get(key) {
            None => Vec::new(),
            Some(Value::Object(m)) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            Some(_) => {
                errs.push(format!("{at}: {key} must be an object"));
                Vec::new()
            }
        }
    };
    let colour = |key: &str, v: &Value, errs: &mut Vec<String>| -> Option<Rgba> {
        let c = v.as_str().and_then(parse_colour);
        if c.is_none() {
            errs.push(format!("{at}: {key} '{}' is not a colour (#rrggbb or #rrggbbaa)", v.as_str().map_or_else(|| v.to_string(), String::from)));
        }
        c
    };
    for (k, v) in object("colors", errs) {
        if t.role(&k).is_none() {
            errs.push(format!("{at}: unknown colour role '{k}' (allowed: {})", ROLES.join(", ")));
            continue;
        }
        if let Some(c) = colour(&format!("colors.{k}"), &v, errs) {
            *t.role_mut(&k).unwrap() = c;
        }
    }
    for (k, v) in object("game_colors", errs) {
        if !game_colour_name(&k) {
            errs.push(format!("{at}: game colour name '{k}' must be [a-z][a-z0-9_]{{1,31}}"));
            continue;
        }
        if let Some(c) = colour(&format!("game_colors.{k}"), &v, errs) {
            t.game_colors.insert(k, c);
        }
    }
    for (k, v) in object("fonts", errs) {
        let Some(face) = Typeface::ALL.into_iter().find(|f| f.name() == k) else {
            errs.push(format!("{at}: unknown font face '{k}' (allowed: body, display, mono)"));
            continue;
        };
        let Some(spec) = v.as_object() else {
            errs.push(format!("{at}: fonts.{k} must be {{\"file\": \"fonts/<name>.ttf\", \"scale\": 1.0}}"));
            continue;
        };
        let mut f = t.faces[face.index()].clone();
        for bad in spec.keys().filter(|x| *x != "file" && *x != "scale") {
            errs.push(format!("{at}: fonts.{k}: unknown key '{bad}' (allowed: file, scale)"));
        }
        if let Some(s) = spec.get("scale") {
            match s.as_f64().filter(|s| (0.5..=2.0).contains(s)) {
                Some(s) => f.scale = s as f32,
                None => errs.push(format!("{at}: fonts.{k}.scale must be a number 0.5-2.0")),
            }
        }
        if let Some(file) = spec.get("file") {
            let rel = file.as_str().unwrap_or("");
            match layers.find(rel).filter(|_| !rel.is_empty()) {
                None => errs.push(format!("{at}: fonts.{k}.file '{rel}' is in no asset layer")),
                Some(path) => match std::fs::read(&path) {
                    Ok(bytes) if fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings::default()).is_ok() => {
                        f.file = rel.into();
                        f.bytes = Some(Arc::new(bytes));
                    }
                    Ok(_) => errs.push(format!("{at}: fonts.{k}.file '{rel}' is not a font")),
                    Err(e) => errs.push(format!("{at}: fonts.{k}.file '{rel}' can't be read: {e}")),
                },
            }
        }
        t.faces[face.index()] = f;
        face_set[face.index()] = true;
    }
    for (k, v) in object("layout", errs) {
        let Some(&(_, lo, hi, whole)) = LAYOUT_KEYS.iter().find(|l| l.0 == k) else {
            errs.push(format!("{at}: unknown layout key '{k}' (allowed: {})", LAYOUT_KEYS.map(|l| l.0).join(", ")));
            continue;
        };
        let n = v.as_f64().map(|n| n as f32).filter(|n| !whole || n.fract() == 0.0);
        match n.filter(|n| (lo..=hi).contains(n)) {
            Some(n) => t.layout.set(&k, n),
            None => errs.push(format!("{at}: layout.{k} must be {} {lo}-{hi}", if whole { "a whole number" } else { "a number" })),
        }
    }
    for (k, v) in object("metrics", errs) {
        if k == "breakpoints" {
            apply_breakpoints(&mut t.metrics, &v, at, errs);
            continue;
        }
        let Some(&(_, lo, hi, whole)) = METRIC_KEYS.iter().find(|l| l.0 == k) else {
            errs.push(format!("{at}: unknown metrics key '{k}' (allowed: {}, breakpoints)", METRIC_KEYS.map(|l| l.0).join(", ")));
            continue;
        };
        let n = v.as_f64().map(|n| n as f32).filter(|n| !whole || n.fract() == 0.0);
        match n.filter(|n| (lo..=hi).contains(n)) {
            Some(n) => t.metrics.set(&k, n),
            None => errs.push(format!("{at}: metrics.{k} must be {} {lo}-{hi}", if whole { "a whole number" } else { "a number" })),
        }
    }
    if let Some(f) = o.get("frames") {
        match f.as_bool() {
            Some(b) => t.frames_on = b,
            None => errs.push(format!("{at}: frames must be true or false")),
        }
    }
}

/// `metrics.breakpoints`: `width` and `height`, each `[compact, medium]` ascending whole numbers. A file sets only the axes
/// it names.
fn apply_breakpoints(keys: &mut MetricKeys, v: &Value, at: &str, errs: &mut Vec<String>) {
    let Some(o) = v.as_object() else { return errs.push(format!("{at}: metrics.breakpoints must be an object")) };
    for (k, axis) in o {
        let slot = match k.as_str() {
            "width" => &mut keys.breakpoints_width,
            "height" => &mut keys.breakpoints_height,
            _ => {
                errs.push(format!("{at}: unknown metrics.breakpoints key '{k}' (allowed: width, height)"));
                continue;
            }
        };
        let vals: Option<Vec<i64>> = axis.as_array().and_then(|a| a.iter().map(Value::as_i64).collect());
        match vals.as_deref() {
            Some(&[a, b]) if a >= 1 && a < b && b <= BREAKPOINT_MAX => *slot = Some([a as i32, b as i32]),
            _ => errs.push(format!("{at}: metrics.breakpoints.{k} must be [compact, medium], whole numbers ascending, 1-{BREAKPOINT_MAX}")),
        }
    }
}

/// The frame files of one layer's `ui/frames/` (each `<role>.png` with its `<role>.json`).
fn load_frames(t: &mut Theme, dir: &Path, layer: &str, errs: &mut Vec<String>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<String> = read.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    files.sort();
    let roles: Vec<&str> = FRAME_ROLES.iter().map(|r| r.0).collect();
    let mut stems: Vec<String> = Vec::new();
    for f in &files {
        let at = format!("{FRAMES_DIR}/{f} ({layer})");
        let (stem, ext) = f.rsplit_once('.').unwrap_or((f.as_str(), ""));
        if ext != "png" && ext != "json" {
            errs.push(format!("{at}: not a frame file (<role>.png with <role>.json)"));
        } else if !roles.contains(&stem) {
            errs.push(format!("{at}: unknown frame role '{stem}' (allowed: {})", roles.join(", ")));
        } else if !stems.iter().any(|s| s == stem) {
            stems.push(stem.into());
        }
    }
    for stem in stems {
        let (png, side) = (format!("{stem}.png"), format!("{stem}.json"));
        let (has_png, has_side) = (files.contains(&png), files.contains(&side));
        if !has_side {
            errs.push(format!("{FRAMES_DIR}/{png} ({layer}): no sidecar {side} beside it"));
            continue;
        }
        if !has_png {
            errs.push(format!("{FRAMES_DIR}/{side} ({layer}): no {png} beside it"));
            continue;
        }
        let Some(img) = crate::art::load_png(&dir.join(&png)) else {
            errs.push(format!("{FRAMES_DIR}/{png} ({layer}): not a readable RGBA, RGB or grey PNG"));
            continue;
        };
        let at = format!("{FRAMES_DIR}/{side} ({layer})");
        let v: Value = match std::fs::read_to_string(dir.join(&side)).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string())) {
            Ok(v) => v,
            Err(e) => {
                errs.push(format!("{at}: {e}"));
                continue;
            }
        };
        match Nine::parse(&v, img.w, img.h, &at) {
            Ok(nine) => {
                t.frames.insert(stem.clone(), Frame { img: Arc::new(img), nine, source: format!("{FRAMES_DIR}/{png} ({layer})") });
            }
            Err(e) => errs.extend(e),
        }
    }
}

thread_local! {
    /// The theme in force on this thread, and the one the asset layers gave (the UI Gallery's switch goes between it,
    /// the engine default and it with frames off).
    static CURRENT: RefCell<Rc<Theme>> = RefCell::new(Rc::new(Theme::default()));
    static LOADED: RefCell<Rc<Theme>> = RefCell::new(Rc::new(Theme::default()));
    /// The asset layers of the last `reload` (LOOK1-E2: the Kit page's Tiles page reads `tilesets/` through them, as
    /// the page draws with no `Cx`).
    static LAYERS: RefCell<Rc<Layers>> = RefCell::new(Rc::new(Layers::default()));
}

/// The asset layers the last `reload` read (none before it). Each `reload` gives a new handle, so a cache of what they
/// hold can tell (`Rc::ptr_eq`).
pub fn layers() -> Rc<Layers> {
    LAYERS.with(|c| c.borrow().clone())
}

/// Sets what `layers` answers (`reload` does; tests with fixture layers).
pub fn set_layers(layers: Layers) {
    LAYERS.with(|c| *c.borrow_mut() = Rc::new(layers));
}

/// The theme in force (cheap: a shared handle).
pub fn th() -> Rc<Theme> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Puts `t` in force (until the next `set_theme` or `install`).
pub fn set_theme(t: Theme) {
    CURRENT.with(|c| *c.borrow_mut() = Rc::new(t));
}

/// The theme the asset layers gave (`install`), the default before any.
pub fn loaded() -> Rc<Theme> {
    LOADED.with(|c| c.borrow().clone())
}

/// Puts `t` in force as the layers' theme.
pub fn install(t: Theme) {
    let t = Rc::new(t);
    LOADED.with(|c| *c.borrow_mut() = t.clone());
    CURRENT.with(|c| *c.borrow_mut() = t);
}

/// Loads the theme again from the layers of the last `reload` and installs it; on a problem the theme in force stays and
/// the problems come back (a tool that edits a theme file, with no `Cx` to show them on).
pub fn reinstall() -> Result<(), Vec<String>> {
    load(&layers()).map(install)
}

/// Loads the theme from the game's asset layers (`cx.art`) and installs it; on any problem the theme in force stays
/// and the problems are shown (`show_errors`). The shell calls it at start; a game calls it on its Reload data.
pub fn reload<G: 'static>(cx: &mut Cx<G>) -> bool {
    set_layers(cx.art.layers().clone());
    match load(cx.art.layers()) {
        Ok(t) => {
            install(t);
            true
        }
        Err(errs) => {
            show_errors(cx, errs);
            false
        }
    }
}
