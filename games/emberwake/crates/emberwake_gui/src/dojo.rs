//! Gear Dojo (scripts/ui/dojo, the DojoWorkspace desk; dev tool): the gear dojo's sandbox (emberwake_battle::dojo) on
//! a desk of engine windows (UIKIT3): party with the carry bar, unit inspector with stat allocation, derived stats,
//! gear with the battle pouch, stash with filters, the stage (hero against the training dummy), the action console and
//! the test log with each entry's breakdown. Items drag between the stash, the gear slots and the pouch (UIKIT2);
//! the log and the stash scroll; layouts are Godot's (Workbench, Combat Focus, Gear Focus) and the desk is saved.

use crate::screens::{Cx, GameCx, Nav, Screen};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_ui::window::{Desk, Frac, Layout, Win};
use crate::party::{DeckTerms, DECK_KINDS};
use emberwake_battle::dojo::{stance_label, DojoSession, Fx, PlanSetup, PlanView, Tone, SLOTS};
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{ManeuverKind, RoundIntent};
use emberwake_data::Deck;
use emberwake_data::STATS;
use emberwake_rules::damage;
use emberwake_rules::gear::item_fits_slot;
use serde_json::{json, Value};
use aegis_core::clock::{Duration, Instant};

const SCREEN_ID: &str = "gear_dojo";
/// The desk under the two toolbar rows.
const AREA: Rect = Rect::new(0, 30, W, 330);
const LOG_FILTERS: [&str; 5] = ["ALL", "ATTACK", "DEFEND", "BUFF", "EVENT"];
const STASH_FILTERS: [(&str, &str); 5] = [("all", "All"), ("weapon", "Weapons"), ("armor", "Armor"), ("accessory", "Acc."), ("consumable", "Items")];
const BENCH: [usize; 3] = [10, 100, 1000];
const FX_LIFE: Duration = Duration::from_millis(1400);

#[derive(Debug, Clone)]
pub enum DAct {
    Hero(usize),
    Strike(&'static str),
    UseSkill,
    Skill(usize),
    DummySkill(usize),
    Counter,
    ForceCrit,
    DummyPower(i32),
    HealHero,
    HealDummy,
    NextRound,
    Stance,
    /// RULESET1-R: Classic / Timeline.
    /// CLN-46: the ruleset at this index of the catalog's rows.
    Ruleset(usize),
    DummyGuard,
    Bench(usize),
    Switch,
    Preset(usize),
    ClearGear,
    Unequip(&'static str),
    StashClick(String),
    Pouch(usize),
    Unpouch(usize),
    StashFilter(&'static str),
    LogFilter(&'static str),
    LogClear,
    LogPick(usize),
    AllocBegin,
    Alloc(&'static str, bool),
    AllocReset,
    AllocConfirm,
    Reset,
    Exit,
    /// RULESET1-G: "Dummy in windup".
    DummyWindup,
    /// The planner view (RULESET1_INTENT UI4): an intent of the ruleset (its index), a Maneuver kind (None: the
    /// planner's), an opener (0: the planner's, else the order's index + 1), the piece a board click moves, a board
    /// cell, the second dummy on or off.
    PlanIntent(usize),
    PlanKind(Option<ManeuverKind>),
    PlanOpener(usize),
    PlanPiece(&'static str),
    PlanCell(i32, i32),
    PlanPartner,
    /// The deck view (RULESET1 9 D6): slot an owned one, take slot `i` of a kind out, move an order entry one place
    /// up, the automatic deck, the automatic order.
    DeckSlot(&'static str, String),
    DeckUnslot(&'static str, usize),
    OrderUp(String, usize),
    DeckDefault,
    OrderAuto,
    /// INPUT1 FL3 (the party screen's terms): open or close an opener's follow-up list, add a family at its end, take
    /// place `k` out (a click), empty it ("Use the <Intent> order").
    FollowOpen(String),
    FollowAdd(String, String),
    FollowRemove(String, usize),
    FollowClear(String),
}

/// The planner cache's key: (screen revision, active hero, setup, rules).
type PlanKey = (u64, usize, PlanSetup, Vec<&'static str>);

/// What the planner window shows, computed when its inputs change (each plan compiles a scratch battle).
#[derive(Clone)]
struct PlanCache {
    key: PlanKey,
    view: PlanView,
    by_intent: Vec<PlanView>,
    openers: Vec<(String, String)>,
}

pub struct Dojo {
    s: DojoSession,
    desk: Desk,
    log_filter: &'static str,
    stash_filter: &'static str,
    /// The log entry whose breakdown is shown (its number).
    picked: Option<usize>,
    /// The newest log entry drawn so far (a newer one scrolls the log to the end).
    seen: usize,
    floats: Vec<(Fx, Instant)>,
    /// The planner piece a board click moves ("hero", "dummy", "partner").
    plan_piece: &'static str,
    plan: Option<PlanCache>,
    /// Counts the screen's actions and drops (the planner cache's key).
    rev: u64,
    /// INPUT1 FL3: the opener family whose follow-up list is open in the Deck window.
    follow_open: Option<String>,
    w: Vec<Widget<DAct>>,
}

/// The planner board's cell size (canvas pixels).
const CELL: i32 = 7;

fn win(id: &str, title: &str) -> Win {
    Win::new(id, title, (110, 60))
}

/// DojoWorkspace.LAYOUTS (fractions of the desk; a window a layout leaves out is closed).
fn layouts() -> Vec<Layout> {
    let l = |name: &str, places: &[(&str, [f32; 4])]| Layout { name: name.into(), places: places.iter().map(|(id, r)| (id.to_string(), Frac::new(r[0], r[1], r[2], r[3]))).collect() };
    vec![
        l("Workbench", &[("inspector", [0.0, 0.0, 0.21, 0.55]), ("derived", [0.0, 0.55, 0.21, 0.45]), ("gear", [0.21, 0.0, 0.25, 1.0]), ("stage", [0.46, 0.0, 0.29, 0.5]), ("console", [0.46, 0.5, 0.29, 0.5]), ("log", [0.75, 0.0, 0.25, 1.0])]),
        l("Combat Focus", &[("stage", [0.0, 0.0, 0.5, 0.62]), ("console", [0.0, 0.62, 0.5, 0.38]), ("log", [0.5, 0.0, 0.5, 0.62]), ("derived", [0.5, 0.62, 0.25, 0.38]), ("inspector", [0.75, 0.62, 0.25, 0.38])]),
        l("Gear Focus", &[("gear", [0.0, 0.0, 0.34, 1.0]), ("inspector", [0.34, 0.0, 0.26, 0.55]), ("derived", [0.34, 0.55, 0.26, 0.45]), ("stage", [0.6, 0.0, 0.4, 0.42]), ("console", [0.6, 0.42, 0.4, 0.3]), ("log", [0.6, 0.72, 0.4, 0.28])]),
        // RULESET1-G: the planner view and the deck view beside the stage and the console (applied when Timeline is
        // switched on with the planner closed)
        l("Timeline", &[("planner", [0.0, 0.0, 0.42, 1.0]), ("deck", [0.42, 0.0, 0.29, 1.0]), ("stage", [0.71, 0.0, 0.29, 0.28]), ("console", [0.71, 0.28, 0.29, 0.34]), ("derived", [0.71, 0.62, 0.29, 0.38])]),
    ]
}

fn tone(t: Tone) -> Rgba {
    match t {
        Tone::Info => th().info,
        Tone::Good => th().good,
        Tone::Bad => th().foe,
        Tone::Warn => rgb(251, 191, 36),
        Tone::Muted => th().dim,
        Tone::Gold => th().gold,
    }
}

fn category_colour(c: &str) -> Rgba {
    match c {
        "ATTACK" => th().foe,
        "DEFEND" => th().info,
        "BUFF" => th().good,
        _ => th().dim,
    }
}

fn hp_bar(ui: &mut Ui<DAct>, r: Rect, now: i32, max: i32, col: Rgba) {
    ui.p.fill(r, th().track);
    let w = (r.w as f32 * now.max(0) as f32 / max.max(1) as f32).round() as i32;
    ui.p.fill(Rect::new(r.x, r.y, w, r.h), col);
}

impl Dojo {
    pub fn new(cx: &Cx) -> Self {
        let wins = vec![
            win("party", "Party"),
            win("inspector", "Unit Inspector"),
            win("derived", "Derived Stats"),
            win("gear", "Gear"),
            win("stash", "Stash"),
            win("stage", "Stage"),
            win("console", "Console"),
            win("log", "Test Log"),
            win("planner", "Planner"),
            win("deck", "Deck"),
        ];
        let mut desk = Desk::new(AREA, wins, layouts());
        desk.restore(&cx.settings.v["desks"][SCREEN_ID]);
        Self { s: DojoSession::new(cx.data.cat.clone()), desk, log_filter: "ALL", stash_filter: "all", picked: None, seen: 0, floats: Vec::new(), plan_piece: "dummy", plan: None, rev: 0, follow_open: None, w: Vec::new() }
    }

    /// The active hero's deck terms (None without the deck rule).
    fn deck_terms(&self) -> Option<DeckTerms> {
        let def = self.s.cat.units.get(self.s.hero_ids.get(self.s.active)?)?;
        DeckTerms::for_unit(&self.s.cat, &self.s.ruleset, def, self.s.hero())
    }

    /// A deck edit on the terms: the hero is rebuilt with the new deck, or the log says why not.
    fn deck_edit(&mut self, edit: impl FnOnce(&DeckTerms, &Deck) -> Result<Deck, String>) {
        let Some(t) = self.deck_terms() else { return };
        match edit(&t, &self.s.deck_of(self.s.active)) {
            Ok(d) => self.s.set_deck(d),
            Err(e) => self.s.log_action("EVENT", "Deck", &e, 0, "FAILED", "", "", false),
        }
    }

    /// What the planner's contents depend on: the screen's revision (every action and drop counts one, so a change
    /// that logs nothing, such as the dummy's power, still counts), the active hero, the setup and the rules.
    fn plan_key(&self) -> PlanKey {
        (self.rev, self.s.active, self.s.planner.clone(), self.s.ruleset.rules.clone())
    }

    /// The planner's contents now (None without round intents).
    fn plan_now(&self) -> Option<PlanCache> {
        if self.s.plan_intents().is_empty() {
            return None;
        }
        Some(PlanCache { view: self.s.plan_view(), by_intent: self.s.plans_by_intent(), openers: self.s.plan_openers(), key: self.plan_key() })
    }

    /// The planner window's contents when its inputs changed.
    fn refresh_plan(&mut self) {
        if self.plan.as_ref().is_some_and(|p| p.key == self.plan_key()) {
            return;
        }
        self.plan = self.plan_now();
    }

    fn act(&mut self, cx: &mut Cx, a: DAct) {
        self.rev += 1;
        let s = &mut self.s;
        match a {
            DAct::Hero(i) => s.set_active(i),
            DAct::Strike(k) => s.strike(k),
            DAct::UseSkill => s.use_skill(None),
            DAct::Skill(i) => {
                let id = s.usable_skills().get(i).map(|k| k.id.clone());
                if let Some(id) = id {
                    s.select_skill(&id);
                }
            }
            DAct::DummySkill(i) => {
                let id = s.attack_skills().get(i).map(|k| k.id.clone());
                if let Some(id) = id {
                    s.select_dummy_skill(&id);
                }
            }
            DAct::Counter => s.dummy_counter = !s.dummy_counter,
            DAct::ForceCrit => s.force_crit = !s.force_crit,
            DAct::DummyPower(step) => {
                let p = s.dummy.power_override + step;
                s.set_dummy_power(p);
            }
            DAct::HealHero => s.heal_hero(),
            DAct::HealDummy => s.heal_dummy(),
            DAct::NextRound => s.next_round(),
            DAct::Stance => s.shift_stance(),
            DAct::Ruleset(i) => {
                use emberwake_rules::ruleset::Ruleset;
                // CLN-46: a pick over the catalog's rows
                if let Some(id) = Ruleset::ids(&s.cat).get(i) {
                    match Ruleset::of(&s.cat, id) {
                        Ok(r) => s.set_ruleset(r),
                        Err(e) => s.log_action("EVENT", "Ruleset", &e, 0, "FAILED", "", "", false),
                    }
                }
                // RULESET1-G: under rules with round intents the planner and the deck view come up (unless the planner is open)
                if !s.plan_intents().is_empty() && !self.desk.win("planner").is_some_and(|w| w.open) {
                    self.desk.apply_layout("Timeline");
                }
            }
            DAct::DummyWindup => s.toggle_dummy_windup(),
            DAct::PlanIntent(i) => {
                if let Some(intent) = s.plan_intents().get(i).copied() {
                    s.set_plan_intent(intent);
                }
            }
            DAct::PlanKind(k) => s.planner.kind = k,
            DAct::PlanOpener(i) => s.planner.opener = i.checked_sub(1).and_then(|k| self.plan.as_ref()?.openers.get(k)).map(|o| o.0.clone()),
            DAct::PlanPiece(p) => self.plan_piece = p,
            DAct::PlanCell(x, y) => {
                s.place_plan(self.plan_piece, Some(V2::new(x, y)));
            }
            DAct::PlanPartner => {
                let at = if s.planner.partner.is_some() { None } else { Some(free_beside(&s.planner)) };
                s.place_plan("partner", at);
                if at.is_none() && self.plan_piece == "partner" {
                    self.plan_piece = "dummy";
                }
            }
            DAct::DeckSlot(kind, id) => self.deck_edit(|t, d| t.slot(d, kind, &id)),
            DAct::DeckUnslot(kind, i) => self.deck_edit(|t, d| t.unslot(d, kind, i)),
            DAct::OrderUp(intent, k) => self.deck_edit(|t, d| t.after_drop(d, &format!("order.{intent}.{k}"), &format!("order.{intent}.{}", k.saturating_sub(1)))),
            DAct::DeckDefault => s.set_deck(Deck::default()),
            DAct::OrderAuto => self.deck_edit(|t, d| {
                let mut d = t.shown(d);
                d.order.clear();
                Ok(d)
            }),
            DAct::FollowOpen(op) => self.follow_open = if self.follow_open.as_deref() == Some(op.as_str()) { None } else { Some(op) },
            DAct::FollowAdd(op, fam) => self.deck_edit(|t, d| t.follow_add(d, &op, &fam, None)),
            DAct::FollowRemove(op, k) => self.deck_edit(|t, d| t.follow_remove(d, &op, k)),
            DAct::FollowClear(op) => self.deck_edit(|t, d| t.follow_clear(d, &op)),
            DAct::DummyGuard => s.toggle_dummy_defense(),
            DAct::Bench(n) => s.run_benchmark(n),
            DAct::Switch => s.switch_weapon(),
            // PK1-KITFIX (c): the built-in presets with the catalog's dojo_presets rows over them
            DAct::Preset(i) => {
                if let Some(p) = s.presets().get(i) {
                    let id = p.id.clone();
                    s.apply_preset(&id);
                }
            }
            DAct::ClearGear => s.clear_gear(),
            DAct::Unequip(slot) => s.unequip(slot),
            DAct::StashClick(id) => {
                // A click wears the piece in the first slot it fits (a consumable goes to the pouch).
                if s.cat.items.get(&id).is_some_and(|i| i.item_type == "CONSUMABLE") {
                    s.add_to_pouch(&id);
                } else if let Some((slot, _)) = SLOTS.iter().find(|(k, _)| s.eligible(k).contains(&id)) {
                    s.equip(&id, slot);
                } else {
                    let name = s.cat.items.get(&id).map(|i| i.name.clone()).unwrap_or(id);
                    s.log_action("EVENT", "Equip", &format!("{name} fits no slot this hero can use"), 0, "FAILED", "", "", false);
                }
            }
            DAct::Pouch(i) => s.use_pouch(i),
            DAct::Unpouch(i) => s.remove_from_pouch(i),
            DAct::StashFilter(f) => self.stash_filter = f,
            DAct::LogFilter(f) => self.log_filter = f,
            DAct::LogClear => {
                s.clear_log();
                self.picked = None;
            }
            DAct::LogPick(n) => self.picked = if self.picked == Some(n) { None } else { Some(n) },
            DAct::AllocBegin => s.begin_allocation(),
            DAct::Alloc(st, up) => {
                if up {
                    s.allocate(st);
                } else {
                    s.deallocate(st);
                }
            }
            DAct::AllocReset => s.reset_allocation(),
            DAct::AllocConfirm => s.confirm_allocation(),
            DAct::Reset => {
                s.reset();
                self.picked = None;
            }
            DAct::Exit => cx.nav.push(Nav::Pop),
        }
        // New log entries: the newest is shown in the breakdown pane.
        if let Some(last) = self.s.log.last().filter(|e| !e.breakdown.is_empty()) {
            if self.picked.is_none_or(|p| p < last.n) {
                self.picked = Some(last.n);
            }
        }
    }

    fn stash_rows(&self) -> Vec<(String, usize)> {
        let mut rows: Vec<(String, usize)> = Vec::new();
        for id in &self.s.stash {
            let Some(item) = self.s.cat.items.get(id) else { continue };
            let keep = match self.stash_filter {
                "weapon" => matches!(item.item_type.as_str(), "WEAPON" | "SHIELD"),
                "armor" => item.item_type == "ARMOR",
                "accessory" => item.item_type == "ACCESSORY",
                "consumable" => item.item_type == "CONSUMABLE",
                _ => true,
            };
            if !keep {
                continue;
            }
            match rows.iter_mut().find(|r| r.0 == *id) {
                Some(r) => r.1 += 1,
                None => rows.push((id.clone(), 1)),
            }
        }
        rows.sort_by_key(|(id, _)| self.s.cat.items.get(id).map(|i| (i.item_type.clone(), i.name.clone())));
        rows
    }

    // ---- Windows -----------------------------------------------------------------------------------------------

    fn draw_party(&self, ui: &mut Ui<DAct>, b: Rect) {
        let mut y = b.y + 2;
        for (i, h) in self.s.heroes.iter().enumerate() {
            let lines = [(h.name.clone(), if i == self.s.active { th().gold } else { th().text }), (format!("HP {}/{}  AP {}/{}  {}", h.hp, h.max_hp, h.ap, h.max_ap, stance_label(h.stance)), th().dim)];
            ui.card(&format!("dojo.hero.{i}"), Rect::new(b.x + 2, y, b.w - 4, 24), &lines, Some(DAct::Hero(i)), i == self.s.active, true, tip(&h.name, "Make this hero the active one."));
            y += 27;
        }
        let c = self.s.carry_now();
        ui.p.text(b.x as f32 + 3.0, y as f32 + 2.0, &format!("Carry {:.0} / {}  {}", c.load, c.capacity, c.tier.label()), SMALL, if c.overload_penalty > 0 { th().foe } else { th().text });
        let bar = Rect::new(b.x + 3, y + 11, b.w - 6, 4);
        let col = match c.tier {
            emberwake_rules::carry::Tier::Light => th().good,
            emberwake_rules::carry::Tier::Heavy => th().gold,
            emberwake_rules::carry::Tier::Overloaded => th().foe,
        };
        hp_bar(ui, bar, (c.ratio.min(1.0) * 100.0) as i32, 100, col);
        let help = format!(
            "Party carry: capacity {} (base {} + STR x3 + CONS per member); load {:.1} = stash {:.1} + worn {:.1} + pouches {:.1}. Light under 75%, Heavy to 100%, Overloaded above: every member -1 initiative per 10 over, move rate only past initiative 1 ({}).",
            c.capacity, emberwake_rules::carry::BASE_CAPACITY, c.load, c.stash_weight, c.equipped_weight, c.pouch_weight, c.overload_penalty
        );
        ui.push("dojo.carry", "row", format!("carry {}", c.tier.label()), Rect::new(b.x + 2, y, b.w - 4, 16), true, false, None, tip("Carry", &help));
    }

    fn draw_inspector(&self, ui: &mut Ui<DAct>, b: Rect) {
        let h = self.s.hero();
        let mut y = b.y as f32 + 2.0;
        let class = self.s.cat.classes.get(&self.s.classes[self.s.active]).map(|c| c.name.clone()).unwrap_or_default();
        ui.p.text_fit(b.x as f32 + 3.0, y, &format!("{} ({class}) Lv {}", h.name, h.stats.level), SMALL, b.w as f32 - 6.0, th().gold);
        y += 10.0;
        let ctx = h.weapon_context();
        let alloc = self.s.alloc.as_ref();
        for st in STATS {
            let (base, eff) = (h.stats.base(st), h.stats.effective(st, ctx.as_deref(), ""));
            let pending = alloc.and_then(|a| a.pending.get(st).copied()).unwrap_or(0);
            let text = format!("{st}  {eff}{}{}", if eff != base { format!(" ({:+})", eff - base) } else { String::new() }, if pending > 0 { format!("  +{pending}") } else { String::new() });
            ui.p.text(b.x as f32 + 3.0, y, &text, SMALL, if pending > 0 { th().good } else { th().text });
            if alloc.is_some() {
                ui.button(&format!("dojo.alloc.{st}.minus"), Rect::new(b.x + b.w - 28, y as i32 - 1, 12, 9), "-", DAct::Alloc(st, false), pending > 0);
                ui.button(&format!("dojo.alloc.{st}.plus"), Rect::new(b.x + b.w - 14, y as i32 - 1, 12, 9), "+", DAct::Alloc(st, true), alloc.is_some_and(|a| a.free > 0));
            }
            y += 9.0;
        }
        y += 2.0;
        match alloc {
            Some(a) => {
                ui.p.text(b.x as f32 + 3.0, y, &format!("{} points left", a.free), SMALL, th().gold);
                ui.button("dojo.alloc.confirm", Rect::new(b.x + 3, y as i32 + 9, 44, 11), "Confirm", DAct::AllocConfirm, true);
                ui.button("dojo.alloc.reset", Rect::new(b.x + 50, y as i32 + 9, 34, 11), "Reset", DAct::AllocReset, true);
                ui.button("dojo.alloc.cancel", Rect::new(b.x + 87, y as i32 + 9, 40, 11), "Cancel", DAct::AllocBegin, true);
                y += 22.0;
            }
            None => {
                ui.button_full("dojo.alloc", Rect::new(b.x + 3, y as i32, 90, 11), &format!("Allocate ({})", h.stats.unallocated), DAct::AllocBegin, true, false, SMALL, tip("Allocate stats", "Spend the hero's unspent attribute points."));
                y += 14.0;
            }
        }
        ui.p.text(b.x as f32 + 3.0, y, "Proficiency", SMALL, th().info);
        y += 9.0;
        for gear in h.stats.proficiency.entries.keys() {
            let lv = h.stats.proficiency.level_of(gear);
            ui.p.text(b.x as f32 + 3.0, y, &format!("{gear}  Lv {lv}  x{:.2}", h.stats.proficiency.scaling_of(gear)), SMALL, if lv > 0 { th().text } else { th().dim });
            y += 8.5;
        }
    }

    fn draw_derived(&self, ui: &mut Ui<DAct>, b: Rect) {
        let u = self.s.hero();
        let off = if u.inventory.offhand_weapon().is_some() { damage::attack_power(u, false, true).attack_power.to_string() } else { "-".into() };
        let rows = [
            ("Melee power", damage::attack_power(u, false, false).attack_power.to_string()),
            ("Ranged power", damage::attack_power(u, true, false).attack_power.to_string()),
            ("Off-hand power", off),
            ("Initiative", u.initiative.to_string()),
            ("Max HP", u.max_hp.to_string()),
            ("Armor", damage::armor_rating(u).to_string()),
            ("Burden", u.worn_burden().to_string()),
            ("Tiles per move", u.move_per_block().to_string()),
            ("Attack range", u.attack_range.to_string()),
            ("Crit rate", format!("{:.1}%", damage::crit_chance(u))),
            ("Debuff resist", format!("{:.0}%", u.debuff_resistance())),
        ];
        // DERIVE1 4.4: the catalog's derived stats shown on the Dojo under its ruleset toggle follow, each hovering to its
        // breakdown. With them the rows close up from 9 px to one line of text to fit the window; past that the list
        // scrolls (the wheel), so no row runs out of the window. Without them the window is as before.
        let extra = crate::party::data_rows(&self.s.cat, &self.s.ruleset.id, u, "dojo");
        if extra.is_empty() {
            for (i, (k, v)) in rows.iter().enumerate() {
                let y = b.y as f32 + 2.0 + i as f32 * 9.0;
                ui.p.text(b.x as f32 + 3.0, y, k, SMALL, th().dim);
                ui.p.text_right((b.x + b.w) as f32 - 4.0, y, v, SMALL, th().text);
            }
            return;
        }
        let n = (rows.len() + extra.len()) as i32;
        let line = (ui.p.line_h(SMALL).ceil() as i32).min(9);
        let step = ((b.h - 3) / n).clamp(line, 9);
        let off = ui.scroll_begin("dojo.derived_scroll", b, n * step + 3);
        let top = |i: usize| b.y + 2 - off + i as i32 * step;
        for (i, (k, v)) in rows.iter().enumerate() {
            ui.p.text(b.x as f32 + 3.0, top(i) as f32, k, SMALL, th().dim);
            ui.p.text_right((b.x + b.w) as f32 - 6.0, top(i) as f32, v, SMALL, th().text);
        }
        for (i, r) in extra.iter().enumerate() {
            let y = top(rows.len() + i);
            ui.p.text(b.x as f32 + 3.0, y as f32, &r.label, SMALL, th().dim);
            ui.p.text_right((b.x + b.w) as f32 - 6.0, y as f32, &r.shown, SMALL, th().text);
            ui.push(format!("dojo.derived.{}", r.label), "row", format!("{} {}", r.label, r.shown), Rect::new(b.x + 1, y - 1, b.w - 5, step), true, false, None, tip(&r.label, &r.desc));
        }
        ui.scroll_end();
    }

    fn draw_gear(&self, ui: &mut Ui<DAct>, b: Rect) {
        let inv = &self.s.hero().inventory;
        let dragging = ui.dragging().map(String::from);
        let mut y = b.y + 2;
        for (key, slot) in SLOTS {
            let worn = inv.get(slot).map(|e| e.item.clone());
            let label = format!("{}: {}", key.replace('_', " "), worn.as_ref().map(|w| w.name.clone()).unwrap_or_else(|| "empty".into()));
            // While a stash item is dragged, the slots it fits are named in gold.
            let fits = dragging.as_deref().and_then(|d| d.strip_prefix("stash.")).is_some_and(|id| self.s.eligible(key).iter().any(|e| e == id));
            // R1-BAL (FB-80): the worn item's wield penalty (untrained or a stat deficit), as on the party screen
            let badge = worn.as_ref().and_then(|w| crate::party::wield_badge(Some(self.s.hero()), w));
            let card = match &worn {
                Some(w) => tip(&w.name, &format!("Click to unequip, or drag it to the stash.{}", badge.as_ref().map(|(_, n)| format!(" {n}")).unwrap_or_default())),
                None => tip(key, "Drag an item from the stash here."),
            };
            ui.row(&format!("dojo.slot.{key}"), Rect::new(b.x + 1, y, b.w - 2, 12), &label, if fits { th().gold } else if worn.is_some() { th().text } else { th().dim }, worn.as_ref().map(|_| DAct::Unequip(key)), false, card);
            if let Some((text, _)) = &badge {
                ui.p.text_right((b.x + b.w - 4) as f32, y as f32 + 1.0, text, SMALL, th().foe);
            }
            if worn.is_some() {
                ui.draggable(format!("equip.{key}"));
            }
            ui.drop_target(format!("slot.{key}"));
            y += 13;
        }
        y += 3;
        ui.p.text(b.x as f32 + 3.0, y as f32, "Battle pouch", SMALL, th().info);
        y += 9;
        for i in 0..inv.max_pouch_slots {
            let item = inv.pouch.get(i);
            let label = format!("{}. {}", i + 1, item.map(|i| i.name.clone()).unwrap_or_else(|| "Empty".into()));
            ui.row(&format!("dojo.pouch.{i}"), Rect::new(b.x + 1, y, b.w - 26, 12), &label, if item.is_some() { th().text } else { th().dim }, item.map(|_| DAct::Pouch(i)), false, tip(&label, "Click to use it on the dummy or hero; drag it to the stash to unload."));
            if item.is_some() {
                ui.draggable(format!("pouch.{i}"));
                ui.button_full(&format!("dojo.pouch.{i}.out"), Rect::new(b.x + b.w - 23, y, 20, 11), "x", DAct::Unpouch(i), true, false, SMALL, tip("Unload", "Back to the stash."));
            }
            ui.drop_target(format!("pouch.{i}"));
            y += 13;
        }
        y += 3;
        let names: Vec<String> = self.s.presets().into_iter().map(|p| p.name).collect();
        ui.p.text(b.x as f32 + 3.0, y as f32 + 1.0, "Quick setup", SMALL, th().info);
        ui.select("dojo.preset", Rect::new(b.x + 52, y, b.w - 56, 11), &names, usize::MAX, DAct::Preset, true, tip("Loadout presets", "Stows all gear, then equips the preset (from the stash, else made); pieces this hero can't use are skipped."));
        ui.button("dojo.cleargear", Rect::new(b.x + 3, y + 14, 60, 11), "Clear gear", DAct::ClearGear, true);
    }

    fn draw_stash(&self, ui: &mut Ui<DAct>, b: Rect) {
        let mut x = b.x + 2;
        for (key, label) in STASH_FILTERS {
            let w = ui.p.text_w(label, SMALL).ceil() as i32 + 8;
            ui.button_full(&format!("dojo.stash.filter.{key}"), Rect::new(x, b.y + 1, w, 10), label, DAct::StashFilter(key), true, self.stash_filter == key, SMALL, Vec::new());
            x += w + 2;
        }
        let view = Rect::new(b.x, b.y + 14, b.w, b.h - 14);
        // The whole list takes back worn and pouch items.
        ui.push("dojo.stash", "panel", "stash", view, true, false, None, tip("Stash", "Drop a worn or pouch item here to stow it."));
        ui.drop_target("stash");
        let rows = self.stash_rows();
        let off = ui.scroll_begin("dojo.stash.rows", view, rows.len() as i32 * 12 + 2);
        for (i, (id, n)) in rows.iter().enumerate() {
            let Some(item) = self.s.cat.items.get(id) else { continue };
            let usable = item.item_type == "CONSUMABLE" || SLOTS.iter().any(|(k, _)| item_fits_slot(item, k));
            let r = Rect::new(view.x + 1, view.y + 1 - off + i as i32 * 12, view.w - 5, 12);
            // U3.2-S3 (FB-34): untrained gear equips; the help names the wield penalty
            let pen = emberwake_rules::gear::Equipment::new(item).wield_penalty(&self.s.hero().stats);
            let untrained = if item.is_equipment() && pen.untrained { format!(" Untrained: x{} power and armour, +{} burden.", pen.untrained_mult, pen.burden_add) } else { String::new() };
            let help = format!("{}{untrained} Click to wear it in its first free slot (or load the pouch), or drag it.", item_line(item));
            ui.row(&format!("dojo.stash.{id}"), r, &format!("{} x{n}", item.name), if usable { th().text } else { th().dim }, Some(DAct::StashClick(id.clone())), false, tip(&item.name, &help));
            ui.draggable(format!("stash.{id}"));
        }
        ui.scroll_end();
    }

    fn draw_stage(&self, ui: &mut Ui<DAct>, b: Rect) {
        let half = b.w / 2;
        for (i, (u, name)) in [(self.s.hero(), "hero"), (&self.s.dummy, "dummy")].into_iter().enumerate() {
            let x = b.x + 3 + i as i32 * half;
            let w = half - 6;
            ui.p.text_fit(x as f32, b.y as f32 + 2.0, &u.name, BODY, w as f32, if i == 0 { th().gold } else { th().foe });
            hp_bar(ui, Rect::new(x, b.y + 13, w, 5), u.hp, u.max_hp, if i == 0 { th().good } else { th().foe });
            ui.p.text(x as f32, b.y as f32 + 20.0, &format!("HP {}/{}", u.hp, u.max_hp), SMALL, th().text);
            let mut line2 = if i == 0 { format!("AP {}/{}  {}", u.ap, u.max_ap, stance_label(u.stance)) } else { format!("{}  guard {:.0}%", stance_label(u.stance), u.guard * 100.0) };
            // RULESET1 5 (rule poise): the poise bar under the HP bar
            if self.s.poise_on() {
                let p = if i == 0 { self.s.poise() } else { self.s.dummy_poise };
                // S2.2-S3 (FB-71): the bar's own pips under poise_mode pips
                let n = &self.s.ruleset.numbers;
                let (now, full, word) = if emberwake_battle::poise::pips_mode(n) { (p.pips, n.poise_pips.max(1), "pips") } else { (p.value, p.max, "poise") };
                hp_bar(ui, Rect::new(x, b.y + 18, w, 2), now, full, th().gold);
                line2 = format!("{}  {word} {now}/{full}{}", if i == 0 { format!("AP {}/{}", u.ap, u.max_ap) } else { format!("guard {:.0}%", u.guard * 100.0) }, if p.broken { " BROKEN" } else { "" });
                if i == 1 && self.s.dummy_windup {
                    line2 += "  windup";
                }
            }
            ui.p.text(x as f32, b.y as f32 + 28.0, &line2, SMALL, th().dim);
            let mut effects: Vec<String> = u.st.active.iter().map(|s| emberwake_rules::status::display_name(s)).collect();
            effects.extend(u.st.pending.iter().map(|s| format!("{} next", emberwake_rules::status::display_name(s))));
            if !effects.is_empty() {
                ui.p.text_fit(x as f32, b.y as f32 + 36.0, &effects.join(", "), SMALL, w as f32, th().info);
            }
            if u.attack_boost > 1.0 {
                ui.p.text(x as f32, b.y as f32 + 44.0, &format!("Focus x{:.2} ({} rds)", u.attack_boost, u.attack_boost_rounds), SMALL, th().gold);
            }
            ui.push(format!("dojo.stage.{name}"), "row", u.name.clone(), Rect::new(x, b.y, w, 50), true, false, None, Vec::new());
        }
        // Stage effects float up over the side they hit and fade: a hero's strike lands on the dummy (right).
        let now = Instant::now();
        let mut rows = [0, 0];
        for (fx, born) in self.floats.iter().rev() {
            let t = now.duration_since(*born).as_secs_f32() / FX_LIFE.as_secs_f32();
            let side = usize::from(fx.on_hero == (fx.kind == "strike"));
            let y = b.y as f32 + 62.0 + rows[side] as f32 * 9.0 - t * 10.0;
            let mut col = tone(fx.tone);
            col[3] = (255.0 * (1.0 - t).clamp(0.0, 1.0)) as u8;
            ui.p.text_fit((b.x + 3 + side as i32 * half) as f32, y, &fx.text, BODY, (half - 6) as f32, col);
            rows[side] += 1;
        }
        ui.p.text(b.x as f32 + 3.0, (b.y + b.h) as f32 - 9.0, &format!("Round {}", self.s.round), SMALL, th().dim);
    }

    fn draw_console(&self, ui: &mut Ui<DAct>, b: Rect) {
        let avail = self.s.availability();
        let mut x = b.x + 2;
        let mut y = b.y + 2;
        for (key, label, act) in [("main", "Main", DAct::Strike("main")), ("offhand", "Off-hand", DAct::Strike("offhand")), ("ranged", "Volley", DAct::Strike("ranged")), ("dual", "Dual", DAct::Strike("dual")), ("switch", "Switch", DAct::Switch)] {
            let a = &avail[key];
            let w = ui.p.text_w(label, SMALL).ceil() as i32 + 8;
            if x + w > b.x + b.w {
                x = b.x + 2;
                y += 13;
            }
            ui.button_full(&format!("dojo.strike.{key}"), Rect::new(x, y, w, 11), label, act, a.enabled, false, SMALL, tip(label, &a.tip));
            x += w + 2;
        }
        y += 14;
        let skills: Vec<String> = self.s.usable_skills().iter().map(|k| format!("{} ({} AP)", k.name, self.s.skill_ap(k))).collect();
        let at = self.s.usable_skills().iter().position(|k| k.id == self.s.skill).unwrap_or(0);
        ui.select("dojo.skill", Rect::new(b.x + 2, y, b.w - 42, 11), &skills, at, DAct::Skill, true, tip("Skill", "The skill Use applies (the same costs and stance rules as in battles)."));
        ui.button_full("dojo.use", Rect::new(b.x + b.w - 38, y, 36, 11), "Use", DAct::UseSkill, true, false, SMALL, tip("Use skill", "Pays AP (debt allowed) and the stance switch, then acts on the dummy."));
        y += 14;
        let attacks: Vec<String> = self.s.attack_skills().iter().map(|k| format!("Dummy: {}", k.name)).collect();
        let at = self.s.attack_skills().iter().position(|k| k.id == self.s.dummy_skill).unwrap_or(0);
        ui.select("dojo.dummy_skill", Rect::new(b.x + 2, y, b.w - 4, 11), &attacks, at, DAct::DummySkill, true, tip("Dummy skill", "What the dummy strikes back with."));
        y += 14;
        // CLN-46: the ruleset, a pick over the catalog's rows (a ruleset with round intents opens the Planner and Deck windows)
        let ids = emberwake_rules::ruleset::Ruleset::ids(&self.s.cat);
        let names: Vec<String> = ids.iter().map(|id| format!("Rules: {}", emberwake_rules::ruleset::Ruleset::of_or_classic(&self.s.cat, id).name)).collect();
        let at = ids.iter().position(|id| *id == self.s.ruleset.id).unwrap_or(0);
        ui.select("dojo.ruleset", Rect::new(b.x + 2, y, b.w - 4, 11), &names, at, DAct::Ruleset, !ids.is_empty(), tip("Ruleset", "The rules the Dojo plays by (the catalog's rows). Without stances there is no switch AP and no stance change; a deck rule rebuilds the heroes with their decks; round intents bring up the Planner and Deck windows."));
        y += 14;
        x = b.x + 2;
        let toggles = [
            ("counter", "Counter", DAct::Counter, self.s.dummy_counter, "The dummy strikes back after every hero skill."),
            ("crit", "Force crit", DAct::ForceCrit, self.s.force_crit, "Every hero hit crits."),
            ("guard", "Dummy guard", DAct::DummyGuard, self.s.dummy.guard > 0.0, "The dummy guards (Defensive stance, hits halved) or drops its guard."),
            ("windup", "Dummy windup", DAct::DummyWindup, self.s.dummy_windup, "The dummy is winding up: hits stagger it by the windup factor and may break it (rule poise)."),
        ];
        for (id, label, act, on, help) in toggles {
            if id == "windup" && !self.s.poise_on() {
                continue;
            }
            let w = ui.p.text_w(label, SMALL).ceil() as i32 + 8;
            if x + w > b.x + b.w {
                x = b.x + 2;
                y += 13;
            }
            ui.button_full(&format!("dojo.{id}"), Rect::new(x, y, w, 11), label, act, true, on, SMALL, tip(label, help));
            x += w + 2;
        }
        y += 14;
        x = b.x + 2;
        let actions = [
            ("round", "Next round", DAct::NextRound, "Focus ticks, statuses roll over, +4 AP, the guard drops."),
            ("stance", "Stance", DAct::Stance, "Cycle the hero's stance (the matchup against the dummy's)."),
            ("healhero", "Heal hero", DAct::HealHero, "The hero back to full HP."),
            ("healdummy", "Heal dummy", DAct::HealDummy, "The dummy back to full HP."),
        ];
        for (id, label, act, help) in actions {
            let w = ui.p.text_w(label, SMALL).ceil() as i32 + 8;
            if x + w > b.x + b.w {
                x = b.x + 2;
                y += 13;
            }
            let enabled = id != "stance" || self.s.ruleset.stances();
            ui.button_full(&format!("dojo.{id}"), Rect::new(x, y, w, 11), label, act, enabled, false, SMALL, tip(label, help));
            x += w + 2;
        }
        y += 14;
        x = b.x + 2;
        for n in BENCH {
            let label = format!("x{n}");
            ui.button_full(&format!("dojo.bench.{n}"), Rect::new(x, y, 30, 11), &label, DAct::Bench(n), true, false, SMALL, tip("Benchmark", "Simulated swings of the selected attack (Strike otherwise): min, average, max, crits; the dummy is not hurt."));
            x += 32;
        }
        y += 14;
        let power = self.s.dummy.power_override;
        ui.button_full("dojo.dummypower.minus", Rect::new(b.x + 2, y, 11, 11), "-", DAct::DummyPower(-5), power > 0, false, SMALL, Vec::new());
        ui.p.text(b.x as f32 + 16.0, y as f32 + 1.5, &format!("Dummy power {}", if power > 0 { power.to_string() } else { "auto (STR + DEX)".into() }), SMALL, th().dim);
        ui.button_full("dojo.dummypower.plus", Rect::new(b.x + 110, y, 11, 11), "+", DAct::DummyPower(5), true, false, SMALL, tip("Dummy power", "The dummy's authored strike power (0: from its stats)."));
    }

    /// RULESET1_INTENT UI4: the hero's intent, kind and opener against the dummy on a small board (click a tile to move
    /// the picked piece; the second dummy engages the dummy and sets its facing), the planned commands with their
    /// reasons, AP, estimates and rules, AP before and after, the Rest gain, and the plan each intent would make now.
    fn draw_planner(&self, ui: &mut Ui<DAct>, b: Rect) {
        let x0 = b.x + 2;
        let fit = (b.w - 6) as f32;
        let intents = self.s.plan_intents();
        let (Some(pc), false) = (self.plan.as_ref(), intents.is_empty()) else {
            ui.p.text_fit(x0 as f32, b.y as f32 + 2.0, "Plans timeline rounds: switch Timeline on in the Console.", SMALL, fit, th().dim);
            return;
        };
        let content = 120 + pc.view.rows.len() as i32 * 8 + pc.by_intent.len() as i32 * 8 + 24;
        let off = ui.scroll_begin("dojo.plan_scroll", b, content);
        let mut y = b.y + 2 - off;
        let setup = &self.s.planner;
        let mut x = x0;
        // A4.2-K1b: with an opener picked (field_intent), only its own rows are open; the others are locked
        let variants = self.s.plan_variants();
        for (i, it) in intents.iter().enumerate() {
            let name = it.label(&self.s.cat);
            let w = ui.p.text_w(&name, SMALL).ceil() as i32 + 8;
            let open = variants.get(i).is_none_or(|v| v.1);
            let mut card = tip(&name, "Plan the hero's round with this intent (the target is the dummy, the dummy's tile for Field, or the hero for Guard and Support).");
            if let Some(sk) = setup.opener.as_deref().and_then(|id| self.s.cat.skill(id)) {
                if let Some(line) = crate::intent_text::variant_line(&self.s.ruleset, sk, *it) {
                    card.push((line, th().text));
                }
                if !open {
                    card.push((format!("{} has no {name} row.", sk.name), th().foe));
                }
            }
            ui.button_full(&format!("dojo.plan.intent.{}", it.id()), Rect::new(x, y, w, 11), &name, DAct::PlanIntent(i), open, *it == setup.intent, SMALL, card);
            x += w + 2;
        }
        y += 13;
        if setup.intent == RoundIntent::Maneuver {
            x = x0;
            for (id, label, k) in [("auto", "Auto", None), ("engage", "Engage", Some(ManeuverKind::Engage)), ("disengage", "Disengage", Some(ManeuverKind::Disengage)), ("utility", "Utility", Some(ManeuverKind::Utility))] {
                let w = ui.p.text_w(label, SMALL).ceil() as i32 + 8;
                ui.button_full(&format!("dojo.plan.kind.{id}"), Rect::new(x, y, w, 11), label, DAct::PlanKind(k), true, setup.kind == k, SMALL, tip(label, "The Maneuver kind (Auto: Engage, else Disengage, else Utility)."));
                x += w + 2;
            }
            y += 13;
        }
        let mut opts = vec!["Opener: the planner's".to_string()];
        opts.extend(pc.openers.iter().map(|o| format!("Opener: {}", o.1)));
        let at = setup.opener.as_ref().and_then(|id| pc.openers.iter().position(|o| &o.0 == id)).map_or(0, |k| k + 1);
        ui.select("dojo.plan.opener", Rect::new(x0, y, b.w - 8, 11), &opts, at, DAct::PlanOpener, true, tip("Opener", "The round's first skill: the planner's pick (the first of the order it can afford and reach) or one of the order."));
        y += 14;
        // The board: a click moves the picked piece.
        let board = Rect::new(x0, y, GRID_W * CELL, GRID_H * CELL);
        ui.p.fill(board, rgba(22, 28, 42, 255));
        let path: Vec<V2> = pc.view.rows.iter().map(|r| r.cell).collect();
        for cy in 0..GRID_H {
            for cx in 0..GRID_W {
                let r = Rect::new(board.x + cx * CELL, board.y + cy * CELL, CELL - 1, CELL - 1);
                let c = V2::new(cx, cy);
                let (glyph, col) = if c == setup.hero {
                    ("H", th().gold)
                } else if c == setup.dummy {
                    ("D", th().foe)
                } else if setup.partner == Some(c) {
                    ("2", th().info)
                } else {
                    ("", if path.contains(&c) { rgba(60, 90, 70, 255) } else { rgba(34, 42, 60, 255) })
                };
                ui.p.fill(r, if glyph.is_empty() { col } else { rgba(40, 48, 66, 255) });
                if !glyph.is_empty() {
                    ui.p.text(r.x as f32 + 1.0, r.y as f32, glyph, SMALL, col);
                }
                ui.push(format!("dojo.plan.cell.{cx}.{cy}"), "button", format!("{cx},{cy}"), r, true, false, Some(DAct::PlanCell(cx, cy)), Vec::new());
            }
        }
        let bx = board.x + board.w + 4;
        let mut by = board.y;
        for (id, label) in [("hero", "Move hero"), ("dummy", "Move dummy"), ("partner", "Move 2nd dummy")] {
            let enabled = id != "partner" || setup.partner.is_some();
            ui.button_full(&format!("dojo.plan.piece.{id}"), Rect::new(bx, by, 70, 11), label, DAct::PlanPiece(id), enabled, self.plan_piece == id, SMALL, tip(label, "A click on the board moves it there."));
            by += 13;
        }
        let partner = if setup.partner.is_some() { "2nd dummy: off" } else { "2nd dummy: on" };
        ui.button_full("dojo.plan.partner", Rect::new(bx, by, 70, 11), partner, DAct::PlanPartner, true, setup.partner.is_some(), SMALL, tip("Second dummy", "A second dummy on the hero's side engages the dummy: the dummy faces it, so the hero may take its flank."));
        y = board.y + board.h + 4;
        let v = &pc.view;
        let head = if v.refused.is_empty() {
            format!("{}{} on {}: AP {} -> {}", v.intent, if v.kind.is_empty() { String::new() } else { format!(" ({})", v.kind) }, v.target, v.ap_before, v.ap_after)
        } else {
            format!("{}: not offered: {}", v.intent, v.refused)
        };
        ui.p.text_fit(x0 as f32, y as f32, &head, SMALL, fit, if v.refused.is_empty() { th().gold } else { th().foe });
        y += 9;
        for (k, r) in v.rows.iter().enumerate() {
            let blocks = if r.start == r.end { format!("b{}", r.start) } else { format!("b{}-{}", r.start, r.end) };
            let mut line = format!("{blocks} {}  {} AP", r.name, r.ap);
            if r.damage > 0 {
                line += &format!("  ~{}", r.damage);
            }
            if !r.reason.is_empty() {
                line += &format!("  [{}]", r.reason);
            }
            if !r.notes.is_empty() {
                line += &format!("  {}", r.notes.join(", "));
            }
            ui.p.text_fit(x0 as f32 + 2.0, y as f32, &line, SMALL, fit - 2.0, th().text);
            ui.push(format!("dojo.plan.row.{k}"), "row", line.clone(), Rect::new(x0, y - 1, b.w - 6, 8), true, false, None, tip(&r.name, "A planned command: its blocks, AP, the planner's damage estimate on the dummy, its reason tag (PL6) and the rules that apply."));
            y += 8;
        }
        if v.rest_blocks > 0 {
            let mut poise = if v.rest_poise > 0 { format!(", {} poise", v.rest_poise) } else { String::new() };
            // S2.2-S3: the pips the Rest restores under poise_mode pips
            if v.rest_pips > 0 {
                poise += &format!(", {} pip{}", v.rest_pips, if v.rest_pips == 1 { "" } else { "s" });
            }
            ui.p.text_fit(x0 as f32, y as f32, &format!("Rest: {} AP{poise} ({} block{})", v.rest_ap, v.rest_blocks, if v.rest_blocks == 1 { "" } else { "s" }), SMALL, fit, th().good);
            y += 9;
        }
        y += 2;
        ui.p.text(x0 as f32, y as f32, "Each intent now", SMALL, th().info);
        y += 9;
        for p in &pc.by_intent {
            let line = format!("{}: {}", p.intent, plan_line(p));
            ui.p.text_fit(x0 as f32 + 2.0, y as f32, &line, SMALL, fit - 2.0, if p.refused.is_empty() { th().text } else { th().dim });
            ui.push(format!("dojo.plan.intent_plan.{}", p.intent.to_lowercase()), "row", line.clone(), Rect::new(x0, y - 1, b.w - 6, 8), true, false, None, Vec::new());
            y += 8;
        }
        ui.scroll_end();
    }

    /// RULESET1 9 D6 (the Dojo's heroes): the active hero's deck under the deck rule: Actives, Passives and Keystone
    /// buckets (a click on a slot takes it out), the read-only Basics, the owned list (a click slots it) and the order
    /// per intent (a click moves an entry one place up); "Use default" and "Auto order". Each edit rebuilds the hero.
    fn draw_deck(&self, ui: &mut Ui<DAct>, b: Rect) {
        let x0 = b.x + 2;
        let fit = (b.w - 8) as f32;
        let Some(t) = self.deck_terms() else {
            ui.p.text_fit(x0 as f32, b.y as f32 + 2.0, "The deck applies under timeline rules (rule deck).", SMALL, fit, th().dim);
            return;
        };
        let cat = &self.s.cat;
        let draft = self.s.deck_of(self.s.active);
        let shown = t.shown(&draft);
        let name_of = |kind: &str, id: &str| -> String {
            if kind == "active" {
                cat.tier_one(id).map(|s| s.name.clone()).unwrap_or_else(|| id.to_string())
            } else {
                cat.passives.get(id).map(|p| p.name.clone()).unwrap_or_else(|| id.to_string())
            }
        };
        let owned: Vec<(&'static str, String)> = DECK_KINDS.iter().flat_map(|k| t.owned(k.0).iter().map(move |id| (k.0, id.clone()))).collect();
        // AUDIT-05 (OFFHAND1): the worn items' skills, read-only (open while worn, no slot)
        let worn: Vec<String> = self.s.hero().gear_skills.clone();
        let worn_lines = if worn.is_empty() { 0 } else { 1 + worn.len() as i32 };
        let orders: Vec<(String, Vec<String>)> = t.intents.iter().map(|i| (i.clone(), t.order(&shown, i))).collect();
        // INPUT1 FL3: the open opener's follow-up list (its header, entries, the skills to add, "Use the order")
        let open = self.follow_open.clone().filter(|op| orders.iter().any(|o| o.1.contains(op)));
        let (follow, adds) = open.as_ref().map(|op| (t.follow_list(&shown, op), t.follow_candidates(&shown, op))).unwrap_or_default();
        let follow_lines = if open.is_some() { 3 + follow.len().max(1) as i32 + adds.len() as i32 } else { 0 };
        let lines = DECK_KINDS.iter().map(|k| 1 + t.cap(k.0) as i32).sum::<i32>() + 3 + owned.len() as i32 + worn_lines + orders.iter().map(|o| 1 + o.1.len() as i32).sum::<i32>() + follow_lines + 3;
        let off = ui.scroll_begin("dojo.deck_scroll", b, lines * 9 + 20);
        let mut y = b.y + 2 - off;
        for (kind, label, takes) in DECK_KINDS {
            let list = DeckTerms::list(&shown, kind);
            ui.p.text(x0 as f32, y as f32, &format!("{label} {} / {}", list.len(), t.cap(kind)), SMALL, th().info);
            if kind == "active" {
                ui.p.text_right((b.x + b.w) as f32 - 6.0, y as f32, if draft.is_empty() { "Automatic deck" } else { "Custom deck" }, SMALL, th().dim);
            }
            y += 9;
            for i in 0..t.cap(kind) {
                let r = Rect::new(x0, y, b.w - 8, 8);
                match list.get(i) {
                    Some(id) => {
                        let n = name_of(kind, id);
                        ui.row(&format!("dojo.deck.{kind}.{i}"), r, &n, th().text, Some(DAct::DeckUnslot(kind, i)), false, tip(&n, "Slotted: a click takes it out (the last active stays: an empty row is the automatic pick)."));
                    }
                    None => ui.row(&format!("dojo.deck.{kind}.{i}"), r, "Empty", th().dim, None, false, tip(&format!("Empty {kind} slot"), &format!("Holds one of the hero's {takes}: click one in the owned list."))),
                }
                y += 9;
            }
        }
        let basics: Vec<String> = t.basics.iter().map(|f| name_of("active", f)).collect();
        ui.p.text(x0 as f32, y as f32, "Basics (no slot)", SMALL, th().info);
        y += 9;
        ui.p.text_fit(x0 as f32 + 2.0, y as f32, &basics.join(", "), SMALL, fit, th().dim);
        ui.push("dojo.deck.basics", "row", basics.join(", "), Rect::new(x0, y - 1, b.w - 8, 8), true, false, None, tip("Basics", "Open to every unit: they take no slot and cannot be taken out."));
        y += 10;
        ui.p.text(x0 as f32, y as f32, "Owned (click to slot)", SMALL, th().info);
        y += 9;
        for (kind, id) in &owned {
            let slotted = DeckTerms::list(&shown, kind).contains(id);
            let full = !slotted && DeckTerms::list(&shown, kind).len() >= t.cap(kind);
            let n = name_of(kind, id);
            let label = format!("{n} [{}]{}", kind, if slotted { " slotted" } else if full { " full" } else { "" });
            ui.row(&format!("dojo.deck.owned.{kind}.{id}"), Rect::new(x0, y, b.w - 8, 8), &label, if slotted { th().dim } else { th().text }, (!slotted && !full).then(|| DAct::DeckSlot(kind, id.clone())), slotted, tip(&n, "Click to slot it in the first free slot of its bucket."));
            y += 9;
        }
        if !worn.is_empty() {
            ui.p.text(x0 as f32, y as f32, "Worn gear (no slot)", SMALL, th().info);
            y += 9;
        }
        for id in &worn {
            let n = name_of("active", cat.family_of(id));
            ui.row(&format!("dojo.deck.worn.{id}"), Rect::new(x0, y, b.w - 8, 8), &format!("{n} [worn]"), th().gold, None, false, tip(&n, "Open while the item is worn: it takes no slot, and taking the item off removes it."));
            y += 9;
        }
        for (intent, list) in &orders {
            let name = RoundIntent::parse(intent).map(|i| i.label(cat)).unwrap_or_else(|| intent.clone());
            ui.p.text(x0 as f32, y as f32, &format!("{name} order"), SMALL, th().info);
            y += 9;
            for (k, f) in list.iter().enumerate() {
                let set = !t.follow_list(&shown, f).is_empty();
                let n = format!("{}. {}{}", k + 1, name_of("active", f), if set { " +" } else { "" });
                ui.row(&format!("dojo.deck.order.{intent}.{k}"), Rect::new(x0, y, b.w - 20, 8), &n, if t.basics.contains(f) { th().info } else { th().text }, (k > 0).then(|| DAct::OrderUp(intent.clone(), k)), false, tip(&n, "The planner fills the round from the top down: a click moves it one place up."));
                let is_open = open.as_deref() == Some(f.as_str());
                let line = format!("After {} the planner tries its follow-up list first, then the {name} order. A click opens it.", name_of("active", f));
                ui.button_full(&format!("dojo.deck.follow.{f}"), Rect::new(x0 + b.w - 19, y, 11, 8), if is_open { "v" } else { ">" }, DAct::FollowOpen(f.clone()), true, is_open, SMALL, tip("Follow-ups", &line));
                y += 9;
                if !is_open {
                    continue;
                }
                let on = name_of("active", f);
                ui.p.text_fit(x0 as f32 + 4.0, y as f32, &format!("After {on}:"), SMALL, fit - 4.0, th().gold);
                y += 9;
                if follow.is_empty() {
                    ui.p.text_fit(x0 as f32 + 6.0, y as f32, &format!("None set: the {name} order fills"), SMALL, fit - 6.0, th().dim);
                    y += 9;
                }
                for (j, g) in follow.iter().enumerate() {
                    let gn = format!("{}. {}", j + 1, name_of("active", g));
                    ui.row(&format!("dojo.deck.follow.{f}.{j}"), Rect::new(x0 + 4, y, b.w - 12, 8), &gn, th().text, Some(DAct::FollowRemove(f.clone(), j)), false, tip(&gn, "A follow-up: a click takes it out of the list."));
                    y += 9;
                }
                for g in &adds {
                    let gn = format!("+ {}", name_of("active", g));
                    ui.row(&format!("dojo.deck.follow.add.{f}.{g}"), Rect::new(x0 + 4, y, b.w - 12, 8), &gn, th().info, Some(DAct::FollowAdd(f.clone(), g.clone())), false, tip(&gn, &format!("A click adds it at the end of the follow-ups after {on}.")));
                    y += 9;
                }
                ui.button_full(&format!("dojo.deck.follow.clear.{f}"), Rect::new(x0 + 4, y, 90, 8), &format!("Use the {name} order"), DAct::FollowClear(f.clone()), !follow.is_empty(), false, SMALL, tip("Use the order", &format!("Empties the list: after {on} the planner fills from the {name} order.")));
                y += 10;
            }
        }
        y += 2;
        ui.button_full("dojo.deck.default", Rect::new(x0, y, 56, 11), "Use default", DAct::DeckDefault, !draft.is_empty(), false, SMALL, tip("Use default", "The automatic deck: the first class path skills, no passives, the automatic order."));
        ui.button_full("dojo.deck.auto_order", Rect::new(x0 + 60, y, 56, 11), "Auto order", DAct::OrderAuto, !shown.order.is_empty(), false, SMALL, tip("Auto order", "Every intent back to the automatic order: the deck's actives in slot order, then the basics."));
        ui.scroll_end();
    }

    fn draw_log(&self, ui: &mut Ui<DAct>, b: Rect) {
        let (mut x, mut y) = (b.x + 2, b.y + 1);
        for f in LOG_FILTERS.iter().copied().chain(["Clear"]) {
            let w = ui.p.text_w(f, SMALL).ceil() as i32 + 8;
            if x + w > b.x + b.w - 2 {
                (x, y) = (b.x + 2, y + 12);
            }
            if f == "Clear" {
                ui.button_full("dojo.log.clear", Rect::new(x, y, w, 10), f, DAct::LogClear, !self.s.log.is_empty(), false, SMALL, Vec::new());
            } else {
                ui.button_full(&format!("dojo.log.filter.{}", f.to_lowercase()), Rect::new(x, y, w, 10), f, DAct::LogFilter(f), true, self.log_filter == f, SMALL, Vec::new());
            }
            x += w + 2;
        }
        let top = y + 12 - b.y;
        let entries: Vec<_> = self.s.log.iter().filter(|e| self.log_filter == "ALL" || e.category == self.log_filter).collect();
        let picked = self.picked.and_then(|n| self.s.log.iter().find(|e| e.n == n));
        let detail_h = if picked.is_some() { (b.h - top) / 2 } else { 0 };
        let view = Rect::new(b.x, b.y + top, b.w, b.h - top - detail_h);
        let content = entries.len() as i32 * 18 + 2;
        // The newest entry at the bottom: a new entry scrolls the list to the end (kept while the user scrolls up).
        let off = ui.scroll_begin("dojo.log.rows", view, content);
        for (i, e) in entries.iter().enumerate() {
            let r = Rect::new(view.x + 1, view.y + 1 - off + i as i32 * 18, view.w - 5, 17);
            let sel = self.picked == Some(e.n);
            let head = format!("#{} {}{}", e.n, if e.by_dummy { "< " } else { "" }, e.title);
            let lines = [(head, category_colour(e.category)), (format!("{}  [{}] {}", e.detail, e.outcome, if e.value != 0 { e.value.to_string() } else { String::new() }), th().dim)];
            ui.push(format!("dojo.log.{}", e.n), "row", lines[0].0.clone(), r, true, sel, Some(DAct::LogPick(e.n)), Vec::new());
            if sel || ui.hover(r) {
                ui.p.panel(r, if sel { th().game("dojo_pick", rgba(86, 68, 22, 200)) } else { th().game("dojo_idle", rgba(34, 50, 82, 200)) }, if sel { th().gold } else { th().edge }, 2.0);
            }
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 1.0, &lines[0].0, SMALL, r.w as f32 - 6.0, lines[0].1);
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 9.0, &lines[1].0, SMALL, r.w as f32 - 6.0, lines[1].1);
        }
        ui.scroll_end();
        if let Some(e) = picked {
            let pane = Rect::new(b.x, b.y + b.h - detail_h, b.w, detail_h);
            ui.p.fill(Rect::new(pane.x, pane.y, pane.w, 1), th().edge);
            let text = if e.breakdown.is_empty() { format!("{}: {}", e.title, e.detail) } else { e.breakdown.clone() };
            let h = ui.p.rich_height(&text, SMALL, (pane.w - 10) as f32).ceil() as i32 + 4;
            let off = ui.scroll_begin("dojo.log.detail", Rect::new(pane.x, pane.y + 2, pane.w, pane.h - 2), h);
            ui.p.rich(pane.x as f32 + 3.0, (pane.y + 3 - off) as f32, (pane.w - 10) as f32, &text, SMALL, th().text);
            ui.scroll_end();
        }
    }

}

/// A free tile beside the dummy for the second dummy (east first, then west, north, south; else the first free tile).
fn free_beside(p: &PlanSetup) -> V2 {
    let free = |c: V2| c.x >= 0 && c.y >= 0 && c.x < GRID_W && c.y < GRID_H && c != p.hero && c != p.dummy;
    [V2::new(1, 0), V2::new(-1, 0), V2::new(0, -1), V2::new(0, 1)]
        .into_iter()
        .map(|d| V2::new(p.dummy.x + d.x, p.dummy.y + d.y))
        .find(|c| free(*c))
        .or_else(|| (0..GRID_W * GRID_H).map(|i| V2::new(i % GRID_W, i / GRID_W)).find(|c| free(*c)))
        .unwrap_or(p.dummy)
}

/// A plan in one line: its commands in order (repeats folded: "Rest x2") and the AP left, or why it is not offered.
fn plan_line(v: &PlanView) -> String {
    if !v.refused.is_empty() {
        return format!("not offered: {}", v.refused);
    }
    let mut parts: Vec<(String, i32)> = Vec::new();
    for r in &v.rows {
        match parts.last_mut() {
            Some((n, k)) if *n == r.name => *k += 1,
            _ => parts.push((r.name.clone(), 1)),
        }
    }
    let steps: Vec<String> = parts.into_iter().map(|(n, k)| if k > 1 { format!("{n} x{k}") } else { n }).collect();
    let kind = if v.kind.is_empty() { String::new() } else { format!("{} ", v.kind) };
    format!("{kind}{} (AP {} -> {})", steps.join(", "), v.ap_before, v.ap_after)
}

/// An item's numbers in one line (hover cards).
fn item_line(item: &emberwake_data::ItemDef) -> String {
    let mut parts: Vec<String> = Vec::new();
    if item.item_type == "WEAPON" {
        parts.push(format!("Power {}, range {}{}", item.base_power, item.base_range, if item.two_handed { ", 2H" } else { "" }));
    }
    if item.mitigation != 0 {
        parts.push(format!("mitigation {}", item.mitigation));
    }
    if item.burden != 0 {
        parts.push(format!("burden {}", item.burden));
    }
    parts.extend(item.stat_bonuses.iter().map(|(s, v)| format!("{v:+} {s}")));
    if !item.required_stats.is_empty() {
        parts.push(format!("needs {}", item.required_stats.iter().map(|(s, v)| format!("{v} {s}")).collect::<Vec<_>>().join(", ")));
    }
    if item.heal > 0 {
        parts.push(format!("heals {}", item.heal));
    }
    if item.damage > 0 {
        parts.push(format!("{} damage", item.damage));
    }
    parts.push(format!("weight {}", item.weight));
    parts.join("; ") + "."
}

impl Screen<GameCx> for Dojo {
    fn id(&self) -> &'static str {
        SCREEN_ID
    }

    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        for fx in self.s.fx.drain(..) {
            self.floats.push((fx, Instant::now()));
        }
        self.floats.retain(|(_, born)| born.elapsed() < FX_LIFE);
        // A new log entry scrolls the log to its end (the kit clamps the offset to the content).
        let newest = self.s.log.last().map_or(0, |e| e.n);
        if newest != self.seen {
            self.seen = newest;
            g.kit.scroll.insert("dojo.log.rows".into(), i32::MAX / 2);
        }
        if self.desk.win("planner").is_some_and(|w| w.open) && !self.s.plan_intents().is_empty() {
            self.refresh_plan();
        }
        let mut ui: Ui<DAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, aegis_ui::ui::H), th().tool_bg);
        ui.p.text(4.0, 3.0, "GEAR DOJO", BODY, th().gold);
        let mut x = 70;
        for (i, h) in self.s.heroes.iter().enumerate() {
            let w = ui.p.text_w(&h.name, SMALL).ceil() as i32 + 10;
            ui.button_full(&format!("dojo.toolbar.hero.{i}"), Rect::new(x, 2, w, 12), &h.name, DAct::Hero(i), true, i == self.s.active, SMALL, tip(&h.name, "Make this hero the active one."));
            x += w + 2;
        }
        ui.button_full("dojo.reset", Rect::new(W - 104, 2, 64, 12), "Reset dojo", DAct::Reset, true, false, SMALL, tip("Reset dojo", "Heroes, stash, loadouts, dummy and log back to the baseline."));
        ui.button_full("dojo.exit", Rect::new(W - 36, 2, 32, 12), "Exit", DAct::Exit, true, false, SMALL, Vec::new());
        self.desk.toolbar(&mut ui, 4, 16);
        let desk = self.desk.clone();
        desk.draw(&mut ui, |ui, id, body| match id {
            "party" => self.draw_party(ui, body),
            "inspector" => self.draw_inspector(ui, body),
            "derived" => self.draw_derived(ui, body),
            "gear" => self.draw_gear(ui, body),
            "stash" => self.draw_stash(ui, body),
            "stage" => self.draw_stage(ui, body),
            "console" => self.draw_console(ui, body),
            "planner" => self.draw_planner(ui, body),
            "deck" => self.draw_deck(ui, body),
            _ => self.draw_log(ui, body),
        });
        self.w = ui.widgets;
    }

    fn desk(&mut self) -> Option<&mut Desk> {
        Some(&mut self.desk)
    }

    /// GEARUI1's drops on the dojo: a stash item onto a slot it is eligible for or onto a pouch slot (a consumable),
    /// a worn or pouch item back onto the stash.
    fn can_drop(&self, _cx: &Cx, from: &str, to: &str) -> bool {
        let (kind, what) = from.split_once('.').unwrap_or((from, ""));
        let (target, at) = to.split_once('.').unwrap_or((to, ""));
        match (kind, target) {
            ("stash", "slot") => self.s.eligible(at).iter().any(|i| i == what),
            ("stash", "pouch") => self.s.cat.items.get(what).is_some_and(|i| i.item_type == "CONSUMABLE") && self.s.hero().inventory.pouch.len() < self.s.hero().inventory.max_pouch_slots,
            ("equip", "stash") | ("pouch", "stash") => true,
            _ => false,
        }
    }

    fn accept_drop(&mut self, _cx: &mut Cx, from: &str, to: &str) {
        self.rev += 1;
        let (kind, what) = from.split_once('.').unwrap_or((from, ""));
        let (target, at) = to.split_once('.').unwrap_or((to, ""));
        match (kind, target) {
            ("stash", "slot") => {
                self.s.equip(what, at);
            }
            ("stash", "pouch") => {
                self.s.add_to_pouch(what);
            }
            ("equip", "stash") => self.s.unequip(SLOTS.iter().find(|(k, _)| *k == what).map(|(k, _)| *k).unwrap_or("main_hand")),
            ("pouch", "stash") => self.s.remove_from_pouch(what.parse().unwrap_or(usize::MAX)),
            _ => {}
        }
    }

    fn state(&self, _cx: &Cx) -> Value {
        let h = self.s.hero();
        let mut out = json!({
            "hero": h.name, "hp": h.hp, "ap": h.ap, "stance": stance_label(h.stance),
            "dummy_hp": self.s.dummy.hp, "round": self.s.round, "skill": self.s.skill, "dummy_skill": self.s.dummy_skill,
            "force_crit": self.s.force_crit, "counter": self.s.dummy_counter, "stash": self.s.stash.len(),
            "gear": SLOTS.iter().map(|(k, s)| (k.to_string(), json!(h.inventory.get(*s).map(|e| e.item.id.clone())))).collect::<serde_json::Map<String, Value>>(),
            "pouch": h.inventory.pouch.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
            "log": self.s.log.iter().rev().take(5).map(|e| json!({"n": e.n, "title": e.title, "detail": e.detail, "outcome": e.outcome, "value": e.value})).collect::<Vec<_>>(),
        });
        // CLN-46: the ruleset always; RULESET1-G: the deck readouts where the rules have a deck
        out["ruleset"] = json!(self.s.ruleset.id);
        if self.s.ruleset.has(emberwake_rules::ruleset::DECK) || self.s.ruleset.has(emberwake_rules::ruleset::ROUND_INTENT) {
            let deck = self.s.deck_of(self.s.active);
            out["deck"] = crate::party::deck_json(&deck);
            out["actives"] = json!(h.active_families());
            if self.s.poise_on() {
                let (p, d) = (self.s.poise(), self.s.dummy_poise);
                out["poise"] = json!([p.value, p.max]);
                out["dummy_poise"] = json!([d.value, d.max, d.broken]);
                // S2.2-S3: the pips under poise_mode pips
                if emberwake_battle::poise::pips_mode(&self.s.ruleset.numbers) {
                    out["pips"] = json!([p.pips, self.s.ruleset.numbers.poise_pips]);
                    out["dummy_pips"] = json!([d.pips, self.s.ruleset.numbers.poise_pips]);
                }
                out["dummy_windup"] = json!(self.s.dummy_windup);
            }
            // the plan as it stands now, the planner window open or not (the window's cache when it is current)
            let fresh;
            let current = match &self.plan {
                Some(pc) if pc.key == self.plan_key() => Some(pc),
                _ => {
                    fresh = self.plan_now();
                    fresh.as_ref()
                }
            };
            if let Some(pc) = current {
                let v = &pc.view;
                out["plan"] = json!({
                    "intent": v.intent, "kind": v.kind, "target": v.target, "refused": v.refused, "ap_before": v.ap_before, "ap_after": v.ap_after,
                    "rest": [v.rest_ap, v.rest_poise], "rest_pips": v.rest_pips,
                    "rows": v.rows.iter().map(|r| json!({"skill": r.skill, "start": r.start, "end": r.end, "ap": r.ap, "reason": r.reason, "damage": r.damage, "notes": r.notes})).collect::<Vec<_>>(),
                    "by_intent": pc.by_intent.iter().map(|p| (p.intent.to_lowercase(), json!(plan_line(p)))).collect::<serde_json::Map<String, Value>>(),
                });
            }
        }
        out
    }

    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use emberwake_rules::ruleset::{CLASSIC, TIMELINE};
    use std::sync::Arc;

    /// Picks the catalog's ruleset row `id` in the Dojo's picker.
    fn pick(cx: &mut Cx, dojo: &mut Dojo, id: &str) {
        let at = emberwake_rules::ruleset::Ruleset::ids(&cx.data.cat).iter().position(|r| r == id).expect("the row");
        dojo.act(cx, DAct::Ruleset(at));
    }

    /// INPUT1 FL3 (the Dojo's mirror of the party screen's terms): an order entry's toggle opens its follow-up list; a
    /// click adds a skill of the intent (the hero is rebuilt with the list), a click on an entry takes it out, "Use the
    /// order" empties it.
    #[test]
    fn the_deck_window_edits_follow_up_lists_and_rebuilds_the_hero() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut dojo = Dojo::new(&cx);
        pick(&mut cx, &mut dojo, TIMELINE);
        let t = dojo.deck_terms().expect("the timeline row has the deck rule");
        let shown = t.shown(&dojo.s.deck_of(dojo.s.active));
        let order = t.intents.iter().map(|i| t.order(&shown, i)).max_by_key(Vec::len).unwrap_or_default();
        let op = order.first().cloned().expect("an order entry");
        let mut g = crate::hud::hud_tests::gfx();
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(dojo.w.iter().any(|w| w.id == format!("dojo.deck.follow.{op}")), "the entry's toggle");
        dojo.act(&mut cx, DAct::FollowOpen(op.clone()));
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        let add = format!("dojo.deck.follow.add.{op}.{}", order.last().unwrap());
        assert!(dojo.w.iter().any(|w| w.id == add) && dojo.w.iter().any(|w| w.id == format!("dojo.deck.follow.clear.{op}")), "the open list");
        dojo.act(&mut cx, DAct::FollowAdd(op.clone(), order.last().unwrap().clone()));
        assert_eq!(dojo.s.hero().deck.follow_ups.get(&op), Some(&vec![order.last().unwrap().clone()]), "the hero fights with the list");
        dojo.act(&mut cx, DAct::FollowAdd(op.clone(), op.clone()));
        dojo.act(&mut cx, DAct::FollowRemove(op.clone(), 0));
        assert_eq!(dojo.s.deck_of(dojo.s.active).follow_ups.get(&op), Some(&vec![op.clone()]));
        dojo.act(&mut cx, DAct::FollowClear(op.clone()));
        assert!(!dojo.s.hero().deck.follow_ups.contains_key(&op), "Use the order empties it");
    }

    /// R1-BAL (FB-80): a worn item the hero wears under a wield penalty (here a stat deficit, on a test-only copy of a
    /// weapon that asks for more than any stat holds) gets the party screen's red "x0.75"-style badge right-aligned in
    /// its gear slot row and the penalty sentence on its card; a copy with no requirement and no weapon type shows none.
    #[test]
    fn a_penalized_worn_item_shows_the_wield_badge_in_the_gear_slot() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut weapons: Vec<String> = cx.data.cat.items.iter().filter(|(_, i)| item_fits_slot(i, "main_hand") && i.item_type == "WEAPON").map(|(id, _)| id.clone()).collect();
        weapons.sort();
        let base = cx.data.cat.items[&weapons[0]].clone();
        for penalized in [true, false] {
            let mut dojo = Dojo::new(&cx);
            let mut item = base.clone();
            item.required_stats.clear();
            if penalized {
                item.required_stats.insert(STATS[0].to_string(), 999);
            } else {
                item.weapon_type.clear();
            }
            let cat = dojo.s.cat.clone();
            let a = dojo.s.active;
            dojo.s.heroes[a].inventory.equip(emberwake_rules::gear::Equipment::new(&item), emberwake_rules::gear::Slot::MainHand);
            dojo.s.heroes[a].sync_from_stats_and_gear(&cat);
            let pen = emberwake_rules::gear::Equipment::new(&item).wield_penalty(&dojo.s.hero().stats);
            assert_eq!(pen.untrained || pen.deficit > 0, penalized, "the fixture's penalty");
            let badge = format!("x{:.2}", pen.factor());
            let recs = crate::hud::hud_tests::frame(&mut dojo, &mut cx, &mut crate::hud::hud_tests::gfx(), 1.0);
            let slot = dojo.w.iter().find(|w| w.id == "dojo.slot.main_hand").expect("the main hand slot row");
            let card: String = slot.tip.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join(" ");
            let r = slot.rect;
            let drawn = recs.iter().find(|t| t.text == badge && t.y >= r.y as f32 && t.y < (r.y + r.h) as f32 && t.x + t.w <= (r.x + r.w) as f32 && t.x > (r.x + r.w / 2) as f32);
            assert_eq!(drawn.is_some(), penalized, "the badge {badge} in the slot row {r:?}");
            assert_eq!(card.contains("Wield penalty"), penalized, "the card: {card}");
        }
    }

    /// AUDIT-05 (OFFHAND1): the Deck window lists the hero's worn items' skills read-only (no slot action) after the
    /// owned list. The skill effect is test-only, on this test's own data.
    #[test]
    fn the_deck_window_lists_worn_gear_skills_read_only() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut rings: Vec<String> = cx.data.cat.items.iter().filter(|(_, i)| i.item_type == "ACCESSORY").map(|(id, _)| id.clone()).collect();
        rings.sort();
        let ring = rings[0].clone();
        let mut dojo = Dojo::new(&cx);
        pick(&mut cx, &mut dojo, TIMELINE);
        let owned = dojo.s.hero().owned_families(&dojo.s.cat);
        let skill = cx.data.cat.skills.iter().find(|s| s.tier == 1 && !s.basic && !s.system && !owned.contains(&s.family_id().to_string())).map(|s| s.id.clone()).expect("a skill the hero does not own");
        let mut item = cx.data.cat.items[&ring].clone();
        item.effects.push(json!({"type": "skill", "skill": skill}));
        let cat = dojo.s.cat.clone();
        let a = dojo.s.active;
        dojo.s.heroes[a].inventory.equip(emberwake_rules::gear::Equipment::new(&item), emberwake_rules::gear::Slot::Accessory1);
        dojo.s.heroes[a].sync_from_stats_and_gear(&cat);
        assert!(dojo.s.hero().gear_skill(&skill));
        let mut g = crate::hud::hud_tests::gfx();
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        let w = dojo.w.iter().find(|w| w.id == format!("dojo.deck.worn.{skill}")).expect("the worn skill's row");
        assert!(w.action.is_none(), "read-only: no slot");
    }

    /// DERIVE1 4.4: the Derived Stats window lists the catalog rows shown on the Dojo after the built-in numbers, each
    /// hovering to its formula and breakdown; a row for other screens only, or no row, adds nothing.
    #[test]
    fn the_derived_window_lists_dojo_rows_with_their_breakdown() {
        for rows in [json!([]), json!([{"id": "nerve", "name": "Nerve", "formula": "KNOW * 2", "show_on": ["dojo"]}, {"id": "zeal", "name": "Zeal", "formula": "DEX", "show_on": ["party"]}])] {
            let mut cx = crate::hud::hud_tests::game_cx();
            let mut data = (*cx.app.data).clone();
            data.edit_cat(|cat| cat.derived_stats = rows.as_array().unwrap().iter().filter_map(|r| emberwake_data::DerivedStatDef::from_obj(r.as_object().unwrap())).map(|r| (r.id.clone(), r)).collect());
            cx.app.data = Arc::new(data);
            let mut dojo = Dojo::new(&cx);
            let mut g = crate::hud::hud_tests::gfx();
            dojo.draw(&mut cx, &mut g, (-500, -500), true);
            let shown: Vec<&Widget<DAct>> = dojo.w.iter().filter(|w| w.id.starts_with("dojo.derived.")).collect();
            if rows.as_array().unwrap().is_empty() {
                assert!(shown.is_empty());
                continue;
            }
            let v = emberwake_rules::derived::value(&dojo.s.cat, &dojo.s.ruleset.id, dojo.s.hero(), "nerve").unwrap();
            assert_eq!(shown.iter().map(|w| (w.id.as_str(), w.text.clone())).collect::<Vec<_>>(), [("dojo.derived.Nerve", format!("Nerve {v}"))], "the party-only row is not listed");
            assert!(shown[0].tip[1].0.starts_with("Formula: KNOW * 2. KNOW "), "{:?}", shown[0].tip);
        }
    }

    /// RULESET1-G (tasks 4.1 G): Timeline brings up the Planner (one plan per intent of the ruleset, the planned rows)
    /// and the Deck window; a click on an owned active slots it (the hero is rebuilt with that deck), a passive alone
    /// leaves the actives automatic (user 2026-10-29); back to classic the planner plans nothing and the state keeps
    /// its classic shape.
    #[test]
    fn timeline_brings_up_the_planner_and_the_deck_view() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut dojo = Dojo::new(&cx);
        assert!(dojo.state(&cx).get("plan").is_none());
        pick(&mut cx, &mut dojo, TIMELINE);
        // a timeline world needs the row (a content error without it), so the data has one with round intents
        assert!(dojo.s.ruleset.id == TIMELINE && !dojo.s.plan_intents().is_empty(), "the catalog's timeline row with round intents");
        let mut g = crate::hud::hud_tests::gfx();
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(dojo.desk.win("planner").is_some_and(|w| w.open) && dojo.desk.win("deck").is_some_and(|w| w.open));
        let st = dojo.state(&cx);
        assert_eq!(st["plan"]["by_intent"].as_object().map(|m| m.len()), Some(dojo.s.plan_intents().len()), "one plan per intent");
        for it in dojo.s.plan_intents() {
            assert!(dojo.w.iter().any(|w| w.id == format!("dojo.plan.intent.{}", it.id())), "{} button", it.id());
        }
        let rows = st["plan"]["rows"].as_array().map_or(0, |r| r.len());
        assert_eq!(dojo.w.iter().filter(|w| w.id.starts_with("dojo.plan.row.")).count(), rows, "a widget per planned row");
        if let Some(t) = dojo.deck_terms() {
            if let Some(p) = t.passives.first().cloned() {
                dojo.act(&mut cx, DAct::DeckSlot("passive", p.clone()));
                let d = dojo.s.deck_of(dojo.s.active);
                assert!(d.passive.contains(&p) && d.active.is_empty(), "a passive alone pins no active: {d:?}");
            }
            let shown = t.shown(&dojo.s.deck_of(dojo.s.active));
            if let Some(f) = t.actives.iter().find(|f| !shown.active.contains(f)).cloned().filter(|_| shown.active.len() < t.cap("active")) {
                dojo.act(&mut cx, DAct::DeckSlot("active", f.clone()));
                assert!(dojo.s.hero().active_families().contains(&f), "the slotted active is the hero's");
            }
        }
        pick(&mut cx, &mut dojo, CLASSIC);
        assert!(dojo.s.plan_intents().is_empty() && dojo.state(&cx).get("plan").is_none());
    }

    /// A4.2-K1b (row W2-4: the Dojo Planner shows only the valid IR21 intent variants of the selected skill): under
    /// `field_intent` the planner offers Field; with a dual opener (a thrown BURST built here on the Dojo's catalog) only
    /// its rows are open and the others locked, switching between them keeps the opener, and the variant previews are one
    /// plan per row, each opening with it. Without an opener every intent is open.
    #[test]
    fn a_dual_opener_opens_only_its_rows() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut dojo = Dojo::new(&cx);
        pick(&mut cx, &mut dojo, TIMELINE);
        assert!(dojo.s.plan_intents().contains(&RoundIntent::Field), "field_intent: the planner offers Field");
        assert!(dojo.s.plan_variants().iter().all(|v| v.1), "no opener: every intent open");
        dojo.s.set_plan_intent(RoundIntent::Assault);
        let id = dojo.s.plan_openers().into_iter().map(|o| o.0).find(|id| dojo.s.cat.skill(id).is_some_and(|s| s.kind == "ATTACK" && !emberwake_battle::skill::is_area(s) && s.position.is_none() && s.dash_tiles == 0 && s.hazard.is_none())).expect("a plain attack opener");
        {
            let cat = Arc::make_mut(&mut dojo.s.cat);
            let k = cat.skills.iter().position(|s| s.id == id).unwrap();
            let sk = &mut cat.skills[k];
            sk.area = "BURST".into();
            sk.reach_min = 2;
            sk.reach_max = 3;
            sk.power_mult = sk.power_mult.max(0.5);
        }
        dojo.s.planner.opener = Some(id.clone());
        let open: Vec<RoundIntent> = dojo.s.plan_variants().into_iter().filter(|v| v.1).map(|v| v.0).collect();
        assert_eq!(open, [RoundIntent::Assault, RoundIntent::Field], "only the opener's rows (in the planner's order)");
        let views = dojo.s.plans_by_intent();
        assert_eq!(views.iter().map(|v| v.intent.clone()).collect::<Vec<_>>(), ["Assault", "Field"], "one variant preview per row");
        dojo.s.set_plan_intent(RoundIntent::Field);
        assert_eq!(dojo.s.planner.opener.as_deref(), Some(id.as_str()), "switching to another row keeps the skill");
        dojo.s.set_plan_intent(RoundIntent::Guard);
        assert_eq!(dojo.s.planner.opener, None, "a row the skill lacks drops it");
        let mut g = crate::hud::hud_tests::gfx();
        dojo.s.planner.opener = Some(id);
        dojo.s.planner.intent = RoundIntent::Field;
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(dojo.w.iter().any(|w| w.id == "dojo.plan.intent.field"), "a Field button");
    }

    /// RULESET1-G review: a change that logs nothing (the dummy's power) still moves the planner cache's key, and the
    /// state's plan follows the setup with the planner window closed (it is computed when the cache is stale).
    #[test]
    fn the_plan_state_follows_changes_that_log_nothing_and_a_closed_window() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut dojo = Dojo::new(&cx);
        pick(&mut cx, &mut dojo, TIMELINE);
        let intents = dojo.s.plan_intents();
        assert!(intents.len() > 1, "the catalog's timeline row offers several intents");
        let mut g = crate::hud::hud_tests::gfx();
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        let (key, logged) = (dojo.plan_key(), dojo.s.log.len());
        dojo.act(&mut cx, DAct::DummyPower(5));
        assert_eq!(dojo.s.log.len(), logged, "the dummy's power logs nothing");
        assert_ne!(dojo.plan_key(), key, "yet the planner's key moves");
        dojo.desk.set_open("planner", false);
        let other = intents.iter().position(|i| *i != dojo.s.planner.intent).unwrap();
        dojo.act(&mut cx, DAct::PlanIntent(other));
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(dojo.state(&cx)["plan"]["intent"], json!(intents[other].name()), "the closed window's plan is current in the state");
    }

    /// DERIVE1 4.4: with several catalog rows the Derived Stats window closes its rows up and scrolls the rest, in
    /// every layout: each row's hover rect lies inside the window body, no two overlap, and scrolled to the end the
    /// last row is on screen.
    #[test]
    fn many_derived_rows_stay_inside_the_window_without_overlapping() {
        const N: usize = 6;
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut data = (*cx.app.data).clone();
        data.edit_cat(|cat| cat.derived_stats = (0..N).map(|i| emberwake_data::DerivedStatDef::from_obj(json!({"id": format!("row_{i}"), "name": format!("Row {i}"), "formula": "KNOW", "show_on": ["dojo"]}).as_object().unwrap()).unwrap()).map(|r| (r.id.clone(), r)).collect());
        cx.app.data = Arc::new(data);
        let inside = |r: Rect, b: Rect| r.x >= b.x && r.y >= b.y && r.x + r.w <= b.x + b.w && r.y + r.h <= b.y + b.h;
        for layout in layouts() {
            let mut dojo = Dojo::new(&cx);
            assert!(dojo.desk.apply_layout(&layout.name), "{}", layout.name);
            let body = dojo.desk.body("derived").expect("the Derived Stats window");
            for end in [false, true] {
                let mut g = crate::hud::hud_tests::gfx();
                if end {
                    g.kit.scroll.insert("dojo.derived_scroll".into(), i32::MAX / 2);
                }
                dojo.draw(&mut cx, &mut g, (-500, -500), true);
                let rows: Vec<Rect> = dojo.w.iter().filter(|w| w.id.starts_with("dojo.derived.Row")).map(|w| w.rect).filter(|r| r.w > 0 && r.h > 0).collect();
                assert!(!rows.is_empty(), "{}: rows on screen", layout.name);
                for (i, r) in rows.iter().enumerate() {
                    assert!(inside(*r, body), "{}: {r:?} inside {body:?}", layout.name);
                    for o in &rows[i + 1..] {
                        let x = r.intersect(*o);
                        assert!(x.w == 0 || x.h == 0, "{}: {r:?} overlaps {o:?}", layout.name);
                    }
                }
                if end {
                    let last = dojo.w.iter().find(|w| w.id == format!("dojo.derived.Row {}", N - 1)).unwrap().rect;
                    assert!(last.h > 0 && inside(last, body), "{}: scrolled to the end the last row shows ({last:?})", layout.name);
                }
            }
        }
    }

    /// CLN-46: the Dojo's ruleset picker lists the catalog's rows (a third appears), picking one plays by it; Reset
    /// keeps the ruleset and rebuilds the decks only where the rules have a deck; the state names the ruleset always.
    #[test]
    fn the_ruleset_picker_lists_the_rows_and_reset_reads_the_deck_rule() {
        let mut cx = crate::hud::hud_tests::game_cx();
        {
            let d = Arc::make_mut(&mut cx.data);
            let mut row = d.cat.rulesets.get(CLASSIC).cloned().expect("the classic row");
            row.insert("id".into(), json!("plain"));
            row.insert("name".into(), json!("Plain"));
            row.insert("rest_ap".into(), json!("on_use"));
            Arc::make_mut(&mut d.cat).rulesets.insert("plain".into(), row);
        }
        let mut dojo = Dojo::new(&cx);
        let mut g = crate::hud::hud_tests::gfx();
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        let sel = dojo.w.iter().find(|w| w.id == "dojo.ruleset").expect("the picker");
        assert!(sel.enabled && sel.text.contains("Classic"), "{}", sel.text);
        assert!(dojo.w.iter().filter(|w| w.id.starts_with("dojo.")).all(|w| w.text != "Timeline"), "no Timeline toggle any more");
        assert_eq!(dojo.state(&cx)["ruleset"], json!(CLASSIC), "the state names the ruleset in classic too");
        pick(&mut cx, &mut dojo, "plain");
        assert_eq!(dojo.s.ruleset.id, "plain");
        assert!(!dojo.s.ruleset.has(emberwake_rules::ruleset::DECK) && dojo.state(&cx).get("deck").is_none());
        dojo.act(&mut cx, DAct::HealHero);
        dojo.s.reset();
        assert_eq!(dojo.s.ruleset.id, "plain", "reset keeps the ruleset");
        pick(&mut cx, &mut dojo, TIMELINE);
        dojo.s.reset();
        assert!(dojo.s.ruleset.has(emberwake_rules::ruleset::DECK) && dojo.state(&cx).get("deck").is_some(), "a deck rule: decks rebuilt and shown");
    }

    /// PK1-KITFIX (c): Quick setup lists the built-in presets with the catalog's dojo_presets rows over them; picking a
    /// data row applies it (the hero holds its passive). The row is test-only, on this test's own copy of the data; its
    /// passive is the first passive id in order.
    #[test]
    fn quick_setup_lists_and_applies_data_presets() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut ids: Vec<String> = cx.data.cat.passives.keys().cloned().collect();
        ids.sort();
        let passive = ids.first().cloned().expect("a passive");
        // the list before the test row: the built-in presets and the shipped rows, however many ship
        let baseline = Dojo::new(&cx).s.presets().len();
        assert!(baseline >= emberwake_battle::dojo::PRESETS.len(), "the built-in presets are listed");
        let mut data = (*cx.app.data).clone();
        let row = emberwake_data::DojoPresetDef::from_obj(json!({"id": "t_gui_preset", "name": "GUI Test Preset", "passives": [passive]}).as_object().unwrap());
        data.edit_cat(|cat| {
            cat.dojo_presets.insert(row.id.clone(), row.clone());
        });
        cx.app.data = Arc::new(data);
        let mut dojo = Dojo::new(&cx);
        let listed = dojo.s.presets();
        assert_eq!(listed.len(), baseline + 1, "the list before it and the row");
        let at = listed.iter().position(|p| p.id == "t_gui_preset").expect("the row is listed");
        let mut g = crate::hud::hud_tests::gfx();
        dojo.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(dojo.w.iter().any(|w| w.id.starts_with("dojo.preset")), "the Quick setup list");
        dojo.act(&mut cx, DAct::Preset(at));
        assert!(dojo.s.hero().passives.contains(&passive), "the hero holds the row's passive {passive}");
    }
}
