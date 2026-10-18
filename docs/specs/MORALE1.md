# MORALE1: Enemy morale (groups that break)

Status: approved (user 2026-10-18 08:42, R1-R3 confirmed with the 09:02 flee-per-state change) (direction: decision world-campaign "DR1 design review answers (user 2026-10-18)", review `docs/specs/DESIGN_REVIEW1.md` 4.3; UI wording per user 08:43)
Depends on: OBJ1-L, BOSS1-L and INTENT1-L (all on develop); MORALE1-A after MORALE1-L; MORALE1-U after INTENT1-U and OBJ1-U (same UI files)
Lane: both (MORALE1-L schema-first, then MORALE1-A data and MORALE1-S sim mapping, then MORALE1-B balance probe; MORALE1-U on the UI/UX thread). Size S-M.
Touches: L: new `scripts/combat/session/encounter_morale.gd` (parse, validate, groups, losses, state), `scripts/combat/session/{catalog_encounter,combat_encounter,combat_session}.gd` (live flee ratio, surrender, intent fields), `scripts/combat/arena/combat_grid.gd` (withdraw in place); `tactical_ai.gd` and `ai_profile.gd` do not change (rule 8), the hit-result path that reads `res["is_crit"]` (`scripts/combat/actions/{attack_command,area_attack_command}.gd`, `scripts/combat/status/riposte_rules.gd`, or one session-level collector; the builder picks one and reports it), `scripts/content/{content_schema,content_catalog}.gd`, `scripts/world/{world_schema,world_catalog}.gd`, `scripts/ui/world/world_preview.gd`, new `scripts/tests/morale_test_suite.gd`, `scripts/tests/suite_registry.gd`. S: `tools/sim/sim_world.gd`, `bal2_rules_test_suite`. U: `scripts/ui/combat/combat_grid_view.gd` (state, group and leader chips), planning header, hostile cards, `scripts/ui/unit_inspector_ui.gd`, `scripts/ui/menus/battle_results_model.gd`, `combat_ui_framework_test_suite`. A: `data/worlds/ashlands/encounters.json` via one seed `data/seeds/morale1.gd`

## Goal

Goblins do not fight to the last. Each enemy group has a morale value that drops as members fall, take critical hits, or lose their leader. As it drops the group goes from Steady to Shaken to Wavering, and each step makes its members run for the exit sooner when hurt (a higher flee ratio). When the value sinks to the break line the group is Broken: its survivors throw down their arms and leave the field at once, and once only fearless or no enemies remain the fight ends as a win. Killing the chief first is a real tactic, crits matter a little more, and the mop-up rounds go away. The player reads each enemy's morale as a word (Steady, Shaken, Wavering, Fearless), never a number. Morale is per encounter and off by default, so every fight without it plays exactly as today. Bosses, elites, berserkers and slay targets ignore morale.

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
| morale_text | string | "" | 0..80 chars ("Morale text holds at most 80 characters."); "" = the generated line of rule 12 |

The `morale_*` fields are ignored while `morale` is false (no error). One shared validator `EncounterMorale.problems(entry: Dictionary) -> Array[String]` is called from `ContentCatalog._encounter_errors` and `WorldCatalog._encounter_errors`. Constants in `EncounterMorale`: `START` 100, `CRIT_HIT` 10, `CRIT_KILL_EXTRA` 10, `LEADER_DOWN` 40, `FLEE_SHAKEN` 0.15, `FLEE_WAVERING` 0.30, `FLEE_CAP` 0.60; group state ids `STATES` = `["steady", "shaken", "wavering", "broken"]`; a unit's shown state is one of the first three or "fearless".

### 1.2 Runtime state

`EncounterMorale` (one per battle, built in `CatalogEncounter.build()`) keeps `groups`: Array of `{members: Array[String] (unit keys as CatalogEncounter spawns them, unit(id, n)), size: int, value: int, broken: bool, at: [round, block], cause: "leader" | "losses", announced: bool}`, plus `leader` (unit key or ""), `leader_down` (bool), `surrendered` (unit keys), `pending` (this block's loss events). `CatalogEncounter.objective_state()` returns the OBJ1 dict plus key `morale` = this state, so `BlockSnapshot` and `_restore_snapshot` need no new field. New virtuals on `CombatEncounter`: `morale_state_of(c) -> String` and `flee_ratio_of(c, base: float) -> float` (defaults "" and `base`).

## 2. Rules (logic lane)

1. **Off means off.** With `morale` false nothing below runs; battles, recordings and sims are identical to today.
2. **Leader.** `morale_leader` when set (`unit(id, 0)`); else the first enemy in `enemies` order whose tier is boss or elite; else none.
3. **Groups (R3).** No per-unit morale; morale belongs to a group. The starting group(s) come from `morale_groups` (default: the whole starting pack is one group). The `wave_units` wave forms one new group when it arrives, and each fired BOSS1 phase that spawns units forms one new group; every new group starts at `START` 100, untouched by earlier losses (Q3 A: late arrivals are fresh). `size` = members that count (not reserved, BOSS1 rule 4) when the group forms and never changes; a reserved member placed later joins its group at the group's current value. A group can break by itself while other groups fight on.
4. **Value and state (R1).** Every group starts at 100; the value is clamped to 0..100 and freezes once the group is broken (no rally in v1). With `B` = `morale_break`, `span` = 100 - B (integer division): Broken = value <= B; Wavering = up to B + span / 3; Shaken = up to B + 2 * span / 3; Steady = above that. At the default B 50: Steady 84-100, Shaken 67-83, Wavering 51-66, Broken 0-50 (at B 0: Steady 67-100, Shaken 34-66, Wavering 1-33, Broken 0).
5. **Fearless (09:02, 09:03).** Fearless units ignore morale entirely: no state, no flee bonus, never surrender. Fearless = tier boss or elite (user 09:03); a live AiProfile that is `reckless` or has `flee_ratio` 0 (the `TacticalAI` never-flee test; a BOSS1 `ai` phase can make a unit fearless); and, kept from the approved spec, the slay `objective_unit` (a fleeing target is an OBJ1 "fled" result, so morale must not be able to cost the slay). Fearless units still belong to a group: their crits taken and deaths cost it morale (rule 6), and a fearless leader's fall still shakes every group.
6. **Losses (R2).** Recorded as they resolve in the block (attacks, area attacks, ripostes and reactions from any attacker; status ticks never crit) and applied in event order at block end:
   - crit hit on a living member that does not defeat it: -10 to its group;
   - member down (defeated, or withdrawn off an exit by its own flee step): -`ceil(100 / size)` to its group, so half the group down always reaches 50 (the approved "half the pack is down" rule lives in the value);
   - member defeated by a crit: -`ceil(100 / size)` - 10, one event that replaces the plain down loss and the crit-hit loss;
   - leader down (defeated or withdrawn): -40 to every group on the field, on top of the down loss in its own group; groups formed later are unaffected.
   With B 50 the leader's fall always breaks its own group (any size up to 11); other groups drop to Shaken or Wavering.
7. **Trigger (once per block end).** In `on_block_end`, after the BOSS1 phase check and before the verdict: apply `pending`, update every group's state, then every unbroken group at or below B breaks (`at` = this block; `cause` = "leader" when the leader fell this block, else "losses"). Several groups can break in the same block.
8. **Shaken and Wavering: run sooner (09:02).** A non-fearless unit's effective flee ratio = min(`FLEE_CAP` 0.60, live `flee_ratio` + `FLEE_SHAKEN` 0.15 when its group is Shaken, + `FLEE_WAVERING` 0.30 when Wavering, + 0 when Steady). For the shipped minions (0.25): Steady runs at 25% HP or less (today), Shaken at 40%, Wavering at 55%: Shaken means a badly hurt goblin runs, Wavering means any bloodied goblin runs, and the cap keeps a healthy unit (above 60% HP) from fleeing on state alone; that last step is Broken's job. The bonus is layered where BOSS1 rebuilds live AI profiles (`CatalogEncounter`, after fired phases), read at each enemy draft; the existing `TacticalAI` HP check, `PlanStep.withdraw()`, exit routing and refusals ("This arena has no exit.", "No route to an exit.") run unchanged, so `tactical_ai.gd` and `ai_profile.gd` do not change (if the live profile cannot carry a per-round flee override, the builder reports it and Touches gain `tactical_ai.gd`). Enemies never re-plan mid-round, so a state reached mid-round counts from the next draft. A unit that walks off an exit is gone, as today, and costs its group the down loss (open question 1).
9. **Broken: surrender at once (09:02).** When a group breaks, each of its non-fearless members that is active and on the grid surrenders in that block end: it leaves the field at its own tile with exit bookkeeping (`withdrawn_combatants`, `Combatant.withdraw()`, `combatant_withdrawn` at its tile; no exit or route needed, so OPEN arenas work too), and its remaining scheduled actions this round are dropped like a defeated unit's. Surrenders cost no further morale. There is no withdraw-to-exit branch for Broken.
10. **End of battle.** Surrendered units are withdrawn, so OBJ1 `verdict` needs no change: rule 3 b wins as soon as no counted foe is active (Q1 A: the fight ends at the break block end, no mop-up), and hold counting no longer sees them on the ring. Rule 3 a still comes first (every hero down = lost). Slay is won only when its fearless target falls; protect is unchanged; not-yet-arrived wave units are foes, so the battle waits for them as today.
11. **Rewards.** A break win is `won`: full encounter XP (fled and surrendered units included, as `BattleBridge.apply` does for withdrawn units), the quest hook and the HEAL1 rally. Nothing extra and nothing lost.
12. **Announcement.** Per group break: the first break of the battle uses `morale_text` when set; else cause leader in the leader's group: "%s falls. The rest throw down their arms." (leader name); else one group in the battle: "The rest throw down their arms."; else "%s throw down their arms." (the surrendering names joined with "and"). Not logged when nobody surrendered. If the battle ends at the break block, `round_summary` puts the lines in the victory status and log (`🏆 %s: <line>`). Otherwise an EVENT entry (actor = leader name or encounter name, skill "Morale", subtype "Morale", detail = the line) comes from the next `on_block_start`, or from `prepare_round` `entries` after block 6; `announced` stops repeats. Steady, Shaken and Wavering changes are not logged; flee walks log as today.
13. **Intents and threat intel (INTENT1).** In `enemy_intents()` every record gains `morale_state` (String: "steady", "shaken", "wavering", "fearless", or "" when morale is off; surrendered units have no record) and `morale_group` (int group index, -1 when off); the value and the flee ratio are never in a record or the UI. Kinds are unchanged (a flee walk is `flee`, as today).
14. **Autoplay and AI.** `TacticalAI`, `PlanPresets` and AiProfile data do not change; autoplay does not pause on a break.
15. **Reset and replay.** `reset_round` restores the morale state from the snapshot. Crits come from the seeded battle RNG, so the same plan gives the same losses, flee plans and breaks, and recordings replay identically.
16. **Viewers.** `world_preview.gd` shows `Morale: breaks at <B> or below`, the group count when more than one, and `leader <name>` (auto leader resolved from roster tiers).

Acceptance (tests, `morale_test_suite`; a 12x9 test encounter with exit (11, 4), three flee-0.25 minions in one group, B 50, unless stated):

- Each validation message in 1.1 fires for a bad entry; a valid entry is clean in both catalogs; old entries normalise to morale false and play identically (one recorded rout battle replays with identical `RoundOutcome` records).
- Values: one plain kill leaves 66 (wavering), a crit kill 56, a crit hit 90 (steady), two crit hits 80 (shaken); two kills break the group (cause losses): the third surrenders at its tile in that block end, its later actions this round do not run, and the battle is won.
- Flee bonus: after two crit hits (Shaken), a minion at 35% HP plans a single withdraw (intent `flee`) next round, while the same unit in a Steady group at 35% does not; after one kill (Wavering), a minion at 50% HP plans a withdraw and one at 60% does not; a minion with flee 0.50 in a Wavering group flees at 58% HP and not at 62% (cap 0.60). In an arena without exits the flee step is refused exactly as today.
- A Wavering minion that walks off the exit costs its group `ceil(100 / 3)` (66 -> 32, broken: the last minion surrenders, won).
- Pack of 2: the first kill ends the battle as won. B 0, no leader, no crits: two of three down does not break.
- Explicit minion leader: its fall alone breaks its group (cause leader) and wins; with two groups the other group drops by 40 (Wavering) and fights on; an auto boss leader is found with no `morale_leader`.
- Fearless: a boss, an elite, a reckless unit, a flee-0 unit and the slay target plan the same in every group state, never surrender and report `morale_state` "fearless"; their deaths still lower their group; with one standing the battle goes on after the others surrender. Slay: the minions surrender, the battle continues until the target is down. An OPEN arena break surrenders the same way.
- Mid-round break with a fearless unit standing: the EVENT entry comes at the next block start (or the next round intro after block 6).
- A wave arriving after a break or a leader loss forms a fresh group at 100 (steady, flee as authored) and must be beaten.
- Intent records carry `morale_state` and `morale_group` and no numeric value; with morale off they are "" and -1. Every hero down in the break block is lost.
- `reset_round` after a break rewinds it, and the same plan breaks the same group at the same block.
- A break win gives the same XP award as a full rout of the same encounter.

## 3. UI hooks (MORALE1-U, UI/UX thread, layout C; UI is the user's preference, open for UI review)

- States are words with a flavour line, never numbers. Hostile card and hover card show the label and line: Steady "holding the line"; Shaken "glancing at the exit; runs sooner when hurt"; Wavering "eyes on the exit; runs once bloodied"; Fearless "will not break". Broken appears only in the log and results ("throws down its arms"), since a broken unit leaves the field.
- Grid: each enemy token gets a small state chip (text until art; no new asset key); when the battle has more than one group, a group letter (A, B, ...) on the chip and in the card ("Wavering, group B"). The leader token gets a "Leader" chip; a fleeing unit shows its `flee` chip as INTENT1-U draws it today.
- Planning header (Q4 A): one line under the objective text. One group, B 50: "They give up when Skarn falls or half of them are down; shaken ones run sooner." (no leader: "They give up when half of them are down; shaken ones run sooner."); several groups: "Each group gives up when half of it is down; Skarn's fall shakes them all."; any other B: "They give up as their losses mount.". The leader's card says "Leader: its fall shakes every group"; the inspector adds "Kills and critical hits shake their group."
- Results: the morale line (rule 12) under the objective line when the battle was won by a break.
- At 1280x720 in all four text sizes the header line and chips never cover the hover card.

## 4. Assets (MORALE1-A, after MORALE1-L; Q2 A)

Via `WorldCatalog.upsert` (ashlands encounters), numbers only from this table; every other encounter keeps `morale` false.

| Row | morale | morale_break | morale_leader | morale_groups | morale_text |
| --- | --- | --- | --- | --- | --- |
| ash_scouts | true | 50 | "" | [] (one group of 2) | "The last scout drops its blade in the ash." |
| ash_raiders | true | 50 | ash_shaman | [] (one group of 3) | "With the smokecaller down, the raiders throw down their arms." |

Why one group each: both packs are 2-3 units, the size of one team; splitting a 3-unit pack would make a single kill break a 1-unit group. Why these two rows: both are rout fights of minions that can flee; `ash_scouts` is the solo fight the warrior clears only 20 of 100 times (`docs/BALANCE_REPORT.md` solo and duo probe table, issue 1: the first kill now makes the other scout surrender), and `ash_raiders` teaches "kill the smokecaller first" (shaman down: 100 - 34 - 40 = 26, broken). Left off, with reasons: `courtyard_skirmish` (the solo hold lesson would end at the first kill), `ash_patrol` (both reckless), `ash_straggler` (one unit), `wagon_raid` and `pass_ambush` (already at or above band: wagon_raid 100 for all ten trios, pass_ambush 70-100 for four of five duos), both stands (slay ends them at Grakka; keeps the BOSS1-B band). Descriptions gain one sentence: ash_scouts "...; drop one and the other yields."; ash_raiders "...; their smokecaller holds them together.".

Acceptance (asset, headless): `validate_all()` clean for both catalogs; a seeded ash_raiders session where the shaman falls first ends as won in that block with the line logged.

## 5. Sim mapping (MORALE1-S, Balance thread, after MORALE1-L and BAL5-SIM)

`SimWorld.encounter` with `full`: copy `morale`, `morale_break`, `morale_groups`, `morale_text`, and map `morale_leader` with `_ref` like `objective_unit`. World-mode runs use the authored flee ratios, so the state bonus applies there. A break win counts as cleared through `objective_wins` (world mode). Catalog scenario units keep flee 0 (fearless), so role, sweep and duel runs are unchanged. Test: one `bal2_rules_test_suite` check that a mapped ash_raiders entry carries the five fields.

## 6. Balance follow-up (MORALE1-B, report only)

After MORALE1-A and MORALE1-S on main: solo `comp=warrior` vs ash_scouts (mob band >= 90; today 20), the five duos and ten trios vs ash_raiders (rounds and party HP before / after; today 80-100 clear), and the campaign chain `heal=none` and `heal=25` (mean battles and party HP after B2 and B3, against BAL4-ATTR v4). Also report per run: units fled while Shaken or Wavering, units surrendered, and how often a break came from crit losses alone (crits are random, seeded per run). DR1 (3): light touch; if ash_raiders falls under 2 rounds for every comp the asset levers are `morale_leader` "" or `morale_break` 40, never stats; if flee cascades end fights before any real fighting, the logic lever is `FLEE_WAVERING` 0.20 (a planner-approved constant change, not data).

## Out of scope

Rallying or morale recovery, per-unit morale, state behaviour beyond the flee ratio (no hesitation, retreat to cover or skipped turns), showing the numeric value or flee ratio anywhere, enemies that surrender to be recruited or questioned, fled or surrendered units returning across battles, morale on the party side or guests, AI that protects its leader, morale loss from statuses, fear skills or terrain, kill gold or loot, new art (chips are text), changing `flee_ratio` data, morale beyond the two rows (ASHPACE1 and DESERT1 can use the fields).

## Overlap with other specs (files shared; run in sequence, not in parallel)

- OBJ1 / BOSS1 (done): MORALE1-L edits the same `catalog_encounter.gd` functions (`on_block_end`, `on_block_start`, `objective_state`, the live AI profile rebuild); no other open Gameplay task touches them.
- INTENT1-U, OBJ1-U, HOVER1-U: same UI files as MORALE1-U (HOVER1-U collapses the threat-intel block the state line sits in); MORALE1-U goes after all three.
- BAL5-SIM: `tools/sim/*`; MORALE1-S after it. ASHPACE1: same ashlands encounter rows; not at the same time as MORALE1-A. HEAL1-L: none.

## Answers (user 2026-10-18)

1. Only broken enemies left (08:42): A, the fight ends at once at that block end as a win (rule 10).
2. Fights with morale now (08:42): A, ash_scouts and ash_raiders, smokecaller as leader (section 4).
3. Reinforcements after a break (08:42): A, they arrive fresh (rule 3: a new group at 100).
4. Leader and rule shown before the fight (08:42): A, header line and "Leader" chip (section 3).
5. R1-R3 (09:02): confirmed, with the size-scaled kill loss and "leader down shakes every group, breaks only its own" as proposed; changed so Shaken and Wavering raise the flee ratio (rule 8), Broken surrenders at once (rule 9), and boss, elite (09:03) and reckless units ignore morale (rule 5); "other refinements were fine" (09:02). UI states are flavour words only (08:43).

## Open question for the user

1. A minion that flees off the exit costs its group the down loss, so one runner can push the rest from Wavering to Broken (a cascade; acceptance case 4).
   - (rec) A: it counts, as already approved ("down = defeated or withdrawn"): a comrade running is the classic morale blow, and the Wavering chip warns the player first.
   - B: only defeats and crits count. Flee plays out one unit at a time, fights last a little longer, and the cascade risk goes away.
