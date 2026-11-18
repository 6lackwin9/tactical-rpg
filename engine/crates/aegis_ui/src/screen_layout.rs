//! UISPLIT1 (`docs/specs/UISPLIT1.md`, row PR7-38): a screen's behaviour apart from its layout and look.
//!
//! - A **view** (US1) is what a screen shows and offers, with no rects or colours: named fields (`Field`) and actions
//!   (`ActionView`). A game's screen builds it from its state; the agent link and the tests read the same thing. A
//!   fixture of it (`ui/views/<screen>.view.json`, US6) lets a layout be drawn with no game running.
//! - A **layout** (US2) is `ui/layouts/<screen>.json` over the asset layers (the first layer holding the file wins: game,
//!   kit, engine): regions on the screen, each placed by an anchor and a size in grid tokens (with overrides by text
//!   size), each holding one component of the fixed set (US3, `Component`) bound to view fields and actions. Look stays
//!   in the theme: a layout names no colour.
//! - A screen with no layout file, or one whose layout does not fit its view, draws the **default** (US4, US5): the
//!   fields top to bottom in a panel shell, the actions in a footer button bar; the problems are kept for the shell's
//!   notice (`Layouts::take_problems`) and the gate's check (`check_game`, `ui_layout check`).
//!
//! A click gives the screen an `Act` (the action's id, and the row, tab or slot id it was for); the screen runs it. A
//! `custom` region hands its rect back (`Drawn::custom`) for code-drawn stages (a board, a map).

use crate::art::Art;
use crate::canvas::Rect;
use crate::ui::{Ui, Widget};
use aegis_core::assets::Layers;
use aegis_core::vfs;
use aegis_layout::{fill, ratio, Anchor, Insets, Part};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where layout files and view fixtures lie, under each asset layer.
pub const LAYOUTS_DIR: &str = "ui/layouts";
pub const VIEWS_DIR: &str = "ui/views";
/// A fixture's name: `<screen>.view.json`.
pub const VIEW_SUFFIX: &str = ".view.json";

// ---- the view ----

/// A row of a list (`list_row`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RowView {
    pub id: String,
    pub label: String,
    /// A second text at the row's right end ("" none).
    pub detail: String,
    /// A picture at its left (an asset path), if any.
    pub picture: Option<String>,
    pub enabled: bool,
}

/// A save or load card (`slot_card`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SlotData {
    pub id: String,
    pub title: String,
    pub lines: Vec<String>,
    pub stamp: String,
    pub picture: Option<String>,
}

/// One field of a view. In JSON, an object with one key naming the kind: `{"text": "Gold 120"}`, `{"number": 3}`,
/// `{"flag": true}`, `{"picture": "ui/x.png"}`, `{"meter": [value, max]}`, `{"rows": [{"id", "label", "detail",
/// "picture", "enabled"}]}`, `{"stats": [["Strength", "12"]]}`, `{"tabs": {"labels": [...], "open": 0}}`, `{"slots":
/// [{"id", "title", "lines", "stamp", "picture"}]}`, `{"icons": ["path", ...]}`.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Text(String),
    Number(f64),
    Flag(bool),
    Picture(String),
    Meter { value: f64, max: f64 },
    Rows(Vec<RowView>),
    Stats(Vec<(String, String)>),
    Tabs { labels: Vec<String>, open: usize },
    Slots(Vec<SlotData>),
    Icons(Vec<String>),
}

impl Field {
    pub fn kind(&self) -> &'static str {
        match self {
            Field::Text(_) => "text",
            Field::Number(_) => "number",
            Field::Flag(_) => "flag",
            Field::Picture(_) => "picture",
            Field::Meter { .. } => "meter",
            Field::Rows(_) => "rows",
            Field::Stats(_) => "stats",
            Field::Tabs { .. } => "tabs",
            Field::Slots(_) => "slots",
            Field::Icons(_) => "icons",
        }
    }

    /// As text, for a text block: a text as is, a number without a needless ".0", a flag as Yes or No.
    pub fn as_text(&self) -> Option<String> {
        match self {
            Field::Text(s) => Some(s.clone()),
            Field::Number(n) => Some(number_text(*n)),
            Field::Flag(b) => Some(if *b { "Yes" } else { "No" }.to_string()),
            _ => None,
        }
    }

    pub fn to_json(&self) -> Value {
        let pic = |p: &Option<String>| p.clone().map_or(Value::Null, Value::String);
        match self {
            Field::Text(s) => json!({"text": s}),
            Field::Number(n) => json!({"number": n}),
            Field::Flag(b) => json!({"flag": b}),
            Field::Picture(p) => json!({"picture": p}),
            Field::Meter { value, max } => json!({"meter": [value, max]}),
            Field::Rows(rows) => json!({"rows": rows.iter().map(|r| json!({"id": r.id, "label": r.label, "detail": r.detail, "picture": pic(&r.picture), "enabled": r.enabled})).collect::<Vec<_>>()}),
            Field::Stats(s) => json!({"stats": s.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>()}),
            Field::Tabs { labels, open } => json!({"tabs": {"labels": labels, "open": open}}),
            Field::Slots(s) => json!({"slots": s.iter().map(|c| json!({"id": c.id, "title": c.title, "lines": c.lines, "stamp": c.stamp, "picture": pic(&c.picture)})).collect::<Vec<_>>()}),
            Field::Icons(i) => json!({"icons": i}),
        }
    }

    /// The field a JSON value states, or why it states none.
    pub fn from_json(v: &Value) -> Result<Field, String> {
        let Some((kind, x)) = v.as_object().filter(|o| o.len() == 1).and_then(|o| o.iter().next()) else {
            return Err("a field is an object with one key naming its kind (text, number, flag, picture, meter, rows, stats, tabs, slots, icons)".into());
        };
        let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let pic = |v: &Value| v.get("picture").and_then(Value::as_str).map(str::to_string);
        let list = |what: &str| x.as_array().ok_or(format!("'{kind}' is a list of {what}"));
        Ok(match kind.as_str() {
            "text" => Field::Text(x.as_str().ok_or("'text' is a text")?.to_string()),
            "number" => Field::Number(x.as_f64().ok_or("'number' is a number")?),
            "flag" => Field::Flag(x.as_bool().ok_or("'flag' is true or false")?),
            "picture" => Field::Picture(x.as_str().ok_or("'picture' is an asset path")?.to_string()),
            "meter" => match x.as_array().map(|a| a.iter().map(Value::as_f64).collect::<Vec<_>>()).as_deref() {
                Some([Some(value), Some(max)]) if *max > 0.0 => Field::Meter { value: *value, max: *max },
                _ => return Err("'meter' is [value, max], max above 0".into()),
            },
            "rows" => Field::Rows(
                list("rows")?
                    .iter()
                    .map(|r| {
                        r.is_object().then(|| RowView { id: s(r, "id"), label: s(r, "label"), detail: s(r, "detail"), picture: pic(r), enabled: r.get("enabled").and_then(Value::as_bool).unwrap_or(true) }).ok_or("a row is an object")
                    })
                    .collect::<Result<_, _>>()?,
            ),
            "stats" => Field::Stats(
                list("[label, value] pairs")?
                    .iter()
                    .map(|p| match p.as_array().map(Vec::as_slice) {
                        Some([a, b]) => Ok((a.as_str().unwrap_or("").to_string(), b.as_str().map(str::to_string).unwrap_or_else(|| b.to_string()))),
                        _ => Err("a stat is [label, value]"),
                    })
                    .collect::<Result<_, _>>()?,
            ),
            "tabs" => {
                let labels: Vec<String> = x.get("labels").and_then(Value::as_array).ok_or("'tabs' has 'labels', a list of texts")?.iter().map(|l| l.as_str().unwrap_or("").to_string()).collect();
                let open = x.get("open").and_then(Value::as_u64).unwrap_or(0) as usize;
                if open >= labels.len().max(1) {
                    return Err("'tabs' 'open' is the place of one of its labels".into());
                }
                Field::Tabs { labels, open }
            }
            "slots" => Field::Slots(
                list("slots")?
                    .iter()
                    .map(|c| {
                        c.is_object()
                            .then(|| SlotData { id: s(c, "id"), title: s(c, "title"), lines: c.get("lines").and_then(Value::as_array).map(|l| l.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default(), stamp: s(c, "stamp"), picture: pic(c) })
                            .ok_or("a slot is an object")
                    })
                    .collect::<Result<_, _>>()?,
            ),
            "icons" => Field::Icons(list("asset paths")?.iter().map(|i| i.as_str().unwrap_or("").to_string()).collect()),
            other => return Err(format!("unknown field kind '{other}'")),
        })
    }
}

fn number_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// An action a screen offers.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionView {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

/// What a screen shows and offers (US1), in order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct View {
    pub screen: String,
    pub fields: Vec<(String, Field)>,
    pub actions: Vec<ActionView>,
}

impl View {
    pub fn new(screen: &str) -> Self {
        Self { screen: screen.to_string(), ..Self::default() }
    }
    pub fn field(mut self, name: &str, f: Field) -> Self {
        self.fields.push((name.to_string(), f));
        self
    }
    pub fn text(self, name: &str, t: impl Into<String>) -> Self {
        self.field(name, Field::Text(t.into()))
    }
    pub fn action(mut self, id: &str, label: &str, enabled: bool) -> Self {
        self.actions.push(ActionView { id: id.to_string(), label: label.to_string(), enabled });
        self
    }
    pub fn get(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, f)| f)
    }
    pub fn act(&self, id: &str) -> Option<&ActionView> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// `{"screen", "fields": {name: field}, "actions": [{"id", "label", "enabled"}]}`, the fixture's form.
    pub fn to_json(&self) -> Value {
        let fields: Map<String, Value> = self.fields.iter().map(|(n, f)| (n.clone(), f.to_json())).collect();
        json!({"screen": self.screen, "fields": fields, "actions": self.actions.iter().map(|a| json!({"id": a.id, "label": a.label, "enabled": a.enabled})).collect::<Vec<_>>()})
    }

    /// A view from its JSON, or every problem with it.
    pub fn from_json(v: &Value) -> Result<View, Vec<String>> {
        let mut problems = Vec::new();
        let Some(o) = v.as_object() else { return Err(vec!["a view is an object".into()]) };
        for k in o.keys().filter(|k| !["screen", "fields", "actions"].contains(&k.as_str())) {
            problems.push(format!("unknown key '{k}' (screen, fields, actions)"));
        }
        let screen = o.get("screen").and_then(Value::as_str).unwrap_or("").to_string();
        if !plain_id(&screen) {
            problems.push("'screen' is the screen's id: letters, digits and _".into());
        }
        let mut fields = Vec::new();
        match o.get("fields") {
            Some(Value::Object(f)) => {
                for (name, x) in f {
                    match Field::from_json(x) {
                        Ok(field) => fields.push((name.clone(), field)),
                        Err(e) => problems.push(format!("field '{name}': {e}")),
                    }
                }
            }
            None => {}
            Some(_) => problems.push("'fields' is an object of fields by name".into()),
        }
        let mut actions = Vec::new();
        for (i, a) in o.get("actions").and_then(Value::as_array).cloned().unwrap_or_default().iter().enumerate() {
            match a.get("id").and_then(Value::as_str).filter(|id| plain_id(id)) {
                Some(id) => actions.push(ActionView { id: id.to_string(), label: a.get("label").and_then(Value::as_str).unwrap_or(id).to_string(), enabled: a.get("enabled").and_then(Value::as_bool).unwrap_or(true) }),
                None => problems.push(format!("action {}: 'id' is letters, digits and _", i + 1)),
            }
        }
        if problems.is_empty() {
            Ok(View { screen, fields, actions })
        } else {
            Err(problems)
        }
    }
}

fn plain_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ---- the layout ----

/// A size along one side: grid tokens (a whole number: that many `layout.grid` units), a share of the parent
/// (`"40%"`), or all of it (`"fill"`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Tokens(i32),
    Percent(u16),
    Fill,
}

impl Size {
    fn parse(v: &Value) -> Result<Size, String> {
        if let Some(n) = v.as_i64() {
            return (1..=1000).contains(&n).then_some(Size::Tokens(n as i32)).ok_or("a size in tokens is a whole number from 1 to 1000".into());
        }
        match v.as_str() {
            Some("fill") => Ok(Size::Fill),
            Some(s) if s.ends_with('%') => s[..s.len() - 1].parse::<u16>().ok().filter(|p| (1..=100).contains(p)).map(Size::Percent).ok_or(format!("'{s}': a share is 1% to 100%")),
            _ => Err("a size is a whole number of grid tokens, a share like \"40%\" or \"fill\"".into()),
        }
    }
    fn part(self, grid: i32) -> Part {
        match self {
            Size::Tokens(n) => Part::from(n * grid),
            Size::Percent(p) => ratio(p, 100),
            Size::Fill => fill(1),
        }
    }
}

/// Where a region sits in its parent: the anchor, the size, an offset in tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub anchor: Anchor,
    pub w: Size,
    pub h: Size,
    pub dx: i32,
    pub dy: i32,
}

pub const ANCHORS: [(&str, Anchor); 9] = [
    ("top_left", Anchor::TopLeft),
    ("top", Anchor::Top),
    ("top_right", Anchor::TopRight),
    ("left", Anchor::Left),
    ("center", Anchor::Center),
    ("right", Anchor::Right),
    ("bottom_left", Anchor::BottomLeft),
    ("bottom", Anchor::Bottom),
    ("bottom_right", Anchor::BottomRight),
];

impl Place {
    /// The keys `anchor`, `w`, `h`, `x`, `y` of `o` over `base`.
    fn parse(o: &Map<String, Value>, base: Option<Place>, at: &str, problems: &mut Vec<String>) -> Place {
        let mut p = base.unwrap_or(Place { anchor: Anchor::TopLeft, w: Size::Fill, h: Size::Fill, dx: 0, dy: 0 });
        match o.get("anchor").map(|a| a.as_str().and_then(|a| ANCHORS.iter().find(|(n, _)| *n == a))) {
            None => {}
            Some(Some((_, a))) => p.anchor = *a,
            Some(None) => problems.push(format!("{at}: 'anchor' is one of {}", ANCHORS.iter().map(|a| a.0).collect::<Vec<_>>().join(", "))),
        }
        for (k, slot) in [("w", &mut p.w), ("h", &mut p.h)] {
            if let Some(v) = o.get(k) {
                match Size::parse(v) {
                    Ok(s) => *slot = s,
                    Err(e) => problems.push(format!("{at}: '{k}': {e}")),
                }
            }
        }
        for (k, slot) in [("x", &mut p.dx), ("y", &mut p.dy)] {
            if let Some(v) = o.get(k) {
                match v.as_i64().filter(|n| (-1000..=1000).contains(n)) {
                    Some(n) => *slot = n as i32,
                    None => problems.push(format!("{at}: '{k}' is an offset in tokens, -1000 to 1000")),
                }
            }
        }
        p
    }

    fn rect_in(&self, parent: Rect, grid: i32) -> Rect {
        parent.place(self.anchor, self.w.part(grid), self.h.part(grid)).offset(self.dx * grid, self.dy * grid)
    }
}

/// A panel shell's frame: the main screen's (ornate) or a popup's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Main,
    Popup,
}

impl Shell {
    fn role(self) -> &'static str {
        match self {
            Shell::Main => "ornate",
            Shell::Popup => "panel",
        }
    }
}

/// The fixed component set (US3). Each names the view fields (by kind) and actions it binds.
#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    /// A frame; regions `in` it are placed in its body.
    PanelShell { title: Option<String>, shell: Shell },
    TextBlock { text: String },
    StatBlock { stats: String },
    ListRow { rows: String, select: Option<String> },
    TabRail { tabs: String, select: String },
    SlotCard { slots: String, select: Option<String> },
    MenuColumn { actions: Vec<String> },
    Portrait { picture: String },
    IconGrid { icons: String, select: Option<String> },
    BarMeter { meter: String, label: Option<String> },
    ButtonBar { actions: Vec<String> },
    Tooltip { text: String },
    /// The rect goes back to the screen's code under this name.
    Custom { slot: String },
}

/// Each component: its name, the bind keys it needs, the ones it may take.
pub const COMPONENTS: [(&str, &[&str], &[&str]); 13] = [
    ("panel_shell", &[], &["title"]),
    ("text_block", &["text"], &[]),
    ("stat_block", &["stats"], &[]),
    ("list_row", &["rows"], &["select"]),
    ("tab_rail", &["tabs", "select"], &[]),
    ("slot_card", &["slots"], &["select"]),
    ("menu_column", &["actions"], &[]),
    ("portrait", &["picture"], &[]),
    ("icon_grid", &["icons"], &["select"]),
    ("bar_meter", &["meter"], &["label"]),
    ("button_bar", &["actions"], &[]),
    ("tooltip", &["text"], &[]),
    ("custom", &["slot"], &[]),
];

#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub id: String,
    /// The earlier region it is placed in (a panel shell's body, or any region's rect); None: the screen.
    pub within: Option<String>,
    pub place: Place,
    /// (text scale at least, the place then): the last that holds wins.
    pub when: Vec<(f32, Place)>,
    pub component: Component,
}

impl Region {
    /// Its place at text scale `scale`: the last `when` override whose `text_scale_min` it reaches, else its own.
    pub fn place_at(&self, scale: f32) -> Place {
        self.when.iter().rev().find(|(min, _)| scale >= *min).map_or(self.place, |(_, p)| *p)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub screen: String,
    pub regions: Vec<Region>,
}

fn bind_text(b: &Map<String, Value>, k: &str) -> Option<String> {
    b.get(k).and_then(Value::as_str).map(str::to_string)
}

impl Layout {
    /// A layout from its JSON (`{"screen", "regions": [{"id", "component", "bind", "in", "anchor", "w", "h", "x", "y",
    /// "style", "when": [{"text_scale_min", ...place keys}]}]}`), or every problem in it. This checks the file alone;
    /// `check` holds it against a view.
    pub fn from_json(v: &Value) -> Result<Layout, Vec<String>> {
        let mut problems = Vec::new();
        let p = &mut problems;
        let Some(o) = v.as_object() else { return Err(vec!["a layout is an object".into()]) };
        for k in o.keys().filter(|k| !["screen", "regions", "note"].contains(&k.as_str())) {
            p.push(format!("unknown key '{k}' (screen, regions, note)"));
        }
        let screen = o.get("screen").and_then(Value::as_str).unwrap_or("").to_string();
        if !plain_id(&screen) {
            p.push("'screen' is the screen's id: letters, digits and _".into());
        }
        let mut regions: Vec<Region> = Vec::new();
        let list = o.get("regions").and_then(Value::as_array).cloned().unwrap_or_default();
        if list.is_empty() {
            p.push("'regions' is a list of regions, not empty".into());
        }
        for (i, r) in list.iter().enumerate() {
            let Some(r) = r.as_object() else {
                p.push(format!("region {}: a region is an object", i + 1));
                continue;
            };
            let id = r.get("id").and_then(Value::as_str).unwrap_or("").to_string();
            let at = format!("region {} '{id}'", i + 1);
            if !plain_id(&id) {
                p.push(format!("{at}: 'id' is letters, digits and _"));
            } else if regions.iter().any(|x| x.id == id) {
                p.push(format!("{at}: another region has this id"));
            }
            for k in r.keys().filter(|k| !["id", "component", "bind", "in", "anchor", "w", "h", "x", "y", "style", "when", "note"].contains(&k.as_str())) {
                p.push(format!("{at}: unknown key '{k}'"));
            }
            let within = r.get("in").and_then(Value::as_str).map(str::to_string);
            if let Some(w) = &within {
                if !regions.iter().any(|x| &x.id == w) {
                    p.push(format!("{at}: 'in' names '{w}', which is no earlier region"));
                }
            }
            let place = Place::parse(r, None, &at, p);
            let mut when = Vec::new();
            for (j, x) in r.get("when").and_then(Value::as_array).cloned().unwrap_or_default().iter().enumerate() {
                let wat = format!("{at} when {}", j + 1);
                match x.as_object() {
                    Some(wo) => {
                        let scale = wo.get("text_scale_min").and_then(Value::as_f64).filter(|s| (0.5..=2.0).contains(s));
                        if scale.is_none() {
                            p.push(format!("{wat}: 'text_scale_min' is a text scale from 0.5 to 2"));
                        }
                        for k in wo.keys().filter(|k| !["text_scale_min", "anchor", "w", "h", "x", "y"].contains(&k.as_str())) {
                            p.push(format!("{wat}: unknown key '{k}'"));
                        }
                        when.push((scale.unwrap_or(1.0) as f32, Place::parse(wo, Some(place), &wat, p)));
                    }
                    None => p.push(format!("{wat}: an override is an object")),
                }
            }
            let name = r.get("component").and_then(Value::as_str).unwrap_or("");
            let empty = Map::new();
            let bind = r.get("bind").and_then(Value::as_object).unwrap_or(&empty);
            let Some((_, needs, may)) = COMPONENTS.iter().find(|c| c.0 == name) else {
                p.push(format!("{at}: unknown component '{name}' (one of {})", COMPONENTS.iter().map(|c| c.0).collect::<Vec<_>>().join(", ")));
                continue;
            };
            for k in *needs {
                if !bind.contains_key(*k) {
                    p.push(format!("{at}: {name} needs bind '{k}'"));
                }
            }
            for k in bind.keys().filter(|k| !needs.contains(&k.as_str()) && !may.contains(&k.as_str())) {
                p.push(format!("{at}: {name} takes no bind '{k}'"));
            }
            let style = r.get("style").and_then(Value::as_str);
            if style.is_some() && name != "panel_shell" {
                p.push(format!("{at}: 'style' is for a panel_shell"));
            }
            let text = |k: &str| bind_text(bind, k).unwrap_or_default();
            let actions = |p: &mut Vec<String>| match bind.get("actions") {
                Some(Value::Array(a)) if !a.is_empty() && a.iter().all(Value::is_string) => a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect(),
                Some(Value::String(s)) if s == "all" => vec!["*".to_string()],
                _ => {
                    p.push(format!("{at}: bind 'actions' is a list of action ids or \"all\""));
                    Vec::new()
                }
            };
            let component = match name {
                "panel_shell" => Component::PanelShell {
                    title: bind_text(bind, "title"),
                    shell: match style {
                        None | Some("main") => Shell::Main,
                        Some("popup") => Shell::Popup,
                        Some(other) => {
                            p.push(format!("{at}: 'style' is main or popup, not '{other}'"));
                            Shell::Main
                        }
                    },
                },
                "text_block" => Component::TextBlock { text: text("text") },
                "stat_block" => Component::StatBlock { stats: text("stats") },
                "list_row" => Component::ListRow { rows: text("rows"), select: bind_text(bind, "select") },
                "tab_rail" => Component::TabRail { tabs: text("tabs"), select: text("select") },
                "slot_card" => Component::SlotCard { slots: text("slots"), select: bind_text(bind, "select") },
                "menu_column" => Component::MenuColumn { actions: actions(p) },
                "portrait" => Component::Portrait { picture: text("picture") },
                "icon_grid" => Component::IconGrid { icons: text("icons"), select: bind_text(bind, "select") },
                "bar_meter" => Component::BarMeter { meter: text("meter"), label: bind_text(bind, "label") },
                "button_bar" => Component::ButtonBar { actions: actions(p) },
                "tooltip" => Component::Tooltip { text: text("text") },
                _ => Component::Custom { slot: text("slot") },
            };
            regions.push(Region { id, within, place, when, component });
        }
        if problems.is_empty() {
            Ok(Layout { screen, regions })
        } else {
            Err(problems)
        }
    }

    /// What keeps this layout from drawing `view`: a field it binds that the view lacks or holds as another kind, an
    /// action the view does not offer, another screen.
    pub fn check(&self, view: &View) -> Vec<String> {
        let mut p = Vec::new();
        if self.screen != view.screen {
            p.push(format!("the layout is for '{}', the view for '{}'", self.screen, view.screen));
        }
        for r in &self.regions {
            let at = format!("region '{}'", r.id);
            if view.act(&r.id).is_some() {
                p.push(format!("{at}: an action has this id too, so their widget ids would clash; rename the region"));
            }
            let field = |name: &str, kinds: &[&str], p: &mut Vec<String>| match view.get(name) {
                None => p.push(format!("{at}: the view has no field '{name}'")),
                Some(f) if !kinds.contains(&f.kind()) => p.push(format!("{at}: field '{name}' is {}, not {}", f.kind(), kinds.join(" or "))),
                Some(_) => {}
            };
            let action = |id: &str, p: &mut Vec<String>| {
                if id != "*" && view.act(id).is_none() {
                    p.push(format!("{at}: the view offers no action '{id}'"));
                }
            };
            match &r.component {
                Component::PanelShell { title, .. } => {
                    if let Some(t) = title {
                        field(t, &["text"], &mut p);
                    }
                }
                Component::TextBlock { text } | Component::Tooltip { text } => field(text, &["text", "number", "flag"], &mut p),
                Component::StatBlock { stats } => field(stats, &["stats"], &mut p),
                Component::ListRow { rows, select } => {
                    field(rows, &["rows"], &mut p);
                    if let Some(a) = select {
                        action(a, &mut p);
                    }
                }
                Component::TabRail { tabs, select } => {
                    field(tabs, &["tabs"], &mut p);
                    action(select, &mut p);
                }
                Component::SlotCard { slots, select } => {
                    field(slots, &["slots"], &mut p);
                    if let Some(a) = select {
                        action(a, &mut p);
                    }
                }
                Component::MenuColumn { actions } | Component::ButtonBar { actions } => {
                    for a in actions {
                        action(a, &mut p);
                    }
                }
                Component::Portrait { picture } => field(picture, &["picture"], &mut p),
                Component::IconGrid { icons, select } => {
                    field(icons, &["icons"], &mut p);
                    if let Some(a) = select {
                        action(a, &mut p);
                    }
                }
                Component::BarMeter { meter, label } => {
                    field(meter, &["meter"], &mut p);
                    if let Some(l) = label {
                        field(l, &["text", "number"], &mut p);
                    }
                }
                Component::Custom { .. } => {}
            }
        }
        p
    }
}

// ---- drawing ----

/// What a click on a drawn layout asks the screen to do: the action, and the row, tab, slot or icon it was for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    pub action: String,
    pub arg: Option<String>,
}

/// What `draw` leaves for the screen: the rects of its `custom` regions, by slot name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Drawn {
    pub custom: BTreeMap<String, Rect>,
}

/// The letter an icon with no picture shows: its file name's first letter.
fn icon_letter(path: &str) -> char {
    path.rsplit('/').next().and_then(|n| n.chars().find(|c| c.is_alphanumeric())).map(|c| c.to_ascii_uppercase()).unwrap_or('?')
}

fn act(action: &str, arg: Option<String>) -> Act {
    Act { action: action.to_string(), arg }
}

/// Draws `view` through `layout` (None: the default) into `area`. Widget ids are `<screen>.<action>` for actions and
/// `<screen>.<region>.<i>` for the rows, tabs, slots and icons of a region. Pictures come from `art`.
pub fn draw(ui: &mut Ui<Act>, art: &mut Art, layout: Option<&Layout>, view: &View, area: Rect) -> Drawn {
    match layout {
        Some(l) => draw_layout(ui, art, l, view, area),
        None => draw_default(ui, art, view, area),
    }
}

fn draw_layout(ui: &mut Ui<Act>, art: &mut Art, l: &Layout, view: &View, area: Rect) -> Drawn {
    let m = ui.m();
    let scale = ui.p.fonts.scale;
    let mut placed: BTreeMap<String, Rect> = BTreeMap::new();
    let mut drawn = Drawn::default();
    for r in &l.regions {
        let parent = r.within.as_ref().and_then(|w| placed.get(w).copied()).unwrap_or(area);
        let place = r.place_at(scale);
        let rect = place.rect_in(parent, m.grid);
        let inner = component(ui, art, &r.id, &r.component, view, rect, &mut drawn);
        placed.insert(r.id.clone(), inner);
    }
    drawn
}

/// One component in `r`; returns the rect a region placed `in` it uses (a panel shell's body, else `r`).
fn component(ui: &mut Ui<Act>, art: &mut Art, region: &str, c: &Component, view: &View, r: Rect, drawn: &mut Drawn) -> Rect {
    let m = ui.m();
    let screen = view.screen.clone();
    let text_of = |name: &str| view.get(name).and_then(Field::as_text).unwrap_or_default();
    // A select action the view offers and has enabled; otherwise the items draw but do nothing.
    let live = |a: &Option<String>| a.as_ref().filter(|a| view.act(a).is_some_and(|v| v.enabled)).cloned();
    match c {
        Component::PanelShell { title, shell } => {
            let mut body = ui.frame_body(shell.role(), r);
            if let Some(t) = title {
                let head = body.cut_top(ui.p.line_h(m.head).ceil() as i32 + m.u(2));
                let gold = ui.t.gold;
                let line = text_of(t);
                ui.p.text_fit(head.x as f32, head.y as f32, &line, m.head, head.w as f32, gold);
            }
            body
        }
        Component::TextBlock { text } => {
            let col = ui.t.text;
            ui.para(r.x, r.y, r.w, &text_of(text), m.body, col);
            r
        }
        Component::Tooltip { text } => {
            let (fill, edge) = (ui.t.tip, ui.t.edge);
            ui.p.panel(r, fill, edge, m.radius);
            let col = ui.t.text;
            let pad = m.padding;
            ui.para(r.x + pad, r.y + pad, r.w - 2 * pad, &text_of(text), m.small, col);
            r
        }
        Component::StatBlock { stats } => {
            if let Some(Field::Stats(s)) = view.get(stats) {
                let (dim, text) = (ui.t.dim, ui.t.text);
                let step = ui.p.line_h(m.body).ceil() as i32 + m.u(1);
                for (i, (k, v)) in s.iter().enumerate() {
                    let y = r.y + i as i32 * step;
                    if y + step > r.y + r.h {
                        ui.report_overflow(region, r, (s.len() - i) as i32 * step);
                        break;
                    }
                    ui.p.text_fit(r.x as f32, y as f32, k, m.body, r.w as f32 * 0.6, dim);
                    let w = ui.p.text_w(v, m.body);
                    ui.p.text((r.x + r.w) as f32 - w, y as f32, v, m.body, text);
                }
            }
            r
        }
        Component::ListRow { rows, select } => {
            if let Some(Field::Rows(list)) = view.get(rows) {
                let (h, gap) = (m.row_h, m.u(1));
                let text = ui.t.text;
                for (i, row) in list.iter().enumerate() {
                    let y = r.y + i as i32 * (h + gap);
                    if y + h > r.y + r.h {
                        ui.report_overflow(region, r, (list.len() - i) as i32 * (h + gap));
                        break;
                    }
                    let mut rr = Rect::new(r.x, y, r.w, h);
                    let on = live(select).filter(|_| row.enabled).map(|a| act(&a, Some(row.id.clone())));
                    let col = if row.enabled { text } else { ui.t.dim };
                    // A row's picture sits right of the label's text, before the detail.
                    ui.row(&format!("{screen}.{region}.{i}"), rr, &row.label, col, on, false, Vec::new());
                    if let Some(img) = row.picture.as_ref().and_then(|p| art.image(p, h, h)) {
                        let lx = rr.x + m.u(4) + ui.p.text_w(&row.label, m.body).ceil() as i32 + m.u(2);
                        if lx + h <= rr.x + rr.w {
                            ui.image_in(img, Rect::new(lx, rr.y, h, h));
                        }
                    }
                    if !row.detail.is_empty() {
                        rr = rr.pad(Insets::xy(m.u(4), 0));
                        let w = ui.p.text_w(&row.detail, m.body);
                        let dim = ui.t.dim;
                        ui.p.text((rr.x + rr.w) as f32 - w, rr.y as f32 + (h as f32 - ui.p.line_h(m.body)) / 2.0, &row.detail, m.body, dim);
                    }
                }
            }
            r
        }
        Component::TabRail { tabs, select } => {
            let n0 = ui.widgets.len();
            let on = live(&Some(select.clone())).is_some();
            let out = match view.get(tabs) {
            // A region too low for a rail gets a strip of tabs across it (the default's form).
            Some(Field::Tabs { labels, open }) if r.h < m.u(24) => {
                let parts = vec![fill(1); labels.len().max(1)];
                for (i, (l, cell)) in labels.iter().zip(r.split_x(&parts, m.u(1))).enumerate() {
                    ui.tab(&format!("{screen}.{region}.{i}"), cell, l, act(select, Some(i.to_string())), i == *open);
                }
                r
            }
            Some(Field::Tabs { labels, open }) => {
                let select = select.clone();
                crate::panes::tab_rail(ui, &format!("{screen}.{region}"), r, labels, *open, |i| act(&select, Some(i.to_string())))
            }
            _ => r,
            };
            if !on {
                for w in &mut ui.widgets[n0..] {
                    w.action = None;
                    w.enabled = false;
                }
            }
            // A strip leaves no page under it: a region `in` it goes below the strip.
            if out == r && r.h < m.u(24) { Rect::new(r.x, r.y + r.h, r.w, 0) } else { out }
        }
        Component::SlotCard { slots, select } => {
            if let Some(Field::Slots(list)) = view.get(slots) {
                let h = m.u(18).max(r.h / list.len().max(1) as i32 - m.u(1)).min(m.u(30));
                for (i, s) in list.iter().enumerate() {
                    let y = r.y + i as i32 * (h + m.u(1));
                    if y + h > r.y + r.h {
                        ui.report_overflow(region, r, (list.len() - i) as i32 * (h + m.u(1)));
                        break;
                    }
                    let thumb = s.picture.as_ref().and_then(|p| art.image(p, h, h)).cloned();
                    let card = crate::panes::SlotView { thumb: thumb.as_ref(), title: s.title.clone(), lines: s.lines.clone(), portraits: Vec::new(), stamp: s.stamp.clone() };
                    let on = live(select).map(|a| act(&a, Some(s.id.clone())));
                    crate::panes::slot_card(ui, &format!("{screen}.{region}.{i}"), Rect::new(r.x, y, r.w, h), &card, on, false, &[]);
                }
            }
            r
        }
        Component::MenuColumn { actions } | Component::ButtonBar { actions } => {
            let list: Vec<&ActionView> = if actions.iter().any(|a| a == "*") { view.actions.iter().collect() } else { actions.iter().filter_map(|a| view.act(a)).collect() };
            if list.is_empty() {
                return r;
            }
            let column = matches!(c, Component::MenuColumn { .. });
            let cells = if column {
                let h = m.u(9).min(r.h / list.len() as i32);
                (0..list.len()).map(|i| Rect::new(r.x, r.y + i as i32 * (h + m.u(1)), r.w, h)).collect::<Vec<_>>()
            } else {
                let parts = vec![fill(1); list.len()];
                r.split_x(&parts, m.gutter)
            };
            for (a, cell) in list.iter().zip(cells) {
                ui.button(&format!("{screen}.{}", a.id), cell, &a.label, act(&a.id, None), a.enabled);
            }
            r
        }
        Component::Portrait { picture } => {
            if let Some(Field::Picture(p)) = view.get(picture) {
                let side = r.w.min(r.h);
                let sq = r.place(Anchor::Center, side, side);
                let body = ui.frame_body("inset", sq);
                match art.image(p, body.w.max(1), body.h.max(1)) {
                    Some(img) => ui.image_in(img, body),
                    None => {
                        let dim = ui.t.dim;
                        ui.p.text_fit(body.x as f32, body.y as f32, p, m.small, body.w as f32, dim);
                    }
                }
            }
            r
        }
        Component::IconGrid { icons, select } => {
            if let Some(Field::Icons(list)) = view.get(icons) {
                let side = m.u(16);
                let cols = (r.w / (side + m.u(1))).max(1);
                for (i, p) in list.iter().enumerate() {
                    let (cx, cy) = (i as i32 % cols, i as i32 / cols);
                    let cell = Rect::new(r.x + cx * (side + m.u(1)), r.y + cy * (side + m.u(1)), side, side);
                    if cell.y + side > r.y + r.h {
                        ui.report_overflow(region, r, side);
                        break;
                    }
                    let px = ui.icon_px(cell);
                    let img = art.image(p, px, px).cloned();
                    let id = format!("{screen}.{region}.{i}");
                    match live(select) {
                        Some(a) => ui.icon_button(&id, cell, p, img.as_ref(), crate::ui::Glyph::Letter(icon_letter(p)), act(&a, Some(i.to_string())), true, false, Vec::new()),
                        None => {
                            if let Some(img) = &img {
                                ui.image_in(img, cell);
                            }
                            ui.push(id, "icon", p.clone(), cell, true, false, None, Vec::new());
                        }
                    }
                }
            }
            r
        }
        Component::BarMeter { meter, label } => {
            if let Some(Field::Meter { value, max }) = view.get(meter) {
                let (track, fillc, text) = (ui.t.track, ui.t.track_fill, ui.t.text);
                let mut bar = r;
                if let Some(l) = label {
                    let head = bar.cut_top(ui.p.line_h(m.small).ceil() as i32 + m.u(1));
                    ui.p.text_fit(head.x as f32, head.y as f32, &text_of(l), m.small, head.w as f32, text);
                }
                let bar = bar.place(Anchor::TopLeft, fill(1), m.u(4).min(bar.h.max(1)));
                ui.p.panel(bar, track, track, m.radius);
                let w = ((value / max).clamp(0.0, 1.0) * bar.w as f64) as i32;
                if w > 0 {
                    ui.p.panel(Rect::new(bar.x, bar.y, w, bar.h), fillc, fillc, m.radius);
                }
                let tip = format!("{} / {}", number_text(*value), number_text(*max));
                ui.push(format!("{screen}.{region}"), "meter", tip, r, true, false, None, Vec::new());
            }
            r
        }
        Component::Custom { slot } => {
            drawn.custom.insert(slot.clone(), r);
            r
        }
    }
}

/// US4: the default for `view`: a main panel shell over `area` (its title the field `title` when it is a text), every
/// other field top to bottom in the component its kind takes, and every action in a footer button bar. A list, tab,
/// slot or icon field `<name>` is wired to the action of the same id when the view offers one.
pub fn draw_default(ui: &mut Ui<Act>, art: &mut Art, view: &View, area: Rect) -> Drawn {
    let m = ui.m();
    let mut drawn = Drawn::default();
    let title = view.get("title").filter(|f| matches!(f, Field::Text(_))).map(|_| "title".to_string());
    let shell = Component::PanelShell { title, shell: Shell::Main };
    let mut body = component(ui, art, "panel", &shell, view, area.pad(Insets::all(m.margin)), &mut drawn);
    let foot = if view.actions.is_empty() { None } else { Some(body.cut_bottom(m.u(16))) };
    let wired = |name: &str| view.act(name).map(|a| a.id.clone());
    let line = ui.p.line_h(m.body).ceil() as i32 + m.u(1);
    for (name, f) in view.fields.iter().filter(|(n, _)| n != "title" || !matches!(view.get("title"), Some(Field::Text(_)))) {
        let (c, h) = match f {
            Field::Text(_) | Field::Number(_) | Field::Flag(_) => (Component::TextBlock { text: name.clone() }, line),
            Field::Picture(_) => (Component::Portrait { picture: name.clone() }, m.u(32)),
            Field::Meter { .. } => (Component::BarMeter { meter: name.clone(), label: None }, m.u(5)),
            Field::Rows(rows) => (Component::ListRow { rows: name.clone(), select: wired(name) }, rows.len() as i32 * (m.row_h + m.u(1))),
            Field::Stats(s) => (Component::StatBlock { stats: name.clone() }, s.len() as i32 * line),
            Field::Tabs { .. } => (Component::TabRail { tabs: name.clone(), select: wired(name).unwrap_or_default() }, m.u(12)),
            Field::Slots(s) => (Component::SlotCard { slots: name.clone(), select: wired(name) }, s.len() as i32 * m.u(19)),
            Field::Icons(i) => (Component::IconGrid { icons: name.clone(), select: wired(name) }, ((i.len() as i32 + 7) / 8).max(1) * m.u(17)),
        };
        let label_h = ui.p.line_h(m.small).ceil() as i32;
        let head = body.cut_top(label_h);
        let dim = ui.t.dim;
        ui.p.text_fit(head.x as f32, head.y as f32, name, m.small, head.w as f32, dim);
        if h > body.h {
            ui.report_overflow("default", body, h - body.h);
        }
        let r = body.cut_top(h.min(body.h));
        let _ = body.cut_top(m.u(2).min(body.h));
        component(ui, art, &format!("f_{name}"), &c, view, r, &mut drawn);
    }
    if let Some(foot) = foot {
        let bar = Component::ButtonBar { actions: vec!["*".to_string()] };
        component(ui, art, "actions", &bar, view, foot.pad(Insets { l: 0, t: m.u(2), r: 0, b: 0 }), &mut drawn);
    }
    drawn
}

/// The widget ids and kinds a drawing recorded, for a test or the agent link.
pub fn widget_ids(w: &[Widget<Act>]) -> Vec<String> {
    w.iter().map(|w| w.id.clone()).collect()
}

// ---- files ----

/// The first layer's file at `rel`, and its text.
fn first_text(layers: &Layers, rel: &str) -> Option<(PathBuf, Result<String, String>)> {
    let path = layers.find(rel)?;
    let text = vfs::read_to_string(&path).map_err(|e| e.to_string());
    Some((path, text))
}

/// Screen `screen`'s layout over `layers`: None when no layer has the file; the problems when it does not parse.
pub fn load_layout(layers: &Layers, screen: &str) -> Option<Result<Layout, Vec<String>>> {
    let rel = format!("{LAYOUTS_DIR}/{screen}.json");
    let (path, text) = first_text(layers, &rel)?;
    let at = path.display().to_string();
    Some(
        text.map_err(|e| vec![format!("{at}: {e}")])
            .and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| vec![format!("{at}: {e}")]))
            .and_then(|v| Layout::from_json(&v).map_err(|ps| ps.into_iter().map(|p| format!("{at}: {p}")).collect()))
            .and_then(|l| if l.screen == screen { Ok(l) } else { Err(vec![format!("{at}: 'screen' is '{}', the file names '{screen}'", l.screen)]) }),
    )
}

/// Screen `screen`'s view fixture over `layers`, like `load_layout`.
pub fn load_fixture(layers: &Layers, screen: &str) -> Option<Result<View, Vec<String>>> {
    let rel = format!("{VIEWS_DIR}/{screen}{VIEW_SUFFIX}");
    let (path, text) = first_text(layers, &rel)?;
    let at = path.display().to_string();
    Some(
        text.map_err(|e| vec![format!("{at}: {e}")])
            .and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| vec![format!("{at}: {e}")]))
            .and_then(|v| View::from_json(&v).map_err(|ps| ps.into_iter().map(|p| format!("{at}: {p}")).collect())),
    )
}

/// The screens that have a layout file or a fixture in any layer, sorted.
pub fn screens(layers: &Layers) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for root in &layers.roots {
        for (dir, suffix) in [(LAYOUTS_DIR, ".json"), (VIEWS_DIR, VIEW_SUFFIX)] {
            for p in vfs::list_dir(root.join(dir)).unwrap_or_default() {
                let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                if let Some(stem) = name.strip_suffix(suffix).filter(|s| !s.ends_with(".view") || suffix == VIEW_SUFFIX) {
                    if !out.iter().any(|s| s == stem) {
                        out.push(stem.to_string());
                    }
                }
            }
        }
    }
    out.sort();
    out
}

/// The gate's check (US5) over a game's asset layers: every layout and fixture parses, every layout has a fixture and
/// fits it. Every problem, each naming its file.
pub fn check_layers(layers: &Layers) -> Vec<String> {
    let mut problems = Vec::new();
    for screen in screens(layers) {
        let view = load_fixture(layers, &screen);
        if let Some(Err(ps)) = &view {
            problems.extend(ps.iter().cloned());
        }
        match load_layout(layers, &screen) {
            None => {}
            Some(Err(ps)) => problems.extend(ps),
            Some(Ok(l)) => match &view {
                None => problems.push(format!("{LAYOUTS_DIR}/{screen}.json: no fixture {VIEWS_DIR}/{screen}{VIEW_SUFFIX} to check it against")),
                Some(Ok(v)) => problems.extend(l.check(v).into_iter().map(|p| format!("{LAYOUTS_DIR}/{screen}.json: {p}"))),
                Some(Err(_)) => {}
            },
        }
    }
    problems
}

/// `check_layers` on the game in `game_dir` (its assets, its kit's, the engine's).
pub fn check_game(game_dir: &Path) -> Vec<String> {
    check_layers(&Layers::for_game(game_dir))
}

// ---- the run-time loader ----

/// The layouts a running game draws with: loaded once per screen, checked against the screen's view, and in a native
/// debug build read again when the file changes (US6). A layout that does not load or fit falls back to the default;
/// its problems wait in `take_problems` for the shell's notice, once per change.
#[derive(Default)]
pub struct Layouts {
    layers: Layers,
    cache: BTreeMap<String, Loaded>,
    problems: Vec<String>,
}

struct Loaded {
    layout: Option<Layout>,
    /// The file's change time when it was read (native debug builds only).
    stamp: Option<u128>,
    /// The view's shape (field names and kinds, action ids) it was last checked against, and whether it fit.
    checked: Option<(String, bool)>,
}

impl Layouts {
    pub fn new(layers: Layers) -> Self {
        Self { layers, cache: BTreeMap::new(), problems: Vec::new() }
    }

    /// The layout to draw `view` with (None: the default).
    pub fn get(&mut self, view: &View) -> Option<&Layout> {
        let screen = view.screen.clone();
        let stamp = stamp(&self.layers, &screen);
        let stale = self.cache.get(&screen).is_none_or(|l| l.stamp != stamp);
        if stale {
            let layout = match load_layout(&self.layers, &screen) {
                None => None,
                Some(Ok(l)) => Some(l),
                Some(Err(ps)) => {
                    self.problems.extend(ps);
                    None
                }
            };
            self.cache.insert(screen.clone(), Loaded { layout, stamp, checked: None });
        }
        let shape = shape_of(view);
        let entry = self.cache.get_mut(&screen)?;
        if entry.checked.as_ref().map(|c| &c.0) != Some(&shape) {
            let fits = match &entry.layout {
                None => true,
                Some(l) => {
                    let ps = l.check(view);
                    self.problems.extend(ps.iter().map(|p| format!("{LAYOUTS_DIR}/{screen}.json: {p} (drawn with the default)")));
                    ps.is_empty()
                }
            };
            entry.checked = Some((shape, fits));
        }
        // A layout that does not fit this shape stays loaded: a later view that fits it draws it again.
        if entry.checked.as_ref().is_some_and(|c| !c.1) {
            return None;
        }
        entry.layout.as_ref()
    }

    /// The problems found since the last call.
    pub fn take_problems(&mut self) -> Vec<String> {
        std::mem::take(&mut self.problems)
    }
}

/// What a check depends on: the fields' names and kinds, and the action ids.
fn shape_of(view: &View) -> String {
    let fields: Vec<String> = view.fields.iter().map(|(n, f)| format!("{n}:{}", f.kind())).collect();
    let actions: Vec<&str> = view.actions.iter().map(|a| a.id.as_str()).collect();
    format!("{}|{}", fields.join(","), actions.join(","))
}

/// The layout file's change time, in a native debug build (layouts reload on save); None elsewhere (loaded once).
#[cfg(not(target_arch = "wasm32"))]
fn stamp(layers: &Layers, screen: &str) -> Option<u128> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let path = layers.find(&format!("{LAYOUTS_DIR}/{screen}.json"))?;
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    modified.duration_since(aegis_core::clock::UNIX_EPOCH).ok().map(|d| d.as_nanos())
}

#[cfg(target_arch = "wasm32")]
fn stamp(_layers: &Layers, _screen: &str) -> Option<u128> {
    None
}

// ---- drawing a fixture with no game (US6) ----

/// What `render` drew: the canvas, the widget ids, the problems that sent it to the default.
pub struct Rendered {
    pub canvas: crate::canvas::Canvas,
    pub widgets: Vec<String>,
    /// The widgets a click on does nothing (no action: text, pictures, items whose select action is off).
    pub inert: Vec<String>,
    pub problems: Vec<String>,
    pub drawn: Drawn,
}

/// Screen `screen`'s fixture drawn through its layout (or the default, when `default` or when the layout does not load or
/// fit) on a `size` canvas, in the theme of `layers`, with the engine font found in the last layer (`fonts/`). Err when
/// there is no fixture, it does not parse, or no font is found.
pub fn render(layers: &Layers, screen: &str, size: (u32, u32), default: bool) -> Result<Rendered, Vec<String>> {
    let view = load_fixture(layers, screen).unwrap_or_else(|| Err(vec![format!("no fixture {VIEWS_DIR}/{screen}{VIEW_SUFFIX} in any asset layer")]))?;
    let font = layers.roots.iter().rev().map(|r| r.join("fonts/OpenSans-SemiBold.ttf")).find(|p| vfs::is_file(p)).ok_or(vec!["no fonts/OpenSans-SemiBold.ttf in the asset layers".to_string()])?;
    let ttf = vfs::read(&font).map_err(|e| vec![format!("{}: {e}", font.display())])?;
    crate::theme::install(crate::theme::load(layers)?);
    let mut problems = Vec::new();
    let layout = if default {
        None
    } else {
        match load_layout(layers, screen) {
            None => None,
            Some(Err(ps)) => {
                problems.extend(ps);
                None
            }
            Some(Ok(l)) => {
                let ps = l.check(&view);
                if ps.is_empty() {
                    Some(l)
                } else {
                    problems.extend(ps);
                    None
                }
            }
        }
    };
    let mut g = crate::ui::Gfx {
        canvas: crate::canvas::Canvas::new(size.0 as i32, size.1 as i32),
        view: crate::painter::View::fit(size.0, size.1).for_grid(crate::painter::Grid::Native),
        fonts: crate::painter::Fonts::new(&ttf),
        kit: crate::ui::Kit::default(),
    };
    let mut art = Art::layered(layers.clone());
    let mut ui: Ui<Act> = Ui::new(&mut g, (-1, -1), true);
    let (bg, area) = (ui.t.bg, ui.screen());
    ui.p.fill(area, bg);
    let drawn = draw(&mut ui, &mut art, layout.as_ref(), &view, area);
    let widgets = widget_ids(&ui.widgets);
    let inert = ui.widgets.iter().filter(|w| w.action.is_none()).map(|w| w.id.clone()).collect();
    drop(ui);
    Ok(Rendered { canvas: g.canvas, widgets, inert, problems, drawn })
}
