//! The battle world: units, grid, sides and AI profiles (CombatGrid unit operations, PlanContext, AiProfile,
//! ContentFactory.ai_profile). Units are referred to by their index in `units`.

use crate::grid::{Grid, V2, DEPLOY_ANCHOR_Y, DEPLOY_ROWS, GRID_H, GRID_W, NO_CELL};
use crate::skill::{self, Book};
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::Unit;
use indexmap::IndexMap;
use std::sync::Arc;

pub type UnitId = usize;

pub const NO_CEILING: i32 = 5;
pub const DEFAULT_HEAL_BELOW: f64 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Aggressor,
    Skirmisher,
    Sniper,
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPref {
    Nearest,
    Weakest,
    /// HOOKS1 AT1 (rule `ai_tactics`): lowest armour rating, then lowest HP, then nearest; unreachable this round last.
    Softest,
    /// HOOKS1 AT1 (rule `ai_tactics`): highest max HP among melee foes, then the rest; ties nearest.
    Toughest,
}

impl TargetPref {
    /// A `target_pref` value (`TARGET_PREFS`; ROLE and unknown values: None).
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "NEAREST" => Some(TargetPref::Nearest),
            "WEAKEST" => Some(TargetPref::Weakest),
            "SOFTEST" => Some(TargetPref::Softest),
            "TOUGHEST" => Some(TargetPref::Toughest),
            _ => None,
        }
    }
}

/// HOOKS1 AT2 (rule `ai_tactics`): a unit's AI tactics (`rpgkit_data::TACTICS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitTactic {
    /// A row before AI table row 4: Guard when a friend within 2 tiles, or the unit itself, lost HP to an attack hit
    /// last round (not reckless, a Guard skill usable).
    BraceAfterHit,
    /// PL4 for this unit ranks, after threat, tiles next to a friend first, then fewer walkable free neighbours first.
    HoldLine,
    /// An area opener's aim scores +1 per foe hit on a tile with cover.
    FlushCover,
}

impl UnitTactic {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "brace_after_hit" => Some(UnitTactic::BraceAfterHit),
            "hold_line" => Some(UnitTactic::HoldLine),
            "flush_cover" => Some(UnitTactic::FlushCover),
            _ => None,
        }
    }

    /// The tactics of a `tactics` list in order (unknown ids and repeats dropped; the content check reports them).
    pub fn list(ids: &[String]) -> Vec<Self> {
        let mut out: Vec<Self> = Vec::new();
        for t in ids.iter().filter_map(|id| Self::parse(id)) {
            if !out.contains(&t) {
                out.push(t);
            }
        }
        out
    }
}

/// HOOKS1 AT2 `brace_after_hit` (rule `ai_tactics`) and RULESET1-AI2 (rule `round_intent`): the units that lost HP to an
/// attack hit this round and last round, and the units in the Guard intent as last round ended (rotated as the round
/// ends; empty in classic; no save key).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HitMemory {
    pub this_round: Vec<UnitId>,
    pub last_round: Vec<UnitId>,
    pub guarded_last_round: Vec<UnitId>,
}

impl HitMemory {
    /// RULESET1-AI2: `c` was in the Guard intent last round and no attack hit took HP from it during that round.
    pub fn guarded_unhit(&self, c: UnitId) -> bool {
        self.guarded_last_round.contains(&c) && !self.last_round.contains(&c)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiProfile {
    pub role: Role,
    pub skill_cap: i32,
    pub flee_ratio: f64,
    pub defend_below: f64,
    pub target_pref: TargetPref,
    pub prefer_level: i32,
    pub reckless: bool,
    pub heal_below: f64,
    pub area_min_foes: i32,
    pub use_items: bool,
    /// HOOKS1 AT2: the unit's tactics (a phase `ai` effect's `tactics` replaces the list); read only under the rule
    /// `ai_tactics` (`World::has_tactic`).
    pub tactics: Vec<UnitTactic>,
}

impl AiProfile {
    pub fn create(role: Role, flee_ratio: f64, skill_cap: i32) -> Self {
        Self {
            role,
            skill_cap,
            flee_ratio,
            defend_below: 0.4,
            target_pref: if role == Role::Aggressor { TargetPref::Nearest } else { TargetPref::Weakest },
            prefer_level: 0,
            reckless: false,
            heal_below: DEFAULT_HEAL_BELOW,
            area_min_foes: 2,
            use_items: false,
            tactics: Vec::new(),
        }
    }

    /// The role's own preference (`target_pref` ROLE): nearest for an Aggressor, else weakest.
    pub fn role_pref(&self) -> TargetPref {
        if self.role == Role::Aggressor {
            TargetPref::Nearest
        } else {
            TargetPref::Weakest
        }
    }

    pub fn area_threshold(&self) -> i32 {
        self.area_min_foes.clamp(1, 6)
    }

    /// ContentFactory.ai_profile: the unit's AI role (or its class's), thresholds and AP ceiling.
    pub fn for_unit(cat: &Catalog, e: &UnitDef) -> Self {
        let arch = (!e.class_id.is_empty()).then(|| cat.classes.get(&e.class_id)).flatten();
        let role_name = if !e.ai_role.is_empty() { e.ai_role.clone() } else { arch.map(|a| a.ai_role.clone()).unwrap_or_else(|| "AGGRESSOR".into()) };
        let role = match role_name.as_str() {
            "SKIRMISHER" => Role::Skirmisher,
            "SNIPER" => Role::Sniper,
            "SUPPORT" => Role::Support,
            _ => Role::Aggressor,
        };
        let mut cap = e.skill_cap;
        if cap <= 0 && arch.is_some() {
            let tier_cap = rpgkit_rules::stats::tier_scales(&e.tier).2;
            cap = if tier_cap > 0 { tier_cap } else { NO_CEILING };
        } else if cap <= 0 {
            cap = 1;
        }
        let mut p = AiProfile::create(role, e.flee_ratio, cap);
        p.defend_below = e.defend_below;
        p.prefer_level = e.prefer_level;
        p.reckless = e.reckless;
        p.heal_below = e.heal_below;
        if e.target_pref != "ROLE" {
            // HOOKS1 AT1: SOFTEST and TOUGHEST are kept here and read as ROLE without the rule (`World::target_pref`)
            p.target_pref = TargetPref::parse(&e.target_pref).unwrap_or(TargetPref::Nearest);
        }
        p.tactics = UnitTactic::list(&e.tactics);
        p
    }

    /// PlanPresets.profile_for.
    pub fn with_preset(&self, preset: &str) -> Self {
        let mut p = self.clone();
        match preset {
            "aggressive" => {
                p.role = Role::Aggressor;
                p.target_pref = TargetPref::Nearest;
                p.defend_below = 0.0;
                p.flee_ratio = 0.0;
            }
            "focus_weakest" => {
                p.role = Role::Aggressor;
                p.target_pref = TargetPref::Weakest;
                p.defend_below = 0.25;
            }
            "defensive" => p.defend_below = 1.0,
            "ranged_kite" => {
                p.role = Role::Sniper;
                p.target_pref = TargetPref::Weakest;
            }
            "economy" => p.prefer_level = 1,
            "withdraw" => p.flee_ratio = 1.0,
            _ => {}
        }
        p
    }
}

/// RULESET1_INTENT AI5 (RULESET1-AI): an AUTO1 tactic as the round intent AI reads it (the preset id of
/// `AiProfile::with_preset`; any other id, "withdraw" included, drafts by the AI table with that preset's profile).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tactic {
    /// "balanced": the AI table.
    Balanced,
    /// "aggressive" (All-out Attack): Assault, the first ranked target.
    AllOut,
    /// "focus_weakest": Assault, the lowest-HP foe reachable this round.
    FocusWeakest,
    /// "defensive" (Guard & Counter): Guard, self.
    GuardCounter,
    /// "ranged_kite" (Kite at Range): ranged units Assault (and kite); melee units Disengage when offered, else Assault.
    KiteAtRange,
    /// "economy": Assault with the cheapest opener and no follow-ups (Rest).
    Economy,
}

impl Tactic {
    pub fn parse(preset: &str) -> Self {
        match preset {
            "aggressive" => Tactic::AllOut,
            "focus_weakest" => Tactic::FocusWeakest,
            "defensive" => Tactic::GuardCounter,
            "ranged_kite" => Tactic::KiteAtRange,
            "economy" => Tactic::Economy,
            _ => Tactic::Balanced,
        }
    }
}

/// Where one step goes (`World::step_from`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTo {
    Blocked,
    /// Onto an exit, or off an open arena: the unit leaves the battle.
    Leaves,
    Enters(V2),
}

#[derive(Clone)]
pub struct World {
    pub cat: Arc<Catalog>,
    pub units: Vec<Unit>,
    pub grid: Grid,
    pub allies: Vec<UnitId>,
    pub enemies: Vec<UnitId>,
    pub profiles: IndexMap<UnitId, AiProfile>,
    pub total_blocks: i32,
    /// Every HP change in order while someone follows them (Combatant.health_changed): (unit, HP after). None = off;
    /// a battle's metrics switch it on and drain it. Battle code changes HP only through `damage` and `heal`.
    pub hp_journal: Option<Vec<(UnitId, i32)>>,
    /// The battle's rules (RULESET1: `EncounterDef.ruleset`); every rule beyond classic reads its flag here first.
    pub ruleset: rpgkit_rules::ruleset::Ruleset,
    /// RULESET1_INTENT (rule `round_intent`): each unit's round intent, target, facing and round marks; empty in classic.
    pub round_states: IndexMap<UnitId, crate::round_intent::RoundState>,
    /// RULESET1 V2 (`EncounterDef.plan_reveal`): every enemy's whole plan is shown while planning; read only under the
    /// rule `plan_badges` (the Arena and the Dojo may flip it mid-battle).
    pub plan_reveal: bool,
    /// RULESET1 5 (rule `poise`): each unit's poise bar; empty in classic.
    pub poise: IndexMap<UnitId, crate::poise::PoiseState>,
    /// ENCOUNTER2 H2, H3: the gate of a hold escape (None: exits always let a unit out, as before).
    pub hold: Option<crate::encounter::HoldGate>,
    /// ENCOUNTER2 S1: units damage never takes below 1 HP (the encounter's `unkillable`).
    pub unkillable: Vec<UnitId>,
    /// S1: the unkillable units damage would have downed, in the order it first happened (once each; the session logs
    /// each one's cell).
    pub spared: Vec<UnitId>,
    /// HOOKS1 6 (rule `ground_hazards`): the placements, hazard hits and fades the session has not logged yet (the
    /// overlay itself is `grid.ground`).
    pub ground: crate::ground::Ledger,
    /// HOOKS1 9 (rule `unit_meters`): the meters the units hold this battle (empty in classic; no save key).
    pub meters: crate::meter::Meters,
    /// HOOKS1 7 (rule `arena_hazards`): the encounter's scheduled arena hazards and the round they are told for (empty
    /// in classic).
    pub arena: crate::arena::Arena,
    /// HOOKS1 AT2 (rule `ai_tactics`): who lost HP to an attack hit this round and last round (`brace_after_hit`).
    pub hits: HitMemory,
}

impl World {
    /// HOOKS1 4.1: whether the battle plays by AI tactics (target preferences SOFTEST and TOUGHEST, unit tactics,
    /// hazard-aware tiles).
    pub fn ai_tactics(&self) -> bool {
        self.ruleset.has(rpgkit_rules::ruleset::AI_TACTICS)
    }

    /// AT1: the preference `p` ranks by: without the rule SOFTEST and TOUGHEST read as ROLE (the role's own).
    pub fn target_pref(&self, p: &AiProfile) -> TargetPref {
        match p.target_pref {
            TargetPref::Softest | TargetPref::Toughest if !self.ai_tactics() => p.role_pref(),
            t => t,
        }
    }

    /// AT2: whether `c`'s live profile holds tactic `t` (never without the rule).
    pub fn has_tactic(&self, c: UnitId, t: UnitTactic) -> bool {
        self.ai_tactics() && self.profiles.get(&c).is_some_and(|p| p.tactics.contains(&t))
    }

    /// Whether the hit memory is kept: AT2 `brace_after_hit` (rule ai_tactics) or RULESET1-AI2's Guard memory (rule
    /// round_intent).
    fn keeps_hits(&self) -> bool {
        self.ai_tactics() || self.ruleset.has(rpgkit_rules::ruleset::ROUND_INTENT)
    }

    /// AT2 `brace_after_hit`, RULESET1-AI2: records the units an action's attack hits took HP from this round.
    pub fn note_hits(&mut self, struck: &[crate::trigger::Struck]) {
        if !self.keeps_hits() {
            return;
        }
        for s in struck.iter().filter(|s| s.damage > 0) {
            if !self.hits.this_round.contains(&s.target) {
                self.hits.this_round.push(s.target);
            }
        }
    }

    /// AT2, RULESET1-AI2: the round ends: this round's hits become last round's, and the units in the Guard intent now
    /// are last round's guards.
    pub fn roll_hits(&mut self) {
        if self.keeps_hits() {
            self.hits.last_round = std::mem::take(&mut self.hits.this_round);
            let guard = crate::round_intent::RoundIntent::Guard;
            self.hits.guarded_last_round =
                self.round_states.iter().filter(|(_, r)| r.intent == Some(guard)).map(|(&c, _)| c).collect();
        }
    }

    /// Combatant.take_damage on unit `c` (journaled). Returns the damage dealt. ENCOUNTER2 S1: an unkillable unit keeps
    /// 1 HP (the first time it would have fallen is recorded in `spared`).
    pub fn damage(&mut self, c: UnitId, amount: i32, mitigated: bool) -> i32 {
        self.damage_as(c, amount, mitigated, false)
    }

    /// ENCOUNTER2 P1: a phase `kill` on unit `c`: all its HP through `damage`'s path (journaled); `past_unkillable`
    /// (encounter::KILL_OVERRIDES_UNKILLABLE) takes an unkillable unit down too, recording nothing in `spared`.
    /// Returns the HP it lost.
    pub fn kill(&mut self, c: UnitId, past_unkillable: bool) -> i32 {
        let hp = self.units[c].hp;
        self.damage_as(c, hp, true, past_unkillable)
    }

    fn damage_as(&mut self, c: UnitId, amount: i32, mitigated: bool, past_unkillable: bool) -> i32 {
        let dealt = self.units[c].take_damage(amount, mitigated);
        if self.units[c].hp == 0 && !past_unkillable && self.unkillable.contains(&c) {
            self.units[c].hp = 1;
            if !self.spared.contains(&c) {
                self.spared.push(c);
            }
        }
        self.journal(c);
        dealt
    }

    /// Combatant.heal on unit `c` (journaled; a downed unit is not healed and nothing is recorded).
    pub fn heal(&mut self, c: UnitId, amount: i32) {
        if self.units[c].is_alive() {
            self.units[c].heal(amount);
            self.journal(c);
        }
    }

    fn journal(&mut self, c: UnitId) {
        if let Some(j) = self.hp_journal.as_mut() {
            j.push((c, self.units[c].hp));
        }
    }

    pub fn book(&self) -> Book<'_> {
        Book { cat: &self.cat }
    }

    /// Adds the synthetic skill (skill::item_skill) of every pouch item missing from this battle's catalog copy;
    /// call after pouches change.
    pub fn add_item_skills(&mut self) {
        let mut missing: Vec<rpgkit_data::SkillDef> = Vec::new();
        for p in self.units.iter().flat_map(|u| u.inventory.pouch.iter()) {
            let id = skill::item_skill_id(&p.id);
            if self.cat.skill(&id).is_none() && !missing.iter().any(|s| s.id == id) {
                missing.push(skill::item_skill(p));
            }
        }
        if !missing.is_empty() {
            let mut cat = (*self.cat).clone();
            cat.add_skills(missing);
            self.cat = Arc::new(cat);
        }
    }

    /// HOOKS1 4.1: with the rule `ground_hazards` off a skill's `hazard` is ignored, so the battle's catalog copy drops
    /// every skill's hazard: a zero-power ATTACK with one is then a plain zero-power attack on a unit, not a placement
    /// aimed at a tile (GH3), wherever the skill is read (plan, AI, dojo, resolver). Call after `add_item_skills` (item
    /// skills are covered too). No copy when the rule is on or no skill has a hazard (classic data has none); the skill
    /// order and ids stay, so skill indices hold.
    pub fn ignore_hazards_when_off(&mut self) {
        if crate::ground::on(self) || self.cat.skills.iter().all(|s| s.hazard.is_none()) {
            return;
        }
        let mut cat = (*self.cat).clone();
        for s in &mut cat.skills {
            s.hazard = None;
        }
        self.cat = Arc::new(cat);
    }

    pub fn u(&self, id: UnitId) -> &Unit {
        &self.units[id]
    }

    pub fn profile_of(&self, c: UnitId) -> AiProfile {
        self.profiles.get(&c).cloned().unwrap_or_else(|| AiProfile::create(if self.units[c].attack_range > 1 { Role::Sniper } else { Role::Aggressor }, 0.25, 1))
    }

    pub fn skill_cap_of(&self, c: UnitId) -> i32 {
        self.profile_of(c).skill_cap
    }

    pub fn foes_of(&self, c: UnitId) -> &[UnitId] {
        if self.allies.contains(&c) {
            &self.enemies
        } else if self.enemies.contains(&c) {
            &self.allies
        } else {
            &[]
        }
    }

    /// Whether `c` pays AP (RULESET1 T11): always, but for enemy-side units under the option `enemy_moves`, which pay
    /// nothing at a start (no chase tile, no move, no switch), gain none (Rest, refunds) and so never fall into debt.
    pub fn pays_ap(&self, c: UnitId) -> bool {
        !(self.ruleset.has(rpgkit_rules::ruleset::ENEMY_MOVES) && self.enemies.contains(&c))
    }

    /// The recovery blocks `c` books after the impact of `s` (RULESET1 T1, T4): the row's `recovery_blocks` for an
    /// enemy-side unit under `enemy_moves` using an Attack, Support or Focus skill (not an item); else 0 (party-side
    /// units, classic).
    pub fn recovery_of(&self, c: UnitId, s: &rpgkit_data::SkillDef) -> i32 {
        if self.pays_ap(c) || skill::is_item(s) || !matches!(skill::kind(s), skill::Kind::Attack | skill::Kind::Support | skill::Kind::Focus) {
            return 0;
        }
        s.recovery_blocks.max(0)
    }

    pub fn friends_of(&self, c: UnitId) -> &[UnitId] {
        if self.allies.contains(&c) {
            &self.allies
        } else if self.enemies.contains(&c) {
            &self.enemies
        } else {
            &[]
        }
    }

    // ---- CombatGrid operations that need unit state -----------------------------------------------------------

    /// On the grid and active (alive, not withdrawn).
    pub fn has(&self, c: UnitId) -> bool {
        self.grid.positions[c].is_some() && self.units[c].is_active()
    }

    pub fn pos(&self, c: UnitId) -> V2 {
        self.grid.position(c)
    }

    pub fn dist(&self, a: UnitId, b: UnitId) -> i32 {
        self.pos(a).dist(self.pos(b))
    }

    pub fn occupant(&self, cell: V2) -> Option<UnitId> {
        (0..self.units.len()).find(|&u| self.grid.positions[u] == Some(cell) && self.units[u].is_active())
    }

    pub fn is_occupied(&self, cell: V2) -> bool {
        self.occupant(cell).is_some()
    }

    pub fn place(&mut self, c: UnitId, cell: V2) -> bool {
        if !self.grid.is_valid(cell) || !self.grid.is_walkable(cell) || self.is_occupied(cell) {
            return false;
        }
        self.grid.positions[c] = Some(cell);
        true
    }

    pub fn remove(&mut self, c: UnitId) {
        self.grid.positions[c] = None;
    }

    fn leave(&mut self, c: UnitId) {
        self.grid.positions[c] = None;
        if !self.grid.withdrawn.contains(&c) {
            self.grid.withdrawn.push(c);
        }
        self.units[c].withdraw();
    }

    /// MORALE1: leaves the field at its own tile.
    pub fn withdraw_in_place(&mut self, c: UnitId) -> bool {
        if !self.units[c].is_active() || self.grid.positions[c].is_none() {
            return false;
        }
        self.leave(c);
        true
    }

    /// One step by `offset`; stepping onto an exit (or off an open arena) withdraws the unit. ENCOUNTER2 H3: while a
    /// hold escape's gate is shut (or for its holder) that step fails like a blocked tile.
    pub fn move_unit(&mut self, c: UnitId, offset: V2) -> bool {
        if !self.units[c].is_active() || self.grid.positions[c].is_none() {
            return false;
        }
        match self.step_from(c, self.pos(c), offset) {
            StepTo::Blocked => false,
            StepTo::Leaves => {
                self.leave(c);
                true
            }
            StepTo::Enters(target) => {
                self.grid.positions[c] = Some(target);
                true
            }
        }
    }

    /// `move_unit`'s rule for one step of `c` by `offset` from `from` (nothing changes): the tile it enters, the exit
    /// it leaves by, or blocked (off the board, not walkable, taken, a shut hold gate). The battle screen's walk preview
    /// reads it too (`command::walk_preview`).
    pub fn step_from(&self, c: UnitId, from: V2, offset: V2) -> StepTo {
        let target = from + offset;
        if self.grid.is_exit(target) || (self.grid.open && !self.grid.is_valid(target)) {
            if self.hold.as_ref().is_some_and(|g| !g.lets_out(self, c)) {
                return StepTo::Blocked;
            }
            return StepTo::Leaves;
        }
        // a step that stays put is blocked by the unit itself, as `is_occupied` has it
        if target == from || !self.grid.is_valid(target) || !self.grid.is_walkable(target) || self.occupant(target).is_some_and(|o| o != c) {
            return StepTo::Blocked;
        }
        StepTo::Enters(target)
    }

    pub fn jump_unit(&mut self, c: UnitId, cell: V2) -> bool {
        if !self.units[c].is_active() || self.grid.positions[c].is_none() {
            return false;
        }
        if !self.grid.is_valid(cell) || !self.grid.is_walkable(cell) || self.grid.is_exit(cell) || self.is_occupied(cell) {
            return false;
        }
        self.grid.positions[c] = Some(cell);
        true
    }

    // ---- Deployment (CombatGrid.deploy_combatants and friends) --------------------------------------------------

    pub fn layout_columns(deployment: &str, side: &str) -> Vec<i32> {
        let (party, a, b): (Vec<i32>, Vec<i32>, Vec<i32>) = match deployment {
            "ambush" => (vec![5, 4], vec![7, 8], vec![2, 1]),
            "centre" => (vec![5, 6], vec![8, 9], vec![3, 2]),
            _ => (vec![5, 4], vec![7, 8], vec![]),
        };
        match side {
            "party" => party,
            "a" => a,
            _ => b,
        }
    }

    pub fn group_of(index: usize, deployment: &str) -> usize {
        if Self::layout_columns(deployment, "b").is_empty() {
            0
        } else {
            index % 2
        }
    }

    pub fn spill_columns(front: i32, back: i32) -> Vec<i32> {
        let mut out = Vec::new();
        let probe = V2::new(front, 0);
        if Grid::is_party_half(probe) == Grid::is_party_half(V2::new(back, 0)) {
            let dir = (back - front).signum();
            let mut x = back + dir;
            while x >= 0 && x < GRID_W && Grid::is_party_half(V2::new(x, 0)) == Grid::is_party_half(probe) {
                out.push(x);
                x += dir;
            }
            return out;
        }
        let (mut lo, mut hi) = (front.min(back) - 1, front.max(back) + 1);
        while lo >= 0 || hi < GRID_W {
            if lo >= 0 {
                out.push(lo);
            }
            if hi < GRID_W {
                out.push(hi);
            }
            lo -= 1;
            hi += 1;
        }
        out
    }

    pub fn deploy(&mut self, allies: &[UnitId], enemies: &[UnitId], deployment: &str) {
        let party = Self::layout_columns(deployment, "party");
        self.deploy_side(allies, party[0], party[1]);
        let mut groups: [Vec<UnitId>; 2] = [Vec::new(), Vec::new()];
        for (i, e) in enemies.iter().enumerate() {
            groups[Self::group_of(i, deployment)].push(*e);
        }
        for (g, units) in groups.iter().enumerate() {
            let cols = Self::layout_columns(deployment, if g == 0 { "a" } else { "b" });
            if cols.len() == 2 && !units.is_empty() {
                self.deploy_side(units, cols[0], cols[1]);
            }
        }
    }

    fn deploy_side(&mut self, units: &[UnitId], front: i32, back: i32) {
        let spill = Self::spill_columns(front, back);
        for &c in units {
            let mut columns = if self.units[c].attack_range > 1 { vec![back, front] } else { vec![front, back] };
            columns.extend(&spill);
            for col in columns {
                if self.place_in_column(c, col) {
                    break;
                }
            }
        }
    }

    fn place_in_column(&mut self, c: UnitId, x: i32) -> bool {
        for y in DEPLOY_ROWS {
            let cell = V2::new(x, y);
            if self.grid.is_walkable(cell) && !self.grid.is_exit(cell) && !self.is_occupied(cell) {
                return self.place(c, cell);
            }
        }
        false
    }

    pub fn place_near_front(&mut self, c: UnitId, front: i32) -> bool {
        let anchor = V2::new(front, DEPLOY_ANCHOR_Y);
        let right = !Grid::is_party_half(anchor);
        let mut best = NO_CELL;
        let mut best_score = 1 << 30;
        for y in 0..GRID_H {
            let rank = DEPLOY_ROWS.iter().position(|r| *r == y).unwrap_or(DEPLOY_ROWS.len()) as i32;
            for x in 0..GRID_W {
                let cell = V2::new(x, y);
                if !self.grid.is_walkable(cell) || self.grid.is_exit(cell) || self.is_occupied(cell) {
                    continue;
                }
                let outward = if right { GRID_W - 1 - x } else { x };
                let score = cell.dist(anchor) * 10000 + rank * 100 + outward;
                if score < best_score {
                    best_score = score;
                    best = cell;
                }
            }
        }
        best != NO_CELL && self.place(c, best)
    }

    /// CatalogEncounter._place_leftovers (standard deployment): units off the grid take the free tile on their half
    /// nearest the centre.
    pub fn place_leftovers(&mut self, units: &[UnitId], party_side: bool, deployment: &str) {
        if !party_side && deployment != "standard" {
            let mut turn = 0;
            for &c in units {
                if self.has(c) {
                    continue;
                }
                let cols = Self::layout_columns(deployment, if turn % 2 == 0 { "a" } else { "b" });
                self.place_near_front(c, cols[0]);
                turn += 1;
            }
            return;
        }
        for &c in units {
            if self.has(c) {
                continue;
            }
            let mut best = NO_CELL;
            let mut best_score = 1 << 30;
            for y in 0..GRID_H {
                for x in 0..GRID_W {
                    let cell = V2::new(x, y);
                    if Grid::is_party_half(cell) != party_side || !self.grid.is_walkable(cell) || self.grid.is_exit(cell) || self.is_occupied(cell) {
                        continue;
                    }
                    let score = Grid::centre_distance(cell) * 1000 + y * 20 + x;
                    if score < best_score {
                        best_score = score;
                        best = cell;
                    }
                }
            }
            if best != NO_CELL {
                self.place(c, best);
            }
        }
    }
}
