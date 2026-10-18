# PACE1: Fight pacing targets and the two-act boss

Status: approved (user 2026-10-18 09:33 UTC, "pace all recs": Q1 A, Q2 A; Q3 B at 09:40). Revised for the user's two later changes. At 09:34 UTC he removed the party buff at the phase change: "for empowered that might not be needed though, since party has time to activate their buff for the round or have the optimal position for the fight" (old Q3 and the `party_buff` effect are gone). At 09:35 UTC he moved Empowered to a story reward: "you can have empowered as a bonus for say saving an npc before the boss fight, but not as boss mechanic" (the `empower` status stays; new encounter field `start_status`; open question 3). Direction: user Tushar, project chat 09:24 UTC, and his yes to Design's take: boss 7-9 rounds, phase 1 about 5 as a mechanic check, phase 2 about 3 as a lethal race, enrage at the change, elite about 5 as a future target
Depends on: BOSS1-L, INTENT1-L, BAL5-SIM, BAL5-PH (all on develop); none (Q3 answered B: no `start_status` data yet)
Lane: logic (PACE1-L schema-first, PACE1-S sim), asset (PACE1-A), UI/UX (PACE1-U), balance (PACE1-B); split in section 7
Touches: L: `scripts/combat/status/status_registry.gd` (row `empower`), `scripts/combat/planning/skill_def.gd` (`power_for`) and the riposte counter site in `scripts/core/combat/combatant.gd` (both where weaken applies, decision combat "D1-L weaken"), `scripts/combat/session/{encounter_phases,catalog_encounter,combat_session,intent_rules}.gd`, `scripts/exploration/game_encounter.gd` (flags, `start_status`) and its construction site (the builder reports it), `scripts/content/{content_schema,content_catalog}.gd`, `scripts/world/{world_schema,world_catalog}.gd`, `scripts/ui/world/world_preview.gd`, `boss_phase_test_suite`, `intent_test_suite`, `status_registry_test_suite`, `weaken_test_suite`, new `start_status` cases in the game-encounter suite. S: `tools/sim/{sim_metrics,sim_scenarios,sim_runner,sim_args}.gd`, `bal2_rules_test_suite`. A: `data/worlds/ashlands/encounters.json` via new `data/seeds/pace1.gd`. U: `scripts/ui/combat/*` (phase banner, Enraged chip), `scripts/ui/unit_inspector_ui.gd`

## Goal

Every fight has a length the player can feel: a mob fight is over in about 3 rounds, an elite fight takes about 5, and a boss is a two-act fight of 7-9 rounds. Act one (about 5 rounds) is a mechanic check. The party learns the boss's threats, kills or ignores the helpers, sets up its own buffs, takes the best position and avoids the big hits. When the boss breaks it enrages: it deals more damage and its whole plan is in the open. Act two (about 3 rounds) is a race where both sides are more lethal: the boss through the enrage, the party through the buffs and position it set up in act one. A clean act one lets the party burst the boss down; a sloppy one makes the race losable. Separately, the story can reward the party before a fight (for example for saving someone) with a start-of-battle buff; that is a reward, never a boss mechanic. Round targets become sim bands next to the clear % bands, so pacing is measured, not guessed.

## What exists (cited)

- BOSS1 phases (`docs/specs/BOSS1.md` 1.2-1.5): `hp_below` and `round` triggers; effects `line`, `wave`, `status` (buffs on enemy units, `grant_now`, rounds 1..5), `ai`; at most 5 effects and 4 phases; `pause_autoplay`; AI changes from the next draft; `phase_state` in `BlockSnapshot`. Grakka's Crown of Nails (hp_below 50): line, fortify 5 and regen 4 for 3 rounds, reckless at WEAKEST, ash_shaman heal_below 0.9 (decision combat "BOSS1 and OBJ1"). Both stands: max_rounds 8, objective slay (OBJ1).
- Buffs: haste, taunt, riposte, fortify, regen (`status_registry.gd`); no outgoing-damage buff. Weaken (debuff, 1..25 %) scales outgoing power in `SkillDef.power_for` and the riposte counter.
- Flags: `GameState.flags`; condition lists `[{key, value}]` validated by `QuestRules.condition_errors`. Route `scripted` fights set their `sets_flags` only on a win (`TravelRunner.after_fight`). `wagon_raid` is the scripted fight on the iron_reaches route (`requires_flags` steel_loaded, `sets_flags` []), with Wenn as a `protect` guest (OBJ1: if Wenn falls, the fight is lost).
- INTENT1: every enemy's first-block tell is shown; the full plan only via `foresight` (Bryn) for enemies targeting him (decision combat "DR1 enemy intent tells").
- MORALE1: bosses, elites and reckless units are Fearless; morale is off on the stands. HEAL1: a defeat costs a forced rest (3 supplies) or the walk back to town; no injuries (HISTORY 2026-10-18).
- BAL5: boss band 30-50; the sim boss carries a phase (`tools/sim/sim_scenarios.gd` `BOSS_PHASE`, the Crown of Nails effects); boss tier HP 1.5 -> 1.8 / 2.0 allowed on trigger (lever B3, decision balance "BAL5 levers" Q2); "boss acts twice" (B4) is a paused option build (DR2 8 B).
- Rounds today (`docs/BALANCE_REPORT.md`): courtyard 2.8, ash_straggler 1.9, ash_scouts 4.5, ash_raiders 2.7-4.4, ash_patrol 2.2-4.0, pass_ambush 3.7-5.0, wagon_raid 1.7-2.1, ashmaw_stand 4.8-7.2 (clear 60), ashmaw_stand_barred 3.8-5.6 (clear 97).

## 1. Round bands (balance targets)

| Tier | Target | Band | Read on | Today |
| --- | --- | --- | --- | --- |
| mob (any fight without an elite or boss) | 3 | 2.0-4.0 | won fights, median per cell | mostly in; wagon_raid 1.7-2.1 below, ash_scouts 4.5 and pass_ambush up to 5.0 above |
| elite (an elite-tier unit, or `pace` elite) | 5 | 4.0-6.0 | won fights, median per cell | no elite fight shipped |
| boss total | 8 | 7.0-9.0 | won fights, median per cell | stand 4.8-7.2, barred 3.8-5.6: below |
| boss phase 1 (start to the enrage) | 5 | 4.0-6.0 | fights where the phase fired | not measured |
| boss phase 2 (enrage to the end) | 3 | 2.0-4.0 | won fights where it fired | not measured |
| phase 2 one-shots | 0 | at most 2 per 100 fights | all boss fights | not measured |

- **Length unit.** A fight's length is `(r - 1) + b / 6` for its last resolved block (round r, block b). A phase fired at round r, block b (0 = round start) starts at `(r - 1) + b / 6`. The existing whole-round `rounds` column stays.
- **Pass rule.** A fight passes pacing when at least 70% of its cells have a median inside the band. Clear % bands (decision balance "BAL2 role answers" (2), "BAL5 levers" Q5) come first: a pacing lever may never push a fight out of its clear band.
- **One-shot.** One boss action that takes a hero from at least 80% of max HP to down in phase 2.
- **Exempt.** Single-foe fights (ash_straggler) and hold or escape objectives are reported but exempt from the mob band.
- **Bands are read without start buffs.** Pacing is judged with `start_status` off (sim default, section 5), so a story reward never hides a slow boss.

## 2. Rules

1. **What phase 1 must teach** (authoring rule, checked in review and by the phase 1 band): at least two threats the party can read and answer without out-trading the boss, each visible through something that exists (a status chip, an intent tell, a wave, a phase line). Grakka has four:
   - Riposte: do not melee him while it is up.
   - The smokecallers' heals: kill them first, or phase 2 lasts longer, since they heal at 90% in phase 2.
   - The round-2 flank wave (arms stand only).
   - The round-6 enrage timer (rule 3).
2. **Setting up (user 09:34).** Phase 1 is where the party spends AP on its own buffs (fortify, regen, haste, taunt) and takes the best position for the race. A phase never buffs the party: the party's burst in phase 2 is what it prepared, plus every hero still standing.
3. **Trigger.** The enrage phase uses `hp_below` (Grakka 50) with the new `or_round` (Grakka 6). The phase is due at whichever comes first, so a slow party meets phase 2 by round 6 at the latest. This is the "sloppy act one" pressure: the boss enrages with most of its HP left.
4. **Enrage** (effect `enrage`): the unit gets `empower` at `power` % for the rest of the battle, and it is Enraged: its whole round plan is shown to the party (Q2 A). The power is queued: it is active from the next planning phase on, never in a block the party planned before the phase fired. AI changes stay as BOSS1 rule 5 (next draft).
5. **Empower** (new buff row): outgoing skill power x (1 + magnitude / 100), applied wherever weaken applies (`power_for`, riposte counter); with weaken the factors multiply. Heals and the damage formula are untouched (decision combat "Combat extensions (S12)": additive inputs only). Only two things grant it: an `enrage` effect, and a `start_status` entry (rule 7). No skill or phase effect gives it to the party.
6. **Guardrails** (phase 2 can never one-shot without a tell):
   - Enrage power is at most 25.
   - One `enrage` per encounter.
   - A phase with `enrage` must have `pause_autoplay` true, so the player always sees the change before the next round is played.
   - The enraged plan is shown in full.
   - The sim one-shot line (section 1) is a hard band; when it fails, the first lever is enrage power down.
7. **Start buffs (user 09:35).** A world encounter may carry `start_status` entries. At round 1 start, before the first planning phase, each entry whose `requires_flags` all hold in `GameState.flags` grants its status to every hero and guest on the field. The grant uses `grant_now`, source `start:<encounter id>`, and its rounds count round 1. Each granted entry logs one EVENT entry with its `text`. With no game state (catalog encounters, dojo, sim without the key) nothing is granted. A start buff is a story reward chosen by the author, never part of a boss's phases.
8. **Losing the race.** A lost phase 2 is an ordinary defeat with the HEAL1 cost; no new penalty (HISTORY "Injuries and a defeat cost" stays rejected).

## 3. Schema contract (schema-first: yes)

### 3.1 Status row

| id | kind | resistable | default_rounds | stacking | cap | display | grant | magnitude |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| empower | buff | false | 2 | refresh_max_duration | 25 | Empowered | "" (no skill grants it) | 1..25 pct |

### 3.2 Trigger key (BOSS1 1.3)

| Key | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| or_round | int | 0 (none) | `hp_below` only ("Phase '%s': or_round needs an hp_below trigger."); 0 or 2..`max_rounds` ("Phase '%s': or_round must be 2..%d."). Due at that round's start when not yet fired; dropping by the trigger unit falling still applies |

### 3.3 Effect (BOSS1 1.4; the 5-effect limit stays)

| type | Keys (default) | Validation (message) |
| --- | --- | --- |
| enrage | unit (roster ref), power (20) | unit in `enemies` ("Phase '%s': enrage unit '%s' is not in enemies."); power 1..25 ("Phase '%s': enrage power must be 1..25."); one per encounter ("An encounter holds at most one enrage."); the phase has `pause_autoplay` true ("Phase '%s': an enrage phase must pause autoplay.") |

Runtime: the enrage is stored as a queued `empower` with rounds `max_rounds + 1` (it never ticks out). `EncounterPhases.enraged(unit) -> bool` reads `fired`, so `reset_round` and replays rebuild it with no new snapshot key.

### 3.4 Encounter field `start_status` (world encounters only)

| Field | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| start_status | json (array of entries) | [] | at most 2 entries ("An encounter holds at most 2 start buffs.") |

| Key | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| status | string | (required) | a buff from `StatusEffects.buffs()`, not taunt or riposte ("Start buff '%s' must be a buff other than taunt or riposte.") |
| rounds | int | 2 | 1..5 ("Start buff '%s': rounds must be 1..5.") |
| magnitude | int | 0 | inside the row's range; 0 only for rows without one ("Start buff '%s': magnitude must be %d..%d.") |
| requires_flags | json | [] | `QuestRules.condition_errors` (label "Start buff flags"); [] = always granted in the game |
| text | string | (required) | 1..120 chars ("Start buff '%s' needs a text.") |

Unknown keys are errors ("Start buff: unknown key '%s'."). The field lives in `world_schema.gd` `_encounter_fields` and is validated in `WorldCatalog._encounter_errors`; the catalog encounter schema does not get it. `GameEncounter` receives the flags from its construction site (the builder reports which).

### 3.5 Encounter field `pace` (catalog and world encounters)

| Field | Type | Default | Allowed |
| --- | --- | --- | --- |
| pace | enum | auto | auto, mob, elite, boss; auto = boss when any enemy (waves included) is tier boss, else elite when any is tier elite, else mob |

Warning (not an error, `warnings_for`): a fight whose pace resolves to boss and has no phase with an `enrage` ("Boss fight '%s' has no enrage phase (PACE1)."). `EncounterPhases.pace_of(entry) -> String` is the one resolver (validator, sim, preview).

### 3.6 Intents (INTENT1 1)

`enemy_intents()` records gain `enraged: bool`. An enraged enemy has `revealed` true and its full `plan`, whoever holds foresight (Q2 A). The preview line adds "or round N" and "enrage", plus one line per start buff (`Start: <status> <rounds> if <flags>`).

## 4. Data plan (PACE1-A, via `WorldCatalog.upsert`, seed `data/seeds/pace1.gd`)

| Row | Change |
| --- | --- |
| ashmaw_stand | `max_rounds` 8 -> 10 (a 9-round fight fits). `crown_of_nails` becomes: trigger hp_below grakka_ashmaw 50, or_round 6; effects line (new text below), enrage grakka_ashmaw power 20, ai grakka_ashmaw reckless true and target_pref WEAKEST (kept), ai ash_shaman heal_below 0.9 (kept); pause_autoplay true. Fortify 5 and regen 4 are removed (Q1 A) |
| ashmaw_stand_barred | same `max_rounds` and `crown_of_nails`; `empty_gorge` (round 4 line) unchanged |
| both | `pace` left auto (Grakka is tier boss); `start_status` per open question 3 (none under B) |

New line (1..160 chars): "Grakka tears off his crown of nails and roars, enraged. The smokecallers chant. Strike now, Wardens." Description endings: "...at half strength, or when the fight drags, Grakka enrages." Phase 1 needs no new content: Riposte, the smokecallers and the wave already exist.

Under open question 3 A, two more data changes:

- iron_reaches route scripted `wagon_raid`: `sets_flags` [{key `ashlands.wenn_saved`, value true}].
- Both stands: `start_status` [{status empower, rounds 2, magnitude 10, requires_flags [{key `ashlands.wenn_saved`, value true}], text "Wenn rides ahead with word of the camp's watch rotations. The Wardens strike first."}].

Lever order for PACE1-B: data only, at most two passes (DR1 (3)), measured with `start_status` off.

1. **Phase split:** trigger `pct` 50 to 60 (longer phase 2) or 40 (shorter); `or_round` 6 to 7 if the timer cuts phase 1 short.
2. **Boss total below 7:** BAL5 B3 boss HP 1.8, then 2.0. This is a second trigger for the approved lever and uses the same BAL5-A row.
3. **Enrage strength, within the 25 cap:** if one-shots exceed the band or clear drops below 30, go 20 to 15, then 10. If clear is above 50 or phase 2 runs over 4 rounds, go 20 to 25.

Boss acts twice (B4) is not used by PACE1 and stays with DR2-LEVERS.

## 5. Sim (PACE1-S)

- `BOSS_PHASE` (BAL5-PH) takes the PACE1 shape: or_round 6, enrage 20, ai reckless WEAKEST; fortify and regen are removed; `phase=off` is unchanged.
- New key `start=off|on` (default off). `on` grants every `start_status` entry of a world encounter regardless of flags, for the reward A/B.
- Per fight: `length`, `p1_len`, `p2_len` (section 1 unit, from the `phase_fired` info), `p2_oneshots` and the `pace` tier. Party and campaign tables add "Len", "P1 / P2", "1-shots" and a pace verdict next to the clear verdict, with won-fight medians as in section 1.
- Acceptance: `bal2_rules_test_suite` covers the length unit, the one-shot rule on a fixed board, `pace_of` for the three tiers and `start=on` granting a start buff.

## 6. Tests (PACE1-L)

- Validation: each message in 3.2-3.5 fires; the PACE1 Crown of Nails is clean in both catalogs; the boss warning fires for a boss fight without enrage; `start_status` on a catalog encounter is an unknown field.
- Empower: `power_for` x1.10 at 10; with weaken 20, x1.10 x0.80; heals unchanged; the riposte counter scales.
- Enrage fired mid-round: the boss's remaining actions this round deal unchanged damage. From the next planning phase its hits use x1.20, its intent record is enraged and revealed without foresight, and no hero gains a status from the phase.
- or_round 6 with the boss at 80%: the phase fires in round 6's intro. With the boss below 50% in round 3, it fires there and never again.
- Start buff: with its flag set, every hero and guest is Empowered in round 1 planning, one EVENT entry carries the text, and it ends after its rounds. With the flag unset, or with no game state, nobody is.
- `reset_round` and a recorded replay after the enrage and after a start buff give identical intents, statuses and `RoundOutcome` records.
- The existing BOSS1 tests stay green (old effect lists still validate).

## 7. Task split (rows for the planner)

| Id | Title | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- |
| PACE1-L | Enrage phase effect, `or_round`, empower buff, `start_status` (world encounters, flag-gated), `pace` field and boss warning, enraged intents; commit 1 = schema slice | logic (Gameplay) | not parallel with MORALE1-L (same session and catalog files) | status_registry, skill_def (`power_for`), combatant (riposte counter), encounter_phases, catalog_encounter, combat_session, intent_rules, game_encounter and its construction site, content_schema/content_catalog, world_schema/world_catalog, world_preview, boss_phase / intent / status_registry / weaken suites, game-encounter suite |
| PACE1-A | Grakka two-act data on both stands (section 4); the Wenn reward only under open question 3 A | asset | PACE1-L schema slice; not alongside MORALE1-A or ASHPACE1 (same file) | data/worlds/ashlands/encounters.json, (A only) data/worlds/iron_reaches/routes.json, data/seeds/pace1.gd |
| PACE1-S | Sim pace metrics, `start=` key and the PACE1 sim boss phase | logic (Balance) | PACE1-L; not parallel with HEAL1-S or MORALE1-S (tools/sim) | tools/sim/{sim_metrics,sim_scenarios,sim_runner,sim_args}.gd, bal2_rules_test_suite |
| PACE1-U | Enraged chip and readout on the boss token, hostile and hover card ("Enraged: +20% damage, plan shown"), second banner line; start buffs use the existing status chip and log line (UI preference, not final) | logic (UI/UX) | PACE1-L, BOSS1-U; not parallel with MORALE1-U | scripts/ui/combat/*, scripts/ui/unit_inspector_ui.gd, combat_ui_framework_test_suite |
| PACE1-B | Pace run: story fights and mobs (party mode, 10 trios, 10 seeds), both stands before and after PACE1-A, then the section 4 levers on trigger; under open question 3 A also the stands with `start=on` (report only; data passes go to the asset lane) | balance | PACE1-A, PACE1-S on main; after HEAL1-B if both queue | docs/BALANCE_REPORT.md |

## Out of scope

A party buff from a boss phase (user 09:34, 09:35); start buffs on catalog encounters or enemies; flag setters beyond the one optional `sets_flags` line; boss acts twice (B4, paused); new boss skills or art; phase 1 scripted mechanics (HISTORY "Hand-scripted combat layer"); debuffs or damage on the party from a phase; chained or party-side triggers; morale changes at the enrage; retuning mob fights (ASHPACE1, HEAL1-B own wagon_raid and ash_scouts); elite content; tier power changes; damage formula changes.

## Open questions

Q1 and Q2 were answered at 09:33 (both A, recorded in the status line).

1. **Grakka's old phase buffs.** Chosen A: replace fortify 5 and regen 4 with enrage (supersedes that part of decision combat "BOSS1 and OBJ1").
2. **Scarier tells.** Chosen A: an enraged boss shows its whole round plan to everyone (extends INTENT1 rule 3 for enraged units only).
3. **First use of a start buff.** Chosen B (user 09:40 UTC): schema only, no data yet.
   - A: Wenn. Winning `wagon_raid` sets `ashlands.wenn_saved` (one `sets_flags` line), which grants Empowered 10 for 2 rounds at both stands. Cheap and on-theme, but Wenn is a `protect` guest and the route leg completes only on a win, so every party that reaches Grakka has saved him. The bonus would be a fixed opening buff at the stand in practice, close to the boss mechanic the user ruled out, and it lifts a stand already above its band (clear 60 vs 30-50).
   - B (recommended): schema only, no data yet. The first use waits for a rescue the player can fail or skip, for example the DESERT1 deserter choice or an optional NPC from ASHPACE1. Wenn could qualify later if `wagon_raid` stops requiring a win to pass or he stops being `protect` (DR2 8 option B). The flag line from A is then a one-row asset task.
