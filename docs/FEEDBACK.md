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
| `U2.2` (`U2_CLASS`) | auditing | - | Read-only audit running; findings come as FB items. |
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
