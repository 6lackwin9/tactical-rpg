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
| `TESTCLEAN1` | review passed, landing | `claude/testclean1`, lane 1 | `aegis_core::scratch::Scratch`; 81 test files converted; docs delta FB-39. |
| `SKILLS1` | staged | `staging` | Review passed; reaches `develop` with the next full gate. |
| `KITIDS1` | staged | `staging` | Review fixes in; reaches `develop` with the next full gate; FB-23, FB-24, docs delta FB-38. |
| `BASERULES1-M` | building | `claude/baserules1m`, lane 3 | |
| `U1.2` (`U1_STATS`) | slice 1 building | `claude/u12s1`, lane 2 | FB-04 to FB-09 closed. Exact pipeline port and pool facade, goldens byte-identical. |
| `U2.2` (`U2_CLASS`) | audited, slice 1 queued | - | FB-10 to FB-15 closed. `U2.2-S1` follows `U1.2-S1`. |
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
- **Note**: `develop` is about 90 commits ahead of `main` (release PR #54 was 2026-11-03). Releases go by PR at milestone ends or when asked; CI passes on `develop`.
- **Ask**: Whether to release `develop` to `main` now, or at M1's close.
