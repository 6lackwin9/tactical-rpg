# PACE1: Fight pacing targets and the two-act boss

Status: approved (user 2026-10-18 09:33 UTC, "pace all recs": Q1 A, Q2 A, Q3 A; spec 2026-10-18; direction: user Tushar, project chat 09:24 UTC, and his yes to Design's take: boss 7-9 rounds, phase 1 about 5 as a mechanic check, phase 2 about 3 as a lethal race, enrage plus party buff at the change, elite about 5 as a future target)
Depends on: BOSS1-L, INTENT1-L, BAL5-SIM, BAL5-PH (all on develop); open questions 1-3 for the data
Lane: logic (PACE1-L schema-first, PACE1-S sim), asset (PACE1-A), UI/UX (PACE1-U), balance (PACE1-B); split in section 7
Touches: L: `scripts/combat/status/status_registry.gd` (row `empower`), `scripts/combat/planning/skill_def.gd` (`power_for`) and the riposte counter site in `scripts/core/combat/combatant.gd` (both where weaken applies, decision combat "D1-L weaken"), `scripts/combat/session/{encounter_phases,catalog_encounter,combat_session,intent_rules}.gd`, `scripts/combat/arena/block_snapshot.gd`, `scripts/content/{content_schema,content_catalog}.gd`, `scripts/world/{world_schema,world_catalog}.gd`, `scripts/ui/world/world_preview.gd`, `boss_phase_test_suite`, `intent_test_suite`, `status_registry_test_suite`, `weaken_test_suite`. S: `tools/sim/{sim_metrics,sim_scenarios,sim_runner}.gd`, `bal2_rules_test_suite`. A: `data/worlds/ashlands/encounters.json` via new `data/seeds/pace1.gd`. U: `scripts/ui/combat/*` (phase banner, chips), `scripts/ui/unit_inspector_ui.gd`

## Goal

Every fight has a length the player can feel: a mob fight is over in about 3 rounds, an elite fight takes about 5, and a boss is a two-act fight of 7-9 rounds. Act one (about 5 rounds) is a mechanic check: the party learns the boss's threats, kills or ignores the helpers, sets up its own buffs and avoids the big hits. When the boss breaks, it enrages (more damage, its whole plan in the open) and the party gets a burst buff for a few rounds. Act two (about 3 rounds) is a race where both sides are more lethal: a clean act one lets the party burst the boss down, a sloppy one makes the race losable. Round targets become sim bands next to the clear % bands, so pacing is measured, not guessed.

## What exists (cited)

- BOSS1 phases (`docs/specs/BOSS1.md` 1.2-1.5): `hp_below` and `round` triggers; effects `line`, `wave`, `status` (buffs on enemy units, `grant_now`, rounds 1..5), `ai`; at most 5 effects and 4 phases; `pause_autoplay`; AI changes from the next draft; `phase_state` in `BlockSnapshot`. Grakka's Crown of Nails (hp_below 50): line, fortify 5 and regen 4 for 3 rounds, reckless at WEAKEST, ash_shaman heal_below 0.9 (decision combat "BOSS1 and OBJ1"). Both stands: max_rounds 8, objective slay (OBJ1).
- Buffs: haste, taunt, riposte, fortify, regen (`status_registry.gd`); no outgoing-damage buff. Weaken (debuff, 1..25 %) scales outgoing power in `SkillDef.power_for` and the riposte counter.
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
| boss phase 1 (start to the first enrage) | 5 | 4.0-6.0 | fights where the phase fired | not measured |
| boss phase 2 (enrage to the end) | 3 | 2.0-4.0 | won fights where it fired | not measured |
| phase 2 one-shots | 0 | at most 2 per 100 fights | all boss fights | not measured |

- **Length unit.** A fight's length is `(r - 1) + b / 6` for its last resolved block (round r, block b); a phase fired at round r, block b (0 = round start) starts at `(r - 1) + b / 6`. The existing whole-round `rounds` column stays.
- **Pass rule.** A fight passes pacing when at least 70% of its cells have a median inside the band. Clear % bands (decision balance "BAL2 role answers" (2), "BAL5 levers" Q5) come first: a pacing lever may never push a fight out of its clear band.
- **One-shot.** One boss action that takes a hero from at least 80% of max HP to down in phase 2.
- Single-foe fights (ash_straggler) and hold or escape objectives are reported but exempt from the mob band.

## 2. The two-act boss (rules)

1. **What phase 1 must teach** (authoring rule, checked in review and by the phase 1 band): at least two threats the party can read and answer without out-trading the boss, each visible through something that exists (a status chip, an intent tell, a wave, a phase line). Grakka: Riposte (do not melee him while it is up), the smokecallers' heals (kill them first or phase 2 lasts longer, since they heal at 90% in phase 2), the round-2 flank wave (arms stand) and the round-6 enrage timer (rule 3).
2. **Setting up.** Phase 1 is where the party spends AP on its own buffs (fortify, regen, haste, taunt) and position; the party buff (rule 5) lands only on allies who are active at the phase change, so every hero still standing is worth more burst.
3. **Trigger.** The enrage phase uses `hp_below` (Grakka 50) with the new `or_round` (Grakka 6): it is due at whichever comes first, so a slow party meets phase 2 by round 6 at the latest. This is the "sloppy act one" pressure: the boss enrages with most of its HP left.
4. **Enrage** (effect `enrage`): the unit's outgoing power rises by `power` % (status `empower`, lasting for the battle) and it is Enraged: its whole round plan is shown to the party (open question 2). The power is queued: it is active from the next planning phase on, never in a block the party planned before the phase fired. AI changes stay as BOSS1 rule 5 (next draft).
5. **Party buff** (effect `party_buff`): every active party-side unit on the field (heroes and guests) gets the status at once (`grant_now`, source `phase:<id>`, rounds count the fire round). Downed or withdrawn allies get nothing. Default: `empower` 15 for 3 rounds, the burst window of the race.
6. **Guardrails** (phase 2 can never one-shot without a tell): enrage power at most 25 (registry cap); one `enrage` per encounter; a phase with `enrage` must have `pause_autoplay` true (the player always sees the change before the next round is played); the enraged plan is shown in full; the sim one-shot line (section 1) is a hard band, and the first lever when it fails is enrage power down.
7. **Losing the race.** A lost phase 2 is an ordinary defeat with the HEAL1 cost; no new penalty (HISTORY "Injuries and a defeat cost" stays rejected).
8. **Empower** (new buff row): outgoing skill power x (1 + magnitude / 100), applied wherever weaken applies (`power_for`, riposte counter); with both, the factors multiply. Heals and the damage formula are untouched (decision combat "Combat extensions (S12)": additive inputs only).

## 3. Schema contract (schema-first: yes)

### 3.1 Status row

| id | kind | resistable | default_rounds | stacking | cap | display | grant | magnitude |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| empower | buff | false | 2 | refresh_max_duration | 25 | Empowered | "" (phases only; no skill grants it) | 1..25 pct |

### 3.2 Trigger key (BOSS1 1.3)

| Key | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| or_round | int | 0 (none) | `hp_below` only ("Phase '%s': or_round needs an hp_below trigger."); 0 or 2..`max_rounds` ("Phase '%s': or_round must be 2..%d."). Due at that round's start when not yet fired; dropping by the trigger unit falling still applies |

### 3.3 Effects (BOSS1 1.4; the 5-effect limit stays)

| type | Keys (default) | Validation (message) |
| --- | --- | --- |
| enrage | unit (roster ref), power (20) | unit in `enemies` ("Phase '%s': enrage unit '%s' is not in enemies."); power 1..25 ("Phase '%s': enrage power must be 1..25."); one per encounter ("An encounter holds at most one enrage."); the phase has `pause_autoplay` true ("Phase '%s': an enrage phase must pause autoplay.") |
| party_buff | status ("empower"), magnitude (15), rounds (3) | status a buff (`StatusEffects.buffs()`) and not taunt or riposte ("Phase '%s': '%s' cannot be a party buff."); magnitude inside the row's range, 0 only for rows without one; rounds 1..5; one per phase ("Phase '%s': one party buff per phase.") |

Runtime: the enrage power is stored as a queued `empower` with rounds `max_rounds + 1` (never ticks out); `EncounterPhases.enraged(unit) -> bool` reads `fired`, so `reset_round` and replays rebuild it with no new snapshot key.

### 3.4 Encounter field `pace` (catalog and world encounters)

| Field | Type | Default | Allowed |
| --- | --- | --- | --- |
| pace | enum | auto | auto, mob, elite, boss; auto = boss when any enemy (waves included) is tier boss, else elite when any is tier elite, else mob |

Warning (not error, `warnings_for`): a fight whose pace resolves to boss and has no phase with an `enrage` ("Boss fight '%s' has no enrage phase (PACE1)."). `EncounterPhases.pace_of(entry) -> String` is the one resolver (validator, sim, preview).

### 3.5 Intents (INTENT1 1)

`enemy_intents()` records gain `enraged: bool`. An enraged enemy has `revealed` true and its full `plan`, whoever holds foresight (open question 2 A). The preview line adds "or round N" and "enrage" / "party buff".

## 4. Data plan (PACE1-A, both via `WorldCatalog.upsert`, seed `data/seeds/pace1.gd`)

| Row | Change |
| --- | --- |
| ashmaw_stand | `max_rounds` 8 -> 10 (a 9-round fight fits); `crown_of_nails` effects become: line (new text below); enrage grakka_ashmaw power 20; party_buff empower 15, 3 rounds; ai grakka_ashmaw reckless true, target_pref WEAKEST (kept); ai ash_shaman heal_below 0.9 (kept); trigger hp_below grakka_ashmaw 50, or_round 6; pause_autoplay true. Fortify 5 and regen 4 are removed (open question 1 A) |
| ashmaw_stand_barred | same `max_rounds` and `crown_of_nails`; `empty_gorge` (round 4 line) unchanged |
| both | `pace` left auto (Grakka is tier boss) |

New line (1..160 chars): "Grakka tears off his crown of nails and roars. He is enraged; the Wardens see the opening. The smokecallers chant." Description endings: "...at half strength, or when the fight drags, Grakka enrages." Phase 1 needs no new content: Riposte, the smokecallers and the wave already exist.

Lever order for PACE1-B (data only, at most two passes, DR1 (3)): phase split -> `pct` 50 to 60 (longer phase 2) or 40 (shorter); boss total below 7 -> BAL5 B3 boss HP 1.8, then 2.0 (a second trigger for the approved lever, same BAL5-A row); one-shots or clear below 30 -> enrage power 20 to 15, then party buff 15 to 20; clear above 50 -> enrage 25. Boss acts twice (B4) is not used by PACE1 and stays with DR2-LEVERS.

## 5. Sim (PACE1-S)

- `BOSS_PHASE` (BAL5-PH) takes the PACE1 shape: or_round 6, enrage 20, party_buff empower 15 x 3, ai reckless WEAKEST; `phase=off` unchanged.
- Per fight: `length`, `p1_len`, `p2_len` (section 1 unit, from `phase_fired` info), `p2_oneshots`, `pace` tier; party and campaign tables add "Len", "P1 / P2", "1-shots" and a pace verdict next to the clear verdict; won-fight medians as in section 1.
- Acceptance: `bal2_rules_test_suite` covers the length unit, the one-shot rule on a fixed board and `pace_of` for the three tiers.

## 6. Tests (PACE1-L)

- Validation: each message in 3.2-3.4 fires; the PACE1 Crown of Nails is clean in both catalogs; the boss warning fires for a boss fight without enrage.
- Empower: power_for x1.15 at 15; with weaken 20, x1.15 x0.80; heals unchanged; the riposte counter scales.
- Enrage fired mid-round: the boss's remaining actions this round deal unchanged damage; from the next planning phase its hits use x1.20 and its intent record is enraged and revealed without foresight.
- Party buff: active heroes and guests get empower at once; a downed hero does not.
- or_round 6 with the boss at 80%: the phase fires in round 6's intro; with the boss below 50% in round 3 it fires there and never again.
- `reset_round` and a recorded replay after the enrage give identical intents, statuses and `RoundOutcome` records.
- The existing BOSS1 tests stay green (old effect lists still validate).

## 7. Task split (rows for the planner)

| Id | Title | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- |
| PACE1-L | Enrage and party buff phase effects, `or_round`, empower buff, `pace` field and boss warning, enraged intents; commit 1 = schema slice | logic (Gameplay) | Q1-Q3; not parallel with MORALE1-L (same session and catalog files) | status_registry, skill_def, combatant (riposte counter), encounter_phases, catalog_encounter, combat_session, intent_rules, block_snapshot, both schemas and catalogs, world_preview, boss_phase / intent / status_registry / weaken suites |
| PACE1-A | Grakka two-act data on both stands (section 4) | asset | PACE1-L schema slice; not alongside MORALE1-A or ASHPACE1 (same file) | data/worlds/ashlands/encounters.json, data/seeds/pace1.gd |
| PACE1-S | Sim pace metrics and the PACE1 sim boss phase | logic (Balance) | PACE1-L; not parallel with HEAL1-S or MORALE1-S (tools/sim) | tools/sim/{sim_metrics,sim_scenarios,sim_runner}.gd, bal2_rules_test_suite |
| PACE1-U | Enraged chip and readout on the boss token, hostile and hover card ("Enraged: +20% damage, plan shown"), Empowered chip on heroes, second banner line (UI preference, not final) | logic (UI/UX) | PACE1-L, BOSS1-U; not parallel with MORALE1-U | scripts/ui/combat/*, scripts/ui/unit_inspector_ui.gd, combat_ui_framework_test_suite |
| PACE1-B | Pace run: story fights and mobs (party mode, 10 trios, 10 seeds), both stands before and after PACE1-A, then the section 4 levers on trigger (report only; data passes go to the asset lane) | balance | PACE1-A, PACE1-S on main; after HEAL1-B if both queue | docs/BALANCE_REPORT.md |

## Out of scope

Boss acts twice (B4, paused); new boss skills or art; phase 1 scripted mechanics (HISTORY "Hand-scripted combat layer"); debuffs or damage on the party from a phase; chained or party-side triggers; morale changes at the enrage; retuning mob fights (ASHPACE1, HEAL1-B own wagon_raid and ash_scouts); elite content; tier power changes; damage formula changes.

## Open questions

1. **Grakka's old phase buffs.** Fortify 5 and regen 4 make phase 2 longer, against "phase 2 is quicker". (rec) A: replace them with enrage (supersedes that part of decision combat "BOSS1 and OBJ1"). B: keep both plus enrage (tankier and deadlier; phase 2 likely 4+ rounds). C: keep regen 4 only.
2. **Scarier tells.** (rec) A: an enraged boss shows its whole round plan to everyone (extends INTENT1 rule 3 for enraged units only); the race is lethal but readable. B: first-block tell as today plus an Enraged chip; one-shots guarded by numbers only. C: full plan only through Bryn's foresight (today's rule).
3. **Earning the party buff.** (rec) A: automatic at the change for every active ally (what the user agreed; standing heroes are the reward). B: only if the boss's helpers are down by then (a sharper phase 1 check; needs a condition key, BOSS1 kept chained conditions out). C: magnitude scaled by party HP (10-20).
