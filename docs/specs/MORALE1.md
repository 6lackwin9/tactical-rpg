# MORALE1: Enemy morale (groups that break)

Status: approved (user 2026-10-18 08:42); refinements R1-R3 proposed, awaiting user confirmation (direction: decision world-campaign "DR1 design review answers (user 2026-10-18)", review `docs/specs/DESIGN_REVIEW1.md` 4.3; UI wording per user 08:43)
Depends on: OBJ1-L, BOSS1-L and INTENT1-L (all on develop); MORALE1-A after MORALE1-L; MORALE1-U after INTENT1-U and OBJ1-U (same UI files)
Lane: both (MORALE1-L schema-first, then MORALE1-A data and MORALE1-S sim mapping, then MORALE1-B balance probe; MORALE1-U on the UI/UX thread). Size S-M.
Touches: L: new `scripts/combat/session/encounter_morale.gd` (parse, validate, groups, losses, state), `scripts/combat/session/{catalog_encounter,combat_encounter,combat_session,intent_rules}.gd`, `scripts/combat/arena/combat_grid.gd` (withdraw in place), the hit-result path that reads `res["is_crit"]` (`scripts/combat/actions/{attack_command,area_attack_command}.gd`, `scripts/combat/status/riposte_rules.gd`, or one session-level collector; the builder picks one and reports it), `scripts/content/{content_schema,content_catalog}.gd`, `scripts/world/{world_schema,world_catalog}.gd`, `scripts/ui/world/world_preview.gd`, new `scripts/tests/morale_test_suite.gd`, `scripts/tests/suite_registry.gd`. S: `tools/sim/sim_world.gd`, `bal2_rules_test_suite`. U: `scripts/ui/combat/combat_grid_view.gd` (state, group and leader chips), planning header, hostile cards, `scripts/ui/unit_inspector_ui.gd`, `scripts/ui/menus/battle_results_model.gd`, `combat_ui_framework_test_suite`. A: `data/worlds/ashlands/encounters.json` via one seed `data/seeds/morale1.gd`

## Goal

Goblins do not fight to the last. Each enemy group has a morale value that drops as members fall, take critical hits, or lose their leader; when it sinks to the break line the group's survivors run for the exit, or throw down their arms when they cannot get there, and once only broken enemies remain the fight ends as a win. Killing the chief first is a real tactic, crits matter a little more, and the mop-up rounds go away. The player reads each enemy's morale as a word (Steady, Shaken, Wavering, Broken), never a number. Morale is per encounter and off by default, so every fight without it plays exactly as today. Bosses, elites and berserkers never break.

## What exists (cited)

- `AiProfile.flee_ratio` (default 0.25; 0 = never) and `reckless` (never flees); `TacticalAI.build_steps` adds `PlanStep.withdraw()` at or below the ratio (`tactical_ai.gd` line 86). `CombatPlan._apply_withdraw` routes to `grid.designated_exits` ("This arena has no exit.", "No route to an exit."); a long path runs as far as it can and finishes next round. A unit on an exit tile withdraws (`combat_grid.gd`, `combatant_withdrawn`).
- Shipped minions (`goblin_brawler`, `goblin_skirmisher`, `goblin_archer`, `goblin_shaman`) have flee 0.25; `goblin_berserker` and `ambush_berserker` are reckless with flee 0 (decision classes-units "Berserker behaviour"); `goblin_warlord` (Grakka) is tier boss, flee 0. No Ashlands enemy is tier elite.
- Crits: every damage result carries `is_crit` (`attack_command.gd` `last_crit`, `area_attack_command.gd`, `riposte_rules.gd`); crits come from the battle's seeded RNG.
- OBJ1: `CombatEncounter.verdict` after each block; `EncounterObjectives.verdict(...)` rule 3 b: every counted foe down or withdrawn = won; `objective_state()` is copied into every `BlockSnapshot`.
- BOSS1: `on_block_end` marks phases due; phases fire from the next `on_block_start` or `prepare_round` `entries`; `is_reserved`, `live_enemies()`; live AI profiles are rebuilt from fired phases.
- INTENT1: `IntentRules.kind_of` (a walk to an exit is `flee`), `CombatSession.enemy_intents()`; enemies never re-plan mid-round.
- XP: `BattleBridge.apply` gives full XP for every listed enemy, wave and phase-wave unit on `won`. No kill gold or loot exists.
- Shipped rows: `ash_scouts` = `ash_skirmisher`, `ash_archer`; `ash_raiders` = `ash_brawler`, `ash_skirmisher`, `ash_shaman`; both rout, CLOSED, exit (11, 4), no waves or phases.

## 1. Schema contract (schema-first: yes)

### 1.1 Encounter fields (catalog `encounters` and world `encounters`)

| Field | Type | Default | Allowed / validation (message) |
| --- | --- | --- | --- |
| morale | bool | false | false = no morale rules at all (today's behaviour) |
| morale_break | int | 50 | 0..90: a group is Broken at or below this value ("Morale break must be 0..90.") |
| morale_leader | string (roster ref) | "" | "" = auto (rule 2); else an id in `enemies` ("Morale leader must be a unit from enemies.") |
| morale_groups | Array of Array of int | [] | [] = all of `enemies` is one group (R3). Else every inner array is non-empty ("A morale group cannot be empty."), every int is an index into `enemies` ("Morale group index out of range."), and every enemy index appears exactly once ("Every enemy must be in exactly one morale group.") |
| morale_text | string | "" | 0..80 chars ("Morale text holds at most 80 characters."); "" = the generated line of rule 9 |

The `morale_*` fields are ignored while `morale` is false (no error). One shared validator `EncounterMorale.problems(entry: Dictionary) -> Array[String]` is called from `ContentCatalog._encounter_errors` and `WorldCatalog._encounter_errors`. Constants in `EncounterMorale`: `START` 100, `CRIT_HIT` 10, `CRIT_KILL_EXTRA` 10, `LEADER_DOWN` 40; state ids `STATES` = `["steady", "shaken", "wavering", "broken"]`.

### 1.2 Runtime state

`EncounterMorale` (one per battle, built in `CatalogEncounter.build()`) keeps `groups`: Array of `{members: Array[String] (unit keys as CatalogEncounter spawns them, unit(id, n)), size: int, value: int, broken: bool, at: [round, block], cause: "leader" | "losses", announced: bool}`, plus `leader` (unit key or ""), `leader_down` (bool), `surrendering` (unit keys), `pending` (this block's loss events). `CatalogEncounter.objective_state()` returns the OBJ1 dict plus key `morale` = this state, so `BlockSnapshot` and `_restore_snapshot` need no new field. New virtuals on `CombatEncounter`: `is_broken(c) -> bool` and `morale_state_of(c) -> String` (defaults false and "").

## 2. Rules (logic lane)

1. **Off means off.** With `morale` false nothing below runs; battles, recordings and sims are identical to today.
2. **Leader.** `morale_leader` when set (`unit(id, 0)`); else the first enemy in `enemies` order whose tier is boss or elite; else none.
3. **Groups (R3).** No per-unit morale; morale belongs to a group. The starting group(s) come from `morale_groups` (default: the whole starting pack is one group). The `wave_units` wave forms one new group when it arrives, and each fired BOSS1 phase that spawns units forms one new group; every new group starts at `START` 100, untouched by earlier losses (Q3 A: late arrivals are fresh). `size` = members that count (not reserved, BOSS1 rule 4) when the group forms and never changes; a reserved member placed later joins its group at the group's current value. A group can break by itself while other groups fight on.
4. **Value and state (R1).** Every group starts at 100; the value is clamped to 0..100 and freezes once the group is broken (no rally in v1). With `B` = `morale_break`, `span` = 100 - B (integer division): Broken = value <= B; Wavering = up to B + span / 3; Shaken = up to B + 2 * span / 3; Steady = above that. At the default B 50: Steady 84-100, Shaken 67-83, Wavering 51-66, Broken 0-50 (at B 0: Steady 67-100, Shaken 34-66, Wavering 1-33, Broken 0). Only Broken changes behaviour; the other states are tells.
5. **Losses (R2).** Recorded as they resolve in the block (attacks, area attacks, ripostes and reactions from any attacker; status ticks never crit) and applied in event order at block end:
   - crit hit on a living member that does not defeat it: -10 to its group;
   - member down (defeated, or withdrawn by its own `flee_ratio` step): -`ceil(100 / size)` to its group, so half the group down always reaches 50 (the approved "half the pack is down" rule now lives in the value; see open question 1);
   - member defeated by a crit: -`ceil(100 / size)` - 10, one event that replaces the plain down loss and the crit-hit loss;
   - leader down (defeated or withdrawn): -40 to every group on the field, on top of the down loss in its own group; groups formed later are unaffected.
   With B 50 the leader's fall always breaks its own group (any size up to 11); other groups drop to Shaken or Wavering.
6. **Trigger (once per block end).** In `on_block_end`, after the BOSS1 phase check and before the verdict: apply `pending`, then every unbroken group at or below B breaks (`at` = this block; `cause` = "leader" when the leader fell this block, else "losses"). Several groups can break in the same block.
7. **Who breaks.** When a group breaks, each member that is active and on the grid breaks, except steadfast units: tier boss or elite, the slay `objective_unit`, and a unit whose live AiProfile is `reckless` or has `flee_ratio` 0 (the `TacticalAI` withdraw test; a BOSS1 `ai` phase can make a unit stand firm). Steadfast members never break, but their crits taken, deaths and (for the leader) leader loss count. Broken is permanent for the battle.
8. **What a broken unit does.** Its actions already scheduled this round still resolve (tells stay true, INTENT1). From the next enemy draft on, `CatalogEncounter.schedule_round` drafts broken units first, outside `TacticalAI`: one `PlanStep.withdraw()` and nothing else (a long route continues next round). If the withdraw step is refused (no exit, no route, rooted), the unit surrenders: no step, added to `surrendering`, and it leaves the field at block 1 start with exit bookkeeping (`withdrawn_combatants`, `Combatant.withdraw()`, `combatant_withdrawn` at its own tile). In an OPEN arena without designated exits every broken unit surrenders. Withdrawals and surrenders of broken units cost no further morale.
9. **End of battle.** `CatalogEncounter.verdict` passes the counted foes minus broken units to `EncounterObjectives.verdict`: rule 3 b wins as soon as no unbroken foe is active (Q1 A: the fight ends at that block end, no mop-up), and hold counting ignores broken units on the ring. Rule 3 a still comes first (every hero down = lost). Slay never breaks its target (won only when it falls); protect is unchanged; not-yet-arrived wave units are unbroken foes, so the battle waits for them as today.
10. **Rewards.** A break win is `won`: full encounter XP (fled and surrendered units included), the quest hook and the HEAL1 rally. Nothing extra and nothing lost.
11. **Announcement.** Per group break, the line is: the first break of the battle uses `morale_text` when set; else cause leader in the leader's group: "%s falls. The rest break and run." (leader name); else one group in the battle: "The rest break and run."; else "%s break and run." (the group's breaking names joined with "and"). Not logged when nobody in the group broke. If the battle ends at the break block, `round_summary` puts the lines in the victory status and log (`🏆 %s: <line>`). Otherwise an EVENT entry (actor = leader name or encounter name, skill "Morale", subtype "Morale", detail = the line) comes from the next `on_block_start`, or from `prepare_round` `entries` after block 6; `announced` stops repeats. Surrenders log one EVENT entry per block-1 start: "%s throw down their arms.". State changes short of Broken are not logged.
12. **Intents and threat intel (INTENT1).** `IntentRules.KINDS` gains `surrender`. In `enemy_intents()` every record gains `morale_state` (String: one of `STATES`, or "" when morale is off), `morale_group` (int group index, -1 when off), `steadfast` (bool) and `broken` (bool); the value is never in a record or the UI. A broken unit's record has `kind` `flee` when it has a scheduled walk and `surrender` (blocks (1, 1), skill "Surrender", no target) when surrendering; unbroken records keep today's kinds.
13. **Autoplay and AI.** `TacticalAI`, `PlanPresets` and AiProfile do not change; autoplay does not pause on a break.
14. **Reset and replay.** `reset_round` restores the morale state from the snapshot. Crits come from the seeded battle RNG, so the same plan gives the same losses and breaks, and recordings replay identically.
15. **Viewers.** `world_preview.gd` shows `Morale: breaks at <B> or below`, the group count when more than one, and `leader <name>` (auto leader resolved from roster tiers).

Acceptance (tests, `morale_test_suite`; a 12x9 test encounter with exit (11, 4), three flee-0.25 minions in one group, B 50, unless stated):

- Each validation message in 1.1 fires for a bad entry; a valid entry is clean in both catalogs; old entries normalise to morale false and play identically (one recorded rout battle replays with identical `RoundOutcome` records).
- Values: one plain kill leaves 66 (wavering), a crit kill 56, a crit hit 90 (steady); two kills break the group (cause losses) and the battle is won at that block end with the third standing.
- Pack of 2: the first kill ends the battle as won. B 0, no leader, no crits: two of three down does not break.
- Explicit minion leader: its fall alone breaks its group (cause leader) and wins; with two groups the other group drops by 40 and fights on; an auto boss leader is found with no `morale_leader`.
- Steadfast: a boss, an elite, a reckless unit, a flee-0 unit and the slay target never break; their deaths still lower their group; with one standing the battle goes on, and broken members' next-round plans are a single withdraw (intent `flee`).
- A broken unit that is rooted, or in an OPEN arena without exits, reports intent `surrender` and is withdrawn at block 1 with one EVENT entry.
- Mid-round break with a steadfast unit standing: the broken units' remaining actions resolve as scheduled; the EVENT entry comes at the next block start (or the next round intro after block 6).
- A wave arriving after a break or a leader loss forms a fresh group at 100 (steady) and must be beaten.
- Intent records carry `morale_state`, `morale_group`, `steadfast`, `broken` and no numeric value; with morale off they are "", -1, false, false.
- Hold: a broken unit alone on the ring does not lower `held`. Slay: minions break, the battle continues until the target is down. Every hero down in the break block is lost.
- `reset_round` after a break rewinds it, and the same plan breaks the same group at the same block.
- A break win gives the same XP award as a full rout of the same encounter.

## 3. UI hooks (MORALE1-U, UI/UX thread, layout C; UI is the user's preference, open for UI review)

- States are words with a flavour line, never numbers. Hostile card and hover card show the label and line: Steady "holding the line"; Shaken "glancing at the fallen"; Wavering "eyes on the exit"; Broken "running for the exit" (surrendering: "throwing down their arms"). Steadfast units show "Fearless: will not break" instead of a state.
- Grid: each enemy token gets a small state chip (text until art; no new asset key); when the battle has more than one group, a group letter (A, B, ...) on the chip and in the card ("Wavering, group B"). The leader token gets a "Leader" chip; a broken unit shows its `flee` chip as INTENT1-U draws it, a surrendering one a "Yield" chip.
- Planning header (Q4 A): one line under the objective text. One group, B 50: "They break when Skarn falls or half of them are down." (no leader: "They break when half of them are down."); several groups: "Each group breaks when half of it is down; Skarn's fall shakes them all."; any other B: "They break as their losses mount.". The leader's card says "Leader: its fall shakes every group"; the inspector adds "Kills and critical hits shake their group."
- Results: the morale line (rule 11) under the objective line when the battle was won by a break.
- At 1280x720 in all four text sizes the header line and chips never cover the hover card.

## 4. Assets (MORALE1-A, after MORALE1-L; Q2 A)

Via `WorldCatalog.upsert` (ashlands encounters), numbers only from this table; every other encounter keeps `morale` false.

| Row | morale | morale_break | morale_leader | morale_groups | morale_text |
| --- | --- | --- | --- | --- | --- |
| ash_scouts | true | 50 | "" | [] (one group of 2) | "The scouts scatter into the ash." |
| ash_raiders | true | 50 | ash_shaman | [] (one group of 3) | "The raiders break and bolt for the ridge." |

Why one group each: both packs are 2-3 units, the size of one team; splitting a 3-unit pack would make a single kill break a 1-unit group. Why these two rows: both are rout fights of breakable minions; `ash_scouts` is the solo fight the warrior clears only 20 of 100 times (`docs/BALANCE_REPORT.md` solo and duo probe table, issue 1: the first kill now breaks the other scout), and `ash_raiders` teaches "kill the smokecaller first" (shaman down: 100 - 34 - 40 = 26, broken). Left off, with reasons: `courtyard_skirmish` (the solo hold lesson would end at the first kill), `ash_patrol` (both reckless), `ash_straggler` (one unit), `wagon_raid` and `pass_ambush` (already at or above band: wagon_raid 100 for all ten trios, pass_ambush 70-100 for four of five duos), both stands (slay ends them at Grakka; keeps the BOSS1-B band). Descriptions gain one sentence: ash_scouts "...; drop one and the other runs."; ash_raiders "...; their smokecaller holds them together.".

Acceptance (asset, headless): `validate_all()` clean for both catalogs; a seeded ash_raiders session where the shaman falls first ends as won in that block with the line logged.

## 5. Sim mapping (MORALE1-S, Balance thread, after MORALE1-L and BAL5-SIM)

`SimWorld.encounter` with `full`: copy `morale`, `morale_break`, `morale_groups`, `morale_text`, and map `morale_leader` with `_ref` like `objective_unit`. A break win counts as cleared through `objective_wins` (world mode). Catalog scenario units keep flee 0 (steadfast), so role, sweep and duel runs never break. Test: one `bal2_rules_test_suite` check that a mapped ash_raiders entry carries the five fields.

## 6. Balance follow-up (MORALE1-B, report only)

After MORALE1-A and MORALE1-S on main: solo `comp=warrior` vs ash_scouts (mob band >= 90; today 20), the five duos and ten trios vs ash_raiders (rounds and party HP before / after; today 80-100 clear), and the campaign chain `heal=none` and `heal=25` (mean battles and party HP after B2 and B3, against BAL4-ATTR v4); also report how often a break came from crit losses alone (crits are random, seeded per run). DR1 (3): light touch; if ash_raiders falls under 2 rounds for every comp the asset levers are `morale_leader` "" or `morale_break` 40, never stats.

## Out of scope

Rallying or morale recovery, per-unit morale, behaviour for Shaken or Wavering (tells only in v1), showing the numeric value anywhere, enemies that surrender to be recruited or questioned, fled units returning across battles, morale on the party side or guests, AI that protects its leader, morale loss from statuses, fear skills or terrain, kill gold or loot, new art (chips are text), changing `flee_ratio` data, morale beyond the two rows (ASHPACE1 and DESERT1 can use the fields).

## Overlap with other specs (files shared; run in sequence, not in parallel)

- OBJ1 / BOSS1 (done): MORALE1-L edits the same `catalog_encounter.gd` functions (`verdict`, `on_block_end`, `on_block_start`, `objective_state`); no other open Gameplay task touches them.
- INTENT1-U, OBJ1-U, HOVER1-U: same UI files as MORALE1-U (HOVER1-U collapses the threat-intel block the state line sits in); MORALE1-U goes after all three.
- BAL5-SIM: `tools/sim/*`; MORALE1-S after it. ASHPACE1: same ashlands encounter rows; not at the same time as MORALE1-A. HEAL1-L: none.

## Answers (user 2026-10-18 08:42, all recommendations)

1. Only broken enemies left: A, the fight ends at once at that block end as a win (rule 9).
2. Fights with morale now: A, ash_scouts and ash_raiders, smokecaller as leader (section 4).
3. Reinforcements after a break: A, they arrive fresh (rule 3: a new group at 100).
4. Leader and rule shown before the fight: A, header line and "Leader" chip (section 3).

## Open questions for the user (R1-R3 refinements)

1. A kill costs `ceil(100 / group size)` instead of a flat -10. A flat -10 means kills alone never break a group (three kills = -30, still Steady), so the approved ash_scouts effect vanishes: one kill leaves the other scout at 90 (Steady) and it fights on; only the leader route still breaks a group.
   - (rec) A: size-scaled kill loss (rule 5). Half the group down breaks it at B 50, exactly the approved rule; crits stay a small extra (-10).
   - B: flat -10 per kill and B raised to 70. Simpler numbers, but a group of 2 needs a kill plus two crits to break, and big groups break after three kills whatever their size.
2. Leader down shakes every group by 40 but breaks only its own group (other groups need one more loss).
   - (rec) A: as written; the chief matters to everyone, but separate teams keep a fight going.
   - B: leader down breaks every group at once. Closer to the original "kill the chief" ends it, but makes groups pointless in a leader fight.
