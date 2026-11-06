# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates `## 1. Build Status` and appends items to `## 2. Feedback Items` (feasibility findings, cost surprises, spec conflicts, sim results, and docs deltas for `README.md`, `STRUCTURE.md`, `GAME_BRIEF.md`, `HISTORY.md`).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| `ATTACKLOCK1` (slice 1) | built (off) | `develop` | Waiting on `ATTACKLOCK1-O` design sign-off. |
| `SIMCOUNT1` | done | `develop` | `sim feel counts=on` landed. |
| `VISCLAR1-D` | built | `develop` | Battle view parts 1 and 2 landed; waiting on user look to close M1. |
| `TESTCLEAN1` | building | `claude/testclean1`, lane 1 | One scratch helper removing its folder on drop; `aegis_ui` first. |
| `SKILLS1` | staged | `staging` | Review passed; reaches `develop` with the next full gate. |
| `KITIDS1` | building | `claude/kitids1`, lane 2 | |
| `BASERULES1-M` | queued | - | Starts when a lane frees; see FB-01. |
| `U1.2` (`U1_STATS`) | audited | - | Feasible as six slices, about 2S + 3M + 1L; FB-04 to FB-09. |
| `U2.2` (`U2_CLASS`) | audited | - | Feasible as six slices, about 1S + 4M + 1L; FB-10 to FB-15. |
| `A4.2` (`INTENT2`) | auditing | - | Read-only audit running; findings come as FB items. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

### FB-01: BASERULES1-M points at the wrong sections

- **Spec**: `docs/READY.md` section 1 row 4; `docs/specs/BASERULES1.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: The row says to build from `BASERULES1.md` "sections 1-2 only". Those sections describe today's rules and their weaknesses. The register metadata, `check/ruleset.rs`, bundle tests and `aegis rules` with `LOCKED` and `--contract` are specified in `BASERULES1.md` section 6 ("How the engine enforces it") and its section 7 row BASERULES1-M, and in `BASERULES1_LIBRARY.md` (the option table, bundles, and section 7's authoring note).
- **Recommendation**: Point the row at `BASERULES1.md` sections 6-7 and `BASERULES1_LIBRARY.md`. Until answered, the build follows the row's own summary and acceptance test, which match section 6.

### FB-02: Planner skill still sends the build session to shared docs

- **Spec**: `.claude/skills/planner/SKILL.md`; `CLAUDE.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: CLAUDE.md says the build session writes only `docs/FEEDBACK.md` (plus crate-local READMEs). The planner skill's step 3.3 still says to update `docs/AGENT_MAP.md` and append to the agent run log in `docs/WORKFLOW.md`; the `task-landing` skill and `docs_needed.py` ask the lander to update README, STRUCTURE, GAME_BRIEF and HISTORY. The build session will follow CLAUDE.md: agent runs and docs deltas go here.
- **Recommendation**: Drop or reword those steps in `planner` and `task-landing` for the build session, or name which shared files, if any, it may write.

### FB-03: The gate checks every doc, so a spec can block a build push

- **Spec**: `CLAUDE.md` standing rules; `tools/fc.sh docs`
- **Kind**: feasibility
- **Status**: open
- **Finding**: The full gate (`check_project.ps1`, and `flush.sh`) runs `tools/fc.sh docs --all`. A spec or shared doc that fails the lint blocks the build session's push to `develop`, and under the split the build session may not fix it.
- **Recommendation**: The design session runs `tools/fc.sh docs` before each commit; if a doc still breaks the gate, the build session files a feedback item and waits, rather than editing.

### FB-04: U1 is three builds, and the pool layer should wrap the typed fields

- **Spec**: `docs/sessions/U1_STATS.md`
- **Kind**: cost
- **Status**: open
- **Finding**: Derived values live in three places today: `rpgkit_rules/src/derived.rs` (a rating evaluator: formula, constants, min, max, status modifiers; no bands or current value), hard-coded methods in `stats.rs:258-282` (HP, initiative speed, armour rating, crit, resist), and unit meters in `rpgkit_battle/src/meter.rs` (bands, decay, battle-only). Poise is a fourth (`poise.rs`, max from a derived row, refill hard-coded); AP is `Unit.ap` and `max_ap`, read in about 20 files including the AI and planner; HP is `unit.hp`. So U1 is three builds: the derived class and pools (L, about 8-12 files), the damage pipeline (M), and the stat re-wiring (M). Saves are unaffected while pools are battle-only, which "full each fight" implies.
- **Recommendation**: State that HP, AP and poise keep their typed fields, with the pool layer as a facade over them; only new pools use the generic store. Moving them into a generic map touches every AI and planner file for no gameplay gain.

### FB-05: The pipeline has a third rounding path, and some factors do not fit a stage

- **Spec**: `docs/sessions/U1_STATS.md`, modular damage
- **Kind**: conflict
- **Status**: open
- **Finding**: Besides the two rounding points (attack power, final hit; `damage.rs:130`, `:221`), `unit.rs:1091` `take_damage` rounds flat hazard damage, arena pushes and trigger moves separately, applying guard only. Also: `power_scale` (tier) is a multiplier inside the base stage, against "flat modifiers only in base"; authored `power` and fixed-stat units return early and bypass everything; the ally intercept (`command.rs:775-830`) calls the formula twice and splits damage with another rounding. Poise stagger (`poise.rs:164`) is purely multiplicative with one rounding, a floor of 0 and no armour or crit, so it fits only if the stage names are generic. An exact port is possible if multiplication keeps today's f64 order; `rules_golden` will catch any off-by-one at a half.
- **Recommendation**: Decide whether flat hazard damage and intercepts run the pipeline; allow multipliers in the base stage (tier) while flat adds stay base-only; keep the fixed-stat and authored-power overrides; make the first build slice an exact port with every golden byte-identical before any new modifier.

### FB-06: There is no guard status today

- **Spec**: `docs/sessions/U1_STATS.md` (two mitigation layers); decision "What Stand is"
- **Kind**: conflict
- **Status**: open
- **Finding**: Guard is a bare `Unit.guard` number set by Defend commands (`command.rs:585`), the resolver and triggers, cleared on a poise break and at round end, and read in eight or more places (log, portrait, session snapshot, poise, AI, planner, metrics). Cover comes from terrain. The two-layer shape already exists (guard plus cover capped at 0.9, armour with K 50); fortify and expose are statuses inside the armour rating.
- **Recommendation**: Keep `Unit.guard` as the situational layer's accumulator, with new sources feeding it, rather than turning guard into a status; or, if it must be a status, size that as its own slice. Treat the cap move from 0.9 to about 0.75 as a balance change with a named re-record, separate from the refactor.

### FB-07: The intended changes move all three goldens; the speed split reaches the world crate

- **Spec**: `docs/sessions/U1_STATS.md`, stats to values
- **Kind**: cost
- **Status**: open
- **Finding**: Crit from PERC (`stats.rs:276`, `damage.rs:10-13`), potency from KNOW against resist (`status.rs:210`, chance = potency - resist; one changed chance shifts later draws on the RNG stream), and the cap move each re-record `rules_golden`, the timeline golden and `sim_golden`. `Unit.speed` is both initiative and move rate and is read in `plan.rs`, `command.rs`, `unit.rs`, `rpgkit_data` unit rows (fixed-stat rows carry `speed`), the host's UI JSON, the sim print in `sim_golden`, and `rpgkit_world/src/battle.rs:151`, which writes it. Burden (`stats.rs:263`) and carry overload (`carry.rs:79`) both lower speed today.
- **Recommendation**: Say which of initiative or move rate burden and carry lower, and what a fixed-stat row's `speed` means after the split. Build each intended change as its own commit so each re-record is attributable, with a HISTORY row each.

### FB-08: Composure already exists, and there is no corruption pool

- **Spec**: `docs/sessions/U1_STATS.md`, composure acceptance test
- **Kind**: conflict
- **Status**: open
- **Finding**: Composure is already a rating row in `derived_stats.json` (`KNOW x 2 + CONS`, 0 to 100, shown on the party, dojo and battle screens). No corruption pool exists in code or data. For composure to run as data alone, these are missing today: a pool mode with a current value and a per-fight refill; a formula max (unit meters have fixed min, max and start); bands in percent of max (bands use absolute values); a distance filter on the `ally_downed` trigger ("within 2"). The pool-change effect exists as `meter` and `meter_set`, with its delta clamped to -10..10 (`meter.rs:285`).
- **Recommendation**: Drop the corruption clause from the acceptance test or give it a task; state composure's max (the formula, or 100); add percent bands and the distance filter to U1's build scope, as the acceptance test depends on them. Is the -10..10 delta clamp the intended limit for pool changes?

### FB-09: Tier as a formula input needs a number

- **Spec**: `docs/sessions/U2_CLASS.md`, tiers; `U1_STATS.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: Formula inputs resolve in `derived.rs` (`Eval::input`, stats, level, gear mitigation, row constants, other derived rows). Adding `tier` is one branch, size S, but tier is a string (`unit.rs:174`: hero, minion, standard, elite, boss), so it needs a numeric mapping, plus the content check's allowlist.
- **Recommendation**: Name tier's numeric form: a data table of factors per tier, read by formulas as `tier_hp`, `tier_power` and so on, or an ordinal.

### FB-10: One share vector cannot reproduce today's classes; two can

- **Spec**: `docs/sessions/U2_CLASS.md`, C1 and C2
- **Kind**: conflict
- **Status**: open
- **Finding**: Checked by script over all 7 classes at levels 1-10. Base shares (base / 62) reproduce every class at level 1. One share vector for the whole curve (budget 62 + 5 per level) differs from today's stats at levels 2-10 for all 7 classes; for example a level 2 warrior gets KNOW 11 and DEX 8 where today it has 10 and 7, as its growth puts nothing in KNOW or DEX. Base shares plus growth shares (growth / 5), with largest-remainder rounding on the running total, reproduce every class at every level exactly, so `rules_golden` holds.
- **Recommendation**: Two vectors per class: identity at level 1, and growth identity. A single vector changes 5 of 7 class curves visibly and moves every class-level golden row. Ties in largest-remainder break by largest fraction, then stat order, as `auto_spend` does.

### FB-11: Tiers scale only HP and power today, and budget scaling cannot replace power_scale

- **Spec**: `docs/sessions/U2_CLASS.md`, C7 tiers
- **Kind**: cost
- **Status**: open
- **Finding**: `stats::tier_scales` (`stats.rs:35`) gives HP, power and an AI skill cap: minion 0.4 / 0.6 / 2, standard 0.7 / 0.8 / 0, hero 1 / 1 / 0, elite 1.25 / 1.0 / 0, boss 1.5 / 1.1 / 0. HP multiplies max HP (`unit.rs:407`), power multiplies attack power (`damage.rs:115`). Scaling the budget instead moves every stat together (HP, damage, armour, crit, initiative, resist), and boss power 1.1 or elite HP 1.25 have no budget equivalent. A minion budget of about 25 breaks the "every stat 6 to 18" rule. "Tier" also means the AI skill cap, XP `kill_weight`, morale, portrait tier, gear tiers and skill ranks. This is the costliest U2 slice (L): it moves `rules_golden` (28 class-by-tier rows and the non-hero units), the timeline, sim, PHASEFOLD and `checks_golden` HP lines.
- **Recommendation**: Name what replaces `power_scale` (a tier modifier through the pipeline is the natural fit with U1); apply the 6-18 check to the class row, not the tier-scaled unit; keep the AI cap; rename the other meanings of "tier" in code; build this slice after U1's derived formulas exist, so the HP base's tier term goes in once.

### FB-12: Static templates exist already as derived units; the builders are what fail the acceptance test

- **Spec**: `docs/sessions/U2_CLASS.md`, unit types and acceptance test
- **Kind**: feasibility
- **Status**: open
- **Finding**: All 47 units except `training_dummy` are already class, level and tier derived, and 38 world characters point at them through `base_unit`, so they read as static templates at a fixed level; no unit row needs rewriting. "Template plus overrides" also exists: `base_unit`, the character's `tuning` (5 keys, used by 1 of 38), `named_items` and passives. What fails "any unit fights with no unit-specific code" today: heroes build through `progression::build_as`, NPCs through `build_unit_in` plus `customize` plus `apply_character`; `apply_character` patches HP, power, armour, speed and AP after the build; the AI profile comes from the base row for heroes and from the entry for NPCs; the sim and Dojo call `build_unit` directly. Only `training_dummy` uses the classless fixed path, with a fallback to warrior stats (KITIDS1 removes the fallback).
- **Recommendation**: Define a static unit as derived with a fixed level and budget, reuse `base_unit`, `tuning` and named items as the overrides rather than a second override system, retire or migrate the fixed-stat mode, and route every builder through one function.

### FB-13: World stage needs a source in worlds with no campaign, and XP must follow it

- **Spec**: `docs/sessions/U2_CLASS.md`, mob growth by world stage
- **Kind**: feasibility
- **Status**: open
- **Finding**: Nothing passes world state into enemy creation today. The smallest seam is a `stage` on `WorldRoster`, computed by the battle screen from game state (act, or flags), applied when the roster builds each entry; `build_unit_in` needs no new parameter, and the host and sim pass 0. But `roster()` computes a foe's level and XP weight without game state, so XP would disagree with the level fought unless the stage reaches it too. A world with no campaign has act 0. No save change, as the stage is derived.
- **Recommendation**: Say which flag or act defines the stage where there is no campaign, and that the XP weight follows the stage. Pass only the stage now, leaving WORLDFIGHT1's `begin_world` held.

### FB-14: Proficiency levels exist but mean something else today

- **Spec**: `docs/sessions/U2_CLASS.md`, C3
- **Kind**: conflict
- **Status**: open
- **Finding**: Level 0 means absent, and the class check requires 1-3. The requirement discount is level x 2, so level 1 already lowers it by 2 and level 2 by 4. Crit and burden bonuses start at level 2, and a block-cost reduction at level 3 is hard-coded. Scaling exists per class and gear (`ProfEntry.scaling`, 1.0-2.0, default 1.5) as the unit-power multiplier (`damage.rs:91-112`), so "scaling as a class weight in the weapon power rating" depends on U1's weapon power rating.
- **Recommendation**: Say which of today's level 2 and 3 bonuses (crit, burden, block cost) stay and which become the level 3 effect; whether level 1 keeps a requirement discount. Build this slice after U3 and U4, as it is the only one that changes hero combat numbers.

### FB-15: Smaller U2 points

- **Spec**: `docs/sessions/U2_CLASS.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: (a) Loadout templates save little today: 34 of 48 unit rows carry gear, with 32 distinct loadouts; they mostly help future mob families. (b) `rpgkit_rules/src/budget.rs` is the gear budget (target 23), not the unit budget. (c) The party screen (`party.rs:1003`, `:1575`), the Forge (`forge.rs:180`) and the sim's `primary_stat` read `base_stats` and `growth` directly. (d) Saves store `spend`, XP and `base_unit`, never stats, so they are safe if the free pool is unchanged. (e) The frozen kit the goldens load (`timeline_kit.json`) still has `base_stats` and `growth`.
- **Recommendation**: Loadout templates as an optional last piece; name the new module or term "unit budget"; keep `base_stats` and `growth` as a compatibility view (or a loader for the frozen kit), so readers and goldens hold while the rows move to shares.
