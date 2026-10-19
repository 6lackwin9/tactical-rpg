//! Content Forge and World Forge viewers (scripts/ui/forge, scripts/ui/world; dev only, read-only): kinds as tabs, the
//! entries on the left, every field of the selected entry in the middle, an engine summary on the right (built units,
//! skill costs, world inheritance and who uses the entry). Validation and editing stay in Godot for now.

use aegis_ui::canvas::{rgb, rgba, Rect};
use crate::screens::{Cx, GameCx, Nav, Screen};
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, BORDER, DIM, GOLD, GOOD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use serde_json::{json, Value};

const CONTENT_TABS: [(&str, &str); 8] =
    [("skills", "Skills"), ("items", "Items"), ("item_families", "Item families"), ("passives", "Passives"), ("classes", "Classes"), ("units", "Units"), ("encounters", "Encounters"), ("terrain", "Terrain")];
const WORLD_TABS: [(&str, &str); 14] = [
    ("overview", "Overview"),
    ("regions", "Regions"),
    ("locations", "Locations"),
    ("factions", "Factions"),
    ("characters", "Characters"),
    ("items", "Items"),
    ("disciplines", "Disciplines"),
    ("quests", "Quests"),
    ("lore", "Lore"),
    ("encounters", "Encounters"),
    ("loot", "Loot"),
    ("dialogues", "Dialogues"),
    ("shops", "Shops"),
    ("routes", "Routes"),
];

#[derive(Debug, Clone)]
pub enum FAct {
    Tab(usize),
    Select(String),
    NextWorld,
    Exit,
}

pub struct Forge {
    world_forge: bool,
    world: String,
    tab: usize,
    selected: String,
    list_scroll: usize,
    field_scroll: usize,
    mouse: (i32, i32),
    w: Vec<Widget<FAct>>,
}

impl Forge {
    pub fn content() -> Self {
        Self { world_forge: false, world: String::new(), tab: 0, selected: String::new(), list_scroll: 0, field_scroll: 0, mouse: (0, 0), w: Vec::new() }
    }

    pub fn world(cx: &Cx) -> Self {
        let world = cx.data.world_ids().first().cloned().unwrap_or_default();
        Self { world_forge: true, world, ..Self::content() }
    }

    fn tabs(&self) -> &'static [(&'static str, &'static str)] {
        if self.world_forge {
            &WORLD_TABS
        } else {
            &CONTENT_TABS
        }
    }

    fn kind(&self) -> &'static str {
        self.tabs()[self.tab].0
    }

    /// (id, label, entry) rows of the open tab.
    fn rows(&self, cx: &Cx) -> Vec<(String, String, Dict)> {
        let d = &cx.data;
        if !self.world_forge {
            let mut rows: Vec<(String, String, Dict)> = d.content.get(self.kind()).map(|m| m.values().map(|e| (s(e, "id"), s_or(e, "name", &s(e, "id")), e.clone())).collect()).unwrap_or_default();
            rows.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
            return rows;
        }
        if self.kind() == "overview" {
            return d
                .world_ids()
                .into_iter()
                .map(|w| {
                    let m = d.manifest(&w).clone();
                    let ext = json::sget(&m, "extends");
                    let label = format!("{}  {}", s_or(&m, "name", &w), if ext.is_empty() { "(root)".to_string() } else { format!("-> extends {}", ext.join(", ")) });
                    (w, label, m)
                })
                .collect();
        }
        d.merged_entries(&self.world, self.kind())
            .into_iter()
            .map(|e| {
                let mut label = s_or(e, "name", &s(e, "id"));
                if s(e, "_world") != self.world {
                    label = format!("{label}  ({})", s(e, "_world"));
                }
                (s(e, "_qid"), label, e.clone())
            })
            .collect()
    }

    fn act(&mut self, cx: &mut Cx, a: FAct) {
        match a {
            FAct::Tab(t) => {
                self.tab = t;
                self.selected.clear();
                self.list_scroll = 0;
                self.field_scroll = 0;
            }
            FAct::Select(id) => {
                self.selected = id;
                self.field_scroll = 0;
            }
            FAct::NextWorld => {
                let ids = cx.data.world_ids();
                let at = ids.iter().position(|w| *w == self.world).unwrap_or(0);
                self.world = ids.get((at + 1) % ids.len().max(1)).cloned().unwrap_or_default();
                self.selected.clear();
            }
            FAct::Exit => cx.nav.push(Nav::Pop),
        }
    }

    /// The right panel: what the engine makes of the entry.
    fn summary(&self, cx: &Cx, id: &str, e: &Dict) -> Vec<(String, aegis_ui::canvas::Rgba)> {
        let d = &cx.data;
        let cat = &d.cat;
        let mut out: Vec<(String, aegis_ui::canvas::Rgba)> = Vec::new();
        let mut line = |t: String, c| out.push((t, c));
        match (self.world_forge, self.kind()) {
            (false, "skills") => {
                if let Some(sk) = cat.skill(id) {
                    line(format!("{} [{}]", sk.name, sk.grade), GOLD);
                    line(format!("Cost: {} AP, {} block{}", sk.level, sk.blocks, if sk.blocks == 1 { "" } else { "s" }), TEXT);
                    line(format!("Kind: {}   Stance: {}", sk.kind, sk.stance), TEXT);
                    line(format!("Family: {}, tier {}", cat.family_of(id), sk.tier), TEXT);
                    line(format!("Targets: {}, reach {}-{}", sk.target_mode, sk.reach_min, sk.reach_max), TEXT);
                    if !sk.grant_status.is_empty() {
                        line(format!("Grants: {}", sk.grant_status), TEXT);
                    }
                    if !sk.status_effect.is_empty() {
                        line(format!("Status: {} ({}% potency, {} rounds)", sk.status_effect, sk.status_potency, sk.status_rounds), TEXT);
                    }
                }
            }
            (false, "units") => {
                if let Some(def) = cat.units.get(id) {
                    let u = emberwake_rules::build_unit(cat, def, 0);
                    line(format!("{}  Lv {} {}", u.name, def.level, def.tier), GOLD);
                    line(format!("HP {}   Speed {}   Reach {}   AP {}", u.max_hp, u.speed, u.attack_range, u.max_ap), TEXT);
                    line(format!("Power {} melee / {} ranged", emberwake_rules::damage::power_of(&u, false, false), emberwake_rules::damage::power_of(&u, true, false)), TEXT);
                    line(format!("Armor rating {}   Crit {:.0}%", emberwake_rules::damage::armor_rating(&u), emberwake_rules::damage::crit_chance(&u)), TEXT);
                    let gear: Vec<String> = u.inventory.gear().map(|g| g.item.name.clone()).collect();
                    line(format!("Gear: {}", if gear.is_empty() { "none".into() } else { gear.join(", ") }), TEXT);
                    line(format!("Skills: {}", u.active_families().join(", ")), TEXT);
                }
            }
            (false, "items") => {
                if let Some(it) = cat.items.get(id) {
                    line(format!("{} ({})", it.name, it.item_type), GOLD);
                    if it.is_equipment() {
                        line(format!("Slot {}   Power {}   Mitigation {}   Burden {}", it.slot, it.base_power, it.mitigation, it.burden), TEXT);
                    }
                    if it.item_type == "CONSUMABLE" {
                        line(format!("Heal {}   Damage {}   Use {} block(s)", it.heal, it.damage, it.use_blocks), TEXT);
                    }
                    line(format!("Value {}", json::i(e, "value", 0)), TEXT);
                }
            }
            (false, "classes") => {
                if let Some(c) = cat.classes.get(id) {
                    line(c.name.clone(), GOLD);
                    line(format!("Base: {}", c.base_stats.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", ")), TEXT);
                    line(format!("Growth: {}", c.growth.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", ")), TEXT);
                    line(format!("Path: {}", c.path_steps().iter().map(|(l, s)| format!("{s} (Lv {l})")).collect::<Vec<_>>().join(", ")), TEXT);
                }
            }
            (true, "overview") => {
                let m = d.manifest(id);
                line(format!("{}  {}", s_or(m, "name", id), s(m, "tagline")), GOLD);
                let ext = json::sget(m, "extends");
                line(format!("Inheritance: {}", if ext.is_empty() { "none (a root world)".into() } else { ext.join(", ") }), TEXT);
                let con = json::sget(m, "connects");
                line(format!("Connects to: {}", if con.is_empty() { "-".into() } else { con.join(", ") }), TEXT);
                let below: Vec<String> = d.world_ids().into_iter().filter(|w| w != id && d.lineage(w).iter().any(|x| x == id)).collect();
                line(format!("Extended by: {}", if below.is_empty() { "-".into() } else { below.join(", ") }), TEXT);
                let counts: Vec<String> = WORLD_TABS[1..].iter().map(|(k, _)| format!("{k} {}", d.own_entries(id, k).len())).collect();
                line(format!("Entries: {}", counts.join(", ")), TEXT);
                line(String::new(), TEXT);
                line(s(m, "description"), TEXT);
            }
            (true, _) => {
                line(format!("{}  [{}]", s_or(e, "name", &s(e, "id")), s(e, "_qid")), GOLD);
                line(format!("Defined in {}", s(e, "_world")), TEXT);
                let ov = s(e, "override");
                if !ov.is_empty() {
                    line(format!("Overrides {ov}"), TEXT);
                }
                if self.kind() == "characters" {
                    let unit = d.character_unit_entry(e);
                    if let Some(def) = (!unit.is_empty()).then(|| emberwake_data::UnitDef::from_obj(&unit)) {
                        let u = emberwake_rules::build_unit(cat, &def, 0);
                        line(format!("Plays as {}: Lv {} {}, HP {}, speed {}", s(e, "base_unit"), def.level, def.tier, u.max_hp, u.speed), TEXT);
                    }
                    let passives = d.character_passives(e, &self.world);
                    if !passives.is_empty() {
                        line(format!("Passives: {}", passives.join(", ")), TEXT);
                    }
                }
            }
            _ => line(s_or(e, "name", id), GOLD),
        }
        // Used by: entries that mention this id (content: any kind; world: this world's merged entries).
        let needle = format!("\"{}\"", if self.world_forge { s(e, "id") } else { id.to_string() });
        let mut users = Vec::new();
        if self.world_forge {
            if self.kind() != "overview" {
                for (k, _) in WORLD_TABS[1..].iter() {
                    for other in d.merged_entries(&self.world, k) {
                        if s(other, "_qid") != id && Value::Object(other.clone()).to_string().contains(&needle) {
                            users.push(format!("{k}:{}", s(other, "id")));
                        }
                    }
                }
            }
        } else {
            for (k, m) in &d.content {
                for (oid, other) in m {
                    if !(k == self.kind() && oid == id) && Value::Object(other.clone()).to_string().contains(&needle) {
                        users.push(format!("{k}:{oid}"));
                    }
                }
            }
        }
        if !users.is_empty() || (self.world_forge && self.kind() != "overview") || !self.world_forge {
            out.push((String::new(), TEXT));
            out.push((format!("Used by: {}", if users.is_empty() { "nothing".into() } else { users.into_iter().take(12).collect::<Vec<_>>().join(", ") }), DIM));
        }
        out
    }
}

fn value_text(v: &Value) -> String {
    match v {
        Value::Array(a) if a.is_empty() => "(none)".into(),
        Value::Array(a) if a.iter().all(|x| !x.is_object() && !x.is_array()) => a.iter().map(json::gstr).collect::<Vec<_>>().join(", "),
        Value::Object(o) if o.is_empty() => "{}".into(),
        Value::String(s) if s.is_empty() => "(none)".into(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
        _ => json::gstr(v),
    }
}

impl Screen<GameCx> for Forge {
    fn id(&self) -> &'static str {
        if self.world_forge {
            "world_forge"
        } else {
            "content_forge"
        }
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.mouse = mouse;
        let mut ui: Ui<FAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(12, 16, 26));
        let head = if self.world_forge { "WORLD FORGE (viewer)" } else { "CONTENT FORGE (viewer)" };
        let mut x = ui.p.text(6.0, 4.0, head, BODY, GOLD) as i32 + 6;
        if self.world_forge {
            let name = format!("{}  [{}]", cx.world_name(&self.world), self.world);
            ui.dropdown("forge.world", Rect::new(x, 2, 140, 14), &name, FAct::NextWorld, true, Vec::new());
            x = 6;
        }
        let ty = if self.world_forge { 20 } else { 3 };
        for (i, (k, label)) in self.tabs().iter().enumerate() {
            let w = ui.p.text_w(label, SMALL) as i32 + 8;
            ui.button_full(&format!("forge.tab.{k}"), Rect::new(x, ty, w, 12), label, FAct::Tab(i), true, i == self.tab, SMALL, Vec::new());
            x += w + 2;
        }
        ui.button("forge.exit", Rect::new(W - 34, 3, 30, 12), "Exit", FAct::Exit, true);
        let top = ty + 16;
        let (list, mid, right) = (Rect::new(5, top, 135, H - top - 5), Rect::new(144, top, 286, H - top - 5), Rect::new(434, top, 201, H - top - 5));
        for r in [list, mid, right] {
            ui.p.panel(r, rgba(18, 24, 38, 255), BORDER, 2.0);
        }
        let rows = self.rows(cx);
        if self.selected.is_empty() || !rows.iter().any(|r| r.0 == self.selected) {
            self.selected = rows.first().map(|r| r.0.clone()).unwrap_or_default();
        }
        let fit = ((list.h - 8) / 13) as usize;
        self.list_scroll = self.list_scroll.min(rows.len().saturating_sub(fit));
        for (i, (id, label, _)) in rows.iter().enumerate().skip(self.list_scroll).take(fit) {
            let r = Rect::new(list.x + 3, list.y + 4 + (i - self.list_scroll) as i32 * 13, list.w - 6, 12);
            ui.row(&format!("forge.entry.{}", id.replace(':', ".")), r, label, if *id == self.selected { GOLD } else { TEXT }, Some(FAct::Select(id.clone())), *id == self.selected, Vec::new());
        }
        if let Some((id, label, e)) = rows.iter().find(|r| r.0 == self.selected) {
            ui.p.text_fit(mid.x as f32 + 5.0, mid.y as f32 + 4.0, &format!("{label}  [{}]", s(e, "id")), BODY, mid.w as f32 - 10.0, INFO);
            let mut lines: Vec<(String, String)> = Vec::new();
            for (k, v) in e {
                if !k.starts_with('_') {
                    lines.push((k.clone(), value_text(v)));
                }
            }
            let mut wrapped: Vec<(String, String)> = Vec::new();
            for (k, v) in &lines {
                for (n, part) in ui.p.wrap(v, SMALL, (mid.w - 100) as f32).into_iter().enumerate() {
                    wrapped.push((if n == 0 { k.clone() } else { String::new() }, part));
                }
            }
            let fit = ((mid.h - 22) / 9) as usize;
            self.field_scroll = self.field_scroll.min(wrapped.len().saturating_sub(fit));
            for (i, (k, v)) in wrapped.iter().skip(self.field_scroll).take(fit).enumerate() {
                let y = mid.y as f32 + 17.0 + i as f32 * 9.0;
                ui.p.text(mid.x as f32 + 5.0, y, k, SMALL, DIM);
                ui.p.text(mid.x as f32 + 92.0, y, v, SMALL, if v == "(none)" { DIM } else { TEXT });
            }
            ui.p.text(right.x as f32 + 5.0, right.y as f32 + 4.0, if self.world_forge { "WHAT IT MEANS" } else { "ENGINE SUMMARY" }, SMALL, INFO);
            let mut y = right.y as f32 + 16.0;
            for (t, col) in self.summary(cx, id, e) {
                for part in ui.p.wrap(&t, SMALL, (right.w - 10) as f32) {
                    if y > (right.y + right.h) as f32 - 18.0 {
                        break;
                    }
                    ui.p.text(right.x as f32 + 5.0, y, &part, SMALL, col);
                    y += 8.5;
                }
                if t.is_empty() {
                    y += 4.0;
                }
            }
            ui.p.text(right.x as f32 + 5.0, (right.y + right.h) as f32 - 12.0, "Validation: run it in Godot's Forge for now.", SMALL, GOOD);
        }
        self.w = ui.widgets;
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        let target = if self.mouse.0 < 142 { &mut self.list_scroll } else { &mut self.field_scroll };
        *target = (*target as i64 - lines as i64 * 3).max(0) as usize;
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({"tab": self.kind(), "world": self.world, "selected": self.selected})
    }

    widget_plumbing!();
}
