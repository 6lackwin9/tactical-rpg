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
