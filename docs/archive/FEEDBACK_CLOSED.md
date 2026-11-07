# Closed Feedback Archive (Build Session writes, Design Session reads)

This file is owned exclusively by the **Build Session** alongside `docs/FEEDBACK.md`.

When `docs/FEEDBACK.md` approaches the 300-line / 25 KB doc limit, the Build Session moves `answered` or `closed` feedback items (`FB-nn`) and superseded build logs from `docs/FEEDBACK.md` into this archive file. `tools/fc.sh docs` skips `docs/archive/`.

## FB-01 to FB-22 (moved 2026-11-07)

### FB-01: BASERULES1-M points at the wrong sections

- **Spec**: `docs/READY.md` section 1 row 4; `docs/specs/BASERULES1.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: The row says to build from `BASERULES1.md` "sections 1-2 only". Those sections describe today's rules and their weaknesses. The register metadata, `check/ruleset.rs`, bundle tests and `aegis rules` with `LOCKED` and `--contract` are specified in `BASERULES1.md` section 6 ("How the engine enforces it") and its section 7 row BASERULES1-M, and in `BASERULES1_LIBRARY.md` (the option table, bundles, and section 7's authoring note).
- **Recommendation**: Point the row at `BASERULES1.md` sections 6-7 and `BASERULES1_LIBRARY.md`. Until answered, the build follows the row's own summary and acceptance test, which match section 6.

### FB-02: Planner skill still sends the build session to shared docs

- **Spec**: `.claude/skills/planner/SKILL.md`; `CLAUDE.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: CLAUDE.md says the build session writes only `docs/FEEDBACK.md` (plus crate-local READMEs). The planner skill's step 3.3 still says to update `docs/AGENT_MAP.md` and append to the agent run log in `docs/WORKFLOW.md`; the `task-landing` skill and `docs_needed.py` ask the lander to update README, STRUCTURE, GAME_BRIEF and HISTORY. The build session will follow CLAUDE.md: agent runs and docs deltas go here.
- **Recommendation**: Drop or reword those steps in `planner` and `task-landing` for the build session, or name which shared files, if any, it may write.

### FB-03: The gate checks every doc, so a spec can block a build push

- **Spec**: `CLAUDE.md` standing rules; `tools/fc.sh docs`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: The full gate (`check_project.ps1`, and `flush.sh`) runs `tools/fc.sh docs --all`. A spec or shared doc that fails the lint blocks the build session's push to `develop`, and under the split the build session may not fix it.
- **Recommendation**: The design session runs `tools/fc.sh docs` before each commit; if a doc still breaks the gate, the build session files a feedback item and waits, rather than editing.

### FB-04: U1 is three builds, and the pool layer should wrap the typed fields

- **Spec**: `docs/sessions/U1_STATS.md`
- **Kind**: cost
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Derived values live in three places today: `rpgkit_rules/src/derived.rs` (a rating evaluator: formula, constants, min, max, status modifiers; no bands or current value), hard-coded methods in `stats.rs:258-282` (HP, initiative speed, armour rating, crit, resist), and unit meters in `rpgkit_battle/src/meter.rs` (bands, decay, battle-only). Poise is a fourth (`poise.rs`, max from a derived row, refill hard-coded); AP is `Unit.ap` and `max_ap`, read in about 20 files including the AI and planner; HP is `unit.hp`. So U1 is three builds: the derived class and pools (L, about 8-12 files), the damage pipeline (M), and the stat re-wiring (M). Saves are unaffected while pools are battle-only, which "full each fight" implies.
- **Recommendation**: State that HP, AP and poise keep their typed fields, with the pool layer as a facade over them; only new pools use the generic store. Moving them into a generic map touches every AI and planner file for no gameplay gain.

### FB-05: The pipeline has a third rounding path, and some factors do not fit a stage

- **Spec**: `docs/sessions/U1_STATS.md`, modular damage
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Besides the two rounding points (attack power, final hit; `damage.rs:130`, `:221`), `unit.rs:1091` `take_damage` rounds flat hazard damage, arena pushes and trigger moves separately, applying guard only. Also: `power_scale` (tier) is a multiplier inside the base stage, against "flat modifiers only in base"; authored `power` and fixed-stat units return early and bypass everything; the ally intercept (`command.rs:775-830`) calls the formula twice and splits damage with another rounding. Poise stagger (`poise.rs:164`) is purely multiplicative with one rounding, a floor of 0 and no armour or crit, so it fits only if the stage names are generic. An exact port is possible if multiplication keeps today's f64 order; `rules_golden` will catch any off-by-one at a half.
- **Recommendation**: Decide whether flat hazard damage and intercepts run the pipeline; allow multipliers in the base stage (tier) while flat adds stay base-only; keep the fixed-stat and authored-power overrides; make the first build slice an exact port with every golden byte-identical before any new modifier.

### FB-06: There is no guard status today

- **Spec**: `docs/sessions/U1_STATS.md` (two mitigation layers); decision "What Stand is"
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Guard is a bare `Unit.guard` number set by Defend commands (`command.rs:585`), the resolver and triggers, cleared on a poise break and at round end, and read in eight or more places (log, portrait, session snapshot, poise, AI, planner, metrics). Cover comes from terrain. The two-layer shape already exists (guard plus cover capped at 0.9, armour with K 50); fortify and expose are statuses inside the armour rating.
- **Recommendation**: Keep `Unit.guard` as the situational layer's accumulator, with new sources feeding it, rather than turning guard into a status; or, if it must be a status, size that as its own slice. Treat the cap move from 0.9 to about 0.75 as a balance change with a named re-record, separate from the refactor.

### FB-07: The intended changes move all three goldens; the speed split reaches the world crate

- **Spec**: `docs/sessions/U1_STATS.md`, stats to values
- **Kind**: cost
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Crit from PERC (`stats.rs:276`, `damage.rs:10-13`), potency from KNOW against resist (`status.rs:210`, chance = potency - resist; one changed chance shifts later draws on the RNG stream), and the cap move each re-record `rules_golden`, the timeline golden and `sim_golden`. `Unit.speed` is both initiative and move rate and is read in `plan.rs`, `command.rs`, `unit.rs`, `rpgkit_data` unit rows (fixed-stat rows carry `speed`), the host's UI JSON, the sim print in `sim_golden`, and `rpgkit_world/src/battle.rs:151`, which writes it. Burden (`stats.rs:263`) and carry overload (`carry.rs:79`) both lower speed today.
- **Recommendation**: Say which of initiative or move rate burden and carry lower, and what a fixed-stat row's `speed` means after the split. Build each intended change as its own commit so each re-record is attributable, with a HISTORY row each.

### FB-08: Composure already exists, and there is no corruption pool

- **Spec**: `docs/sessions/U1_STATS.md`, composure acceptance test
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Composure is already a rating row in `derived_stats.json` (`KNOW x 2 + CONS`, 0 to 100, shown on the party, dojo and battle screens). No corruption pool exists in code or data. For composure to run as data alone, these are missing today: a pool mode with a current value and a per-fight refill; a formula max (unit meters have fixed min, max and start); bands in percent of max (bands use absolute values); a distance filter on the `ally_downed` trigger ("within 2"). The pool-change effect exists as `meter` and `meter_set`, with its delta clamped to -10..10 (`meter.rs:285`).
- **Recommendation**: Drop the corruption clause from the acceptance test or give it a task; state composure's max (the formula, or 100); add percent bands and the distance filter to U1's build scope, as the acceptance test depends on them. Is the -10..10 delta clamp the intended limit for pool changes?

### FB-09: Tier as a formula input needs a number

- **Spec**: `docs/sessions/U2_CLASS.md`, tiers; `U1_STATS.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Formula inputs resolve in `derived.rs` (`Eval::input`, stats, level, gear mitigation, row constants, other derived rows). Adding `tier` is one branch, size S, but tier is a string (`unit.rs:174`: hero, minion, standard, elite, boss), so it needs a numeric mapping, plus the content check's allowlist.
- **Recommendation**: Name tier's numeric form: a data table of factors per tier, read by formulas as `tier_hp`, `tier_power` and so on, or an ordinal.

### FB-10: One share vector cannot reproduce today's classes; two can

- **Spec**: `docs/sessions/U2_CLASS.md`, C1 and C2
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Checked by script over all 7 classes at levels 1-10. Base shares (base / 62) reproduce every class at level 1. One share vector for the whole curve (budget 62 + 5 per level) differs from today's stats at levels 2-10 for all 7 classes; for example a level 2 warrior gets KNOW 11 and DEX 8 where today it has 10 and 7, as its growth puts nothing in KNOW or DEX. Base shares plus growth shares (growth / 5), with largest-remainder rounding on the running total, reproduce every class at every level exactly, so `rules_golden` holds.
- **Recommendation**: Two vectors per class: identity at level 1, and growth identity. A single vector changes 5 of 7 class curves visibly and moves every class-level golden row. Ties in largest-remainder break by largest fraction, then stat order, as `auto_spend` does.

### FB-11: Tiers scale only HP and power today, and budget scaling cannot replace power_scale

- **Spec**: `docs/sessions/U2_CLASS.md`, C7 tiers
- **Kind**: cost
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `stats::tier_scales` (`stats.rs:35`) gives HP, power and an AI skill cap: minion 0.4 / 0.6 / 2, standard 0.7 / 0.8 / 0, hero 1 / 1 / 0, elite 1.25 / 1.0 / 0, boss 1.5 / 1.1 / 0. HP multiplies max HP (`unit.rs:407`), power multiplies attack power (`damage.rs:115`). Scaling the budget instead moves every stat together (HP, damage, armour, crit, initiative, resist), and boss power 1.1 or elite HP 1.25 have no budget equivalent. A minion budget of about 25 breaks the "every stat 6 to 18" rule. "Tier" also means the AI skill cap, XP `kill_weight`, morale, portrait tier, gear tiers and skill ranks. This is the costliest U2 slice (L): it moves `rules_golden` (28 class-by-tier rows and the non-hero units), the timeline, sim, PHASEFOLD and `checks_golden` HP lines.
- **Recommendation**: Name what replaces `power_scale` (a tier modifier through the pipeline is the natural fit with U1); apply the 6-18 check to the class row, not the tier-scaled unit; keep the AI cap; rename the other meanings of "tier" in code; build this slice after U1's derived formulas exist, so the HP base's tier term goes in once.

### FB-12: Static templates exist already as derived units; the builders are what fail the acceptance test

- **Spec**: `docs/sessions/U2_CLASS.md`, unit types and acceptance test
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: All 47 units except `training_dummy` are already class, level and tier derived, and 38 world characters point at them through `base_unit`, so they read as static templates at a fixed level; no unit row needs rewriting. "Template plus overrides" also exists: `base_unit`, the character's `tuning` (5 keys, used by 1 of 38), `named_items` and passives. What fails "any unit fights with no unit-specific code" today: heroes build through `progression::build_as`, NPCs through `build_unit_in` plus `customize` plus `apply_character`; `apply_character` patches HP, power, armour, speed and AP after the build; the AI profile comes from the base row for heroes and from the entry for NPCs; the sim and Dojo call `build_unit` directly. Only `training_dummy` uses the classless fixed path, with a fallback to warrior stats (KITIDS1 removes the fallback).
- **Recommendation**: Define a static unit as derived with a fixed level and budget, reuse `base_unit`, `tuning` and named items as the overrides rather than a second override system, retire or migrate the fixed-stat mode, and route every builder through one function.

### FB-13: World stage needs a source in worlds with no campaign, and XP must follow it

- **Spec**: `docs/sessions/U2_CLASS.md`, mob growth by world stage
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Nothing passes world state into enemy creation today. The smallest seam is a `stage` on `WorldRoster`, computed by the battle screen from game state (act, or flags), applied when the roster builds each entry; `build_unit_in` needs no new parameter, and the host and sim pass 0. But `roster()` computes a foe's level and XP weight without game state, so XP would disagree with the level fought unless the stage reaches it too. A world with no campaign has act 0. No save change, as the stage is derived.
- **Recommendation**: Say which flag or act defines the stage where there is no campaign, and that the XP weight follows the stage. Pass only the stage now, leaving WORLDFIGHT1's `begin_world` held.

### FB-14: Proficiency levels exist but mean something else today

- **Spec**: `docs/sessions/U2_CLASS.md`, C3
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Level 0 means absent, and the class check requires 1-3. The requirement discount is level x 2, so level 1 already lowers it by 2 and level 2 by 4. Crit and burden bonuses start at level 2, and a block-cost reduction at level 3 is hard-coded. Scaling exists per class and gear (`ProfEntry.scaling`, 1.0-2.0, default 1.5) as the unit-power multiplier (`damage.rs:91-112`), so "scaling as a class weight in the weapon power rating" depends on U1's weapon power rating.
- **Recommendation**: Say which of today's level 2 and 3 bonuses (crit, burden, block cost) stay and which become the level 3 effect; whether level 1 keeps a requirement discount. Build this slice after U3 and U4, as it is the only one that changes hero combat numbers.

### FB-15: Smaller U2 points

- **Spec**: `docs/sessions/U2_CLASS.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (a) Loadout templates save little today: 34 of 48 unit rows carry gear, with 32 distinct loadouts; they mostly help future mob families. (b) `rpgkit_rules/src/budget.rs` is the gear budget (target 23), not the unit budget. (c) The party screen (`party.rs:1003`, `:1575`), the Forge (`forge.rs:180`) and the sim's `primary_stat` read `base_stats` and `growth` directly. (d) Saves store `spend`, XP and `base_unit`, never stats, so they are safe if the free pool is unchanged. (e) The frozen kit the goldens load (`timeline_kit.json`) still has `base_stats` and `growth`.
- **Recommendation**: Loadout templates as an optional last piece; name the new module or term "unit budget"; keep `base_stats` and `growth` as a compatibility view (or a loader for the frozen kit), so readers and goldens hold while the rows move to shares.

### FB-16: Guard as a status needs its own task before Stand is built

- **Spec**: `docs/specs/INTENT2.md` IR22; decisions "What Stand is", "Stand answers"; see FB-06
- **Kind**: cost
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `Unit.guard` is a number that `resolver.rs:112-126` zeroes every block and sets only while a Defend command runs; `command.rs:585` and `guard_start` triggers (`trigger.rs:209`, `:332`) set it, `poise.rs:461` and `command.rs:890` clear it. The decisions want a mitigation status from impact to the round's end or several rounds, the higher guard applying. No INTENT2 task (K1 to K3) owns this; it crosses A2, S1 and F1.
- **Recommendation**: Add a task "guard as a status", tied to the S1 and U1 designs, ahead of the Stand rows. FB-06 is the same seam from the U1 side, so settle both together.

### FB-17: INTENT2 still carries text the decisions replaced

- **Spec**: `docs/specs/INTENT2.md`, `INTENT2_MATRIX.md`; `games/emberwake/docs/specs/INTENT2_CASES.md`, `INTENT2_SKILLS.md`
- **Kind**: docs-delta
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: "Guard only while a Defend runs" remains in IR16, MATRIX Table B (Stand, follow-up fill), CASES rows 21 and 29, CASES section 4 round 3 (so its 2.6 power figure is wrong under the decision) and section 5 ("guarded in 4 of 6 blocks"). IR22 lacks the decided parts: the common guard is lower in value; a multi-round guard is a separate, costlier skill at the lowest value; guards do not stack, the higher applies; it starts at impact; only mitigation carries over and Riposte stays tied to Stand. Stale since decision "Stand answers": line 3 ("only section 7 is open"), IR21's "3 gap-close skills are the open question", IR22's "Open: section 7", CASES row 4 (Lunge as Assault), and SKILLS listing Lunge, Pounce and Charge In as Assault (counts: Assault 62 becomes 59, Maneuver 11 becomes 14). Removed authored fields still named: line 7 ("capstone 3"), CASES section 5 ("capstone only" for Quell and Nemeth), IR20 ("a Field row names one of a fixed set", "Assignments: the listing"), SKILLS' Bonus column and its counts (derived: 11 reach, 5 lingering, 7 area, 0 power; 19 skills list two intents), section 6 ("no item needs `intent_versions`"), and INTENT2-A's seed (28 intents, 12 second intents and 23 Field bonuses are no longer seeded). Wrong line references: `ITEM_AP` is at `command.rs:1451`; the "not used" log at `:668`; the `buff_pct` bug at `:688-691`. Section 8 says items gain `max_stack`, which already exists; only item `area` is new.
- **Recommendation**: Rewrite those passages. INTENT2-A then seeds only the 7 `area` fixes, the 3 gap-close moves, the removal of 72 redundant `round_intent` values, `strike` for two bosses, Brand's recovery, the item table and `intents`. IR15 defines "primary" as the first of the derived list.

### FB-18: The multi-round guard is a new skill, but INTENT2 says none are needed

- **Spec**: `games/emberwake/docs/specs/INTENT2_CASES.md` section 5 ("New skills needed before the slice: none"); decision "Stand answers"
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: No existing skill gives multi-round mitigation; `riposte`, `tl_fade` and `seventh_ward_bulwark` carry rounds only for their riposte, stealth and taunt statuses.
- **Recommendation**: Add one skill row to INTENT2-A, or defer the multi-round guard by name.

### FB-19: IR21's derivation needs a precedence order and a few rules

- **Spec**: `docs/specs/INTENT2.md` IR21
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Re-run over all 129 skills: exactly the 17 stated differences; after the 7 SWEEP data fixes every skill gets one answer, and the Field bonus changes are exactly the 3 stated (the rule reads `reach_max`). Gaps: engine kinds (MOVE, REST, TARGET, STATUS) are neutral; `hazard` applies to ATTACK only (`ash_cloud_fade` is a DEFEND with a hazard and stays Stand); `tripwire_snare` has a hazard and no shape, so hazard must come before "no shape: Assault"; a short PLUS or BURST (reach_max 1 or less) is undefined (none in data); CASES rows 26, 30 and 31 cannot be derived, as `area` and `target_mode` exist only on ATTACK and SUPPORT. Items have no `kind` or `target_mode`, so they need their own mapping: heal gives Support and Stand; buff gives Stand; damage with an `area` and `use_reach` above 1 gives Field and Assault; damage alone gives Assault. Today `item_intent` (`round_intent.rs:166`) makes foe-aimed bombs Maneuver. The skill's intent has three diverging copies (`round_intent::skill_intent`, `skill::intent_id`, the content check's `intent_of`), and `skill::position_by` uses one to find Maneuver, so a derived Maneuver without an authored `round_intent` would not dash. About 15 call sites compare intents for equality and become "row allows"; `deck_errors` would flag a skill in its second row. The pick must carry its row, so the planner knows which version of a dual skill was chosen. No current skill derives the `power` Field bonus.
- **Recommendation**: State the precedence (engine kind, then kind, dash or position, shape, hazard, no shape), the item mapping, and that CASES rows 26, 30 and 31 are out of scope. One derivation function in `rpgkit_rules`, used by all three callers. Keep `power` as a future default and say so.

### FB-20: Section 5's winners and losers contradict IR21; three bosses sit at the block limit

- **Spec**: `games/emberwake/docs/specs/INTENT2_CASES.md` sections 3-5
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: IR21 gives Volley an Assault row, so the ranger keeps its mix (and its data is reach 2-3, not 2-4). Spine Volley and Lard Spray are now dual, so the Amalgam has an Assault row and Orlov's Spray has two; CASES rows 11-13 say Field only; Spine Volley is reach 1-3, not 1-4. Of the 7 throws gaining an Assault row, Pulverizer, Cataclysm and Bloom reach exactly 6 blocks with their recovery and the Assault row's extra block, so a rank row could pass `MAX_ACTION_BLOCKS` 6.
- **Recommendation**: Rewrite section 5 and rows 11-13. The content check tests the derived extra block against the block limit on every rank row.

### FB-21: INTENT2 against U1 and U2

- **Spec**: `docs/specs/INTENT2.md`; `docs/sessions/U1_STATS.md`, `U2_CLASS.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (a) CASES round 6 has item damage "40 flat, no armour"; U1 puts flat values in the base stage, then mitigation, minimum 1. (b) Brimstone Draught raises CONS 30 % mid-fight, and U1 makes CONS feed max HP and poise, so a drink moves max HP. (c) S1's chance "potency - INT x 2.5 - resist" is the old formula; U1 makes potency come from KNOW. (d) "Enemies use no items" against U2's "any unit fights with no unit-specific code". (e) INTENT2's AI row ladder (row Z, ring row) against U2's weighted AI roles, moved to I1. (f) The 3-slot pouch and `pouch_packed` against U2's loadout templates. (g) U1's situational cap near 75 % against 0.9 in code; INTENT2's "higher guard applies" sits inside U1's additive guard plus cover. (h) The attack lock reads band 1-2 as melee and IR21 reads reach above 1 as thrown; they differ for Transfusion Pulse and Cataclysm, but shaped skills never lock, so behaviour does not clash.
- **Recommendation**: Rule on (a) to (f) in U1, U2 and I1; note (h) in IR17.

### FB-22: INTENT2's slices after IR21

- **Spec**: `docs/specs/INTENT2.md` section 8
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: No `also_intents`, `intent_versions`, `field_bonus` or `capstone` exists in code or data, so IR21 needs no rollback. K1 stays M but lighter: the derivation function, `RoundIntent::Field`, the row-aware planner, `field_fill_once`, Stand's tile as a Move, and the check printing what it derived; about 40 kit sites and 6 GUI files name `RoundIntent::Guard` or `::Support`. K2 stays M and is the only schema change (RING and ARC replace SWEEP, a new `affects`); it touches `skill.rs`, `ai.rs:1270`, `plan.rs:1314-1320`, `ground.rs` and the SWEEP entry in `HAZARD_SHAPES`. K3 is a smaller M (only item `area` is new; `item_ap` 0 replaces `ITEM_AP`). AI stays S; its row numbering maps onto the named `ROWS` at `ai.rs:1590`. Goldens: `checks_golden` has 2 cases quoting the `area` enum and changes with K2, a named re-record; the timeline, sim and rules goldens read the frozen kit, whose `tl_` units hold no area or gap-close skill, so no other golden should move. The `lingering` bonus adds 1 to `hazard.rounds`, whose range is 0-5 with 0 meaning the row's duration.
- **Recommendation**: Define `lingering` as one more than the row's duration, capped. Add the guard task (FB-16) to the slice list.

## FB-23 to FB-59 and UN-01 to UN-04 (moved 2026-11-07)

### FB-23: A unit with no class takes the catalog's first class, silently

- **Spec**: `docs/READY.md` row KITIDS1; `docs/specs/SILOS1.md` T4
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: KITIDS1 removed the kit's fallback to class `warrior` (`unit.rs:926`), but SILOS1's fix (a content error for a classless unit) would break `content check` and move `training_dummy`'s stats, the one classless unit. So a classless unit now takes the catalog's first class at level 1. That is neutral today only because warrior is first in `classes.json`; reordering the file would change the dummy's stats with no warning.
- **Recommendation**: The game names the class explicitly (for example `classless_class` in `game.json` or a tuning row), with a content error when a classless unit exists and it is missing; or give `training_dummy` a class. Either is a small task; U2's static templates (FB-12) may retire the classless path altogether.

### FB-24: The schema's class art default is still "warrior"

- **Spec**: `kits/rpg/schema.json` (class `art`); KITIDS1
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: KITIDS1 made a class with no `art` wear the body named like its own id, but `schema.json` still defaults class `art` to "warrior", so a class made in the editor still gets warrior, and the kit names a body by a game-like id. Changing a schema default can change what an omitted field means in rows already shipped (the writer leaves defaults out).
- **Recommendation**: Decide the default ("" so the code default applies is the reviewer's pick), and check no shipped class relies on the omitted default before changing it; a schema change with a re-save, so its own small task.

### FB-25: Fourteen effect types, and items and meter bands use them too

- **Spec**: `docs/sessions/U4_PASSIVES.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: There are 14 effect types, not 13: `spell_reach` (`check/simple.rs:72`, applied in `unit.rs`, 1-2 tiles, no shipped row uses it). Meter bands reuse the grammar with a 15th pseudo-type, `crit`. Gear items share it through `apply_effects` with 6 types (skill and status potency, spell reach, resist, trigger, forced move) and separate `gear_*` accumulators rebuilt on every sync.
- **Recommendation**: Name items and meter bands as consumers of the same grant kinds; say where `spell_reach` goes (U1 moved reach to the skill, so retire it, or make it a modifier).

### FB-26: Not every effect is additive; the modifier kind needs combine rules, and meter fits no kind

- **Spec**: `docs/sessions/U4_PASSIVES.md`, the four grant kinds; `U1_STATS.md`, the pipeline
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `foresight` takes the largest range (capped) with a first-source id; `unlock` takes the largest level (0-2) and ends in a path refresh; `resist` sums per status and caps at 100; `forced_move` sums bonus tiles (0-2 a row) and ORs two immunity flags; `stagger_taken` multiplies, filtered by action length; `skill_potency` adds (1 + sum), so it pools before multiplying. `meter` spawns a gauge at battle start, raises its floor and opens the meter's skills: none of the four kinds. These fit cleanly: `stat`, `ap_regen`, `status_potency` and `skill_potency` as `modifier`; `proficiency` as itself; `skill` and `unlock` as `grant_skill`; `trigger` as itself. 55 shipped rows hold 80+ effects; 19 rows have more than one.
- **Recommendation**: Write a combine rule per target (sum, product, max, cap) into the modifier kind; add a fifth kind for `meter` (for example `grant_pool`) or fold it into `grant_skill` with a pool declaration, rather than forcing it into `modifier`.

### FB-27: Proficiency is also read from raw data outside the passive code

- **Spec**: `docs/sessions/U4_PASSIVES.md`; UNITDRILL1
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `Proficiency::with_drills` (`stats.rs:146`) reads `slot` and `type` straight from the passive JSON, called by `check/unit.rs:301`, `check/world.rs:386` and `state.rs:180`, so the gear and equip checks read passives separately from the battle unit. It keeps the rule that slottable passives do not train outside a battle.
- **Recommendation**: The typed loader replaces this reader too, keeping that rule, or the checks and the battle unit diverge.

### FB-28: Sources already deduplicate; the spec's order and its item source differ from code

- **Spec**: `docs/sessions/U4_PASSIVES.md`; MASTERPLAN1 C04
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `character_passives` (`check/unit.rs:103`) gathers character passives, then faction grants, then discipline passives, then named items' grants, each once; `apply_passive` and deck passives dedupe again, so no duplicate bug exists. Drills are ordinary character passive ids. The spec's order (class or discipline, character and temperament, items, faction) differs; order matters only for `foresight`'s first source, trigger order and meter floors, so reordering can move trigger-order goldens. Shipped data is nearly empty here: all 9 factions' grants and all items' grants are `[]`, 2 disciplines carry one passive each, so the 4-source test needs fixture content. Two build paths apply passives: `battle.rs:125` for NPCs, live, and `progression.rs:202` for party members, from a snapshot taken at new game (`new_game.rs:19`), so a faction grant added later never reaches an existing save. The spec's "Equipped Items (U3)" source: worn gear applies inline effects through `gear_*`, rebuilt on every sync, not passive ids; named items' `grants` are ids.
- **Recommendation**: Keep today's order, or declare order irrelevant and accept the trigger-order golden moves. Say that dedupe means one application per id, while different passives still stack. Say whether U3 gear grants ids (deduped) or inline effects (not). The single builder of FB-12 unifies the two build paths, and decides whether a party member's passives are re-gathered from sources at each build rather than snapshotted.

### FB-29: Temperaments are not built; pick one home for default intent and AI role

- **Spec**: `docs/sessions/U4_PASSIVES.md`, ROSTER1-T fold-in
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Nothing of ROSTER1-T exists in code (only its spec). What exists: `UnitDef.default_intent` and `ai_role`, validated and read at battle start from the unit entry. Folding in touches the schema (`characters.temperament`), the character check (the passive must be slotless), a per-character hook for the AI profile and default intent (`AiProfile::for_unit` takes a unit row, not a character), and a save key for members' temperaments. The spec says the tag goes on "the passive or character".
- **Recommendation**: Put it on the character: `temperament` naming a slotless passive, plus `default_intent` and `ai_role` keys, leaving passive rows pure. Size M, after the single builder (FB-12) and ROSTER1-L.

### FB-30: U4's slices and goldens

- **Spec**: `docs/sessions/U4_PASSIVES.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: A read-time loader turning `PassiveDef.effects` into a typed grant view, with today's accumulators kept as evaluators at first, gives identical numbers with no row rewrite; pitfalls: trigger limit keys are `source#index`, so trigger order and index must not change; `foresight`'s first source and meter floors depend on add order; meter bands diff the old accumulator shapes. Saves store only passive ids, so no migration except a new temperament key. Goldens: none move with the loader; rewriting rows or error texts moves `checks_golden` (21 passive references, the raw `sword_training` row) and the hand-copied fixture catalog; temperament fields move `checks_golden` and possibly `journey_golden`. Slices: (1) typed view and loader with a double-load test asserting old and new are equal for all 55 rows and the fixture's 16, M; (2) one source gatherer and both apply sites into the FB-12 builder, S-M; (3) temperament field and AI override, M; (4) evaluators onto U1's pipeline, L, after U1.2; (5) optional row migration, S. The validation messages (`check/simple.rs:100-210`, 12 of them) and the editor's per-type rows change when the schema does.
- **Recommendation**: Land the loader first with zero golden diffs; move evaluators onto the pipeline only after U1.2's pipeline lands.

### FB-31: Gear's family table exists already; keep `weapon_type`

- **Spec**: `docs/sessions/U3_GEAR.md` section 1
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `item_families.json` already declares `hands`, `scaling_stats`, `off_hand_category`, `engine_rules` (always_ranged, focus, shield, spell_source, reach), per-hand base range and bands, and the loader resolves them onto `ItemDef`. There is no `weapon_kind` field: the item field is `weapon_type` (about 40 uses), also the key of class proficiency, the skill `weapon_types` gate and the passive `proficiency.gear`. `sling` is not a family. What does name families in code: `ALWAYS_RANGED_TYPES` (`gear.rs:10`, read at `:78`) duplicates the `always_ranged` rule, which only the content check reads, so a data-only `always_ranged` family with range 1 behaves as melee; `off_hand_ok` names orb and tome (`gear.rs:210`); `is_shield` reads the item type, not the `shield` rule; `skill_in_group` names staff and wand (`unit.rs:877`); `FIST` and the trained-fists rule (`damage.rs`); the art, Dojo and party-screen spots.
- **Recommendation**: Keep `weapon_type`; "migration" means making those code sites read the table's `engine_rules`. No rename.

### FB-32: "Zero Rust changes" fails as written

- **Spec**: `docs/sessions/U3_GEAR.md`, acceptance test 1
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Besides the sites above, the content check holds fixed lists: `GEAR_TYPES` (21, `check/skill.rs:537`), `WEAPON_TYPES` (18, `check/unit.rs:22`; the two disagree), `OFF_CATEGORIES` and `FOCUS_TYPES` (`check/item.rs:19-20`, `source.rs:690`), `ENGINE_RULES` (5, `check/item.rs:28`). A new off-hand category or engine rule needs code today.
- **Recommendation**: Derive those lists from `item_families.json`, and word the test as "a new family using existing engine rules, in data only"; a new engine rule stays a code change.

### FB-33: The wield penalty already exists in a different shape; and unarmed damage would change

- **Spec**: `docs/sessions/U3_GEAR.md`, soft wield penalty
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `wield_penalty` (`gear.rs:96`) costs 15 % of weapon power per missing stat point, capped at 60 %, plus an extra block cost at a deficit of 4 or more (whose only reader, `effective_block_cost`, has no callers); it touches weapon power only. Proficiency also adds +level weapon power (`stats.rs:179`), which the spec leaves out; level 3's only effect today, `block_cost_reduction` (`stats.rs:199`), is dead, so a level 3 mastery is new content. The spec's flat 0.75 plus 1 burden per missing point, on power and armour, also at proficiency 0. Unarmed: today weapon power is 0 unless fist proficiency is 2 or more (then 15 plus the level bonus, `damage.rs:18-31`, `:76-85`); "an implicit fist weapon at -25 % untrained" would make untrained unarmed deal 15 x 0.75 instead of 0. "No proficiency list means no restriction" (`class_can_equip`, `gear.rs:216`) must stay, or classless and hand-built units are all penalised.
- **Recommendation**: Give one formula for a stat deficit and for proficiency 0, say whether it replaces or stacks with today's ramp, add armour if wanted, keep or drop the +level power bonus, and decide unarmed explicitly; the unarmed change and the penalty change are separate commits with named golden re-records.

### FB-34: The soft penalty removes four hard gates, and the party screen must show it

- **Spec**: `docs/sessions/U3_GEAR.md`; UNITDRILL1
- **Kind**: cost
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Today an item a unit has no training for cannot be worn, gated in four places: `class_can_equip` (`gear.rs:216`), `UnitState::can_wear` (`state.rs:174-185`, also behind `validate` and the party pickers), the content check (`check/unit.rs:136-176`, `check/world.rs:623`) and the Dojo (`dojo.rs:1274`, `:1324`). The party pickers hide items that fail `can_wear` (`party.rs:1222`, `:2261`, `:2349`, `:2375`); soft means they all show, so the screen must display the penalty. `with_drills` stays as training. `wears_untrained` (one unit, the Purifier, and seed `cast1a4b.json`) retires. Skill gates stay hard. Only 8 shipped slots are untrained, 7 covered by drills.
- **Recommendation**: Include the party screen's penalty display in the slice (a game UI change, mock first), and say the skill gates stay hard.

### FB-35: Item-granted skills and stat changes are not reversible today

- **Spec**: `docs/sessions/U3_GEAR.md` (OFFHAND1 through `grant_skill`); `U4_PASSIVES.md` (items as a source)
- **Kind**: cost
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `apply_effects` (`unit.rs:463`) pushes a granted skill to `granted_skills` for good, and `stat` and `proficiency` effects are never cleared when gear changes; `rebuild_gear_effects` (`:429`) clears only potency, resist, status potency, spell reach, forced moves and triggers. Items may not carry `skill`, `stat` or `proficiency` at all (`ITEM_TYPES`, `check/simple.rs:74`). With the `deck` rule on (emberwake's row), `apply_deck` takes only owned families up to the active slots, and the world deck builder (`owned_in`, `state.rs:193`) reads known and granted skills, so a worn item's skill would be owned but not active, and invisible to the deck screen.
- **Recommendation**: One "sources recomputed on every sync" pass, shared by U3 items and U4's sources (built with FB-12's single builder), rather than a separate item list on the unit. Design calls: item skills are always active while worn and take no deck slot (as off-hand basics do); worn or held under a two-handed main.

### FB-36: Hands, budget, reach and carry differ from the built model

- **Spec**: `docs/sessions/U3_GEAR.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (a) Hands: the spec says a two-handed weapon uses both hands; built, a two-handed main may pair with an off-hand weapon that is not a focus (`off_hand_ok`, `gear.rs:208`, with a weapon switch); only shields and foci are excluded. MASTERPLAN1 says 3 hands, the spec 2. (b) Budget: 23 +/- 20 % is a class loadout's target, not an item's (`budget.rs:19`); an item's static override already exists as `special` and `special_note`; item `tiers` are threshold unlocks, so "tier_mult" has no meaning; the weights are code constants, not data, and derived-value modifiers carry no weight. (c) Reach: `range_band` overlaps `base_range`, `effective_range` (PERC 18 adds 1), the constant `REACH_WEAPON_REACH` 2 (`unit.rs:17`), `melee_reach` and `melee_band_max` (`skill.rs:338`), which the attack lock and INTENT2's "engaged" read; U1 moves ranged reach to the skill. (d) Carry is party-wide and always on today (`carry.rs`); the spec makes it a per-unit rating, off by default. (e) "Level 1 gets tier-1 family skills" has no code hook: gating is the skill's `min_proficiency` and `weapon_types`, plus path level.
- **Recommendation**: (a) correct the spec to the built rule or change it on purpose; (b) restate the budget as the loadout's; (c) `range_band.max` replaces the `reach` rule and its constant, ranged families keep `base_range`, and name one owner; (d) carry goes with U1's speed split; (e) "data on the skill rows".

### FB-37: U3's slices and goldens

- **Spec**: `docs/sessions/U3_GEAR.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Dropping `block_cost` touches `ItemDef`, the budget (`W_EXTRA_BLOCK`), the content check, `source.rs`, `store.rs`, the schema, 71 places in `item_families.json` and two items (`shd_tower` +1, `wpn_glass_wand` +2, whose notes promise "slow to raise"); their budgets rise by 3 and 6, which can change class budget warnings; the loader must ignore a stored key and drop it on resave. Skill and command `block_cost` is separate and untouched. Goldens: `rules_golden` runs on the frozen kit, so data edits do not move it; the penalty and unarmed changes do. `checks_golden` (43 `block_cost` lines) moves with the removal, the soft penalty and `wears_untrained`'s retirement. Slices: (1) drop `block_cost`, M, `checks_golden` re-recorded; (2) the family table drives the rules, M-L, every golden identical; (3) soft penalty, M, after (2) and FB-14, moving `rules_golden`, the timeline and checks goldens in separate commits; (4) reversible item grants, OFFHAND1 and the level 2-3 family bonuses, L, after U1.2 and U4.2; (5) per-unit carry, S, with U1's speed split.
- **Recommendation**: Answer FB-33 to FB-36 before slice 3; slices 1 and 2 can be ready-to-build now.

### FB-38: Docs delta from SKILLS1 and KITIDS1

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: SKILLS1: planner scripts live in `tools/planner/` (README there); `tools/fc.sh test` prints each panic's message; `flush.sh` logs gate times to `$AEGIS_PLANNER_LOG`. KITIDS1: `tools/fc.sh layers` also fails when kit `src` quotes a class id from a game's `classes.json`; the sim's classes, roles, rosters and limit tests come from `emberwake_sim::CLASSES`.
- **Recommendation**: Apply to `docs/WORKFLOW.md` (fc.sh help, the planner scripts), `games/emberwake/docs/GAME_BRIEF.md` ("where to look": sim classes), `docs/STRUCTURE.md` (`tools/planner/`), and the planner and task-landing skills' script paths; TASKS: SKILLS1 and KITIDS1 done.

### FB-39: Docs delta from TESTCLEAN1

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Tests make scratch folders only through `aegis_core::scratch::Scratch` (`new`, `vacant`, `until_thread_end`), removed when the test ends; folders are named `aegis_*`. A helper that returns a derived path must hand back the guard or use `until_thread_end` (31 tests failed on a first pass that dropped it early).
- **Recommendation**: `docs/STRUCTURE.md` `aegis_core/src/` row and `engine/README.md`: add `scratch.rs`; the `rust-conventions` skill: "tests make temp folders only through `Scratch`; never `temp_dir().join`; a helper returning a path keeps its guard"; TASKS: TESTCLEAN1 done.

### FB-40: BASERULES1-M as built: three parts of the spec are left for later rows

- **Spec**: `docs/specs/BASERULES1.md` section 4.2, `BASERULES1_LIBRARY.md` sections 2 and 7; READY row 4
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Built to the READY row: every REGISTER rule carries `requires`, `conflicts`, `bundle`, `system`, `tested` and `does`; `FIELD_RULES` (15), `BUNDLES` (7), `TESTED_SETS` (21, library section 4 without `[poise, flank]`), `LOCKED` (50 code constants with system and reason), `PROMOTED`. One hard error, `flank` without `round_intent`; warnings for conflicts, fields set with nothing reading them, `attack_lock` with `chase=classic` (untested), a split must-ship bundle, and an untested set. Both shipped rows are clean. Not built: (a) spec 4.2's contract surface (the `compute` signature, RNG draw order, the role list; no role table exists yet), as `--contract` lists ruleset fields per READY; (b) a schema entry for `move_after_blocks`, which library section 2 says this task adds; it would make the next save rewrite shipped data, so the contract shows it as `row-only`; (c) library section 7's short fight per tested set; (d) `game.json` `ruleset` and `test_rulesets`. The sim's `rules=` stays silent, where library section 7 says `adjust` prints the same warnings. The `LOCKED` scan checks both ways but does not recurse into subfolders, skips slice types, `pub(crate)`, `static` and multi-line declarations, and matches `PROMOTED` by name only.
- **Recommendation**: Rows for (a) to (d) when their designs need them; (b) as a schema-first slice with the re-save; the sim printing notes and a recursive `LOCKED` scan are small and can ride the next base-rules slice.

### FB-41: Docs delta from BASERULES1-M

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: New command `aegis rules [--contract]` (`tools/fc.sh rules`): rules by system, fields, bundles, tested sets, locked constants and the game's rows; `aegis battle rules=` prints the set's notes to stderr. A new golden, `kits/rpg/crates/rpgkit_host/tests/fixtures/rules_contract_golden.json`, recorded with `RECORD=1`.
- **Recommendation**: `CLAUDE.md` "Engine goldens": add the rules contract golden; `engine/README.md` and `docs/WORKFLOW.md`: the command; `docs/HISTORY.md`: a row for the new golden; `engine/docs/authoring/rule_library.md` (the design session's): the additive rule, the metadata, one picked set per game, the frozen-catalog test, value versus form versus new rule, and the candidates table; TASKS: BASERULES1-M done once landed.

### FB-42: Stage names differ between READY and the session notes

- **Spec**: `docs/READY.md` row 5; `docs/sessions/U1_STATS.md`, modular damage
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: READY names five stages (base, flat, mult, mitigation, round); U1_STATS names four (base, amplifiers, mitigation, finish) with flat adds in base only. U1.2-S1 follows READY: Flat right after rounding point one, Add allowed in Base and Flat, Mul in Base (tier), Mult and Mitigation, Reduce in Mitigation only, Round with no modifiers. No flat modifier exists today, so neither reading moves a number. Changing it is `Stage::ORDER` and one line in `Op::fits`.
- **Recommendation**: Name one set of stages in both docs, and say whether a flat add may sit after attack power's rounding (READY's Flat) or only before it (U1_STATS).

### FB-43: Docs delta from U1.2-S1, and take_damage could join the pipeline early

- **Spec**: shared docs; FB-05
- **Kind**: docs-delta
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: New `rpgkit_rules/src/pipeline.rs` (stages, `Op`, `Modifier`, `Pipeline` with `DAMAGE` and `STAGGER`, a trace per hit, `Hit.trace`) and `pool.rs` (a read-only `PoolView` over hp, ap and poise; `Unit::pool`, `World::pool`). Attack power, the hit and poise stagger run through it; `take_damage(mitigated=false)` and the intercept split stay on their own paths, as FB-05 allows. An exact port of `take_damage` is easy (Add, Reduce(guard), round to minimum 1, only when guard is above 0).
- **Recommendation**: `docs/STRUCTURE.md` `rpgkit_rules/src/` row: `pipeline.rs`, `pool.rs`, tests `damage_pipeline.rs`, `pools.rs`; `engine/docs/PORT_NOTES.md` (`damage_calculator.gd` row): runs `rpgkit_rules::pipeline`, exact port; `kits/rpg/README.md`: a U1.2-S1 line. Say whether `take_damage` joins in A2.2 or in the next U1 slice.

### FB-44: Share data cannot be stored yet: the schema rounds stats to integers

- **Spec**: `docs/READY.md` row 6 ("kit + data"); `docs/sessions/U2_CLASS.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: U2.2-S1 computes shares from today's integer rows and moved no shipped data. The schema's `stats` field type rounds values to integers, so share fields need a float-stats field type in `aegis_content` first (an engine change); adding share fields to every class row would also break the byte-for-byte resave until the data is re-saved. The writer prints floats to 15 significant digits (13/62 comes back as 0.209677419354839); the tie rounding on whole billionths absorbs it, so stats stay identical. Share keys are read as extra keys on a row, not schema fields, so the editor and the Forge do not show them.
- **Recommendation**: Move the class data to shares with the stat-weights slice: a float-stats field type, schema rows for `base_shares` and `growth_shares`, and a seed with the re-save, as one schema-first step.

### FB-45: An authored power below 1 is now raised to 1 before the multipliers

- **Spec**: `docs/sessions/U1_STATS.md`; U1.2-S1
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: In the pipeline, an authored `power` enters the base stage and is rounded to at least 1 at rounding point one, so a power of 0 or less would now be 1 before the multipliers, and the hit could pass the old floor of 1. No shipped data authors a power below 1, so nothing moves today.
- **Recommendation**: A schema minimum of 1 on authored power, or a note that it is the intended floor.

### FB-46: Free-spend allocation still reads level 2's gain

- **Spec**: `docs/sessions/U2_CLASS.md`, free allocation
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `auto_spend` (party screen and `unit.rs`) reads the `growth` view, which on a share row is level 2's rounded gain. That is exact for today's whole-point growth rows, but not for share rows whose growth is not whole points a level.
- **Recommendation**: When share rows ship (FB-44), `auto_spend` reads the growth shares, as `primary_stat` now does.

### FB-47: Docs delta from U2.2-S1

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: New `rpgkit_data/src/unit_budget.rs` (re-exported as `rpgkit_rules::unit_budget`): the unit budget, 62 plus 5 a level, two share vectors per class, largest-remainder rounding on a running total, ties on whole billionths to the larger fraction then the earlier stat; not the gear budget `rpgkit_rules::budget`. `primary_stat` reads the largest growth share.
- **Recommendation**: `docs/STRUCTURE.md` (`rpgkit_data`): `unit_budget.rs`; `games/emberwake/docs/GAME_BRIEF.md` stats line: "class stats are base_shares x 62 + growth_shares x 5 x (level - 1), rounded by largest remainder; integer rows read as shares"; TASKS: U2.2-S1 done once on `develop`.

### FB-48: S1.2 audit: the status class

- **Spec**: `docs/sessions/S1_STATUS.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (1) Guard values: S1 names `guard_common_mult` 0.50 and `guard_multi_round_mult` 0.30, but shipped Defends use mitigation 0.15-0.35 (guard 0.3, shield_wall 0.35, riposte 0.15) and `take_damage` multiplies by (1 - guard), so it is unclear whether these are damage multipliers or mitigation fractions; and the closed decision says the common guard is lower than the multi-round one, the reverse of 0.5 against 0.3. (2) "Wind-up start through block 6" cannot be a plain status: statuses are unit-wide and tick only at `roll_over` (`status.rs:290`); block timing exists only in `HeldGuard {mitigation, last_block}` (`trigger.rs:200-210`, `:324-333`), which already means "from block N to last_block, ends on a break or round end". Under 1C guard covers far more blocks than a Defend's own span today, so outcomes move broadly. A break must remove the status instance, not only set `guard_broken`. The AI picks Defend by skill kind and never reads `Unit.guard`, so it needs to see an active guard to avoid redundant Defends. (3) The 17 statuses are a Rust const (`status.rs:47-65`), not data; no `statuses.json`; about 25 call sites use `cap_of`, `stacking_of`, `display_name` and `row_of` without a catalog. `BLEED_PCT`, `DEBT_STAGGER`, `DEBT_STUN` are LOCKED, `COUNTER_MULT` a const, so S1's tunables must become ruleset numbers. (4) Behaviour that differs from the spec: stealth hides a unit only if it moved this round (`round_intent.rs:392-394`), not "untargetable, breaks on attack or damage"; bleed ticks at round end only (`resolver.rs:315`), not "on move or action". (5) Resist is already one formula, `potency - (INT x 2.5 + resist_for)` (`status.rs:210-213`); the trigger `resist` effect writes `status_resist` directly (`trigger.rs:726-732`), bypassing U4's combine; `resistable` is read only by the content checks, and `try_inflict` ignores it. (6) Fit to the six primitives: modifiers (weaken, empower, fortify, expose, haste, slow), ticks (poison, burn, regen, bleed), block locks (stagger, stun), reactive (riposte), rule flag (steadfast) fit; root (locks Move and stops the chase, not blocks), stealth, taunt's authoring rules and guard need special cases. Goldens: guard and 1C move the timeline and sim goldens and the rules contract; `rules_golden` holds if `Unit.guard` stays as the accumulator.
- **Recommendation**: Restate guard values as mitigation fractions per skill, and fix which guard is lower; build guard on the HeldGuard mechanism with `Unit.guard` kept, not a second mechanism; load the 17 rows as a frozen kit table behind the same function signatures first; narrow stealth and bleed to today's behaviour or schedule each change as a named golden move; route the trigger `resist` effect through U4's combine; honour `resistable` or drop it; do the KNOW potency term in U1's KNOW commit. Slices: (a) the table, M, neutral; (b) id literals to primitive reads, M, neutral (AI heuristics stay on ids); (c) tunables to ruleset numbers and `resistable`, S, rules contract only; (d) guard as a status with 1C, L, moves timeline and sim; (e) KNOW term, stealth and bleed, S each, moving goldens.

### FB-49: S2.2 audit: poise pips

- **Spec**: `docs/sessions/S2_POISE.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (1) S2 makes a poise break queue `stagger` for next round; today a break only cancels the unit's unlanded actions, drops its guard and holds poise at 0 until refill (`poise.rs:453-488`); `stagger` is queued only by AP debt. A broken unit would lose the rest of the round and blocks 1-4 of the next: the largest balance lever in either spec, and it stacks with the 0.5 `repeat` multiplier. (2) S2 covers only the hit-to-pips step; rules built on today's continuous bar need a pip meaning: `windup_poise` above max (`poise.rs:305-327`), `unbreakable` holding at 1 or more (`:392`), break chance from taken over before (`:219`), `sure_break` (`:394`), Rest restoring poise (`:331`, `rest_poise_per_block` 5), `stagger_taken` passives, the arena `stagger` key, the trigger `stagger` effect and collision stagger. (3) With shipped stagger 4-20 (76 of 129 skills at 0) and max poise 24-90, a tank at 60 has T 12, so hits of 4-10 deal no pips; 7 skills at stagger 4-5 never matter above max poise 25 unless doubled; boss breaks likely fall (T 15-18); the timeline golden's 25 break lines will not survive. S2 says `roundi(max / 5)`, READY says `Max_Poise / 5`. (4) Pips need a state field beside `value` (the pool view is read-only); U1 adding STR to max poise changes T and every poise, so it lands separately. (5) The battle view already draws 5 pips from value over max (`marks.rs:822-836`) and forecasts from `poise::forecast_hits`; the forecast needs pip fields or the "after" is wrong.
- **Recommendation**: Confirm the break-queues-stagger rule, and ship it as its own slice. Add a decision list for (2) to S2; defaults: `windup_poise` as bonus pips, break chance from pips lost over pips before, Rest restores 1 pip per N blocks, `unbreakable` holds at 1 pip. State one formula for T. Slices: (a) pip numbers and a `pips` field, unused, S, rules contract only; (b) the floor(D / T) rule on every path behind a bar-or-pips switch, default bar, M, neutral; (c) flip timeline to pips with the decisions and real pips in the UI, M, moves timeline and sim; (d) break queues stagger, S, after S1's table, moves timeline and sim.

### FB-50: W1.2-W3.2 audit: world fights, services, clock

- **Spec**: `docs/sessions/W_WORLD.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: The fight start is `BattleScreen::world` (`emberwake_gui/src/battle.rs:677-747`); the settle is already party-first in the kit (`battle::apply`, `rpgkit_world/src/battle.rs:301-330`), so `finish_world` is mostly moving the runner dispatch and `FIGHT_SOURCE` into the kit (S); `begin_world` is the real M, as `Prebuilt` carries GUI-only art, so the kit returns units and a callback paints them. It is exact only after the single builder (READY row 10): heroes use `field_unit` and `build_played`, enemies `WorldRoster::customize`. `WorldRoster { d, world }` is a struct literal at 16 sites, so a constructor comes first. The spec's five clock causes do not match `Cause` (`Days`, `Leg`, `Battle`, `Enter`, `Effects`, act change). W3.2's clock parameters are already manifest data (`campaign`, `purse`, `rest`, `defeat`); `purse::block` returns `{}` outside a campaign world. No headless test plays a real world fight, so the round-trip acceptance test is new. SERVICES1 overlaps `start_fight` and edits the 5,764-line `region.rs`.
- **Recommendation**: Fix the cause list; drop W3.2 as a build slice (a contract listing only). Slices: (a) `WorldRoster::new` and `finish_world`, S, neutral; (b) `begin_world` with art through a callback and the round-trip test, M, neutral; (c) stage from `campaign.act` after row 10 and mob growth, S; SERVICES1, M, after (b).

### FB-51: P1.2-P2.2 audit: terrain and hazards

- **Spec**: `docs/sessions/P_SPACE.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `grid.rs:64-72` fixes `TERRAIN_IDS` (17), `BLOCKED` and `SLOW`; `TerrainRow.walkable` exists and about 35 sites read `grid.is_walkable`, so moving `walkable` to data is S. But "no terrain index special-cased" is bigger: ids, glyphs, the slow default and zone glyph rows are index-bound, and tests index by `TERRAIN_IDS`. `stops_forced_move` has no consumer: forced moves stop on unwalkable tiles (`forced.rs:295`). The frozen kit's 17 terrain rows have no `walkable`, so the loader keeps a missing-key fallback. The store resave test reads shipped data, so schema and data land together, as does `move_after_blocks` (FB-40b), which moves the rules contract golden. `max_move_blocks` is a const (`plan.rs:23`) and the collide values are per-skill `forced_move` keys defaulting to 0, not ruleset tunables. Terrain hazards (`hazard_status`, `potency`, `magnitude`, `rounds`, `on`) and hazard rows (`status`, `potency`, `magnitude`, `status_rounds`, `damage`) use different keys; unifying renames keys, a re-save of both files and `checks_golden`, after A4.2 defines `EffectSpec`. Hazard damage runs "through guard, not armour" in `ground.rs`, so moving it to `take_damage` may move goldens.
- **Recommendation**: P1.2 stops at `walkable`, `stops_forced_move` (default: not walkable, no row sets it) and the cost default as data on the 17 rows; a dynamic terrain list is its own row; cut `max_move_blocks` and the collide values from the tunables table, or add a row to promote `MAX_MOVE_BLOCKS`. Slices: (a) walkable data with a fallback and re-save, S; (b) `move_after_blocks` schema-first, S, rules contract moves; (c) `MAX_MOVE_BLOCKS`, M, if kept; (d) unified cell effect, L, after A4.2.

### FB-52: I1.2 audit: AI

- **Spec**: `docs/sessions/I_AI.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `ai.rs` reads the 8 status ids in about 25 places, as heuristics, not plain predicates (`debuff_order`, `ai.rs:1087-1125`, has hand-written cases for expose and slow; skill pickers match `grant_status` riposte, fortify, regen). `ROWS` (`ai.rs:1590`) is an 8-name precedence array; there is no Row Z, Maneuver, Field or ring row. The five predicate names exist nowhere, and S1's status rows have no behaviour fields yet (S1.2 unbuilt). The spec maps `modifies_rating(AttackPower)` to fortify, but S1 makes fortify an armour modifier; weaken and empower are the attack ones. A generic "score any rating modifier" would start scoring haste, weaken, empower and burn, which the AI ignores today, moving the sim and timeline goldens. Status ids are also read on AI paths outside `ai.rs` (`plan.rs:493`, `:521`, `round_intent.rs:771`, `command.rs`, `metrics.rs:1331`). The spec's `ai_role` list (tank, striker, skirmisher, ranged, support, caster, boss) differs from the schema enum (AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT), and no role table exists.
- **Recommendation**: Fix the fortify mapping; scope "zero ids" to `ai.rs` or list the other files; reconcile the role names. Slices: (a) predicates replacing the ids one to one, after S1's behaviour fields, M, neutral; (b) a generic scorer behind a ruleset flag, default off, M; (c) Row Z, Maneuver, Field and ring rows, after A3.2, A4.2 and F3, L, named timeline and sim re-record.

### FB-53: E1.2-E3.2 audit: economy

- **Spec**: `docs/sessions/E_ECONOMY.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: `SUPPLIES_MAX` and `GOLD_MAX` are consts (`state.rs:15-16`) used by `Party::add_gold`, `add_supplies` and `validate`, which take no data or world; 8 callers; the check layer has its own copies (`check/world.rs:18`, `check/map.rs:24-25`, `quest.rs:11`). The party and purse are global across worlds, so a per-world cap is ambiguous and a save loaded under a lower cap would fail `validate`; the manifest `purse` block is campaign-gated. `gold_max` already names a different thing on encounters (`check/encounter.rs:1098`). Stage-scaled XP has no source yet: XP weight is computed in `roster()` and again in `rpgkit_sim/src/runner.rs:386`. E3 (keepsakes, party size, bunks, wounds, settle) is built; E3.2 is a contract pass.
- **Recommendation**: Caps on `game.json` (one per game), clamped on load so no save breaks; name it `purse_gold_cap`. Slices: (a) `PurseCaps` read by the party methods with today's values, S; (b) the same in the checks, S; (c) stage in `roster()` and the sim, after mob growth and FB-50 (a), M.

### FB-54: T1.2-T2.2 audit: story

- **Spec**: `docs/sessions/T_STORY.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: The `stat` condition is `best_stat >= min` (`dialogue.rs:187`), where `best_stat` builds every active member on each call and takes the party's best. `condition_holds` takes the game state by shared reference and runs on every redraw (choice listing, lock text, the stat prefix, story card `when`), so a roll inside it would advance `gs.rolls` every frame, cannot mutate the state, and could be save-scummed. The spec says "the unit's stat"; today it is the party maximum. "Pools and ratings" need U1's ratings, not built yet. `dialogue_roll_die` has no home in the ruleset numbers. T2 needs little new.
- **Recommendation**: The roll is a choice-time effect (a `check` effect setting an outcome flag), and conditions read the recorded flag; `threshold` stays a pure condition. Say whose stat. Slices: (a) `threshold` mode with `threshold` as an alias of `min`, S; (b) the roll effect with a `dialogue` stream and `dialogue_roll_die`, plus a determinism and reload test, M; (c) rating reads after U1.2, S.

### FB-55: A1.2 audit: value, consumables, stance cleanup, budget

- **Spec**: `docs/sessions/A1_VALUE.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (1) Buff items apply at once today (`apply_item` calls `apply_attack_boost(1 + buff_pct)`, `command.rs:688-691`; `buff_stat` is never read). Rerouting them to `empower` through `status::queue` makes them land next round, and `empower` is capped at 25 (`status.rs:62`), so the 50 % Elixir of Focus and 30 % Brimstone Draught would be clipped; `grant_now` (`status.rs:249`) applies at once. (2) Item damage is `w.damage(target, item.damage, true)` (`command.rs:683`): no armour, no guard, no minimum 1; routing it through mitigation is a balance change. Item range and use are duplicated in `dojo.rs:1188` and the AI (`ai.rs:399-429`, `:1809`), and items have no `area` yet. (3) `ITEM_AP` 1 is a constant (`command.rs:1451`); `item_ap` must stay 1 under classic, as classic goldens use pouches. (4) `{flow}` and `{disrupt}` render from the row with no ruleset (`source.rs:652-665`), and classic shares these skills with stances on, so the stance cleanup cannot be a pure data edit; `feint` (1 block, 0.7) is not dominated by Strike (2 blocks). (5) A new budget warning on every ranked skill is a named `checks_golden` re-record.
- **Recommendation**: Items use `grant_now`; decide the magnitude cap or new rows in the balance lane; port item damage exactly first, and make mitigation for items its own flagged slice. Pass a `stances` flag to the token renderer so tokens are empty with stances off. Gate the budget check behind a flag, off by default. Slices: (a) `item_ap` as a ruleset number, classic 1, timeline 0, S; (b) stance tokens and warning, S-M; (c) items through the pipeline, after A2's first slice, rule-gated, M; (d) budget check, flag-gated, S.

### FB-56: A2.2 audit: resolution

- **Spec**: `docs/sessions/A2_RESOLUTION.md`
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Not yet routed through the pipeline: `take_damage` (`unit.rs:1091`), `heal` (`:1100`), `heal_amount`, item damage, hazard damage (`ground.rs:346`, `arena.rs:231`) and tick damage. A2 says Round is `max(1, ...)` everywhere, but today `take_damage` keeps a minimum of 1 only when guard is above 0, and hazard and tick damage of 0 stays 0. `heal_amount` is `heal_base + roundi(stat x mult)` (`command.rs:1387-1394`); there is no KNOW potency or passive heal multiplier yet. `skill_mult` is one pre-multiplied scalar into which callers fold flank, splash and counter (`command.rs:787-823`); splitting it into separate modifiers changes the f64 order and can flip rounding. Of the three re-derivation triggers, armour already reads fortify and expose per hit (`damage.rs:158`); the new work is pool-band modifiers, after U1's bands; max HP and max poise are already not re-derived mid-fight; initiative is not a rating yet.
- **Recommendation**: Port exactly first; make the minimum-1 floor everywhere a flagged rule change of its own; show the folded factors as trace-only entries rather than splitting the multiplication. Slices: (a) `take_damage` and heal ports with traces, S-M, neutral; (b) trace entries for the folded factors and the heal trace, M, neutral; (c) band re-reads, after U1's bands, M; the guard cap 0.90 to 0.75 as its own named balance commit.

### FB-57: A3.2 and A4.2 audit: shapes, effects, derivation

- **Spec**: `docs/sessions/A3_SHAPES.md`, `A4_EFFECTS.md`; `docs/specs/INTENT2.md` IR9, IR17, IR21
- **Kind**: conflict
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: (1) Retiring SWEEP can move classic goldens: the frozen kit's brawler knows `sweep_kick` and `cyclone_kick`, and classic SWEEP never fires today (the planner refuses it), so a working ARC under classic would start the brawler swinging; SWEEP lives in two enums (`skill::Area`, `ground::Shape`), `HAZARD_SHAPES` and `ai.rs:1270`. (2) A3 says shaped skills always lock to their ground tiles at wind-up; IR17 says an Assault shape centres where the target stands at impact and only Field is tile-fixed. A3 defaults `affects` to enemies for attacks; IR9 says Field defaults to all, as today. (3) A4's step 5 gives a self-centred ARC to Field and Stand and every melee shape to Field and Assault; IR21 gives RING to Stand only and ARC and LINE to Field only, and A4's own counts (Assault 59, Maneuver 14, 19 dual) match IR21; A4 also omits FOCUS (Stand) and DISENGAGE (Maneuver). (4) K1 is nearer L: about 108 `RoundIntent::` sites in `rpgkit_battle` plus the GUI. The schema is a single-line file, so K2, K3 and `affects` edits must go one at a time. (5) `EffectSpec` normalises 5 vocabularies at about 11 call sites; debuffs calling `try_inflict` add RNG draws.
- **Recommendation**: Add RING and ARC beside SWEEP first, keeping SWEEP as a legacy alias under classic until the frozen kit migrates, and remove it last with the `checks_golden` re-record; rewrite A3 section 2 ("Field and drawn rows lock; Assault shapes track") and derive `affects`'s default per row; replace A4 step 5 with IR21's rows plus FOCUS and DISENGAGE. Slices: K1a derive function and check output, rule-gated, M; K1b the Field intent and row-aware picks, L; K2 shapes, direction snap, `affects`, M-L, with the 2 `checks_golden` cases quoting the `area` enum; K3 item area, item intent, spends and refill, M; `EffectSpec` after S1's table, M.

### FB-58: F1.2-F4.2 audit: fight flow

- **Spec**: `docs/sessions/F_FIGHT_FLOW.md`
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: F1: the attack lock needs no A3 code; areas never lock today. `lock_tile` has no power test, though the spec says power above 0, and zero-power attacks exist (`intimidate`, `tripwire_snare`): an S fix. The switch-on is one data row in its own commit; the frozen kit's ruleset has no `attack_lock`, so the timeline golden moves only if the fixture is re-frozen. Initiative waits for U1's speed split (`Command::initiative` reads `speed`, `command.rs:275-277`). F2: the debt penalty goes straight into `st.active` with no `rounds_left` (`status.rs:436-439`), not queued, so a literal queue would land a round late; `DEBT_STAGGER` and `DEBT_STUN` are LOCKED constants, and making them ruleset fields re-records the rules contract golden. F3 is INTENT2-K1b; Stand's 1-tile Move uses the standard walk. F4: `fire_phase` uses `grant_now` with no resist roll (`session.rs:955`); no shipped phase grants a status.
- **Recommendation**: READY's "switches on once A3.2 lands" is a design gate, not a code dependency; say which is meant. Keep the debt status in `st.active` with consistent `rounds_left`; give `EffectSpec` an explicit `resistable`, default no roll for phases. Slices: (a) attack lock power test and switch-on, S; (b) debt ids and thresholds as ruleset numbers, S, rules contract; (c) an initiative accessor, then the burden change after U1, S; (d) phase status through `EffectSpec`, after A4, S.

### FB-59: Build order across the audited blocks

- **Spec**: `docs/READY.md`; all `.1` sessions
- **Kind**: feasibility
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Finding**: Hard dependencies found by the audits: S1's status table and behaviour fields before I1's predicates, F2's custom ids and A4's `EffectSpec`; A2's exact `take_damage` and heal ports before A1's consumables; U1's speed split before F1's initiative; K1 before K2 and K3, K2 before IR21's full derivation; the single builder (READY row 10) and mob growth before W's `begin_world`, world stage and stage-scaled XP; `WorldRoster::new` and `finish_world` before SERVICES1; A4's `EffectSpec` before the unified cell effect and the AI's Field rows. Behaviour-neutral slices that can run in any order now: A2 ports, F2 debt ids, A1 `item_ap`, the stance tokens, K1a, additive RING and ARC, P's `walkable` data, E's purse caps, T's threshold mode, W's constructor and `finish_world`.
- **Recommendation**: Order the next READY rows from that list; each named golden move (guard cap, item mitigation, break-queues-stagger, pips, attack lock switch-on) as its own commit with a HISTORY row.

### UN-01: Old test folders in the temp dir need a manual delete or a permission

- **For**: user
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Note**: About 19,000 folders from tests before TESTCLEAN1 sit in `C:\Users\tusha\AppData\Local\Temp` (`emberwake_*`, about 18,870, and `rpgkit_*`, about 194). New test runs no longer add any. The build session's cleanup command (delete those older than 90 minutes, so nothing a running build uses is touched) was blocked by the permission check, and it does not retry denied actions.
- **Ask**: Delete them by hand, allow that command, or leave them; they are small.

### UN-02: M1 waits on the user's look at the battle view

- **For**: user
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Note**: Battle view parts 1 and 2 (VISCLAR1-D) are on `develop`. The checklist with exact steps is `games/emberwake/docs/NEEDS_YOUR_EYES.md`, battle view section (Scene Runner > Arena > `tl_flank`, Ruleset set to Timeline).
- **Ask**: The user's look closes M1; it blocks no build work.

### UN-03: `main` is far behind `develop`

- **For**: both
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Note**: `develop` is 111 commits ahead of `main` (release PR #54 was 2026-11-03). Releases go by PR at milestone ends or when asked; CI passes on `develop`.
- **Ask**: Whether to release `develop` to `main` now, or at M1's close.

### UN-04: FEEDBACK.md is past its size limit until open items are answered

- **For**: design
- **Status**: closed (answered in `docs/READY.md` section 4, 2026-11-07)
- **Note**: The 12 feasibility audits added FB-48 to FB-59, so `docs/FEEDBACK.md` is about 50 KB, past the docs check's 25 KB warning (a warning, not a failure). The build session moves only answered or closed items to the archive.
- **Ask**: Answer FB-23 to FB-59 in READY section 4 so they can be archived, or allow the build session to move a long audit item's finding to the archive and leave a two-line summary here.

## FB-60 and UN-05 (raised through the chat relay, answered in READY 9c29b73a, recorded 2026-11-07)

### FB-60: FB-40's follow-ups were tracked nowhere

- **Spec**: `docs/READY.md` section 4, FB-40
- **Kind**: conflict
- **Status**: closed: accepted (@9c29b73a), follow-up `BASERULES1-M2` (READY row 21)
- **Finding**: FB-40's answer folded the sim `rules=` notes and the recursive `LOCKED` scan into "the next base-rules slice", but no READY row carried them, so archiving FB-40 would have lost them.
- **Recommendation**: A READY row for them.

### UN-05: Feedback approval pipeline gaps

- **For**: design
- **Status**: closed: policy (@9c29b73a), follow-up `SKILLS1-FB` (READY row 22)
- **Note**: Archiving checked answers by eye; nothing tied an accepted answer's promised work to a READY row, answers to a commit, or kept the disposition.
- **Ask**: A `Follow-up` and `Disposition` column in READY section 4, a check script, and `<disposition> (@<commit>)` on archived items. Adopted.

## FB-61, FB-62, FB-63, FB-64, FB-65, FB-66, UN-06 (answered in READY @57b5b77d, moved 2026-11-07)

### FB-61: U3.2-S1 landed: docs delta and two item notes that promise removed behaviour

- **Spec**: `docs/sessions/U3_GEAR.md`; FB-37
- **Kind**: docs-delta
- **Status**: closed: accepted (@57b5b77d), follow-up `U3.2-S4`
- **Finding**: Gear `block_cost` is retired: gone from the schema, item rows, families, the gear budget (`budget.rs` `W_EXTRA_BLOCK` removed, `LOCKED` now 49) and the checks; `source.rs` `RETIRED_NUMBERS` makes a row that still sets it fail on load. Data moved through seed `u32s1` (22 item families, 2 items). Two intended golden re-records: `checks_golden` (only the block_cost check lines) and `rules_contract_golden` (one line, `locked budget.rs::W_EXTRA_BLOCK gear`). The `special_note` texts on `shd_tower` and `wpn_glass_wand` still describe the removed block cost. Old kit test fixtures and `timeline_kit.json` still carry `block_cost` keys; the loader accepts and ignores them (build-side tidy, no ask).
- **Recommendation**: `docs/HISTORY.md` row naming both golden re-records; GAME_BRIEF and `U3_GEAR.md` gear lines without block cost; new `special_note` text for the two items (a content row for the asset lane, or say to drop the notes).

### FB-62: U4.2-S1 landed: docs delta

- **Spec**: `docs/sessions/U4_PASSIVES.md`
- **Kind**: docs-delta
- **Status**: closed: accepted (@57b5b77d)
- **Finding**: New `rpgkit_rules/src/grants.rs`: `grants::read(effects)` gives `Grant`s in row order (`Modifier`, `Proficiency`, `GrantSkill`, `GrantPool`, `Trigger`); `Target::combine()` is Sum, SumCapped(100) for resist, Product for `stagger_taken`, Max for foresight and unlock, OrFlag for forced-move immunity; `Unit::apply_grants` folds them; `apply_effects`, `apply_band` and `Proficiency::with_drills` read it. No row rewritten; goldens unchanged.
- **Recommendation**: `docs/STRUCTURE.md` rpgkit_rules row: `grants.rs`; `engine/docs/PORT_NOTES.md`: the passive and gear effect readers now go through `grants`.

### FB-63: The single unit builder landed (READY row 10): docs delta and three questions

- **Spec**: `docs/sessions/U2_CLASS.md`, `U4_PASSIVES.md`; FB-12, FB-23, FB-24, FB-28
- **Kind**: docs-delta
- **Status**: closed: accepted (@57b5b77d), follow-up `U3.2-S3`
- **Finding**: `rpgkit_rules::build_from(cat, UnitBuild)` builds every unit (session spawn through `Roster::overrides`, which replaces `customize`; WorldRoster and SimRoster through `battle::character_overrides`, which replaces `apply_character`; heroes through `progression::build_as`; Dojo and sim stats through `UnitBuild::row`); `build_unit` and `build_unit_in` are wrappers. Static and growth units keep their old step orders inside it, so no golden moved. Passives are re-gathered per build (`progression::member_passives`); every shipped hero's list equals its new-game list. `training_dummy` is a fixed warrior with no gear (seed `u22s2`); a fixed unit with no class is a content error; class `art` defaults to `""` and an empty class `art` is still an error (rule A1). Open: (1) should an empty class `art` mean the body named like the class? (2) the equip check (`state.rs:180`) and deck ownership still read the saved passive list, not the re-gathered one; (3) hero character `tuning` and named items still do not apply (RECRUIT1 forbids them).
- **Recommendation**: `engine/docs/PORT_NOTES.md:264` and `:277` (`WorldRoster::customize` becomes `overrides`), `docs/STRUCTURE.md` rpgkit_rules row and `kits/rpg/README.md:61` (Dojo uses `build_from`, not `build_unit_in`): name `build_from`, `UnitBuild`, `Overrides`, `Growth`, `Roster::overrides`, `character_overrides`, `member_passives`; GAME_BRIEF: "`training_dummy` is a fixed warrior with no gear; a fixed unit with no class is a content error"; HISTORY rows for the FB-23 error and the FB-28 re-gather. For (2), a follow-up row that moves the equip check and deck ownership to the re-gathered list, or "none" if the saved list is meant.

### FB-64: SKILLS1-FB landed: the feedback check script, one reading of "match", and docs delta

- **Spec**: `docs/READY.md` row 22; UN-05
- **Kind**: docs-delta
- **Status**: closed: accepted (@57b5b77d)
- **Finding**: `tools/planner/feedback_check.py` checks this file against READY section 4: every open `FB-nn` / `UN-nn` here has a section 4 row; each row's Disposition is `accepted`, `deferred` or `policy` and its Follow-up is not empty; every Follow-up other than `none` is a task id in the Task ID column of section 1 or 3 (a range such as `U4.2-S3..S4` also matches `U4.2-S3`, both ways); a Feedback ID such as `FB-48a` is a parse error (one row per plain id); a row for an item in neither file is a warning. `archive` moves every item with a valid row to `docs/archive/FEEDBACK_CLOSED.md` with `closed: <disposition> (@<commit>)` plus `, follow-up <ids>`, the commit being the last one that changed READY.md. Reading taken: an open item with no section 4 row is a mismatch (exit 1), as the row asks; `--pending-ok` lists it as "waiting for Design" instead, for the build session's start-up pass. Today: all 43 section 4 rows valid; FB-61 to FB-64 and UN-06 wait for Design. Tests: `tools/planner/test_feedback_check.py` (17), run by a new `plannertests` step in `tools/fc.sh tooltests` and the gate.
- **Recommendation**: `.claude/skills/build-session/SKILL.md` step 3 (and the design-session skill's archive step): run `python3 tools/planner/feedback_check.py --pending-ok`, then `archive`; `docs/STRUCTURE.md` tools row: `tools/planner/feedback_check.py`; `CLAUDE.md` / gate wording: the gate also runs the tools/planner tests.

### FB-65: R1 needs Wave 2 rows W2-1 to W2-3 in section 1; build order for R1 first

- **Spec**: `docs/READY.md` sections 1 and 3; `games/emberwake/docs/MILESTONES.md` (R1); UN-03
- **Kind**: conflict
- **Status**: closed: accepted (@57b5b77d), follow-up `U3.2-S2`, `S1.2-S1`, `S1.2-S2`, `S2.2-S1`, `BASERULES1-M2`, `U1.2-S2..S5`, `U2.2-S3`, `U3.2-S3..S5`, `U4.2-S3..S4`, `S1.2-S4`, `S2.2-S3..S4`
- **Finding**: The user's goal for the current build session is to build to R1 and release `develop` to `main`. R1 (Unit & Status Foundation: U1-U4, S1-S2, C01-C05) ships Option 1C Guard and 5-pip Poise, which exist only in Wave 2: section 3 marks W2-2 as closing the R1 Unit half and W2-3 the R1 Status half, and W2-1 (the U1/U2 formula switches) comes before both. Section 3 promotes Wave 2 only "after Wave 1 Rows 7-20", but rows 14-20 (A, F, P, I, E, T, W) are R2 and R3 work, so R1 waits on them as written. Wave 1 rows R1 needs: 22 (`SKILLS1-FB`, landed), 8 (`U3.2-S2`), 11 (`S1.2-S1`), 12 (`S1.2-S2/S3`), 13 (`S2.2`), 21 (`BASERULES1-M2`). R1 also needs FB-63 (2) answered before W2-2, which rewrites the same equip check (`state.rs:180`), and W2-2 carries the party-screen penalty display, mock first (FB-34), which needs the user's look.
- **Recommendation**: (1) Approve building rows 8, 11, 12, 13, 21 in that order before rows 14-20 (rows 14-20 stay in section 1 for after R1, or move to section 3). (2) Promote W2-1, W2-2 and W2-3 into section 1 now, below row 21, with their slice order and named golden re-records, so the build session moves on without waiting a session; W2-4 to W2-6 stay held. (3) Answer FB-63 (2). (4) Say how the user reviews the W2-2 party-screen mock (screenshot in FEEDBACK, or a desktop look before W2-2 lands). (5) Confirm the R1 release is the `develop` -> `main` PR right after W2-3 lands with a clean gate, and whether a balance run or the user's look is part of the R1 ship gate.

### FB-66: The cloud session-start hook installs the ALSA headers: docs delta

- **Spec**: user approval in the build session chat (2026-11-06); known-issues row "alsa-sys"
- **Kind**: docs-delta
- **Status**: closed: accepted (@57b5b77d)
- **Finding**: A fresh cloud container could not run `tools/fc.sh build` or the gate: `alsa-sys` (rodio -> cpal) needs `libasound2-dev` and `pkg-config`, which only CI's `gate.yml` installed. `.claude/hooks/session-start.sh` now installs both when `pkg-config --exists alsa` fails (warns, does not stop the hook, if apt fails).
- **Recommendation**: `docs/workflow/cloud.md` "Environment" bullet: the hook also installs the ALSA headers (`libasound2-dev`, `pkg-config`); optionally the known-issues row: cloud sessions get them from the hook.

### UN-06: The old test folders in Temp are still there: the delete was blocked

- **For**: user
- **Status**: closed: accepted (@57b5b77d)
- **Note**: UN-01 approved deleting the pre-TESTCLEAN1 `emberwake_*` and `rpgkit_*` folders (about 19,000) in `%TEMP%`, but the session's permission check blocked the delete twice, so the build session will not retry or work around it. No cleanup agent or script covers them (the housekeeper is read-only; `tools/planner/cleanup.sh` removes only lane branches, removed worktrees and their build caches). They are harmless; new test runs no longer add any.
- **Ask**: Run this in PowerShell when no gate or build is running (it keeps the `aegis-target-*` build caches and anything under two hours old), or add a permission rule for it: `Get-ChildItem $env:TEMP -Directory | Where-Object { $_.Name -match '^(emberwake|rpgkit)_' -and $_.LastWriteTime -lt (Get-Date).AddHours(-2) } | Remove-Item -Recurse -Force`

## Open items, full text (moved for size, UN-04; still open in docs/FEEDBACK.md)

### FB-67: U3.2-S2 built: reach and the printed check lists wait on Design; docs delta

- **Spec**: `docs/sessions/U3_GEAR.md`; FB-31, FB-32, FB-36 (c), FB-37
- **Kind**: conflict
- **Status**: open
- **Finding**: Built with 0 golden diffs: `ALWAYS_RANGED_TYPES` is gone (`is_ranged_weapon` = the `always_ranged` rule or `base_range > 1`, so a data-only always-ranged family at range 1 is now ranged; no shipped item changes); `is_shield`, `off_hand_ok` and the spell group (`skill_in_group`, new resolved skill key `spell_source`) read the family's `engine_rules` (new resolved item key `family_rules`, which replaces `reach_rule`; the writer drops both); the check's membership tests (skill `weapon_types`, passive `proficiency.gear`, class proficiency, the `weapon_types` options) derive their lists from `item_families.json` (the 21 vs 18 "disagreement" is armour: gear = WEAPON, SHIELD and ARMOR families, weapon = WEAPON and SHIELD). Rows the loader did not resolve (the frozen golden catalog `timeline_kit.json`, hand-built test rows) fall back to `rpgkit_data::LEGACY_FAMILY_RULES` (eight family ids). Held, as the row cannot be met without moving a golden or gameplay: (1) **reach**: the family table has no `range_band`; the nearest field, `bands.<hand>.range.max`, is the allowed `base_range` band, which gives polearm 2 but scourge 1, so reading it would cut scourge reach from 2 to 1; `REACH_WEAPON_REACH` 2 stays. (2) **printed lists**: the check messages in `checks_golden` print `GEAR_TYPES` / `OPTIONAL_GEAR_TYPES` / `OFF_CATEGORIES` in an order the table cannot give (the table puts crossbow after cloth), and derived from the same table the family-level checks (gear-token name, off-hand ids, completeness) would always pass; the schema enum `items.weapon_type` also lists the tokens in the code order. Those lists stay code. (3) Resolved-row shape changed as above. (4) `OFF_CATEGORIES` and `FOCUS_TYPES` stay code too (printed in "Off-hand families are shield, orb, tome"; `FOCUS_TYPES` is read only by the no-family-table legacy check), so a new off-hand category still needs code (FB-31). Side effects on other catalogs, none on ours: with a family table, a skill, class or passive naming an optional gear type (polearm to sprayer) that has no family is now rejected (before, `OPTIONAL_GEAR_TYPES` let it through).
- **Recommendation**: (1) Reach, choose: (A) a new family field `range_band {min, max}` on polearm and scourge (kit schema slice, then a seed), the loader putting its max on the item with fallback 2; or (B) keep `REACH_WEAPON_REACH` as the value of the `reach` rule and drop "range_band.max replaces it" from FB-36 (c). Build recommends (B): no gameplay change and no new field. (2) Printed lists (and `OFF_CATEGORIES`), choose: (A) derive and re-record `checks_golden` in a follow-up, rewording or dropping the family-level checks; (B) make the kit schema's `items.weapon_type` options the one ordered list (a new family is then a schema edit, data only); or (C) leave them as code. Build recommends (C) for R1 and (B) later. (3) Confirm the resolved-row keys. Docs delta: PORT_NOTES "U3.2-S2: `is_ranged_weapon`, `is_shield`, `off_hand_ok` and the spell group read the family's `engine_rules` (resolved `family_rules`, skill `spell_source`); unresolved rows use `rpgkit_data::LEGACY_FAMILY_RULES`"; `kits/rpg/README.md`: the loader resolves `family_rules` and `spell_source`, the check derives gear and weapon tokens (`check::skill::gear_tokens`, `weapon_tokens`); GAME_BRIEF gear: "a new gear family using existing engine rules works in the rules from data alone; the printed check lists, reach 2 and fist are still code"; stale lines to fix: `engine/docs/PORT_NOTES.md:265` and `games/emberwake/docs/GAME_BRIEF.md:72` (the `reach_rule` item key), GAME_BRIEF `:207` ("Spell = ATTACK skill gated on staff or wand", "Orb and tome are off-hand only": both now family rules); `docs/STRUCTURE.md:43`: tests `rpgkit_content/tests/family_rules.rs`, `emberwake_content/tests/family_table.rs` and the updated `reach_rule.rs`; HISTORY: "U3.2-S2: `ALWAYS_RANGED_TYPES` retired (an always-ranged family at range 1 is now ranged; no shipped item changed); resolved `reach_rule` replaced by `family_rules`".

### FB-69: S1.2-S1 built: the status table is kit data; two questions; docs delta

- **Spec**: `docs/sessions/S1_STATUS.md`; FB-48 (a)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: The Rust const `status::ROWS: [StatusRow; 17]` is replaced by the frozen kit table `kits/rpg/crates/rpgkit_rules/src/statuses.json` (17 rows, old order), compiled in with `include_str!` and read as `ROWS: LazyLock<Vec<StatusRow>>`; every reader (`row_of`, `cap_of`, `stacking_of`, `display_name`, `intent_bound`, `magnitude_unit`, `has_magnitude`, `magnitude_or_min`, `buffs`, `debuffs`, `defend_buffs`) keeps its signature and values. Each row carries the S1 lifecycle header (`id`, `display`, `kind`, `resistable`, `default_rounds`, `stacking`, `cap`, `magnitude_min`/`max`, `magnitude_unit`, `intent_bound`) plus today's `grant`, and one behaviour primitive tag (modifier: weaken, haste, slow, expose, fortify, empower; tick: bleed, regen, poison, burn; block_lock: stagger, stun, root; targeting: taunt, stealth; reactive: riposte; rule_flag: steadfast); the tags are unread until row 12. No golden moved; no kit schema entry (the table is compiled-in kit data, not loader content). Test `rpgkit_rules/tests/status_table.rs` (4). Open: (1) home of the file: `rpgkit_rules/src/statuses.json` (beside its reader) or `kits/rpg/statuses.json` (beside `schema.json`, embedded the same way); (2) `grant` (support / defend / "") is not in S1's header but the content check and `defend_buffs` read it: add it to the header, or replace it with a primitive or intent field in row 12?
- **Recommendation**: Answer (1) and (2) (Build defaults: keep the file where it is; keep `grant` until row 12 says otherwise). Docs delta: `docs/STRUCTURE.md` rpgkit_rules row: "`status.rs` (status readers and stacking; rows from the compiled-in kit table `statuses.json`, S1 header fields and `Primitive` tags), test `tests/status_table.rs`"; GAME_BRIEF "where to look": status rows in `kits/rpg/crates/rpgkit_rules/src/statuses.json`; `kits/rpg/README.md`: the status table is kit data, compiled in, not overridden by games; PORT_NOTES `status.rs`: since S1.2-S1 the rows are data (`statuses.json`), same order and values; HISTORY: "2026-11-07 S1.2-S1: the const `status::ROWS` became the frozen kit table `statuses.json` (S1 statuses as data over six primitives; no golden moved)".

### FB-70: S1.2-S2/S3 built: status reads, four ruleset numbers, one named contract re-record; docs delta and four choices

- **Spec**: `docs/sessions/S1_STATUS.md` sections 1-4; FB-48 (b), (c)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit status rules read the frozen table: round-end ticks are the `tick` rows in table order (`status::tick_rows`, `tick_amount`, replacing `poison_damage`, `burn_damage`, `regen_amount`), block locks come from a new `lock_blocks` column on `block_lock` rows (stagger `[1, 4]`, stun `[1, 2]`), a flat tick shows as HP. Kept on ids, as S1 says or for want of a field: bleed formula, stealth, riposte, root, steadfast, modifier targets (no target/sign field), `debt_status`, the Dojo lost-action check, the `regen` metric, the stun book entry, content authoring rules, and `ai.rs` (I1.2-S1). New ruleset numbers, row-only (no schema entry, so no data re-save), defaults equal today's: `bleed_pct` 0.06 (0.01..0.25), `counter_mult` 0.5 (0.10..1.50), `debt_stagger` 3 and `debt_stun` 5 (both 1..10, per `S2_POISE.md` section 3; a row with `debt_stagger` above `debt_stun` is a content error, as stagger could never fire) (`FieldRule.range`). `try_inflict` honours `resistable` (a non-resistable row lands with no draw; no shipped path rolled one, checked across every inflict path and all shipped JSON); the trigger `resist` effect folds by U4's `SumCapped(100)`. Named golden re-record: `rules_contract_golden` only (+4 tunable lines, -3 locked lines, +4 promoted lines); every battle, sim, checks, journey, store and timeline golden byte-identical. Tests: `rpgkit_rules/tests/status_reads.rs` (7), `rpgkit_battle/tests/status_numbers.rs` (3).
- **Recommendation**: Confirm (Build defaults in brackets): (1) the `lock_blocks` column on the frozen table, which S1's header does not list [keep]; (2) an out-of-range row-only number is a content error [yes]; (3) `sim feel tune=` cannot tune the four until they get schema entries with a data re-save, as `move_after_blocks` waits for P2.2-S1 [later]. Docs delta: HISTORY: "2026-11-07 S1.2-S2/S3: `rules_contract_golden` re-recorded on purpose: `BLEED_PCT`, `DEBT_STAGGER`, `DEBT_STUN` (LOCKED) and `COUNTER_MULT` (const) became ruleset numbers `bleed_pct`, `counter_mult`, `debt_stagger`, `debt_stun` (row-only, promoted); no battle or sim golden changed"; GAME_BRIEF statuses: the rules read the table's primitives (ticks, block locks), per-status behaviour stays on ids, a non-resistable status lands without a roll, trigger `resist` folds by U4's combine; STRUCTURE `statuses.json`: "`lock_blocks` per block_lock row"; `kits/rpg/README.md` or the BASERULES notes: row-only numbers can carry a range; PORT_NOTES `status.rs`: `poison_damage`, `burn_damage`, `regen_amount` became `tick_amount`.

### FB-71: S2.2-S1/S2 built: poise pips behind `poise_mode` (default `bar`); docs delta and four questions

- **Spec**: `docs/sessions/S2_POISE.md` sections 1, 3-5; FB-49 (a), (b)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New row-only ruleset keys (no schema entry, no data re-save): `poise_mode` `bar` | `pips` (default `bar`; `FieldRule.values`), `poise_pips` 5 (3..8) and `max_pip_loss_per_hit` 3 (1..5), both needing `poise_mode=pips`. `poise::settle` is now the one stagger rule (hits, collisions, arena and trigger `stagger`, the Dojo); under `pips` a hit removes `clamp(floor(D / T), 0, max_pip_loss_per_hit)` pips with `T = max(1, roundi(max_poise / poise_pips))` and the unit breaks at 0 pips; `windup_poise` adds `roundi(windup_poise / T)` bonus pips (min 1), `unbreakable` / `steadfast` hold at 1 pip, `sure_break` clears, break chance = pips lost / pips before (a chip hit draws no RNG), Rest restores 1 pip per 2 Rest blocks up to `poise_pips`. `PoiseState` gains `pips` (0 under `bar`) and `rest_blocks`; `Forecast` gains `threshold`, `pips_before`, `pips_after`, `StaggerHit` the pip pair (derived from the bar under `bar`). A break under `pips` keeps today's consequences; nothing queues `stagger` (S2.2-S4) and no shipped ruleset uses `pips` (S2.2-S3). Named golden re-record: `rules_contract_golden` only, +3 tunable lines; every other golden byte-identical. Tests: `rpgkit_battle/tests/poise_pips.rs` (12), `rpgkit_content/tests/rulesets.rs` (+1).
- **Recommendation**: Answer for S2.2-S3 (W2-3): (1) under `pips` the continuous `value` still drops by D and the battle view, metrics and `feel.rs` read it: should S2.2-S3 move them to pips, or make `value` = pips x T? (2) windup bonus pips sit above `poise_pips` (as `windup_poise` sits above max today) [keep]; (3) the planner's Rest preview (`round_intent_plan::rest_of`) still shows a poise value [S2.2-S3 with the UI]; (4) `sim feel tune=` cannot set row-only keys [schema entries later]. Build note for S2.2-S3: under `pips`, `drop_bonus` recomputes the windup bonus pips from the current max poise instead of storing the count on the command; store it beside `poise_bonus` when a shipped ruleset flips to pips. Docs delta: HISTORY: "2026-11-07 S2.2-S1/S2: `rules_contract_golden` re-recorded on purpose: +3 row-only tunables `poise_mode` (bar|pips, default bar), `poise_pips` (5, 3..8), `max_pip_loss_per_hit` (3, 1..5); every other golden byte-identical under the default bar"; `kits/rpg/README.md`: `poise::settle` is the one stagger rule, the pip rule, `PoiseState.pips`, the forecast pip fields; `FieldRule.values` and `PoiseMode`; GAME_BRIEF poise: "Poise mode: bar (default, shipped) or pips (5 pips of T, floor(D/T) capped 3, break at 0 pips; windup bonus pips, hold at 1, sure_break clears, chance = pips lost / pips before, Rest 1 pip per 2 blocks); break consequences unchanged until S2.2-S4"; STRUCTURE: test `rpgkit_battle/tests/poise_pips.rs`.
