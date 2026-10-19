//! CombatPlan, PlanStep, PlanEntry (scripts/combat/planning/). Each unit's round is an ordered list of steps,
//! compiled by simulating the unit (position, stance, AP, blocks, target lock, expected damage) and placing every
//! command at the lowest free block. Like Godot, compiling may write a chosen approach or landing tile back into
//! the step (`PlanStep::cell`), and steps are shared by reference.

use crate::command::{self, cover_for, dash_fits, CmdKind, Command};
use crate::grid::{V2, DIRS, NO_CELL, ZERO};
use crate::path::{self, Cells, Hooks};
use crate::resolver::Resolver;
use crate::skill::{self, Kind, SkillId};
use crate::world::{UnitId, World};
use aegis_data::ItemDef;
use aegis_rules::damage::{self, HitInput};
use aegis_rules::sort::godot_sort;
use aegis_rules::{status, Stance};
use indexmap::{IndexMap, IndexSet};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub const MAX_MOVE_BLOCKS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Engage,
    Move,
    Goto,
    Area,
    Withdraw,
    SelfCast,
    Ally,
    Item,
}

#[derive(Debug)]
pub struct PlanStep {
    pub mode: Mode,
    pub skill: Option<SkillId>,
    pub target: Option<UnitId>,
    pub direction: V2,
    pub tiles: i32,
    /// GOTO / AREA destination, or the approach / landing tile fixed when first compiled.
    pub cell: Cell<V2>,
    /// ITEM: the catalog consumable id used from the battle pouch.
    pub item_id: String,
}

pub type Step = Rc<PlanStep>;

impl PlanStep {
    fn make(mode: Mode, skill: Option<SkillId>, target: Option<UnitId>, cell: V2) -> Step {
        Rc::new(PlanStep { mode, skill, target, direction: ZERO, tiles: 1, cell: Cell::new(cell), item_id: String::new() })
    }
    pub fn engage(skill: SkillId, target: UnitId) -> Step {
        Self::make(Mode::Engage, Some(skill), Some(target), NO_CELL)
    }
    pub fn mv(skill: SkillId, direction: V2, tiles: i32) -> Step {
        Rc::new(PlanStep { mode: Mode::Move, skill: Some(skill), target: None, direction, tiles, cell: Cell::new(NO_CELL), item_id: String::new() })
    }
    pub fn go_to(skill: SkillId, cell: V2) -> Step {
        Self::make(Mode::Goto, Some(skill), None, cell)
    }
    pub fn area_at(skill: SkillId, cell: V2) -> Step {
        Self::make(Mode::Area, Some(skill), None, cell)
    }
    pub fn withdraw() -> Step {
        Self::make(Mode::Withdraw, None, None, NO_CELL)
    }
    pub fn ally(skill: SkillId, target: UnitId) -> Step {
        Self::make(Mode::Ally, Some(skill), Some(target), NO_CELL)
    }
    pub fn self_skill(skill: SkillId) -> Step {
        Self::make(Mode::SelfCast, Some(skill), None, NO_CELL)
    }
    /// ITEMUSE1 1.3: use pouch item `item_id` on `target` (the user itself for self items). Skill-less step.
    pub fn use_item(item_id: &str, target: UnitId) -> Step {
        Rc::new(PlanStep { mode: Mode::Item, skill: None, target: Some(target), direction: ZERO, tiles: 1, cell: Cell::new(NO_CELL), item_id: item_id.to_string() })
    }
}

#[derive(Debug, Clone)]
pub struct PlanEntry {
    pub skill: SkillId,
    pub start_block: i32,
    pub direction: V2,
    pub path: Vec<V2>,
    pub tiles: i32,
    pub target: Option<UnitId>,
    pub step_index: i32,
    pub pos_before: V2,
    pub pos_after: V2,
    pub goal: V2,
    pub skill_ap: i32,
    pub blocks_override: i32,
    pub est_damage: i32,
    pub est_heal: i32,
    pub cells: Vec<V2>,
    pub aim: V2,
    pub hits_allies: i32,
    pub status_name: String,
    pub chase: bool,
    pub switch_ap: i32,
    pub stance_before: Stance,
    pub stance_after: Stance,
    /// ITEM (ITEMUSE1): the pouch consumable used (`skill` is its synthetic skill).
    pub item: Option<ItemDef>,
}

impl PlanEntry {
    fn new(skill: SkillId, start: i32, step_index: i32) -> Self {
        Self {
            skill,
            start_block: start,
            direction: ZERO,
            path: Vec::new(),
            tiles: 0,
            target: None,
            step_index,
            pos_before: NO_CELL,
            pos_after: NO_CELL,
            goal: NO_CELL,
            skill_ap: 0,
            blocks_override: 0,
            est_damage: 0,
            est_heal: 0,
            cells: Vec::new(),
            aim: NO_CELL,
            hits_allies: 0,
            status_name: String::new(),
            chase: false,
            switch_ap: 0,
            stance_before: Stance::Neutral,
            stance_after: Stance::Neutral,
            item: None,
        }
    }

    pub fn total_ap(&self) -> i32 {
        self.skill_ap + self.switch_ap
    }

    pub fn blocks(&self, w: &World) -> i32 {
        if self.blocks_override > 0 {
            self.blocks_override
        } else {
            w.cat.skills[self.skill].blocks
        }
    }

    pub fn end_block(&self, w: &World) -> i32 {
        self.start_block + self.blocks(w) - 1
    }

    /// PlanEntry.build_command: the real command for the resolver.
    pub fn build_command(&self, w: &World, actor: UnitId) -> Command {
        let s = &w.cat.skills[self.skill];
        let u = &w.units[actor];
        let blocks = self.blocks(w);
        let mut cmd = match skill::kind(s) {
            // ItemCommand: the SupportCommand slot (priority 0), `blocks` long
            _ if self.item.is_some() => {
                Command::new(actor, 0, blocks, CmdKind::Item { target: self.target.unwrap_or(actor), item: Box::new(self.item.clone().unwrap()), used: false, last_heal: 0, last_damage: 0, last_boost: 0.0 })
            }
            Kind::Attack => {
                let mut c = if skill::is_area(s) {
                    Command::new(actor, 0, s.blocks, CmdKind::Area { cells: self.cells.clone(), aim: self.aim, skill_mult: u.power_for(s), status_rounds: s.status_rounds, last_damage: 0, last_hits: 0, crit_hits: Vec::new() })
                } else {
                    let (lo, hi) = skill::reach_for(s, u);
                    Command::new(
                        actor,
                        0,
                        s.blocks,
                        CmdKind::Attack {
                            target: self.target,
                            skill_mult: u.power_for(s),
                            use_offhand: skill::is_off_hand(s),
                            follow_tiles: if self.chase && s.dash_tiles <= 0 { 1 } else { 0 },
                            dash_tiles: s.dash_tiles,
                            status_rounds: s.status_rounds,
                            min_range: lo,
                            max_range: hi,
                            last_damage: 0,
                            last_crit: false,
                            last_target: None,
                        },
                    )
                };
                c.status_effect = s.status_effect.clone();
                c.status_potency = s.status_potency + if s.status_effect.is_empty() { 0 } else { u.status_potency_bonus() };
                c.status_magnitude = s.status_magnitude;
                c
            }
            Kind::Status => Command::new(actor, 0, 1, CmdKind::StatusBlock { status: self.status_name.clone() }),
            Kind::Support => Command::new(actor, 0, s.blocks, CmdKind::Support { target: self.target.unwrap_or(actor), skill: self.skill, last_heal: 0 }),
            Kind::Defend => {
                let mut c = Command::new(actor, 1, s.blocks, CmdKind::Defend { mitigation: s.mitigation, grant_status: s.grant_status.clone(), status_rounds: s.status_rounds, extra_grants: s.extra_grants.clone(), guard_broken: false });
                c.status_magnitude = s.status_magnitude;
                c
            }
            Kind::Focus => Command::new(actor, 0, s.blocks, CmdKind::Focus { boost: s.boost, rounds: s.boost_rounds }),
            Kind::Move => Command::new(actor, 0, blocks, CmdKind::Walk { path: self.path.clone(), expected_start: self.pos_before, goal: self.goal, moved_tiles: 0, cut_short: false }),
            Kind::Disengage => Command::new(actor, 0, s.blocks, CmdKind::Disengage { dest: self.pos_after, max_tiles: s.jump_tiles, landed: false }),
            Kind::Target => Command::new(actor, 2, 1, CmdKind::Target { target: self.target }),
            Kind::Rest => Command::new(actor, 0, s.blocks, CmdKind::Rest { recover_ap: s.recover_ap }),
        };
        cmd.name = s.name.clone();
        cmd.ap_cost = self.skill_ap;
        cmd.stance_switch_cost = self.switch_ap;
        cmd.stance_on_start = if skill::keeps_stance(s) { None } else { Some(skill::stance(s)) };
        cmd.stance_on_end = skill::ends_in_stance(s);
        if skill::kind(s) == Kind::Attack {
            cmd.disrupt_break = skill::is_stance_break(s);
            cmd.exploit = s.exploit.clone();
            cmd.exploit_consume = s.exploit_consume;
        }
        cmd.est_damage = self.est_damage;
        cmd
    }
}

#[derive(Debug, Clone)]
pub struct SimState {
    pub pos: V2,
    pub stance: Stance,
    pub ap_left: i32,
    pub max_ap: i32,
    pub total_blocks: i32,
    pub next_block: i32,
    pub blocked_blocks: IndexMap<i32, String>,
    pub rooted: bool,
    pub withdrawn: bool,
    pub acted: bool,
    pub moved: bool,
    pub positioned: bool,
    pub lock: Option<UnitId>,
    pub est_hp: HashMap<UnitId, i32>,
    pub consumed: HashMap<UnitId, Vec<String>>,
    pub entries: Vec<PlanEntry>,
    pub error_index: i32,
    pub error_text: String,
    pub dry_entry: i32,
    pub dry_step: i32,
}

impl SimState {
    pub fn blocks_left(&self) -> i32 {
        (self.next_block..=self.total_blocks).filter(|b| !self.blocked_blocks.contains_key(b)).count() as i32
    }
    pub fn runs_dry(&self) -> bool {
        self.ap_left < 0
    }
    pub fn est(&self, u: UnitId) -> i32 {
        self.est_hp.get(&u).copied().unwrap_or(0)
    }
}

/// A unit's opening move: destination (None = withdraws), end block, block it leaves its tile.
#[derive(Debug, Clone, Copy)]
struct FirstMove {
    to: V2,
    leave: i32,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub sequences: IndexMap<UnitId, Vec<Step>>,
    /// AI drafts: units whose routes pay the hazard penalty, and ranged units' cover preference (the shooters).
    pub path_penalties: IndexSet<UnitId>,
    pub goal_preferences: IndexMap<UnitId, Vec<UnitId>>,
    pub ai_pressure: IndexSet<UnitId>,
    blocked_override: RefCell<Option<Rc<Cells>>>,
}

pub const AI_HAZARD_PENALTY: i32 = 3;

/// The AI cover preference: the cover a cell gives against the ranged foe nearest to it.
pub fn cover_pref(w: &World, shooters: &[UnitId], cell: V2) -> f64 {
    let mut nearest = None;
    let mut best = 1 << 20;
    for &f in shooters {
        let d = cell.dist(w.pos(f));
        if d < best {
            best = d;
            nearest = Some(f);
        }
    }
    nearest.map(|f| cover_for(&w.grid, w.pos(f), cell, true)).unwrap_or(0.0)
}

impl Plan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_move_tiles(&self, w: &World, c: UnitId) -> i32 {
        w.units[c].move_per_block() * MAX_MOVE_BLOCKS as i32
    }

    pub fn fits_move(&self, w: &World, c: UnitId, from: V2, route: &[V2]) -> bool {
        path::blocks_needed(&w.grid, from, route, w.units[c].move_per_block()) <= MAX_MOVE_BLOCKS
    }

    // ---- Sequence editing ---------------------------------------------------------------------------------------

    pub fn steps_of(&self, c: UnitId) -> Vec<Step> {
        self.sequences.get(&c).cloned().unwrap_or_default()
    }

    /// Appends `step` if it compiles (and, with allow_dry false, does not run into AP debt); "" or the reason.
    pub fn add_step(&mut self, w: &World, c: UnitId, step: Step, allow_dry: bool) -> String {
        let mut steps = self.steps_of(c);
        steps.push(step);
        self.set_steps(c, steps.clone());
        let st = self.compile(w, c);
        let problem = if st.error_index >= 0 { st.error_text.clone() } else if !allow_dry && st.runs_dry() { "Not enough AP.".to_string() } else { String::new() };
        if !problem.is_empty() {
            steps.pop();
            self.set_steps(c, steps);
        }
        problem
    }

    pub fn remove_step(&mut self, c: UnitId, index: usize) {
        let mut steps = self.steps_of(c);
        if index < steps.len() {
            steps.remove(index);
            self.set_steps(c, steps);
        }
    }

    /// `sequences[c] = steps` (an existing key keeps its place).
    pub fn set_steps(&mut self, c: UnitId, steps: Vec<Step>) {
        self.sequences.insert(c, steps);
    }

    pub fn truncate(&mut self, c: UnitId, mark: usize) {
        let mut steps = self.steps_of(c);
        if steps.len() > mark {
            steps.truncate(mark);
            self.set_steps(c, steps);
        }
    }

    pub fn clear(&mut self, c: UnitId) {
        self.sequences.shift_remove(&c);
    }

    // ---- Compilation --------------------------------------------------------------------------------------------

    fn start_state(&self, w: &World, c: UnitId) -> SimState {
        let u = &w.units[c];
        let mut st = SimState {
            pos: if w.has(c) { w.pos(c) } else { NO_CELL },
            stance: u.stance,
            ap_left: u.ap,
            max_ap: u.max_ap,
            total_blocks: w.total_blocks,
            next_block: 1,
            blocked_blocks: IndexMap::new(),
            rooted: u.has_status("root"),
            withdrawn: false,
            acted: false,
            moved: false,
            positioned: false,
            lock: u.active_target.filter(|t| w.units[*t].is_active()),
            est_hp: w.foes_of(c).iter().map(|f| (*f, w.units[*f].hp)).collect(),
            consumed: HashMap::new(),
            entries: Vec::new(),
            error_index: -1,
            error_text: String::new(),
            dry_entry: -1,
            dry_step: -1,
        };
        for s in &u.st.active {
            for &b in status::blocked_blocks(s) {
                if b <= st.total_blocks && !st.blocked_blocks.contains_key(&b) {
                    st.blocked_blocks.insert(b, s.clone());
                }
            }
        }
        let book = w.book();
        for b in 1..=st.total_blocks {
            if let Some(name) = st.blocked_blocks.get(&b).cloned() {
                let id = book.find(if name == "stun" { "stunned" } else { "staggered" }).unwrap();
                let mut lost = PlanEntry::new(id, b, -1);
                lost.status_name = name;
                lost.pos_before = st.pos;
                lost.pos_after = st.pos;
                lost.stance_before = st.stance;
                lost.stance_after = st.stance;
                st.entries.push(lost);
            }
        }
        st
    }

    pub fn compile(&self, w: &World, c: UnitId) -> SimState {
        let mut st = self.start_state(w, c);
        let steps = self.steps_of(c);
        for (i, step) in steps.iter().enumerate() {
            let backup = st.clone();
            let ap_before = st.ap_left;
            let problem = self.apply(w, c, &mut st, step, i as i32);
            if !problem.is_empty() {
                st = backup;
                st.error_index = i as i32;
                st.error_text = problem;
                break;
            }
            if st.ap_left < 0 && ap_before >= 0 && st.dry_step < 0 {
                st.dry_step = i as i32;
            }
        }
        st
    }

    pub fn entries_for(&self, w: &World, c: UnitId) -> Vec<PlanEntry> {
        self.compile(w, c).entries
    }

    pub fn is_valid(&self, w: &World, c: UnitId) -> bool {
        self.compile(w, c).error_index < 0
    }

    fn apply(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let u = &w.units[c];
        if st.withdrawn {
            return format!("{} has already left the battle.", u.name);
        }
        if !u.is_active() || st.pos == NO_CELL {
            return format!("{} is not on the field.", u.name);
        }
        let cap = w.skill_cap_of(c);
        if let Some(sid) = step.skill {
            let s = &w.cat.skills[sid];
            if s.level > cap {
                return format!("{} isn't unlocked yet.", s.name);
            }
            if !u.knows_skill(&w.cat, &s.id) {
                return format!("{} doesn't know {}.", u.name, s.name);
            }
            if !u.tier_unlocked(&w.cat, s) {
                return format!("{} unlocks at unit level {}.", s.name, u.skill_unlock_level(&w.cat, s));
            }
            if skill::kind(s) == Kind::Disengage {
                return self.apply_disengage(w, c, st, step, index);
            }
        }
        match step.mode {
            Mode::SelfCast => {
                let sid = step.skill.unwrap();
                if skill::kind(&w.cat.skills[sid]) == Kind::Support {
                    return self.apply_support_self(w, c, st, sid, index);
                }
                self.add_entry(w, c, st, sid, index, None, &[], 0, &[])
            }
            Mode::Ally => self.apply_ally(w, c, st, step, index),
            Mode::Move => self.apply_move(w, c, st, step, index),
            Mode::Goto => self.apply_goto(w, c, st, step, index),
            Mode::Withdraw => self.apply_withdraw(w, c, st, index),
            Mode::Engage => self.apply_engage(w, c, st, step, index),
            Mode::Area => self.apply_area(w, c, st, step.skill.unwrap(), step.cell.get(), index),
            Mode::Item => self.apply_item(w, c, st, step, index),
        }
    }

    /// ITEMUSE1 rules 2-3: a battle-pouch consumable on its target. 1 AP, `use_blocks` blocks at the lowest free blocks
    /// like a SELF step, no auto-move: the target must stand within the item's reach of the user's planned tile when the
    /// step starts. Earlier ITEM steps of this round use up copies (nothing leaves the pouch before the command resolves).
    fn apply_item(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let con = command::pouch_item(w, c, &step.item_id);
        let item_name = con.map(|i| i.name.clone()).or_else(|| w.cat.items.get(&step.item_id).map(|i| i.name.clone())).unwrap_or_else(|| step.item_id.clone());
        let Some(con) = con.filter(|_| self.items_left(w, c, &step.item_id, index) > 0) else {
            return format!("No {item_name} left in {}'s pouch.", w.units[c].name);
        };
        if !con.offered_in_battle() {
            return format!("{item_name} can't be used in battle.");
        }
        let target = step.target.unwrap_or(c);
        let side = command::item_side_problem(w, c, con, Some(target));
        if !side.is_empty() {
            return side;
        }
        if !w.units[target].is_active() || w.grid.positions[target].is_none() {
            return format!("{} is not on the field.", w.units[target].name);
        }
        if target != c && st.pos.dist(w.pos(target)) > con.battle_reach() {
            return format!("Out of reach for {item_name}.");
        }
        let Some(sid) = w.book().find(&skill::item_skill_id(&con.id)) else {
            return format!("{item_name} can't be used in battle.");
        };
        let problem = self.add_entry(w, c, st, sid, index, Some(target), &[], 0, &[]);
        if !problem.is_empty() {
            return problem;
        }
        let last = st.entries.len() - 1;
        let e = &mut st.entries[last];
        e.item = Some(con.clone());
        e.skill_ap = command::ITEM_AP;
        // add_entry charged the skill's AP; an item costs ITEM_AP
        st.ap_left += skill::ap_cost(&w.cat.skills[sid], Some(&w.units[c])) - command::ITEM_AP;
        if st.dry_entry == last as i32 && st.ap_left >= 0 {
            st.dry_entry = -1;
        }
        let tu = &w.units[target];
        if con.heal > 0 {
            st.entries[last].est_heal = con.heal.min((tu.max_hp - tu.hp).max(0));
        }
        if con.damage > 0 {
            st.entries[last].est_damage = con.damage;
            let hp = st.est_hp.get(&target).copied().unwrap_or(tu.hp);
            st.est_hp.insert(target, (hp - con.damage).max(0));
        }
        String::new()
    }

    /// Copies of `item_id` in `c`'s pouch not claimed by its ITEM steps before step `before` (-1 = all planned steps).
    pub fn items_left(&self, w: &World, c: UnitId, item_id: &str, before: i32) -> i32 {
        let steps = self.steps_of(c);
        let upto = if before < 0 { steps.len() } else { (before as usize).min(steps.len()) };
        command::pouch_count(w, c, item_id) - steps[..upto].iter().filter(|s| s.mode == Mode::Item && s.item_id == item_id).count() as i32
    }

    fn move_problem(&self, w: &World, c: UnitId, st: &SimState) -> String {
        if st.rooted {
            return format!("{} is rooted this round and can't move.", w.units[c].name);
        }
        if st.acted {
            return format!("Moving must come first: {} has already acted this round.", w.units[c].name);
        }
        String::new()
    }

    fn too_far(&self) -> String {
        "Too far".into()
    }

    fn apply_move(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let p = self.move_problem(w, c, st);
        if !p.is_empty() {
            return p;
        }
        if step.direction.x.abs() + step.direction.y.abs() != 1 {
            return "Pick a direction to move.".into();
        }
        if step.tiles < 1 || step.tiles > self.max_move_tiles(w, c) {
            return "Too many tiles.".into();
        }
        let blocked = self.blocked_cells(w, c);
        let mut cell = st.pos;
        let mut route = Vec::new();
        for _ in 0..step.tiles {
            cell = cell + step.direction;
            if !w.grid.is_valid(cell) {
                return "The arena edge blocks that move.".into();
            }
            if w.grid.is_exit(cell) {
                return "Use Withdraw to leave through the exit.".into();
            }
            if !w.grid.is_walkable(cell) {
                return "Terrain blocks that move.".into();
            }
            if blocked.contains(&cell) {
                return "Another unit blocks that move.".into();
            }
            route.push(step.direction);
        }
        if !self.fits_move(w, c, st.pos, &route) {
            return self.too_far();
        }
        self.add_move(w, c, st, &route, None, index)
    }

    fn apply_goto(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let p = self.move_problem(w, c, st);
        if !p.is_empty() {
            return p;
        }
        let blocked = self.blocked_cells(w, c);
        let dest = step.cell.get();
        if !w.grid.is_valid(dest) || !w.grid.is_walkable(dest) || w.grid.is_exit(dest) {
            return "That tile can't be moved to.".into();
        }
        if blocked.contains(&dest) {
            return "Another unit is (or will be) standing there.".into();
        }
        if st.pos == dest {
            return "Already there.".into();
        }
        let route = self.route(w, c, st.pos, &[dest].into_iter().collect(), &blocked, false);
        if route.is_empty() {
            return "No path to that tile.".into();
        }
        if !self.fits_move(w, c, st.pos, &route) {
            return self.too_far();
        }
        self.add_move(w, c, st, &route, None, index)
    }

    fn apply_withdraw(&self, w: &World, c: UnitId, st: &mut SimState, index: i32) -> String {
        let p = self.move_problem(w, c, st);
        if !p.is_empty() {
            return p;
        }
        let exits: Cells = w.grid.exits.iter().copied().collect();
        if exits.is_empty() {
            return "This arena has no exit.".into();
        }
        let mut route = self.route(w, c, st.pos, &exits, &self.blocked_cells(w, c), true);
        if route.is_empty() {
            return "No route to an exit.".into();
        }
        let reaches = self.fits_move(w, c, st.pos, &route);
        if !reaches {
            route = path::fit_blocks(&w.grid, st.pos, &route, w.units[c].move_per_block(), MAX_MOVE_BLOCKS);
        }
        let p = self.add_move(w, c, st, &route, None, index);
        if p.is_empty() && reaches {
            st.withdrawn = true;
        }
        p
    }

    fn apply_engage(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let Some(foe) = step.target.filter(|f| w.units[*f].is_active() && w.has(*f)) else { return "Choose an enemy to attack.".into() };
        let sid = step.skill.unwrap();
        let s = &w.cat.skills[sid];
        if skill::is_area(s) {
            return self.apply_area(w, c, st, sid, w.pos(foe), index);
        }
        if s.dash_tiles > 0 {
            return self.apply_dash(w, c, st, sid, foe, index);
        }
        let band = skill::reach_for(s, &w.units[c]);
        let foe_pos = w.pos(foe);
        let d = st.pos.dist(foe_pos);
        if d < band.0 || d > band.1 {
            let p = self.move_problem(w, c, st);
            if !p.is_empty() {
                return if d < band.0 { "Too close.".into() } else { format!("Out of reach: {p}") };
            }
            let route;
            let stored = step.cell.get();
            if stored != NO_CELL {
                let sd = stored.dist(foe_pos);
                if sd < band.0 || sd > band.1 {
                    return "The planned approach no longer reaches it.".into();
                }
                let blocked = self.blocked_cells(w, c);
                if blocked.contains(&stored) {
                    return "The planned approach tile is taken.".into();
                }
                route = self.route(w, c, st.pos, &[stored].into_iter().collect(), &blocked, false);
            } else {
                route = self.approach_path(w, c, st.pos, foe, band.0, band.1);
                if !route.is_empty() {
                    step.cell.set(route.iter().fold(st.pos, |a, d| a + *d));
                }
            }
            if route.is_empty() {
                return "No path to get within reach.".into();
            }
            if !self.fits_move(w, c, st.pos, &route) {
                return self.too_far();
            }
            let p = self.add_move(w, c, st, &route, Some(foe), index);
            if !p.is_empty() {
                return p;
            }
        }
        if st.lock.is_some_and(|l| l != foe) {
            let sw = w.book().find("switch_target").unwrap();
            let p = self.add_entry(w, c, st, sw, index, Some(foe), &[], 0, &[]);
            if !p.is_empty() {
                return p;
            }
        }
        st.lock = Some(foe);
        let p = self.add_entry(w, c, st, sid, index, Some(foe), &[], 0, &[]);
        if p.is_empty() {
            let positioned = st.positioned;
            let e = st.entries.last_mut().unwrap();
            e.chase = !positioned;
            let dmg = e.est_damage;
            *st.est_hp.entry(foe).or_insert(0) -= dmg;
        }
        p
    }

    fn apply_dash(&self, w: &World, c: UnitId, st: &mut SimState, sid: SkillId, foe: UnitId, index: i32) -> String {
        let s = &w.cat.skills[sid];
        let band = skill::reach_for(s, &w.units[c]);
        let d = st.pos.dist(w.pos(foe));
        if d >= band.0 && d <= band.1 {
            return "Already in reach.".into();
        }
        if d < band.0 {
            return "Too close.".into();
        }
        let p = self.move_problem(w, c, st);
        if !p.is_empty() {
            return format!("Can't dash: {p}");
        }
        let route = self.approach_path(w, c, st.pos, foe, band.0, band.1);
        if route.is_empty() {
            return "No route to get within reach.".into();
        }
        if !dash_fits(&w.grid, st.pos, &route, s.dash_tiles) {
            return "Too far for the dash.".into();
        }
        if st.lock.is_some_and(|l| l != foe) {
            let sw = w.book().find("switch_target").unwrap();
            let p = self.add_entry(w, c, st, sw, index, Some(foe), &[], 0, &[]);
            if !p.is_empty() {
                return p;
            }
        }
        let from = st.pos;
        let dest = route.iter().fold(from, |a, d| a + *d);
        st.lock = Some(foe);
        let p = self.add_entry(w, c, st, sid, index, Some(foe), &route, 0, &[]);
        if p.is_empty() {
            let e = st.entries.last_mut().unwrap();
            e.pos_before = from;
            e.pos_after = dest;
            e.goal = dest;
            let dmg = e.est_damage;
            st.pos = dest;
            st.moved = true;
            st.positioned = false;
            *st.est_hp.entry(foe).or_insert(0) -= dmg;
        }
        p
    }

    fn apply_disengage(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let sid = step.skill.unwrap();
        let s = &w.cat.skills[sid];
        let p = self.move_problem(w, c, st);
        if !p.is_empty() {
            return format!("Can't disengage: {p}");
        }
        let mut threat = step.target;
        if threat.is_none() {
            let mut best = 1 << 20;
            for &f in w.foes_of(c) {
                if w.units[f].is_active() && w.has(f) {
                    let d = st.pos.dist(w.pos(f));
                    if d < best {
                        best = d;
                        threat = Some(f);
                    }
                }
            }
        }
        let Some(threat) = threat.filter(|t| w.units[*t].is_active() && w.has(*t) && w.foes_of(c).contains(t)) else { return "No enemy to jump away from.".into() };
        let threat_pos = w.pos(threat);
        let here = st.pos.dist(threat_pos);
        let blocked = self.blocked_cells(w, c);
        let reachable = self.jump_tiles_from(w, st.pos, s.jump_tiles);
        let mut dest = NO_CELL;
        let mut dest_steps = 0;
        let chosen = step.cell.get();
        if chosen != NO_CELL {
            if !reachable.contains_key(&chosen) || blocked.contains(&chosen) {
                return "Can't jump there.".into();
            }
            if chosen.dist(threat_pos) <= here {
                return "Not away from the enemy.".into();
            }
            dest = chosen;
            dest_steps = reachable[&chosen];
        } else {
            for (&cell, &steps) in &reachable {
                if blocked.contains(&cell) {
                    continue;
                }
                let d = cell.dist(threat_pos);
                if d <= here {
                    continue;
                }
                if dest == NO_CELL || better_jump(cell, d, steps, dest, dest.dist(threat_pos), dest_steps) {
                    dest = cell;
                    dest_steps = steps;
                }
            }
            if dest == NO_CELL {
                return "No free tile to jump to.".into();
            }
            step.cell.set(dest);
        }
        let from = st.pos;
        let p = self.add_entry(w, c, st, sid, index, Some(threat), &[], 0, &[]);
        if p.is_empty() {
            let e = st.entries.last_mut().unwrap();
            e.tiles = dest_steps;
            e.pos_before = from;
            e.pos_after = dest;
            e.goal = dest;
            st.pos = dest;
            st.moved = true;
            st.positioned = true;
        }
        p
    }

    /// Tiles reachable in at most `max_steps` steps over walkable, non-exit tiles (units are leapt over).
    pub fn jump_tiles_from(&self, w: &World, from: V2, max_steps: i32) -> IndexMap<V2, i32> {
        let mut depth: IndexMap<V2, i32> = IndexMap::new();
        depth.insert(from, 0);
        let mut queue = vec![from];
        let mut head = 0;
        while head < queue.len() {
            let cur = queue[head];
            head += 1;
            let cd = depth[&cur];
            if cd >= max_steps {
                continue;
            }
            for d in DIRS {
                let n = cur + d;
                if depth.contains_key(&n) || !w.grid.is_valid(n) || !w.grid.is_walkable(n) || w.grid.is_exit(n) {
                    continue;
                }
                depth.insert(n, cd + 1);
                queue.push(n);
            }
        }
        depth.shift_remove(&from);
        depth
    }

    fn apply_support_self(&self, w: &World, c: UnitId, st: &mut SimState, sid: SkillId, index: i32) -> String {
        let s = &w.cat.skills[sid];
        if s.target_mode == "ALLY" {
            return "Must be aimed at an ally.".into();
        }
        let p = self.add_entry(w, c, st, sid, index, Some(c), &[], 0, &[]);
        if p.is_empty() {
            let heal = command::effective_heal(&w.units[c], command::heal_amount(w, c, sid));
            st.entries.last_mut().unwrap().est_heal = heal;
        }
        p
    }

    fn apply_ally(&self, w: &World, c: UnitId, st: &mut SimState, step: &Step, index: i32) -> String {
        let sid = step.skill.unwrap();
        let s = &w.cat.skills[sid];
        if skill::kind(s) != Kind::Support {
            return "That isn't a support skill.".into();
        }
        let ally = step.target;
        if ally == Some(c) {
            return self.apply_support_self(w, c, st, sid, index);
        }
        let p = command::target_problem(w, c, sid, ally);
        if !p.is_empty() {
            return p;
        }
        let ally = ally.unwrap();
        if !w.has(ally) {
            return "Not on the field.".into();
        }
        let band = skill::support_reach(s, &w.units[c]);
        let near = band.0.max(1);
        let ally_pos = w.pos(ally);
        let d = st.pos.dist(ally_pos);
        if d < near || d > band.1 {
            let p = self.move_problem(w, c, st);
            if !p.is_empty() {
                return format!("Out of reach: {p}");
            }
            let route;
            let stored = step.cell.get();
            if stored != NO_CELL {
                let sd = stored.dist(ally_pos);
                if sd < near || sd > band.1 {
                    return "The planned approach no longer reaches it.".into();
                }
                let blocked = self.blocked_cells(w, c);
                if blocked.contains(&stored) {
                    return "The planned approach tile is taken.".into();
                }
                route = self.route(w, c, st.pos, &[stored].into_iter().collect(), &blocked, false);
            } else {
                route = self.approach_path(w, c, st.pos, ally, near, band.1);
                if !route.is_empty() {
                    step.cell.set(route.iter().fold(st.pos, |a, d| a + *d));
                }
            }
            if route.is_empty() {
                return "No path to get within reach.".into();
            }
            if !self.fits_move(w, c, st.pos, &route) {
                return self.too_far();
            }
            let p = self.add_move(w, c, st, &route, None, index);
            if !p.is_empty() {
                return p;
            }
        }
        let p = self.add_entry(w, c, st, sid, index, Some(ally), &[], 0, &[]);
        if p.is_empty() {
            let heal = command::effective_heal(&w.units[ally], command::heal_amount(w, c, sid));
            st.entries.last_mut().unwrap().est_heal = heal;
        }
        p
    }

    fn apply_area(&self, w: &World, c: UnitId, st: &mut SimState, sid: SkillId, aim: V2, index: i32) -> String {
        let s = &w.cat.skills[sid];
        if !skill::is_area(s) {
            return "That isn't an area attack.".into();
        }
        if !w.grid.is_valid(aim) {
            return "Aim inside the arena.".into();
        }
        let band = skill::reach_for(s, &w.units[c]);
        let d = st.pos.dist(aim);
        let a = skill::area(s);
        if a == skill::Area::Line && d != 1 {
            return "Needs an adjacent tile.".into();
        }
        if a == skill::Area::Sweep && ((aim.x - st.pos.x).abs() != 1 || (aim.y - st.pos.y).abs() != 1) {
            return "Needs a diagonal tile.".into();
        }
        if a != skill::Area::Line && a != skill::Area::Sweep && (d < band.0 || d > band.1) {
            return "Out of reach.".into();
        }
        let cells: Vec<V2> = skill::area_cells(s, st.pos, aim).into_iter().filter(|cell| w.grid.is_valid(*cell) && *cell != st.pos).collect();
        let p = self.add_entry(w, c, st, sid, index, None, &[], 0, &cells);
        if p.is_empty() {
            let stance_before = st.entries.last().unwrap().stance_before;
            let attack_stance = skill::stance_during(s, stance_before);
            let pos = st.pos;
            let mut total = 0;
            for &foe in w.foes_of(c) {
                if w.units[foe].is_active() && w.has(foe) && cells.contains(&w.pos(foe)) {
                    let hit = self.estimate_hit(w, c, st, sid, attack_stance, foe, pos);
                    total += hit;
                    *st.est_hp.entry(foe).or_insert(0) -= hit;
                }
            }
            let mut friends_hit = 0;
            for &f in w.friends_of(c) {
                if f != c && w.units[f].is_active() && cells.contains(&self.planned_position(w, f)) {
                    friends_hit += 1;
                }
            }
            let e = st.entries.last_mut().unwrap();
            e.aim = aim;
            e.est_damage += total;
            e.hits_allies += friends_hit;
        }
        p
    }

    /// Steps from `from` to the nearest tile at distance near..far of `foe` (defaults: the basic attack band).
    pub fn approach_path(&self, w: &World, c: UnitId, from: V2, foe: UnitId, near: i32, far: i32) -> Vec<V2> {
        let foe_pos = w.pos(foe);
        let basic = skill::reach_for(&w.cat.skills[w.book().lowest(Kind::Attack)], &w.units[c]);
        let lo = if near > 0 { near } else { basic.0 };
        let hi = if far > 0 { far } else { basic.1 };
        let blocked = self.blocked_cells(w, c);
        let mut goals = Cells::new();
        for dx in -hi..=hi {
            for dy in -hi..=hi {
                let d = dx.abs() + dy.abs();
                let cell = foe_pos + V2::new(dx, dy);
                if d >= lo && d <= hi && w.grid.is_valid(cell) && w.grid.is_walkable(cell) && !w.grid.is_exit(cell) && !blocked.contains(&cell) {
                    goals.insert(cell);
                }
            }
        }
        self.route(w, c, from, &goals, &blocked, false)
    }

    /// GridPath.find_path with the unit's AI hazard penalty and cover preference, if any.
    pub fn route(&self, w: &World, c: UnitId, from: V2, goals: &Cells, blocked: &Cells, goals_are_exits: bool) -> Vec<V2> {
        let penalty = |cell: V2| if w.grid.has_hazard(cell) { AI_HAZARD_PENALTY } else { 0 };
        let shooters = self.goal_preferences.get(&c);
        let prefer = |cell: V2| cover_pref(w, shooters.unwrap(), cell);
        let hooks = Hooks {
            penalty: if self.path_penalties.contains(&c) { Some(&penalty) } else { None },
            prefer: if shooters.is_some() { Some(&prefer) } else { None },
        };
        path::find_path(&w.grid, from, goals, blocked, goals_are_exits, &hooks)
    }

    /// One Move entry per cost-chunked block (1 AP per tile).
    fn add_move(&self, w: &World, c: UnitId, st: &mut SimState, dirs: &[V2], chase: Option<UnitId>, index: i32) -> String {
        let mv = w.book().lowest(Kind::Move);
        let goal = dirs.iter().fold(st.pos, |a, d| a + *d);
        for part in path::chunk(&w.grid, st.pos, dirs, w.units[c].move_per_block()) {
            let p = self.add_entry(w, c, st, mv, index, chase, &part, 1, &[]);
            if !p.is_empty() {
                return p;
            }
            for d in &part {
                st.pos = st.pos + *d;
            }
            let pos = st.pos;
            let e = st.entries.last_mut().unwrap();
            e.pos_after = pos;
            e.goal = goal;
        }
        st.moved = true;
        st.positioned = chase.is_none();
        String::new()
    }

    fn free_start(&self, st: &SimState, n: i32) -> i32 {
        for s in st.next_block..(st.total_blocks - n + 2) {
            if (s..s + n).all(|b| !st.blocked_blocks.contains_key(&b)) {
                return s;
            }
        }
        -1
    }

    /// Places one skill at the lowest free block and pays its AP (skill + stance switch; debt allowed).
    #[allow(clippy::too_many_arguments)]
    fn add_entry(&self, w: &World, c: UnitId, st: &mut SimState, sid: SkillId, index: i32, foe: Option<UnitId>, dirs: &[V2], blocks_override: i32, cells: &[V2]) -> String {
        let s = &w.cat.skills[sid];
        let n = if blocks_override > 0 { blocks_override } else { s.blocks };
        let start = self.free_start(st, n);
        if start < 0 {
            return format!("Not enough free blocks: {} needs {}, {} left.", s.name, n, st.blocks_left());
        }
        let u = &w.units[c];
        let mut e = PlanEntry::new(sid, start, index);
        e.blocks_override = blocks_override;
        e.target = foe;
        e.path = dirs.to_vec();
        e.tiles = dirs.len() as i32;
        e.cells = cells.to_vec();
        e.direction = dirs.first().copied().unwrap_or(ZERO);
        e.skill_ap = skill::ap_cost(s, Some(u)) * if skill::kind(s) == Kind::Move { (dirs.len() as i32).max(1) } else { 1 };
        e.switch_ap = skill::switch_cost(s, st.stance);
        e.stance_before = st.stance;
        e.stance_after = skill::stance_after(s, st.stance);
        e.pos_before = st.pos;
        e.pos_after = st.pos;
        if skill::kind(s) == Kind::Attack && !skill::is_area(s) {
            if let Some(f) = foe {
                let from = dirs.iter().fold(st.pos, |a, d| a + *d);
                e.est_damage = self.estimate_hit(w, c, st, sid, skill::stance_during(s, e.stance_before), f, from);
            }
        }
        let total = e.total_ap();
        let after = e.stance_after;
        st.entries.push(e);
        st.next_block = start + n;
        st.acted = true;
        let was_solvent = st.ap_left >= 0;
        st.ap_left -= total;
        if st.ap_left < 0 && was_solvent && st.dry_entry < 0 {
            st.dry_entry = st.entries.len() as i32 - 1;
        }
        st.stance = after;
        if skill::kind(s) == Kind::Rest {
            st.ap_left = st.max_ap.min(st.ap_left + s.recover_ap);
        }
        String::new()
    }

    /// Expected hit: attack power x Focus x skill x exploit x stance x armor x cover; no crit, no guard.
    pub fn estimate_damage(&self, w: &World, c: UnitId, sid: SkillId, stance: Stance, foe: UnitId, from: V2, consumed: &[String]) -> i32 {
        let s = &w.cat.skills[sid];
        if !skill::deals_damage(s) {
            return 0;
        }
        let exploit_mult = skill::exploit_match(&s.exploit, &w.units[foe], consumed).1;
        expected(w, c, foe, sid, Some(stance), self.estimate_cover(w, c, sid, foe, from), exploit_mult)
    }

    fn estimate_hit(&self, w: &World, c: UnitId, st: &mut SimState, sid: SkillId, stance: Stance, foe: UnitId, from: V2) -> i32 {
        let mut used = st.consumed.get(&foe).cloned().unwrap_or_default();
        let hit = self.estimate_damage(w, c, sid, stance, foe, from, &used);
        let s = &w.cat.skills[sid];
        if s.exploit_consume && skill::deals_damage(s) {
            let matched = skill::exploit_match(&s.exploit, &w.units[foe], &used).0;
            if !matched.is_empty() {
                used.push(matched);
                st.consumed.insert(foe, used);
            }
        }
        hit
    }

    pub fn estimate_cover(&self, w: &World, c: UnitId, sid: SkillId, foe: UnitId, from: V2) -> f64 {
        if !w.has(foe) {
            return 0.0;
        }
        let at = if from != NO_CELL { from } else { w.pos(c) };
        cover_for(&w.grid, at, w.pos(foe), expected_is_ranged(w, c, sid))
    }

    // ---- Occupancy ------------------------------------------------------------------------------------------------

    pub fn planned_position(&self, w: &World, c: UnitId) -> V2 {
        let moves = self.ally_moves(w);
        self.position_after(w, c, &moves)
    }

    pub fn planned_positions(&self, w: &World, units: &[UnitId]) -> HashMap<UnitId, V2> {
        let moves = self.ally_moves(w);
        units.iter().map(|u| (*u, self.position_after(w, *u, &moves))).collect()
    }

    fn position_after(&self, w: &World, c: UnitId, moves: &HashMap<UnitId, FirstMove>) -> V2 {
        match moves.get(&c) {
            Some(m) => m.to,
            None => {
                if w.has(c) {
                    w.pos(c)
                } else {
                    NO_CELL
                }
            }
        }
    }

    pub fn blocked_cells(&self, w: &World, c: UnitId) -> Rc<Cells> {
        if let Some(o) = self.blocked_override.borrow().as_ref() {
            return o.clone();
        }
        Rc::new(self.blocked_for(w, c, &self.ally_moves(w)))
    }

    fn blocked_for(&self, w: &World, c: UnitId, moves: &HashMap<UnitId, FirstMove>) -> Cells {
        let mut blocked = Cells::new();
        let friends = w.friends_of(c);
        for &other in w.allies.iter().chain(w.enemies.iter()) {
            if other == c || !w.has(other) {
                continue;
            }
            let start = w.pos(other);
            match moves.get(&other).filter(|_| friends.contains(&other)) {
                Some(m) => {
                    if m.to != NO_CELL {
                        blocked.insert(m.to);
                    }
                    let leaves_first = m.leave == 1 && w.units[other].speed > w.units[c].speed;
                    if !leaves_first {
                        blocked.insert(start);
                    }
                }
                None => {
                    blocked.insert(start);
                }
            }
        }
        blocked
    }

    /// Each planning unit's opening move, fastest first (ties in planning order); each sees earlier claims.
    fn ally_moves(&self, w: &World) -> HashMap<UnitId, FirstMove> {
        let mut moves: HashMap<UnitId, FirstMove> = HashMap::new();
        let mut units: Vec<UnitId> = Vec::new();
        let mut order: HashMap<UnitId, usize> = HashMap::new();
        let mut pinned: IndexMap<UnitId, V2> = IndexMap::new();
        for &m in self.sequences.keys() {
            order.insert(m, units.len());
            units.push(m);
            let cell = self.pinned_cell(w, m);
            if cell != NO_CELL {
                pinned.insert(m, cell);
            }
        }
        godot_sort(&mut units, |a, b| {
            let (sa, sb) = (w.units[*a].speed, w.units[*b].speed);
            sa > sb || (sa == sb && order[a] < order[b])
        });
        for &unit in &units {
            let mut blocked = self.blocked_for(w, unit, &moves);
            if !pinned.contains_key(&unit) {
                let friends = w.friends_of(unit);
                for (other, cell) in &pinned {
                    if *other != unit && friends.contains(other) {
                        blocked.insert(*cell);
                    }
                }
            }
            if let Some(m) = self.first_move(w, unit, blocked) {
                moves.insert(unit, m);
            }
        }
        moves
    }

    pub fn pinned_cell(&self, w: &World, c: UnitId) -> V2 {
        let steps = self.steps_of(c);
        let Some(first) = steps.first() else { return NO_CELL };
        if !w.has(c) {
            return NO_CELL;
        }
        if first.skill.is_some_and(|s| skill::kind(&w.cat.skills[s]) == Kind::Disengage) {
            return first.cell.get();
        }
        match first.mode {
            Mode::Move => w.pos(c) + first.direction * first.tiles,
            Mode::Goto | Mode::Engage | Mode::Ally => first.cell.get(),
            _ => NO_CELL,
        }
    }

    fn first_move(&self, w: &World, c: UnitId, blocked: Cells) -> Option<FirstMove> {
        let steps = self.steps_of(c);
        let first = steps.first()?;
        if !w.units[c].is_active() {
            return None;
        }
        let jumps = first.skill.is_some_and(|s| skill::kind(&w.cat.skills[s]) == Kind::Disengage);
        if !jumps && !matches!(first.mode, Mode::Move | Mode::Goto | Mode::Withdraw | Mode::Engage | Mode::Ally) {
            return None;
        }
        let previous = self.blocked_override.replace(Some(Rc::new(blocked)));
        let mut st = self.start_state(w, c);
        let problem = self.apply(w, c, &mut st, first, 0);
        *self.blocked_override.borrow_mut() = previous;
        if !problem.is_empty() {
            return None;
        }
        let mut leave = -1;
        let mut last: Option<&PlanEntry> = None;
        for e in &st.entries {
            let k = skill::kind(&w.cat.skills[e.skill]);
            if k == Kind::Move || (k == Kind::Disengage && e.pos_after != e.pos_before) {
                if leave < 0 {
                    leave = e.end_block(w);
                }
                last = Some(e);
            } else if w.cat.skills[e.skill].dash_tiles > 0 && e.pos_after != e.pos_before {
                if leave < 0 {
                    leave = e.start_block;
                }
                last = Some(e);
            }
        }
        let last = last?;
        Some(FirstMove { to: if st.withdrawn { NO_CELL } else { last.pos_after }, leave })
    }

    // ---- Hand-off to the resolver -------------------------------------------------------------------------------

    /// Schedules every unit's compiled commands, in plan order; returns how many were accepted.
    pub fn commit(&self, w: &World, resolver: &mut Resolver) -> usize {
        let mut count = 0;
        for &c in self.sequences.keys() {
            for e in self.entries_for(w, c) {
                if resolver.schedule(c, e.build_command(w, c), e.start_block) {
                    count += 1;
                }
            }
        }
        count
    }
}

/// _better_jump: farther from the threat, then a shorter jump, then nearer the centre, then lower x, then lower y.
fn better_jump(a: V2, a_dist: i32, a_steps: i32, b: V2, b_dist: i32, b_steps: i32) -> bool {
    if a_dist != b_dist {
        return a_dist > b_dist;
    }
    if a_steps != b_steps {
        return a_steps < b_steps;
    }
    let (ac, bc) = (crate::grid::Grid::centre_distance(a), crate::grid::Grid::centre_distance(b));
    if ac != bc {
        return ac < bc;
    }
    if a.x != b.x {
        return a.x < b.x;
    }
    a.y < b.y
}

/// DamageCalculator.expected_is_ranged: a ranged unit's main-hand attack.
pub fn expected_is_ranged(w: &World, c: UnitId, sid: SkillId) -> bool {
    w.units[c].attack_range > 1 && !skill::is_off_hand(&w.cat.skills[sid])
}

/// DamageCalculator.expected: no crit, no guard, cover included; `power_scale` multiplies the skill power.
pub fn expected(w: &World, c: UnitId, foe: UnitId, sid: SkillId, stance: Option<Stance>, cover: f64, power_scale: f64) -> i32 {
    let s = &w.cat.skills[sid];
    let u = &w.units[c];
    damage::compute(
        u,
        &w.units[foe],
        HitInput {
            ranged: expected_is_ranged(w, c, sid),
            offhand: skill::is_off_hand(s),
            skill_mult: u.power_for(s) * power_scale,
            attacker_stance: stance,
            include_guard: false,
            cover,
            ..HitInput::default()
        },
    )
    .final_dmg
}
