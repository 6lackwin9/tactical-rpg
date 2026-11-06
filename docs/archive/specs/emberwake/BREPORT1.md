# BREPORT1: Battle report on the results screen (what worked, from sim metrics)

Status update (2026-10-20): approved; BREPORT1-L done on Godot, PORT-BREPORT and BREPORT1-U approved and queued. The status line below is the original.

Status: proposed (spec 2026-10-18; approved direction: decision world-campaign "DR1 design review answers (user 2026-10-18)" (all recommendations), review `games/emberwake/docs/specs/DESIGN_REVIEW1.md` 3.7)
Depends on: L: none (sequence with OBJ1-L for the one-line `tools/sim/sim_runner.gd` change). U: BREPORT1-L and UIR-4 (same two results files; build on its grid columns)
Lane: logic only (BREPORT1-L engine and model, then BREPORT1-U view on the UI/UX thread); no asset part, no data
Touches: L: new `scripts/combat/session/{battle_observer,battle_metrics,battle_report}.gd`, `tools/sim/{sim_metrics,sim_observer,sim_runner}.gd`, `scripts/combat/arena/combat_resolver.gd` (one signal), `scripts/core/combat/battle_command.gd`, `scripts/combat/actions/{attack_command,area_attack_command}.gd`, `scripts/ui/overworld/{battle_screen,region_map_view}.gd`, `scripts/ui/menus/{main_menu_view,menu_router,battle_results_model}.gd`, `scripts/tests/{bal2_sim_test_suite,status_expansion_ai_test_suite}.gd` (preload path only if it moves), new `scripts/tests/battle_report_test_suite.gd`, `scripts/tests/suite_registry.gd`. U: `scripts/ui/menus/battle_results_view.gd`, `combat_ui_framework_test_suite` or `journal_test_suite` (wherever UIR-4 puts the results checks)

## Goal

After every game battle (won, lost or fled) the results popup shows a short report per hero: how much damage they dealt and took, how much they healed, how many effects (statuses) they put on, how many combos they landed, and one "best moment" line. Players learn what worked, and the combo and status systems (DESIGN_REVIEW1 section 2, gap 7: barely used) become visible. The numbers come from the same per-unit collector the balance sim already uses (BAL2_SIM section 4), moved into shipped code, so the screen and the sim can never disagree. No combat rule changes; recordings replay identically.

## What exists (cited)

- `tools/sim/sim_metrics.gd` (extends `tools/sim/sim_observer.gd`): one record per unit per fight, keyed by display name, from `Combatant.health_changed`, `session.log_appended`, `resolver.block_completed` and `resolver.last_round_end_steps`. Fields used here: `dmg_dealt` (direct hits and riposte counters), `dot_dealt` (bleed, poison, burn credited to the status source), `dmg_taken`, `dot_taken`, `ff_taken`, `ff_dealt`, `heal_done` (heals and regen), `buffs_applied`, `debuffs_applied`, `kills`, `near_death_saves`. Self-checks (`METRIC FAIL`) guard HP conservation. It is driven by `sim_runner.gd` hooks `on_start/on_step/on_round/on_end` and never sees a rewind.
- It lives in `tools/`, which shipped code must not preload.
- Combos (S12C-L) leave no structured trace: stance breaks are `BattleCommand.broken_targets` + `combo_notes` text, flows are a `[Flow]` text line from `CombatResolver._apply_combo`, exploits are only the " (exploits X)" suffix in `execution_log`. The collector counts none of them.
- Display names are unique per battle (`CatalogEncounter._display_names` numbers repeats); wave units are in `session.enemies` from the start, off the field until they arrive.
- Live sessions rewind: `reset_round` emits `round_reset(r)` then restores the round-start snapshot (round 1: `start()`, which rebuilds and emits `mounted`); `round_started(r)` fires at the end of `_begin_round` (also after a redo); `round_concluded(r, over)` fires after `apply_round_end_effects`. Snapshot restores set `hp` directly.
- Results flow: `RegionMapView.settle` -> `fight_settled(res)` -> `MainMenuView._on_fight_settled` -> router `RESULTS` args `{award, advanced, outcome, rewards}` -> `BattleResultsModel`. The `BattleScreen` stays mounted under the popup for Replay (`replay()` = `select_round(1)` + autoplay over snapshots).
- Recorder (`RoundOutcome`) keeps only end-of-round state (hp, ap, pos, stance, statuses); it has no attribution, so it is not a source.

## What the report shows (plain words)

One table, one row per hero (then guests, OBJ1), in roster order, under the XP rows:

| Column | id | Value | Hover tip |
| --- | --- | --- | --- |
| Hero | name | name; " (guest)" for guests; "down" or "left" tag if not standing at the end | - |
| Damage | dealt | `dmg_dealt + dot_dealt` | "%d from hits, %d from bleed, poison and burn." (+ " Hit allies for %d." when `ff_dealt > 0`) |
| Taken | taken | `dmg_taken + dot_taken + ff_taken` | "%d from hits, %d from effects." |
| Healed | healed | `heal_done` | "Includes %d from regeneration." when regen > 0 |
| Effects | effects | `buffs_applied + debuffs_applied` | "Bleed x2, Slow, Haste on allies x1" (status display names with counts) |
| Combos | combos | exploits + breaks + flows | "Exploit x1, Stance break x1, Flow x2" |

Then one line: "Best moment: ..." (rule 6). A 0 shows as "-". Damage and Taken always show; Healed, Effects and Combos are hidden when every row is 0 (Q3). Headline, XP rows, rewards and objective line (OBJ1-U) are unchanged.

## 1. Schema and API contract (schema-first: yes; no data files)

### 1.1 `BattleObserver` (`scripts/combat/session/battle_observer.gd`, moved from `tools/sim/sim_observer.gd`)

Same four do-nothing hooks and signatures. `tools/sim/sim_runner.gd` preloads it instead of `sim_observer.gd` (one line); `tools/sim/sim_observer.gd` is removed (or kept as a one-line `extends` alias if anything else preloads it; builder's choice, say which in the docs delta).

### 1.2 `BattleMetrics` (`scripts/combat/session/battle_metrics.gd`, the counting part of `sim_metrics.gd`)

- Moved as is: `FIELDS`, the record build, HP attribution, round-end credit, status diff, block sampling, self-checks, `accumulate`. `tools/sim/sim_metrics.gd` becomes `extends` BattleMetrics and keeps only the sim-only parts (`kpi_header`, `kpi_line`, `per_fight`, `ratio`, `keep_totals`/`totals` if the builder prefers). The sim's printed output must be byte-identical before and after.
- New numeric field appended to `FIELDS`: `combos` (int). New per-unit detail outside `FIELDS` (not accumulated, not printed): `detail[name] = {status_counts: {status_id: int}, combo_counts: {exploit: int, break: int, flow: int}, regen_done: int, best_hit: {}, best_finish: {}, best_save: {}}`.
- `best_hit` = `{amount, skill, target, round, order}` (largest single foe hit by this unit, damage after the victim's HP cap); `best_finish` = `{skill, target, combo, round, order}` (a hit that downed a foe and carried an exploit or break); `best_save` = `{ally, hp, round, order}` (a heal on an ally below `NEAR_DEATH_FRAC` who is standing at the end; `hp` = the ally's HP before the heal). `order` = index of the log entry, for ties.
- Live mode (the game): `attach(session: CombatSession) -> void` connects `mounted`, `round_started`, `round_reset`, `round_concluded` and calls `on_start`; `detach() -> void` disconnects everything (also `_release`). `freeze() -> void` detaches and keeps the records. The sim never calls `attach`; its path is unchanged.

### 1.3 Combo signal (engine, no outcome change)

- `BattleCommand.exploited: Array[Dictionary]` (`{target: Combatant, status: String}`), cleared and filled in `AttackCommand.execute` / `AreaAttackCommand.execute` from the `matched` result (one per victim with a non-empty status).
- `CombatResolver` signal `combo_resolved(kind: String, actor: Combatant, target: Combatant, status: String, skill: String, killed: bool)`; kind is `exploit`, `break` or `flow`. Emitted in `_apply_combo`: every `exploited` row, then every `broken_targets` row, then the flow (target = actor, status = ""), only when the existing code would apply or log them. `killed` = target not alive.

### 1.4 Report dict (`BattleReport.build(metrics: BattleMetrics, session: CombatSession, outcome: String) -> Dictionary`, static, pure)

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| version | int | 1 | bump on a shape change |
| outcome | String | "" | won, lost or fled |
| rounds | int | 0 | `session.current_round` at the end |
| rows | Array[Dictionary] | [] | one per party-side unit: heroes in `session.allies` order, then guests |
| rows[i].ref | String | "" | `session.encounter.hero_key(c)` (member id in a game) |
| rows[i].name | String | "" | display name |
| rows[i].guest | bool | false | not a party member (OBJ1 guest; false until OBJ1-L) |
| rows[i].state | String | "standing" | standing, down, left (withdrawn) |
| rows[i].dealt, dealt_dot, ff_dealt, taken, taken_dot, healed, regen, effects, combos, kills | int | 0 | as the column table |
| rows[i].status_counts, combo_counts | Dictionary | {} | for the tips |
| columns | Array[String] | ["name", "dealt", "taken"] | shown column ids in the order of the table above (rule 5) |
| best | String | "" | the best-moment sentence, "" = no line |

Validation: `BattleReport.problems(report) -> Array[String]` (missing key, wrong type, negative number, unknown column id); the model treats a report with problems as empty and the popup shows no report section.

### 1.5 Plumbing

- `BattleScreen`: creates `BattleMetrics` and calls `attach(session)` in `_mount` (game battles only); `report() -> Dictionary` builds once, the first time it is called with a non-empty `outcome`, then `freeze()`s and caches; later calls return the cache.
- `RegionMapView.settle`: `res["report"] = battle.report()` before `_close_battle` (or `{}` with no battle, for example the debug outcome buttons); every path that emits `fight_settled` (road fights re-emitted from the world map included) carries `report`.
- `MainMenuView._on_fight_settled` passes `"report": DictUtil.dict_of(res.get("report", {}))`; `MenuRouter` passes it to `BattleResultsModel.new(..., p_report: Dictionary = {})` (last parameter, default `{}`).
- `BattleResultsModel`: `report: Dictionary`; `has_report() -> bool`; `report_columns() -> Array[Dictionary]` (`{id, label}`); `report_rows() -> Array[Dictionary]` (`{name, state, guest, cells: {id: String}, tips: {id: String}}`, "-" for 0); `best_moment() -> String` ("Best moment: " + best, or "").

## 2. Rules

1. **Shown for every outcome.** Won, lost and fled battles all show the report (a lost fight is where the lesson is). Dev arena, dojo and sim fights show nothing new.
2. **Only what was kept counts.** At `round_started(r)` the collector credits pending round-end steps, re-baselines HP, statuses and tiles of every unit, and stores a deep-copy checkpoint of its records and details for round r. At `round_reset(r)` it restores checkpoint r and drops later ones. At `mounted` (round 1 redo rebuilds the units) it re-runs `on_start`. Undone rounds therefore leave no trace; the report equals the battle as it finally happened.
3. **Frozen at the end.** The report is built and the collector frozen when the outcome is first known (before settle); Replay and the playback cursor cannot change it.
4. **Same numbers as the sim.** Attribution is exactly BAL2_SIM section 4 (the moved code). Friendly fire is not Damage (tip only); overheal is not shown; riposte counters count as Damage; regen counts as Healed for the unit that granted it; DoT counts as Damage for the status source.
5. **Columns.** name, dealt, taken always; healed, effects, combos only if some row is non-zero. Effects counts buffs on own side plus debuffs on foes (re-applying an equal queued status is not counted, as in the sim).
6. **Best moment** (one line, heroes and guests only; first rule that has a candidate wins; within a rule the largest number, then the earliest `order`):
   - a. Save: "%s healed %s at %d HP, and %s made it through." (healer, ally, hp, ally).
   - b. Combo finish: "%s finished %s with %s (%s)." (hero, target, skill, "exploited Bleed" / "broke the guard").
   - c. Biggest hit: "%s's %s hit %s for %d." (hero, skill, target, amount); skipped when the amount is below 1.
   - d. None: no line.
7. **Withdrawn and down.** A hero who left by an exit keeps their numbers with state "left"; a downed hero keeps theirs with "down". Units that never deployed (reserve members) are not in the battle and have no row.
8. **No gameplay effect.** The combo signal and the collector change no HP, AP, status, order or RNG draw; every recorded playtest replays with identical `RoundOutcome`s.

## 3. Acceptance tests (`battle_report_test_suite`, new, plus existing suites)

- Sim parity: one seeded party fight run through `SimRunner` with `SimMetrics` and the same fight driven through the live session API with `attach` give equal records for every unit and field; the sim's KPI lines for that fight are byte-identical to a fixture taken before the move.
- `bal2_sim_test_suite` and `status_expansion_ai_test_suite` conservation checks pass unchanged.
- Combo signal: a `crushing_blow` on a guarding foe emits one `break`; an `opportunist` hit on a bleeding foe emits one `exploit` with status bleed; `feint` emits one `flow`; an area attack exploiting two victims emits two; a failed command emits none. `combos` and `combo_counts` match.
- Rewind: confirm round 2, then `reset_round` and replay it with a different plan: the report equals a fresh battle that played only the second plan. Redo of round 1 (`start()`) gives all zeros at that point.
- Frozen: after the outcome, `report()` twice is the same dict; running Replay to the end leaves it unchanged.
- Bleed from a hero shows in that hero's Damage and in the victim's Taken; a regen granted by the priest shows in the priest's Healed and the tip.
- Columns: a party without a healer has no Healed column; an all-zero Combos column is hidden; Damage and Taken always present.
- Best moment: a fixture with a save and a bigger hit picks the save; with a combo finish and a bigger plain hit picks the finish; ties pick the earlier entry; a fight where no hero hit anything has no line.
- States: an escaped hero is "left", a downed one "down"; lost and fled outcomes still carry a full report.
- Plumbing: `settle` with no battle gives `report = {}` and the model's `has_report()` is false; a report with a negative number or unknown column fails `problems` and is hidden; old router args without `report` still open the popup.
- Determinism: one shipped playtest recording replays with identical round outcomes before and after BREPORT1-L.

## 4. View (BREPORT1-U, UI/UX thread, after UIR-4)

- Section "Battle report" below the XP rows and above the reward lines, inside UIR-4's grid frame: fixed-width numeric columns right-aligned, hero name column left; tips on hover and keyboard focus; best-moment line under the table in `UiTheme.COL_TEXT`, wrapped.
- At most 6 rows (4 heroes + 2 guests); at 1280x720 in all four text sizes the table, XP rows, rewards and the action bar fit without overlap; if not, the report section scrolls, not the action bar.
- Minimise pill and Replay keep working; the report stays when the popup is restored.

## Out of scope

Enemy rows, per-round graphs, damage by skill, tips or advice text ("try using combos"), saving reports to the save file or the journal, adding reports to playtest recordings or `TrailReport`, sim table changes, XP or rewards based on performance, overheal display, the combat log or HUD (COMBAT-UI2).

## Overlap with other specs

- UIR-4 (approved): owns the results screen layout (grid columns, loot rows, Continue in the action bar) in the same two files. BREPORT1-L only adds model API and a constructor parameter; BREPORT1-U lands after UIR-4 and uses its frame.
- OBJ1-U: also adds a line to `battle_results_model.gd` (objective line) and OBJ1-L adds guests (rows get `guest: true` then; until OBJ1-L every row is a hero). OBJ1-L edits `tools/sim/sim_runner.gd` (win from `outcome`): sequence the one-line preload change with it.
- COMBAT-UI2 / UIR-5: own the in-battle victory banner over the arena in `battle_screen.gd`; BREPORT1-L adds only the metrics wiring and `report()` there. Not in parallel with COMBAT-UI2-L1 if it edits `battle_screen.gd`.
- AUTO1: autoplayed rounds count like player rounds (no `source` split).
- BAL2 sim (balance lane): the move must not change sim output; the balance thread should be told the collector moved.
- ITEMUSE1-L: a pouch item heal goes through the support log path; if it logs as a heal entry it counts as Healed, else it fails the sim checks there first. Check in that task's review.

## Open questions

1. Where does the counting live?
   - (rec) A: move the sim collector into shipped code and attach it live with round checkpoints. One counting truth, sim parity is testable; cost is a careful move and a byte-identical sim check.
   - B: a new lean shipped tally (only the report fields) plus a parity test against `SimMetrics`. No risk to the sim, but two copies of the attribution rules to keep in step.
   - C: read the final `session.log_entries` after the battle. Cheapest, but DoT, regen, statuses and combos are only in text, so those columns would be guesses.
2. Which best-moment line?
   - (rec) A: the priority list of rule 6 (save, combo finish, biggest hit). Rewards support and combos, the systems the report is meant to show.
   - B: biggest hit only. Simplest and always present, but always praises the damage dealer.
   - C: no best-moment line, only the table. Shortest, but drops a DR1 item.
3. Columns that are 0 for every hero?
   - (rec) A: hide Healed, Effects and Combos when all zero; Damage and Taken always. Short tables for simple parties.
   - B: always show all five. A row of "-" under Combos hints that combos exist, but adds noise every battle.
   - C: hide any all-zero column, Damage and Taken included. Shortest, but the shape changes from battle to battle.
4. Is the report always open?
   - (rec) A: always shown under the XP rows. The lesson is seen without a click.
   - B: a "Report" toggle, closed by default. Keeps the popup short but most players will never open it.
   - C: open after lost and fled, closed after won. Teaches when it matters most; the rule is not obvious to players.
