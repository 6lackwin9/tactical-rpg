# Rules Engine Architecture & Systems Implementation Plan

> **Category:** Technical Systems & Core Engine Specifications  
> **Location:** `games/emberwake/lore/systems/rules_engine_architecture.md`  
> **Target Crates:** `emberwake_rules`, `emberwake_battle`, `emberwake_data`  
> **Related Specs:** [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md), [magic_and_corruption.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/magic_and_corruption.md), [skill_acquisition_and_loadout_systems.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/skill_acquisition_and_loadout_systems.md)

---

## 1. Architectural Overview & Philosophy

The *Emberwake* rules engine governs combat math, timeline scheduling, character stats, poise/stagger, and eldritch corruption. 

The existing codebase (`games/emberwake/crates/`) already contains a robust foundation ported from Godot:
- **`emberwake_rules`**: Pure deterministic game rules (stats, gear, stat modifiers, damage formula).
- **`emberwake_battle`**: Grid coordinates, A* pathing, 6-block timeline scheduling (`Resolver`), command execution (`Command`), and encounter hooks.

Our objective is to **reuse over 80% of existing code**, surgically extending data structures and replacing legacy legacy rock-paper-scissors stances with the **6-Tick Timeline** and **Strain/Poise** mechanics.

```text
                                  [ EMBERWAKE ENGINE ARCHITECTURE ]
                                                 |
         +---------------------------------------+---------------------------------------+
         |                                                                               |
         v                                                                               v
[ emberwake_rules ] (Deterministic Core)                             [ emberwake_battle ] (Timeline & Arena)
- CharacterStats & Proficiencies (100% REUSED)                       - 2D Discrete Grid & A* Pathing (100% REUSED)
- AttackPower & Weapon Scaling (100% REUSED)                         - Resolver Timeline & ScheduledActions (90% REUSED)
- Armor Mitigation Curve (100% REUSED)                               - Command Execution & Kind Enums (85% REUSED)
+ Strain & Calcification State (NEW)                                 + Impact Tick vs. Windup/Recovery (EXTENDED)
+ Poise / Stagger Bar Math (NEW)                                     + Stagger Interrupt Trigger (NEW)
- Legacy Stance Rock-Paper-Scissors (REMOVED)                         - Stance Switch AP Cost (REMOVED)
```

---

## 2. Pillar 1: Character State & Stats (`emberwake_rules::unit` & `stats`)

### 2.1 Direct Reuse Matrix
* **`stats::CharacterStats` (100% Reused):** Base attributes (`STR`, `DEX`, `CON`, `INT`, `KNOW`, `PERC`), spend pool, and stat modifiers operate unchanged.
* **`stats::StatModifier` (100% Reused):** Flat and percentage stat buffs/debuffs (`context`, `duration`, `matches()`) cleanly handle status effects and talisman passives.
* **`gear::Inventory` & `Equipment` (100% Reused):** Mainhand, offhand, armor, trinket slots, wield penalties, and training power boosts require zero changes.
* **`unit::Unit` (Base Fields Reused):** `hp`, `max_hp`, `speed`, `ap`, `max_ap`, `stats`, `inventory`, `known_skills`.

### 2.2 Structural Extensions to `Unit`
To support corruption and poise, we extend `Unit` with two compact sub-states:

```rust
// Proposed additions in emberwake_rules::unit:

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StrainState {
    /// Current combat strain level (0 to 100+). Resets toward dormant_floor post-battle.
    pub current: i32,
    /// Permanent floor established by installed aberrant organ grafts (0% to 50%).
    pub dormant_floor: i32,
    /// Permanent anatomical decay stage (0 = Pure Unsealed, 1 = Vein-Blackened ... 5 = Petrified/Dead).
    pub calcification_stage: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoiseState {
    /// Current poise/stagger points remaining before an interrupt occurs.
    pub current: i32,
    /// Maximum poise, derived from CON + Armor Weight + Passive Traits.
    pub max_poise: i32,
    /// Whether the unit suffered a Poise Break this round (cleared at round end).
    pub is_broken: bool,
}

// In struct Unit:
pub struct Unit {
    // ... existing fields (name, hp, ap, speed, stats, inventory) ...
    
    // NEW FIELDS:
    pub strain: StrainState,
    pub poise: PoiseState,
    
    // RETIRED FIELDS:
    // pub stance: Stance,  // <-- DROPPED (Eliminates legacy Rock-Paper-Scissors)
}
```

### 2.3 Options for Strain Allocation
* **Option A: Real-Time Strain Accumulator (Preferred Baseline — Subject to Playtest Review):**
  - **Mechanic:** Inoculated maneuvers immediately add their `strain_cost` to `unit.strain.current`. Surfing brackets (`Tremor`, `Flesh-Tension`, `Rupture Verge`) activate instantaneously on the tick the action begins.
  - **Why it improves experience:** Provides immediate visual feedback and dynamic risk. An operative can push into *Flesh-Tension* (+25% damage) on Tick 1 to boost a heavy strike landing on Tick 4.
  - **Status:** Preferred for vertical slice; balance will be tuned during combat playtests.
* **Option B: Round-End Strain Batching:**
  - **Mechanic:** Strain costs accumulate in a pending buffer and apply at the end of the 6-tick round.
  - **Trade-off:** Less tactical reactivity; removes the excitement of "riding the surge" mid-round.

---

## 3. Pillar 2: Damage & Combat Calculation (`emberwake_rules::damage`)

### 3.1 Direct Reuse Matrix
* **`damage::attack_power()` (100% Reused):**
  $$\text{Attack Power} = (\text{Weapon Power} + \text{Unit Scaling}) \times \text{Tier Scale}$$
  Weapon fitting rules, wield penalties for insufficient STR/DEX, and training boosts function without modification.
* **`damage::armor_reduction()` (100% Reused):**
  $$\text{Mitigation Ratio} = \frac{\text{Rating}}{\text{Rating} + 50.0}$$
  The existing hyperbolic armor curve ($K=50.0$) remains mathematically sound and prevents armor stacking from achieving true invulnerability.
* **`damage::crit_chance()` & `GUARD_CAP` (100% Reused):**
  DEX-based crit scaling and the 90% maximum guard cap are preserved.

### 3.2 Refinements to `damage::compute()`
In the legacy code:
```rust
// Legacy Godot formula in damage.rs:
let stance_factor = stance.matchup(target.stance); // 0.75, 1.0, or 1.25 rock-paper-scissors
let gross = power as f64 * buff * h.skill_mult * stance_factor * crit_mult * (1.0 - guard_term) * (1.0 - armor_reduction(rating));
```

We streamline this by replacing `stance_factor` with **Strain Multipliers** and **Armor Piercing**:

```rust
// Enhanced Emberwake damage calculation in damage.rs:
pub struct HitInput {
    pub ranged: bool,
    pub offhand: bool,
    pub skill_mult: f64,
    pub force_crit: bool,
    pub crit_roll: Option<f64>,
    pub include_guard: bool,
    pub cover: f64,
    
    // NEW EMBERWAKE PARAMETERS:
    /// True for Inoculated / aberrant attacks that ignore armor mitigation.
    pub armor_piercing: bool,
    /// Poise damage to apply to target's PoiseState.
    pub stagger_damage: i32,
}

pub fn compute(attacker: &Unit, target: &Unit, h: HitInput) -> Hit {
    let explained = attack_power(attacker, h.ranged, h.offhand);
    let power = explained.attack_power;
    let buff = attacker.attack_boost.max(1.0);
    
    // 1. Strain Bracket Multiplier (Surfing the Corruption)
    let strain_mult = match attacker.strain.current {
        50..=74 => 1.25, // Flesh-Tension (+25% damage)
        75..=99 => 1.50, // Rupture Verge (+50% damage)
        _ => 1.0,
    };
    
    let chance = crit_chance(attacker) + if attacker.strain.current >= 30 { 15.0 } else { 0.0 };
    let is_crit = h.force_crit || h.crit_roll.is_some_and(|r| r * 100.0 < chance);
    let crit_mult = if is_crit { damage::CRIT_MULT } else { 1.0 };
    
    // 2. Active Guard & Cover
    let guard = if h.include_guard { target.guard } else { 0.0 };
    let guard_term = damage::GUARD_CAP.min(guard + h.cover.max(0.0));
    
    // 3. Armor Mitigation (Bypassed if armor_piercing == true)
    let rating = armor_rating(target);
    let armor_mult = if h.armor_piercing {
        1.0 // Pure aberrant bypass
    } else {
        1.0 - armor_reduction(rating)
    };
    
    let gross = power as f64 * buff * strain_mult * h.skill_mult * crit_mult * (1.0 - guard_term) * armor_mult;
    
    Hit {
        attack_power: power,
        is_crit,
        crit_rate: chance,
        armor_rating: rating,
        final_dmg: roundi(gross).max(1),
        // ... explanation diagnostics ...
    }
}
```

### 3.3 Options for Armor Piercing Mechanics
* **Option A: Binary Full Piercing for Inoculation (Preferred Baseline — Subject to Playtest Review):**
  - Inoculated attacks (`armor_piercing: true`) completely ignore physical armor, making them indispensable against heavily fortified Inquisitors and Cathedral Knights.
  - Keeps mental math crystal clear: Cold Steel sunders armor or breaks poise; Inoculation bypasses armor at the cost of biological strain.
  - **Status:** Preferred for vertical slice; viability vs. flat penetration will be reviewed during combat balancing.
* **Option B: Percentage Armor Penetration:**
  - Inoculated skills ignore a flat $50\%$ of target armor rating.
  - **Trade-off:** Adds unnecessary math rounding and reduces the dramatic contrast between Cold Steel and Inoculation.

---

## 4. Pillar 3: The 6-Tick Simultaneous Timeline (`emberwake_battle::resolver`)

### 4.1 Direct Reuse Matrix
* **`Resolver` Core Loop (90% Reused):**
  - `Resolver::schedule(unit, command, start)`: Validates block availability and collision checks.
  - `Resolver::prepare_block(w, block)`: Prepares the active tick, raises Active Guards, and sorts resolving commands.
  - `ScheduledAction.cancelled`: Pre-existing field used directly to implement **Stagger Interrupts**!
  - `total_blocks = 6`: The engine's native default `total_blocks` matches the 6-tick timeline exactly!

### 4.2 Refining Action Resolution: Impact Ticks vs. Legacy Block-End
In the legacy code, all commands resolve their effects strictly at `end == block`.  
In *Emberwake*, we enhance `Command` to recognize **Impact Ticks**:

```text
LEGACY TIMELINE RESOLUTION:
Tick:       [ 1 ]       [ 2 ]       [ 3 ]       [ 4 ]       [ 5 ]       [ 6 ]
Action:     |==================== Command (4 blocks) ===================>| Effect Resolves
                                                                          (Too late for interrupts)

EMBERWAKE 3-PHASE TIMELINE:
Tick:       [ 1 ]       [ 2 ]       [ 3 ]       [ 4 ]       [ 5 ]       [ 6 ]
Action:     |-- Windup (T1-T3) --| [ IMPACT (T4) ] |-- Recovery (T5-T6) --|
            ^ Vulnerable to Stagger Interrupts     ^ Committed / Unable to Dodge
```

```rust
// Proposed extension to ScheduledAction in emberwake_battle::resolver:

#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub unit: UnitId,
    pub command: Command,
    pub start: i32,         // e.g. Tick 1
    pub impact: i32,        // e.g. Tick 4 (Damage and status hit here)
    pub end: i32,           // e.g. Tick 6 (Recovery completes here)
    pub cancelled: bool,    // If true, interrupted by Stagger!
}
```

### 4.3 Execution in `prepare_block`
During `prepare_block(w, block)`:
1. **At `block < action.impact` (Windup):** If the unit's poise reaches 0 from incoming hits, `action.cancelled = true`. The attack is shattered before landing!
2. **At `block == action.impact` (Impact):** If `!action.cancelled`, the strike, heal, or area effect resolves on target(s).
3. **At `block > action.impact` (Recovery):** The unit cannot be interrupted (the attack already landed), but remains committed and cannot schedule new actions until `action.end`.

### 4.4 Options for Defining `impact_tick`
* **Option A: Standard Speed Tier Mapping (Preferred Baseline — Subject to Playtest Review):**
  - *Snap (2 Ticks):* Windup = 1 tick, Impact = Tick 2, Recovery = 0 ticks.
  - *Standard (4 Ticks):* Windup = 2 ticks, Impact = Tick 3, Recovery = 1 tick.
  - *Heavy (6 Ticks):* Windup = 3 ticks, Impact = Tick 4, Recovery = 2 ticks.
  - **Benefit:** Highly predictable telegraphing for the player. Scrubbing the timeline UI clearly indicates when the danger window occurs.
  - **Status:** Preferred for vertical slice; telegraph feel and interrupt satisfaction will be evaluated in-engine.
* **Option B: Fully Authored Per-Skill Impact Ticks:**
  - Every skill in `SkillDef` authors individual `windup_ticks` and `recovery_ticks`.
  - **Trade-off:** Increased catalog authoring burden; harder for players to read enemy intentions at a glance.

---

## 5. Pillar 4: Poise, Stagger & Interrupt Engine

### 5.1 Mechanics Breakdown
Every combatant has a real-time **Poise Bar**:
$$\text{Max Poise} = (\text{CON} \times 2) + \text{Armor Poise Rating} + \text{Discipline Buffs}$$

When damage is dealt, the attacker's `stagger_damage` reduces the defender's current poise:
1. **Active Guard Defense:** If the defender is executing **Active Guard** on that tick, incoming stagger damage is reduced by $50\%$.
2. **Flesh-Tension Vulnerability:** If the defender is in the *Flesh-Tension* Strain bracket (50%–74%), incoming stagger damage is **doubled**.
3. **Poise Break (Stagger):** When poise reaches $\le 0$:
   - Any active action currently in its **Windup phase** is marked `cancelled = true`.
   - The unit is inflicted with `status::STAGGERED` ($-50\%$ Evasion, takes $+25\%$ damage from subsequent hits this round).
   - A floating visual indicator **`"BREAK!"`** triggers in the battle log.

```rust
// Proposed handler in emberwake_battle::resolver:
pub fn apply_stagger_damage(w: &mut World, target: UnitId, amount: i32, scheduled: &mut [ScheduledAction]) {
    let u = &mut w.units[target];
    let actual_stagger = if u.guard > 0.0 {
        amount / 2 // Guard halves stagger
    } else if u.strain.current >= 50 && u.strain.current <= 74 {
        amount * 2 // Flesh-Tension doubles stagger
    } else {
        amount
    };
    
    u.poise.current -= actual_stagger;
    
    if u.poise.current <= 0 && !u.poise.is_broken {
        u.poise.is_broken = true;
        u.poise.current = 0;
        
        // Interrupt any action in windup!
        for action in scheduled.iter_mut() {
            if action.unit == target && !action.cancelled {
                action.cancelled = true;
                // Log interrupt event
            }
        }
    }
}
```

---

## 6. Pillar 5: Aberration Crisis Engine (Strain 100%+)

When an operative executes an Inoculated maneuver that pushes their Strain to $100\%$ or beyond:
1. The maneuver executes with catastrophic force (+50% damage, full armor pierce).
2. At the completion of the round, the operative triggers an **Aberration Crisis Roll**:
   $$D100 + \text{Dormant Strain Floor} - \text{Composure}$$

### 6.1 Crisis Outcome Mapping
```rust
// In emberwake_battle::resolver or session:
pub enum AberrationCrisisResult {
    /// 01–30: The operative retains human will; Strain drops to 70%.
    FleshMastery,
    /// 31–55: Violent bone-spikes burst outward, dealing 15 damage to all adjacent units (allies & foes).
    MarrowRupture,
    /// 56–75: Mind blurs into predatory frenzy; operative attacks nearest unit next round regardless of allegiance.
    PredatoryFrenzy,
    /// 76–90: Rapid cellular hardening; operative gains +5 Armor but permanently advances 1 Calcification Stage!
    SpontaneousCalcification,
    /// 91–100+: Complete biological breakdown; operative permanently transforms into a hostile Aberration!
    IrreversibleMetamorphosis,
}
```

---

## 7. Comparison: What We Kept, What We Replaced, and What We Improved

| Subsystem | Legacy Engine / Godot Baseline | Emberwake Redesign | Improvement / Rationale |
| :--- | :--- | :--- | :--- |
| **Combat Stances** | 4 stances (`Neutral`, `Offensive`, `Defensive`, `Tactical`) with rock-paper-scissors multiplier ($0.75\times / 1.25\times$) | **Dropped.** Absorbed into Active Guard, Speed Tiers, and Strain brackets | Eliminates tedious stance toggling; aligns defense with temporal commitment. |
| **Action Resolution** | Commands resolve strictly at end of block cost (`end == block`) | **3-Phase Blocks:** Windup $\to$ Impact Tick $\to$ Recovery | Enables counter-play, telegraphing, and dramatic Stagger interrupts. |
| **Armor Formula** | $r / (r + 50.0)$ hyperbolic curve | **Reused $100\%$,** plus binary **Armor Piercing** for Inoculated skills | Keeps physical plate tanky against steel while rewarding eldritch risk against heavy armor. |
| **Resource Economy** | AP only (10 base, 4 regen/turn) | **Dual Economy:** Cold Steel (AP-heavy, 0% Strain) vs. Inoculated (Low AP, +Strain) | Creates visceral philosophical and tactical tension between discipline and corruption. |
| **Interruption** | No formal poise system; basic status checks | **Dedicated Poise / Stagger Bar** with Windup interrupt cancel | Introduces dynamic fight tempo where heavy blows can shut down enemy execution. |

---

## 8. Summary of Engine Crates to Touch (When Implementation Begins)

```text
[ emberwake_data ]
  └── models/skill.rs        ---> Add strain_cost, speed_tier, stagger_damage, armor_piercing; remove stance.
  └── models/unit.rs         ---> Add strain_floor, graft_slots, default_poise.

[ emberwake_rules ]
  └── unit.rs                ---> Add StrainState and PoiseState structs; remove enum Stance.
  └── damage.rs              ---> Add armor_piercing and strain_mult; remove stance_factor matchup math.
  └── status.rs              ---> Add STAGGERED and ABERRANT_FRENZY status effects.

[ emberwake_battle ]
  └── resolver.rs            ---> Add impact tick checking and poise interrupt cancellation logic.
  └── command.rs             ---> Clean stance_switch_cost; bind speed_tier to block phases.
  └── session.rs             ---> Add post-round Aberration Crisis evaluation loop.
```
