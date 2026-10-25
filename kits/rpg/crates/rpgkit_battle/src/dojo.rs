//! The gear dojo's sandbox (DojoSession, scripts/dojo/dojo_session.gd, with DojoBreakdown and DojoLoadouts): two catalog
//! heroes and a training dummy, a shared stash, and the battle's own rules applied one action at a time. Strike with a
//! hand, use any catalog skill (AP, stance switch, guard, focus, rest, statuses), let the dummy strike back, roll rounds,
//! shift stances, use the battle pouch, benchmark a skill, equip from the stash (class training, slot fit and the
//! two-handed rule as everywhere), apply loadout presets, switch the ready weapon and spend attribute points. Every
//! action is logged with a step-by-step damage breakdown (markup `[color=#rrggbb]`, `[b]`); stage effects queue for the
//! screen. Crit rolls are seeded, so benchmarks repeat.

use crate::skill;
use aegis_core::rng::GodotRng;
use rpgkit_data::{Catalog, ItemDef, SkillDef, STATS};
use rpgkit_rules::carry;
use rpgkit_rules::damage::{self, Hit, HitInput, CRIT_MULT};
use rpgkit_rules::gear::{class_can_equip, item_fits_slot, off_hand_ok, Equipment, Slot};
use rpgkit_rules::stats::Proficiency;
use rpgkit_rules::status;
use rpgkit_rules::unit::{Stance, Unit};
use rpgkit_rules::build_unit;
use indexmap::IndexMap;
use std::sync::Arc;

pub const DUMMY_MAX_HP: i32 = 500;
pub const MAX_LOG_ENTRIES: usize = 300;
/// AP regained each round, as in battles.
pub const ROUND_AP_REGEN: i32 = 4;
/// The battle AP pool.
pub const HERO_AP: i32 = 12;
pub const RNG_SEED: i64 = 1337;
/// The heroes and the dummy (catalog units).
pub const HEROES: [&str; 2] = ["dojo_warrior", "dojo_ranger"];
pub const DUMMY: &str = "training_dummy";
/// The equipment slots, in their order (slot keys as the saves write them).
pub const SLOTS: [(&str, Slot); 6] = [
    ("main_hand", Slot::MainHand),
    ("off_hand", Slot::OffHand),
    ("head", Slot::Head),
    ("body", Slot::Body),
    ("accessory_1", Slot::Accessory1),
    ("accessory_2", Slot::Accessory2),
];

/// One line of the test log (DojoLogEntry).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub n: usize,
    /// ATTACK, DEFEND, BUFF or EVENT (the log's filters).
    pub category: &'static str,
    pub title: String,
    pub detail: String,
    pub value: i32,
    pub outcome: String,
    /// The step-by-step explanation (markup), "" when there is none.
    pub breakdown: String,
    pub target: String,
    pub by_dummy: bool,
}

/// How a stage effect is coloured (UiTheme COL_*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Info,
    Good,
    Bad,
    Warn,
    Muted,
    Gold,
}

/// A stage effect (DojoSession.stage_fx): a strike flash with a number, a glow or a floating text, on the hero or the
/// dummy.
#[derive(Debug, Clone, PartialEq)]
pub struct Fx {
    pub kind: &'static str,
    pub on_hero: bool,
    pub text: String,
    pub tone: Tone,
}

/// Whether an action can be taken now and why (get_action_availability).
#[derive(Debug, Clone, PartialEq)]
pub struct Avail {
    pub enabled: bool,
    pub tip: String,
}

/// Points being spent (the interactive allocation).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Alloc {
    pub pending: IndexMap<String, i32>,
    pub free: i32,
}

/// A loadout preset (DojoLoadouts): gear per slot and pouch items, taken from the stash when there, else made.
#[derive(Debug, Clone, PartialEq)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub gear: &'static [(&'static str, &'static str)],
    pub pouch: &'static [&'static str],
}

/// DojoLoadouts.get_presets (item ids for the Godot ItemCatalog makers).
pub const PRESETS: [Preset; 8] = [
    Preset { id: "sword_board", name: "Sword & Board", desc: "1H Broadsword, Buckler, Brigandine, Ring of the Bear", gear: &[("main_hand", "wpn_broadsword"), ("off_hand", "shd_buckler"), ("body", "arm_leather"), ("accessory_1", "acc_bear_ring")], pouch: &["con_heal_potion", "con_fire_bomb"] },
    Preset { id: "claymore_tank", name: "Claymore Bruiser", desc: "2H Iron Claymore, Plate Cuirass, Ring of the Bear (Tier 1 test)", gear: &[("main_hand", "wpn_claymore"), ("body", "arm_plate"), ("accessory_1", "acc_bear_ring")], pouch: &["con_heal_potion", "con_focus_elixir"] },
    Preset { id: "claymore_backup", name: "Claymore + Backup Blade", desc: "2H Iron Claymore with an Iron Shortsword off-hand: Switch Weapon (1 block) to use either", gear: &[("main_hand", "wpn_claymore"), ("off_hand", "wpn_shortsword"), ("body", "arm_leather"), ("accessory_1", "acc_bear_ring")], pouch: &["con_heal_potion"] },
    Preset { id: "titan_deficit", name: "Titan Hammer (Deficit)", desc: "2H Titan Warhammer with Plate - stresses deficit & burden penalties", gear: &[("main_hand", "wpn_warhammer"), ("body", "arm_plate")], pouch: &[] },
    Preset { id: "dual_wield", name: "Dual Wield", desc: "Broadsword + Iron Shortsword off-hand, Brigandine, Ring of the Bear", gear: &[("main_hand", "wpn_broadsword"), ("off_hand", "wpn_shortsword"), ("body", "arm_leather"), ("accessory_1", "acc_bear_ring")], pouch: &["con_heal_potion"] },
    Preset { id: "archer", name: "Archer", desc: "2H Oak Shortbow with a Shadow Dagger backup (Switch Weapon to go melee), Brigandine, Swiftness Amulet", gear: &[("main_hand", "wpn_shortbow"), ("off_hand", "wpn_dagger"), ("body", "arm_leather"), ("accessory_2", "acc_swift_amulet")], pouch: &["con_focus_elixir", "con_heal_potion"] },
    Preset { id: "duelist", name: "Blade & Dagger", desc: "1H Broadsword + Shadow Dagger off-hand (Dual Strike), Brigandine, Swiftness Amulet", gear: &[("main_hand", "wpn_broadsword"), ("off_hand", "wpn_dagger"), ("body", "arm_leather"), ("accessory_2", "acc_swift_amulet")], pouch: &["con_heal_potion"] },
    Preset { id: "arcanist", name: "Arcanist", desc: "1H Wand with a Mystic Tome off-hand (a tome needs a one-handed main hand; the 2H staff leaves the off hand empty)", gear: &[("main_hand", "wpn_wand"), ("off_hand", "wpn_tome")], pouch: &["con_focus_elixir"] },
];

pub fn stance_label(s: Stance) -> &'static str {
    match s {
        Stance::Neutral => "Neutral",
        Stance::Offensive => "Offensive",
        Stance::Tactical => "Tactical",
        Stance::Defensive => "Defensive",
    }
}

fn slot_of(key: &str) -> Option<Slot> {
    SLOTS.iter().find(|(k, _)| *k == key).map(|(_, s)| *s)
}

fn slot_label(key: &str) -> String {
    key.split('_').map(|w| w[..1].to_uppercase() + &w[1..]).collect::<Vec<_>>().join(" ")
}

/// One hit's numbers for the log (DojoSession._apply_hit's dictionary).
#[derive(Debug, Clone, PartialEq)]
pub struct Struck {
    pub dmg: i32,
    pub crit: bool,
    pub mitigated: bool,
    pub outcome: &'static str,
    pub detail: String,
    pub breakdown: String,
}

pub struct DojoSession {
    pub cat: Arc<Catalog>,
    pub heroes: Vec<Unit>,
    /// The class of each hero (its proficiency decides what it can wear).
    pub classes: Vec<String>,
    pub active: usize,
    pub dummy: Unit,
    /// The shared stash: one entry per item (equipment and consumables), by id.
    pub stash: Vec<String>,
    pub carry: carry::Report,
    pub force_crit: bool,
    rng: GodotRng,
    pub skill: String,
    pub dummy_skill: String,
    pub dummy_counter: bool,
    pub round: i32,
    pub alloc: Option<Alloc>,
    pub log: Vec<Entry>,
    counter: usize,
    /// Stage effects not yet shown (the screen drains them).
    pub fx: Vec<Fx>,
}

impl DojoSession {
    /// DojoSession.init_data: the catalog heroes (HERO_AP) and the dummy (DUMMY_MAX_HP, unguarded), the party starter
    /// stash, the carry penalty applied, Strike selected for both sides, round 1.
    pub fn new(cat: Arc<Catalog>) -> Self {
        let mut heroes = Vec::new();
        let mut classes = Vec::new();
        for id in HEROES {
            let Some(def) = cat.units.get(id) else { continue };
            let mut u = build_unit(&cat, def, 0);
            u.max_ap = HERO_AP;
            u.ap = HERO_AP;
            heroes.push(u);
            classes.push(def.class_id.clone());
        }
        let mut dummy = cat.units.get(DUMMY).map(|d| build_unit(&cat, d, 0)).unwrap_or_else(|| build_unit(&cat, &rpgkit_data::UnitDef::from_obj(&Default::default()), 0));
        dummy.max_hp = DUMMY_MAX_HP;
        dummy.hp = DUMMY_MAX_HP;
        dummy.guard = 0.0;
        let mut stash: Vec<String> = cat.items.values().filter(|i| i.raw.get("starter_stash").and_then(|v| v.as_bool()).unwrap_or(false)).map(|i| i.id.clone()).collect();
        stash.sort();
        let weight = Self::weight_of(&cat, &stash);
        let carry = carry::apply(&cat, &mut heroes, weight);
        Self {
            cat,
            heroes,
            classes,
            active: 0,
            dummy,
            stash,
            carry,
            force_crit: false,
            rng: GodotRng::from_i64(RNG_SEED),
            skill: "strike".into(),
            dummy_skill: "strike".into(),
            dummy_counter: false,
            round: 1,
            alloc: None,
            log: Vec::new(),
            counter: 0,
            fx: Vec::new(),
        }
    }

    /// DojoSession.reset: everything back to the baseline, the log too.
    pub fn reset(&mut self) {
        *self = Self::new(self.cat.clone());
        self.log_event("EVENT", "Dojo Reset", "Heroes, stash, loadouts and dummy restored to baseline", 0, "RESET");
        self.fx("text", true, "Dojo Reset", Tone::Info);
    }

    fn weight_of(cat: &Catalog, ids: &[String]) -> f64 {
        ids.iter().filter_map(|i| cat.items.get(i)).map(|i| i.weight).sum()
    }

    /// The party carry report now (the carry bar).
    pub fn carry_now(&self) -> carry::Report {
        carry::evaluate(&self.heroes, Self::weight_of(&self.cat, &self.stash))
    }

    /// After a change to gear, stash, pouch or stats: the carry penalty applied again (DojoSession._notify).
    fn changed(&mut self) {
        let weight = Self::weight_of(&self.cat, &self.stash);
        self.carry = carry::apply(&self.cat, &mut self.heroes, weight);
    }

    pub fn hero(&self) -> &Unit {
        &self.heroes[self.active]
    }

    fn hero_mut(&mut self) -> &mut Unit {
        &mut self.heroes[self.active]
    }

    pub fn set_active(&mut self, i: usize) {
        if i < self.heroes.len() && i != self.active {
            self.alloc = None;
            self.active = i;
        }
    }

    fn fx(&mut self, kind: &'static str, on_hero: bool, text: &str, tone: Tone) {
        self.fx.push(Fx { kind, on_hero, text: text.to_string(), tone });
    }

    /// DojoSession.log_action: a new entry (the oldest drop past MAX_LOG_ENTRIES).
    #[allow(clippy::too_many_arguments)]
    pub fn log_action(&mut self, category: &'static str, title: &str, detail: &str, value: i32, outcome: &str, breakdown: &str, target: &str, by_dummy: bool) {
        self.counter += 1;
        self.log.push(Entry { n: self.counter, category, title: title.into(), detail: detail.into(), value, outcome: outcome.into(), breakdown: breakdown.into(), target: target.into(), by_dummy });
        if self.log.len() > MAX_LOG_ENTRIES {
            self.log.remove(0);
        }
    }

    fn log_event(&mut self, category: &'static str, title: &str, detail: &str, value: i32, outcome: &str) {
        self.log_action(category, title, detail, value, outcome, "", "", false);
    }

    fn fail(&mut self, title: &str, reason: &str) {
        self.log_action("EVENT", title, reason, 0, "FAILED", &format!("[color=#f87171]{reason}[/color]"), "", false);
    }

    /// Empties the log (entry numbers keep counting, as Godot's).
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn set_dummy_power(&mut self, power: i32) {
        self.dummy.power_override = power.max(0);
    }

    // ---- Strikes ------------------------------------------------------------------------------------------------

    /// get_action_availability: main strike, ranged volley, off-hand strike, dual strike and weapon switch.
    pub fn availability(&self) -> IndexMap<&'static str, Avail> {
        let inv = &self.hero().inventory;
        let (main, off, shield) = (inv.main_weapon(), inv.offhand_weapon(), inv.shield());
        let (switch_needed, off_active) = (inv.needs_weapon_switch(), inv.is_offhand_active());
        let off_offensive = off.is_some_and(|o| o.item.base_power > 0);
        let name = |e: Option<&Equipment>| e.map(|e| e.item.name.clone()).unwrap_or_else(|| "Unarmed".into());
        let a = |enabled: bool, tip: String| Avail { enabled, tip };
        let mut out = IndexMap::new();
        let busy = format!("{} is active - Switch Weapon (1 block) to draw {}", name(off), name(main));
        match main {
            None => {
                out.insert("main", a(true, "Unarmed melee strike against the Dummy".into()));
                out.insert("ranged", a(false, "Requires a ranged weapon (e.g. Shortbow)".into()));
            }
            Some(m) if m.is_ranged_weapon() => {
                out.insert("main", a(false, format!("Cannot melee with ranged weapon ({})", m.item.name)));
                out.insert("ranged", a(!off_active, if off_active { busy.clone() } else { format!("Ranged volley with {}", m.item.name) }));
            }
            Some(m) => {
                out.insert("main", a(!off_active, if off_active { busy.clone() } else { format!("Main-hand strike with {}", m.item.name) }));
                out.insert("ranged", a(false, format!("Cannot fire a volley with melee weapon ({})", m.item.name)));
            }
        }
        out.insert(
            "offhand",
            if off_offensive {
                if switch_needed && !off_active {
                    a(false, format!("2H {} is active - Switch Weapon (1 block) to ready {}", name(main), name(off)))
                } else {
                    a(true, format!("Off-hand strike with {} (scales with {})", name(off), off.map(|o| o.item.scaling_stat.clone()).unwrap_or_default()))
                }
            } else if let Some(s) = shield {
                a(false, format!("{} is a shield (mitigation only)", s.item.name))
            } else {
                a(false, "No off-hand weapon equipped".into())
            },
        );
        let can_dual = main.is_some_and(|m| m.is_melee_weapon() && !m.item.two_handed) && off_offensive;
        out.insert("dual", if can_dual { a(true, format!("Combo: strike with {} and {} in one action", name(main), name(off))) } else { a(false, "Needs a 1H melee main weapon and an offensive off-hand weapon".into()) });
        out.insert(
            "switch",
            if switch_needed {
                a(true, format!("1 block: ready {} (2H main + off-hand weapons share one active slot)", if off_active { name(main) } else { name(off) }))
            } else {
                a(false, "No switch needed: 1H weapons attack with both hands freely. Pair a 2H weapon with an off-hand weapon to use this.".into())
            },
        );
        out
    }

    /// DojoSession.perform_strike: "main", "offhand", "ranged" or "dual".
    pub fn strike(&mut self, kind: &str) {
        let (main, off, off_active, switch_needed) = {
            let inv = &self.hero().inventory;
            (inv.main_weapon().cloned(), inv.offhand_weapon().cloned(), inv.is_offhand_active(), inv.needs_weapon_switch())
        };
        let name = |e: &Option<Equipment>| e.as_ref().map(|e| e.item.name.clone()).unwrap_or_else(|| "Unarmed".into());
        let busy = format!("{} is active - Switch Weapon (1 block) to draw {}", name(&off), name(&main));
        let (title, ranged, offhand) = match kind {
            "main" => {
                if let Some(m) = main.as_ref().filter(|m| m.is_ranged_weapon()) {
                    return self.fail("Main Strike", &format!("Cannot melee while wielding ranged weapon ({})", m.item.name));
                }
                if off_active {
                    return self.fail("Main Strike", &busy);
                }
                ("Main Strike", false, false)
            }
            "offhand" => {
                let Some(o) = off.as_ref() else { return self.fail("Off-Hand Strike", "No off-hand weapon equipped") };
                if o.item.base_power == 0 {
                    return self.fail("Off-Hand Strike", &format!("{} is a shield (mitigation only, not offensive)", o.item.name));
                }
                if switch_needed && !off_active {
                    return self.fail("Off-Hand Strike", &format!("2H {} is active - Switch Weapon (1 block) to ready {}", name(&main), o.item.name));
                }
                ("Off-Hand Strike", false, true)
            }
            "ranged" => {
                if !main.as_ref().is_some_and(|m| m.is_ranged_weapon()) {
                    return self.fail("Ranged Volley", &format!("Requires a ranged weapon (wielding: {})", name(&main)));
                }
                if off_active {
                    return self.fail("Ranged Volley", &busy);
                }
                ("Ranged Volley", true, false)
            }
            "dual" => return self.dual_strike(),
            _ => return,
        };
        let hit = self.apply_hit(title, ranged, offhand, 1.0);
        let target = self.dummy.name.clone();
        self.log_action("ATTACK", title, &hit.detail, hit.dmg, hit.outcome, &hit.breakdown, &target, false);
        self.hit_fx(&hit);
    }

    /// The Dual Strike combo: both hands hit in one action.
    fn dual_strike(&mut self) {
        let (main, off) = (self.hero().inventory.main_weapon().cloned(), self.hero().inventory.offhand_weapon().cloned());
        let Some(m) = main.filter(|m| m.is_melee_weapon() && !m.item.two_handed) else { return self.fail("Dual Strike", "Dual Strike needs a 1H melee main-hand weapon") };
        let Some(o) = off.filter(|o| o.item.base_power > 0) else { return self.fail("Dual Strike", "Dual Strike needs an offensive off-hand weapon") };
        let first = self.apply_hit("Dual Strike > Main Hand", false, false, 1.0);
        let second = self.apply_hit("Dual Strike > Off-Hand", false, true, 1.0);
        let total = first.dmg + second.dmg;
        let outcome = if !self.dummy.is_alive() {
            "DEFEATED"
        } else if first.crit || second.crit {
            "CRIT"
        } else if first.mitigated {
            "MITIGATED"
        } else {
            "HIT"
        };
        let detail = format!("{} {} + {} {}", m.item.name, first.dmg, o.item.name, second.dmg);
        let breakdown = format!("{}\n\n{}\n\n[color=#4ade80][b]COMBO TOTAL: {total} damage[/b][/color]", first.breakdown, second.breakdown);
        let target = self.dummy.name.clone();
        self.log_action("ATTACK", "Dual Strike", &detail, total, outcome, &breakdown, &target, false);
        self.hit_fx(&first);
        self.hit_fx(&second);
    }

    /// DojoSession._apply_hit: one weapon hit on the dummy (the breakdown read right after, so multi-hits show the HP
    /// after each).
    fn apply_hit(&mut self, title: &str, ranged: bool, offhand: bool, mult: f64) -> Struck {
        let roll = if self.force_crit { None } else { Some(self.rng.randf()) };
        let res = damage::compute(&self.heroes[self.active], &self.dummy, HitInput { ranged, offhand, skill_mult: mult, force_crit: self.force_crit, crit_roll: roll, ..HitInput::default() });
        let dmg = self.dummy.take_damage(res.final_dmg, true);
        let mitigated = res.mit_ratio > 0.0;
        let outcome = if !self.dummy.is_alive() {
            "DEFEATED"
        } else if res.is_crit {
            "CRIT"
        } else if mitigated {
            "MITIGATED"
        } else {
            "HIT"
        };
        let weapon = if res.power.weapon.is_empty() { "Unarmed".to_string() } else { res.power.weapon.clone() };
        let breakdown = breakdown_strike(title, &self.heroes[self.active].name, &self.dummy, &res, dmg);
        Struck { dmg, crit: res.is_crit, mitigated, outcome, detail: format!("{weapon} • {} ATK", res.final_attack_power), breakdown }
    }

    fn hit_fx(&mut self, hit: &Struck) {
        let text = if hit.crit {
            format!("CRIT -{}", hit.dmg)
        } else if hit.mitigated {
            format!("-{} [MIT]", hit.dmg)
        } else {
            format!("-{}", hit.dmg)
        };
        let tone = if hit.mitigated && !hit.crit {
            Tone::Info
        } else if hit.crit {
            Tone::Warn
        } else {
            Tone::Bad
        };
        self.fx("strike", true, &text, tone);
    }

    // ---- Catalog skills (the same costs and stance rules as in battles) -----------------------------------------

    /// Skills the hero can test: every catalog Attack, Defend, Focus and Rest skill.
    pub fn usable_skills(&self) -> Vec<&SkillDef> {
        self.cat.skills.iter().filter(|s| matches!(s.kind.as_str(), "ATTACK" | "DEFEND" | "FOCUS" | "REST")).collect()
    }

    /// The attacks (the dummy's options and the benchmark's).
    pub fn attack_skills(&self) -> Vec<&SkillDef> {
        self.usable_skills().into_iter().filter(|s| s.kind == "ATTACK").collect()
    }

    fn find(&self, id: &str) -> Option<SkillDef> {
        self.cat.skills.iter().find(|s| s.id == id).cloned()
    }

    pub fn selected_skill(&self) -> Option<SkillDef> {
        self.find(&self.skill).or_else(|| self.find("strike"))
    }

    /// The selected skill when it is an attack, else Strike.
    pub fn benchmark_skill(&self) -> Option<SkillDef> {
        self.selected_skill().filter(|s| s.kind == "ATTACK").or_else(|| self.find("strike"))
    }

    pub fn select_skill(&mut self, id: &str) {
        if self.find(id).is_some() {
            self.skill = id.into();
        }
    }

    pub fn select_dummy_skill(&mut self, id: &str) {
        if self.find(id).is_some_and(|s| s.kind == "ATTACK") {
            self.dummy_skill = id.into();
        }
    }

    /// AP the hero would pay now: the skill's cost + the automatic stance switch.
    pub fn skill_ap(&self, s: &SkillDef) -> i32 {
        skill::ap_cost(s, Some(self.hero())) + skill::switch_cost(s, self.hero().stance)
    }

    /// DojoSession.use_skill: pays AP (debt allowed) and switches stance, then strikes (attack), raises a guard
    /// (defend), boosts (focus) or recovers AP (rest); the dummy may strike back.
    pub fn use_skill(&mut self, id: Option<&str>) {
        let Some(s) = id.map_or_else(|| self.selected_skill(), |i| self.find(i)) else {
            return self.fail("Skill", &format!("Unknown skill '{}'", id.unwrap_or("")));
        };
        if !self.hero().is_alive() {
            return self.fail(&s.name, &format!("{} is down - Heal Hero first", self.hero().name));
        }
        if s.kind == "ATTACK" && !self.dummy.is_alive() {
            return self.fail(&s.name, "The dummy is defeated - use Full Heal");
        }
        let cost = self.skill_ap(&s);
        let switch = skill::switch_cost(&s, self.hero().stance);
        let after = skill::stance_after(&s, self.hero().stance);
        let h = self.hero_mut();
        h.ap -= cost;
        h.set_stance(after);
        let ap_note = format!(
            "{cost} AP{}, {} left{}",
            if switch > 0 { format!(" (incl. +{switch} stance switch)") } else { String::new() },
            self.hero().ap,
            if self.hero().ap < 0 { " - AP debt!" } else { "" }
        );
        let dummy_name = self.dummy.name.clone();
        match s.kind.as_str() {
            "ATTACK" => {
                let ranged = self.hero().inventory.main_weapon().is_some_and(|m| m.is_ranged_weapon()) && !skill::is_off_hand(&s);
                let mult = self.hero().power_for(&s);
                let hit = if skill::deals_damage(&s) {
                    Some(self.apply_hit(&s.name, ranged, skill::is_off_hand(&s), mult))
                } else {
                    None
                };
                let mut detail = format!("{} • {ap_note}", hit.as_ref().map(|h| h.detail.clone()).unwrap_or_else(|| "No damage".into()));
                if skill::is_area(&s) {
                    detail += " • area: the dummy is the only target here";
                }
                if !s.status_effect.is_empty() && self.dummy.is_alive() {
                    let potency = s.status_potency + self.hero().status_potency_bonus();
                    let chance = status::apply_chance(&self.dummy, &s.status_effect, potency);
                    let source = self.hero().name.clone();
                    let landed = status::try_inflict(&mut self.dummy, &s.status_effect, potency, Some(&mut self.rng), 1, s.status_magnitude, &source);
                    detail += &format!(" • {} {} ({chance}%)", status::display_name(&s.status_effect), if landed { "next round" } else { "resisted" });
                }
                let (dmg, outcome, breakdown) = hit.as_ref().map(|h| (h.dmg, h.outcome, h.breakdown.clone())).unwrap_or((0, "STATUS", String::new()));
                self.log_action("ATTACK", &s.name, &detail, dmg, outcome, &breakdown, &dummy_name, false);
                if let Some(h) = hit {
                    self.hit_fx(&h);
                }
            }
            "DEFEND" => {
                self.hero_mut().guard = s.mitigation;
                let pct = (s.mitigation * 100.0).round() as i32;
                self.log_event("DEFEND", &s.name, &format!("Guard -{pct}% until the next round • {ap_note}"), pct, "GUARD");
                self.fx("glow", true, &format!("{} -{pct}%", s.name), Tone::Info);
            }
            "FOCUS" => {
                self.hero_mut().apply_attack_boost(s.boost, s.boost_rounds);
                self.log_event("BUFF", &s.name, &format!("Attack x{:.2} for {} rounds • {ap_note}", s.boost, s.boost_rounds), ((s.boost - 1.0) * 100.0).round() as i32, &format!("BUFF x{:.2}", s.boost));
                self.fx("glow", true, &format!("x{:.2} ATK ({} rds)", s.boost, s.boost_rounds), Tone::Gold);
            }
            _ => {
                let before = self.hero().ap;
                let h = self.hero_mut();
                h.ap = h.max_ap.min(h.ap + s.recover_ap);
                let gained = self.hero().ap - before;
                let detail = format!("+{gained} AP (now {}/{}), now {}", self.hero().ap, self.hero().max_ap, stance_label(self.hero().stance));
                self.log_event("BUFF", &s.name, &detail, gained, &format!("+{gained} AP"));
                self.fx("text", true, &format!("+{gained} AP"), Tone::Good);
            }
        }
        if self.dummy_counter && self.dummy.is_alive() && self.hero().is_alive() {
            self.dummy_attack();
        }
    }

    /// DojoSession.dummy_attack: the dummy strikes back with its skill (its power, stance and crit against the hero's
    /// guard and armor); a stunned or staggered dummy loses the action.
    pub fn dummy_attack(&mut self) {
        let s = self.find(&self.dummy_skill).or_else(|| self.find("strike"));
        let Some(s) = s else { return };
        let title = format!("Dummy {}", s.name);
        if !self.dummy.is_alive() {
            return self.fail(&title, "The dummy is defeated");
        }
        if !self.hero().is_alive() {
            return self.fail(&title, &format!("{} is already down", self.hero().name));
        }
        let hero_name = self.hero().name.clone();
        if let Some(lost) = ["stun", "stagger"].into_iter().find(|st| self.dummy.has_status(st)) {
            let shown = status::display_name(lost);
            self.log_action("EVENT", &title, &format!("The dummy is {} and loses its action this round", shown.to_lowercase()), 0, &shown.to_uppercase(), "", &hero_name, true);
            return;
        }
        // Attacking: the dummy drops its guard and enters the skill's stance.
        self.dummy.guard = 0.0;
        self.dummy.set_stance(skill::stance(&s));
        let ranged = self.dummy.attack_range > 1 && !skill::is_off_hand(&s);
        let roll = self.rng.randf();
        let res = damage::compute(&self.dummy, &self.heroes[self.active], HitInput { ranged, offhand: skill::is_off_hand(&s), skill_mult: s.power_mult, crit_roll: Some(roll), ..HitInput::default() });
        let dealt = self.hero_mut().take_damage(res.final_dmg, true);
        let outcome = if !self.hero().is_alive() {
            "DEFEATED"
        } else if res.is_crit {
            "CRIT"
        } else if res.mit_ratio > 0.0 {
            "MITIGATED"
        } else {
            "HIT"
        };
        let breakdown = breakdown_strike(&title, &self.dummy.name, self.hero(), &res, dealt);
        let detail = format!("{} ATK vs {hero_name} ({} stance)", res.final_attack_power, stance_label(self.hero().stance));
        self.log_action("ATTACK", &title, &detail, dealt, outcome, &breakdown, &hero_name, true);
        self.fx("strike", false, &if res.is_crit { format!("CRIT -{dealt}") } else { format!("-{dealt}") }, if res.is_crit { Tone::Warn } else { Tone::Bad });
    }

    pub fn heal_hero(&mut self) {
        let h = self.hero_mut();
        h.hp = h.max_hp;
        let (hp, max) = (self.hero().hp, self.hero().max_hp);
        self.log_event("EVENT", "Hero Heal", &format!("{} restored to {hp} / {max} HP", self.hero().name), max, "+HEAL");
        self.fx("text", true, &format!("{max} HP"), Tone::Good);
    }

    /// DojoSession.next_round: Focus ticks down, statuses roll over (queued ones take effect, AP debt is paid back,
    /// AP regenerates), the hero's guard drops.
    pub fn next_round(&mut self) {
        self.round += 1;
        let mut notes = Vec::new();
        for c in [&mut self.heroes[self.active], &mut self.dummy] {
            c.tick_round_buffs();
            for st in status::roll_over(c, ROUND_AP_REGEN) {
                notes.push(format!("{} {}", c.name, status::display_name(&st).to_lowercase()));
            }
        }
        self.hero_mut().guard = 0.0;
        let detail = format!(
            "Focus ticks, guard drops, +{ROUND_AP_REGEN} AP (now {}/{}){}",
            self.hero().ap,
            self.hero().max_ap,
            if notes.is_empty() { String::new() } else { format!("; {}", notes.join(", ")) }
        );
        let round = self.round;
        self.log_event("EVENT", &format!("Round {round}"), &detail, round, "ROUND");
        self.fx("text", true, &format!("Round {round}"), Tone::Info);
    }

    /// Cycles the hero's stance: Neutral -> Offensive -> Tactical -> Defensive -> Neutral.
    pub fn shift_stance(&mut self) {
        let order = [Stance::Neutral, Stance::Offensive, Stance::Tactical, Stance::Defensive];
        let at = order.iter().position(|s| *s == self.hero().stance).unwrap_or(0);
        let next = order[(at + 1) % order.len()];
        self.hero_mut().set_stance(next);
        let vs = next.matchup(self.dummy.stance);
        let label = stance_label(next);
        self.log_event("EVENT", "Stance Shift", &format!("{label} stance (x{vs:.2} vs the dummy's {})", stance_label(self.dummy.stance)), ((vs - 1.0) * 100.0).round() as i32, &label.to_uppercase());
        self.fx(if next == Stance::Neutral { "text" } else { "glow" }, true, &label.to_uppercase(), if next == Stance::Offensive { Tone::Bad } else { Tone::Muted });
    }

    /// Uses battle pouch item `index`: a potion heals the hero, a bomb hits the dummy, anything else boosts attack.
    pub fn use_pouch(&mut self, index: usize) {
        if self.hero().inventory.pouch.is_empty() {
            return self.fail("Battle Pouch", "Battle pouch is empty - load consumables in the Gear window");
        }
        let i = index.min(self.hero().inventory.pouch.len() - 1);
        let item = self.hero_mut().inventory.pouch.remove(i);
        self.changed();
        let dummy_name = self.dummy.name.clone();
        match item.consumable_type.as_str() {
            "HEALTH_POTION" => {
                let before = self.hero().hp;
                self.hero_mut().heal(item.heal);
                let healed = self.hero().hp - before;
                self.log_event("BUFF", &item.name, &format!("Restored {healed} HP"), healed, &format!("+{healed} HP"));
                self.fx("glow", true, &format!("+{healed} HP"), Tone::Good);
            }
            "THROWABLE_BOMB" => {
                let dealt = self.dummy.take_damage(item.damage, false);
                let outcome = if self.dummy.is_alive() { "HIT" } else { "DEFEATED" };
                self.log_action("ATTACK", &item.name, "Explosive damage", dealt, outcome, "", &dummy_name, false);
                self.fx("strike", true, &format!("-{dealt}"), Tone::Warn);
            }
            _ => {
                let pct = (item.buff_pct * 100.0).round() as i32;
                self.hero_mut().apply_attack_boost(1.0 + item.buff_pct, item.duration);
                self.log_event("BUFF", &item.name, &format!("+{pct}% ATK for {} rounds", item.duration), pct, &format!("BUFF +{pct}%"));
                self.fx("glow", true, "ELIXIR BOOST", Tone::Gold);
            }
        }
    }

    /// The dummy guards (Defensive stance, half damage) or drops its guard.
    pub fn toggle_dummy_defense(&mut self) {
        let name = self.dummy.name.clone();
        if self.dummy.guard > 0.0 {
            self.dummy.guard = 0.0;
            self.dummy.set_stance(Stance::Neutral);
            self.log_action("EVENT", "Dummy Stance", "Dummy neutral - hits deal full damage", 0, "NEUTRAL", "", &name, false);
            self.fx("text", false, "0% Mit", Tone::Good);
        } else {
            self.dummy.guard = 0.5;
            self.dummy.set_stance(Stance::Defensive);
            self.log_action("DEFEND", "Dummy Stance", "Dummy guarding (Defensive stance) - hits halved", 50, "MITIGATED 50%", "", &name, false);
            self.fx("text", false, "50% Mit", Tone::Info);
        }
    }

    pub fn heal_dummy(&mut self) {
        self.dummy.hp = self.dummy.max_hp;
        let (name, max) = (self.dummy.name.clone(), self.dummy.max_hp);
        self.log_action("EVENT", "Dummy Heal", &format!("{name} restored to {max} / {max} HP"), max, "+HEAL", "", &name, false);
        self.fx("text", false, &format!("{max} HP"), Tone::Good);
    }

    /// DojoSession.run_benchmark: `count` simulated swings of the benchmark skill (dummy HP untouched): min, average,
    /// max, crits, the expected damage per swing and the swings to defeat the dummy.
    pub fn run_benchmark(&mut self, count: usize) {
        let Some(s) = self.benchmark_skill() else { return };
        let count = count.max(1);
        let ranged = self.hero().inventory.main_weapon().is_some_and(|m| m.is_ranged_weapon()) && !skill::is_off_hand(&s);
        let mult = self.hero().power_for(&s);
        let input = HitInput { ranged, offhand: skill::is_off_hand(&s), skill_mult: mult, ..HitInput::default() };
        let base = damage::compute(self.hero(), &self.dummy, input);
        let crit_rate = base.crit_rate.clamp(0.0, 100.0);
        let (mut total, mut min, mut max, mut crits) = (0i64, i32::MAX, 0, 0);
        for _ in 0..count {
            let roll = self.rng.randf();
            let r = damage::compute(&self.heroes[self.active], &self.dummy, HitInput { crit_roll: Some(roll), ..input });
            total += r.final_dmg as i64;
            min = min.min(r.final_dmg);
            max = max.max(r.final_dmg);
            crits += r.is_crit as usize;
        }
        let avg = total as f64 / count as f64;
        let expected = base.final_dmg as f64 * (1.0 + crit_rate / 100.0 * (CRIT_MULT - 1.0));
        let to_kill = (self.dummy.hp as f64 / expected.max(1.0)).ceil() as i32;
        let title = format!("Benchmark x{count} {} ({})", s.name, if ranged { "Ranged" } else { "Melee" });
        let breakdown = [
            format!("[color=#fde047][b]{title}[/b][/color]  ({count} simulated swings, dummy HP untouched)"),
            format!("Per-hit (no crit): [color=#e2e8f0]{}[/color]", base.final_dmg),
            format!("Observed: min [color=#e2e8f0]{min}[/color] / avg [color=#38bdf8]{avg:.1}[/color] / max [color=#e2e8f0]{max}[/color]"),
            format!("Crits: {crits} / {count}  (rate {crit_rate:.1}%)"),
            format!("Expected damage / swing: [color=#4ade80][b]{expected:.1}[/b][/color]"),
            format!("Swings to defeat dummy ({} HP): [color=#fbbf24]{to_kill}[/color]", self.dummy.hp),
        ]
        .join("\n");
        let detail = format!("avg {avg:.1} • min {min} • max {max} • {crits} crits");
        let name = self.dummy.name.clone();
        self.log_action("ATTACK", &title, &detail, expected.round() as i32, "SIM", &breakdown, &name, false);
    }

    // ---- Gear (stash, equip, presets, pouch, weapon switch) ------------------------------------------------------

    /// ClassArchetype.can_equip_gear for the active hero.
    pub fn can_equip(&self, item: &ItemDef) -> bool {
        self.cat.classes.get(&self.classes[self.active]).is_none_or(|c| class_can_equip(&Proficiency::from_class(c), item))
    }

    /// The stash items the active hero can put in `slot` (fit, class training, the two-handed rule), one per id.
    pub fn eligible(&self, slot: &str) -> Vec<String> {
        let main = self.hero().inventory.main_weapon().map(|e| e.item.clone());
        let mut out: Vec<String> = Vec::new();
        for id in &self.stash {
            let Some(item) = self.cat.items.get(id) else { continue };
            if out.contains(id) || !item_fits_slot(item, slot) || !self.can_equip(item) {
                continue;
            }
            if slot == "off_hand" && !off_hand_ok(main.as_ref(), item) {
                continue;
            }
            out.push(id.clone());
        }
        out
    }

    /// The consumables in the stash, one per id.
    pub fn stash_consumables(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for id in &self.stash {
            if !out.contains(id) && self.cat.items.get(id).is_some_and(|i| i.item_type == "CONSUMABLE") {
                out.push(id.clone());
            }
        }
        out
    }

    fn take_from_stash(&mut self, id: &str) {
        if let Some(i) = self.stash.iter().position(|s| s == id) {
            self.stash.remove(i);
        }
    }

    fn stow(&mut self, slot: Slot) -> Option<String> {
        let eq = self.hero_mut().inventory.unequip(slot)?;
        self.stash.push(eq.item.id.clone());
        Some(eq.item.id)
    }

    /// DojoSession._equip_internal: the checks, the two-handed stow, the slot's old piece back to the stash, then the
    /// piece (from the stash when there).
    fn equip_internal(&mut self, id: &str, slot_key: &str, announce: bool) -> bool {
        let (Some(item), Some(slot)) = (self.cat.items.get(id).cloned(), slot_of(slot_key)) else { return false };
        let hero = self.hero().name.clone();
        if !self.can_equip(&item) {
            if announce {
                self.fail("Equip", &format!("{hero} cannot equip {} (class proficiency)", item.name));
            }
            return false;
        }
        if !item_fits_slot(&item, slot_key) {
            if announce {
                self.fail("Equip", &format!("{} cannot be equipped in the {} slot", item.name, slot_label(slot_key)));
            }
            return false;
        }
        if slot == Slot::OffHand {
            let main = self.hero().inventory.main_weapon().map(|e| e.item.clone());
            if !off_hand_ok(main.as_ref(), &item) {
                if announce {
                    let m = main.map(|m| m.name).unwrap_or_default();
                    if item.item_type == "WEAPON" {
                        // A focus (orb, tome): Equipment.conflicts_with_main.
                        self.fail("Equip", &format!("{} needs a one-handed main hand: 2H {m} leaves the off hand empty", item.name));
                    } else {
                        self.fail("Equip", &format!("2H {m} cannot be paired with {} - only an off-hand weapon (swap with Switch Weapon)", item.name));
                    }
                }
                return false;
            }
        }
        if slot == Slot::MainHand && item.two_handed {
            let held = self.hero().inventory.get(Slot::OffHand).map(|e| e.item.clone());
            if let Some(off) = held.filter(|o| !off_hand_ok(Some(&item), o)) {
                self.stow(Slot::OffHand);
                if announce {
                    self.log_event("EVENT", "Unequip", &format!("{} stowed: {} is 2-Handed", off.name, item.name), 0, "STASHED");
                }
            }
        }
        self.stow(slot);
        self.take_from_stash(id);
        self.hero_mut().inventory.equip(Equipment::new(&item), slot);
        if announce {
            self.log_event("EVENT", "Equip", &format!("{} > {}", item.name, slot_label(slot_key)), 0, "EQUIPPED");
        }
        true
    }

    pub fn equip(&mut self, id: &str, slot: &str) -> bool {
        let ok = self.equip_internal(id, slot, true);
        if ok {
            self.changed();
        }
        ok
    }

    pub fn unequip(&mut self, slot_key: &str) {
        let Some(slot) = slot_of(slot_key) else { return };
        if let Some(id) = self.stow(slot) {
            let name = self.cat.items.get(&id).map(|i| i.name.clone()).unwrap_or(id);
            self.log_event("EVENT", "Unequip", &format!("{name} returned to stash"), 0, "STASHED");
        }
        self.changed();
    }

    fn stow_all(&mut self) {
        for (_, slot) in SLOTS {
            self.stow(slot);
        }
        let pouch: Vec<String> = self.hero_mut().inventory.pouch.drain(..).map(|i| i.id).collect();
        self.stash.extend(pouch);
    }

    pub fn clear_gear(&mut self) {
        self.stow_all();
        self.changed();
        self.log_event("EVENT", "Clear Gear", "All gear and pouch items returned to stash", 0, "STASHED");
    }

    /// DojoSession.apply_loadout: everything to the stash, then the preset's pieces (from the stash, else made) and
    /// pouch; pieces the hero can't use are skipped and named.
    pub fn apply_preset(&mut self, id: &str) {
        let Some(p) = PRESETS.iter().find(|p| p.id == id) else { return };
        self.stow_all();
        let (mut equipped, mut skipped) = (0, Vec::new());
        for (slot, item) in p.gear {
            // A piece not in the stash is made for the hero (and dropped when it is skipped).
            if self.equip_internal(item, slot, false) {
                equipped += 1;
            } else {
                skipped.push(self.cat.items.get(*item).map(|i| i.name.clone()).unwrap_or_else(|| item.to_string()));
            }
        }
        for item in p.pouch {
            let Some(def) = self.cat.items.get(*item).cloned() else { continue };
            self.take_from_stash(item);
            self.hero_mut().inventory.add_to_pouch(&def);
        }
        self.changed();
        let mut detail = format!("{}: {equipped} pieces equipped", p.name);
        let mut breakdown = format!("[color=#4ade80]Applied loadout [b]{}[/b][/color]\n{}", p.name, p.desc);
        if !skipped.is_empty() {
            detail += &format!(" • skipped {}", skipped.join(", "));
            breakdown += &format!("\n[color=#f87171]Skipped (not eligible for {}): {}[/color]", self.hero().name, skipped.join(", "));
        }
        let target = self.hero().name.clone();
        self.log_action("EVENT", "Quick Setup", &detail, equipped, "LOADOUT", &breakdown, &target, false);
        self.fx("glow", true, p.name, Tone::Good);
    }

    pub fn add_to_pouch(&mut self, id: &str) -> bool {
        let Some(def) = self.cat.items.get(id).cloned().filter(|i| i.item_type == "CONSUMABLE") else { return false };
        if !self.stash.iter().any(|s| s == id) {
            return false;
        }
        if !self.hero_mut().inventory.add_to_pouch(&def) {
            self.fail("Battle Pouch", "Battle pouch is full (3 slots)");
            return false;
        }
        self.take_from_stash(id);
        self.changed();
        self.log_event("EVENT", "Load Pouch", &format!("{} loaded into battle pouch", def.name), 0, "LOADED");
        true
    }

    pub fn remove_from_pouch(&mut self, index: usize) {
        if index >= self.hero().inventory.pouch.len() {
            return;
        }
        let item = self.hero_mut().inventory.pouch.remove(index);
        self.stash.push(item.id.clone());
        self.changed();
        self.log_event("EVENT", "Unload Pouch", &format!("{} returned to stash", item.name), 0, "STASHED");
    }

    /// Switch Weapon (1 block): readies the other weapon of a 2H main + off-hand weapon pair.
    pub fn switch_weapon(&mut self) {
        if !self.hero_mut().inventory.toggle_active_weapon() {
            return self.fail("Switch Weapon", "Nothing to switch: pair a 2H weapon with an off-hand weapon (1H weapons attack with both hands freely)");
        }
        let ready = self.hero().inventory.active_weapon().map(|e| e.item.name.clone()).unwrap_or_default();
        let breakdown = format!("[color=#38bdf8][b]Switch Weapon[/b][/color]  costs 1 block.\nActive weapon is now [b]{ready}[/b]; the other weapon is sheathed until you switch back.");
        let target = self.hero().name.clone();
        self.log_action("EVENT", "Switch Weapon", &format!("1 block • {ready} now active"), 1, "SWITCHED", &breakdown, &target, false);
        self.fx("text", true, &ready, Tone::Info);
    }

    // ---- Attribute points ---------------------------------------------------------------------------------------

    /// Starts spending the hero's unspent attribute points (or cancels a session already open).
    pub fn begin_allocation(&mut self) {
        if self.alloc.take().is_some() {
            return;
        }
        let free = self.hero().stats.unallocated;
        if free <= 0 {
            return self.log_event("EVENT", "Allocate Stats", "No unspent attribute points available", 0, "NONE");
        }
        self.alloc = Some(Alloc { pending: STATS.iter().map(|s| (s.to_string(), 0)).collect(), free });
    }

    pub fn allocate(&mut self, stat: &str) {
        if let Some(a) = self.alloc.as_mut().filter(|a| a.free > 0 && a.pending.contains_key(stat)) {
            a.free -= 1;
            *a.pending.get_mut(stat).unwrap() += 1;
        }
    }

    /// Takes back a point spent in this session (never below the baseline).
    pub fn deallocate(&mut self, stat: &str) {
        if let Some(a) = self.alloc.as_mut().filter(|a| a.pending.get(stat).is_some_and(|p| *p > 0)) {
            a.free += 1;
            *a.pending.get_mut(stat).unwrap() -= 1;
        }
    }

    pub fn reset_allocation(&mut self) {
        if self.alloc.is_some() {
            let free = self.hero().stats.unallocated;
            self.alloc = Some(Alloc { pending: STATS.iter().map(|s| (s.to_string(), 0)).collect(), free });
        }
    }

    /// Adds the pending points to the hero's stats.
    pub fn confirm_allocation(&mut self) {
        let Some(a) = self.alloc.take() else { return };
        let mut summary = Vec::new();
        let h = self.hero_mut();
        for (st, pts) in a.pending.iter().filter(|(_, p)| **p > 0) {
            *h.stats.core.entry(st.clone()).or_insert(0) += pts;
            summary.push(format!("+{pts} {st}"));
        }
        h.stats.unallocated = a.free;
        self.changed();
        let detail = if summary.is_empty() { "No points spent".to_string() } else { summary.join(", ") };
        let left = self.hero().stats.unallocated;
        self.log_event("EVENT", "Allocate Stats", &format!("{detail} (remaining {left})"), 0, "CONFIRMED");
    }

    pub fn cancel_allocation(&mut self) {
        self.alloc = None;
    }
}

/// DojoBreakdown.build_strike: one hit explained step by step (markup).
pub fn breakdown_strike(title: &str, attacker: &str, target: &Unit, res: &Hit, actual: i32) -> String {
    let p = &res.power;
    let weapon = if p.weapon.is_empty() { "Unarmed" } else { &p.weapon };
    let mut lines = vec![format!("[color=#fde047][b]{title}[/b][/color]  {attacker} > {}  ({weapon})", target.name)];
    if p.authored {
        lines.push(format!("1-3. Attack Power: [color=#38bdf8][b]{}[/b][/color] (authored strike power)", p.attack_power));
    } else if p.stat_name == "STR+DEX" {
        lines.push(if p.trained_fists { format!("1. Weapon Power: {} (trained fists)", p.total_w_power) } else { "1. Weapon Power: 0 (unarmed)".into() });
        lines.push(format!("2. Unit Power: STR + DEX = [color=#38bdf8]{}[/color]", p.stat_val));
        lines.push(format!("3. Attack Power: [color=#e2e8f0]{}[/color]", p.attack_power));
    } else {
        if p.weapon_fits {
            let mut parts = vec![format!("Base {}", p.base_w_power)];
            if p.tier_boost > 0 {
                parts.push(format!("[color=#fbbf24]Tier +{}[/color]", p.tier_boost));
            }
            if p.training_pwr > 0 {
                parts.push(format!("[color=#a855f7]Training +{}[/color]", p.training_pwr));
            }
            lines.push(format!("1. Weapon Power: [color=#e2e8f0]{}[/color] ({})", p.total_w_power, parts.join(", ")));
            if p.pen_ratio > 0.0 {
                lines.push(format!("   [color=#f87171]Deficit -{}: {} x (1 - {}%) = {:.1}[/color]", p.def_amount, p.total_w_power, (p.pen_ratio * 100.0).round(), p.weapon_power));
            }
        } else {
            lines.push("1. Weapon Power: 0 (this weapon doesn't suit the attack)".into());
        }
        lines.push(format!("2. Unit Power: {} {} x {:.1} = [color=#38bdf8]+{:.1}[/color]", p.stat_val, p.stat_name, p.unit_scaling, p.unit_power));
        lines.push(format!("3. Attack Power: {:.1} + {:.1} = [color=#e2e8f0]{}[/color]", p.weapon_power, p.unit_power, p.attack_power));
    }
    if res.buff_mult != 1.0 {
        lines.push(format!("   [color=#c084fc]Focus x{:.2} > [b]{} ATK[/b][/color]", res.buff_mult, res.final_attack_power));
    }
    let mut mult = Vec::new();
    if res.skill_mult != 1.0 {
        mult.push(format!("Skill x{:.2}", res.skill_mult));
    }
    mult.push(format!("Guard x{:.2} ({}%)", res.mit_mult, (res.mit_ratio * 100.0).round()));
    mult.push(format!("Armor {} x{:.2}", res.armor_rating, 1.0 - res.armor_ratio));
    mult.push(if res.stance_factor != 1.0 { format!("[color=#fbbf24]Stance x{:.2}[/color]", res.stance_factor) } else { "Stance x1.00".into() });
    mult.push(if res.is_crit { format!("[color=#f43f5e]CRIT x{:.1}[/color]", res.crit_mult) } else { format!("Crit no ({:.1}% chance)", res.crit_rate) });
    lines.push(format!("4. Multipliers: {}", mult.join(" | ")));
    lines.push(format!("[color=#4ade80][b]RESULT: {actual} damage[/b][/color]  ({} HP {} / {})", target.name, target.hp, target.max_hp));
    lines.join("\n")
}
