# UI2-L: party screen, level-up draft with preview, battle results with a skippable level-up prompt

Status: spec (design-brainstorm 2026-10-17; DECISIONS progression-save "UI menus baseline answers (user 2026-10-17)", "Level-up spend", "M3 progression and save"; docs/specs/UI_MENUS_RULES.md 2.4-2.6; awaiting approval)
Depends on: UI1-L (router, `CurrentGame`, hub); EXP1-L for the live battle hookup (`BattleBridge.apply` result); works without it on the award dict of `LevelUp.award_encounter`
Lane: logic only (no asset part)
Touches: new `scripts/ui/menus/{party_model,stats_model,level_up_model,loadout_model,gear_model,battle_results_model}.gd` + views `*_view.gd`, new `scripts/ui/menus/party_gear_bridge.gd`, `scripts/ui/character_equipment_modal.gd` (decoupling only if needed, rule 6), new `scripts/tests/party_screen_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`

## Goal

The party screen shows and edits the roster, the active four, stats with player-allocated level-up points, gear, skills and loadout, and passives; the battle results screen shows XP and level-ups and offers a skippable level-up prompt (the party screen allows it any time out of combat). Every edit is a draft validated by the existing rule APIs; nothing re-implements a game rule.

## Schema contract (logic lane, schema first: no; no data or save change)

### Models

| Model | Exposes | Actions (return `Array[String]` problems) |
| --- | --- | --- |
| `PartyModel` | members `[{id, name, class, level, active, unspent}]`, `active_count`, `badge` (any unspent) | `toggle_active(id)`, `select(id)` |
| `StatsModel` | per stat `{base, growth, spend, total}`, derived block (2.1), XP `{total, level, to_next}` | - |
| `LevelUpModel` | `points_left`, `draft: {stat: int}`, `before` / `after` derived blocks | `add(stat)`, `remove(stat)`, `auto()`, `confirm()`, `cancel()` |
| `LoadoutModel` | owned families grouped `class` / `extra` with `{family, grade, tiers: [{tier, unlock_level, castable}]}`, slots `class[5]`, `extra[3]`, basics list (read only) | `slot(family, group, index)`, `unslot(group, index)`, `use_default()`, `confirm()`, `cancel()` |
| `GearModel` | per slot item id and name, stash list with counts | `equip(slot, item_id)`, `unequip(slot)`, `confirm()`, `cancel()` |
| `BattleResultsModel` | rows `[{id, name, xp, old_level, new_level, unspent}]`, `rewards_text` (quest rewards granted this battle, if any), `prompt_level_up` (any row with `new_level > old_level` and `unspent > 0`) | `level_up_now(id)` (push the level-up screen), `skip()`, `continue_on()` |

### Derived block (one function: `PartyPreview.derived(state: UnitState) -> Dictionary`)

Built from `PartyBuilder.build(state.duplicate_state())` with unspent points LEFT unspent on the copy (no auto-spend in the preview, so the before/after shows only confirmed and drafted points): `max_hp`, `speed`, `move_per_block`, `crit_pct` (`DamageCalculator.crit_chance`), `melee_power` and `ranged_power` (`DamageCalculator.attack_power(c, false)` / `(c, true)` `attack_power`), `armor_rating` (`DamageCalculator.armor_rating`), `debuff_resist` (`Combatant.debuff_resistance`), `ap`.

## Rules (logic lane)

1. Drafts: every editing model works on `CurrentGame.state.party.member(id).duplicate_state()` (and a copy of `PartyState.stash` for gear); `confirm()` validates and writes back into `CurrentGame.state`; `cancel()` drops the copy. `MenuRouter.has_draft` is true while a copy differs from the original (UI1-L rule 10).
2. Active four: `toggle_active` edits `PartyState.active_ids`; adding a fifth: `"The active party holds at most 4."`; removing the last: `"Keep at least one active member."`; the first edit on an empty `active_ids` first writes the implicit first four explicitly.
3. Level-up allocation (DECISIONS "Level-up spend"; HISTORY row 14): `points_left = state.unspent() - sum(draft)`; `add` refused at 0 left (`"No points left."`), `remove` refused at 0 drafted for that stat; `auto()` fills the draft with `StatRules.auto_spend(class growth, points_left)`; `confirm()` calls `LevelUp.confirm(state, draft)` and returns its errors verbatim; confirmed points are permanent (no respec). Unspent points may stay: the battle builder auto-spends them on a copy (M3), so deferring is allowed and the party badge stays on.
4. New skills after a level-up: `LevelUpModel.new_tiers(old_level)` = tiers castable at the current level but not at `old_level`, computed by building two copies (`total_xp = XpCurve.xp_for_level(old_level)` vs current) and comparing `Combatant.tier_unlocked` per owned family tier; shown as a list with "Open loadout" when a slot is free.
5. Loadout: slot limits `StatRules.MAX_CLASS_SLOTS` (5) and `MAX_EXTRA_SLOTS` (3); basics never slottable (HISTORY row 31); `use_default()` clears `UnitState.loadout` to `{"class": [], "extra": []}` (empty = default rule); `confirm()` writes the draft and returns `UnitState.validate()` loadout messages verbatim. No AP cap or skill cap shown (SK2; HISTORY row 28). Locked tiers show their unlock level.
6. Gear: `GearModel` edits `UnitState.equipment` (slot -> item id; unlisted = class default, `ContentSchema.NO_ITEM` is not offered, DECISIONS balance "BAL2 role answers" (7)) and moves items between the member and the stash copy; `confirm()` returns `UnitState.validate()` gear messages verbatim (proficiency, hands, requirements). The view reuses `CharacterEquipmentModal` through `PartyGearBridge` (builds the member's Combatant with `PartyBuilder.build`, a stash `Inventory` from the stash copy, and maps `equipment_updated` back to the model). If the modal cannot run without a `DojoSession`, the builder adds the smallest decoupling (an optional session) to the modal; if that is more than a small change, stop and report (feedback rule). Gear budget stays a Forge-only warning.
7. Passives tab: read only, `UnitState.passives` with display names; source text only when a source records it (none does today: show "-").
8. Battle results: input is the award dict (`LevelUp.award_encounter` result: member id -> `{xp, old_level, new_level, unspent, levels_gained}`), from `BattleBridge.apply(...)["xp"]` once EXP1-L is merged. Benched members are not listed; downed active members are (M3 rule). `continue_on()` autosaves (`SaveGame.autosave(CurrentGame.state)`, `CurrentGame.mark_saved(0)`) and returns to the caller; `skip()` = `continue_on()` without opening level-up; `level_up_now(id)` opens the level-up screen for that member and returns to results afterwards. The prompt never blocks: skipping keeps the points (rule 3).
9. Gold: when `PartyState` has `gold` (DLG1-L), the party header shows it; read only here.

### Commits

1. Models, `PartyPreview`, bridge, suite.
2. Views and hub wiring (`HubModel.open("party")`), battle results scene hook in the exploration scene's fight end (or the test arena runner until EXP1's scene calls it).

## Acceptance checks (rule-derived; `party_screen_test_suite.gd`)

1. For a member with 2 unspent points, `add(STR)` twice then `add(STR)` returns "No points left."; `confirm()` makes `state.spend.STR` +2 and `unspent()` 0; an equal draft passed to `LevelUp.confirm` on a second copy gives the same result (equivalence).
2. `cancel()` leaves the member equal (`UnitState.equals`) to before; `has_draft` false.
3. `before`/`after` derived blocks differ exactly as `PartyPreview.derived` of the confirmed state (max HP after +1 CONS equals the before value + 8 x tier HP factor, derived from `StatRules` rather than a literal).
4. `auto()` drafts the same points `LevelUp.auto_for` would spend on a copy.
5. Toggling a fifth active member and removing the last one give the rule 2 messages; the first toggle on empty `active_ids` writes the first four.
6. Slotting a sixth class family is refused with the `UnitState.validate` message; a basic is never in the slottable list; `use_default()` gives an empty loadout and the default rule's families in the preview.
7. `new_tiers(4)` for a member now level 5 equals the castable-tier difference computed by the rule, and is empty when no unlock happened.
8. Gear: equipping an item the class cannot use returns the validator message; a valid swap moves the old item to the stash copy and back on cancel.
9. Battle results from a fixture award dict list only active members; `prompt_level_up` is true exactly when a member gained a level with unspent points; `continue_on()` writes slot 0 and `CurrentGame.is_dirty()` is false afterwards.
10. Existing progression, save and dojo suites pass unchanged.

## Out of scope

Respec, talent trees, passive slot selection, item crafting or durability, shop and gold spending (DLG1), journal (UI3-L), the combat HUD, tooltips and art pass, gamepad.

## Open questions

None (UI menus answers: skippable prompt after battle plus party screen any time).
