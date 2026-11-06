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
| `TESTCLEAN1` | done | `develop` 41de4e5c | Docs delta FB-39. UN-01 for the old folders. |
| `SKILLS1` | done | `develop` 41de4e5c | Full gate 2215 tests. |
| `KITIDS1` | done | `develop` 41de4e5c | FB-23, FB-24 open; docs delta FB-38. |
| `BASERULES1-M` | done | `develop` c804f8af | Full gate 2235 tests. FB-40, FB-41 open. |
| `U1.2` (`U1_STATS`) | slice 1 done | `develop` c804f8af | Pipeline and pool view, goldens byte-identical; FB-42, FB-43, FB-45. Slices 2-6 wait for READY. |
| `U2.2` (`U2_CLASS`) | slice 1 staged | `staging` | Two share vectors; every class identical at levels 1-10; FB-44, FB-46, FB-47. |
| `A4.2` (`INTENT2`) | audited | - | FB-16 to FB-22 closed; design updating INTENT2. |
| `U3.2` (`U3_GEAR`) | audited | - | Feasible, but the premise is partly stale; about 2M + 1M-L + 1L + 1S; FB-31 to FB-37. |
| `U4.2` (`U4_PASSIVES`) | audited | - | Feasible as a typed view plus compatibility loader, M-L in five slices; FB-25 to FB-30. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

FB-01 to FB-22 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-23.

### FB-23: A unit with no class takes the catalog's first class, silently

- **Spec**: `docs/READY.md` row KITIDS1; `docs/specs/SILOS1.md` T4
- **Kind**: conflict
- **Status**: open
- **Finding**: KITIDS1 removed the kit's fallback to class `warrior` (`unit.rs:926`), but SILOS1's fix (a content error for a classless unit) would break `content check` and move `training_dummy`'s stats, the one classless unit. So a classless unit now takes the catalog's first class at level 1. That is neutral today only because warrior is first in `classes.json`; reordering the file would change the dummy's stats with no warning.
- **Recommendation**: The game names the class explicitly (for example `classless_class` in `game.json` or a tuning row), with a content error when a classless unit exists and it is missing; or give `training_dummy` a class. Either is a small task; U2's static templates (FB-12) may retire the classless path altogether.

### FB-24: The schema's class art default is still "warrior"

- **Spec**: `kits/rpg/schema.json` (class `art`); KITIDS1
- **Kind**: conflict
- **Status**: open
- **Finding**: KITIDS1 made a class with no `art` wear the body named like its own id, but `schema.json` still defaults class `art` to "warrior", so a class made in the editor still gets warrior, and the kit names a body by a game-like id. Changing a schema default can change what an omitted field means in rows already shipped (the writer leaves defaults out).
- **Recommendation**: Decide the default ("" so the code default applies is the reviewer's pick), and check no shipped class relies on the omitted default before changing it; a schema change with a re-save, so its own small task.

### FB-25: Fourteen effect types, and items and meter bands use them too

- **Spec**: `docs/sessions/U4_PASSIVES.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: There are 14 effect types, not 13: `spell_reach` (`check/simple.rs:72`, applied in `unit.rs`, 1-2 tiles, no shipped row uses it). Meter bands reuse the grammar with a 15th pseudo-type, `crit`. Gear items share it through `apply_effects` with 6 types (skill and status potency, spell reach, resist, trigger, forced move) and separate `gear_*` accumulators rebuilt on every sync.
- **Recommendation**: Name items and meter bands as consumers of the same grant kinds; say where `spell_reach` goes (U1 moved reach to the skill, so retire it, or make it a modifier).

### FB-26: Not every effect is additive; the modifier kind needs combine rules, and meter fits no kind

- **Spec**: `docs/sessions/U4_PASSIVES.md`, the four grant kinds; `U1_STATS.md`, the pipeline
- **Kind**: conflict
- **Status**: open
- **Finding**: `foresight` takes the largest range (capped) with a first-source id; `unlock` takes the largest level (0-2) and ends in a path refresh; `resist` sums per status and caps at 100; `forced_move` sums bonus tiles (0-2 a row) and ORs two immunity flags; `stagger_taken` multiplies, filtered by action length; `skill_potency` adds (1 + sum), so it pools before multiplying. `meter` spawns a gauge at battle start, raises its floor and opens the meter's skills: none of the four kinds. These fit cleanly: `stat`, `ap_regen`, `status_potency` and `skill_potency` as `modifier`; `proficiency` as itself; `skill` and `unlock` as `grant_skill`; `trigger` as itself. 55 shipped rows hold 80+ effects; 19 rows have more than one.
- **Recommendation**: Write a combine rule per target (sum, product, max, cap) into the modifier kind; add a fifth kind for `meter` (for example `grant_pool`) or fold it into `grant_skill` with a pool declaration, rather than forcing it into `modifier`.

### FB-27: Proficiency is also read from raw data outside the passive code

- **Spec**: `docs/sessions/U4_PASSIVES.md`; UNITDRILL1
- **Kind**: feasibility
- **Status**: open
- **Finding**: `Proficiency::with_drills` (`stats.rs:146`) reads `slot` and `type` straight from the passive JSON, called by `check/unit.rs:301`, `check/world.rs:386` and `state.rs:180`, so the gear and equip checks read passives separately from the battle unit. It keeps the rule that slottable passives do not train outside a battle.
- **Recommendation**: The typed loader replaces this reader too, keeping that rule, or the checks and the battle unit diverge.

### FB-28: Sources already deduplicate; the spec's order and its item source differ from code

- **Spec**: `docs/sessions/U4_PASSIVES.md`; MASTERPLAN1 C04
- **Kind**: conflict
- **Status**: open
- **Finding**: `character_passives` (`check/unit.rs:103`) gathers character passives, then faction grants, then discipline passives, then named items' grants, each once; `apply_passive` and deck passives dedupe again, so no duplicate bug exists. Drills are ordinary character passive ids. The spec's order (class or discipline, character and temperament, items, faction) differs; order matters only for `foresight`'s first source, trigger order and meter floors, so reordering can move trigger-order goldens. Shipped data is nearly empty here: all 9 factions' grants and all items' grants are `[]`, 2 disciplines carry one passive each, so the 4-source test needs fixture content. Two build paths apply passives: `battle.rs:125` for NPCs, live, and `progression.rs:202` for party members, from a snapshot taken at new game (`new_game.rs:19`), so a faction grant added later never reaches an existing save. The spec's "Equipped Items (U3)" source: worn gear applies inline effects through `gear_*`, rebuilt on every sync, not passive ids; named items' `grants` are ids.
- **Recommendation**: Keep today's order, or declare order irrelevant and accept the trigger-order golden moves. Say that dedupe means one application per id, while different passives still stack. Say whether U3 gear grants ids (deduped) or inline effects (not). The single builder of FB-12 unifies the two build paths, and decides whether a party member's passives are re-gathered from sources at each build rather than snapshotted.

### FB-29: Temperaments are not built; pick one home for default intent and AI role

- **Spec**: `docs/sessions/U4_PASSIVES.md`, ROSTER1-T fold-in
- **Kind**: feasibility
- **Status**: open
- **Finding**: Nothing of ROSTER1-T exists in code (only its spec). What exists: `UnitDef.default_intent` and `ai_role`, validated and read at battle start from the unit entry. Folding in touches the schema (`characters.temperament`), the character check (the passive must be slotless), a per-character hook for the AI profile and default intent (`AiProfile::for_unit` takes a unit row, not a character), and a save key for members' temperaments. The spec says the tag goes on "the passive or character".
- **Recommendation**: Put it on the character: `temperament` naming a slotless passive, plus `default_intent` and `ai_role` keys, leaving passive rows pure. Size M, after the single builder (FB-12) and ROSTER1-L.

### FB-30: U4's slices and goldens

- **Spec**: `docs/sessions/U4_PASSIVES.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: A read-time loader turning `PassiveDef.effects` into a typed grant view, with today's accumulators kept as evaluators at first, gives identical numbers with no row rewrite; pitfalls: trigger limit keys are `source#index`, so trigger order and index must not change; `foresight`'s first source and meter floors depend on add order; meter bands diff the old accumulator shapes. Saves store only passive ids, so no migration except a new temperament key. Goldens: none move with the loader; rewriting rows or error texts moves `checks_golden` (21 passive references, the raw `sword_training` row) and the hand-copied fixture catalog; temperament fields move `checks_golden` and possibly `journey_golden`. Slices: (1) typed view and loader with a double-load test asserting old and new are equal for all 55 rows and the fixture's 16, M; (2) one source gatherer and both apply sites into the FB-12 builder, S-M; (3) temperament field and AI override, M; (4) evaluators onto U1's pipeline, L, after U1.2; (5) optional row migration, S. The validation messages (`check/simple.rs:100-210`, 12 of them) and the editor's per-type rows change when the schema does.
- **Recommendation**: Land the loader first with zero golden diffs; move evaluators onto the pipeline only after U1.2's pipeline lands.

### FB-31: Gear's family table exists already; keep `weapon_type`

- **Spec**: `docs/sessions/U3_GEAR.md` section 1
- **Kind**: conflict
- **Status**: open
- **Finding**: `item_families.json` already declares `hands`, `scaling_stats`, `off_hand_category`, `engine_rules` (always_ranged, focus, shield, spell_source, reach), per-hand base range and bands, and the loader resolves them onto `ItemDef`. There is no `weapon_kind` field: the item field is `weapon_type` (about 40 uses), also the key of class proficiency, the skill `weapon_types` gate and the passive `proficiency.gear`. `sling` is not a family. What does name families in code: `ALWAYS_RANGED_TYPES` (`gear.rs:10`, read at `:78`) duplicates the `always_ranged` rule, which only the content check reads, so a data-only `always_ranged` family with range 1 behaves as melee; `off_hand_ok` names orb and tome (`gear.rs:210`); `is_shield` reads the item type, not the `shield` rule; `skill_in_group` names staff and wand (`unit.rs:877`); `FIST` and the trained-fists rule (`damage.rs`); the art, Dojo and party-screen spots.
- **Recommendation**: Keep `weapon_type`; "migration" means making those code sites read the table's `engine_rules`. No rename.

### FB-32: "Zero Rust changes" fails as written

- **Spec**: `docs/sessions/U3_GEAR.md`, acceptance test 1
- **Kind**: conflict
- **Status**: open
- **Finding**: Besides the sites above, the content check holds fixed lists: `GEAR_TYPES` (21, `check/skill.rs:537`), `WEAPON_TYPES` (18, `check/unit.rs:22`; the two disagree), `OFF_CATEGORIES` and `FOCUS_TYPES` (`check/item.rs:19-20`, `source.rs:690`), `ENGINE_RULES` (5, `check/item.rs:28`). A new off-hand category or engine rule needs code today.
- **Recommendation**: Derive those lists from `item_families.json`, and word the test as "a new family using existing engine rules, in data only"; a new engine rule stays a code change.

### FB-33: The wield penalty already exists in a different shape; and unarmed damage would change

- **Spec**: `docs/sessions/U3_GEAR.md`, soft wield penalty
- **Kind**: conflict
- **Status**: open
- **Finding**: `wield_penalty` (`gear.rs:96`) costs 15 % of weapon power per missing stat point, capped at 60 %, plus an extra block cost at a deficit of 4 or more (whose only reader, `effective_block_cost`, has no callers); it touches weapon power only. Proficiency also adds +level weapon power (`stats.rs:179`), which the spec leaves out; level 3's only effect today, `block_cost_reduction` (`stats.rs:199`), is dead, so a level 3 mastery is new content. The spec's flat 0.75 plus 1 burden per missing point, on power and armour, also at proficiency 0. Unarmed: today weapon power is 0 unless fist proficiency is 2 or more (then 15 plus the level bonus, `damage.rs:18-31`, `:76-85`); "an implicit fist weapon at -25 % untrained" would make untrained unarmed deal 15 x 0.75 instead of 0. "No proficiency list means no restriction" (`class_can_equip`, `gear.rs:216`) must stay, or classless and hand-built units are all penalised.
- **Recommendation**: Give one formula for a stat deficit and for proficiency 0, say whether it replaces or stacks with today's ramp, add armour if wanted, keep or drop the +level power bonus, and decide unarmed explicitly; the unarmed change and the penalty change are separate commits with named golden re-records.

### FB-34: The soft penalty removes four hard gates, and the party screen must show it

- **Spec**: `docs/sessions/U3_GEAR.md`; UNITDRILL1
- **Kind**: cost
- **Status**: open
- **Finding**: Today an item a unit has no training for cannot be worn, gated in four places: `class_can_equip` (`gear.rs:216`), `UnitState::can_wear` (`state.rs:174-185`, also behind `validate` and the party pickers), the content check (`check/unit.rs:136-176`, `check/world.rs:623`) and the Dojo (`dojo.rs:1274`, `:1324`). The party pickers hide items that fail `can_wear` (`party.rs:1222`, `:2261`, `:2349`, `:2375`); soft means they all show, so the screen must display the penalty. `with_drills` stays as training. `wears_untrained` (one unit, the Purifier, and seed `cast1a4b.json`) retires. Skill gates stay hard. Only 8 shipped slots are untrained, 7 covered by drills.
- **Recommendation**: Include the party screen's penalty display in the slice (a game UI change, mock first), and say the skill gates stay hard.

### FB-35: Item-granted skills and stat changes are not reversible today

- **Spec**: `docs/sessions/U3_GEAR.md` (OFFHAND1 through `grant_skill`); `U4_PASSIVES.md` (items as a source)
- **Kind**: cost
- **Status**: open
- **Finding**: `apply_effects` (`unit.rs:463`) pushes a granted skill to `granted_skills` for good, and `stat` and `proficiency` effects are never cleared when gear changes; `rebuild_gear_effects` (`:429`) clears only potency, resist, status potency, spell reach, forced moves and triggers. Items may not carry `skill`, `stat` or `proficiency` at all (`ITEM_TYPES`, `check/simple.rs:74`). With the `deck` rule on (emberwake's row), `apply_deck` takes only owned families up to the active slots, and the world deck builder (`owned_in`, `state.rs:193`) reads known and granted skills, so a worn item's skill would be owned but not active, and invisible to the deck screen.
- **Recommendation**: One "sources recomputed on every sync" pass, shared by U3 items and U4's sources (built with FB-12's single builder), rather than a separate item list on the unit. Design calls: item skills are always active while worn and take no deck slot (as off-hand basics do); worn or held under a two-handed main.

### FB-36: Hands, budget, reach and carry differ from the built model

- **Spec**: `docs/sessions/U3_GEAR.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: (a) Hands: the spec says a two-handed weapon uses both hands; built, a two-handed main may pair with an off-hand weapon that is not a focus (`off_hand_ok`, `gear.rs:208`, with a weapon switch); only shields and foci are excluded. MASTERPLAN1 says 3 hands, the spec 2. (b) Budget: 23 +/- 20 % is a class loadout's target, not an item's (`budget.rs:19`); an item's static override already exists as `special` and `special_note`; item `tiers` are threshold unlocks, so "tier_mult" has no meaning; the weights are code constants, not data, and derived-value modifiers carry no weight. (c) Reach: `range_band` overlaps `base_range`, `effective_range` (PERC 18 adds 1), the constant `REACH_WEAPON_REACH` 2 (`unit.rs:17`), `melee_reach` and `melee_band_max` (`skill.rs:338`), which the attack lock and INTENT2's "engaged" read; U1 moves ranged reach to the skill. (d) Carry is party-wide and always on today (`carry.rs`); the spec makes it a per-unit rating, off by default. (e) "Level 1 gets tier-1 family skills" has no code hook: gating is the skill's `min_proficiency` and `weapon_types`, plus path level.
- **Recommendation**: (a) correct the spec to the built rule or change it on purpose; (b) restate the budget as the loadout's; (c) `range_band.max` replaces the `reach` rule and its constant, ranged families keep `base_range`, and name one owner; (d) carry goes with U1's speed split; (e) "data on the skill rows".

### FB-37: U3's slices and goldens

- **Spec**: `docs/sessions/U3_GEAR.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: Dropping `block_cost` touches `ItemDef`, the budget (`W_EXTRA_BLOCK`), the content check, `source.rs`, `store.rs`, the schema, 71 places in `item_families.json` and two items (`shd_tower` +1, `wpn_glass_wand` +2, whose notes promise "slow to raise"); their budgets rise by 3 and 6, which can change class budget warnings; the loader must ignore a stored key and drop it on resave. Skill and command `block_cost` is separate and untouched. Goldens: `rules_golden` runs on the frozen kit, so data edits do not move it; the penalty and unarmed changes do. `checks_golden` (43 `block_cost` lines) moves with the removal, the soft penalty and `wears_untrained`'s retirement. Slices: (1) drop `block_cost`, M, `checks_golden` re-recorded; (2) the family table drives the rules, M-L, every golden identical; (3) soft penalty, M, after (2) and FB-14, moving `rules_golden`, the timeline and checks goldens in separate commits; (4) reversible item grants, OFFHAND1 and the level 2-3 family bonuses, L, after U1.2 and U4.2; (5) per-unit carry, S, with U1's speed split.
- **Recommendation**: Answer FB-33 to FB-36 before slice 3; slices 1 and 2 can be ready-to-build now.

### FB-38: Docs delta from SKILLS1 and KITIDS1

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: open
- **Finding**: SKILLS1: planner scripts live in `tools/planner/` (README there); `tools/fc.sh test` prints each panic's message; `flush.sh` logs gate times to `$AEGIS_PLANNER_LOG`. KITIDS1: `tools/fc.sh layers` also fails when kit `src` quotes a class id from a game's `classes.json`; the sim's classes, roles, rosters and limit tests come from `emberwake_sim::CLASSES`.
- **Recommendation**: Apply to `docs/WORKFLOW.md` (fc.sh help, the planner scripts), `games/emberwake/docs/GAME_BRIEF.md` ("where to look": sim classes), `docs/STRUCTURE.md` (`tools/planner/`), and the planner and task-landing skills' script paths; TASKS: SKILLS1 and KITIDS1 done.

### FB-39: Docs delta from TESTCLEAN1

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Tests make scratch folders only through `aegis_core::scratch::Scratch` (`new`, `vacant`, `until_thread_end`), removed when the test ends; folders are named `aegis_*`. A helper that returns a derived path must hand back the guard or use `until_thread_end` (31 tests failed on a first pass that dropped it early).
- **Recommendation**: `docs/STRUCTURE.md` `aegis_core/src/` row and `engine/README.md`: add `scratch.rs`; the `rust-conventions` skill: "tests make temp folders only through `Scratch`; never `temp_dir().join`; a helper returning a path keeps its guard"; TASKS: TESTCLEAN1 done.

### FB-40: BASERULES1-M as built: three parts of the spec are left for later rows

- **Spec**: `docs/specs/BASERULES1.md` section 4.2, `BASERULES1_LIBRARY.md` sections 2 and 7; READY row 4
- **Kind**: feasibility
- **Status**: open
- **Finding**: Built to the READY row: every REGISTER rule carries `requires`, `conflicts`, `bundle`, `system`, `tested` and `does`; `FIELD_RULES` (15), `BUNDLES` (7), `TESTED_SETS` (21, library section 4 without `[poise, flank]`), `LOCKED` (50 code constants with system and reason), `PROMOTED`. One hard error, `flank` without `round_intent`; warnings for conflicts, fields set with nothing reading them, `attack_lock` with `chase=classic` (untested), a split must-ship bundle, and an untested set. Both shipped rows are clean. Not built: (a) spec 4.2's contract surface (the `compute` signature, RNG draw order, the role list; no role table exists yet), as `--contract` lists ruleset fields per READY; (b) a schema entry for `move_after_blocks`, which library section 2 says this task adds; it would make the next save rewrite shipped data, so the contract shows it as `row-only`; (c) library section 7's short fight per tested set; (d) `game.json` `ruleset` and `test_rulesets`. The sim's `rules=` stays silent, where library section 7 says `adjust` prints the same warnings. The `LOCKED` scan checks both ways but does not recurse into subfolders, skips slice types, `pub(crate)`, `static` and multi-line declarations, and matches `PROMOTED` by name only.
- **Recommendation**: Rows for (a) to (d) when their designs need them; (b) as a schema-first slice with the re-save; the sim printing notes and a recursive `LOCKED` scan are small and can ride the next base-rules slice.

### FB-41: Docs delta from BASERULES1-M

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New command `aegis rules [--contract]` (`tools/fc.sh rules`): rules by system, fields, bundles, tested sets, locked constants and the game's rows; `aegis battle rules=` prints the set's notes to stderr. A new golden, `kits/rpg/crates/rpgkit_host/tests/fixtures/rules_contract_golden.json`, recorded with `RECORD=1`.
- **Recommendation**: `CLAUDE.md` "Engine goldens": add the rules contract golden; `engine/README.md` and `docs/WORKFLOW.md`: the command; `docs/HISTORY.md`: a row for the new golden; `engine/docs/authoring/rule_library.md` (the design session's): the additive rule, the metadata, one picked set per game, the frozen-catalog test, value versus form versus new rule, and the candidates table; TASKS: BASERULES1-M done once landed.

### FB-42: Stage names differ between READY and the session notes

- **Spec**: `docs/READY.md` row 5; `docs/sessions/U1_STATS.md`, modular damage
- **Kind**: conflict
- **Status**: open
- **Finding**: READY names five stages (base, flat, mult, mitigation, round); U1_STATS names four (base, amplifiers, mitigation, finish) with flat adds in base only. U1.2-S1 follows READY: Flat right after rounding point one, Add allowed in Base and Flat, Mul in Base (tier), Mult and Mitigation, Reduce in Mitigation only, Round with no modifiers. No flat modifier exists today, so neither reading moves a number. Changing it is `Stage::ORDER` and one line in `Op::fits`.
- **Recommendation**: Name one set of stages in both docs, and say whether a flat add may sit after attack power's rounding (READY's Flat) or only before it (U1_STATS).

### FB-43: Docs delta from U1.2-S1, and take_damage could join the pipeline early

- **Spec**: shared docs; FB-05
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New `rpgkit_rules/src/pipeline.rs` (stages, `Op`, `Modifier`, `Pipeline` with `DAMAGE` and `STAGGER`, a trace per hit, `Hit.trace`) and `pool.rs` (a read-only `PoolView` over hp, ap and poise; `Unit::pool`, `World::pool`). Attack power, the hit and poise stagger run through it; `take_damage(mitigated=false)` and the intercept split stay on their own paths, as FB-05 allows. An exact port of `take_damage` is easy (Add, Reduce(guard), round to minimum 1, only when guard is above 0).
- **Recommendation**: `docs/STRUCTURE.md` `rpgkit_rules/src/` row: `pipeline.rs`, `pool.rs`, tests `damage_pipeline.rs`, `pools.rs`; `engine/docs/PORT_NOTES.md` (`damage_calculator.gd` row): runs `rpgkit_rules::pipeline`, exact port; `kits/rpg/README.md`: a U1.2-S1 line. Say whether `take_damage` joins in A2.2 or in the next U1 slice.

### FB-44: Share data cannot be stored yet: the schema rounds stats to integers

- **Spec**: `docs/READY.md` row 6 ("kit + data"); `docs/sessions/U2_CLASS.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: U2.2-S1 computes shares from today's integer rows and moved no shipped data. The schema's `stats` field type rounds values to integers, so share fields need a float-stats field type in `aegis_content` first (an engine change); adding share fields to every class row would also break the byte-for-byte resave until the data is re-saved. The writer prints floats to 15 significant digits (13/62 comes back as 0.209677419354839); the tie rounding on whole billionths absorbs it, so stats stay identical. Share keys are read as extra keys on a row, not schema fields, so the editor and the Forge do not show them.
- **Recommendation**: Move the class data to shares with the stat-weights slice: a float-stats field type, schema rows for `base_shares` and `growth_shares`, and a seed with the re-save, as one schema-first step.

### FB-45: An authored power below 1 is now raised to 1 before the multipliers

- **Spec**: `docs/sessions/U1_STATS.md`; U1.2-S1
- **Kind**: conflict
- **Status**: open
- **Finding**: In the pipeline, an authored `power` enters the base stage and is rounded to at least 1 at rounding point one, so a power of 0 or less would now be 1 before the multipliers, and the hit could pass the old floor of 1. No shipped data authors a power below 1, so nothing moves today.
- **Recommendation**: A schema minimum of 1 on authored power, or a note that it is the intended floor.

### FB-46: Free-spend allocation still reads level 2's gain

- **Spec**: `docs/sessions/U2_CLASS.md`, free allocation
- **Kind**: feasibility
- **Status**: open
- **Finding**: `auto_spend` (party screen and `unit.rs`) reads the `growth` view, which on a share row is level 2's rounded gain. That is exact for today's whole-point growth rows, but not for share rows whose growth is not whole points a level.
- **Recommendation**: When share rows ship (FB-44), `auto_spend` reads the growth shares, as `primary_stat` now does.

### FB-47: Docs delta from U2.2-S1

- **Spec**: shared docs
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New `rpgkit_data/src/unit_budget.rs` (re-exported as `rpgkit_rules::unit_budget`): the unit budget, 62 plus 5 a level, two share vectors per class, largest-remainder rounding on a running total, ties on whole billionths to the larger fraction then the earlier stat; not the gear budget `rpgkit_rules::budget`. `primary_stat` reads the largest growth share.
- **Recommendation**: `docs/STRUCTURE.md` (`rpgkit_data`): `unit_budget.rs`; `games/emberwake/docs/GAME_BRIEF.md` stats line: "class stats are base_shares x 62 + growth_shares x 5 x (level - 1), rounded by largest remainder; integer rows read as shares"; TASKS: U2.2-S1 done once on `develop`.

### FB-48: S1.2 audit: the status class

- **Spec**: `docs/sessions/S1_STATUS.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: (1) Guard values: S1 names `guard_common_mult` 0.50 and `guard_multi_round_mult` 0.30, but shipped Defends use mitigation 0.15-0.35 (guard 0.3, shield_wall 0.35, riposte 0.15) and `take_damage` multiplies by (1 - guard), so it is unclear whether these are damage multipliers or mitigation fractions; and the closed decision says the common guard is lower than the multi-round one, the reverse of 0.5 against 0.3. (2) "Wind-up start through block 6" cannot be a plain status: statuses are unit-wide and tick only at `roll_over` (`status.rs:290`); block timing exists only in `HeldGuard {mitigation, last_block}` (`trigger.rs:200-210`, `:324-333`), which already means "from block N to last_block, ends on a break or round end". Under 1C guard covers far more blocks than a Defend's own span today, so outcomes move broadly. A break must remove the status instance, not only set `guard_broken`. The AI picks Defend by skill kind and never reads `Unit.guard`, so it needs to see an active guard to avoid redundant Defends. (3) The 17 statuses are a Rust const (`status.rs:47-65`), not data; no `statuses.json`; about 25 call sites use `cap_of`, `stacking_of`, `display_name` and `row_of` without a catalog. `BLEED_PCT`, `DEBT_STAGGER`, `DEBT_STUN` are LOCKED, `COUNTER_MULT` a const, so S1's tunables must become ruleset numbers. (4) Behaviour that differs from the spec: stealth hides a unit only if it moved this round (`round_intent.rs:392-394`), not "untargetable, breaks on attack or damage"; bleed ticks at round end only (`resolver.rs:315`), not "on move or action". (5) Resist is already one formula, `potency - (INT x 2.5 + resist_for)` (`status.rs:210-213`); the trigger `resist` effect writes `status_resist` directly (`trigger.rs:726-732`), bypassing U4's combine; `resistable` is read only by the content checks, and `try_inflict` ignores it. (6) Fit to the six primitives: modifiers (weaken, empower, fortify, expose, haste, slow), ticks (poison, burn, regen, bleed), block locks (stagger, stun), reactive (riposte), rule flag (steadfast) fit; root (locks Move and stops the chase, not blocks), stealth, taunt's authoring rules and guard need special cases. Goldens: guard and 1C move the timeline and sim goldens and the rules contract; `rules_golden` holds if `Unit.guard` stays as the accumulator.
- **Recommendation**: Restate guard values as mitigation fractions per skill, and fix which guard is lower; build guard on the HeldGuard mechanism with `Unit.guard` kept, not a second mechanism; load the 17 rows as a frozen kit table behind the same function signatures first; narrow stealth and bleed to today's behaviour or schedule each change as a named golden move; route the trigger `resist` effect through U4's combine; honour `resistable` or drop it; do the KNOW potency term in U1's KNOW commit. Slices: (a) the table, M, neutral; (b) id literals to primitive reads, M, neutral (AI heuristics stay on ids); (c) tunables to ruleset numbers and `resistable`, S, rules contract only; (d) guard as a status with 1C, L, moves timeline and sim; (e) KNOW term, stealth and bleed, S each, moving goldens.

### FB-49: S2.2 audit: poise pips

- **Spec**: `docs/sessions/S2_POISE.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: (1) S2 makes a poise break queue `stagger` for next round; today a break only cancels the unit's unlanded actions, drops its guard and holds poise at 0 until refill (`poise.rs:453-488`); `stagger` is queued only by AP debt. A broken unit would lose the rest of the round and blocks 1-4 of the next: the largest balance lever in either spec, and it stacks with the 0.5 `repeat` multiplier. (2) S2 covers only the hit-to-pips step; rules built on today's continuous bar need a pip meaning: `windup_poise` above max (`poise.rs:305-327`), `unbreakable` holding at 1 or more (`:392`), break chance from taken over before (`:219`), `sure_break` (`:394`), Rest restoring poise (`:331`, `rest_poise_per_block` 5), `stagger_taken` passives, the arena `stagger` key, the trigger `stagger` effect and collision stagger. (3) With shipped stagger 4-20 (76 of 129 skills at 0) and max poise 24-90, a tank at 60 has T 12, so hits of 4-10 deal no pips; 7 skills at stagger 4-5 never matter above max poise 25 unless doubled; boss breaks likely fall (T 15-18); the timeline golden's 25 break lines will not survive. S2 says `roundi(max / 5)`, READY says `Max_Poise / 5`. (4) Pips need a state field beside `value` (the pool view is read-only); U1 adding STR to max poise changes T and every poise, so it lands separately. (5) The battle view already draws 5 pips from value over max (`marks.rs:822-836`) and forecasts from `poise::forecast_hits`; the forecast needs pip fields or the "after" is wrong.
- **Recommendation**: Confirm the break-queues-stagger rule, and ship it as its own slice. Add a decision list for (2) to S2; defaults: `windup_poise` as bonus pips, break chance from pips lost over pips before, Rest restores 1 pip per N blocks, `unbreakable` holds at 1 pip. State one formula for T. Slices: (a) pip numbers and a `pips` field, unused, S, rules contract only; (b) the floor(D / T) rule on every path behind a bar-or-pips switch, default bar, M, neutral; (c) flip timeline to pips with the decisions and real pips in the UI, M, moves timeline and sim; (d) break queues stagger, S, after S1's table, moves timeline and sim.

### FB-50: W1.2-W3.2 audit: world fights, services, clock

- **Spec**: `docs/sessions/W_WORLD.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: The fight start is `BattleScreen::world` (`emberwake_gui/src/battle.rs:677-747`); the settle is already party-first in the kit (`battle::apply`, `rpgkit_world/src/battle.rs:301-330`), so `finish_world` is mostly moving the runner dispatch and `FIGHT_SOURCE` into the kit (S); `begin_world` is the real M, as `Prebuilt` carries GUI-only art, so the kit returns units and a callback paints them. It is exact only after the single builder (READY row 10): heroes use `field_unit` and `build_played`, enemies `WorldRoster::customize`. `WorldRoster { d, world }` is a struct literal at 16 sites, so a constructor comes first. The spec's five clock causes do not match `Cause` (`Days`, `Leg`, `Battle`, `Enter`, `Effects`, act change). W3.2's clock parameters are already manifest data (`campaign`, `purse`, `rest`, `defeat`); `purse::block` returns `{}` outside a campaign world. No headless test plays a real world fight, so the round-trip acceptance test is new. SERVICES1 overlaps `start_fight` and edits the 5,764-line `region.rs`.
- **Recommendation**: Fix the cause list; drop W3.2 as a build slice (a contract listing only). Slices: (a) `WorldRoster::new` and `finish_world`, S, neutral; (b) `begin_world` with art through a callback and the round-trip test, M, neutral; (c) stage from `campaign.act` after row 10 and mob growth, S; SERVICES1, M, after (b).

### FB-51: P1.2-P2.2 audit: terrain and hazards

- **Spec**: `docs/sessions/P_SPACE.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: `grid.rs:64-72` fixes `TERRAIN_IDS` (17), `BLOCKED` and `SLOW`; `TerrainRow.walkable` exists and about 35 sites read `grid.is_walkable`, so moving `walkable` to data is S. But "no terrain index special-cased" is bigger: ids, glyphs, the slow default and zone glyph rows are index-bound, and tests index by `TERRAIN_IDS`. `stops_forced_move` has no consumer: forced moves stop on unwalkable tiles (`forced.rs:295`). The frozen kit's 17 terrain rows have no `walkable`, so the loader keeps a missing-key fallback. The store resave test reads shipped data, so schema and data land together, as does `move_after_blocks` (FB-40b), which moves the rules contract golden. `max_move_blocks` is a const (`plan.rs:23`) and the collide values are per-skill `forced_move` keys defaulting to 0, not ruleset tunables. Terrain hazards (`hazard_status`, `potency`, `magnitude`, `rounds`, `on`) and hazard rows (`status`, `potency`, `magnitude`, `status_rounds`, `damage`) use different keys; unifying renames keys, a re-save of both files and `checks_golden`, after A4.2 defines `EffectSpec`. Hazard damage runs "through guard, not armour" in `ground.rs`, so moving it to `take_damage` may move goldens.
- **Recommendation**: P1.2 stops at `walkable`, `stops_forced_move` (default: not walkable, no row sets it) and the cost default as data on the 17 rows; a dynamic terrain list is its own row; cut `max_move_blocks` and the collide values from the tunables table, or add a row to promote `MAX_MOVE_BLOCKS`. Slices: (a) walkable data with a fallback and re-save, S; (b) `move_after_blocks` schema-first, S, rules contract moves; (c) `MAX_MOVE_BLOCKS`, M, if kept; (d) unified cell effect, L, after A4.2.

### FB-52: I1.2 audit: AI

- **Spec**: `docs/sessions/I_AI.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: `ai.rs` reads the 8 status ids in about 25 places, as heuristics, not plain predicates (`debuff_order`, `ai.rs:1087-1125`, has hand-written cases for expose and slow; skill pickers match `grant_status` riposte, fortify, regen). `ROWS` (`ai.rs:1590`) is an 8-name precedence array; there is no Row Z, Maneuver, Field or ring row. The five predicate names exist nowhere, and S1's status rows have no behaviour fields yet (S1.2 unbuilt). The spec maps `modifies_rating(AttackPower)` to fortify, but S1 makes fortify an armour modifier; weaken and empower are the attack ones. A generic "score any rating modifier" would start scoring haste, weaken, empower and burn, which the AI ignores today, moving the sim and timeline goldens. Status ids are also read on AI paths outside `ai.rs` (`plan.rs:493`, `:521`, `round_intent.rs:771`, `command.rs`, `metrics.rs:1331`). The spec's `ai_role` list (tank, striker, skirmisher, ranged, support, caster, boss) differs from the schema enum (AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT), and no role table exists.
- **Recommendation**: Fix the fortify mapping; scope "zero ids" to `ai.rs` or list the other files; reconcile the role names. Slices: (a) predicates replacing the ids one to one, after S1's behaviour fields, M, neutral; (b) a generic scorer behind a ruleset flag, default off, M; (c) Row Z, Maneuver, Field and ring rows, after A3.2, A4.2 and F3, L, named timeline and sim re-record.

### FB-53: E1.2-E3.2 audit: economy

- **Spec**: `docs/sessions/E_ECONOMY.md`
- **Kind**: feasibility
- **Status**: open
- **Finding**: `SUPPLIES_MAX` and `GOLD_MAX` are consts (`state.rs:15-16`) used by `Party::add_gold`, `add_supplies` and `validate`, which take no data or world; 8 callers; the check layer has its own copies (`check/world.rs:18`, `check/map.rs:24-25`, `quest.rs:11`). The party and purse are global across worlds, so a per-world cap is ambiguous and a save loaded under a lower cap would fail `validate`; the manifest `purse` block is campaign-gated. `gold_max` already names a different thing on encounters (`check/encounter.rs:1098`). Stage-scaled XP has no source yet: XP weight is computed in `roster()` and again in `rpgkit_sim/src/runner.rs:386`. E3 (keepsakes, party size, bunks, wounds, settle) is built; E3.2 is a contract pass.
- **Recommendation**: Caps on `game.json` (one per game), clamped on load so no save breaks; name it `purse_gold_cap`. Slices: (a) `PurseCaps` read by the party methods with today's values, S; (b) the same in the checks, S; (c) stage in `roster()` and the sim, after mob growth and FB-50 (a), M.

### FB-54: T1.2-T2.2 audit: story

- **Spec**: `docs/sessions/T_STORY.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: The `stat` condition is `best_stat >= min` (`dialogue.rs:187`), where `best_stat` builds every active member on each call and takes the party's best. `condition_holds` takes the game state by shared reference and runs on every redraw (choice listing, lock text, the stat prefix, story card `when`), so a roll inside it would advance `gs.rolls` every frame, cannot mutate the state, and could be save-scummed. The spec says "the unit's stat"; today it is the party maximum. "Pools and ratings" need U1's ratings, not built yet. `dialogue_roll_die` has no home in the ruleset numbers. T2 needs little new.
- **Recommendation**: The roll is a choice-time effect (a `check` effect setting an outcome flag), and conditions read the recorded flag; `threshold` stays a pure condition. Say whose stat. Slices: (a) `threshold` mode with `threshold` as an alias of `min`, S; (b) the roll effect with a `dialogue` stream and `dialogue_roll_die`, plus a determinism and reload test, M; (c) rating reads after U1.2, S.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

### UN-01: Old test folders in the temp dir need a manual delete or a permission

- **For**: user
- **Status**: open
- **Note**: About 19,000 folders from tests before TESTCLEAN1 sit in `C:\Users\tusha\AppData\Local\Temp` (`emberwake_*`, about 18,870, and `rpgkit_*`, about 194). New test runs no longer add any. The build session's cleanup command (delete those older than 90 minutes, so nothing a running build uses is touched) was blocked by the permission check, and it does not retry denied actions.
- **Ask**: Delete them by hand, allow that command, or leave them; they are small.

### UN-02: M1 waits on the user's look at the battle view

- **For**: user
- **Status**: open
- **Note**: Battle view parts 1 and 2 (VISCLAR1-D) are on `develop`. The checklist with exact steps is `games/emberwake/docs/NEEDS_YOUR_EYES.md`, battle view section (Scene Runner > Arena > `tl_flank`, Ruleset set to Timeline).
- **Ask**: The user's look closes M1; it blocks no build work.

### UN-03: `main` is far behind `develop`

- **For**: both
- **Status**: open
- **Note**: `develop` is 111 commits ahead of `main` (release PR #54 was 2026-11-03). Releases go by PR at milestone ends or when asked; CI passes on `develop`.
- **Ask**: Whether to release `develop` to `main` now, or at M1's close.
