# INPUT1 tasks: schema, kit and host API, tests, parity and the task split

Status: proposed (spec 2026-10-31, task INPUT1-S); companion of [INPUT1.md](INPUT1.md), whose rule ids it cites.
Paths: `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`.

## 1. Schema contract (schema-first: yes for INPUT1-L's field; the rest is code and settings)

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
| `pick_options(w, plan, c, at: RoundTarget) -> Vec<PickOption>` | read-only; the rows of IF2 / ET1 in order; `PickOption { picks: Vec<Pick>, intent: Option<RoundIntent>, kind: Option<ManeuverKind>, ap: i32, blocks: i32, problem: Option<String>, drops: Vec<String> }` (`picks` has two entries for "Move here, then X") |
| `pick(w, plan, c, at, picks: &[Pick]) -> (Result<(), String>, Vec<(UnitId, String)>)` | atomic (IF6, GH3-GH5); sets the target when no pick holds it, the intent and kind from the first intent pick; rebuilds the member, then later members (`replan`'s shape: the dropped picks of later members with reasons) |
| `unpick(w, plan, c)`, `clear_picks(w, plan, c)` | IF7; rebuild as `pick` |
| `set_autofill(w, plan, on)` | rebuilds every party member's ghost |
| `ghost_from(w, plan, c) -> usize` | the index (in `Plan::entries_for`) of the first ghost entry |
| `preview_pick(w, plan, c, at, picks) -> Result<(Vec<Step>, ApForecast), String>` | IF5, on clones; changes nothing |
| `ap_forecast(w, plan, c) -> ApForecast` | AP1: `{ now, spent, picked, filled, left, gain, rest_gain, next, max }`, computed with `status::roll_over`'s and `round_intent::round_start`'s own rules |
| `follow_order(w, c, intent, opener: Option<&Opener>) -> Vec<SkillId>` | FL2 (INPUT1-L); FU2's fill calls it where it calls `order` today |

`plan_unit`, `replan`, `fill_rest`, `preview`, `rest_preview`, `set_manual` stay (the Dojo, the sims and old host
requests use them). With no pick the member's plan is exactly `plan_unit`'s (GH2), which keeps every auto plan as
today.

### 1.3 Host requests (`rpgkit_host`, INPUT1-U)

| Request | Recorded | Reply |
| --- | --- | --- |
| `pick {unit, target, pick, then?}`; `pick`: `{"skill": id, "kind"?: "engage"\|"disengage"\|"utility"}`, `{"item": id}`, `"rest"`, `{"move": [x, y]}`, `{"step_after": [x, y]}`; `then`: a skill pick after a move | yes (joins the recorded list beside `round_intent`) | `intent_reply`'s shape plus `dropped` [{unit, reason}] |
| `unpick {unit}` | yes | `intent_reply` |
| `autofill {on}` | yes | `{ok, autofill}` plus `round_plan` |
| `pick_options {unit, target}` | no (read-only) | `{ok, rows: [{picks, intent, kind, ap, blocks, problem, drops}]}` |
| `clear {unit}` (exists) | yes | under `round_intent`: clears the picks (IF7) |
| `round_plan` (exists) | no | per member adds `picks`, `ghost_from`, `autofill`, `ap` {now, spent, picked, filled, left, gain, next, max}; `manual` stays (true when the member has a pick); never an enemy, as today |

`round_intent`, `manual` and `fill` stay for recordings, sims and tools; the battle screen stops sending them. The
sim drivers (`feel`, `timed`, autoplay, `Session::auto_draft`) send none of the new requests.

### 1.4 Engine (`aegis_ui::timeline`, INPUT1-U)

`Segment` gains `ghost: bool` (default false): drawn with its fill at 45% alpha, a dashed outline and the label in
`DIM`; `pub fn draw_ghost(ui, bar: Rect)` gives the battle strip the same look for its own spans. Game-agnostic
(the module doc names "a planned, not chosen, span"); every existing caller is unchanged (default false).

### 1.5 Settings (game, `screens::default_settings`, Settings screen in `menus.rs`)

| Key | Tab | Default | Notes |
| --- | --- | --- | --- |
| `auto_fill` | Gameplay | true | GH7, checkbox `settings.auto_fill` |
| `battle_overlays.first_move` | Battle | false | VC1 |
| `battle_overlays.all_bars` | Battle | false | VC3 |
| `debug_party_foresight_off` | Battle, "Debug" heading | absent (= false) | VC4, VC5: debug builds only |

`battle::OVERLAY_GROUPS` gains a default per group (VC6). The Controls list: "Skills of the selected hero
(timeline): 1-9", "Target self: S", "Inspect a unit: right click", "Full timeline: T", "Every bar and arrow: hold
Alt".

### 1.6 Agent link (game)

Widget ids: menu rows as the popup's (`popup.row.<k>`), `hero.self`, `strip.ap`, `timeline_view` (the engine widget's
ids under it), `skills.follow.*` (FL3), `dojo.deck.follow.*` (the Dojo's mirror), `settings.auto_fill`,
`settings.overlay.first_move`, `settings.overlay.all_bars`, `settings.debug.party_foresight_off`,
`gallery.timeline_pane`. Battle state keys: `picks` [{unit, picks}], `ap_readout` {text, red}, `inspect` (unit or
null), `focus` [units], `timeline_view` {open, tracks [{key, masked, segments}]}, `overlay.arrows` [{unit, points,
alpha}]. Removed: `hero.intent.<id>`, `hero.kind`, `hero.opener`, `hero.manual`, `hero.fill`; the state key `intents`
gives way to the popup rows.

## 2. Layers (CLAUDE.md; the kit names no game)

| Layer | What |
| --- | --- |
| engine | `aegis_ui::timeline` `Segment.ghost`, `draw_ghost` (1.4) |
| kit | picks, options, ghost, Auto-fill flag, AP forecast, hotkey order (1.2); host requests (1.3); `deck.follow_ups`, its checks, save and the planner's reader (1.1, FL2) |
| game | the battle screen's flow, menus, hotkeys, selection, inspect, readout, timeline view (`timeline_view.rs`), the settings and the Controls list, VISCLAR1-A, the deck screen's lists, the Dojo's Deck window, the Gallery entries |

The setting's name "Auto-fill the rest", every label and the Gallery pane's sample data are the game's; the kit has
the bool `autofill` and returns refusal reasons in today's style.

## 3. Tests (rule-derived)

| Task | Test file | Checks |
| --- | --- | --- |
| INPUT1-U (kit) | `rpgkit_battle/tests/picks.rs` (new) | IF2 rows by target kind; IF6 a-e refusals with their texts; IF4 intent and kind from the first pick, neutral picks set none; GH2 no-pick plan equals `plan_unit`'s step for step; GH3 a pick replaces the ghost from its block and the fill rebuilds after it; Auto-fill off leaves Rest; GH4 the Step-after block reserved; GH5 atomic refusal, a later member's dropped pick; GH6 picks cleared at round start, the intent kept; HK1 order; ET2-ET4 refusals; AP1 the four worked examples; `ap_forecast.next` equals the next round's AP on a dummy fight; `preview_pick` leaves `World` and `Plan` equal |
| INPUT1-U (host) | `rpgkit_host` tests | `pick`, `unpick`, `autofill`, `pick_options`, `round_plan`'s new keys; a refused `pick` leaves the battle unchanged; a recording with `pick` replays to the same state; an old recording with `round_intent` / `manual` / `fill` still replays |
| INPUT1-U (engine) | `aegis_ui` timeline tests | a ghost segment draws with the ghost style; a default segment draws as before (pixel compare) |
| INPUT1-U (game) | `battle.rs` `input1_tests` | IF1 click sets target and opens the menu; IF8 none of the removed ids exists; HK2-HK3 digits; SL1-SL7; IS1-IS3 (an unrevealed enemy's inspect text holds no plan skill); AP3 red and back; AP4 short form at 1.3; IF9 tactic skips members with picks; GH7 the setting rebuilds ghosts; `text_audit` at 0.85, 1.0, 1.15, 1.3 for the menu, the hero panel and `strip.ap` |
| INPUT1-T | `battle.rs` `timeline_view_tests`, `gallery.rs` tests | TV1 T and Esc; TV2 sections and order; TV3 picked and ghost segments; TV4 masked note text, badges as marks, playback holds only started actions, a revealed track unmasked; TV5 track click selects; TV6 `text_audit` at four sizes, 16 tracks before scroll; TV7 the Kit pane renders with no session; battle view and Gallery build equal `Timeline` values |
| INPUT1-L | `rpgkit_battle/tests/picks.rs`, `rpgkit_content` check tests, `rpgkit_world` save tests, `party.rs` tests | FL1-FL5 as INPUT1.md 9's acceptance; FL3 widget flow (open, add, reorder, remove, clear); the Dojo's Deck window edits rebuild the hero |
| VISCLAR1-A | `battle.rs` `visclar_tests` | VC1-VC6 and AR1-AR5 as INPUT1.md 10's acceptance; `party_foresight_off(_, false)` is false; Restore defaults gives `first_move` and `all_bars` off |

## 4. Slices and task rows (the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- | --- |
| INPUT1-S | Spec: the one-flow battle input and VISCLAR1 slice A | game + kit | design | RULESET1-U, RULESET1-PL | done (this spec) | `docs/specs/INPUT1.md` (+ `_TASKS.md`) | `docs/specs/INPUT1*.md` |
| INPUT1-U | Battle input as one flow (sections 2-7) | engine + kit + game | logic | INPUT1-S, CAMPAIGN1-U, ROSTER1-L | spec | `docs/specs/INPUT1.md` | `aegis_ui/src/timeline.rs`; `rpgkit_battle/src/{round_intent_plan,round_intent}.rs`, `rpgkit_battle/tests/picks.rs`; `rpgkit_host/src/lib.rs`; `emberwake_gui/src/{battle,menus,screens,intent_text}.rs` |
| VISCLAR1-A | VISCLAR1 slice A: first-move chip behind a setting, bars in focus, one quiet arrow per unit, debug Party foresight off (section 10) | game | logic | INPUT1-U | spec | `docs/specs/INPUT1.md` 10 | `emberwake_gui/src/{battle,menus,screens}.rs` |
| INPUT1-T | Full timeline view (key T) and the Gallery's Timeline pane (section 8) | game | logic | INPUT1-U, VISCLAR1-A (both edit `battle.rs`) | spec | `docs/specs/INPUT1.md` 8 | `emberwake_gui/src/{battle,gallery}.rs`, new `emberwake_gui/src/timeline_view.rs` |
| INPUT1-L | Per-opener follow-up lists (section 9) | kit + game | logic | INPUT1-U | spec | `docs/specs/INPUT1.md` 9 | `rpgkit_data/src/lib.rs`, `rpgkit_content/src/check/unit.rs`, `rpgkit_world/src/state.rs`, `rpgkit_battle/src/round_intent_plan.rs`, `kits/rpg/schema.json`; `emberwake_gui/src/{party,dojo}.rs` |

INPUT1-U may land in three slices: (1) engine and kit with host (1.2-1.4, kit and host tests); (2) the battle
screen's flow, hotkeys, empty tiles, ghost look, readout and the setting; (3) selection and inspect. INPUT1-T and
INPUT1-L touch different files and may run side by side after VISCLAR1-A. No asset task: no shipped data needs a
follow-up list.

Row changes asked of the planner: INPUT1-U's Layer becomes "engine + kit + game" and its Touches add the engine
timeline file and the host; INPUT1-T's Layer becomes "game" (the pane is the engine widget INPUT1-U extends, shown
by the game's Gallery; no new kit code) and it depends on VISCLAR1-A for the shared file; a new logic row VISCLAR1-A
(the VISCLAR1 design row keeps the later slices); INPUT1-S moves to done.

## 5. Parity

Input only: no rule, number or AI choice changes. The Godot parity suites (classic battle traces, world battles,
journeys, saves, loader and writer) are untouched: classic input is unchanged, `deck` is already stripped from the
Godot data comparison (HISTORY 2026-10-25 "Godot parity over the timeline content fields"), Godot saves carry no
deck. The timeline golden traces (RULESET1-X) must not change: they come from the kit's auto driver, which sends no
pick; a member with no pick plans exactly `plan_unit`'s plan (GH2); `autofill` is true unless a request sets it;
`follow_order` with no list returns `order` (FL2); picks and the ghost index are in no trace. Every golden and
parity test passes byte-identical, and no trace or expectation file is edited; a moved golden is a bug to report,
not to re-record. No parity check is retired. The feel sim's numbers stay (no balance-lane run is needed).

## 6. Wanted HISTORY rows (the planner writes them)

| Date | Idea | Status | Replaced by | Reason |
| --- | --- | --- | --- | --- |
| 2026-10-31 | The round controls of RULESET1-U slice 1 (intent buttons, keys 1-9 as intents, the Maneuver kind and opener menus, the Manual toggle, Fill; RULESET1_INTENT UI1, MM1-MM3 as a mode) | replaced | INPUT1: target, then a skill; intent and kind from the skill; ghost fill; the setting Auto-fill the rest; keys 1-9 as skill hotkeys | User decision "Combat input feedback": fewer controls a round, the intent read from the chosen skill |
| 2026-10-31 | A hand-picked carried-move tile (RULESET1_INTENT MM2, MM4) | dropped (if Q3 a) | PL4's tile | One input flow; the planner's ranked tile stands |
| 2026-10-31 | Several lines per unit on the board: tell arrows, the hero's planned-move lines, revealed enemy walks (also the hovered enemy's walk above the tokens, RULESET1-U slices 5-6) | replaced | One quiet dashed target arrow per unit (move leg when the move is visible), the selected unit's by default, all with Alt (INPUT1 AR1-AR5) | User addition 2026-10-31: a calmer Stage |
| 2026-10-31 | The enemy first-move chip on tokens by default (INTENT1-U) | narrowed | Behind the debug overlay group `first_move`, off by default | User decision "Combat input feedback" (VISCLAR1 slice A) |

Decision rows touched (for the planner): "Combat input feedback" gains the arrow addition (AR1-AR5); it supersedes,
for arrows, that row's "arrows ... for the selected member, its target and the unit under the cursor" (bars keep
that focus set) and "RULESET1-U slice 5 answers"' "draws the hovered enemy's revealed moves above the tokens", and
the RULESET1-U row's "draw a revealed enemy's planned walk only for the hovered enemy or while Alt is held".

## 7. Docs delta (builders report, the planner applies)

`games/emberwake/docs/GAME_BRIEF.md`: a line for the one-flow input (kit `round_intent_plan` picks, host `pick`,
`unpick`, `autofill`, `pick_options`; game `battle.rs`), the timeline view (`timeline_view.rs`, key T), the
follow-up lists (`deck.follow_ups`), VISCLAR1-A's groups and the debug toggle; the RULESET1-U slice 1 line marks its
controls replaced. `games/emberwake/docs/SCREENS.md` row 23 (Battle) and the UI Gallery row (`timeline_pane`).
`games/emberwake/docs/preferences/ui.md` "Battle overlays and the clarity pass": slice A's outcome. README's controls
list.
