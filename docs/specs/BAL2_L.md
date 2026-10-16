# BAL2-L: sim tooling, PERC range fix, unit gear `none`, elite and boss tiers

Status: spec (design-brainstorm 2026-10-17; user answers in DECISIONS "BAL2 answers (user 2026-10-17)"; awaiting approval)
Depends on: SKILL1-L (merged; it touched the same content hub files)
Lane: logic (one branch, commit order fixed)
Touches: `tools/balance_sim.gd`, new `tools/sim/*.gd`, `tools/fc.sh` (sim timeout, usage), `scripts/core/specs/character_stats.gd` (`get_effective_range`), `scripts/content/content_schema.gd` (unit gear fields, `NO_ITEM`), `scripts/content/content_catalog.gd` (ref check, `_unit_errors`, `_item_errors`, class gear check), `scripts/content/content_factory.gd` (`_equip`), `scripts/content/asset_scan.gd` (ignore `none` if it reports refs), `scripts/core/specs/stat_rules.gd` (`TIERS`), `scripts/tests/class_rules_test_suite.gd` (line 100), `scripts/tests/engine_pieces_test_suite.gd` (`_test_unarmed`), new `scripts/tests/bal2_rules_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`, README

Companion spec: `docs/specs/BAL2_SIM.md` holds the full sim contract (arguments, scores, metrics, party mode, role limit tests, composition sweep, target options). This file fixes the commit order and the engine/data-layer parts.

## Goal

Make the balance numbers trustworthy before any data is tuned: a sim that measures margins, variants, parties and role performance (commit 1); remove the engine bug that turns melee units into ranged ones at PERC 18 (commit 2); let a unit say "no item here" per slot (commit 3, unblocks BAL2-A); set the elite and boss tier values the user picked (commit 4). Commit 3 also makes two tests independent of the brawler's and warrior's shipped gear, because BAL2-A changes both.

## Commit order

| # | Commit | Spec | Unblocks |
| --- | --- | --- | --- |
| 1a | Sim core: argument parser, HP-margin score, cap 10, variants (smart_skills, loadout, spend, preset, distance), `stats` and `group` modes, continuous score in `tier` and `levels` | BAL2_SIM.md sections 1-3 | balance lane diagnostics |
| 1b | Sim metrics collector, `party` mode, `role` limit tests, `sweep` mode | BAL2_SIM.md sections 4-7 | BAL2 party and role benchmarks |
| 2 | PERC range: +1 only for ranged weapons | this file, Rules 2 | every BAL2 number |
| 3 | Unit gear sentinel `none` + gear-independent unarmed and tier tests | this file, Schema 3 and Rules 3 | BAL2-A |
| 4 | `StatRules.TIERS` elite 1.25 / 1.1, boss 1.5 / 1.1 | this file, Rules 4 | BAL2 tier table |

Commits 1a and 1b only touch `tools/`. If 1b runs long, the planner may merge 1a + 2 + 3 + 4 first and land 1b as a follow-up branch (BAL2-A needs only commit 3).

## Schema contract (logic lane)

Schema-first: yes for commit 3 (BAL2-A writes `none` values).

### 3.1 Sentinel

- New constant `ContentSchema.NO_ITEM: String = "none"`.
- Unit entries (`units` kind) only. Gear slot fields `main_hand`, `off_hand`, `head`, `body`, `accessory_1`, `accessory_2` accept: `""` (default, inherit the class's item for that slot), an item id, or `"none"` (the slot stays empty, no class fallback).
- Unit `pouch` accepts: `[]` (default, inherit the class pouch), a list of consumable ids, or exactly `["none"]` (empty pouch, no class fallback).
- Field help text for unit slots (the `_loadout_fields(inherit = true)` branch): "Empty = the class's gear; 'none' = no item." Pouch help: "Empty = the class's pouch; ['none'] = no consumables."
- `normalize` keeps `"none"` unchanged (no rewrite to `""`).

### 3.2 Validation

| Case | Result | Message (exact text) |
| --- | --- | --- |
| Unit slot `"none"` | valid (the generic ref check skips `NO_ITEM` for unit gear slots and the pouch) | - |
| Unit pouch `["none"]` | valid | - |
| Unit pouch with `"none"` and any other entry | error | `Battle pouch: 'none' must be the only entry.` |
| Class slot or class pouch `"none"` | error (a class leaves a slot blank instead) | `<Slot label>: 'none' is only for units; leave the class slot empty.` |
| Item entry with id `none` | error | `The item id 'none' is reserved.` |
| Unit `main_hand` 2H item with `off_hand` `"none"` | valid (no off-hand item) | - |
| `validate_all()` on shipped data | stays `{}` | - |

`_loadout_errors` treats `"none"` like an empty slot (no fit or training check). `AssetScan` (safe delete reference scan) must not report `"none"` as a reference; if it already skips unknown ids, no change.

## Rules (logic lane)

### Rule 1: sim (commits 1a, 1b)

Contract, formulas and acceptance checks: `docs/specs/BAL2_SIM.md`. Rules that bind this branch:

- Sim code lives in `tools/balance_sim.gd` (entry point, mode dispatch) plus helper scripts under `tools/sim/` loaded with `preload` (no `class_name`, so the game's global class list stays clean). Each file stays under ~450 lines.
- The sim never edits shipped data: everything runs on `ContentCatalog.fork_to(sandbox)` as today, and the sandbox is removed at exit.
- No engine change for metrics. Metrics come from `session.log_entries`, per-block HP/position/status sampling and the sandbox catalog. If a KPI needs data the engine does not expose, log it as `n/a` and list the gap in the builder report (do not change engine code for it).
- `tools/fc.sh`: the `sim` timeout becomes `${SIM_TIMEOUT:-900}`; the usage block lists every mode and the key=value arguments.

### Rule 2: PERC range bonus only for ranged weapons (commit 2)

- `CharacterStats.get_effective_range(base_range, context)`: `bonus = 1 if effective PERC >= RANGE_PERC_THRESHOLD and base_range > 1 else 0`; return `maxi(1, base_range + bonus)`. New constant `CharacterStats.RANGE_PERC_THRESHOLD: int = 18` (the current literal). The only caller is `Combatant.sync_from_stats_and_gear` (`base_rng` = main weapon `base_range`, 1 when unarmed); no caller change.
- Why `base_range > 1`: `Equipment.is_ranged_weapon()` is bow/crossbow/sling or `base_range > 1`; every shipped ranged weapon (shortbow 3, wand 3, staffs) has `base_range > 1`, so the range-based test matches the weapon-based one without passing the weapon in.

Acceptance (commit 2), rule-derived, in `bal2_rules_test_suite.gd`:

1. Bug confirmed before the fix. The builder runs a scratch probe on the branch base BEFORE editing (report the output): priest L7 hero, default spend: PERC 18, `wpn_cleric_mace` `base_range` 1, `attack_range` 2. The design probe on main c3520b4 found exactly this, plus rogue L7 (PERC 20, range 2) and warrior L9 (PERC 19, range 2).
2. For every class at levels 1..9 (hero, default spend, class gear): `attack_range == base + (1 if base > 1 and PERC >= RANGE_PERC_THRESHOLD else 0)`, where `base` is the main weapon `base_range` (1 if none).
3. Priest L7, rogue L7, warrior L9: `attack_range == 1`.
4. Ranger at a level with PERC >= 18 (shortbow 3): range 4. Mage L7 (PERC 18, wand 3): range 4; mage L5 (PERC 16): 3.
5. A unit with no main weapon and PERC >= 18 (hand-built, PERC raised via spend): range 1.
6. A priest L7 strike on an adjacent foe resolves as a melee hit: the damage equals `DamageCalculator.compute(priest, foe, false, false, ...)` for the same RNG seed (not the ranged path).

### Rule 3: unit gear sentinel (commit 3)

- `ContentFactory._equip(c, e, arch)`: per slot, `"none"` leaves the slot empty and skips the class fallback; `""` keeps today's fallback. Pouch: `["none"]` equips nothing; `[]` keeps the class pouch.
- Anything else that builds units from entries (`PartyBuilder`, `WorldFactory` if it passes unit slots through) gets the same meaning by going through `ContentFactory.unit`; no separate handling. World characters keep their current blank = inherit behaviour.
- Gear budget is computed for classes only and is unaffected.

Test preparation in the same commit (BAL2-A changes the brawler main hand and the warrior off-hand, so these tests must stop depending on shipped gear):

- `class_rules_test_suite.gd` line 100 (`minion.inventory.get_main_weapon().id == hero...`): compare null-safely (both null, or both non-null with equal ids).
- `engine_pieces_test_suite.gd` `_test_unarmed`: build the "armed" fighter and the `armed` budget entry from a fixture copy of the brawler class with `main_hand` set to `wpn_flanged_mace` (in memory, not saved), so the checks hold whether the shipped brawler carries a weapon or not. The check "warnings for the shipped classes stay empty" stays as it is (BAL2-A must keep the shipped classes warning-free; see BAL2_A.md section 3).

Acceptance (commit 3), rule-derived:

1. Priest unit with `off_hand: "none"`: no off-hand item; main hand = the class `main_hand`; with `off_hand: ""`: off-hand = the class `off_hand` (`wpn_tome`).
2. Unit with `pouch: ["none"]`: battle pouch empty; `pouch: []`: pouch equals the class pouch (same ids, same count).
3. Each validation row of the table in 3.2 produces exactly the listed result.
4. `validate_all()` stays `{}`; `warnings_all()` unchanged on shipped data.
5. A unit with every slot `"none"` builds with no equipment and `attack_power` uses the unarmed path (`stat_name == "STR+DEX"`).
6. `class_rules` and `engine_pieces` suites pass both on the shipped brawler and on an in-memory brawler with `main_hand: ""`.

### Rule 4: tier values (commit 4)

`StatRules.TIERS`: `elite` = `{"hp": 1.25, "power": 1.1, "label": "Elite"}`, `boss` = `{"hp": 1.5, "power": 1.1, "label": "Boss"}`; hero, minion, standard unchanged (1.0/1.0, 0.4/0.6, 0.7/0.8). Authored unit `power` stays absolute (DECISIONS). XP tier multipliers are not touched.

Acceptance (commit 4), rule-derived:

1. `tier_hp("elite") == 1.25`, `tier_power("elite") == 1.1`, `tier_hp("boss") == 1.5`, `tier_power("boss") == 1.1`.
2. For one class and level: elite `max_hp == roundi(hero_unscaled_hp * 1.25)` and elite attack power `== maxi(1, roundi(unscaled_power * 1.1))` (same formula as `DamageCalculator.attack_power`).
3. Ordering holds for HP: minion < standard < hero < elite < boss; for power: minion < standard < hero < elite <= boss (elite and boss power are equal by choice).
4. Existing tier checks in `class_rules_test_suite.gd` still pass.

## Assets (asset lane)

None in this task. BAL2-A (`docs/specs/BAL2_A.md`) needs commit 3 merged.

## Out of scope

- Any data edit (growth, gear, paths, loadouts, shaman): BAL2-A.
- Brawler unarmed compensation: options in BAL2_A.md section 6, awaiting the user; if option A is picked it becomes a follow-up logic commit (its contract is written there).
- Default loadout rule change ("newest five" in the engine): DECISIONS keeps the (level, order) rule; the sim emulates "newest" by writing explicit unit loadouts in the sandbox.
- Melee "reach" weapons (BAL2 proposal 7 B), AI behaviour changes, terrain, Forge UI for `none` (the unit form shows the raw value; a picker entry is a FORGE1 follow-up).
- XP curve and kill XP tier multipliers.

## Open questions

1. Forge unit form: should the slot picker offer "none" as an explicit choice now (small UI change in `scripts/ui/forge/*`), or wait for the FORGE1 unit tab? Recommendation: wait; the seed writes the value.
2. Should world characters (`data/worlds/*`) also accept `none`? Recommendation: not now (no world unit needs it).
