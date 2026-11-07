# PHASEFOLD: the single reinforcement wave becomes an arrival phase

Status: approved (user 2026-10-22: open questions 1 A, a wave in the Phases JSON, a Forge phase editor later; 2 A,
the two partial parity retirements; 3 none, no idea rows). Was: proposed (spec 2026-10-22; direction: user 2026-10-22 "spec first, behaviour kept": every battle plays exactly
as now and no recorded parity trace changes)
Depends on: BOSS1-L (done); PHASEFOLD-A (data) depends on PHASEFOLD (logic)
Lane: both (PHASEFOLD logic, schema-first; then PHASEFOLD-A, one `aegis content` seed)
Touches: logic: `games/emberwake/schema.json` (encounters, both sections), `emberwake_battle/src/{encounter,session}.rs`,
`emberwake_content/src/check/{encounter,world,reward}.rs`, `emberwake_content/src/{source,store}.rs` (the fold on load,
upsert and resave), `emberwake_world/src/battle.rs` (`roster`), `emberwake_sim/src/{scenarios,world,runner}.rs`,
`tools/worldbook.py` (line 123), tests: `emberwake_world/tests/{data_source,store,rewards}.rs`, a new
`emberwake_world/tests/phasefold.rs` and its fixture. Data (PHASEFOLD-A): `games/emberwake/data/catalog/encounters.json`,
`data/worlds/{ashlands,tidewrack}/encounters.json`, new seed `data/seeds/phasefold.json`

## Goal

An encounter has two ways to bring enemies in mid-fight: the old single wave (`wave_units`, `wave_round`, `wave_block`,
`wave_label`) and BOSS1's phase `wave` effect. PHASEFOLD removes the four fields and expresses the single wave as one
phase with a new trigger kind, `arrival`, so there is one code path, one validator and one place to author
reinforcements. The arrival phase reproduces the old wave exactly: same units, same spawn order and names, same moment,
same log cell and event line, same win rule and morale group. Nothing new becomes possible for players; it is a cleanup.

Note on the task row wording: `wave_block` is the block number the wave enters before, not a condition that blocks it.
Nothing blocks the wave today; the "barred" Ashmaw stand is a separate encounter row picked by the quest and dialogue
(`data/worlds/ashlands/quests.json` 459, `dialogues.json` 468).

## What exists (cited, develop at 0353423)

Single-wave semantics, W1 to W12 (session = `emberwake_battle/src/session.rs`, encounter = `.../encounter.rs`):

- W1 Fields and defaults: `wave_units` refs (enemy filter, dupes), `wave_round` 1 (1..30), `wave_block` 4 (1..10),
  `wave_label` "Reinforcements arrive!" (schema.json content and world `encounters`; `EncounterDef::from_obj`
  session 98-101).
- W2 Spawn: resolvable refs only (session 253); spawned at battle start after `enemies` and before every phase-wave unit
  (session 333-342); display names numbered over party, enemies, wave, phase waves, guests (session 270-284).
- W3 Off the field until it enters; not deployed (session 383-385 deploy only `deployed`).
- W4 Moment: in `on_block_start` (session 980-1008) when `round == wave_round && block == wave_block`, once
  (`reinforcements_spawned`), only if at least one unit spawned (`self.wave.is_empty()` returns false).
- W5 Order inside a block start, one item per step: every due phase first (982-985, list order), then each morale break
  line (986-991), then the wave (992-1007). The block resolves on the following step.
- W6 Placement: `place_leftovers(&wave, false, deployment)` (1001), the free tile nearest the arena middle on the enemy
  half (decision combat "GRID12").
- W7 Draft: the round's enemy draft runs in `begin_round` (445-446) before any block, on active units on the field only
  (816). A wave placed at block >= 1 has no plan that round (INTENT1 "arriving") and is drafted from the next round.
- W8 Win rule: wave units are never reserved (`is_reserved` false, session 621-623; only phase-wave units are reserved,
  338-342). `verdict` (1058-1064) counts them as foes, and `Objectives::verdict` (encounter 69) needs no active foe to
  win; an alive unit off the field is active (`emberwake_rules/src/unit.rs` 662). So a battle is not won while the wave
  is still to come; it can be lost, fled, or won by slay or hold.
- W9 Morale (when `morale` is on): on entry the wave forms one group, source "wave", size = every wave unit
  (`form_morale_group` 782-796; none reserved). Its units have unit keys from battle start (355-368).
- W10 Log: one EVENT cell `Entry::event(round, block, "Reinforcements", false, wave_label, "Wave", "ARRIVED",
  "<placed names, comma-joined> join the battle.", "<placed count> more hostiles")` and the event line
  `"<wave_label>: <names> join the battle."` (1003-1006). No banner, no autoplay pause.
- W11 Rewards and rosters: roster order `enemies`, `wave_units`, then each phase's wave units in phase and effect
  order; keys `ref#n` count earlier entries of the same ref (`emberwake_world/src/battle.rs` 138-162, `Tally` 187-194,
  `check/reward.rs` 105-118). `fates` lists a wave that never came as standing (session 627-654). Saves store paid keys
  in `reward_paid` (save v12).
- W12 Validation (only when `wave_units` is non-empty): "The wave block is past the end of the round (N blocks)." and
  "The wave round is after the last round." (`check/encounter.rs` 49-56, `check/world.rs` 440-447), placed after the
  party and enemy count messages; unknown refs give the field message "Reinforcements: unknown unit 'x'." (catalog) or
  "Reinforcements: unknown character 'x'." (world). The 12-unit cap counts enemies, wave and phase waves (146-170).

How a phase wave differs today (BOSS1, encounter 216-344, session 656-714):

| Topic | Single wave | Phase wave (`wave` effect) | Arrival phase (this spec) |
| --- | --- | --- | --- |
| Trigger | round + block | hp_below (+or_round) or round start | round + block (`arrival`) |
| Slot in a block start | after due phases and morale lines | first (due list) | after due phases and morale lines |
| Drafted that round | no (block >= 1) | yes at round start, no mid-round | no (block >= 1 only) |
| Counts as a foe before it comes | yes | no (reserved, can stay back on a win) | yes (never reserved) |
| Morale group | source "wave", all units | source `phase:<id>`, reserved count | source "wave", all units |
| Log cell | actor "Reinforcements", "Wave", "ARRIVED" | trigger unit or encounter, "Phase", "PHASE" | as the wave |
| Autoplay pause, banner | none | per `pause_autoplay` | none |

## 1. Schema contract (logic lane, schema-first: yes)

### 1.1 Removed fields

`wave_units`, `wave_round`, `wave_block`, `wave_label` leave both `encounters` sections of
`games/emberwake/schema.json`. `EncounterDef` loses its four `wave_*` fields; `Session` loses `wave` and
`reinforcements_spawned`. The writer never writes them again. They stay readable through the fold (1.4).

### 1.2 Trigger kind `arrival`

| Key | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| kind | "arrival" | (required) | new allowed kind next to hp_below and round |
| round | int | 1 | 1..`max_rounds`; above: "The wave round is after the last round."; below 1: "Phase '%s': round must be 1..%d." |
| block | int | 4 | 1..`total_blocks`; above: "The wave block is past the end of the round (%d blocks)."; below 1: "Phase '%s': block must be 1..%d." |

Any other trigger key (unit, pct, or_round) is the existing "Phase '%s': unknown trigger key '%s'.".

### 1.3 Arrival phase rules

A phase whose trigger kind is `arrival` is an arrival phase. Shape and checks, besides the existing id and name rules:

| Rule | Message |
| --- | --- |
| Effects: exactly one `wave` effect, nothing else | "Phase '%s': an arrival holds one wave and nothing else." |
| At most one arrival phase per encounter | "An encounter holds at most one arrival." |
| It is the first phase of the list (fixes spawn order, rule 2.2) | "Phase '%s': the arrival comes first in the phase list." |
| `pause_autoplay` absent or false | "Phase '%s': an arrival never pauses autoplay." |
| Does not count toward "at most 4 phases" | (the existing message counts the other phases only) |
| Wave label: text, any length; default "Reinforcements arrive!" | existing "Phase '%s': the wave label must be text." |
| Wave units: each a known enemy ref | the old field messages "Reinforcements: unknown unit '%s'." / "Reinforcements: unknown character '%s'." and the old field's filter message, not "Phase '%s': wave unit '%s' is not an enemy." |
| Unit cap 12 (enemies plus every phase wave, arrival included) | existing "Encounter enemies, waves and phase waves hold at most 12 units." |

Message order: the round/block messages come where W12 put them (after the party and enemy count messages, before the
other phase problems); the ref messages come where the field errors came. The removed `wave_units` field descriptor
(label "Reinforcements", ref units or characters, enemy filter, dupes) is kept as a constant in the check module and
also serves as the ref filter of every phase wave (today read from the schema, `check/encounter.rs` 58,
`check/world.rs` 449).

### 1.4 The fold (legacy input to arrival phase)

One pure function `encounter::fold_legacy_wave(entry: &mut Obj)` in `emberwake_battle` (content, world and sim already
depend on it). It runs on every encounter before anything reads it: `EncounterDef::from_obj`, the content loader
(`source::load`), `store` upsert validation and resave, `check::validate_entry` and `check::world::validate_entry`,
`Data::load`. Steps:

1. If none of the four keys is present, return.
2. Read `units` = strings of `wave_units`, `round` = int `wave_round` (default 1), `block` = int `wave_block` (default
   4), `label` = string `wave_label` (default "Reinforcements arrive!"), with the same conversions `from_obj` uses.
3. If `phases` is present and not an array, return unchanged (the row is already invalid: "Phases must be a list of
   phase objects.").
4. Remove the four keys. If `units` is empty, return (every row without a wave just loses its default keys).
5. Insert at index 0 of `phases` (created if absent):
   `{"id": "reinforcements", "name": "Reinforcements", "trigger": {"kind": "arrival", "round": R, "block": B},
   "effects": [{"type": "wave", "units": [...], "label": "..."}], "pause_autoplay": false}`.
   If the id is taken, use "reinforcements_2", "_3", and so on.

The sim builds the same object through `encounter::arrival_phase(units, round, block, label)`, which the fold also uses.

### 1.5 Phases help text

The `phases` help in both sections gains: `{"kind": "arrival", "round": 1, "block": 4}`: a reinforcement wave that
enters before that block, counts as an enemy from the start and holds one wave effect; at most one, listed first.

## 2. Rules (logic lane)

1. **Parse.** `parse_phases` parses an `arrival` trigger into kind, round, block; `PhaseDef::is_arrival()`.
2. **Spawn.** Phase-wave units spawn in phase list order as now; the arrival is first, so its units spawn right after
   `enemies`, as the wave did (W2), and names, unit ids and `units_by_ref` are unchanged. Arrival units are tracked in
   `Phases.wave_units` but never put in `reserved` (W8).
3. **Never due.** `mark_round`, `check_block_end` and `next_due` ignore arrival phases; they never drop.
4. **Firing.** `on_block_start` keeps its order: a due phase, else one morale line, else the arrival phase whose round
   and block match the current ones, is not fired, and has at least one spawned unit (W4, W5). Firing places its units
   with `place_leftovers` (W6), forms the morale group with source "wave" (W9; size counts every arrival unit), appends
   the W10 cell with actor = the phase `name` and skill = the wave label, pushes the W10 event line, then marks the
   phase fired at (round, block). No other effect, no banner, no autoplay pause.
5. **Draft.** Unchanged: block >= 1, so arrival units are drafted from the next round (W7).
6. **Boss-phase readers skip arrivals:** autoplay pause and banner (when ported), pace metrics, enrage and AI profile
   rebuilds (no ai or enrage effects anyway), any "phase fired" count, previews that list boss phases. Readers of all
   enemies (rosters, `pace_of`, `enemy_levels`, the music boss check, the 12-unit cap) include them, as they included
   `wave_units`.
7. **Sim.** Generated encounters with a wave (`scenarios.rs` 158-165, WAVE_ROUND 2, WAVE_BLOCK 1) emit an arrival
   phase. Catalog and world encounters (`scenarios.rs` 173-180, `sim/world.rs` 167-174) copy the arrival phase only,
   units mapped through `catalog_ids` / `map_refs`, as they copied the wave fields; the other phases stay out unless
   `full` (`sim/world.rs` 187-188), as now. `runner.rs` 238-241 collects units from phases only.
8. **Worldbook.** `tools/worldbook.py` 123 notes "reinforcements" when an encounter has an arrival phase.

## 3. Roster keys, rewards, saves

The roster becomes `enemies`, then every phase wave in list order (the `wave_units` step disappears). Since the arrival
is first, the ref sequence is identical, so every key `ref#n`, every kill weight, every `reward_paid` entry and every
`Tally` stays the same. No save field mentions waves: the save version stays 12, no migration step. Battles are not
saved mid-fight; playtest recordings replay commands against the data and give the same states.

## 4. Proof of equivalence and parity

The Godot recordings are the old behaviour and are never edited. Their inputs carry the legacy fields, so they now run
through the fold and the arrival path:

| Suite | Input with a wave | After PHASEFOLD |
| --- | --- | --- |
| `emberwake_battle/tests/trace_parity.rs` | `traces_encounters_l1.json`, goblin_ambush (3 seeds, legacy `base`) | must match unchanged |
| `emberwake_world/tests/world_traces.rs` | parity exports: pass_ambush; ashmaw_stand with escape, centre, morale on, hold, start buffs; 4 presets x 2 seeds | must match unchanged |
| `emberwake_sim/tests/sim_parity.rs` | generated `wave` archetype (`parity/out/sim`) | must match unchanged |
| `emberwake_world/tests/journey.rs`, `saves.rs` | exports and save fixtures | must match unchanged |
| `emberwake_content/tests/checks_parity.rs` | the wave cases (W12 messages, "Reinforcements:" refs) | must match unchanged (1.2, 1.3 keep the messages) |
| `emberwake_world/tests/data_source.rs` | engine-loaded shipped data vs Godot `catalog.json`, `worlds.json`, `schema.json` | retired in part |
| `emberwake_world/tests/store.rs` (resave) | Godot's `resave/after` encounters files | retired in part |

Retirements (named in the commit and a HISTORY row, following the AFTER_GODOT pattern): `data_source.rs` folds the
Godot side's encounter rows with `fold_legacy_wave` and drops the four schema rows before the diff; `store.rs` compares
the writer's encounters files with Godot's `after` file parsed, folded and printed by the same document writer (every
other file stays byte for byte). No trace, export or expectation file is edited.

Shipped data (the exports match it today: goblin_ambush, pass_ambush and ashmaw_stand have the same wave fields and
phases in `parity/out/resave/before` and `data/`) is proven by a golden recorded with today's engine:

1. First step of PHASEFOLD, before any code change: a recorder (`phasefold.rs` with `RECORD=1`) writes
   `emberwake_world/tests/fixtures/phasefold_golden.json`: for goblin_ambush (catalog) and pass_ambush, ashmaw_stand
   (ashlands) from `games/emberwake/data`, overrides {} and {"morale": true, "morale_break": 70}, presets balanced,
   aggressive, defensive, withdraw, seeds 1, 2, 3: per step the unit states (the `world_traces.rs` format), per round
   the schedule and intents, then outcome, log cells, events, `fates()` and `Tally` keys. Committed once, never
   re-recorded.
2. `phasefold.rs` replays the same 24 fights per encounter and must match. It runs after PHASEFOLD (data still legacy,
   folded on load) and after PHASEFOLD-A (data folded on disk).

## 5. Assets (PHASEFOLD-A, asset lane)

Seed `games/emberwake/data/seeds/phasefold.json`, applied with `tools/fc.sh content` (upsert steps, whole rows). Every
upsert rewrites its file through the writer, which drops the default `wave_*` keys of every other row in that file.

| Kind / world | Row | Change |
| --- | --- | --- |
| catalog encounters | goblin_ambush | phases = [arrival: round 1, block 4, units ambush_berserker, goblin_skirmisher, goblin_archer, goblin_shaman, label "Ambush! The goblin pack"] |
| ashlands encounters | pass_ambush | phases = [arrival: round 1, block 4, units nok_flamebow, gruk_smokecaller, label "The warband springs the ambush!"] |
| ashlands encounters | ashmaw_stand | phases = [arrival: round 2, block 1, units ash_berserker, ash_brawler, ash_skirmisher, label "The flankers pour out of the gorge!"], then crown_of_nails unchanged |
| tidewrack encounters | strand_clash | none (rewrites the file without the default keys) |

Each arrival has id "reinforcements", name "Reinforcements", `pause_autoplay` false. Descriptions are not changed.
Afterwards `wave_units` appears under `games/emberwake/data` only in `seeds/reward1.json` (history; a re-apply folds).

## Acceptance

Logic (PHASEFOLD):

- Every suite in 4 marked "must match unchanged" passes with no edit to `games/emberwake/parity/out`; the two retired
  ones pass with the documented fold of the Godot side only.
- `phasefold.rs` matches the golden for all 72 fights (3 encounters x 2 overrides x 4 presets x 3 seeds).
- Fold unit tests: a row with the four default keys and no units loses them and gets no phase; a legacy wave row gets
  the 1.4 phase at index 0 before its existing phases; a taken id gives "reinforcements_2"; a non-array `phases`
  leaves the row unchanged; folding twice changes nothing.
- Validation: each 1.2 and 1.3 message fires for a bad arrival; an arrival with round 7 in a 6-round fight gives
  exactly "The wave round is after the last round."; 4 phases plus an arrival is clean; an upsert with legacy `wave_*`
  keys is stored as an arrival phase.
- A rout fight whose deployed enemies all fall before the arrival is not won until the arrival units are down (W8); a
  phase wave in the same setup is still held back on a win (BOSS1 rule 4 unchanged).
- With morale on, the arrival forms a group with source "wave" and size = its placed units.

Asset (PHASEFOLD-A): `tools/fc.sh content` check clean for the catalog and all worlds; the three rows read as in 5;
`phasefold.rs` still matches the golden.

## Risks

- A fold entry point missed: a legacy row loses its wave silently. Mitigation: the fold sits in the loaders and in
  `from_obj`, and the parity suites feed legacy rows through every path.
- Message order in checks parity: the arrival checks must land where W12 put them; `checks_parity.rs` catches drift.
- Roster order if an author moves the arrival: rule "first in the list" makes it a validation error.
- Sim: copying all phases instead of the arrival only would add Crown of Nails to world sims and change balance numbers
  (rule 2.7).
- PACE1 metrics or the future phase banner counting the arrival as a boss phase (rule 2.6).

## Out of scope

Anything an arrival could newly do: a line, buffs, AI changes, round-start arrivals drafted the same round, arrivals
held back on a win, several arrivals, conditional arrivals (a flag or a unit's state). Changing any shipped fight,
label or description. A Forge phase editor. Re-recording any parity file. Any balance run (the sim output is
unchanged by construction; the balance lane need not rerun).

## Overlap with other work

`session.rs` and `encounter.rs` are shared with any battle task (OBJ1 follow-ups, MORALE1); the content checks with
any validation task; `data/worlds/ashlands/encounters.json` with any ashlands asset task. Run PHASEFOLD and
PHASEFOLD-A alone on those files.

## Open questions

1. Authoring: after the fold the Forge's encounter form has no Reinforcements picker; a wave is written inside the
   Phases JSON (as boss phases are today).
   - (rec) A: accept it now; a Forge phase editor can be a later row.
   - B: add a small Forge form that writes the arrival phase, in this task (more UI work, a Forge-area overlap).
2. Parity: battle traces, sims, journeys, saves and checks stay green unchanged; the loader diff and the resave check
   compare Godot's side after the same fold (section 4).
   - (rec) A: accept these two partial retirements.
   - B: keep the four fields in the schema as hidden read-only legacy (no retirement, but the fold is only half done
     and two code paths stay).
3. Later capabilities: should any of the out-of-scope arrival powers (held back on a win, round-start drafting, a line
   or buff, several waves) become an idea row? Each changes how fights play, so none is assumed here.
