//! The gear dojo's sandbox (DojoSession, scripts/dojo/dojo_session.gd, with DojoBreakdown and DojoLoadouts): two catalog
//! heroes and a training dummy, a shared stash, and the battle's own rules applied one action at a time. Strike with a
//! hand, use any catalog skill (AP, stance switch, guard, focus, rest, statuses), let the dummy strike back, roll rounds,
//! shift stances, use the battle pouch, benchmark a skill, equip from the stash (class training, slot fit and the
//! two-handed rule as everywhere), apply loadout presets, switch the ready weapon and spend attribute points. Every
//! action is logged with a step-by-step damage breakdown (markup `[color=#rrggbb]`, `[b]`); stage effects queue for the
//! screen. Crit rolls are seeded, so benchmarks repeat.

use crate::grid::V2;
use crate::poise::{self, BreakCause, Factors, Impact, PoiseState, Verdict};
use crate::round_intent::{self, ManeuverKind, Opener, RoundIntent};
use crate::round_intent_plan as rip;
use crate::session::{EncounterDef, Prebuilt, Session};
use crate::skill;
use crate::world::{AiProfile, UnitId};
use aegis_core::rng::GodotRng;
use rpgkit_data::{Catalog, Deck, ItemDef, SkillDef, UnitDef, STATS};
use rpgkit_rules::{build_from, UnitBuild};
use rpgkit_rules::carry;
use rpgkit_rules::damage::{self, Hit, HitInput, CRIT_MULT};
use rpgkit_rules::gear::{item_fits_slot, off_hand_ok, trained_for, Equipment, Slot};
use rpgkit_rules::ruleset::{self, Ruleset};
use rpgkit_rules::stats::Proficiency;
use rpgkit_rules::status;
use rpgkit_rules::unit::{DeckRule, Stance, Unit};
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
/// RULESET1_INTENT UI4: the planner's scratch board (the battle grid): the hero's, the dummy's and the second
/// dummy's first tiles, and the round's blocks.
pub const PLAN_HERO: V2 = V2::new(3, 4);
pub const PLAN_DUMMY: V2 = V2::new(6, 4);
pub const PLAN_PARTNER: V2 = V2::new(7, 4);
pub const PLAN_BLOCKS: i32 = 6;
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

/// RULESET1_INTENT UI4: what the Dojo's planner view plans for: the hero's intent, Maneuver kind and opener against
/// the dummy, and the tiles of the hero, the dummy and the second dummy that engages it (None: off the board).
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSetup {
    pub intent: RoundIntent,
    /// None = the planner's pick (MV11).
    pub kind: Option<ManeuverKind>,
    /// A skill id (None = the planner's opener, PL3).
    pub opener: Option<String>,
    pub hero: V2,
    pub dummy: V2,
    pub partner: Option<V2>,
}

impl Default for PlanSetup {
    fn default() -> Self {
        Self { intent: RoundIntent::Assault, kind: None, opener: None, hero: PLAN_HERO, dummy: PLAN_DUMMY, partner: None }
    }
}

/// One planned command (a compiled plan entry) with its reason tag (PL6) and readouts.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRow {
    pub skill: String,
    pub name: String,
    pub start: i32,
    pub end: i32,
    pub ap: i32,
    pub reason: String,
    /// The hero's tile after it.
    pub cell: V2,
    /// The planner's damage estimate on the dummy (0 for no hit).
    pub damage: i32,
    /// "flank", "armour pierced", "stagger 12" (the rules that apply to this hit).
    pub notes: Vec<String>,
}

/// The plan an intent makes now on the scratch board, or why it is not offered (PL7).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanView {
    pub intent: String,
    /// The Maneuver kind the plan uses ("" for the other intents).
    pub kind: String,
    pub target: String,
    pub rows: Vec<PlanRow>,
    /// PL7: the reason the intent (or kind) is refused ("" = planned).
    pub refused: String,
    /// The AP the scratch session plans with (its round-1 AP; the hero's AP now when the plan is refused before it).
    pub ap_before: i32,
    pub ap_after: i32,
    pub rest_blocks: i32,
    /// RS2, RS3: the AP and poise the Rest blocks would give.
    pub rest_ap: i32,
    pub rest_poise: i32,
    /// S2.2-S3 (FB-71): the pips the Rest blocks would restore under `poise_mode` pips (0 under `bar`).
    pub rest_pips: i32,
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
    /// RULESET1-R: the rules the dojo applies (the Classic / Timeline toggle; classic by default). Under
    /// `round_intent` the classic stances are inert: no switch AP, no stance change, no stance shift.
    pub ruleset: Ruleset,
    /// RULESET1-G: each hero's catalog unit id (rebuilds start from it).
    pub hero_ids: Vec<String>,
    /// The attribute points spent on each hero here (a rebuild spends them again).
    spent: Vec<IndexMap<String, i32>>,
    /// Each hero's deck set here (None = its catalog row's; RULESET1 9 D5).
    decks: Vec<Option<Deck>>,
    /// RULESET1 5 (rule `poise`): each hero's and the dummy's poise bar (refilled each round).
    pub hero_poise: Vec<PoiseState>,
    pub dummy_poise: PoiseState,
    /// "Dummy in windup": hits on the dummy count as hits into a windup (P5, P13, P14).
    pub dummy_windup: bool,
    /// RS2: the hero's Rest blocks this round (their AP comes at "Next round" under `round_intent`).
    pub rest_blocks: i32,
    /// RULESET1_INTENT UI4: the planner view's inputs.
    pub planner: PlanSetup,
}

impl DojoSession {
    /// DojoSession.init_data: the catalog heroes (HERO_AP) and the dummy (DUMMY_MAX_HP, unguarded), the party starter
    /// stash, the carry penalty applied, Strike selected for both sides, round 1.
    pub fn new(cat: Arc<Catalog>) -> Self {
        let mut heroes = Vec::new();
        let mut classes = Vec::new();
        let mut hero_ids = Vec::new();
        // CLN-44: the Dojo starts on the catalog's classic row; AUDIT-03: its heroes are built under its wield terms
        let ruleset = Ruleset::of_or_classic(&cat, ruleset::CLASSIC);
        for id in HEROES {
            let Some(def) = cat.units.get(id) else { continue };
            let mut u = build_from(&cat, UnitBuild::row(def, 0, None).with_wield(ruleset.numbers.wield())).unit;
            u.max_ap = HERO_AP;
            u.ap = HERO_AP;
            heroes.push(u);
            classes.push(def.class_id.clone());
            hero_ids.push(id.to_string());
        }
        let n = heroes.len();
        // AUDIT-12: the dummy under the same wield terms as the heroes
        let wield = ruleset.numbers.wield();
        let mut dummy = cat.units.get(DUMMY).map(|d| build_from(&cat, UnitBuild::row(d, 0, None).with_wield(wield)).unit).unwrap_or_else(|| build_from(&cat, UnitBuild::row(&UnitDef::from_obj(&Default::default()), 0, None).with_wield(wield)).unit);
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
            ruleset,
            hero_ids,
            spent: vec![IndexMap::new(); n],
            decks: vec![None; n],
            hero_poise: vec![PoiseState::default(); n],
            dummy_poise: PoiseState::default(),
            dummy_windup: false,
            rest_blocks: 0,
            planner: PlanSetup::default(),
        }
    }

    /// DojoSession.reset: everything back to the baseline, the log too.
    pub fn reset(&mut self) {
        let ruleset = self.ruleset.clone();
        *self = Self::new(self.cat.clone());
        self.ruleset = ruleset;
        // the heroes are rebuilt where the ruleset has a deck (CLN-46: the rule, not the row's name)
        if self.ruleset.has(ruleset::DECK) {
            self.rebuild_all();
        }
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

    /// Switches the hero the Dojo works on. The Rest blocks of this round belong to the hero that rested: the next
    /// hero starts with none, so their AP is lost (RS2 pays at "Next round" to the active hero only).
    pub fn set_active(&mut self, i: usize) {
        if i < self.heroes.len() && i != self.active {
            self.alloc = None;
            self.active = i;
            self.rest_blocks = 0;
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

    // ---- RULESET1_INTENT UI4: the planner view (no resolver: plans compile on a scratch board) ------------------

    /// The intents the Dojo's ruleset offers (none without `round_intent`; A4.2-K1b: Field too under `field_intent`).
    pub fn plan_intents(&self) -> Vec<RoundIntent> {
        if !self.ruleset.has(ruleset::ROUND_INTENT) {
            return Vec::new();
        }
        let mut out: Vec<RoundIntent> = self.ruleset.numbers.intents.iter().filter_map(|i| RoundIntent::parse(i)).collect();
        if self.ruleset.has(ruleset::FIELD_INTENT) && !out.contains(&RoundIntent::Field) {
            out.push(RoundIntent::Field);
        }
        out
    }

    /// A4.2-K1b (INTENT2 IR15, IR21; row W2-4 "display only the valid IR21 intent variants for the selected skill"): each
    /// intent of `plan_intents` with whether it is open now: under `field_intent` with an opener picked, only the
    /// opener's own rows (`intent::allowed`); else every intent (classic Dojo rules unchanged).
    pub fn plan_variants(&self) -> Vec<(RoundIntent, bool)> {
        let rows = self.planner.opener.as_deref().filter(|_| self.ruleset.has(ruleset::FIELD_INTENT)).and_then(|id| self.cat.skill(id)).map(|s| rpgkit_rules::intent::allowed(&self.ruleset, &rpgkit_rules::intent::IntentInput::of_def(s), &s.round_intent));
        self.plan_intents().into_iter().map(|i| (i, rows.as_ref().is_none_or(|r| r.contains(&i.id())))).collect()
    }

    /// The scratch battle the planner plans on: the active hero (as it stands: gear, AP, deck), the second dummy on
    /// its side when placed (it engages the dummy, which faces it: MV1, MV2) and the dummy on the other, on the battle
    /// grid at the setup's tiles, under the Dojo's ruleset. Err without `round_intent`.
    fn plan_session(&self, setup: &PlanSetup) -> Result<(Session, UnitId), String> {
        if !self.ruleset.has(ruleset::ROUND_INTENT) {
            return Err(format!("{} rules have no round intents", self.ruleset.name));
        }
        let Some(dummy_def) = self.cat.units.get(DUMMY) else { return Err(format!("no '{DUMMY}' unit in the catalog")) };
        let hero_def = self.hero_def(self.active).unwrap_or_else(|| UnitDef::from_obj(&Default::default()));
        let row = serde_json::json!({"id": "dojo_planner", "name": "Gear Dojo", "party": [], "enemies": [DUMMY], "total_blocks": PLAN_BLOCKS, "unit_ap": HERO_AP, "max_rounds": 1});
        let def = EncounterDef::from_obj_with(row.as_object().expect("an object"), self.ruleset.clone());
        let prebuilt = |unit: Unit, d: &UnitDef, key: &str| Prebuilt { unit, art: String::new(), palette: String::new(), profile: AiProfile::for_unit(&self.cat, d), key: key.into(), default_intent: d.default_intent.clone() };
        let mut party = vec![prebuilt(self.hero().clone(), &hero_def, "hero")];
        if setup.partner.is_some() {
            let mut partner = build_from(&self.cat, UnitBuild::row(dummy_def, 0, self.deck_rule().as_ref()).with_wield(self.ruleset.numbers.wield())).unit;
            partner.name = format!("{} 2", self.dummy.name);
            party.push(prebuilt(partner, dummy_def, "partner"));
        }
        let mut s = Session::start_with(self.cat.clone(), def, &IndexMap::<String, UnitDef>::new(), party);
        let dummy = s.w.enemies.first().copied().ok_or("the dummy did not spawn")?;
        // the dummy as it stands here (HP, guard, statuses)
        let (name, guard, hp) = (self.dummy.name.clone(), self.dummy.guard, self.dummy.hp);
        let d = &mut s.w.units[dummy];
        (d.name, d.guard, d.hp) = (name, guard, hp);
        for c in 0..s.w.units.len() {
            s.w.grid.positions[c] = None;
        }
        s.w.grid.positions[0] = Some(setup.hero);
        s.w.grid.positions[dummy] = Some(setup.dummy);
        if let Some(cell) = setup.partner {
            s.w.grid.positions[1] = Some(cell);
            s.w.units[dummy].active_target = Some(1);
        } else {
            s.w.units[dummy].active_target = Some(0);
        }
        round_intent::update_facings(&mut s.w);
        Ok((s, dummy))
    }

    /// The plan `setup` makes for the active hero now (PL1-PL7), with each command's reason tag (PL6), AP before and
    /// after, the Rest gain (RS2, RS3) and each hit's estimate and rules (flank, pierce, stagger). Nothing changes.
    pub fn plan_view_of(&self, setup: &PlanSetup) -> PlanView {
        let mut v = PlanView { intent: setup.intent.name().to_string(), ap_before: self.hero().ap, ap_after: self.hero().ap, ..PlanView::default() };
        let (mut s, dummy) = match self.plan_session(setup) {
            Ok(x) => x,
            Err(e) => {
                v.refused = e;
                return v;
            }
        };
        let c: UnitId = 0;
        // the AP the scratch session plans with (its round 1), not the Dojo hero's AP now
        (v.ap_before, v.ap_after) = (s.w.units[c].ap, s.w.units[c].ap);
        let w = &mut s.w;
        if let Err(e) = round_intent::set_intent(w, c, setup.intent) {
            v.refused = e;
            return v;
        }
        if setup.intent == RoundIntent::Maneuver {
            if let Err(e) = rip::set_kind(w, c, setup.kind) {
                v.refused = e;
                return v;
            }
        }
        if let Some(sid) = setup.opener.as_deref().and_then(|id| w.book().find(id)) {
            if let Err(e) = rip::set_opener(w, c, Some(Opener::Skill(sid))) {
                v.refused = e;
                return v;
            }
        }
        if let Err(e) = rip::plan_unit(w, &mut s.plan, c) {
            v.refused = e;
            return v;
        }
        let w = &s.w;
        let rs = w.round_states.get(&c).cloned().unwrap_or_default();
        v.kind = match (setup.intent, rs.maneuver) {
            (RoundIntent::Maneuver, Some(ManeuverKind::Engage)) => "Engage".into(),
            (RoundIntent::Maneuver, Some(ManeuverKind::Disengage)) => "Disengage".into(),
            (RoundIntent::Maneuver, Some(ManeuverKind::Utility)) => "Utility".into(),
            _ => String::new(),
        };
        v.target = match rs.target {
            Some(round_intent::RoundTarget::Myself) => "self".into(),
            Some(round_intent::RoundTarget::Unit(t)) => w.units[t].name.clone(),
            Some(round_intent::RoundTarget::Cell(p)) => format!("tile {},{}", p.x, p.y),
            None => String::new(),
        };
        let entries = s.plan.entries_for(w, c);
        let reasons = rip::reasons(w, &s.plan, c);
        for (e, reason) in entries.iter().zip(reasons) {
            let sk = &w.cat.skills[e.skill];
            let mut notes = Vec::new();
            if skill::kind(sk) == skill::Kind::Attack && e.target == Some(dummy) {
                let mut w2 = w.clone();
                w2.grid.positions[c] = Some(e.pos_after);
                if w.ruleset.has(ruleset::FLANK) && round_intent::flank_bonus(&w2, c, dummy) {
                    notes.push("flank".to_string());
                }
                let imp = Impact::of(sk);
                if imp.armor_piercing && w.ruleset.has(ruleset::PIERCE) {
                    notes.push("armour pierced".to_string());
                }
                if imp.stagger > 0 && w.ruleset.has(ruleset::POISE) {
                    notes.push(format!("stagger {}", imp.stagger));
                }
            }
            v.ap_after -= e.total_ap();
            v.rows.push(PlanRow { skill: sk.id.clone(), name: sk.name.clone(), start: e.start_block, end: e.end_block(w), ap: e.total_ap(), reason, cell: e.pos_after, damage: e.est_damage, notes });
        }
        let rest = rip::rest_preview(w, &s.plan, c);
        (v.rest_blocks, v.rest_ap, v.rest_poise, v.rest_pips) = (rest.blocks, rest.ap, rest.poise, rest.pips);
        v
    }

    /// The planner view of the Dojo's setup.
    pub fn plan_view(&self) -> PlanView {
        self.plan_view_of(&self.planner)
    }

    /// UI4: the plan each intent of the ruleset would make now (its kind and opener the planner's). A4.2-K1b (the skill
    /// variant previews): with an opener picked, one plan per row of that skill, each opening with it.
    pub fn plans_by_intent(&self) -> Vec<PlanView> {
        let opener = self.planner.opener.clone().filter(|_| self.ruleset.has(ruleset::FIELD_INTENT));
        self.plan_variants().into_iter().filter(|v| v.1).map(|(i, _)| self.plan_view_of(&PlanSetup { intent: i, kind: None, opener: opener.clone(), ..self.planner.clone() })).collect()
    }

    /// The openers the hero may pick for the setup's intent: its order for the intent (FU4), as (skill id, name).
    pub fn plan_openers(&self) -> Vec<(String, String)> {
        let setup = PlanSetup { opener: None, ..self.planner.clone() };
        let Ok((mut s, _)) = self.plan_session(&setup) else { return Vec::new() };
        if round_intent::set_intent(&mut s.w, 0, setup.intent).is_err() {
            return Vec::new();
        }
        let mut out: Vec<skill::SkillId> = rip::order(&s.w, 0, setup.intent);
        // A4.2-K1b (IR15): a skill's second row opens by pick: the hero's skills of that row the order leaves out
        if round_intent::field_rows(&s.w) {
            for sid in rip::hotkey_skills(&s.w, 0) {
                if !out.contains(&sid) && round_intent::skill_rows(&s.w, &s.w.cat.skills[sid]).contains(&setup.intent.id()) {
                    out.push(sid);
                }
            }
        }
        out.into_iter().map(|sid| (s.w.cat.skills[sid].id.clone(), s.w.cat.skills[sid].name.clone())).collect()
    }

    /// Sets the planner's intent (the kind and opener go back to the planner's pick; A4.2-K1b: an opener with that row
    /// stays, so switching between a skill's variants keeps the skill).
    pub fn set_plan_intent(&mut self, intent: RoundIntent) {
        let keep = self.plan_variants().iter().any(|v| v.0 == intent && v.1) && self.ruleset.has(ruleset::FIELD_INTENT);
        self.planner.intent = intent;
        self.planner.kind = None;
        if !keep {
            self.planner.opener = None;
        }
    }

    /// Moves a planner piece ("hero", "dummy", "partner") to `cell` when it is on the grid and free; "partner" with
    /// None takes the second dummy off the board.
    pub fn place_plan(&mut self, piece: &str, cell: Option<V2>) -> bool {
        let p = &mut self.planner;
        let on_grid = |c: V2| c.x >= 0 && c.y >= 0 && c.x < crate::grid::GRID_W && c.y < crate::grid::GRID_H;
        if let Some(c) = cell {
            let taken = [Some(p.hero), Some(p.dummy), p.partner].into_iter().flatten().any(|o| o == c);
            if !on_grid(c) || taken {
                return false;
            }
        }
        match (piece, cell) {
            ("hero", Some(c)) => p.hero = c,
            ("dummy", Some(c)) => p.dummy = c,
            ("partner", c) => p.partner = c,
            _ => return false,
        }
        true
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
        self.apply_hit_by(title, ranged, offhand, mult, false)
    }

    /// `apply_hit`; `pierce`: the hit skips the armour term (RULESET1 A2).
    fn apply_hit_by(&mut self, title: &str, ranged: bool, offhand: bool, mult: f64, pierce: bool) -> Struck {
        let roll = if self.force_crit { None } else { Some(self.rng.randf()) };
        let res = damage::compute(&self.heroes[self.active], &self.dummy, HitInput { ranged, offhand, skill_mult: mult, force_crit: self.force_crit, crit_roll: roll, ignore_armor: pierce, ..HitInput::default() });
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

    /// RULESET1-R: plays by `ruleset` from now on (the toggle); the log says which.
    pub fn set_ruleset(&mut self, ruleset: Ruleset) {
        if ruleset == self.ruleset {
            return;
        }
        self.ruleset = ruleset;
        // RULESET1-G: the heroes are built again under the new rules (the deck rule's slots and basics; gear, pouch and
        // spent points kept), the poise bars refill
        self.rebuild_all();
        let name = self.ruleset.name.clone();
        let mut detail = if self.ruleset.stances() { format!("{name} rules: stances and their switch AP apply") } else { format!("{name} rules: no stances, no switch AP") };
        if self.deck_rule().is_some() {
            detail += "; heroes rebuilt with their decks";
        }
        self.log_event("EVENT", "Ruleset", &detail, 0, &name.to_uppercase());
    }

    // ---- RULESET1-G: rebuilds, decks, poise ---------------------------------------------------------------------

    /// The deck rule of the Dojo's ruleset (None in classic).
    pub fn deck_rule(&self) -> Option<DeckRule> {
        DeckRule::of(&self.ruleset)
    }

    /// Whether the Dojo plays by poise (rule `poise`).
    pub fn poise_on(&self) -> bool {
        self.ruleset.has(ruleset::POISE)
    }

    /// Hero `i`'s catalog row with the deck set here (its own when none).
    fn hero_def(&self, i: usize) -> Option<UnitDef> {
        let mut def = self.cat.units.get(self.hero_ids.get(i)?)?.clone();
        if let Some(d) = &self.decks[i] {
            def.deck = d.clone();
        }
        Some(def)
    }

    /// Builds hero `i` again from its catalog row under the Dojo's ruleset (`build_from`, the row alone: under the deck
    /// rule its deck's slotted passives, open basics and active slots; classic: `build_unit`), then puts back its gear,
    /// pouch and the points spent here. HP and AP refill; the poise bar refills.
    fn rebuild(&mut self, i: usize) {
        let Some(def) = self.hero_def(i) else { return };
        let rule = self.deck_rule();
        // U3.2-S3: under the Dojo ruleset's wield terms
        let mut u = build_from(&self.cat, UnitBuild { wield: self.ruleset.numbers.wield(), ..UnitBuild::row(&def, 0, rule.as_ref()) }).unit;
        u.inventory = self.heroes[i].inventory.clone();
        for (st, pts) in &self.spent[i] {
            *u.stats.core.entry(st.clone()).or_insert(0) += pts;
            u.stats.unallocated -= pts;
        }
        u.max_ap = HERO_AP;
        u.ap = HERO_AP;
        self.heroes[i] = u;
        self.changed();
        let h = &mut self.heroes[i];
        h.hp = h.max_hp;
        self.hero_poise[i] = self.full_bar(&self.heroes[i], self.hero_poise[i]);
    }

    fn rebuild_all(&mut self) {
        self.alloc = None;
        for i in 0..self.heroes.len() {
            self.rebuild(i);
        }
        self.dummy_poise = self.full_bar(&self.dummy, self.dummy_poise);
        self.rest_blocks = 0;
    }

    /// The deck hero `i` plays with (its unit's `deck`: empty = the automatic deck, D5).
    pub fn deck_of(&self, i: usize) -> Deck {
        self.heroes.get(i).map(|h| h.deck.clone()).unwrap_or_default()
    }

    /// RULESET1 9 D6 (the Dojo's deck view): the active hero fights with `deck` from now on (empty = the automatic
    /// deck): it is rebuilt with it (gear, pouch and spent points kept).
    pub fn set_deck(&mut self, deck: Deck) {
        let i = self.active;
        self.decks[i] = Some(deck);
        self.alloc = None;
        self.rebuild(i);
        let h = self.hero();
        let detail = if h.deck.is_empty() {
            "Automatic deck".to_string()
        } else {
            format!("Actives: {}; passives: {}; keystone: {}", list_or_auto(&h.active_families()), list_or_none(&h.deck.passive), list_or_none(&h.deck.keystone))
        };
        let name = h.name.clone();
        self.log_action("EVENT", "Deck", &detail, 0, "DECK", "", &name, false);
    }

    /// P1-P3: `u`'s bar refilled to its Max Poise (last round's break remembered for P10); empty without the rule.
    fn full_bar(&self, u: &Unit, last: PoiseState) -> PoiseState {
        if !self.poise_on() {
            return PoiseState::default();
        }
        let max = poise::max_poise_of(&self.cat, &self.ruleset, u);
        let mut st = PoiseState { broken_last: last.broken, ..PoiseState::default() };
        poise::refill(&self.ruleset.numbers, &mut st, max);
        st
    }

    /// The active hero's poise bar.
    pub fn poise(&self) -> PoiseState {
        self.hero_poise.get(self.active).copied().unwrap_or_default()
    }

    /// "Dummy in windup" on or off.
    pub fn toggle_dummy_windup(&mut self) {
        self.dummy_windup = !self.dummy_windup;
        let name = self.dummy.name.clone();
        let detail = if self.dummy_windup { "Dummy winding up: hits stagger it more and may break it" } else { "Dummy not winding up" };
        self.log_action("EVENT", "Dummy Windup", detail, 0, if self.dummy_windup { "WINDUP" } else { "IDLE" }, "", &name, false);
    }

    /// P4-P14 on one bar for one hit (the Dojo's own sandbox, one hit at a time): stagger taken by the factors that
    /// apply, the break by poise, by `sure_break` into a windup, or by the chance roll (P13), and the line the
    /// breakdown shows (`stagger 12 x windup 2 x guard 0.5 = 12 -> poise 40 -> 28`). None without a bar.
    fn stagger(&mut self, on_dummy: bool, s: &SkillDef, guarding: bool) -> Option<String> {
        let mut bar = if on_dummy { self.dummy_poise } else { self.poise() };
        let imp = Impact::of(s);
        if !self.poise_on() || bar.max <= 0 || (imp.stagger <= 0 && !imp.sure_break) {
            return None;
        }
        let n = self.ruleset.numbers.clone();
        let target = if on_dummy { &self.dummy } else { self.hero() };
        let windup = on_dummy && self.dummy_windup;
        let pick = |on: bool, m: f64| if on { m } else { 1.0 };
        let f = Factors {
            windup: pick(windup, n.stagger_windup_mult),
            guard: pick(guarding, n.stagger_guard_mult),
            repeat: pick(bar.broken_last, n.stagger_repeat_mult),
            passive: target.stagger_taken.iter().filter(|e| e.matches(s.blocks)).map(|e| e.mult).product(),
            ..Factors::default()
        };
        let taken = poise::stagger_taken(imp.stagger, &f);
        let mut terms = vec![format!("stagger {}", imp.stagger)];
        for (label, v) in [("windup", f.windup), ("guard", f.guard), ("repeat", f.repeat), ("passive", f.passive)] {
            if v != 1.0 {
                terms.push(format!("{label} {}", num(v)));
            }
        }
        if bar.broken {
            return Some(format!("{} = {taken} (already broken this round)", terms.join(" x ")));
        }
        let before = bar.value;
        // S2.2-S2: the one stagger rule (by pips under poise_mode pips); the Dojo holds no one up (no P12)
        let settled = poise::settle(&n, &bar, taken, poise::Terms { windup, held: false, sure: imp.sure_break.then_some(BreakCause::Sure), chance: true });
        let after = settled.after;
        let mut line = format!("{} = {taken} -> poise {before} -> {after}", terms.join(" x "));
        if poise::pips_mode(&n) {
            line += &format!(" (pips {} -> {}, T {})", bar.pips, settled.pips, poise::threshold(&n, bar.max));
        }
        let cause = match settled.verdict {
            Verdict::Breaks(BreakCause::Sure) => Some("BREAK (sure break)"),
            Verdict::Breaks(_) => Some("BREAK"),
            Verdict::Chance(chance) => {
                let roll = self.rng.randi_range(1, 100);
                line += &format!(" (may break: {chance}%, rolled {roll})");
                (roll <= chance).then_some("BREAK (chance)")
            }
            Verdict::Holds | Verdict::AlreadyBroken => None,
        };
        bar.value = after;
        bar.pips = settled.pips;
        if let Some(c) = cause {
            bar.value = 0;
            bar.pips = 0;
            bar.broken = true;
            line += &format!(" {c}");
            // S2.2-S4: under pips the break queues `stagger` for next round (not on a `steadfast` unit)
            let queued = if on_dummy { poise::break_queues_stagger(&n, &mut self.dummy) } else { poise::break_queues_stagger(&n, self.hero_mut()) };
            if queued {
                line += ", staggered next round";
            }
            // AUDIT-12 (FB-94): a break ends the broken side's guard, as in battle (`poise::break_unit`)
            let side = if on_dummy { &mut self.dummy } else { self.hero_mut() };
            let guarded = side.guard > 0.0;
            if crate::trigger::end_unit_guard(side) || guarded {
                line += ", guard broken";
            }
        }
        if on_dummy {
            self.dummy_poise = bar;
        } else {
            self.hero_poise[self.active] = bar;
        }
        Some(line)
    }

    /// The stance switch AP of `s` now (0 when the ruleset's stances are inert).
    fn switch_ap(&self, s: &SkillDef) -> i32 {
        if self.ruleset.stances() {
            skill::switch_cost(s, self.hero().stance)
        } else {
            0
        }
    }

    /// AP the hero would pay now: the skill's cost + the automatic stance switch.
    pub fn skill_ap(&self, s: &SkillDef) -> i32 {
        skill::ap_cost_by(s, Some(self.hero()), &self.ruleset) + self.switch_ap(s)
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
        let switch = self.switch_ap(&s);
        let after = if self.ruleset.stances() { skill::stance_after(&s, self.hero().stance) } else { self.hero().stance };
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
                // RULESET1 A2: an armour-piercing hit under the rule pierce
                let pierce = Impact::of(&s).armor_piercing && self.ruleset.has(ruleset::PIERCE);
                let guarding = self.dummy.guard > 0.0;
                let mut hit = if skill::deals_damage(&s) {
                    Some(self.apply_hit_by(&s.name, ranged, skill::is_off_hand(&s), mult, pierce))
                } else {
                    None
                };
                // RULESET1 5: the hit's stagger on the dummy's bar (rule poise)
                if let Some(h) = hit.as_mut() {
                    if pierce {
                        h.breakdown += "\n[color=#38bdf8]armour pierced[/color]";
                    }
                    if let Some(line) = self.stagger(true, &s, guarding) {
                        h.breakdown += &format!("\n[color=#fbbf24]{line}[/color]");
                        if line.contains("BREAK") {
                            h.detail += " • BREAK";
                        }
                    }
                }
                let mut detail = format!("{} • {ap_note}", hit.as_ref().map(|h| h.detail.clone()).unwrap_or_else(|| "No damage".into()));
                if skill::is_area(&s) {
                    detail += " • area: the dummy is the only target here";
                }
                if !s.status_effect.is_empty() && self.dummy.is_alive() {
                    let potency = s.status_potency + self.hero().status_potency_bonus();
                    // U1.2-S3b: the hero's KNOW term (ruleset `know_potency`; 0 = potency off), shown and rolled alike
                    let bonus = status::know_bonus(self.hero(), self.ruleset.numbers.know_potency);
                    // A4.2: the skill inflict as an EffectSpec, shown and rolled alike
                    let spec = status::EffectSpec::inflict(&s.status_effect, potency, 1, s.status_magnitude, &self.hero().name);
                    let chance = spec.chance(&self.dummy, bonus);
                    let landed = status::apply_effect(&mut self.dummy, &spec, status::Timing::Next, bonus, Some(&mut self.rng));
                    detail += &format!(" • {} {} ({chance}%)", status::display_name(&s.status_effect), if landed { "next round" } else { "resisted" });
                }
                let (dmg, outcome, breakdown) = hit.as_ref().map(|h| (h.dmg, h.outcome, h.breakdown.clone())).unwrap_or((0, "STATUS", String::new()));
                self.log_action("ATTACK", &s.name, &detail, dmg, outcome, &breakdown, &dummy_name, false);
                if let Some(h) = hit {
                    self.hit_fx(&h);
                }
            }
            "DEFEND" => {
                let mitigation = skill::guard_mitigation(&s, &self.ruleset.numbers);
                self.hero_mut().guard = mitigation;
                let pct = (mitigation * 100.0).round() as i32;
                // AUDIT-12 (S1.2-S4): a multi-round guard queues the `guard` status for its later rounds, as in battle
                let later = skill::guard_later_mitigation(&s, &self.ruleset.numbers);
                let hero_name = self.hero().name.clone();
                let more = if crate::command::queue_later_guard(self.hero_mut(), s.guard_rounds, later, &hero_name) {
                    format!("; guard holds {} more round{}", s.guard_rounds - 1, if s.guard_rounds == 2 { "" } else { "s" })
                } else {
                    String::new()
                };
                self.log_event("DEFEND", &s.name, &format!("Guard -{pct}% until the next round{more} • {ap_note}"), pct, "GUARD");
                self.fx("glow", true, &format!("{} -{pct}%", s.name), Tone::Info);
            }
            "FOCUS" => {
                self.hero_mut().apply_attack_boost(s.boost, s.boost_rounds);
                self.log_event("BUFF", &s.name, &format!("Attack x{:.2} for {} rounds • {ap_note}", s.boost, s.boost_rounds), ((s.boost - 1.0) * 100.0).round() as i32, &format!("BUFF x{:.2}", s.boost));
                self.fx("glow", true, &format!("x{:.2} ATK ({} rds)", s.boost, s.boost_rounds), Tone::Gold);
            }
            "REST" if self.ruleset.rest_at_round_end() => {
                // RULESET1_INTENT RS2, RS3: the Rest blocks pay their AP at the round's end ("Next round"); each one
                // restores poise now (rule poise)
                let blocks = s.blocks.max(1);
                self.rest_blocks += blocks;
                let mut bar = self.poise();
                let (before, pips_before) = (bar.value, bar.pips);
                let pips_mode = poise::pips_mode(&self.ruleset.numbers);
                if self.poise_on() && !pips_mode && bar.max > 0 && !bar.broken && bar.value < bar.max {
                    bar.value = (bar.value + self.ruleset.numbers.rest_poise_per_block * blocks).min(bar.max);
                    self.hero_poise[self.active] = bar;
                }
                // S2.2-S2: a pip per 2 Rest blocks under poise_mode pips instead (S2.2-S3: the value mirrors them)
                if self.poise_on() && poise::pips_mode(&self.ruleset.numbers) && bar.max > 0 && !bar.broken {
                    poise::rest_pips(&self.ruleset.numbers, &mut bar, blocks);
                    self.hero_poise[self.active] = bar;
                }
                let ap = round_intent::rest_gain(&self.ruleset.numbers, self.rest_blocks);
                let mut detail = format!("{} Rest block{} this round: +{ap} AP at the round's end", self.rest_blocks, if self.rest_blocks == 1 { "" } else { "s" });
                if bar.value > before {
                    detail += &format!(", poise {before} -> {}", bar.value);
                }
                if bar.pips > pips_before {
                    detail += &format!(", pips {pips_before} -> {}", bar.pips);
                }
                self.log_event("BUFF", &s.name, &format!("{detail} • {ap_note}"), ap, &format!("REST {}", self.rest_blocks));
                self.fx("text", true, "Rest", Tone::Good);
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
        // RULESET1 P7: a broken dummy's actions are cancelled until the next round
        if self.dummy_poise.broken {
            self.log_action("EVENT", &title, "The dummy is broken and loses its action this round", 0, "BROKEN", "", &hero_name, true);
            return;
        }
        // Attacking: the dummy drops its guard and enters the skill's stance.
        self.dummy.guard = 0.0;
        if self.ruleset.stances() {
            self.dummy.set_stance(skill::stance(&s));
        }
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
        let mut breakdown = breakdown_strike(&title, &self.dummy.name, self.hero(), &res, dealt);
        // RULESET1 5: the hit's stagger on the hero's bar (rule poise; a guarding hero takes the guard factor)
        let guarding = self.hero().guard > 0.0;
        if let Some(line) = self.stagger(false, &s, guarding) {
            breakdown += &format!("\n[color=#fbbf24]{line}[/color]");
        }
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
            for st in status::roll_over(c, ROUND_AP_REGEN, &self.ruleset.numbers) {
                notes.push(format!("{} {}", c.name, status::display_name(&st).to_lowercase()));
            }
        }
        // AUDIT-12: the skill's guard drops; a queued later-round `guard` status that just took effect holds instead
        let held = status::guard_status_mitigation(self.hero());
        self.hero_mut().guard = held;
        // RULESET1_INTENT RS2: the Rest gain of the round that ended (`rest_ap` round_end, CLN-45); RULESET1 P3: the
        // bars refill (poise)
        let mut extra = String::new();
        if self.ruleset.rest_at_round_end() {
            let gain = round_intent::rest_gain(&self.ruleset.numbers, self.rest_blocks);
            if gain > 0 {
                let h = self.hero_mut();
                h.ap = h.max_ap.min(h.ap + gain);
                extra += &format!("; Rest: +{gain} AP from {} block{}", self.rest_blocks, if self.rest_blocks == 1 { "" } else { "s" });
            }
        }
        self.rest_blocks = 0;
        if self.poise_on() {
            let i = self.active;
            self.hero_poise[i] = self.full_bar(&self.heroes[i], self.hero_poise[i]);
            self.dummy_poise = self.full_bar(&self.dummy, self.dummy_poise);
            extra += &format!("; poise refills ({} / {})", self.hero_poise[i].max, self.dummy_poise.max);
        }
        let detail = format!(
            "Focus ticks, guard drops, +{ROUND_AP_REGEN} AP (now {}/{}){}{extra}",
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
        if !self.ruleset.stances() {
            return self.fail("Stance Shift", &format!("{} rules have no stances", self.ruleset.name));
        }
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
            // A1.2-K3 (A1 section 1): the item pipelines and the empower grant, as in battle
            "HEALTH_POTION" => {
                let before = self.hero().hp;
                self.hero_mut().heal(rpgkit_rules::damage::item_heal(item.heal));
                let healed = self.hero().hp - before;
                self.log_event("BUFF", &item.name, &format!("Restored {healed} HP"), healed, &format!("+{healed} HP"));
                self.fx("glow", true, &format!("+{healed} HP"), Tone::Good);
            }
            "THROWABLE_BOMB" => {
                // with item mitigation the dummy's guard and armour run in the pipeline (no cover here); without it the
                // flat damage meets the dummy's guard as before
                let dealt = if self.ruleset.numbers.item_mitigation {
                    let mitigation = Some((self.dummy.guard, rpgkit_rules::damage::armor_rating(&self.dummy)));
                    let amount = rpgkit_rules::damage::item_damage(item.damage, rpgkit_rules::damage::ItemHit { splash: 1.0, mitigation });
                    self.dummy.take_damage(amount, true)
                } else {
                    self.dummy.take_damage(rpgkit_rules::damage::item_damage(item.damage, rpgkit_rules::damage::ItemHit::default()), false)
                };
                let outcome = if self.dummy.is_alive() { "HIT" } else { "DEFEATED" };
                self.log_action("ATTACK", &item.name, "Explosive damage", dealt, outcome, "", &dummy_name, false);
                self.fx("strike", true, &format!("-{dealt}"), Tone::Warn);
            }
            _ => {
                let (status_id, pct) = crate::command::item_grant(&item);
                status::grant_now(self.hero_mut(), status_id, item.duration.max(1), pct, &format!("item:{}", item.id));
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
            if self.ruleset.stances() {
                self.dummy.set_stance(Stance::Defensive);
            }
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

    /// U3.2-S3 (FB-34): whether the active hero wears `item` trained (its class's proficiency; `gear::trained_for`). No
    /// longer a gate: an untrained piece equips and fights under the wield penalty.
    pub fn trained_for(&self, item: &ItemDef) -> bool {
        self.cat.classes.get(&self.classes[self.active]).is_none_or(|c| trained_for(&Proficiency::from_class(c), item))
    }

    /// The stash items the active hero can put in `slot` (fit and the two-handed rule; U3.2-S3: trained or not), one per
    /// id.
    pub fn eligible(&self, slot: &str) -> Vec<String> {
        let main = self.hero().inventory.main_weapon().map(|e| e.item.clone());
        let mut out: Vec<String> = Vec::new();
        for id in &self.stash {
            let Some(item) = self.cat.items.get(id) else { continue };
            if out.contains(id) || !item_fits_slot(item, slot) {
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
        // AUDIT-01: the ready weapon's effects and family bonuses count, the stowed one's not (a resync)
        self.changed();
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
        // RULESET1-G: a rebuild (ruleset toggle, deck) spends them again
        let spent = &mut self.spent[self.active];
        for (st, pts) in a.pending.iter().filter(|(_, p)| **p > 0) {
            *spent.entry(st.clone()).or_insert(0) += pts;
        }
        self.changed();
        let detail = if summary.is_empty() { "No points spent".to_string() } else { summary.join(", ") };
        let left = self.hero().stats.unallocated;
        self.log_event("EVENT", "Allocate Stats", &format!("{detail} (remaining {left})"), 0, "CONFIRMED");
    }

    pub fn cancel_allocation(&mut self) {
        self.alloc = None;
    }
}

/// A factor as the breakdowns write it: `2`, `0.5`, `1.25`.
fn num(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn list_or_none(ids: &[String]) -> String {
    if ids.is_empty() {
        "none".into()
    } else {
        ids.join(", ")
    }
}

fn list_or_auto(ids: &[String]) -> String {
    if ids.is_empty() {
        "auto".into()
    } else {
        ids.join(", ")
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
            // U3.2-S3 (FB-33): the proficiency 0 factor, after any deficit
            if p.untrained {
                lines.push(format!("   [color=#f87171]Untrained x{:.2} = {:.1}[/color]", p.untrained_mult, p.weapon_power));
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
