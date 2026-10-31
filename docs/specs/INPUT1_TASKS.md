# INPUT1 tasks: schema, kit and host API, tests, parity and the task split

Status: proposed (spec 2026-10-31, task INPUT1-S; revised 2026-11-01 with "INPUT1 answers", parts 1-3); companion
of [INPUT1.md](INPUT1.md), whose rule ids it cites.
Paths: `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`, `aegis_*` under
`engine/crates/`.

## 1. Schema contract (schema-first: yes for INPUT1-L's field and INPUT1-M's fields; the rest is code and settings)

### 1.1 Unit and member `deck.follow_ups` (INPUT1-L, FL1)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `deck.follow_ups` | object: family id -> list of family ids | {} | key: a family in `deck.active` or the ruleset's `basics`, intent (IN4) not neutral; values: families in `active` or `basics` with the key's intent; no repeats in a list; the key may be in its own list; an empty list is dropped |

`rpgkit_data::Deck` gains `follow_ups: IndexMap<String, Vec<String>>` (written order kept) and `Deck::KEYS` becomes
`["active", "passive", "keystone", "order", "follow_ups"]`; `from_value` reads it, the writer omits it when empty
(so every deck without lists writes byte-identical). `kits/rpg/schema.json`'s unit `deck` help names the key. The
content check (`rpgkit_content/src/check/unit.rs` `deck_errors`) and the world's `GameState::deck_errors`
(`rpgkit_world/src/state.rs`) give FL5's messages. No save version bump (Q9 a).

### 1.2 Kit API (`rpgkit_battle::round_intent_plan`, INPUT1-U)

The names are the contract; signatures may take `&Plan` where the builder needs it.

| Item | Contract |
| --- | --- |
| `enum Pick` | `Skill { skill: SkillId, kind: Option<ManeuverKind> }`, `Item(String)`, `Rest`, `MoveHere(V2)`, `StepAfter(V2)` |
| `RoundState.picks: Vec<Pick>` | the member's picks this round; cleared in `round_intent::round_start` (GH6); not in any trace or save |
| `World.autofill: bool` | default true; only `set_autofill` changes it (GH7); the sims and the AI never do |
| `hotkey_skills(w, c) -> Vec<SkillId>` | HK1's order |
| `pick_options(w, plan, c, at: RoundTarget) -> Vec<PickOption>` | read-only; the rows of IF2 (both forms: `after_first` filters to placeable rows) / ET1 in order; `PickOption { picks: Vec<Pick>, intent: Option<RoundIntent>, kind: Option<ManeuverKind>, ap: i32, blocks: i32, problem: Option<String>, drops: Vec<String> }` (`picks` has two entries for "Move here, then X") |
| `pick(w, plan, c, at, picks: &[Pick]) -> (Result<(), String>, Vec<(UnitId, String)>)` | atomic (IF6, GH3-GH5, ET5); sets the target when no pick holds it, the intent and kind from the first intent pick; rebuilds the member, then later members (`replan`'s shape: the dropped picks of later members with reasons) |
| `unpick(w, plan, c)`, `clear_picks(w, plan, c)` | IF7; rebuild as `pick`; with no pick left the member's plan is `plan_unit`'s; the game then applies GH2 (a) or (b) |
| `set_autofill(w, plan, on)` | rebuilds every party member's ghost |
| `ghost_from(w, plan, c) -> usize` | the index (in `Plan::entries_for`) of the first ghost entry |
| `preview_pick(w, plan, c, at, picks) -> Result<(Vec<Step>, ApForecast), String>` | IF5, on clones; changes nothing |
| `ap_forecast(w, plan, c) -> ApForecast` | AP1: `{ now, spent, picked, filled, left, gain, rest_gain, next, max }`, computed with `status::roll_over`'s and `round_intent::round_start`'s own rules |
| `regular_move(plan, c) -> Option<MoveKind>` | ET5: the round's one regular move if planned (approach, kite, guard tile, move here, out of range, kite after, step after); the carried move never counts |
| `follow_order(w, c, intent, opener: Option<&Opener>) -> Vec<SkillId>` | FL2 (INPUT1-L): the opener's list, then `order[intent]` (else the auto order), a family in both at its first place; FU2's fill calls it where it calls `order` today |

`plan_unit`, `replan`, `fill_rest`, `preview`, `rest_preview`, `set_manual` stay (the Dojo, the sims and old host
requests use them). GH2's tactic ghost is the game's: it calls today's `Session::auto_draft` for a member with no pick
and no player-set target, and `plan_unit` for one with a player-set target; the kit has no new state for it.

### 1.3 Host requests (`rpgkit_host`, INPUT1-U; `new` gains `melee_diagonal` in INPUT1-M)

| Request | Recorded | Reply |
| --- | --- | --- |
| `pick {unit, target, pick, then?}`; `pick`: `{"skill": id, "kind"?: "engage"\|"disengage"\|"utility"}`, `{"item": id}`, `"rest"`, `{"move": [x, y]}`, `{"step_after": [x, y]}`; `then`: a skill pick after a move | yes (joins the recorded list beside `round_intent`) | `intent_reply`'s shape plus `dropped` [{unit, reason}] |
| `unpick {unit}` | yes | `intent_reply` |
| `autofill {on}` | yes | `{ok, autofill}` plus `round_plan` |
| `pick_options {unit, target}` | no (read-only) | `{ok, rows: [{picks, intent, kind, ap, blocks, problem, drops}]}` |
| `clear {unit}` (exists) | yes | under `round_intent`: clears the picks (IF7) |
| `round_plan` (exists) | no | per member adds `picks`, `ghost_from`, `autofill`, `ap` {now, spent, picked, filled, left, gain, rest_gain, next, max}; `manual` stays (true when the member has a pick); never an enemy, as today |
| `new {..., melee_diagonal?}` (exists; INPUT1-M) | yes | `ruleset` gains `melee_diagonal` |

`round_intent`, `manual` and `fill` stay for recordings, sims and tools; the battle screen stops sending them. The
sim drivers (`feel`, `timed`, autoplay, `Session::auto_draft`) send none of the new requests.

### 1.4 Engine (`aegis_ui`)

1.4a `aegis_ui::timeline` (INPUT1-U): `Segment` gains `ghost: bool` (default false): drawn with its fill at 45% alpha,
a dashed outline and the label in `DIM`; `pub fn draw_ghost(ui, bar: Rect)` gives the battle strip the same look for
its own spans. Game-agnostic (the module doc names "a planned, not chosen, span"); every existing caller is unchanged
(default false).

1.4b `aegis_ui::pips` (new, INPUT1-AP, AP6-AP8):
`PipGauge { pips: Vec<Pip>, alert: bool, before: Option<String>, after: Option<String> }`,
`Pip { fill: PipFill (Full, Ghost, None), outline: PipOutline (Solid, Slot) }`;
`pub fn draw(ui, at: V2, gauge: &PipGauge, tone: Tone)` with the pip size, gap and ghost alpha as constants (4 x 6, 1, 45%);
`alert` draws every outline and the labels in the alert tone. Game-agnostic (no AP in the engine: any pip count);
the game builds the gauge from `ApForecast` (AP7) and owns `AP_PIPS` 12.

### 1.5 Settings (game, `screens::default_settings`, Settings screen in `menus.rs`)

| Key | Tab | Default | Notes |
| --- | --- | --- | --- |
| `auto_fill` | Gameplay | true | GH7, checkbox `settings.auto_fill` (INPUT1-U) |
| `battle_overlays.first_move` | Battle | false | VC1 (VISCLAR1-A) |
| `battle_overlays.all_bars` | Battle | false | VC3 (VISCLAR1-A) |
| `debug_party_foresight_off` | Battle, "Debug" heading | absent (= false) | VC4, VC5: debug builds only (VISCLAR1-A) |
| `battle_overlays.tracers` | Battle | true | PB1 (INPUT1-PB) |
| `playback_speed` | Battle | "1x" | PB3: "1x", "0.5x", "block"; select `settings.playback_speed` (INPUT1-PB) |

`battle::OVERLAY_GROUPS` gains a default per group (VC6). The Controls list: "Skills of the selected hero
(timeline): 1-9", "Target self: S", "Inspect a unit: right click", "Full timeline: T", "Every bar and arrow: hold
Alt", "Playback speed: P", "Next block: N".

### 1.6 Agent link (game)

Widget ids: menu rows as the popup's (`popup.row.<k>`), `hero.self`, `strip.ap` (the gauge), `timeline_view` (the
engine widget's ids under it), `skills.follow.*` (FL3), `dojo.deck.follow.*` (the Dojo's mirror),
`settings.auto_fill`, `settings.overlay.first_move`, `settings.overlay.all_bars`, `settings.overlay.tracers`,
`settings.playback_speed`, `settings.debug.party_foresight_off`, `gallery.timeline_pane`, `gallery.pip_gauge`,
`playback.speed.1`, `playback.speed.half`, `playback.speed.block`, `playback.next`, the Arena card's
`arena.melee_diagonal`. Battle state keys: `picks` [{unit, picks}], `ap_gauge` {now, spent, left, next, max, unit,
red, pips: ["F", "Hg", ...] as INPUT1.md 7's legend}, `inspect` (unit or null), `focus` [units], `timeline_view`
{open, tracks [{key, masked, segments}]}, `overlay.arrows` [{unit, points, alpha}], `overlay.tracers` [{from, to,
alpha, dashed}], `playback` {speed, waiting, block}, `log_groups` [{round, block, lines}], `ruleset.melee_diagonal`.
Removed: `hero.intent.<id>`, `hero.kind`, `hero.opener`, `hero.manual`, `hero.fill`; the state key `intents` gives way
to the popup rows.

### 1.7 Skill and ruleset fields (INPUT1-M, MR1, MR9, MR10)

| Kind.key | Type | Default | Validation (content check; message) |
| --- | --- | --- | --- |
| `skills.maneuver_range` | int 0-6 | 0 | only on a skill whose round intent is Maneuver: "maneuver_range: only a Maneuver skill has a range"; 0 on a melee Maneuver ATTACK warns "no carried move: range = band" (MV5's `dash_tiles` 0 warning moves here) |
| `skills.position` | enum "", `flank`, `behind` | "" | only on a melee Maneuver ATTACK (band max 1): "position: only a melee Maneuver attack names a position"; "" reads `flank` |
| `skills.dash_tiles` on a Maneuver skill | int | 0 | with no `maneuver_range`: warning "dash_tiles on a Maneuver skill is read as maneuver_range (move it)"; with both set: error "set maneuver_range only" |
| `rulesets.melee_diagonal` | enum `none`, `maneuver`, `all` | `maneuver` | `maneuver` without `round_intent` in `rules`: warning "melee_diagonal maneuver needs round_intent (reads as none)" |
| `rulesets.flank_protected_cost` | enum `full`, `half`, `fixed`, `none` | `full` | - |
| `rulesets.flank_protected_ap` | int 0-6 | 1 | set while the cost is not `fixed`: warning "flank_protected_ap is read only with fixed" |

Kit types: `rpgkit_data` skill row `maneuver_range: i32` and `position: Position` (`Flank`, `Behind`; "" = `Flank`);
`rpgkit_rules::ruleset::Numbers` gains `melee_diagonal: MeleeDiagonal` (`None`, `Maneuver`, `All`) and
`flank_protected: FlankCost` (`Full`, `Half`, `Fixed(i32)`, `None`); `Ruleset::classic()` gives `None` and `Full`.
Field help in `kits/rpg/schema.json` (skills: "Maneuver: how far the skill dashes to its position before it hits
(move cost, one block)"; "Maneuver: where it dashes: flank (beside the target) or behind; taken: the nearest
diagonal"; rulesets: "Diagonal melee: none, Maneuver skills only, or every melee attack"; "What a Maneuver whose
flank is protected pays"). `dash_tiles`'s help stays word for word (Godot help comparison).

### 1.8 Kit API (INPUT1-M)

| Item | Contract |
| --- | --- |
| `round_intent::positions(w, t, attacker, position) -> Positions { preferred: Vec<V2>, fallback: Vec<V2> }` | MR2-MR3: open tiles only, `melee_diagonal` applied, in MR4's rank order |
| `round_intent::maneuver_range(skill) -> i32` | MR1: `maneuver_range`, else `dash_tiles` (the transition) |
| `round_intent::flank_protected(w, t, attacker, skill) -> bool` | MR5: no open candidate |
| `command` `carried_engage` | MR8 (a)-(c), MR9; log `[Flank Protected]`; a protected command carries a `missed` flag its impact reads |
| band test (`in_reach`, the planner's band tiles, the chase) | MR10 through one helper, e.g. `round_intent::melee_dist(w, skill, a, b)` |
| `rpgkit_sim` arg `melee_diagonal=` | MR11; joins the args help |
| `skill_text` `{range}`, `{position}` | MR15 |

## 2. Layers (CLAUDE.md; the kit names no game)

| Layer | What |
| --- | --- |
| engine | `aegis_ui::timeline` `Segment.ghost`, `draw_ghost` (1.4a); `aegis_ui::pips` (1.4b) |
| kit | picks, options, ghost, Auto-fill flag, AP forecast, hotkey order, the one regular move (1.2); host requests (1.3); `deck.follow_ups`, its checks, save and the planner's reader (1.1, FL2); the Maneuver fields, positions, protected flank, `melee_diagonal`, sim arg and skill text (1.7, 1.8) |
| game | the battle screen's flow, menus, hotkeys, selection, inspect, the AP gauge, timeline view (`timeline_view.rs`), the playback aids, the settings and the Controls list, VISCLAR1-A, the deck screen's lists, the Dojo's Deck window, the Gallery entries, the Arena's "Diagonal melee" select; data: INPUT1-MA |

The setting's name "Auto-fill the rest", every label, `AP_PIPS` and the Gallery panes' sample data are the game's; the
kit has the bool `autofill` and returns refusal reasons in today's style. The words "flank" and "behind" are the
kit's neutral position ids (no lore).

## 3. Tests (rule-derived)

| Task | Test file | Checks |
| --- | --- | --- |
| INPUT1-M (kit) | `rpgkit_battle/tests/maneuver.rs` (new); `round_intent.rs`, `round_intent_plan.rs`, `intent_ai.rs`, `reach.rs` updated where MV5-MV7 moved | MR2's table for f (1, 0) and (1, 1) and the six rotations; MR3 open tiles, claims, `none` dropping diagonals; MR4 tiers, the worked examples, s itself; MR5 "Flank protected" and PL3 skipping; MR6 follow-ups; MR7 the out-of-range move and no after-move; MR8 (a)-(c) at run time, the chase budget N + 1, no hit event on a miss, the wind-up still breakable, enemy recovery; MR9 each cost value; MR10 a Strike from a diagonal under each value, the riposte under `all` and `maneuver`, MV2 under `all`; MR14 row 4; MR15 texts; the `dash_tiles` fallback warning and the both-set error |
| INPUT1-M (content, host, sim) | `rpgkit_content` check tests; `rpgkit_host` tests; `rpgkit_sim` args test | 1.7's messages; `new {melee_diagonal}` recorded and replayed; the sim arg overrides the row |
| INPUT1-M (game) | `runner.rs` tests; `timeline_golden` | MR11 the Arena select reaches the request and survives Restart; the golden re-recorded (5) |
| INPUT1-MA | the content check | 11.7's acceptance: no fallback warning; the three texts |
| INPUT1-U (kit) | `rpgkit_battle/tests/picks.rs` (new) | IF2 rows by target kind, both forms; IF6 a-e refusals with their texts; IF4 intent and kind from the first pick, neutral picks set none; GH3 a pick replaces the ghost from its block and the fill rebuilds after it; Auto-fill off leaves Rest; GH4 the Step-after block reserved; GH5 atomic refusal, a later member's dropped pick; GH6 picks cleared at round start, the intent kept; HK1 order; ET2-ET5 refusals and the one-move rule; `ap_forecast` on section 4's four examples; `ap_forecast.next` equals the next round's AP on a dummy fight; `preview_pick` leaves `World` and `Plan` equal |
| INPUT1-U (host) | `rpgkit_host` tests | `pick`, `unpick`, `autofill`, `pick_options`, `round_plan`'s new keys; a refused `pick` leaves the battle unchanged; a recording with `pick` replays to the same state; an old recording with `round_intent` / `manual` / `fill` still replays |
| INPUT1-U (engine) | `aegis_ui` timeline tests | a ghost segment draws with the ghost style; a default segment draws as before (pixel compare) |
| INPUT1-U (game) | `battle.rs` `input1_tests` | IF1 click sets target and opens the menu; IF2 the re-listed menu after a pick; IF10 closing clicks do nothing else; IF8 none of the removed ids exists; HK2-HK3 digits; GH2 (a) the tactic ghost, (b) a player-set target's auto plan, Clear returning to each; SL1-SL7; IS1-IS3 (an unrevealed enemy's inspect text holds no plan skill); IF9 tactic skips members with picks; GH7 the setting rebuilds ghosts; MR13 the hovered Engage row's marks; `text_audit` at 0.85, 1.0, 1.15, 1.3 for the menu and the hero panel |
| INPUT1-AP | `aegis_ui` pips tests; `battle.rs` `ap_gauge_tests`; `gallery.rs` tests | 1.4b states draw (pixel compare); AP7 the four rows of INPUT1.md 7; AP8 red, shortfall and debt labels; AP6 14 AP at 2 a pip; AP9 tip lines; AP10 drain in playback and instant under skip animations; the Gallery's `pip_gauge` renders with no session |
| INPUT1-T | `battle.rs` `timeline_view_tests`, `gallery.rs` tests | TV1 T and Esc; TV2 sections and order; TV3 picked and ghost segments, AP9 in the label tip; TV4 masked note text, badges as marks, playback holds only started actions, a revealed track unmasked; TV5 track click selects; TV6 `text_audit` at four sizes, 16 tracks before scroll; TV7 the Kit pane renders with no session; battle view and Gallery build equal `Timeline` values |
| INPUT1-L | `rpgkit_battle/tests/picks.rs`, `rpgkit_content` check tests, `rpgkit_world` save tests, `party.rs` tests | FL1-FL5 as INPUT1.md 9's acceptance (B, then A from the order); FL3 widget flow (open, add, reorder, remove, clear); the Dojo's Deck window edits rebuild the hero |
| INPUT1-PB | `battle.rs` `playback_tests` | PB1 tracer entries, dashed misses, none for self, the group off; PB2 the framed block follows the playback; PB3 the three speeds, P and N, the setting kept, 0.5x doubling a block's time, Block under skip animations; PB4 every line under its block's header, round start and end groups, `text_audit` at four sizes; PB5 T pauses at the block end and closing resumes |
| VISCLAR1-A | `battle.rs` `visclar_tests` | VC1-VC6 and AR1-AR6 as INPUT1.md 10's acceptance; `party_foresight_off(_, false)` is false; Restore defaults gives `first_move` and `all_bars` off |

## 4. Slices and task rows (the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- | --- |
| INPUT1-S | Spec: the one-flow battle input, the Maneuver rework, the AP gauge, playback aids and VISCLAR1 slice A | game + kit | design | RULESET1-U, RULESET1-PL | done (spec 2026-10-31, revised 2026-11-01 with the answers, parts 1-3) | `docs/specs/INPUT1.md` (+ `_TASKS.md`) | `docs/specs/INPUT1*.md` |
| INPUT1-M | Maneuver rework (INPUT1.md 11): range N as the dash to the skill's position (`maneuver_range`, `position` flank or behind), the nearest diagonal fallback, "Flank protected" (disabled while planning; a miss with its AP at run time, cost a rule value), the chase, `melee_diagonal` (none, maneuver, all; start at maneuver) with the Arena select and sim arg; re-record the timeline golden (HISTORY rows) | kit + game | logic | INPUT1-S | spec | `docs/specs/INPUT1.md` 11 | `kits/rpg/schema.json`; `rpgkit_data/src/lib.rs`; `rpgkit_rules/src/ruleset.rs`; `rpgkit_content/src/check/{skill,ruleset}.rs`; `rpgkit_battle/src/{round_intent,round_intent_plan,command,skill,ai}.rs`, `rpgkit_battle/tests/{maneuver,round_intent,round_intent_plan,intent_ai,reach}.rs`; `rpgkit_world/src/skill_text.rs`; `rpgkit_host/src/lib.rs`; `rpgkit_sim/src/args.rs`; `emberwake_gui/src/{runner,intent_text}.rs`; `emberwake_battle/tests/fixtures/{timeline_kit,timeline_golden}.json`; `emberwake_world/tests/data_source.rs` (only if the Godot data comparison sees the new fields) |
| INPUT1-MA | Maneuver skill data: `maneuver_range` and `position` on Flank Cut, Hamstring Cut, Slip Strike; the timeline row's `melee_diagonal` and `flank_protected_cost` (INPUT1.md 11.7) | game | asset | INPUT1-M | spec | `docs/specs/INPUT1.md` 11.7 | `games/emberwake/data/catalog/{skills,rulesets}.json` (through `aegis content`), new `games/emberwake/data/seeds/input1_ma.json` |
| INPUT1-MB | Feel run after the Maneuver rework (INPUT1.md 11.8) | game | balance | INPUT1-MA (on `main`) | spec | `docs/specs/INPUT1.md` 11.8 | `games/emberwake/docs/BALANCE_REPORT.md` |
| INPUT1-U | Battle input as one flow (sections 2-6, AP1-AP2's forecast): target then skill, intent from the skill, the re-listed menu after a pick, the tactic ghost, Auto-fill, keys 1-9 as skill hotkeys, empty-tile moves, the one regular move and Step here after for every intent, selection and inspect, the Engage row's position marks | engine + kit + game | logic | INPUT1-S, CAMPAIGN1-U (done), ROSTER1-L, INPUT1-M (shared kit files) | spec | `docs/specs/INPUT1.md` 2-7 | `aegis_ui/src/timeline.rs`; `rpgkit_battle/src/{round_intent_plan,round_intent}.rs`, `rpgkit_battle/tests/picks.rs`; `rpgkit_host/src/lib.rs`; `emberwake_gui/src/{battle,menus,screens,intent_text}.rs` |
| INPUT1-AP | AP pip gauge (INPUT1.md 7, AP3-AP10): 12 pips, hollow spent, ghost gain, red and debt, numbers on hover, drain in playback; the engine pip widget and the Gallery's `pip_gauge` | engine + game | logic | INPUT1-U | spec | `docs/specs/INPUT1.md` 7 | new `aegis_ui/src/pips.rs`, `aegis_ui/src/lib.rs`; `emberwake_gui/src/{battle,gallery}.rs` |
| VISCLAR1-A | VISCLAR1 slice A: first-move chip behind a setting, bars in focus, one quiet arrow per unit while planning only, debug Party foresight off (INPUT1.md 10) | game | logic | INPUT1-AP (shared `battle.rs`) | spec | `docs/specs/INPUT1.md` 10 | `emberwake_gui/src/{battle,menus,screens}.rs` |
| INPUT1-T | Full timeline view (key T) and the Gallery's Timeline pane (INPUT1.md 8) | game | logic | VISCLAR1-A (shared `battle.rs`) | spec | `docs/specs/INPUT1.md` 8 | `emberwake_gui/src/{battle,gallery}.rs`, new `emberwake_gui/src/timeline_view.rs` |
| INPUT1-PB | Playback aids (INPUT1.md 12): impact tracers, the strip's block cursor, playback speed 1x / 0.5x / block by block, the log grouped by block, the timeline view pausing playback | game | logic | INPUT1-T | spec | `docs/specs/INPUT1.md` 12 | `emberwake_gui/src/{battle,timeline_view,menus,screens}.rs` |
| INPUT1-L | Per-opener follow-up lists, the list first then the deck order (INPUT1.md 9) | kit + game | logic | INPUT1-U | spec | `docs/specs/INPUT1.md` 9 | `rpgkit_data/src/lib.rs`, `rpgkit_content/src/check/unit.rs`, `rpgkit_world/src/state.rs`, `rpgkit_battle/src/round_intent_plan.rs`, `rpgkit_battle/tests/picks.rs`, `kits/rpg/schema.json`; `emberwake_gui/src/{party,dojo}.rs` |

Merge order and parallel work:

1. INPUT1-M first: it can start now beside ROSTER1-L (it touches no file ROSTER1-L's Touches name in
   `emberwake_gui` beyond `runner.rs`, which the planner checks), and INPUT1-U waits on ROSTER1-L anyway. It goes
   before INPUT1-U because both edit `round_intent.rs` and `round_intent_plan.rs`.
2. INPUT1-MA (asset) after INPUT1-M merges; INPUT1-MB (balance) once INPUT1-MA is on `main`. Both run beside the
   INPUT1-U chain (no shared files).
3. INPUT1-U after ROSTER1-L and INPUT1-M. It may land in three slices: (1) engine and kit with host (1.2-1.4a, kit and
   host tests); (2) the battle screen's flow, hotkeys, empty tiles, ghost look and the setting; (3) selection and
   inspect.
4. The `battle.rs` chain, one at a time: INPUT1-AP, then VISCLAR1-A, then INPUT1-T, then INPUT1-PB (INPUT1-AP first so
   playtests of INPUT1-U get the AP display soon; INPUT1-T before INPUT1-PB, whose PB5 pauses the view).
5. INPUT1-L after INPUT1-U, beside the whole step-4 chain (its files are the kit's deck, content, world, the planner's
   `round_intent_plan.rs`, and `party.rs` and `dojo.rs`; none of step 4 edits them). With INPUT1-M merged, nothing
   else edits `round_intent_plan.rs` at that time.

Row changes asked of the planner: INPUT1-S moves to done; INPUT1-M's Layer becomes "kit + game" (its data is
INPUT1-MA) and its Touches as above; new rows INPUT1-MA (asset) and INPUT1-MB (balance); INPUT1-U's Layer becomes
"engine + kit + game", it depends on INPUT1-M too, and its title adds the re-listed menu, the tactic ghost and the one
regular move, and drops "the AP readout"; INPUT1-AP's Layer becomes "engine + game" (the pip widget) and its Touches
add `aegis_ui/src/pips.rs` and `gallery.rs`; INPUT1-T's Layer becomes "game" (the pane is the engine widget INPUT1-U
extends, shown by the game's Gallery; no kit code) and it depends on VISCLAR1-A for the shared file; INPUT1-PB's
Layer becomes "game" (no engine change: the widget already has a playhead, and the kit's log entries already carry
round and block) and its Touches lose `aegis_ui/src/timeline.rs`; a new logic row VISCLAR1-A (the VISCLAR1 design row
keeps the later slices). Statuses: the user approved every row on 2026-10-31; the planner keeps them `approved` if
that approval covers this revision, else `spec`.

## 5. Parity

INPUT1-U, -AP, -T, -L, -PB and VISCLAR1-A are input and display: no rule, number or AI choice changes. The Godot
parity suites (classic battle traces, world battles, journeys, saves, loader and writer) are untouched: classic
input and playback are unchanged, `deck` is already stripped from the Godot data comparison (HISTORY 2026-10-25
"Godot parity over the timeline content fields"), Godot saves carry no deck. The timeline golden traces (RULESET1-X)
do not move for them: they come from the kit's auto driver, which sends no pick; a member with no pick plans exactly
`plan_unit`'s plan (GH2's tactic ghost is the game's); `autofill` is true unless a request sets it; `follow_order`
with no list returns `order` (FL2); picks and the ghost index are in no trace. Every golden and parity test passes
byte-identical for those tasks, and a moved golden is a bug to report, not to re-record.

INPUT1-M changes rules on purpose (CLAUDE.md "Godot parity": an intended change retires the checks it affects in the
same change):

- Retired and re-recorded: `games/emberwake/crates/emberwake_battle/tests/fixtures/timeline_golden.json` (test
  `timeline_golden`, `RECORD=1 tools/fc.sh test --test timeline_golden`), the fights whose Maneuver commands now
  dash to a position, fall back, miss as protected or hit diagonally. Expected: `tl_flank` and `tl_skirmish` (the
  SKIRMISHER `tl_spiker` Engages, and `flank_cut` is a basic of every unit) at seeds 1337, 7 and 42; the three duels
  should not move (no Maneuver round in them today). The builder names every fight and round that moved in the commit
  and the HISTORY row; a duel that moves is reported with its cause before re-recording.
- The frozen catalog `timeline_kit.json` gets the same three skill changes as INPUT1-MA (11.7) and the timeline row's
  `melee_diagonal` `maneuver`, in the same change (an intended input change, named in the HISTORY row), so the golden
  plays the shipped rules; the `dash_tiles` fallback is covered by its own kit test.
- Not retired: the classic battle traces, world battles, journeys and saves (classic has no row: `melee_diagonal`
  `none`, no Maneuver intent); the inert-extensions check (classic with every extension on: `maneuver` reads as
  `none` without `round_intent`); the Godot loader and writer (the new fields are timeline fields, stripped as the
  others are if the comparison sees them; `dash_tiles`'s help is unchanged).
- The feel sim's numbers move: INPUT1-MB measures them (11.8). INPUT1-MA changes no golden (the fixture is frozen).

## 6. Wanted HISTORY rows (the planner writes them)

| Date | Idea | Status | Replaced by | Reason |
| --- | --- | --- | --- | --- |
| 2026-10-31 | The round controls of RULESET1-U slice 1 (intent buttons, keys 1-9 as intents, the Maneuver kind and opener menus, the Manual toggle, Fill; RULESET1_INTENT UI1, MM1-MM3 as a mode) | replaced | INPUT1: target, then a skill; intent and kind from the skill; ghost fill; the setting Auto-fill the rest; keys 1-9 as skill hotkeys | User decision "Combat input feedback": fewer controls a round, the intent read from the chosen skill |
| 2026-10-31 | A hand-picked carried-move tile (RULESET1_INTENT MM2, MM4) | dropped | the Maneuver position rule (INPUT1 MR2-MR4) picks the tile | "INPUT1 answers, part 2" Q3: each skill names its position; one input flow |
| 2026-10-31 | Several lines per unit on the board: tell arrows, the hero's planned-move lines, revealed enemy walks (also the hovered enemy's walk above the tokens, RULESET1-U slices 5-6) | replaced | One quiet dashed target arrow per unit (move leg when the move is visible), the selected unit's by default, all with Alt, while planning only (INPUT1 AR1-AR6) | User addition 2026-10-31 and "INPUT1 answers, part 1" Q6: a calmer Stage, nothing that hides the fight in playback |
| 2026-10-31 | The enemy first-move chip on tokens by default (INTENT1-U) | narrowed | Behind the debug overlay group `first_move`, off by default | User decision "Combat input feedback" (VISCLAR1 slice A) |
| 2026-10-31 | The AP text readout at the strip's end (`AP 6 - 4 = 2 · +3 next -> 5`, decision "Combat input feedback"; spec INPUT1 AP1-AP5 first draft) | replaced before it was built | the AP pip gauge (INPUT1 AP6-AP10, 12 pips first) | "INPUT1 answers, part 2" Q7 |
| 2026-10-31 | A per-opener follow-up list replacing the intent's order (INPUT1 first draft FL2, option a) | not adopted | the list first, then the deck order (FL2, option b) | "INPUT1 answers, part 1" Q2 |
| 2026-10-31 | The Maneuver Engage destination as any tile outside the target's front within `dash_tiles`, ranked by PL4, and Engage not offered without one (RULESET1_INTENT MV4-MV7) | replaced | a named position (flank or behind, skill field `position`) within `maneuver_range`, the nearest diagonal as fallback, "Flank protected" when all are taken (a miss with its AP at run time; INPUT1 MR1-MR9) | "INPUT1 answers, part 2" Q3 and part 3: repositioning and crossing the line quickly; a protected flank points Maneuver at an unprotected back line |
| 2026-10-31 | Orthogonal-only melee under the timeline ruleset (a band of 1 is Manhattan distance 1, so a diagonal neighbour is out of reach) | narrowed | the rule-library option `melee_diagonal` (none, maneuver, all; the timeline row starts at maneuver: Maneuver skills hit diagonally; classic stays none) | "INPUT1 answers, part 2" and "part 3" |
| (INPUT1-M's date) | The timeline golden's Maneuver rounds (`tl_flank`, `tl_skirmish`, seeds 1337, 7, 42; the builder lists the fights and rounds) and the frozen `timeline_kit.json` rows of Flank Cut, Hamstring Cut, Slip Strike | re-recorded | the same golden re-recorded with INPUT1-M (positions, the diagonal fallback, protected flanks, diagonal Maneuver hits); the fixture rows carry `maneuver_range` and `position` | An intended rule change (decision "INPUT1 answers", parts 2 and 3); classic and Godot parity unchanged |

Decision rows touched (for the planner): "Combat input feedback" gains the arrow addition (AR1-AR5); it supersedes,
for arrows, that row's "arrows ... for the selected member, its target and the unit under the cursor" (bars keep
that focus set) and "RULESET1-U slice 5 answers"' "draws the hovered enemy's revealed moves above the tokens", and
the RULESET1-U row's "draw a revealed enemy's planned walk only for the hovered enemy or while Alt is held". The AP
part of "Combat input feedback" ("a readout at the strip's end ... red when a pick cannot be paid") is superseded by
"INPUT1 answers, part 2" Q7 (the gauge; red stays). Nothing supersedes the classic rows "Gap-close" and "Tuning notes
from M1" (classic keeps `dash_tiles` and distance 2 diagonals).

## 7. Docs delta (builders report, the planner applies)

`games/emberwake/docs/GAME_BRIEF.md`: a line for the one-flow input (kit `round_intent_plan` picks, host `pick`,
`unpick`, `autofill`, `pick_options`; game `battle.rs`), the AP gauge (`aegis_ui::pips`, `strip.ap`), the timeline
view (`timeline_view.rs`, key T), the playback aids (speed, tracers, the grouped log), the follow-up lists
(`deck.follow_ups`), the Maneuver rework (`maneuver_range`, `position`, `melee_diagonal`, `flank_protected_cost`;
`round_intent::positions`), VISCLAR1-A's groups and the debug toggle; the RULESET1-U slice 1 line marks its controls
replaced. `docs/specs/RULESET1_INTENT.md` 5.3: a cross-reference line under MV11 ("INPUT1-M, `docs/specs/INPUT1.md`
11 and 13: MV4-MV7 changed, MV9 b retired") and under UI1-UI3 (INPUT1 13). `engine/docs/authoring/combat_hooks.md`
(or the rule-library page the planner picks, memory "Promote generic methods"): `melee_diagonal` and
`flank_protected_cost` as rule-library options. `games/emberwake/docs/SCREENS.md` row 23 (Battle), the UI Gallery
row (`timeline_pane`, `pip_gauge`) and the Scene Runner's Arena card ("Diagonal melee").
`games/emberwake/docs/preferences/ui.md` "Battle overlays and the clarity pass": slice A's outcome. README's controls
list (P, N, T, S, right click, Alt).
