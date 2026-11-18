//! The battle world: units, grid, sides and AI profiles (CombatGrid unit operations, PlanContext, AiProfile,
//! ContentFactory.ai_profile). Units are referred to by their index in `units`.

use crate::grid::{square_band, square_cells, square_covers, square_dist, square_nearest, Grid, V2, DEPLOY_ANCHOR_Y, DEPLOY_ROWS, GRID_H, GRID_W, NO_CELL};
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
            let tier_cap = rpgkit_rules::stats::tier_row(&e.tier).skill_cap;
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

/// BIGUNIT1 BU12: what `World::grow_into_place` did: the units pushed out of the new square (unit, from, to, in order),
/// and whether no square could be cleared, so the unit stays on one cell.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Growth {
    pub pushes: Vec<(UnitId, V2, V2)>,
    pub cramped: bool,
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
    /// INPUT1 GH7 (rule `round_intent`): the planner fills a member's free blocks after its picks (off: Rest). Only
    /// `round_intent_plan::set_autofill` changes it; the sims and the AI never do.
    pub autofill: bool,
    /// The AP every unit regains at a round start (`EncounterDef.ap_regen`; below 0: back to full), read by the AP
    /// forecast (INPUT1 AP1).
    pub ap_regen: i32,
    /// PHASEFX1: the units a phase `side` turned `hostile_all` and the foe / friend lists they bring (empty: the two
    /// sides as always). Only `set_side` changes it.
    pub sides: Sides,
    /// PK1-S1 (3.5, 3.7): the battle's marks, the units an `unmark` calms and the battle-start party side (`mark`).
    pub marks: crate::mark::Marks,
    /// PK2-S1: (unit, skill catalog id) -> how many times the unit started that skill this battle (only skills with a
    /// `uses_per_battle` are counted; `mark::uses_problem`).
    pub skill_uses: IndexMap<(UnitId, String), i32>,
    /// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): unit -> the main-hand weapon families it started an attack skill with
    /// this battle, each once (only heroes built from member state, `Unit::rank_choice`): the settle's mastery use
    /// (`rpgkit_world::roster::HeroEnd::attacked`).
    pub weapon_attacks: IndexMap<UnitId, Vec<String>>,
    /// PR6-6 (`PK4_PK5_CAMPAIGN_HUB.md` 1, B5): unit -> the commands (not moves) it started in a Channel round this
    /// battle (`channel::is_channel`; empty without the rule): the world settle's `HeroEnd::channel_actions`.
    pub channel_actions: IndexMap<UnitId, i32>,
    /// PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7): the sims' alternative dulled mode: a technique that swings a dulled
    /// weapon is locked too (`wear::problem`). False (the default dulled penalty alone) in every battle the sims do not switch;
    /// only the sim key `dulled=lock` sets it.
    pub dulled_lock: bool,
    /// PK3: rounds a downed deployed party member has before bleeding out permanently (0 = classic non-bleeding stasis).
    pub bleed_out_rounds: i32,
    /// PK3: deployed party members eligible for bleed-out (never guests, never enemies).
    pub bleed_heroes: Vec<UnitId>,
    /// PK3: downed party member -> rounds remaining before bleeding out permanently in this battle.
    pub bleed_remaining: IndexMap<UnitId, i32>,
    /// PK3: deployed party members that bled out to 0 rounds this battle (non-revivable).
    pub dead: Vec<UnitId>,
}

/// PHASEFX1 (ACTS23 PK6 hook l, engine-only): who fights whom once a phase `side` turned units `hostile_all`. A rogue
/// stays on the enemy side (the enemy AI plans it, it counts as a foe for the win objectives) but fights every other
/// unit: its foes are every unit of both sides, its only friend itself; the other enemy-side units have the party side
/// and the rogues as foes. The party side's lists are unchanged (the rogues are enemies already). `views` holds each
/// enemy-side unit's (foes, friends) while any rogue exists; empty otherwise, so `foes_of` and `friends_of` read the
/// two side lists exactly as before.
#[derive(Debug, Clone, Default)]
pub struct Sides {
    rogues: Vec<UnitId>,
    views: IndexMap<UnitId, (Vec<UnitId>, Vec<UnitId>)>,
}

impl Sides {
    /// The rogues, in the order they turned.
    pub fn rogues(&self) -> &[UnitId] {
        &self.rogues
    }

    pub fn is_rogue(&self, c: UnitId) -> bool {
        self.rogues.contains(&c)
    }
}

/// PK1-S1 (docs/sessions/PK1_CORRUPTION.md 3.1): a heal of `amount` (after its formula) scaled by a `heal_taken` of
/// `pct` percent: `amount * (100 + pct) / 100`, rounded down, at least 0 (0 = unchanged).
pub fn heal_taken(amount: i32, pct: i32) -> i32 {
    if pct == 0 || amount <= 0 {
        return amount;
    }
    (i64::from(amount) * i64::from(100 + pct)).div_euclid(100).clamp(0, i64::from(i32::MAX)) as i32
}

impl World {
    /// BIGUNIT1 BU3: a world with only units on a board (allies first, `n_allies` of them; classic ruleset, no AI
    /// profiles, nothing placed): what a deployment check needs (`deploy_misfits`).
    pub fn bare(cat: Arc<Catalog>, units: Vec<Unit>, grid: Grid, n_allies: usize) -> Self {
        let count = units.len();
        Self {
            cat,
            units,
            grid,
            allies: (0..n_allies).collect(),
            enemies: (n_allies..count).collect(),
            profiles: IndexMap::new(),
            total_blocks: 0,
            hp_journal: None,
            ruleset: Default::default(),
            round_states: IndexMap::new(),
            plan_reveal: false,
            poise: IndexMap::new(),
            hold: None,
            unkillable: Vec::new(),
            spared: Vec::new(),
            ground: Default::default(),
            meters: Default::default(),
            arena: Default::default(),
            hits: Default::default(),
            autofill: true,
            ap_regen: -1,
            sides: Default::default(),
            marks: crate::mark::Marks::new(&(0..n_allies).collect::<Vec<_>>()),
            skill_uses: IndexMap::new(),
            weapon_attacks: IndexMap::new(),
            channel_actions: IndexMap::new(),
            dulled_lock: false,
            bleed_out_rounds: 0,
            bleed_heroes: Vec::new(),
            bleed_remaining: IndexMap::new(),
            dead: Vec::new(),
        }
    }

    /// BIGUNIT1 BU3: deploys the units on `grid` as a battle start does (`deploy`, then each side's leftovers) and
    /// returns the indices (allies first) of the 2 x 2 units that found no square where all four cells fit.
    pub fn deploy_misfits(cat: Arc<Catalog>, grid: Grid, party: Vec<Unit>, enemies: Vec<Unit>, deployment: &str) -> Vec<UnitId> {
        let n_allies = party.len();
        let units: Vec<Unit> = party.into_iter().chain(enemies).collect();
        let mut grid = grid;
        grid.positions = vec![None; units.len()];
        let mut w = Self::bare(cat, units, grid, n_allies);
        let (allies, foes) = (w.allies.clone(), w.enemies.clone());
        w.deploy(&allies, &foes, deployment);
        w.place_leftovers(&allies, true, deployment);
        w.place_leftovers(&foes, false, deployment);
        (0..w.units.len()).filter(|&c| w.size_of(c) > 1 && !w.has(c)).collect()
    }

    /// U1.2 slice 1 (`FB-04`): unit `c`'s pool `id` through the typed facade: HP and AP from the unit, poise from
    /// `World.poise` (None in classic or before the first refill).
    pub fn pool(&self, c: UnitId, id: rpgkit_rules::pool::PoolId) -> Option<rpgkit_rules::pool::PoolView> {
        match id {
            rpgkit_rules::pool::PoolId::Poise => crate::poise::pool(self, c),
            _ => self.units.get(c)?.pool(id),
        }
    }

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
        let was_alive = self.units[c].is_alive();
        let dealt = self.units[c].take_damage(amount, mitigated);
        if self.units[c].hp == 0 && !past_unkillable && self.unkillable.contains(&c) {
            self.units[c].hp = 1;
            if !self.spared.contains(&c) {
                self.spared.push(c);
            }
        }
        if was_alive
            && self.units[c].hp == 0
            && self.bleed_out_rounds > 0
            && self.bleed_heroes.contains(&c)
            && !self.dead.contains(&c)
            && !self.units[c].is_withdrawn
        {
            self.bleed_remaining.insert(c, self.bleed_out_rounds);
        }
        self.journal(c);
        dealt
    }

    /// PK3: remaining bleed-out rounds for downed bleeding party member `c` (None if alive, dead, withdrawn, or classic).
    pub fn bleed_left(&self, c: UnitId) -> Option<i32> {
        let u = self.units.get(c)?;
        if u.is_alive() || u.is_withdrawn || self.dead.contains(&c) {
            return None;
        }
        self.bleed_remaining.get(&c).copied().filter(|&n| n > 0)
    }

    /// PK3: whether `c` has bled out permanently in this battle.
    pub fn is_dead(&self, c: UnitId) -> bool {
        self.dead.contains(&c)
    }

    /// PK3: whether `c` is a downed bleeding party member eligible to be revived.
    pub fn can_revive(&self, c: UnitId) -> bool {
        self.bleed_left(c).is_some()
    }

    /// PK3: cell where downed unit `c` will stand upon revive: in place if unoccupied, else nearest passable
    /// unoccupied non-exit floor tile within Chebyshev radius `1..=2`, sorted by `(distance, y, x)`.
    pub fn revive_cell(&self, c: UnitId) -> Option<V2> {
        let at = self.grid.positions.get(c).copied().flatten()?;
        // BIGUNIT1 BU4: a big unit needs its whole square free (`fits`)
        if self.size_of(c) > 1 {
            if self.fits(c, at) {
                return Some(at);
            }
        } else if self.occupant(at).is_none() {
            return Some(at);
        }
        for r in 1..=2 {
            let mut found: Option<V2> = None;
            for y in (at.y - r)..=(at.y + r) {
                for x in (at.x - r)..=(at.x + r) {
                    if (x - at.x).abs().max((y - at.y).abs()) != r {
                        continue;
                    }
                    let cell = V2::new(x, y);
                    let open = if self.size_of(c) > 1 { self.fits(c, cell) } else { self.grid.is_valid(cell) && self.grid.is_walkable(cell) && !self.grid.is_exit(cell) && self.occupant(cell).is_none() };
                    if open
                        && found.is_none_or(|b| (cell.y, cell.x) < (b.y, b.x))
                    {
                        found = Some(cell);
                    }
                }
            }
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// PK3: revives downed bleeding party member `c` at `revive_cell(c)` with `max(amount, 1)` clamped to `1..=max_hp`,
    /// clearing `bleed_remaining` and journaling HP; refuses with `"Space is blocked"` if no radius `1..=2` cell is free.
    pub fn revive(&mut self, c: UnitId, amount: i32) -> Result<(V2, i32), &'static str> {
        if !self.can_revive(c) {
            return Err("Target cannot be revived.");
        }
        let dest = self.revive_cell(c).ok_or("Space is blocked")?;
        self.grid.positions[c] = Some(dest);
        let max = self.units[c].max_hp.max(1);
        let hp = heal_taken(amount, self.units[c].band_heal_taken).clamp(1, max);
        self.units[c].hp = hp;
        self.bleed_remaining.shift_remove(&c);
        self.journal(c);
        Ok((dest, hp))
    }

    /// Combatant.heal on unit `c` (journaled; a downed unit is not healed and nothing is recorded). PK1-S1 (3.1): every
    /// heal source passes here after its formula, so the held bands' `heal_taken` scales it (`heal_taken`).
    pub fn heal(&mut self, c: UnitId, amount: i32) {
        if self.units[c].is_alive() {
            let amount = heal_taken(amount, self.units[c].band_heal_taken);
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
        // PHASEFX1: a rogue's battle (none without a `hostile_all` side effect)
        if let Some((foes, _)) = self.sides.views.get(&c) {
            return foes;
        }
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

    /// ACT1-HOOKS2 (FB-244 (4)): the blocks `s` takes for unit `c` (`Unit::skill_blocks`: its own `blocks` plus the
    /// unit's `skill_block_cost` deltas, at least 1); the planners, the AI, the previews and the commands read this.
    pub fn blocks_of(&self, c: UnitId, s: &rpgkit_data::SkillDef) -> i32 {
        self.units[c].skill_blocks(s)
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
        if let Some((_, friends)) = self.sides.views.get(&c) {
            return friends;
        }
        if self.allies.contains(&c) {
            &self.allies
        } else if self.enemies.contains(&c) {
            &self.enemies
        } else {
            &[]
        }
    }

    /// PHASEFX1: unit `c` goes to side `to` (`encounter::SIDE_TARGETS`): `hostile_all` puts it on the enemy side as a
    /// rogue (`Sides`), `friendly` or `hostile` on that side (no longer a rogue; PR5-50: the aliases `party` and `enemy`
    /// read the same, `encounter::side_id`); a unit changing side joins the end of its new side's list. Returns whether
    /// anything changed (an unknown `to` or a unit on no side changes nothing).
    pub fn set_side(&mut self, c: UnitId, to: &str) -> bool {
        use crate::encounter::{side_id, FRIENDLY_SIDE, HOSTILE_ALL, HOSTILE_SIDE};
        let (ally, rogue) = (self.allies.contains(&c), self.sides.rogues.contains(&c));
        if !ally && !self.enemies.contains(&c) {
            return false;
        }
        let (to_party, to_rogue) = match side_id(to) {
            Some(HOSTILE_ALL) => (false, true),
            Some(FRIENDLY_SIDE) => (true, false),
            Some(HOSTILE_SIDE) => (false, false),
            _ => return false,
        };
        if to_party == ally && to_rogue == rogue {
            return false;
        }
        if to_party && !ally {
            self.enemies.retain(|&e| e != c);
            self.allies.push(c);
        } else if !to_party && ally {
            self.allies.retain(|&a| a != c);
            self.enemies.push(c);
        }
        self.sides.rogues.retain(|&r| r != c);
        if to_rogue {
            self.sides.rogues.push(c);
        }
        self.rebuild_sides();
        true
    }

    /// PK1-S1 (3.7): rogue `c` (a unit that started on the party side) returns to the party side (`set_side` party) at
    /// its battle-start place among the allies: before every ally that came later in the start order and before the
    /// units that joined since (guests), so the heroes keep their order. False when it is no rogue.
    pub fn rejoin_party(&mut self, c: UnitId) -> bool {
        if !self.sides.is_rogue(c) || !self.set_side(c, crate::encounter::PARTY_SIDE) {
            return false;
        }
        let rank = |u: UnitId| self.marks.start_party().iter().position(|&s| s == u);
        let mine = rank(c).unwrap_or(usize::MAX);
        let others: Vec<UnitId> = self.allies.iter().copied().filter(|&a| a != c).collect();
        let at = others.iter().position(|&a| rank(a).is_none_or(|r| r > mine)).unwrap_or(others.len());
        let mut allies = others;
        allies.insert(at, c);
        self.allies = allies;
        self.rebuild_sides();
        true
    }

    /// PHASEFX1: each enemy-side unit's (foes, friends) while a rogue exists (`Sides`); none without one.
    fn rebuild_sides(&mut self) {
        self.sides.views.clear();
        if self.sides.rogues.is_empty() {
            return;
        }
        let rogues = self.sides.rogues.clone();
        for &c in &self.enemies {
            let view = if rogues.contains(&c) {
                (self.allies.iter().chain(&self.enemies).copied().filter(|&u| u != c).collect(), vec![c])
            } else {
                let foes = self.allies.iter().copied().chain(rogues.iter().copied()).collect();
                (foes, self.enemies.iter().copied().filter(|u| !rogues.contains(u)).collect())
            };
            self.sides.views.insert(c, view);
        }
    }

    /// PHASEFX1: unit `c` becomes `u` in place (a phase `swap`: same id, cell and side lists); its new HP is journaled.
    pub fn swap_unit(&mut self, c: UnitId, u: Unit) {
        self.units[c] = u;
        self.journal(c);
    }

    /// BIGUNIT1 BU12: a swapped-in unit `c` bigger than the one it replaced takes a square that holds its old cell:
    /// the old cell as top-left, top-right, bottom-left, then bottom-right of the square, the first that fits; else the
    /// fitting anchor nearest the old cell (distance to the square, then reading order). With no free square anywhere
    /// it makes room on the first square whose terrain fits and whose units can all be pushed (`push_out`): the four
    /// around the old cell in that order, then every such square on the board by distance, then reading order. When no
    /// square can be cleared (the board holds too few free cells) the unit stays on its old cell at one cell for the rest
    /// of the battle (`cramped`; never an overlap). Nothing moves for a one-cell unit or one already fitting. Pushes are
    /// not attacks (the caller rolls hazards and logs them).
    pub fn grow_into_place(&mut self, c: UnitId) -> Growth {
        let n = self.size_of(c);
        let Some(old) = self.grid.positions[c] else { return Growth::default() };
        if n == 1 || !self.units[c].is_active() {
            return Growth::default();
        }
        let around: Vec<V2> = vec![old, old - V2::new(n - 1, 0), old - V2::new(0, n - 1), old - V2::new(n - 1, n - 1)];
        if let Some(&a) = around.iter().find(|&&a| self.fits(c, a)) {
            self.grid.positions[c] = Some(a);
            return Growth::default();
        }
        let board = |keep: &dyn Fn(V2) -> bool| -> Vec<V2> {
            let mut all: Vec<(i32, i32, V2)> = Vec::new();
            for y in 0..GRID_H {
                for x in 0..GRID_W {
                    let a = V2::new(x, y);
                    if keep(a) {
                        all.push((square_dist(a, n, old, 1), y * GRID_W + x, a));
                    }
                }
            }
            all.sort_by_key(|k| (k.0, k.1));
            all.into_iter().map(|k| k.2).collect()
        };
        if let Some(&a) = board(&|a| self.fits(c, a)).first() {
            self.grid.positions[c] = Some(a);
            return Growth::default();
        }
        let mut squares: Vec<V2> = around.iter().copied().filter(|&a| self.grid.square_fits(a, n)).collect();
        for a in board(&|a| self.grid.square_fits(a, n)) {
            if !squares.contains(&a) {
                squares.push(a);
            }
        }
        for a in squares {
            if let Some((pos, pushes)) = self.push_out(c, a) {
                self.grid.positions = pos;
                self.grid.positions[c] = Some(a);
                return Growth { pushes, cramped: false };
            }
        }
        self.units[c].footprint = 1;
        Growth { pushes: Vec::new(), cramped: true }
    }

    /// BU12 make room: the positions after `c` takes the square at `a` and every unit on it is pushed out, or None when
    /// some push finds nowhere to go. Units leave in the reading order of the cells they stand on; each goes to the
    /// nearest free spot (its whole footprint walkable, not an exit, outside the cells already claimed, no unit; distance
    /// from where it stood, then reading order). With no free spot it takes the nearest spot whose units have not moved
    /// yet and those units are pushed in turn (a chain). Each unit moves at most once, so the chain is bounded by the
    /// unit count. Returns (positions, pushes (unit, from, to) in order); nothing is written.
    fn push_out(&self, c: UnitId, a: V2) -> Option<(Vec<Option<V2>>, Vec<(UnitId, V2, V2)>)> {
        let n = self.size_of(c);
        let mut pos = self.grid.positions.clone();
        pos[c] = Some(a);
        let live = |u: UnitId| u != c && self.units[u].is_active();
        let on = |pos: &[Option<V2>], cell: V2| -> Vec<UnitId> { (0..pos.len()).filter(|&u| live(u) && pos[u].is_some_and(|p| square_covers(p, self.size_of(u), cell))).collect() };
        let mut claimed: Vec<V2> = square_cells(a, n).collect();
        let mut moved: Vec<UnitId> = vec![c];
        let mut queue: Vec<UnitId> = Vec::new();
        let enqueue = |queue: &mut Vec<UnitId>, moved: &[UnitId], us: Vec<UnitId>| {
            for u in us {
                if !queue.contains(&u) && !moved.contains(&u) {
                    queue.push(u);
                }
            }
        };
        for cell in claimed.clone() {
            enqueue(&mut queue, &moved, on(&pos, cell));
        }
        let mut pushes = Vec::new();
        while !queue.is_empty() {
            let o = queue.remove(0);
            let (from, size) = (pos[o]?, self.size_of(o));
            let nearest = |ok: &dyn Fn(V2) -> bool| -> Option<V2> {
                let mut best: Option<(i32, i32, V2)> = None;
                for y in 0..GRID_H {
                    for x in 0..GRID_W {
                        let to = V2::new(x, y);
                        if self.grid.square_fits(to, size) && square_cells(to, size).all(|q| !claimed.contains(&q)) && ok(to) {
                            let key = (to.dist(from), y * GRID_W + x);
                            if best.is_none_or(|b| key < (b.0, b.1)) {
                                best = Some((key.0, key.1, to));
                            }
                        }
                    }
                }
                best.map(|b| b.2)
            };
            let others = |to: V2| -> Vec<UnitId> {
                let mut us: Vec<UnitId> = Vec::new();
                for q in square_cells(to, size) {
                    for u in on(&pos, q) {
                        if u != o && !us.contains(&u) {
                            us.push(u);
                        }
                    }
                }
                us
            };
            let to = match nearest(&|to| others(to).is_empty()) {
                Some(to) => to,
                None => nearest(&|to| others(to).iter().all(|u| !moved.contains(u)))?,
            };
            let bumped = others(to);
            pos[o] = Some(to);
            claimed.extend(square_cells(to, size));
            moved.push(o);
            enqueue(&mut queue, &moved, bumped);
            pushes.push((o, from, to));
        }
        Some((pos, pushes))
    }

    // ---- CombatGrid operations that need unit state -----------------------------------------------------------

    /// On the grid and active (alive, not withdrawn).
    pub fn has(&self, c: UnitId) -> bool {
        self.grid.positions[c].is_some() && self.units[c].is_active()
    }

    pub fn pos(&self, c: UnitId) -> V2 {
        self.grid.position(c)
    }

    /// BIGUNIT1 BU5: the least distance between any cell of `a` and any cell of `b` (`V2::dist` for one-cell units).
    pub fn dist(&self, a: UnitId, b: UnitId) -> i32 {
        square_dist(self.pos(a), self.size_of(a), self.pos(b), self.size_of(b))
    }

    // ---- BIGUNIT1: footprints (a unit fills the `size_of` x `size_of` square anchored at its position) ------------

    /// BIGUNIT1 BU1: the side of the square unit `c` fills (1, or 2 for a 2 x 2 unit).
    pub fn size_of(&self, c: UnitId) -> i32 {
        if self.units[c].footprint == 2 {
            2
        } else {
            1
        }
    }

    /// BIGUNIT1 BU2: the cells unit `c` fills, in reading order (empty off the grid).
    pub fn cells_of(&self, c: UnitId) -> Vec<V2> {
        self.grid.positions[c].map(|a| square_cells(a, self.size_of(c)).collect()).unwrap_or_default()
    }

    /// BIGUNIT1 BU2: the cells unit `c` would fill anchored at `anchor`.
    pub fn cells_at(&self, c: UnitId, anchor: V2) -> Vec<V2> {
        square_cells(anchor, self.size_of(c)).collect()
    }

    /// BIGUNIT1 BU5: the least distance from `cell` to any cell of unit `b` (`pos(b).dist(cell)` for a one-cell unit).
    pub fn dist_to(&self, b: UnitId, cell: V2) -> i32 {
        square_dist(cell, 1, self.pos(b), self.size_of(b))
    }

    /// BIGUNIT1 BU5: the least distance from unit `a` standing anchored at `at` to any cell of unit `b` (a planned
    /// position's reach and range; `at.dist(pos(b))` for one-cell units).
    pub fn dist_at(&self, a: UnitId, at: V2, b: UnitId) -> i32 {
        square_dist(at, self.size_of(a), self.pos(b), self.size_of(b))
    }

    /// BIGUNIT1 BU5: the least distance from unit `a` standing anchored at `at` to `cell` (`at.dist(cell)` for a
    /// one-cell unit).
    pub fn dist_at_cell(&self, a: UnitId, at: V2, cell: V2) -> i32 {
        square_dist(at, self.size_of(a), cell, 1)
    }

    /// BIGUNIT1 BU5: `round_intent::band_dist` from unit `a` anchored at `at` to unit `b` (diagonals under `diag`).
    pub fn band_at(&self, diag: bool, a: UnitId, at: V2, b: UnitId) -> i32 {
        square_band(diag, at, self.size_of(a), self.pos(b), self.size_of(b))
    }

    /// BIGUNIT1 BU5: `round_intent::band_dist` from `cell` to unit `b`.
    pub fn band_to(&self, diag: bool, b: UnitId, cell: V2) -> i32 {
        square_band(diag, cell, 1, self.pos(b), self.size_of(b))
    }

    /// BIGUNIT1 BU10, BU11: the cell of unit `c` nearest `to` (its position for a one-cell unit).
    pub fn nearest_cell(&self, c: UnitId, to: V2) -> V2 {
        square_nearest(self.pos(c), self.size_of(c), to)
    }

    /// BIGUNIT1 BU3, BU6: unit `c` fits anchored at `anchor`: every cell of its square is on the grid, walkable, not an
    /// exit and holds no active unit but `c`.
    pub fn fits(&self, c: UnitId, anchor: V2) -> bool {
        self.grid.square_fits(anchor, self.size_of(c)) && square_cells(anchor, self.size_of(c)).all(|cell| self.occupant(cell).is_none_or(|o| o == c))
    }

    /// BIGUNIT1: a deployment tile is open for `c` (one-cell: walkable, not an exit, free, as before; a big unit:
    /// `fits`).
    fn open_for(&self, c: UnitId, cell: V2) -> bool {
        if self.size_of(c) == 1 {
            self.grid.is_walkable(cell) && !self.grid.is_exit(cell) && !self.is_occupied(cell)
        } else {
            self.fits(c, cell)
        }
    }

    /// BIGUNIT1 BU5, BU13: the anchors `c` could stand on with the square of side `t_size` at `t_anchor` in the band
    /// `(lo, hi)` (`square_band`, diagonals under `diag`): its square on the grid, walkable, not an exit and clear of
    /// `blocked` (one-cell units: the cells around the target in the band, as before).
    pub fn stand_goals(&self, c: UnitId, t_anchor: V2, t_size: i32, band: (i32, i32), diag: bool, blocked: &crate::path::Cells) -> crate::path::Cells {
        let (sc, hi) = (self.size_of(c), band.1);
        let mut goals = crate::path::Cells::new();
        for y in (t_anchor.y - hi - (sc - 1))..=(t_anchor.y + t_size.max(1) - 1 + hi) {
            for x in (t_anchor.x - hi - (sc - 1))..=(t_anchor.x + t_size.max(1) - 1 + hi) {
                let a = V2::new(x, y);
                let d = square_band(diag, a, sc, t_anchor, t_size);
                if d < band.0 || d > hi {
                    continue;
                }
                if square_cells(a, sc).all(|cell| self.grid.is_valid(cell) && self.grid.is_walkable(cell) && !self.grid.is_exit(cell) && !blocked.contains(&cell)) {
                    goals.insert(a);
                }
            }
        }
        goals
    }

    /// BIGUNIT1 BU6: `c` anchored at `anchor` stands clear of `blocked`: its whole square on the grid, walkable and not
    /// an exit (the planner's tile test for a one-cell unit).
    pub fn clear_at(&self, c: UnitId, anchor: V2, blocked: &crate::path::Cells) -> bool {
        square_cells(anchor, self.size_of(c)).all(|cell| self.grid.is_valid(cell) && self.grid.is_walkable(cell) && !self.grid.is_exit(cell) && !blocked.contains(&cell))
    }

    /// BIGUNIT1 BU9: whether any cell of unit `f` anchored at `anchor` is in `cells` (one-cell: `cells.contains(anchor)`).
    pub fn square_in(&self, f: UnitId, anchor: V2, cells: &[V2]) -> bool {
        let n = self.size_of(f);
        if n == 1 {
            return cells.contains(&anchor);
        }
        cells.iter().any(|c| square_covers(anchor, n, *c))
    }

    /// BIGUNIT1 BU9: the active units filling `cells`, each once, in the order their first cell appears.
    pub fn occupants_once(&self, cells: &[V2]) -> Vec<(V2, UnitId)> {
        let mut out: Vec<(V2, UnitId)> = Vec::new();
        for &cell in cells {
            if let Some(v) = self.occupant(cell) {
                if !out.iter().any(|(_, u)| *u == v) {
                    out.push((cell, v));
                }
            }
        }
        out
    }

    /// BIGUNIT1 BU4: the active unit filling `cell` (any of a big unit's cells).
    pub fn occupant(&self, cell: V2) -> Option<UnitId> {
        (0..self.units.len()).find(|&u| self.grid.positions[u].is_some_and(|a| square_covers(a, self.size_of(u), cell)) && self.units[u].is_active())
    }

    pub fn is_occupied(&self, cell: V2) -> bool {
        self.occupant(cell).is_some()
    }

    pub fn place(&mut self, c: UnitId, cell: V2) -> bool {
        if self.size_of(c) > 1 {
            if !self.fits(c, cell) {
                return false;
            }
        } else if !self.grid.is_valid(cell) || !self.grid.is_walkable(cell) || self.is_occupied(cell) {
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
        // BIGUNIT1 BU6: a big unit steps only when its whole square fits there; it never leaves by an exit
        if self.size_of(c) > 1 {
            return if target != from && self.fits(c, target) { StepTo::Enters(target) } else { StepTo::Blocked };
        }
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
        if self.size_of(c) > 1 {
            if !self.fits(c, cell) {
                return false;
            }
        } else if !self.grid.is_valid(cell) || !self.grid.is_walkable(cell) || self.grid.is_exit(cell) || self.is_occupied(cell) {
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
            if self.open_for(c, cell) {
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
                if !self.open_for(c, cell) {
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
                    if Grid::is_party_half(cell) != party_side || !self.open_for(c, cell) {
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
