# CAMPAIGN1: campaign meters, the day clock, checkpoints, run over, the named purse, rests and retreats

Status: proposed (spec 2026-10-25, from CANON 2.5 and 2.6 and the rows "Inquisition Alert by act", "Campaign meters
and fail states", "Economy", "Campaign group (1.2)", "Campaign follow-ups", "Hub group (1.4)" in
`games/emberwake/docs/decisions/lore.md`; Q-13 to Q-20a, Q-43, Q-45, Q-46)
Depends on: none to start (CAMPAIGN1-E, -S); ROSTER1 (written in parallel) for wounded, captured and the living
count; STORYUI1 (in parallel) for the vignette player, the end cards and the mission board; KIT1-N (this
spec's story-free modules are written in the kit crates, KIT1_TASKS 5)
Lane: logic first (engine, schema, runtime, GUI, sim), then asset (the `sunk_ward` rows); tests, parity plan, task
split, data, docs delta and open questions: [CAMPAIGN1_TASKS.md](CAMPAIGN1_TASKS.md)
Touches (by task, see the task file): `engine/crates/aegis_save/src/lib.rs`; under `kits/rpg/crates/`:
`rpgkit_world/src/{campaign/*,state,save,quest,dialogue,battle,travel,progression,explore,lib}.rs`,
`rpgkit_content/src/{source,store}.rs`, `check/{world,campaign}.rs`, `rpgkit_sim/src/{args,deadlines}.rs`; under
`games/emberwake/crates/`: `emberwake_gui/src/{hud,worldmap,region,battle,screens,menus,forge,runover}.rs`;
`kits/rpg/schema.json`; `games/emberwake/data/worlds/sunk_ward/*` (asset lane)
Paths (KIT1_TASKS 5): `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`;
line numbers in kit files are pre-KIT1 (KIT1-M moved them byte-identical).

History: `docs/HISTORY.md` 2026-10-25 drops gold, the other upkeep versions, a siege Breach as a Game Over and
Extinguished as an ending; this spec keeps all four out. Nothing else there touches meters, clocks or fail screens.

## 1. Goal

One kit hook family for a campaign's pressure, off unless a world names it, so the Ashlands run byte-identical:
meters with named bands and per-act rules (a gauge such as the Inquisition Alert, a countdown such as the Doom
Clock on the travel clock), checkpoint missions with day deadlines and bad ends, a run-over state shown on one
failure screen, purse names and an upkeep zero effect per world, rests that cost days, a retreat after a lost
battle, and data-driven triggers. Everything the game names (Alert, the Red Tithing, Scrap, Tallow, the
Extinguished) is data. The code is story-free, in the kit crates (KIT1).

## 2. Survey: what exists (`develop` at 153ec78)

| Area | Where | Today |
| --- | --- | --- |
| Clock | `rpgkit_world/src/state.rs:468`, `travel.rs:385-389` | `GameState.time`, a unitless count; a leg adds its route `time` (schema default 3; Ashlands routes 2-4, Iron Reaches 6) |
| Upkeep | `state.rs:16-17,299,356-370`, `travel.rs:386-392` | `party.supplies` 0-99, start 10 (`start_supplies` manifest field, `travel.rs:126`); a leg spends `time` supplies; travel never refused |
| Zero upkeep | `travel.rs:12,110-124` | road-fight chance x1.5 (`STARVED_PCT`), capped 100 |
| Currency | `state.rs:15,298,350` | `party.gold` 0-9,999,999; dialogue `gold` condition and `take` gold (`dialogue.rs:177,661`) |
| Rest | `explore.rs:456-460`; `progression.rs:229-319` | entering a `rest` location heals for free; Camp on a world-map node costs 3 supplies, no time, refused when nobody is hurt or supplies < 3 |
| Defeat | `explore.rs:528-562`, `progression.rs:331-347`, `travel.rs:531-534` | `wake`: full heal, wake at the checkpoint or last rest place, a forced rest for 3 supplies or "back to town"; no time passes |
| Flags | `state.rs:515-558`; `quest.rs:18-57,364-379` | bool, int or string; keys `[a-z0-9_.]`; `condition_met` is the one evaluator (`==`, `!=`, `>=`, `<=`, `>`, `<`), used by routes, markers (`explore.rs:207`), quests, shops, dialogue (`dialogue.rs:174`) and PACE1 start grants (`gui/battle.rs:570`) |
| Dialogue effects | `dialogue.rs:607-722` | `set_flag`, `add_flag` (min, max), `start_quest`, `turn_in`, `give`, `take`, `open_shop`, `fight`, `recruit`; `apply` is a `DialogueRunner` method |
| Events, triggers | - | none: routes carry `scripted` fights with flag gates (`travel.rs:426-435`), markers carry `sets_flags`; no timed or meter event |
| Tags | `schema.json` world tables | `encounters`, `quests`, `routes`, `locations` carry free `tags` (all empty in shipped data) |
| Battle start | `gui/battle.rs:548-590`; `world/battle.rs:117-120` | a world battle builds the active members at their stored HP; AP from the unit (character `tuning.ap` only) |
| Results | `world/battle.rs:363`, three `after_fight` (`explore.rs:747`, `travel.rs:482`, `dialogue.rs`) | `settle` pays XP and rewards; won sets done flags; lost wakes |
| HUD | `emberwake_gui/src/hud.rs:67-118` | place name; "Gold" and "Supplies" with fixed icons and tips; shared by the region and world maps (`worldmap.rs:206`) |
| Game over | - | none: no fail condition, no failure screen, no run-over state |
| Saves | `world/src/save.rs:17,115-199`; `aegis_save/src/lib.rs:13-25,143` | version 12; numbered slots 0 (autosave) to 5; no named saves |
| World config | `games/emberwake/game.json`; `data/worlds/*/world.json`; `data.rs:166` | manifest fields resolve over the lineage (`party_max`, `start_supplies`); no `sunk_ward` yet (SUNKWARD1) |
| Parity | `emberwake_world/tests/{journey,saves,world_traces,data_source}.rs` (game) | journeys compare every result, state and view with Godot (`journey.rs:16-18` strips REWARD1's keys); saves compare the loaded state |

## 3. Shared model `[kit hook]`

- X1. A world has a campaign when its lineage sets the manifest field `campaign` (section 11.1). Without it every
  rule below is inert: no state written, no result key added, no read key answered (P-rules, task file 2).
- X2. Day: `campaign.day = gs.time + 1` (day 1 at a new game). In a campaign world route `time` 1 = 1 day; the
  asset lane sets `time` on every route (the schema default 3 is wrong for it). Battles cost no days.
- X3. The campaign state is `gs.campaign` (section 11.5), saved as the key `campaign` only when the world has a
  campaign (omitted otherwise, so Ashlands saves and journey states are unchanged).
- X4. One entry point, `campaign::tick(d, gs, cause) -> events`, runs after every cause in this order: inputs, meter
  rules, countdown rules, checkpoints, triggers, then the run-over check. Causes: `days(n, why)` (why = travel, rest,
  retreat), `battle(encounter, outcome)`, `enter(location)`, `effects` (after a dialogue or trigger applied campaign
  effects), `act(n)`. No randomness; rows run in data order; a rule's effects that move a meter re-enter the rule
  pass at most 8 times per tick (the content check refuses a self-feeding pair it can see).
- X5. Read keys (section 11.4) make meters, bands, the act and the day readable by every existing flag condition, so
  routes, markers, quests, shops, dialogue choices and start grants react to them with no new field.

## 4. Meters `[kit hook]` (world kind `meters`; the game's rows are `[data]`)

- M1. Kinds: `gauge` (a stored value in `min..max`) and `countdown` (a stored due time; its value is the days left,
  `due - gs.time`, never below 0). A countdown runs only after `clock_start`; a stopped one reads its `max`.
- M2. Bands: the value's band is the last band whose `from` is at or below it; each band has a name and a tone
  (`calm`, `warn`, `danger`, `fatal`) for the HUD.
- M3. Inputs move a gauge: `battle` (the settled encounter's tags and outcome), `day` (per day passed, any cause),
  `leg` (per travel leg, route tags). Story amounts and bribes are the effects `meter_add` and `meter_set` on
  dialogue choices (a bribe is `take` Scrap or Rock Salt plus `meter_add` with a negative delta) and on triggers.
- M4. Clamp: a gauge stays in `floor..max`, where `floor` is the highest `floors` entry whose `from_act` is at or
  below the act, else `min`. When the act changes the value rises to the floor (Alert from Act 2: at least 30).
- M5. Rules fire on a crossing: `at` (a band id = its `from`, or a number), `dir` `up` (old below `at`, new at or
  above) or `down` (old above, new at or below; the default for countdowns), `acts` (empty = every act), `once`.
  A rule may `warn` (a notice), run `effects` immediately, queue `then` items (section 9) and, on a countdown,
  `restart_days` (the clock runs again with that many days left after its effects).
- As built (cross-reference, HOOKS1-D 2026-10-31): M2's band pick and M5's crossing are one kit helper,
  `rpgkit_rules::bands` (`band_index`, `point`, `crosses`, `crossing_order`), shared with HOOKS1's battle unit
  meters (HOOKS1 UM4-UM5, HOOKS1-M).
- M6. Soft or hard is data: a rule whose effects hold `end` with kind `fail` is hard for its acts. Alert: the Act 1
  Breach rule ends the run; the Acts 2-3 rule queues a siege battle (reserved, Act 2 data). Doom Clock: Acts 1-2 at
  0 a lesser tithing (a vignette, a flag) and a restart; Act 3 at 0 the end.
- M7. Modifiers while a band holds (Lockdown's effects beyond gated routes and roamer markers): `road_pct`
  multiplies a leg's road-fight chance (capped 100) and `road_encounters` adds encounter refs to the leg's pool on
  routes carrying one of `route_tags` (all routes if empty). Several modifiers multiply in row order.
- M8. Visibility: a meter shows on the map HUD (region and world maps, Q-46) when `visible` is set in its state
  (`visible` row default, or the effects `meter_show` and `clock_start`); hidden meters still run.
- M9. Doctrine (Act 2, reserved shape only): `scope` `unit` keeps one value per member (`units` in the state) beside
  the cell value, bipolar ranges (`min` -100) are allowed; read keys `meter.<id>.lean.<member>`; inputs from grafts,
  Strain, forge projects and flushes come with Act 2. CAMPAIGN1 accepts only `scope` `campaign`; a `unit` row is a
  check error "doctrine lean: CAMPAIGN1-DOC". The endings' count of the dead and retired is ROSTER1's record.

## 5. The Doom Clock: a countdown on the travel clock `[kit hook]`, `[data]` the row

- K1. `clock_start {meter, days}` sets `due = gs.time + days` and shows it (the hearth scene: 42 days, day 1).
- K2. Days pass only through travel (each leg after its `time` is added), a rest (`rest.days`) and a retreat
  (`defeat.days`); each passage runs the `days` tick once per leg or step, with `n` days.
- K3. The correction and relabel at a story beat: `clock_set {meter, days, label}` sets the days left once (a `once`
  trigger or a dialogue choice behind a flag) and replaces the HUD name (the Red Tithing becomes the Great Tithing;
  Act 2, 2.4). `clock_stop {meter}` stops it (Act 3, node 21 falls).
- K4. A soft countdown reaching 0 runs its rule and restarts with `restart_days`; the lesser tithing count is a flag
  the rule adds (`add_flag lesser_tithings +1`) for scenes and endings. A hard countdown reaching 0 ends the run.
- K5. Within one tick a leg that jumps past several rule points fires them in order (7, then 0).

## 6. Checkpoints `[kit hook]` (world kind `checkpoints`), `[data]` which missions

- C1. A checkpoint names an act, a quest (met when its status is `done`) or `met_when` conditions, and a deadline day.
  It is met at the first tick where its condition holds on or before the deadline; it is missed at the first tick
  where `campaign.day > deadline` and it is not met. Each state is recorded once (`met`, `missed`). Planner ruling
  (2026-10-27, CAMPAIGN1-C): a quest checkpoint is met when the quest is `ready` or `completed` (the win counts, not
  the turn-in).
- C2. Warnings: on each `warn_days` value (days left before the deadline) one notice and a journal line.
- C3. Bad end by act (Q-20a): `mode` `end_run` (Act 1) puts the run over with the checkpoint's bad-end card; `go_on`
  (Act 2 on) runs the checkpoint's `effects` (lasting costs as flags, meter moves, a lost facility later) and the
  story continues.
- C4. A checkpoint is never locked by a reveal; only side quests lock (data: quest `requires_flags`).
- C5. Worst-case finish (for the balance lane): `aegis sim deadlines` (task file, CAMPAIGN1-SIM) computes per
  checkpoint the best and worst day it can be met and the slack; the target is in `decisions/balance.md` (planner).

## 7. Run over and the failure screen `[kit hook]` state, `[game code]` screen, `[data]` cards and text

- F1. The effect `end {kind, card, text}` puts the run over: `kind` `fail` (the Extinguished) or `bad_end` (a
  missed Act 1 checkpoint). The first `end` in a tick wins; the tick stops after it.
- F2. The game's three fail conditions are data: the Alert Act 1 Breach rule; a trigger `when roster.living < 1`
  (no living operative in the field or the bunks; ROSTER1 answers the key); the Doom Clock's Act 3 rule. Not a fail:
  a death, a lost siege, a soft clock at 0 (CANON 2.6).
- F3. While the run is over the GUI shows only the run-over screen; the world refuses saves (`save::write` returns
  "The run is over.") so neither the autosave nor a slot ever holds a run-over state.
- F4. The screen shows the card (STORYUI1's end card: art, title, text) and three buttons: "Load the act's start"
  (the act-start save, F5), "Load a save" (the existing load menu; slots and the autosave written before the fatal
  step), "Main menu". The exact offer is CMQ3.
- F5. Act-start save `[engine]` keyed saves: when `set_act` starts an act and the manifest's `act_saves` is on, the
  GUI writes a keyed save `act<n>_<run>` (`run` is the campaign's run id, set at a new game from the seed) after the
  act's effects. It is never overwritten by slot saves and not listed as a slot.

## 8. Purse names, upkeep, rests and retreats

- P1. `[kit hook]` resource purse: the manifest field `purse` names the currency (`party.gold`) and the upkeep
  (`party.supplies`) with icon and tip; Ashlands defaults are today's "Gold" and "Supplies" texts. Stash materials
  (Rock Salt, organs) are items (`material`) and need nothing new.
- P2. Upkeep per day: a leg spends `time x per_day`, a rest `rest.cost`, a retreat `defeat.days x per_day` (CMQ4).
  Never refused for lack of upkeep unless `rest.needs_upkeep` (Ashlands' Camp keeps its refusal).
- P3. At zero upkeep (`party.supplies == 0` when the battle starts or the leg starts): `at_zero.road_pct` (default 150,
  today's x1.5; `sunk_ward` 100, Q-18) and `at_zero.start_ap` (default 0; `sunk_ward` -2): every party unit starts
  the battle with that much less AP (not below 0), until restocked. `campaign::battle_mods(d, gs)` returns it; the
  battle bridge applies it. Nothing else happens at zero.
- P4. Prices per act (Act 3 Tallow costs more Scrap, Q-45; later): shop field `supply_steps` `[{requires_flags,
  price}]`, the first match replaces `supply_price` (`campaign.act >= 3` as the condition). Rock Salt's flush (Q-43)
  is an Act 2 exchange recipe, not this spec.
- R1. `[kit hook]` rest rule (manifest `rest`): `days` (default 0), `cost` (default 3), `where` (`nodes` default:
  Camp on any world-map node; `rest_places`: only where the location has `rest`; `none`), `on_enter` (default true:
  today's free heal on entering a rest place; `sunk_ward` false), `needs_upkeep` (default true).
- R2. A rest heals every member to full, runs ROSTER1's rest hook (clears wounded), spends `cost`, passes `days`
  (tick `days(n, rest)`) and autosaves. Refused when nobody is hurt or wounded (today's "Everyone is rested.").
- R3. Where: the world map's Camp button (when `where` allows the node) and the dialogue effect `rest` (the Dry
  Hollows' keeper or bunk marker, `[data]`). A battle costs no day (X2).
- R4. `[kit hook]` defeat rule (manifest `defeat`): `mode` `wake` (default, HEAL1 unchanged) or `retreat`: after a
  lost battle the cell moves to `to` (the hub), ROSTER1 applies the loss (Act 1: every deployed member wounded; HP
  is ROSTER1's), the encounter stays undone (the mission stays on the board), `days` pass (tick `days(n, retreat)`),
  autosave. Applies to all three `after_fight` paths. Until ROSTER1, a retreat does not heal unless the hub is a rest
  place with `on_enter`; a `to` that cannot be entered falls back to `wake` with a `retreat_failed` log note.

## 9. Triggers `[kit hook]` (world kind `triggers`), `[data]` rows

- T1. A trigger has `on` (`any`, `day`, `battle`, `enter`, `act`), a filter for its cause (`encounter` and `outcome`
  for battles, `location` for enter), `when` (flag conditions, read keys included) and `once` (default true; a
  repeatable one re-arms when `when` stops holding).
- T2. Firing runs `effects` at once (any dialogue effect that needs no screen: `set_flag`, `add_flag`, `start_quest`,
  `give`, `take`, `recruit`, plus section 11.3) and queues `then` items: `{vignette}` (STORYUI1), `{battle}`
  (encounter ref, fought as source `trigger`), `{dialogue, speaker}` (a scene). The queue is `campaign.pending`,
  saved; the GUI plays it head first when a map screen is on top; a queued battle settles like a dialogue fight.
- T3. The slice's minimum: day and battle triggers with effects and vignettes, the wiped-cell fail (F2), and the act
  change. Searchlight sweeps (timed route hazards), sieges and the final battle's retries are later rows on the same
  kind.

## 10. Warnings and the map HUD `[kit hook]` views, `[game code]` drawing, `[data]` texts

- W1. Every fired rule's `warn`, checkpoint warning and run over adds an event (`meter_warning`,
  `checkpoint_warning`, `run_over`) to the action's result; the GUI shows one notice per event, in order, and a log
  line.
- W2. Act 1 Alert (Q-13, Q-15): a warning at each threshold: entering Suspicious (31) and Lockdown (71), and a last
  one at 90; each says what changed and how to lower Alert (quiet jobs, bribes at Tess). Texts are data.
- W3. No fatal step without an ask (CMQ8): `campaign::preview_battle(d, gs, encounter)` and
  `campaign::preview_days(d, gs, n)` list the `end` effects the step would fire in its worst listed case (a battle's
  input outcomes; the days of a trip, rest or retreat). Before a marker or dialogue fight, a trip or a rest that
  would end the run, the GUI asks with No as the default; a road fight cannot be refused, so it gets the notice.
- W4. HUD (`campaign::hud(d, gs)`): currency and upkeep with the purse names, icons and tips, then the visible meters
  by `hud.order`: icon, value with unit ("12%", "37 days"), the band tone's colour; the tip gives the name (or the
  label), band name, the next rule point and the description. Ashlands: today's two counters, ids, texts and order.
- W5. The world map's Travel tip adds the arrival day and the days left ("day 9, 34 days left"); the mission board
  (STORYUI1) shows checkpoint deadlines from `campaign::checkpoints_view`.
- W6. No meter in battle (Q-46); a battle started at zero upkeep logs one line ("No Warded Tallow: -2 AP").

## 11. Schema contract (schema-first: yes; CAMPAIGN1-S)

### 11.1 World manifest fields (table `world`, type `json`, default `{}` = today's behaviour; nearest world wins)

| Field | Keys (default) | Validation |
| --- | --- | --- |
| `campaign` | `start_act` (0), `act_saves` (true) | `start_act` 0-9; presence switches X1 on |
| `purse` | `currency` {`name` ("Gold"), `icon` ("ui/hud/gold.png"), `tip` (today's)}; `upkeep` {`name` ("Supplies"), `short` (= name), `icon` ("ui/hud/supplies.png"), `tip` (today's), `per_day` (1), `at_zero` {`road_pct` (150), `start_ap` (0)}} | names 1-24 chars; `per_day` 0-5; `road_pct` 100-300; `start_ap` -5..0; icons resolve through the art layers (warning) |
| `rest` | `days` (0), `cost` (3), `where` ("nodes"), `on_enter` (true), `needs_upkeep` (true) | `days` 0-3; `cost` 0-10; `where` in `nodes`, `rest_places`, `none` |
| `defeat` | `mode` ("wake"), `to` (""), `days` (0) | `retreat` needs `to` resolving to a location; `days` 0-3 |

### 11.2 New world kinds (files `meters.json`, `checkpoints.json`, `triggers.json`; ids, `override`, `tags` as usual)

| Kind | Field | Type, default | Validation |
| --- | --- | --- | --- |
| meters | `name`, `description` | string "New Meter", text "" | name 1-32 |
| meters | `kind` | enum `gauge` / `countdown`, gauge | - |
| meters | `min`, `max`, `start` | int 0, 100, 0 | min < max; start in range; countdown: min 0, max 1-999 |
| meters | `unit` | enum "" / `%` / `days`, "" | countdown needs `days` |
| meters | `visible` | bool false | - |
| meters | `scope` | enum `campaign` / `unit`, campaign | `unit` refused until CAMPAIGN1-DOC (M9) |
| meters | `bands` | json [] | 1-6 `{id, name, from, tone}`; first `from` = min; ascending; ids unique; tone in M2's four |
| meters | `rules` | json [] | at most 12 `{id, at, dir, acts, once, warn, effects, then, restart_days}`; `at` a band id or int in range; `restart_days` only on countdowns |
| meters | `inputs` | json [] | at most 12 `{on: battle, tags, outcomes, delta, acts}`, `{on: day, delta, acts}`, `{on: leg, route_tags, delta, acts}`; gauges only; outcomes in won, lost, fled |
| meters | `modifiers` | json [] | `{bands, road_pct, road_encounters, route_tags}`; bands exist; encounters resolve |
| meters | `floors` | json [] | `{from_act, min}`; min in range |
| meters | `hud` | json {} | `{short, icon, order}`; short 1-16 |
| checkpoints | `name`, `act`, `deadline` | string, int 1, int 1 | act 1-9; deadline 1-999 |
| checkpoints | `quest`, `met_when` | ref "", json [] | exactly one set; quest resolves |
| checkpoints | `warn_days` | json [7, 3, 1] | ints 1-30, descending |
| checkpoints | `mode`, `card`, `text`, `effects` | enum `end_run` / `go_on` (end_run), string, text, json [] | `end_run` needs `card`; `go_on` needs effects |
| triggers | `name`, `on` | string, enum any / day / battle / enter / act (any) | - |
| triggers | `encounter`, `outcome`, `location` | ref, enum any / won / lost / fled, ref | only with their `on`; refs resolve |
| triggers | `when`, `once`, `effects`, `then` | json [], bool true, json [], json [] | at least one of `effects`, `then` non-empty; effect kinds as T2 |

Every `effects` list (meter rules, checkpoints, triggers, dialogue choices) is checked by the dialogue effect checker,
extended with 11.3. `then` items: exactly one of `vignette` (a STORYUI1 vignette id), `battle` (encounter ref),
`dialogue` (dialogue ref, optional `speaker`).

### 11.3 New effect kinds (dialogue choices, meter rules, checkpoints, triggers)

| Effect | Fields | Rule |
| --- | --- | --- |
| `meter_add` | `meter`, `delta` (int, non-zero) | M3, M4 clamp; tick `effects` after the list |
| `meter_set` | `meter`, `value` | gauge only; clamped |
| `meter_show` | `meter`, `visible` (true) | M8 |
| `clock_start` | `meter`, `days` 1-999 | countdown only; K1 |
| `clock_set` | `meter`, `days` 0-999, `label` ("" keeps) | K3 |
| `clock_stop` | `meter` | K3 |
| `set_act` | `act` 0-9 | floors (M4), act triggers, the act-start save (F5) |
| `rest` | - | R2 with the world's rule; dialogue precheck gives R2's refusal text |
| `end` | `kind` (`fail` / `bad_end`), `card`, `text` | F1; not allowed in dialogue choices with other effects after it |

### 11.4 Read keys (answered by `quest::condition_met` in a campaign world; read-only)

| Key | Value |
| --- | --- |
| `meter.<id>` | int: a gauge's value, a running countdown's days left, a stopped countdown's `max` |
| `meter.<id>.band` | string: the band id |
| `meter.<id>.running` | bool: countdowns |
| `campaign.act`, `campaign.day` | int |
| `roster.living` | int: ROSTER1's living operatives (field and bunks); until ROSTER1 lands, the member count |

`set_flag` and `add_flag` refuse keys starting `meter.`, `campaign.` or `roster.` in every world ("reserved key");
no shipped flag uses them (checked 2026-10-25). The content check reports a condition naming an unknown meter.

### 11.5 State `gs.campaign` (saved key `campaign`, X3)

| Key | Shape | Default |
| --- | --- | --- |
| `act` | int | manifest `start_act` |
| `run` | string, 8 hex digits from the new game's seed | set at a new game |
| `meters` | `{id: {value} or {due, running}, visible, label}` | each row's `start` and `visible` |
| `fired` | sorted list `rule:<meter>/<rule>`, `trigger:<id>` (once-items fired) | [] |
| `armed` | list of repeatable trigger ids whose `when` held at the last tick | [] |
| `checkpoints` | `{id: "met" or "missed"}` | {} |
| `pending` | list of `then` items | [] |
| `over` | `{kind, card, text}`; runtime only, never saved (F3) | absent |

Validation (`GameState::validate`): meter ids resolve, values in range, `act` 0-9, pending items well formed.
Save: version 13 (or the next free number; sequence with ROSTER1, RULESET1-D, ENCOUNTER2-W); the step adds nothing
(an absent `campaign` reads as a fresh one at the current time when the world has a campaign, else empty).

## As built (2026-10-29, CLEANUP.md CLN-24)

- CAMPAIGN1-C (c9fbd8c): the defeat retreat as R4 in section 8, including its fallback: a `to` hub that cannot be
  entered falls back to the wake (`retreat_failed` log note).
