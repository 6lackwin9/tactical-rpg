# Rules engine architecture: building the timeline ruleset

Status: canon for what exists today and for the Act 1 build (ruleset option "timeline", poise, recovery blocks,
`ap_cost`); proposal for the Strain, crisis and Calcification parts (Act 2 on). This is design input, not a task spec:
the field names below are proposals for the spec that a design task writes before any code.

Rules and numbers: [combat and the timeline](combat_and_timeline.md) and [CANON](../CANON.md) section 2. Today's
rules: the game's [GAME_BRIEF](../../GAME_BRIEF.md). Where code goes: [STRUCTURE](../../../../../docs/STRUCTURE.md).
Layer tags as in the combat doc: `[engine]` technology any game needs, `[kit hook]` a generic shape or ruleset option ("(exists)" when
today's crates carry it), `[data]`
content, `[game code]` story-only mechanics.

## 1. What exists today (verified 2026-10-25)

| Piece | Real name | Notes |
| --- | --- | --- |
| Unit state | `emberwake_rules::unit::Unit` | `hp`, `max_hp`, `speed`, `ap`, `max_ap`, `stance`, `guard`, `stats`, `inventory`, `ap_regen_bonus`; `BASE_AP` = 10 |
| Stats | `emberwake_rules::stats` | STR DEX INT CONS PERC KNOW; `MAX_CLASS_SLOTS` 5, `MAX_EXTRA_SLOTS` 3 |
| Damage | `emberwake_rules::damage::compute` | the only formula; `ARMOR_K` 50 (rating / (rating + 50)), `GUARD_CAP` 0.9, `CRIT_MULT` 1.5, stance matchup factor |
| Statuses | `emberwake_rules::status` | fifteen rows incl. `stagger`, `stun`, `expose` (flat armour minus), `fortify`, bleed, burn, poison; AP debt 3 = stagger, 5 = stun |
| Round timeline | `emberwake_battle::resolver::Resolver` | `schedule`, `prepare_block`, `execute_single`, `apply_round_end_effects`, `finish_round`; `ScheduledAction { unit, command, start, end, cancelled }`; `total_blocks` 6 |
| Commands | `emberwake_battle::command::Command` | `ap_cost`, `priority`, initiative = priority x 1000 + speed; attack `dash_tiles` and `follow_tiles` (chase); `disrupt_break` (stance break, classic) |
| Plans, AI, tells | `emberwake_battle::{plan, ai, intent}` | enemy plans drafted first and shown as tells (INTENT1) |
| Encounter rules | `emberwake_battle::{encounter, session}` | objectives (rout, hold, escape, slay, protect, guests), boss phases, reinforcement waves, morale; per-encounter `ap_regen` (default 4) |
| Skill data | `emberwake_data::SkillDef` | `kind`, `level`, `blocks`, `family`, `tier`, `weapon_types`, `min_proficiency`, reach, `area`, `power_mult`, status fields, `mitigation`, `jump_tiles`, `disrupt`, `exploit` |
| Timeline widget | `aegis_ui::timeline` | the block timeline widget (TIMELINE1); the battle strip sits at the top (COMBAT-UI3 layout A) |

Names in older lore drafts that do not exist: `Resolver::step_tick`, `emberwake_data::models::*` (there is no
`models` module; `SkillDef` sits in the crate root), `SpeedTier`, `StrainState`, `PoiseState`, `engine/combat`,
`windup_ticks`, `impact_tick`, `recovery_ticks`. Today's combat is not sequential initiative: 6 blocks per round
resolved simultaneously, 10 AP +4 per round, actions pay AP at start and land on their last block.

## 2. The ruleset option `[kit hook]`

One switch per world picks **classic** or **timeline**: a world field with a Dojo toggle (user 2026-10-25, Q-52; the
field name is the spec's). Reuse rule: the Ashlands are the baseline; a rule incompatible with classic is an option, a
compatible one an extension that can be disabled; bug fixes go to the core and may need an Ashlands or kit revisit. It
gates:

| Part | Classic | Timeline |
| --- | --- | --- |
| Stances, stance switch cost, matchup factor, `disrupt_break` | on | off (matchup factor 1.0) |
| Poise bar and breaks | off | on |
| Recovery blocks and chained follow-ups | ignored | honoured |
| Active Guard base move | - (the `guard` basic) | on |
| Strain meter, crisis, pierce | off | on per world act (Act 2 on) |

Classic must replay the parity suites unchanged; every timeline addition reads the switch first.

## 3. Proposed data fields (for the spec)

### 3.1 Skills `[data]` on new `[kit hook]` fields

| Field | Values, default | Meaning |
| --- | --- | --- |
| `blocks` (exists) | 1-6 | windup length; the impact lands on the last block. Tier = Snap 1, Standard 2, Heavy 3-4 (derived, no extra field) |
| `ap_cost` | int >= 0, default = `level` | AP paid at start; decoupled from tier and level |
| `recovery_blocks` | 0-2, default 0 | committed blocks after the impact |
| `stagger` | int >= 0, default 0 | poise damage on impact (before multipliers) |
| `armor_piercing` | bool, default false | ignore the target's armour rating |
| `strain_cost` | 0-35, default 0 | Strain added at start (Act 2 on) |
| `forced_move` | push or pull, tiles | knockback and pull (new kit hook) |

Also for the spec (user 2026-10-25, no field names yet): which skills a follow-up can chain from (Q-54); the stagger
intent's break chance and the cost of a guaranteed break, and an unbreakable flag or extra poise for heavy enemy
windups (Q-50).

Retired from the old drafts: a `speed_tier` enum and tick vectors (derived from `blocks`), `recoil_pct` (recoil is the
Aberrant Edge bracket's effect, not a skill field), `graft_requirement` (grafts teach skills through `learn_skill`, see
[skill acquisition](skill_acquisition_and_loadout_systems.md)).

### 3.2 Unit state

| State | Shape | Layer |
| --- | --- | --- |
| Poise | current, max (CONS x 2 + armour poise), breaks this round and last round | `[kit hook]` |
| Strain | a per-unit meter 0-100 with a floor (0-50) and bracket thresholds | `[kit hook]` meter, `[data]` brackets |
| Crisis | the 1d3 table at round end | `[game code]` unless the meter's triggers can express it |
| Calcification | a per-unit stage track 0-5, out of combat, saved | `[kit hook]` track, `[data]` stage effects |
| Composure | a derived stat, the corruption resist, computed by its own formula from base stats plus gear, status and Calcification modifiers; read by every corruption effect (CANON 2.1, Q-47a; spec `docs/specs/DERIVE1.md`) | `[engine]` formula evaluator, `[kit hook]` derived stats, `[data]` the row |

The `stance` field stays (classic uses it); the timeline ruleset holds every unit in the neutral stance.

### 3.3 The damage formula

`damage::compute` keeps its shape. Timeline adds two terms and neutralises one:

- strain factor: x1.25 in Flesh-Tension, x1.5 in Aberrant Edge, else x1 (Act 2 on);
- armour term skipped when the hit has `armor_piercing`;
- the stance matchup factor is 1.0 under timeline.

Crit (+15% in Acute Resonance), the guard term (Active Guard 0.5, capped at `GUARD_CAP` 0.9 with cover) and the
hyperbolic armour curve are reused as they are.

## 4. Resolver changes (timeline only)

1. **Recovery**: `schedule` books `blocks + recovery_blocks`; the impact lands at the end of the windup, not at
   `end`. Today an action lands at `end == block`, so the landing block becomes start + blocks - 1. A chained
   follow-up (Q-54) may start in those blocks instead of the recovery.
2. **Poise**: each impact applies stagger x multipliers (windup x2, Flesh-Tension x2, Active Guard x0.5; they
   multiply), doubled again for a repeated break. At 0 the unit breaks: every scheduled action of that unit not yet
   landed this round is set `cancelled` (the field exists; today only downed units are cancelled). The stagger intent
   (Q-50) adds an optional seeded break chance and a guaranteed break at a cost.
3. **Same block**: unchanged; initiative order, and a broken or downed unit does not land later in the block.
4. **Round end**: poise refills; Strain decays (-10 without an Inoculated action, -10 more for Active Guard, floor);
   then the crisis roll for any unit at 100.
5. **Strain gain** at start, beside the AP payment in the resolver's start step.

## 5. Settled options (formerly open in the drafts)

| Question | Settled | Source |
| --- | --- | --- |
| Strain timing: on action start or batched at round end | on start (brackets change mid-round) | DEC-LORE-12, CANON 2.2 |
| Inoculated armour bypass: binary or 50% penetration | binary pierce | DEC-LORE-12, CANON 2.1 |
| Impact timing: tier anchors (T2/T3/T4) or per-skill ticks | neither: relative windups from `blocks`, optional recovery | CANON 2.1 |
| Crisis: d100 with Composure or 1d3 | 1d3 at round end (Composure stays, as a derived stat) | CANON 2.2, 2.1 |
| Where the ruleset switch lives | a world field with a Dojo toggle | Q-52 |
| Stagger: keep the lore values or rescale | rescale; heavy enemy windups get more poise or an unbreakable flag; optional break chance, guaranteed break at a cost | Q-50 |
| Recovery default | 0, set per skill; chained follow-ups use the blocks an action leaves | Q-54 |
| Same-block strikes: mutual trades or initiative | engine initiative, no mutual trades | CANON 2.1 |
| Stances | off in timeline, on in classic | DEC-LORE-11, ruleset option |

## 6. Kept, replaced, added

| Subsystem | Classic | Timeline | Why |
| --- | --- | --- | --- |
| Stances | 4 stances, x1.25 / x0.75 matchup, +1 AP switch | off | defence is an action (Active Guard), vulnerability is the windup |
| Landing | end of the action's blocks | same, then optional recovery | telegraphs come from windup length |
| Armour | rating / (rating + 50) | same, plus binary pierce | Cold Steel sunders (expose), Inoculation bypasses |
| AP | 10, +4 per round, cost = skill level | same pool, `ap_cost` per skill | Cold Steel 2-4, Inoculated 1-2 plus Strain |
| Interrupts | none; statuses only | poise bar and breaks | heavy windups can be shut down |

## 7. Crates to touch (when a task is approved)

| Crate | Change |
| --- | --- |
| `emberwake_data` | `SkillDef` fields of section 3.1; schema and validation |
| `emberwake_rules` | poise and Strain state on `Unit`; strain factor and pierce in `damage::compute`; matchup off under timeline |
| `emberwake_battle` | the ruleset switch; recovery in `Resolver::schedule`; poise and breaks at impact; round-end decay and crisis; Active Guard; forced movement |
| `emberwake_world`, save | Calcification stage and Strain floor on the character (save migration) |
| `aegis_ui::timeline`, `emberwake_gui` | impact badges, windup and recovery shading, BREAK marker, poise and Strain on cards |
| `emberwake_sim` | a timeline-ruleset mode so the balance lane can measure it |

## 8. Gaps

| Gap | Options | Recommendation |
| --- | --- | --- |
| Where the ruleset switch lives | A: a world field inherited by its encounters; B: the game manifest | Decided (2026-10-25, user, Q-52): A, with a Dojo toggle |
| The skill field names above | A: as proposed; B: the spec renames | A unless the spec finds clashes |
| AI under timeline: enemies plan first and never re-plan, so they cannot dodge a visible interrupt | A: give heavy enemy windups higher poise or an unbreakable flag; B: let enemies re-plan | Decided (2026-10-25, user, Q-50 a): A (no change to the INTENT1 contract), with the stagger intent above |
| The Strain meter as kit or game code | A: a generic per-unit meter with thresholds, triggers and decay (kit); B: game code | A; the crisis table stays game code if the triggers cannot roll a table |
