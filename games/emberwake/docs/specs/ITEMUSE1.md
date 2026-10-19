# ITEMUSE1: Use pouch items in battle as a planner step

Status: proposed (spec 2026-10-18; approved direction: review `games/emberwake/docs/specs/DESIGN_REVIEW1.md` 3.1, decision world-campaign "DR1 design review answers (user 2026-10-18)")
Depends on: AUTO1-L logic (merged on develop, save v7), COMBAT-UI2-L1 for the timeline row look (the item step uses the same row)
Lane: logic first (ITEMUSE1-L, schema-first), then a small asset slice (ITEMUSE1-A); the sim key lands with L
Touches: L: `scripts/progression/{unit_state,party_builder}.gd` (`pouch`), `scripts/game_state/{save_game,save_migrations}.gd` + `tools/gen_save_fixtures.gd` + new `scripts/tests/fixtures/saves/save_v8.json`, `scripts/exploration/game_encounter.gd` (write-back), `scripts/content/item_fields.gd` (`use_reach`), `scripts/combat/planning/{plan_step,combat_plan,plan_option}.gd`, new `scripts/combat/actions/item_command.gd`, `scripts/combat/arena/block_snapshot.gd`, `scripts/combat/session/combat_session.gd` (`_restore_snapshot`), `scripts/combat/ai/{ai_profile,tactical_ai}.gd` (`use_items`, off by default), `scripts/ui/menus/gear_model.gd` (pouch draft), `scripts/playtest/playtest_recorder.gd` + replayer (step mode), `tools/sim/sim_args.gd` (`pouch=`), new `scripts/tests/item_use_test_suite.gd`. A: `data/catalog/items.json` (three rows, `use_reach` only)

## Goal

Potions, elixirs and bombs bought from Hest or found in chests can be used in a real battle. Each hero carries a small pouch filled before the fight. In planning, "Use item" is one more step on the hero's timeline; it costs a block and 1 AP and uses the item up when it resolves. Undo and round reset give the item back. Autoplay and enemies never use items on their own; the sim can, behind a key.

## What exists (cited)

- `Inventory.battle_pouch` (max 3, `max_pouch_slots`), `add_to_battle_pouch`, `use_pouch_item(index, target)`; `Consumable.apply_to_combatant` heals, deals `damage_amount` through `take_damage`, or calls `apply_attack_boost` (`scripts/core/inventory/{inventory,consumable}.gd`).
- Only the Dojo uses a pouch item (`scripts/dojo/dojo_session.gd` `use_pouch_item`); `CombatPlan` has no item mode (`PlanStep.Mode`: ENGAGE, MOVE, GOTO, AREA, WITHDRAW, SELF, ALLY).
- Party heroes have no pouch: `UnitState.carried` goes to `inventory.items`, not the pouch (`party_builder.gd`). Units take a pouch from the unit row (`pouch` field, `content_factory.gd` `_equip`); no shipped class or enemy has one.
- Shipped consumables (`data/catalog/items.json`): `con_heal_potion` (HEALTH_POTION, heal 25), `con_focus_elixir` (FOCUS_INCENSE, STR +50% for 2 rounds), `con_fire_bomb` (THROWABLE_BOMB, damage 20); all `use_blocks` 1, `max_stack` 5.
- Enemy plans are drafted and committed at round start (`CombatEncounter.schedule_enemy_ai`), before the party plans; `reset_round` restores the round-start `BlockSnapshot`.

## 1. Schema contract (schema-first: yes)

### 1.1 `UnitState.pouch` and save version 8

| Name | Value | Notes |
| --- | --- | --- |
| `var pouch: Array[String]` | [] | consumable item ids in slot order, at most `Inventory.max_pouch_slots` (3); repeats allowed; `to_dict` key `pouch`, missing reads [] |

`UnitState.validate`: unknown id or not a consumable ("Pouch: '%s' is not a consumable."); more than 3 ("Pouch holds at most 3 items."). `SaveGame.VERSION` 8; step `_v7_to_v8` adds `pouch: []` to every party member, nothing else; v8 fixture as in TRAV2-L 1.2 (older fixtures byte-identical).

### 1.2 Item field `use_reach` (consumables)

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| use_reach | int | -1 | -1 (= type default below) or 0..4; 0 = self only |

Type defaults (`Consumable.reach_for(type)`): HEALTH_POTION 1 (self or an adjacent ally), FOCUS_INCENSE 0 (self), THROWABLE_BOMB 3 (one foe). STAMINA_BREW and ANTIDOTE have no shipped item and are not offered in battle (`ContentCatalog.warnings_for` notes it).

### 1.3 Plan step

`PlanStep.Mode.ITEM` with `item_id: String` and `target: Combatant` (self, ally or foe by type). `PlanStep.use_item(item_id, target)` builds it. Recorded rounds store `{"mode": "item", "item": id, "target": <unit key>}`.

### 1.4 AI switch

`AiProfile.use_items: bool` = false. Only the sim sets it (rule 9).

## 2. Rules (logic lane)

1. **Filling the pouch.** Out of battle only, on the Character popup Gear tab (UIR-3): moving a consumable from the stash into a pouch slot takes one from the stash; removing returns it. `GearModel` drafts and confirms like gear. `PartyBuilder` adds the pouch ids to `inventory.battle_pouch` in order.
2. **Offer.** `CombatPlan.options(c)` lists one `Use %s (%d left)` (item name, count) option per distinct pouch item whose type is offered (1.2) and whose count is not used up by this round's earlier item steps. Self items plan at once; potion and bomb options open target picking (`target_options`) limited to valid targets.
3. **Cost and timing.** 1 AP (AP debt allowed as for skills) and `use_blocks` blocks at the lowest free blocks, like a SELF step. No auto-move: the target must be within `use_reach` (measured like skill reach, `CombatPlan.reach`) of the user's planned tile when the step starts; else the step is invalid ("Out of reach for %s.").
4. **Resolution (`ItemCommand`).** Resolves in the block slot where `SupportCommand` resolves. Heal: `Combatant.heal`, capped at missing HP. Bomb: flat `damage` on the target, ignoring armor, guard and stance, like round-end damage; it is not a hit, so `DamageCalculator` is untouched and riposte never counters it. Elixir: `apply_attack_boost(1 + buff_pct, duration)`; it replaces an active Focus boost (same `focus_attack_boost` slot), never stacks.
5. **Consumed only when used.** The item leaves `battle_pouch` when its command resolves. If the user is down, stunned or withdrawn, or the target is down or out of reach at that block, the command is cancelled, the item stays and the log says why ("Healing Draught not used: Sella is out of reach.").
6. **Plan edits and undo.** Adding, removing, moving or undoing a step never touches the pouch (nothing is used before resolution). `BlockSnapshot` gains `pouches` (combatant -> item ids); `_restore_snapshot` rebuilds `battle_pouch` from them, so `reset_round` and `reset_battle` give used items back.
7. **After battle.** On every outcome (won, fled, lost) `GameEncounter` writes each member's remaining `battle_pouch` ids back to `UnitState.pouch`. Used items are gone; a fled or downed hero keeps what it did not use.
8. **Autoplay (AUTO1).** Party tactics draft with `use_items` false, so autoplay never adds an item step. In a round paused by the player or a safety stop, the player may add item steps; the next autoplayed round drafts fresh without them.
9. **AI and sim (Q2 A).** With `use_items` true, `TacticalAI` adds at most one item step per round, first in the sequence: a heal potion on itself when `TacticalAI.incoming_damage` leaves it at or below 35% max HP by round end and no friendly heal is planned on it; else a bomb on a foe in reach whose HP is at or below the bomb's damage. The elixir is never AI-used. Enemies keep `use_items` false (enemy pouches stay unused). Sim key `pouch=<id,id,id>` gives every party unit that pouch and sets `use_items` true; the default (no key) keeps every current sim number identical.

## 3. Acceptance checks (tests)

- A hero with pouch [con_heal_potion, con_heal_potion] gets one option "Use Healing Draught (2 left)"; after one planned use the option shows (1 left); after two it is gone.
- A potion step heals 25 (capped) in its block; the pouch drops to one; `reset_round` restores two and the HP.
- A bomb on a foe 3 tiles away deals exactly 20 whatever the foe's armor, guard or stance; at 4 tiles the step is invalid.
- A potion whose ally target is downed before its block is not used; the pouch keeps it.
- The elixir sets the attack boost to 1.5 for 2 rounds and replaces a Focus boost.
- After a won battle with one potion used, `UnitState.pouch` has one fewer; after a loss the same rule holds.
- Autoplay over a sample easy encounter adds no item step; the same seed with and without pouches gives the same result.
- Sim: `pouch=con_heal_potion,con_heal_potion` lets a unit at 30% HP drink; without the key, party results equal the last run.
- Save v7 loads as v8 with empty pouches; a v8 save round-trips the pouch.

## 4. Assets (asset lane, ITEMUSE1-A)

| id | use_reach | Note |
| --- | --- | --- |
| con_heal_potion | 1 | pass the bottle to a neighbour |
| con_focus_elixir | 0 | self |
| con_fire_bomb | 3 | thrown |

Values equal the type defaults; writing them makes the rows explicit for the Forge view. No price or stock change (Hest's shop stays as CAMP1 and TRAV2 left it).

## Out of scope

Using items outside battle (exploration healing is HEAL1's question), enemy item use, area bombs, new consumable types (antidote, stamina), item art, a battle report of items used (BREPORT1), changing the full heal after battle (HEAL1).

## Questions for the user

1. Where do battle items come from?
   - (recommended) A: a per-hero pouch of 3, filled from the stash before the fight. A real preparation choice and it reuses the existing pouch model.
   - B: any hero may use any consumable in the stash during battle. No preparation, but the stash becomes an unlimited bag in every fight.
   - C: the pouch refills itself from the stash after each battle. Less menu work, less choice.
2. Who may use items without the player?
   - (recommended) A: nobody in play; only the sim behind `pouch=`. Autoplay's 30% safety stop already hands the moment to the player, and BAL4-ATTR can still model potions.
   - B: autoplay too (the same potion rule in every tactic). Fewer stops, but it spends bought items silently.
   - C: enemies as well (pouches on some Ashlands enemies). More texture, more AI and balance work.
