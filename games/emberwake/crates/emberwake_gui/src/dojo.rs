//! Gear & Skill Dojo (scripts/ui/dojo; dev only), function over form: pick a party unit and level, swap gear per slot
//! from the catalog, read the attributes, proficiencies and derived numbers, and see what each hand and skill deals to
//! the training dummy (DamageCalculator, no crit). The windowed workbench and the step-by-step test console come later.

use aegis_ui::canvas::{rgb, rgba, Rect};
use crate::screens::{Cx, GameCx, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, FOE, GOLD, GOOD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use emberwake_data::{UnitDef, NO_ITEM, STATS};
use emberwake_rules::damage::{self, HitInput};
use emberwake_rules::gear::Slot;
use emberwake_rules::{build_unit, Unit};
use emberwake_world::state::{LOADOUT_SLOTS, POUCH_MAX};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub enum DAct {
    Unit(i64),
    Level(i64),
    Gear(&'static str, i64),
    Reset,
    Exit,
}

pub struct Dojo {
    unit: String,
    level: i32,
    /// Slot -> item id ("" = the class default, NO_ITEM = empty).
    gear: indexmap::IndexMap<&'static str, String>,
    w: Vec<Widget<DAct>>,
}

fn fits(item: &emberwake_data::ItemDef, slot: &str) -> bool {
    let up = slot.to_uppercase();
    item.is_equipment() && (item.slot == up || (up.starts_with("ACCESSORY") && item.slot.starts_with("ACCESSORY")) || (up == "OFF_HAND" && item.slot == "MAIN_HAND" && !item.two_handed && item.item_type == "WEAPON"))
}

impl Dojo {
    pub fn new(cx: &Cx) -> Self {
        let unit = Self::units(cx).first().cloned().unwrap_or_default();
        Self { unit, level: 1, gear: indexmap::IndexMap::new(), w: Vec::new() }
    }

    fn units(cx: &Cx) -> Vec<String> {
        cx.data.cat.units.values().filter(|u| u.side == "party" && u.stat_mode == "derived").map(|u| u.id.clone()).collect()
    }

    fn def(&self, cx: &Cx) -> Option<UnitDef> {
        let mut def = cx.data.cat.units.get(&self.unit)?.clone();
        def.level = self.level;
        def.spend.clear();
        for (slot, item) in &self.gear {
            def.slots.insert(slot.to_string(), item.clone());
        }
        Some(def)
    }

    fn built(&self, cx: &Cx) -> Option<Unit> {
        self.def(cx).map(|d| build_unit(&cx.data.cat, &d, 0))
    }

    fn act(&mut self, cx: &mut Cx, a: DAct) {
        match a {
            DAct::Unit(step) => {
                let ids = Self::units(cx);
                let at = ids.iter().position(|u| *u == self.unit).unwrap_or(0) as i64;
                self.unit = ids[(at + step).rem_euclid(ids.len().max(1) as i64) as usize].clone();
                self.gear.clear();
            }
            DAct::Level(step) => self.level = (self.level + step as i32).clamp(1, emberwake_rules::stats::MAX_LEVEL),
            DAct::Gear(slot, step) => {
                let mut options: Vec<String> = vec![String::new(), NO_ITEM.to_string()];
                options.extend(cx.data.cat.items.values().filter(|i| fits(i, slot)).map(|i| i.id.clone()));
                let cur = self.gear.get(slot).cloned().unwrap_or_default();
                let at = options.iter().position(|o| *o == cur).unwrap_or(0) as i64;
                self.gear.insert(slot, options[(at + step).rem_euclid(options.len() as i64) as usize].clone());
            }
            DAct::Reset => self.gear.clear(),
            DAct::Exit => cx.nav.push(Nav::Pop),
        }
    }
}

impl Screen<GameCx> for Dojo {
    fn id(&self) -> &'static str {
        "gear_dojo"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let cat = &d.cat;
        let mut ui: Ui<DAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(12, 16, 26));
        ui.p.text(6.0, 4.0, "GEAR & SKILL DOJO", BODY, GOLD);
        let name = cat.units.get(&self.unit).map(|u| u.name.clone()).unwrap_or_default();
        ui.p.text(100.0, 4.5, "Unit:", SMALL, DIM);
        ui.button("dojo.unit.prev", Rect::new(122, 3, 10, 12), "<", DAct::Unit(-1), true);
        ui.p.text_center(170.0, 4.5, &name, SMALL, TEXT);
        ui.button("dojo.unit.next", Rect::new(208, 3, 10, 12), ">", DAct::Unit(1), true);
        ui.p.text(226.0, 4.5, "Level:", SMALL, DIM);
        ui.button("dojo.level.minus", Rect::new(252, 3, 10, 12), "-", DAct::Level(-1), self.level > 1);
        ui.p.text_center(271.0, 4.5, &self.level.to_string(), SMALL, TEXT);
        ui.button("dojo.level.plus", Rect::new(280, 3, 10, 12), "+", DAct::Level(1), self.level < emberwake_rules::stats::MAX_LEVEL);
        ui.button("dojo.reset", Rect::new(W - 104, 3, 64, 12), "Reset gear", DAct::Reset, !self.gear.is_empty());
        ui.button("dojo.exit", Rect::new(W - 34, 3, 30, 12), "Exit", DAct::Exit, true);
        let Some(u) = self.built(cx) else {
            self.w = ui.widgets;
            return;
        };
        let panels = [Rect::new(4, 20, 133, 336), Rect::new(141, 20, 150, 336), Rect::new(295, 20, 160, 336), Rect::new(459, 20, 177, 336)];
        for (r, title) in panels.iter().zip(["Unit Inspector", "Unit Gear", "Derived Stats", "Training Dummy"]) {
            ui.p.panel(*r, rgba(18, 24, 38, 255), BORDER, 2.0);
            ui.p.text(r.x as f32 + 5.0, r.y as f32 + 3.0, title, BODY, GOLD);
        }
        // -- Inspector
        let p = panels[0];
        let arch = cat.units.get(&self.unit).and_then(|e| cat.classes.get(&e.class_id)).map(|c| c.name.clone()).unwrap_or_default();
        let mut y = p.y as f32 + 16.0;
        for line in [format!("{}  ({arch})", u.name), format!("Level {} / 10   HP {} / {}", self.level, u.hp, u.max_hp)] {
            ui.p.text(p.x as f32 + 5.0, y, &line, SMALL, TEXT);
            y += 9.0;
        }
        y += 4.0;
        ui.p.text(p.x as f32 + 5.0, y, "CORE ATTRIBUTES", SMALL, INFO);
        y += 10.0;
        let ctx = u.weapon_context();
        for st in STATS {
            let base = u.stats.base(st);
            let eff = u.stats.effective(st, ctx.as_deref(), "");
            let bonus = eff - base;
            ui.p.text(p.x as f32 + 5.0, y, st, SMALL, TEXT);
            ui.p.text(p.x as f32 + 40.0, y, &if bonus != 0 { format!("{eff} ({bonus:+})") } else { eff.to_string() }, SMALL, if bonus > 0 { GOOD } else { TEXT });
            y += 9.0;
        }
        y += 4.0;
        ui.p.text(p.x as f32 + 5.0, y, "PROFICIENCY", SMALL, INFO);
        y += 10.0;
        for gear in u.stats.proficiency.entries.keys().take(14) {
            let lv = u.stats.proficiency.level_of(gear);
            ui.p.text(p.x as f32 + 5.0, y, &format!("{gear}  Lv {lv}  x{:.2}", u.stats.proficiency.scaling_of(gear)), SMALL, if lv > 0 { TEXT } else { DIM });
            y += 8.5;
        }
        // -- Gear
        let p = panels[1];
        let mut y = p.y + 16;
        for slot in LOADOUT_SLOTS {
            let worn = Slot::parse(slot).and_then(|s| u.inventory.get(s)).map(|e| e.item.name.clone()).unwrap_or_else(|| "empty".into());
            let set = self.gear.get(slot).cloned().unwrap_or_default();
            ui.p.text(p.x as f32 + 5.0, y as f32, &slot.replace('_', " "), SMALL, INFO);
            ui.button(&format!("dojo.gear.{slot}.prev"), Rect::new(p.x + 4, y + 9, 10, 12), "<", DAct::Gear(slot, -1), true);
            ui.p.text_fit(p.x as f32 + 17.0, y as f32 + 10.5, &format!("{worn}{}", if set.is_empty() { "  (default)" } else { "" }), SMALL, (p.w - 34) as f32, TEXT);
            ui.button(&format!("dojo.gear.{slot}.next"), Rect::new(p.x + p.w - 14, y + 9, 10, 12), ">", DAct::Gear(slot, 1), true);
            y += 26;
        }
        ui.p.text(p.x as f32 + 5.0, y as f32 + 4.0, "Battle pouch", SMALL, INFO);
        // Every slot, empty ones too (the classes' default pouches are empty), like the Character popup's Gear tab.
        for i in 0..POUCH_MAX as usize {
            let (name, col) = match u.inventory.pouch.get(i) {
                Some(it) => (it.name.clone(), TEXT),
                None => ("Empty".to_string(), DIM),
            };
            ui.p.text(p.x as f32 + 5.0, y as f32 + 14.0 + i as f32 * 9.0, &format!("{}. {name}", i + 1), SMALL, col);
        }
        // -- Derived
        let p = panels[2];
        let rows = [
            ("Melee power", damage::attack_power(&u, false, false).attack_power.to_string()),
            ("Ranged power", damage::attack_power(&u, true, false).attack_power.to_string()),
            ("Off-hand power", if u.inventory.offhand_weapon().is_some() { damage::attack_power(&u, false, true).attack_power.to_string() } else { "-".into() }),
            ("Initiative speed", u.speed.to_string()),
            ("Maximum health", u.max_hp.to_string()),
            ("Armor rating", damage::armor_rating(&u).to_string()),
            ("Unit burden", u.worn_burden().to_string()),
            ("Tiles per move", u.move_per_block().to_string()),
            ("Attack range", u.attack_range.to_string()),
            ("Crit hit rate", format!("{:.1}%", damage::crit_chance(&u))),
            ("Debuff resist", format!("{:.0}%", u.debuff_resistance())),
            ("AP", u.max_ap.to_string()),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let yy = p.y as f32 + 18.0 + i as f32 * 12.0;
            ui.p.text(p.x as f32 + 5.0, yy, k, SMALL, DIM);
            ui.p.text_right((p.x + p.w) as f32 - 6.0, yy, v, SMALL, TEXT);
        }
        let mut yy = p.y as f32 + 18.0 + rows.len() as f32 * 12.0 + 6.0;
        // No loadout set means every owned family is active (Unit::knows_skill), so list the owned ones then.
        let set = u.active_families();
        let (title, fams) = if set.is_empty() { ("Active skills (no loadout: all owned)", u.owned_families(cat)) } else { ("Active skills (loadout)", set) };
        ui.p.text(p.x as f32 + 5.0, yy, title, SMALL, INFO);
        yy += 10.0;
        if fams.is_empty() {
            ui.p.text(p.x as f32 + 5.0, yy, "Basic skills only at this level.", SMALL, DIM);
        }
        for fam in fams {
            if yy > (p.y + p.h) as f32 - 10.0 {
                break;
            }
            let rank = cat.skills.iter().filter(|s| cat.family_of(&s.id) == fam && u.tier_unlocked(cat, s)).map(|s| s.tier).max().unwrap_or(1);
            let name = cat.tier_one(&fam).map(|s| s.name.clone()).unwrap_or(fam);
            ui.p.text_fit(p.x as f32 + 5.0, yy, &format!("{name}  (rank {rank})"), SMALL, (p.w - 10) as f32, TEXT);
            yy += 8.5;
        }
        // -- Dummy
        let p = panels[3];
        match cat.units.get("training_dummy") {
            None => {
                ui.p.text(p.x as f32 + 5.0, p.y as f32 + 18.0, "No training_dummy unit in the catalog.", SMALL, FOE);
            }
            Some(dd) => {
                let dummy = build_unit(cat, dd, 0);
                ui.p.text(p.x as f32 + 5.0, p.y as f32 + 16.0, &format!("{}  HP {}   Armor {}", dummy.name, dummy.max_hp, damage::armor_rating(&dummy)), SMALL, TEXT);
                let mut rows: Vec<(String, i32)> = vec![("Main strike".into(), damage::compute(&u, &dummy, HitInput::default()).final_dmg)];
                if u.inventory.offhand_weapon().is_some() {
                    rows.push(("Off-hand strike".into(), damage::compute(&u, &dummy, HitInput { offhand: true, ..Default::default() }).final_dmg));
                }
                rows.push(("Ranged shot".into(), damage::compute(&u, &dummy, HitInput { ranged: true, ..Default::default() }).final_dmg));
                for fam in u.active_families() {
                    let Some(sk) = cat.skills.iter().filter(|s| cat.family_of(&s.id) == fam && u.tier_unlocked(cat, s)).last() else { continue };
                    if sk.kind == "ATTACK" {
                        let hit = damage::compute(&u, &dummy, HitInput { ranged: sk.reach_max > 1, skill_mult: u.power_for(sk), ..Default::default() });
                        rows.push((format!("{} ({} AP)", sk.name, sk.level), hit.final_dmg));
                    }
                }
                for (i, (label, dmg)) in rows.iter().enumerate() {
                    let y = p.y as f32 + 30.0 + i as f32 * 9.5;
                    if y > (p.y + p.h) as f32 - 12.0 {
                        break;
                    }
                    ui.p.text(p.x as f32 + 5.0, y, label, SMALL, TEXT);
                    ui.p.text_right((p.x + p.w) as f32 - 6.0, y, &format!("{dmg} dmg"), SMALL, GOLD);
                }
                ui.push("dojo.dummy", "panel", "damage vs dummy", p, true, false, None, tip("Damage against the training dummy", "No crit, neutral stances; skills at their highest unlocked rank."));
            }
        }
        self.w = ui.widgets;
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({"unit": self.unit, "level": self.level, "gear": self.gear.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<String, Value>>()})
    }

    widget_plumbing!();
}
