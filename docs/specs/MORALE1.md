# MORALE1: Enemy morale (the pack breaks)

Status: proposed (spec 2026-10-18; approved direction: decision world-campaign "DR1 design review answers (user 2026-10-18)", review `docs/specs/DESIGN_REVIEW1.md` 4.3)
Depends on: OBJ1-L, BOSS1-L and INTENT1-L (all on develop); MORALE1-A after MORALE1-L; MORALE1-U after INTENT1-U and OBJ1-U (same UI files)
Lane: both (MORALE1-L schema-first, then MORALE1-A data and MORALE1-S sim mapping, then MORALE1-B balance probe; MORALE1-U on the UI/UX thread)
Touches: L: new `scripts/combat/session/encounter_morale.gd` (parse, validate, check, state), `scripts/combat/session/{catalog_encounter,combat_encounter,combat_session,intent_rules}.gd`, `scripts/combat/arena/combat_grid.gd` (withdraw in place), `scripts/content/{content_schema,content_catalog}.gd`, `scripts/world/{world_schema,world_catalog}.gd`, `scripts/ui/world/world_preview.gd`, new `scripts/tests/morale_test_suite.gd`, `scripts/tests/suite_registry.gd`. S: `tools/sim/sim_world.gd`. U: `scripts/ui/combat/combat_grid_view.gd` (leader and surrender chips), planning header, hostile cards, `scripts/ui/unit_inspector_ui.gd`, `scripts/ui/menus/battle_results_model.gd`, `combat_ui_framework_test_suite`. A: `data/worlds/ashlands/encounters.json` via one seed `data/seeds/morale1.gd`

## Goal

Goblins do not fight to the last. When the pack's leader falls, or half the pack is down, the rest break: they run for the exit, or throw down their arms when they cannot get there, and the fight ends as a win. Killing the chief first becomes a real tactic and the mop-up rounds go away. Morale is per encounter and off by default, so every fight without it plays exactly as today. Bosses, elites and berserkers never break.

## What exists (cited)

- `AiProfile.flee_ratio` (default 0.25; 0 = never) and `reckless` (never flees); `TacticalAI.build_steps` adds `PlanStep.withdraw()` at or below the ratio (`tactical_ai.gd` line 86). `CombatPlan._apply_withdraw` routes to `grid.designated_exits` ("This arena has no exit.", "No route to an exit."); a path too long for the round runs as far as it can and finishes next round. A unit on an exit tile withdraws (`combat_grid.gd`, `combatant_withdrawn`).
- Shipped minions (`goblin_brawler`, `goblin_skirmisher`, `goblin_archer`, `goblin_shaman`) have flee 0.25; `goblin_berserker` and `ambush_berserker` are reckless with flee 0 (decision classes-units "Berserker behaviour": never flee); `goblin_warlord` (Grakka) is tier boss, flee 0. No Ashlands enemy is tier elite.
- OBJ1: `CombatEncounter.verdict` after each block (after round-end effects and `on_block_end`); `EncounterObjectives.verdict(heroes, guests, protect, foes, grid, end_of_round, target)` rule 3 b: every counted foe down or withdrawn = won; `objective_state()` is copied into every `BlockSnapshot`. Slay: the target withdrawing = fled.
- BOSS1: `on_block_end` marks phases due; due phases fire from the next `on_block_start` or in `prepare_round` `entries`; `is_reserved` and `live_enemies()`; live AI profiles are rebuilt from fired phases.
- INTENT1: `IntentRules.kind_of` (a walk to an exit is `flee`), `CombatSession.enemy_intents()`; enemies never re-plan mid-round.
- XP: `BattleBridge.apply` gives full XP for every listed enemy, wave and phase-wave unit on `won`, whether it died, fled or never arrived. No kill gold or kill loot exists (decision "BATTLE1-L battle link").
- The sim maps world encounter fields to catalog entries in `tools/sim/sim_world.gd` (`full` copy list); generated sim units have flee 0.

## 1. Schema contract (schema-first: yes)

### 1.1 New encounter fields (catalog `encounters` and world `encounters`)

| Field | Type | Default | Allowed / validation (message) |
| --- | --- | --- | --- |
| morale | bool | false | false = no morale rules at all (today's behaviour) |
| morale_break | int | 50 | 0..100: the pack breaks when at least this share of it is down; 0 = no pack trigger ("Morale break must be 0..100.") |
| morale_leader | string (roster ref) | "" | "" = auto (rule 2); else an id in `enemies` ("Morale leader must be a unit from enemies.") |
| morale_text | string | "" | 0..80 chars ("Morale text holds at most 80 characters."); "" = the generated line of rule 9 |

The three `morale_*` fields are ignored while `morale` is false (no error). One shared validator `EncounterMorale.problems(entry: Dictionary) -> Array[String]` is called from `ContentCatalog._encounter_errors` and `WorldCatalog._encounter_errors`.

### 1.2 Runtime state

`EncounterMorale` (one per battle, built in `CatalogEncounter.build()`) keeps: `fired` (bool), `at` ([round, block] of the break), `cause` ("leader" or "pack"), `broken` (indices into `enemies`), `surrendering` (indices; rule 6), `announced` (bool). `CatalogEncounter.objective_state()` returns the OBJ1 dict plus key `morale` = this state, so `BlockSnapshot` and `_restore_snapshot` need no new field. `CombatEncounter.is_broken(c) -> bool` (new virtual, default false).

## 2. Rules (logic lane)

1. **Off means off.** With `morale` false nothing below runs; battles, recordings and sims are identical to today.
2. **Leader.** `morale_leader` when set (the first unit spawned from it, `unit(id, 0)`); else the first enemy in `enemies` order whose tier is boss or elite; else none (only the pack trigger).
3. **Pack.** The pack is every enemy that counts (not reserved, BOSS1 rule 4) and has been placed on the field so far (a wave or phase wave joins when it arrives). Down = defeated or withdrawn.
4. **Trigger (checked once per block end).** In `on_block_end`, after the BOSS1 phase check and before the verdict: if the break has not fired and either (a) the leader is down or withdrawn (cause `leader`), or (b) `morale_break > 0` and `down * 100 >= morale_break * pack size` (cause `pack`; leader first when both hold), the break fires. It fires once per battle, even if nobody is eligible.
5. **Who breaks.** At the moment it fires, every enemy in the pack that is active and on the grid breaks, except: tier boss or elite; the slay `objective_unit`; a unit whose live AiProfile is `reckless` or has `flee_ratio` 0 (the same test `TacticalAI` uses to withdraw; a BOSS1 `ai` phase can therefore make a unit stand firm). A minion or standard leader can break by the pack trigger. Broken is permanent for the battle. Units that arrive after the break are fresh and never break (Q3 A).
6. **What a broken unit does.** Its actions already scheduled this round still resolve (tells stay true, INTENT1). From the next enemy draft on, `CatalogEncounter.schedule_round` drafts broken units first, outside `TacticalAI`: one `PlanStep.withdraw()` in the shared enemy plan and nothing else (a long route runs as far as it can and continues next round). If the withdraw step is refused (no exit, no route, rooted), the unit surrenders: it gets no step, is added to `surrendering`, and leaves the field at the start of block 1 with the same bookkeeping as an exit step (`withdrawn_combatants`, `Combatant.withdraw()`, `combatant_withdrawn` at its own tile). In an OPEN arena without designated exits every broken unit surrenders.
7. **End of battle.** `CatalogEncounter.verdict` passes the counted foes minus broken units to `EncounterObjectives.verdict`, so a broken unit counts as out everywhere: rule 3 b wins as soon as no unbroken foe is active (Q1 A: the fight ends at that block end, no mop-up), and hold counting ignores broken units on the ring. Rule 3 a still comes first (every hero down = lost). Per objective: rout and escape win by rule 3 b; hold wins by rule 3 b or its count; slay never breaks its target, so it is won only when the target falls (minions breaking around a boss make the boss fight shorter, not over); protect is unchanged. Not-yet-arrived wave units are unbroken foes, so the battle waits for them as today.
8. **Rewards.** A break win is `won`: full encounter XP (every listed unit, fled or surrendered ones included, as `BattleBridge.apply` does today), the quest hook and the HEAL1 rally. Nothing extra for fleeing or surrendering units and nothing lost; no kill gold or loot exists to adjust.
9. **Announcement.** Line = `morale_text`, else cause `leader`: "%s falls. The rest break and run." (leader name), cause `pack`: "The rest break and run.". Not logged when nobody broke. If the battle ends at the break block, `round_summary` puts the line in the victory status and log (`🏆 %s: <line>`). Otherwise, like a BOSS1 phase (rule 2), an EVENT entry (actor = leader name or encounter name, skill "Morale", subtype "Morale", detail = the line) comes from the next `on_block_start`, or from `prepare_round` `entries` when the break fired on the last block; `announced` stops a second entry. Surrenders log one EVENT entry per block-1 start: "%s throw down their arms." (names joined with "and").
10. **Intents (INTENT1).** `IntentRules.KINDS` gains `surrender`. In `enemy_intents()`, a broken unit's record has `kind` `flee` when it has a scheduled walk (also for a partial run) and `surrender` (blocks (1, 1), skill "Surrender", no target) when it is surrendering; every record gains key `broken` (bool). Records of unbroken units are unchanged.
11. **Autoplay and AI.** `TacticalAI`, `PlanPresets` and AiProfile do not change; autoplay does not pause on a break (it is not a phase).
12. **Reset and replay.** `reset_round` restores the morale state from the snapshot; the break uses no random numbers, so the same plan breaks the pack at the same block and recordings replay identically.
13. **Viewers.** `world_preview.gd` shows `Morale: breaks at <n>% down` and, with a leader, `or when <name> falls` (auto leader resolved from the roster tiers).

Acceptance (tests, `morale_test_suite`; a 12x9 test encounter with exit (11, 4), three flee-0.25 minions unless stated):

- Each validation message in 1.1 fires for a bad entry; a valid entry is clean in both catalogs; old entries normalise to morale false and play identically (one recorded rout battle replays with identical `RoundOutcome` records).
- Break 50, pack of 3: one down does not fire; two down fires (cause pack) and the battle is won at that block end with the third still standing; with break 0 and no leader nothing ever fires.
- Pack of 2 at break 50: the first kill ends the battle as won.
- Explicit minion leader: its fall alone fires (cause leader) and the battle is won with the other two minions still standing (both broken); an auto boss leader is found with no `morale_leader`.
- Immunity: a boss, an elite, a reckless unit, a flee-0 unit and the slay target never break; with one of them standing the battle goes on after the break, and the broken units' next-round plans are a single withdraw (intent `flee`).
- A broken unit that is rooted, or in an OPEN arena without exits, reports intent `surrender` and is withdrawn at block 1 with one EVENT entry.
- Mid-round break with an immune unit standing: the broken units' remaining actions this round resolve as scheduled; the morale EVENT entry comes at the next block start (or in the next round intro after block 6).
- A wave arriving after the break is not broken and must be beaten; a pack trigger that already fired does not fire again when it arrives.
- Hold: a broken unit alone on the ring at a round end does not lower `held`. Slay: minions break, the battle continues until the target is down, then won. Every hero down in the break block is lost.
- `reset_round` after a break rewinds it (not fired, nobody broken) and the same plan fires it again at the same block.
- A break win gives the same XP award as a full rout of the same encounter.

## 3. UI hooks (MORALE1-U, UI/UX thread, layout C)

- Planning header (with morale on, Q4 A): one line under the objective text, "They break when Skarn falls or half of them are down." (pack only: "They break when half of them are down."; leader only: "They break when Skarn falls.").
- Grid: the leader token gets a small "Leader" chip (text chip until art exists; no new asset key); a broken unit's token shows its `flee` chip as INTENT1-U draws it, a surrendering one a `surrender` chip (white flag; text "Yield" until art).
- Hostile card and hover card: "Broken: running for the exit" or "Surrendering"; the leader's card says "Leader: the pack breaks when it falls".
- Results: the morale line (rule 9) under the objective line when the battle was won by a break.
- At 1280x720 in all four text sizes the header line and chips never cover the hover card.

## 4. Assets (MORALE1-A, after MORALE1-L; Q2 A)

Via `WorldCatalog.upsert` (ashlands encounters), numbers only from this table; every other encounter keeps `morale` false.

| Row | morale | morale_break | morale_leader | morale_text |
| --- | --- | --- | --- | --- |
| ash_scouts | true | 50 | "" | "The scouts scatter into the ash." |
| ash_raiders | true | 50 | ash_shaman | "The raiders break and bolt for the ridge." |

Why these two: both are rout fights of breakable minions; `ash_scouts` is the solo fight the warrior clears only 20 of 100 times (`docs/BALANCE_REPORT.md` solo and duo probe table, issue 1: morale ends it at the first kill instead of removing a scout), and `ash_raiders` teaches "kill the smokecaller first". Left off, with reasons: `courtyard_skirmish` (the solo hold lesson would end at the first kill), `ash_patrol` (both reckless, no effect), `ash_straggler` (one unit), `wagon_raid` and `pass_ambush` (already at or above the line band: wagon_raid 100 for all ten trios, pass_ambush 70-100 for four of five duos), both stands (slay already ends them at Grakka; keeps the BOSS1-B boss band). Descriptions of the two rows gain one sentence: ash_scouts "...; drop one and the other runs."; ash_raiders "...; their smokecaller holds them together.".

Acceptance (asset, headless): `validate_all()` clean for both catalogs; a seeded ash_raiders session where the shaman falls first ends as won in that block with the line logged.

## 5. Sim mapping (MORALE1-S, Balance thread, after MORALE1-L and BAL5-SIM)

`SimWorld.encounter` with `full`: copy `morale`, `morale_break`, `morale_text`, and map `morale_leader` with `_ref` like `objective_unit`. A break win counts as cleared through `objective_wins` (world mode) as today. Catalog scenario units keep flee 0, so role, sweep and duel runs never break. Test: one `bal2_rules_test_suite` check that a mapped ash_raiders entry carries the four fields.

## 6. Balance follow-up (MORALE1-B, report only)

After MORALE1-A and MORALE1-S on main: solo `comp=warrior` vs ash_scouts (mob band >= 90; today 20), the five duos and ten trios vs ash_raiders (rounds and party HP before / after; today 80-100 clear), and the campaign chain `heal=none` and `heal=25` (mean battles and party HP after B2 and B3, against BAL4-ATTR v4). DR1 (3): light touch; if ash_scouts overshoots there is nothing to fix; if ash_raiders falls under 2 rounds for every comp the asset lever is `morale_leader` "" (pack trigger only), never stats.

## Out of scope

Rallying or a morale meter (the break is one event), enemies that surrender to be recruited or questioned, captured or returning fled units across battles, morale on the party side or guests, AI that protects its leader, a break caused by statuses or fear skills, kill gold or loot for fled units, new art (chips are text until an art pass), changing `flee_ratio` data, turning morale on beyond the two rows (ASHPACE1 and DESERT1 can use the fields).

## Overlap with other specs (files shared; run in sequence, not in parallel)

- OBJ1 / BOSS1 (done): MORALE1-L edits the same `catalog_encounter.gd` functions (`verdict`, `on_block_end`, `on_block_start`, `objective_state`); no other open Gameplay task touches them.
- INTENT1-U, OBJ1-U: same UI files as MORALE1-U; MORALE1-U goes after both.
- BAL5-SIM: `tools/sim/*`; MORALE1-S after it.
- ASHPACE1: same `data/worlds/ashlands/encounters.json` rows; not at the same time as MORALE1-A.
- HEAL1-L: none (the break is a normal `won`, rally applies).

## Open questions for the user

1. When only broken enemies are left, when does the fight end?
   - (rec) A: at once, at that block end, as a win; the log and results say they ran. Mop-up disappears entirely, which is the point of the pitch.
   - B: they run for one more round (visible flee), then the win. More to watch, but it brings back one round of waiting.
   - C: the fight goes on until they leave or are killed. Most literal, but chasing gives nothing (full XP either way).
2. Which Ashlands fights get morale now?
   - (rec) A: ash_scouts and ash_raiders (smokecaller as leader). Fixes the 20% solo scouts fight and teaches "kill the chief" without touching the boss or line fights.
   - B: every rout fight with breakable goblins (adds pass_ambush and wagon_raid). Goblins feel the same everywhere, but two fights already above band get easier.
   - C: none yet (schema only; ASHPACE1 decides). No balance change, but nothing in play shows the tactic.
3. Reinforcements arriving after the break?
   - (rec) A: they arrive fresh and fight. Waves stay meaningful; the break happens once.
   - B: they join the rout and break on arrival. Kill the chief is stronger, but a wave becomes free XP.
4. Does the player see the leader and the break rule before the fight?
   - (rec) A: yes: a header line and a "Leader" chip. The tactic is a choice the player can plan for.
   - B: no, learned by play (only the log line when it happens). More surprise, but a hidden rule in a planning game reads as luck.
