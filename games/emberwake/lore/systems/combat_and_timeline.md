# Combat Engine & Timeline Execution Specification

> **Category:** Core Combat Systems & Technical Engine Architecture  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/systems/combat_and_timeline.md`  
> **Reference Specs:** [rpg_kit_and_system_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/rpg_kit_and_system_architecture.md), [rules_engine_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/rules_engine_architecture.md), [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)  
> **Target Crates:** `emberwake_battle` (`resolver`, `command`, `session`, `grid`), `emberwake_rules` (`damage`, `unit`, `status`), `emberwake_data`

---

## 1. Architectural Philosophy: The 6-Tick Simultaneous Engine

Combat in *Emberwake: The Low Sleep* departs from traditional sequential turn-based mechanics (I-Go-You-Go). Instead, all combatants—both allied operatives and hostile entities—schedule their actions across a shared, unified **6-Tick Timeline Track**.

Once both sides commit their maneuvers during the tactical planning phase, the engine unrolls the round dynamically tick-by-tick ($T_1 \to T_6$).

```text
TIMELINE TRACK (1 ROUND = 6 DISCRETE TICKS):
+-----------------+-----------------+-----------------+-----------------+-----------------+-----------------+
|     TICK 1      |     TICK 2      |     TICK 3      |     TICK 4      |     TICK 5      |     TICK 6      |
+-----------------+-----------------+-----------------+-----------------+-----------------+-----------------+
| • Reposition /  | • Fast SNAP     | • Mid Advance / | • STANDARD      | • HEAVY Impacts | • Recovery /    |
|   Early Windup  |   Impacts Land  |   Telegraphs    |   Impacts Land  | • Sunder Swings |   Active Guard  |
| • Active Guard  | • Fast Parries  | • Trajectory    | • Field Sutures | • Catastrophic  | • Strain Decay  |
|   Engages       | • Smoke Drops   |   Tracking      | • Aimed Bolts   |   Hazard Bursts | • Post-Round    |
+-----------------+-----------------+-----------------+-----------------+-----------------+-----------------+
```

### 1.1 Direct Reuse of Engine Capabilities
The timeline engine is not built from scratch; it extends existing capabilities in `games/emberwake/crates/`:
* **`emberwake_battle::resolver::Resolver`:** The existing 6-block timeline scheduling core (`total_blocks = 6`, `ScheduledAction`) is reused as the master timeline track.
* **`emberwake_battle::grid::Grid` & `path::Path`:** 2D discrete coordinate navigation and A* pathfinding are 100% reused for movement chevrons and collision checks.
* **`emberwake_rules::damage`:** Hyperbolic armor reduction ($r / (r + 50)$), weapon scaling, and critical hit formulas are reused, supplemented by binary armor piercing and strain multipliers.
* **`aegis_ui::timeline`:** Timeline canvas rendering (playhead, track rows, tick rulers) provides the foundational UI widget.

---

## 2. Maneuver Block Anatomy & Standardized Speed Tiers

Skills and attacks are not scheduled as atomic instant actions. Instead, every unit executes **Maneuver Blocks** composed of three contiguous, deterministic phases:

```text
+---------------------------------------------------------------------------------------------------+
| MANEUVER BLOCK ANATOMY                                                                            |
+-----------------------------------+-------------------------------+-------------------------------+
| PHASE 1: WINDUP                   | PHASE 2: IMPACT               | PHASE 3: RECOVERY             |
+-----------------------------------+-------------------------------+-------------------------------+
| • Weapon raise, chant, aim        | • Instantaneous resolution    | • Animation follow-through    |
| • Stagger defense halved          | • Damage & armor shred land   | • Cannot start new attacks    |
| • Vulnerable to Stagger Breaks    | • Poise damage applied        | • Unused ticks convert into   |
| • Ground / entity decals visible  | • Knockbacks & statuses land  |   Active Guard or Strain Vent |
+-----------------------------------+-------------------------------+-------------------------------+
```

### 2.1 Standardized Speed Tier Matrix
To eliminate chaotic timing permutations, all maneuvers conform strictly to three standardized speed tiers:

```text
+--------------+---------------+-------------+-----------------+------------------------------------------+
| SPEED TIER   | WINDUP WINDOW | IMPACT TICK | RECOVERY WINDOW | PRIMARY TACTICAL APPLICATION             |
+--------------+---------------+-------------+-----------------+------------------------------------------+
| Snap (Fast)  | None / T1     | **Tick 2**  | Ticks 3–4       | Precision interrupts, fast parries, smoke|
|              |               |             |                 | flasks, evasive repositioning, daggers.  |
+--------------+---------------+-------------+-----------------+------------------------------------------+
| Standard     | Ticks 1–2     | **Tick 4**  | Ticks 5–6       | Broadsword swings, polearm thrusts, aimed|
| (Mid)        |               |             |                 | arbalest bolts, field surgery sutures.   |
+--------------+---------------+-------------+-----------------+------------------------------------------+
| Heavy (Slow) | Ticks 1–4     | **Tick 5    | Tick 6 / Recoil | Greatsword cleaves, two-handed hammers,  |
|              |               | or 6**      |                 | channeled marrow blights. High damage,   |
|              |               |             |                 | colossal stagger, vulnerable to breaks.  |
+--------------+---------------+-------------+-----------------+------------------------------------------+
```

---

## 3. The Tick Execution Loop: State Machine & Lifecycle

The combat loop advances through 6 discrete tick iterations per round. The execution lifecycle of a single tick in `Resolver::step_tick()` follows a strict deterministic pipeline:

```text
TICK STEP LIFECYCLE (Executed for Tick T in 1..=6):
1. [ TICK INITIALIZATION ]
   ├── Reset transient per-tick flags on all active units.
   └── Apply persistent environmental hazard ticks (e.g., Phosphorus Fire, Caustic Tar).
         │
2. [ PHASE TRANSITIONS ]
   ├── Evaluate all scheduled actions: identify units entering Windup, Impact, or Recovery.
   └── Update Active Guard mitigation for units currently holding guard frames.
         │
3. [ MOVEMENT STEPPING ]
   ├── Units executing Move-Then-Act or Disengage advance 1 grid tile along their A* path.
   └── Check Tenacious Auto-Chase (Reach 1 attackers adjust vector toward moving targets).
         │
4. [ IMPACT RESOLUTION ]
   ├── Collect all actions with `impact_tick == T`.
   ├── Sort simultaneous actions by Initiative Tie-Breaker.
   └── Compute damage, armor mitigation, criticals, and poise damage.
         │
5. [ POISE BREAK & INTERRUPT EVALUATION ]
   ├── Apply poise damage to defenders.
   └── If a unit in WINDUP reaches Poise <= 0:
       ├── Action marked `cancelled = true`.
       ├── Remaining action ticks convert into an amber-striped `StaggerBlock`.
       ├── Status `STAGGERED` applied (-50% evasion, +25% damage taken).
       └── Floating combat text: "BREAK!"
         │
6. [ RECOVERY & CLEANUP ]
   ├── Transition units that completed Impact into Recovery frames.
   └── Check for unit fatalities and trigger casualty hooks.
```

---

## 4. Deterministic Simultaneous Resolution & Tie-Breakers

Because multiple combatants can schedule maneuvers impacting on the exact same tick (e.g. Unit A and Unit B both landing Snap attacks on Tick 2), the engine enforces strict deterministic resolution rules.

### 4.1 The Simultaneous Impact Rule
1. **Defensive Snapshot:** All attacks landing on Tick $T$ evaluate against the target's defensive state (Armor Rating, Active Guard, Cover) established at the *beginning* of Tick $T$.
2. **Mutual Trades:** If Unit A strikes Unit B on Tick 2, and Unit B strikes Unit A on Tick 2, **both attacks land**. Neither attack can pre-emptively interrupt the other, because both units completed their windup simultaneously.
3. **Mutual Fatalities:** If both strikes deal lethal damage, both combatants succumb simultaneously at the end of Tick $T$.

### 4.2 Impact vs. Windup Asymmetry (The Core Interrupt Rule)
* **The Golden Rule of Tactical Timing:** An attack impacting on Tick $T$ can interrupt an opponent only if the opponent is currently in their **Windup Phase** on Tick $T$.
* *Example:*
  - Kaelen schedules *Pommel Strike* (Snap: Windup T1, **Impact T2**).
  - Inquisitor schedules *Heavy Cleave* (Heavy: **Windup T1–T4**, Impact T5).
  - On Tick 2, Kaelen's strike lands. The Inquisitor is in his Windup Phase (T2).
  - Kaelen's +25 Stagger breaks the Inquisitor's Poise Bar.
  - The Inquisitor's *Heavy Cleave* is immediately **cancelled**. His T3–T6 ticks shatter into a Stagger Block. He never reaches Tick 5!

### 4.3 Initiative Tie-Breaker Formula
When execution order dictates logging or non-simultaneous environmental triggers, priority is decided deterministically:

$$\text{Initiative} = (\text{DEX} \times 2) + \text{PERC} + \text{Speed Tier Weight} + \text{RngRoll}$$

Where `Speed Tier Weight` is:
* **Snap:** $+20$
* **Standard:** $+10$
* **Heavy:** $+0$

If an exact tie persists, the tie breaks deterministically using the match's seeded `GodotRng` based on `UnitId`.

---

## 5. The Stagger Bar, Poise Breaks & Interruption Engine

Every active combatant possesses a real-time **Poise Bar** derived from physical attributes and equipped gear.

```text
[ POISE BAR COMPOSITION ]
Max Poise = (Constitution x 2) + Armor Poise Rating + Discipline Passives
Current Poise: Ranges from Max Poise down to 0.
```

### 5.1 Poise Damage Calculation
When damage is dealt, the attacker's `stagger_damage` reduces the defender's current poise:

$$\text{Applied Stagger} = \text{Base Stagger} \times \text{Defensive Modifier} \times \text{Vulnerability Modifier}$$

Where:
* **Active Guard Modifier:** If the defender is executing **Active Guard** on that tick, incoming stagger is **halved ($0.5\times$)**.
* **Windup Vulnerability:** If the defender is struck during an active **Windup phase**, incoming stagger is **doubled ($2.0\times$)**.
* **Flesh-Tension Vulnerability:** If the defender is in the *Flesh-Tension* Strain bracket (50%–74%), incoming stagger is **doubled ($2.0\times$)**.
* *Stacking:* Stagger multipliers stack multiplicatively (e.g. struck during Windup while in Flesh-Tension = $4.0\times$ stagger).

### 5.2 The Poise Break State Machine
When `poise.current <= 0`:
1. `poise.is_broken` is set to `true`, and current poise clamps to 0.
2. If the unit was in **Windup**:
   - `action.cancelled` becomes `true`.
   - The action's remaining timeline blocks convert into an amber-striped `StaggerBlock`.
   - The unit loses all active evasion (drops to 0%) and suffers $+25\%$ increased damage from all subsequent hits during the current round.
3. At the end of the round, `poise.current` resets to `max_poise`, and `is_broken` clears to `false`.

---

## 6. Spatial Movement, Collision & Auto-Chase Mechanics

Movement is integrated into maneuvers via `emberwake_battle::grid::Grid` and `path::Path`:

### 6.1 Composite Maneuvers
* **Advance (Move-Then-Act):** Unit spends Ticks 1–2 moving 1–2 tiles along an A* path toward the target, then transitions directly into Windup on Tick 3 and Impact on Tick 4.
* **Disengage (Act-Then-Move):** Unit executes a Snap strike on Tick 2, then uses Ticks 3–4 to retreat 1–2 tiles into cover or behind an ally.

### 6.2 Auto-Chase Rules (Temporal Engagement)
If a targeted unit moves away during an attacker's windup, the engine resolves the spatial discrepancy using weapon reach rules:

```text
+-----------------------+-----------------------+-------------------------------------------------------+
| WEAPON REACH TYPE     | ENGAGEMENT RULE       | RESOLUTION MECHANIC                                   |
+-----------------------+-----------------------+-------------------------------------------------------+
| Reach 1 (Daggers,     | Tenacious Auto-Chase  | Attacker automatically steps forward 1 tile during    |
| Knuckles, Beasts)     |                       | windup to maintain contact. Fails if blocked by wall. |
+-----------------------+-----------------------+-------------------------------------------------------+
| Reach 2 (Spears,      | Range Hold            | Attacker holds ground. If target retreats 1 tile but  |
| Halberds, Billhooks)  |                       | remains in Reach 2, the strike lands without moving.  |
+-----------------------+-----------------------+-------------------------------------------------------+
| Ranged (Arbalests,    | Arc Hold              | Attacker rotates facing arc toward target; fails only |
| Thrown Flasks)        |                       | if target breaks Line of Sight behind full cover.     |
+-----------------------+-----------------------+-------------------------------------------------------+
```

### 6.3 Spatial Demolition & Wall Slams
* If a maneuver applies forced knockback (e.g. *Jaw-Breaker*, *Foundry Shatter*, *Seventh Ward Bulwark*), and the target's trajectory intersects an impassable obstacle, solid wall, or another unit:
  - Movement halts immediately at the obstacle.
  - The remaining unspent knockback tiles convert into **Wall Slam Impact**: deals **+15 blunt physical damage per unspent tile** and **+20 Stagger**.
  - If knocked into another combatant, both units take the collision damage.

---

## 7. Real-Time Strain Surfing & The Aberration Crisis Engine

The corruption system provides dynamic in-combat risk/reward mechanics governed by real-time Strain accumulation:

```text
[ THE COMBAT STRAIN SPECTRUM ]
0% -------- 25% ---------------- 50% ---------------- 75% ---------------- 100%+
[ Quiescence ]  [ Resonance ]   [ Flesh-Tension ]   [ Aberrant Edge ]   [ CRISIS ]
Standard AP     +15% Crit Rate  +25% Damage         +50% Damage         d100 Roll
0% Strain Recoil+1 Move Tile    +2 Armor Rating     100% Armor Pierce   Spasm / Collapse
                -25% Heal Pots  Double Stagger In   10% HP Self-Recoil  Metamorphosis
```

### 7.1 Instantaneous Strain Allocation
* Inoculated maneuvers immediately apply their `strain_cost` to the unit's gauge upon action initiation (Tick 1).
* As a result, an operative can intentionally initiate an Inoculated skill on Tick 1 to push their gauge from 45% into *Flesh-Tension* (50%+), boosting the damage of their strike when it resolves on Tick 4.

### 7.2 The Aberration Crisis Engine (100%+ Strain)
If an operative ends a combat round at **$\ge 100\%$ Strain**, their biological coherence ruptures. The engine executes an immediate **Aberration Crisis Roll**:

$$\text{Crisis Score} = D100 + \text{Dormant Strain Floor} - \text{Composure}$$

```rust
// Mapped in emberwake_battle::resolver:
pub enum AberrationCrisisResult {
    /// 01–30: The hero grits their teeth and maintains control. Strain vents to 70%.
    FleshMastery,
    /// 31–55: Marrow spurs burst outward, dealing 18 true damage to all adjacent units (allies & foes).
    MarrowRupture,
    /// 56–75: Mind blurs into predatory mania; unit attacks the nearest combatant next round.
    PredatoryFrenzy,
    /// 76–90: Rapid ossification; gains +4 Armor but permanently advances +1 Calcification Stage!
    SpontaneousCalcification,
    /// 91–100+: Complete terminal collapse; operative transforms into a hostile Aberration!
    IrreversibleMetamorphosis,
}
```

---

## 8. Rust Engine Structs & Concrete Code Mappings

To implement this specification within `games/emberwake/crates/`, the data structures map directly into the existing modules:

### 8.1 Extensions to `emberwake_rules::unit`
```rust
// In emberwake_rules::unit:

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StrainState {
    pub current: i32,           // 0 to 100+
    pub dormant_floor: i32,     // 0 to 50 established by grafts
    pub calcification_stage: u8,// 0 to 5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoiseState {
    pub current: i32,
    pub max_poise: i32,
    pub is_broken: bool,
}

// In Unit struct:
// pub stance: Stance,       <-- REMOVED per DEC-LORE-11
pub strain: StrainState,    // <-- NEW
pub poise: PoiseState,      // <-- NEW
```

### 8.2 Extensions to `emberwake_data::models::skill`
```rust
// In emberwake_data::models::skill:

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeedTier {
    Snap,      // Impact T2
    Standard,  // Impact T4
    Heavy,     // Impact T5 or T6
}

pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub ap_cost: i32,
    pub speed_tier: SpeedTier,
    pub windup_ticks: Vec<i32>,
    pub impact_tick: i32,
    pub recovery_ticks: Vec<i32>,
    pub strain_cost: i32,
    pub stagger_damage: i32,
    pub armor_piercing: bool,
    pub recoil_pct: f64,
    // ... existing geometry & reach fields ...
}
```

### 8.3 Extensions to `emberwake_battle::resolver`
```rust
// In emberwake_battle::resolver:

#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub unit: UnitId,
    pub command: Command,
    pub speed_tier: SpeedTier,
    pub windup_ticks: Vec<i32>,
    pub impact_tick: i32,
    pub recovery_ticks: Vec<i32>,
    pub cancelled: bool,
    pub converted_to_stagger: bool,
}

impl Resolver {
    /// Steps the timeline simulation forward by 1 discrete tick.
    pub fn step_tick(&mut self, w: &mut World, tick: i32) {
        // 1. Process environmental hazards
        self.apply_tick_hazards(w, tick);

        // 2. Resolve impacts landing on this tick
        let mut impacts: Vec<usize> = self.scheduled.iter().enumerate()
            .filter(|(_, a)| !a.cancelled && a.impact_tick == tick)
            .map(|(i, _)| i)
            .collect();

        // Sort by initiative tie-breaker
        impacts.sort_by_key(|&i| -self.scheduled[i].command.initiative(w));

        for idx in impacts {
            self.execute_impact(w, idx, tick);
        }

        // 3. Update recovery and guard frames
        self.update_active_guards(w, tick);
    }
}
```
