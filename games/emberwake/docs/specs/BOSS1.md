# BOSS1: Boss phases from data triggers (Grakka's finale)

Status update (2026-10-20): approved and built; the rows are done (docs/archive/TASKS_DONE.md); PHASEFOLD (out of scope here) approved 2026-10-20. The status line below is the original.

Status: proposed (spec 2026-10-18; approved direction: decision world-campaign "DR1 design review answers (user 2026-10-18)" (all recommendations), review `games/emberwake/docs/specs/DESIGN_REVIEW1.md` 5.6)
Depends on: CAMP1-A2 (ashmaw_stand data on develop); nothing new for the logic part
Lane: both (BOSS1-L schema-first, then BOSS1-A data, then BOSS1-B balance probe; BOSS1-U is a small UI/UX slice)
Touches: L: new `scripts/combat/session/encounter_phases.gd` (parse, validate, fire), `scripts/combat/session/{combat_encounter,catalog_encounter,combat_session,combat_autoplay}.gd`, `scripts/combat/arena/block_snapshot.gd` (`phase_state`), `scripts/content/{content_schema,content_catalog}.gd` and `scripts/world/{world_schema,world_catalog}.gd` (field `phases`), `scripts/exploration/battle_bridge.gd` (`enemy_levels`), `scripts/audio/music_director.gd` (`combat_context` reads phase waves), `scripts/ui/world/world_preview.gd` (one summary line), new `scripts/tests/boss_phase_test_suite.gd`. U: `scripts/ui/combat/*` (phase banner). A: `data/worlds/ashlands/encounters.json` (two rows), one seed `data/seeds/boss1.gd`

## Goal

Grakka's stand at the Ashmaw Camp gets a second act. When he drops to half health he tears off his crown of nails, his smokecallers start chanting, and he fights differently from the next round on. Any encounter can carry such phases as data: a trigger (a unit's HP, or a round) and a short list of effects (a line, a wave, a buff, an AI change). There are no scripted commands; every unit still plans with the normal planner and AI.

## What exists (cited)

- One reinforcement wave per encounter: `wave_units`, `wave_round`, `wave_block`, `wave_label` (`content_schema.gd` and `world_schema.gd` `_encounter_fields`). Wave units are spawned in `CatalogEncounter.build`, kept off the field, and placed by `_place_leftovers` when `on_block_start` matches the round and block; the AI plans them from the next round.
- `CombatSession._step_resolution` calls `encounter.on_block_start(round, block)` before each block and logs the returned EVENT entry; it calls it again until it returns null. After the last block it runs `resolver.apply_round_end_effects()`, then `resolver.is_combat_resolved(allies, enemies)`.
- `_begin_round` calls `encounter.prepare_round(round)` (intro, one `entry`) before `encounter.schedule_round`, which drafts every enemy with `TacticalAI.build_steps` (`combat_encounter.gd` `schedule_enemy_ai`). Enemies never re-plan mid-round (INTENT1 relies on this).
- AI levers per unit: `AiProfile` `role`, `target_pref`, `reckless`, `flee_ratio`, `defend_below`, `heal_below` (`ai_profile.gd`), built per combatant in `CatalogEncounter._spawn` into `ai_profiles`.
- Buffs: `StatusEffects.grant_now(target, status, rounds, magnitude, source)`; buff ids haste, taunt, riposte, fortify (flat 1..20), regen (1..10 % max HP) (`status_registry.gd`).
- `BlockSnapshot` already stores `reinforcements_spawned`; `reset_round` restores the round-start snapshot.
- ashmaw_stand: grakka_ashmaw, ash_shaman, ash_archer, wave at round 2 block 1 (ash_berserker, ash_brawler, ash_skirmisher), max_rounds 8; ashmaw_stand_barred: same without the wave (`data/worlds/ashlands/encounters.json`). goblin_warlord: warrior class, level 6, tier boss, flee_ratio 0 (`data/catalog/units.json`).

## How this avoids the dropped hand-scripted layer

HISTORY "Hand-scripted combat layer" was dropped because scripted commands could not express planning, AP and AI, and it doubled maintenance. Phases issue no commands and move no unit. They only change inputs between blocks or rounds (who is on the field, a registry buff, an AiProfile field), and every unit then plans through `TacticalAI` and the player's planner as usual. There is one code path: the phase wave reuses the existing wave placement, and the existing `wave_*` fields keep working unchanged.

## 1. Schema contract (schema-first: yes)

### 1.1 Encounter field `phases` (catalog `encounters` and world `encounters`)

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| phases | json (array of phase objects) | [] | rules 1.2 to 1.4; empty = no phases |

### 1.2 Phase object

| Key | Type | Default | Allowed / validation (message) |
| --- | --- | --- | --- |
| id | string | (required) | snake_case, unique in the encounter ("Phase ids must be unique: '%s'.") |
| name | string | (required) | 1..40 chars; shown as the EVENT entry's skill name ("Phase '%s' needs a name.") |
| trigger | object | (required) | 1.3 |
| effects | array | (required) | 1..5 effects, 1.4 ("Phase '%s' needs 1 to 5 effects.") |
| pause_autoplay | bool | true | rule 7 |

At most 4 phases per encounter ("An encounter holds at most 4 phases.").

### 1.3 Trigger

| kind | Keys | Validation (message) |
| --- | --- | --- |
| hp_below | unit (roster ref), pct (int) | unit is an id in `enemies` ("Phase '%s': trigger unit '%s' is not in enemies."); pct 1..99 ("Phase '%s': pct must be 1..99.") |
| round | round (int) | 2..`max_rounds` ("Phase '%s': round must be 2..%d.") |

A roster ref is an id as written in the encounter's lists (a catalog unit id in catalog encounters, a world character id in world encounters). For `hp_below`, the trigger unit is the first unit spawned from that id (`CatalogEncounter.unit(id, 0)`).

### 1.4 Effects

| type | Keys (default) | Validation (message) |
| --- | --- | --- |
| line | text | 1..160 chars; at most one per phase ("Phase '%s': one line per phase.") |
| wave | units (refs), label ("") | refs resolve with the enemy filter of `wave_units`; at most one per phase; `enemies` + `wave_units` + every phase wave together hold at most 12 units ("Encounter enemies, waves and phase waves hold at most 12 units.") |
| status | unit (roster ref), status, rounds (2), magnitude (0) | unit in `enemies`, `wave_units` or a phase wave of this encounter; status is a buff (`StatusEffects.buffs()`) ("Phase '%s': '%s' is not a buff."); rounds 1..5; magnitude inside the registry range, 0 only for statuses without a range |
| ai | unit (roster ref), any of role, target_pref, reckless, flee_ratio, defend_below, heal_below | unit as for status; role in AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT; target_pref in NEAREST, WEAKEST; reckless bool; the three ratios 0.0..1.0; at least one key ("Phase '%s': an ai effect needs at least one setting.") |

Unknown keys or types are errors ("Phase '%s': unknown effect '%s'."). One static validator `EncounterPhases.problems(entry: Dictionary, ref_ok: Callable) -> Array[String]` is used by `ContentCatalog._encounter_errors` and `WorldCatalog._encounter_errors` (each passes its own ref check).

### 1.5 Runtime state

`EncounterPhases` keeps, per battle: `fired` (phase id -> [round, block], block 0 = round start), `due` (ids met but not yet fired) and `dropped` (ids that can no longer fire). `BlockSnapshot.phase_state: Dictionary` copies all three; `_restore_snapshot` restores them and rebuilds every AI profile (rule 5). No save change: battles are not saved.

## 2. Rules (logic lane)

1. **Checking triggers.** After each block resolves (after round-end effects on the last block, before the end-of-battle check), `CombatEncounter.on_block_end(round, block)` (new virtual, default no-op) marks an `hp_below` phase due when its trigger unit is alive and `hp * 100 <= pct * max_hp`. A trigger unit that is down or withdrawn before its phase fired drops the phase. A `round` phase is due at the start of its round.
2. **When a phase fires (Q1 A).** A phase met at the end of block b < `total_blocks` fires from `on_block_start(round, b + 1)`, one EVENT entry per call, before the existing wave check. A phase met at the end of the last block, and every `round` phase, fires in `prepare_round` of the next round (or of its round), before `schedule_round`; `prepare_round` returns these entries under a new key `entries` (Array[ActionLogEntry]) that `_begin_round` logs after `entry`. Several due phases fire in phase list order.
3. **Effects, in this order.** `line`: the EVENT entry's detail (actor = the trigger unit's name, or the encounter name for `round`; skill = phase name; subtype "Phase"). `wave`: the units are placed like the existing wave (`_place_leftovers` on the enemy half); placed mid-round they have no plan until the next round (INTENT1 shows them as `arriving`); placed at round start they are drafted that round. `status`: `grant_now` on every unit of that id that is alive and on the field, source `phase:<id>`; rounds count the current round (as for riposte). `ai`: rule 5.
4. **Reserved units.** Units of a phase wave are spawned in `build()` (registered with the resolver like the existing wave) but are reserved until their phase fires: `CombatEncounter.is_reserved(c) -> bool` (default false). The session's end-of-battle check and `BattleBridge.outcome` skip reserved units, so a battle can be won while a phase never fired. Units of a dropped phase stay reserved for the rest of the battle.
5. **AI changes.** Each spawned unit keeps its base profile; its live profile = the base with the `ai` settings of every fired phase applied in fire order. The live profile is used from the next enemy draft on (never mid-round), so intent tells stay true. After a snapshot restore the live profiles are rebuilt from `fired`.
6. **Reset and replay.** `reset_round` restores the round-start snapshot, which already holds the phases fired at that round's start; `prepare_round` re-logs their entries without applying them again. Phases fire from state only (no random numbers), so recordings replay identically.
7. **Autoplay (Q3 A).** When a phase with `pause_autoplay` true fires, `CombatAutoplay` pauses at the next planning phase with the reason "%s: %s" (trigger unit or encounter name, phase name). It is a battle-only pause like the safety stops; the saved switch stays on.
8. **XP and music.** `BattleBridge.enemy_levels` adds the units of every phase wave (like `wave_units`, they count whether or not they arrived). `MusicDirector.combat_context` also reads phase waves for a boss-tier unit.
9. **Viewers.** `world_preview.gd` shows one line per phase: `Phase <name>: <trigger> -> <effect types>`.

Acceptance (tests, `boss_phase_test_suite`):

- Validation: each message in 1.2 to 1.4 fires for a bad entry; a valid two-phase entry is clean in both catalogs.
- A test encounter with `hp_below` 50 on one enemy: when a block leaves it at 50% or less, the next block starts with the phase EVENT entry; its buff is active in that block; its AI change shows in the next round's `resolver.scheduled_actions`, not in the current round's.
- Crossing at the end of the last block fires at the next round start, before the enemy draft.
- A `round` 3 phase fires in round 3's intro; its wave units have scheduled actions in round 3.
- The trigger unit killed from 60% in one block drops the phase; the battle ends as won with its phase wave never placed.
- `reset_round` after a phase fired in that round rewinds it (not fired, profiles back to base) and replaying the same plan fires it again at the same block; a phase fired at the round start is shown once after the reset.
- The existing wave fields behave as before (current wave tests stay green).
- Autoplay pauses after a phase with `pause_autoplay` true and keeps running for one with false.
- A recorded battle with phases replays with identical `RoundOutcome` records.

## 3. UI hook (BOSS1-U, UI/UX thread)

When a phase fires, a centred banner shows the phase name and line for about 2 s (skipped by any key, never during replay scrubbing), using the COMBAT-UI2 victory popup style. The log ticker already shows the EVENT entry. No new art.

## 4. Assets (BOSS1-A)

Both rows via `WorldCatalog.upsert` (ashlands encounters); nothing else changes.

### 4.1 ashmaw_stand `phases`

| id | name | trigger | effects | pause_autoplay |
| --- | --- | --- | --- | --- |
| crown_of_nails | Crown of Nails | hp_below grakka_ashmaw 50 | line "Grakka tears the nail-crown from his head and hurls it into the fire. The smokecallers begin to chant."; status grakka_ashmaw fortify, rounds 3, magnitude 5; status grakka_ashmaw regen, rounds 3, magnitude 4; ai grakka_ashmaw reckless true, target_pref WEAKEST; ai ash_shaman heal_below 0.9 | true |

### 4.2 ashmaw_stand_barred `phases`

| id | name | trigger | effects | pause_autoplay |
| --- | --- | --- | --- | --- |
| crown_of_nails | Crown of Nails | hp_below grakka_ashmaw 50 | same five effects as 4.1 | true |
| empty_gorge | The Empty Gorge | round 4 | line "Grakka bellows at the gorge. Nothing answers." | false |

Descriptions: ashmaw_stand ends "...at Round 2; at half strength Grakka throws off his crown."; ashmaw_stand_barred ends "...cut off the flankers; at half strength Grakka throws off his crown.".

Acceptance (asset, headless): `validate_all()` clean for both catalogs; a seeded ashmaw_stand session in which Grakka drops to 50% logs "Crown of Nails" and Grakka shows fortify and regen.

## 5. Balance follow-up (BOSS1-B, report only)

Party of three (warrior + two recruits, the BAL4-ATTR comps) at the campaign level vs both stands, before and after BOSS1-A. Target: the boss band 30-50 (`games/emberwake/docs/decisions/balance.md`); BAL3-R0 has the party boss archetype at 100. DR1 (3) says do not tune much: if the phase pushes a stand below 30, the asset fix is magnitude first (fortify 5 to 3, regen 4 to 2), never stats. The sim runs catalog encounters only (`tools/sim/sim_args.gd` `catalog_encounters`); BOSS1-B uses whatever path CAMP1-B uses for world encounters, and reports if there is none.

## Out of scope

Debuffs or damage on the party from a phase, phases that remove units or end the battle, chained conditions (phase B only after phase A), triggers on the party side, enemies re-planning mid-round, moving the existing `wave_*` fields into `phases` (a later cleanup row), new art or sound, enemy morale (MORALE1), a "slay the chief" win (OBJ1), any sim tool change.

## Overlap with other specs (files shared; run in sequence, not in parallel)

- INTENT1-L: `combat_session.gd` (`enemy_intents`); reserved units are off the grid, so they give no record; `arriving` covers phase waves. Land either first.
- ITEMUSE1-L: `combat_session.gd` (`_restore_snapshot`), `block_snapshot.gd`. Both add snapshot fields: merge carefully.
- AUTO1: `combat_autoplay.gd` (new pause reason only).
- OBJ1-L: `catalog_encounter.gd`, `combat_encounter.gd`, `combat_session.gd`, `block_snapshot.gd`, both schemas and catalogs, `battle_bridge.gd`. BOSS1-L first, then OBJ1-L (OBJ1 reuses `is_reserved`).
- RECRUIT1-L: `world_schema.gd`, `world_catalog.gd` (different functions). RECRUIT1-A: `data/worlds/ashlands/encounters.json` (different rows; same file, so not at the same time as BOSS1-A).

## Questions for the user

1. When does a phase take effect?
   - (rec) A: right away, before the next block (like today's wave), with AI changes from the next round. The moment lands when Grakka is hit, and tells stay true.
   - B: only at the start of the next round. Simplest, but a phase met in block 1 waits up to five blocks.
2. What does Grakka's second phase do?
   - (rec) A: a line, two buffs (fortify 5, regen 4 for 3 rounds), Grakka goes reckless for the weakest hero and the smokecallers heal at 90%; no new units. A new feel with a small balance change, the same in both stands.
   - B: A plus two more smokecallers in the unbarred stand only. A bigger climax and a sharper steel choice, but more tuning risk for a party of three.
   - C: only the line and the AI change. Smallest balance impact, weakest moment.
3. Does autoplay stop when a phase fires?
   - (rec) A: yes, it pauses at the next planning phase (per phase, default on). The player sees the boss change before the next round is played for them.
   - B: no, autoplay carries on. Fewer clicks, but the climax can pass unseen.
