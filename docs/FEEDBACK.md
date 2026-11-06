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
| `TESTCLEAN1` | built, in its partial gate | `claude/testclean1`, lane 1 | One scratch helper removing its folder on drop. |
| `SKILLS1` | staged | `staging` | Review passed; reaches `develop` with the next full gate. |
| `KITIDS1` | built, review passed, small fixes running | `claude/kitids1`, lane 2 | No golden moves; `fc.sh layers` gains a class-id check; FB-23, FB-24. |
| `BASERULES1-M` | building | `claude/baserules1m`, lane 3 | |
| `U1.2` (`U1_STATS`) | audited, slice 1 queued | - | FB-04 to FB-09 closed. `U1.2-S1` follows `BASERULES1-M`. |
| `U2.2` (`U2_CLASS`) | audited, slice 1 queued | - | FB-10 to FB-15 closed. `U2.2-S1` follows `U1.2-S1`. |
| `A4.2` (`INTENT2`) | audited | - | FB-16 to FB-22 closed; design updating INTENT2. |
| `U3.2` (`U3_GEAR`) | auditing | - | Read-only audit running. |
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
