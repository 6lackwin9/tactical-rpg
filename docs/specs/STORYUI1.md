# STORYUI1: story UI (vignette player, story sequences, mission board, new-game flow)

Status: proposed (spec 2026-10-25, from the user decisions "Missions and prologue", "Slice purpose", "Slice
onboarding", "Slice group (1.5)", "Core story track" and "Hub group (1.4)" in
`games/emberwake/docs/decisions/lore.md`; Q-30, Q-46, Q-85 to Q-89, Q-96)
Depends on: none for the engine and schema parts; interfaces with CAMPAIGN1, ROSTER1, KIT1, CAST1 and ENCOUNTER2
(section 7), written in parallel and only referenced here
Lane: logic first (E, S, W, U1, U2), then asset (A); assets, tests, parity, tasks, docs delta, conflicts and open
questions: [STORYUI1_TASKS.md](STORYUI1_TASKS.md)
Touches (by task, see the task file): `engine/crates/aegis_ui/src/{vignette,lib,screen}.rs`;
`games/emberwake/schema.json`; under `games/emberwake/crates/`: `emberwake_content/src/{source,store}.rs` and
`check/{mod,world,quest,dialogue,story,board}.rs`, `emberwake_world/src/{story,board,dialogue,explore,travel,state,
save,new_game}.rs`, `emberwake_gui/src/{story,board,screens,battle,region,worldmap,menus,gallery}.rs`; data under
`games/emberwake/data/worlds/sunk_ward/` and `data/dialogue/services.json` (asset lane)

History: `docs/HISTORY.md` has no row on vignettes, cutscenes or a mission board. DESIGN_REVIEW1 4.6's "contracts
board" (generated, repeatable contracts) was skipped on 2026-10-18 as post-story content (`decisions/world-campaign.md`
"DR1 design review answers"); not re-proposed: this board lists authored story quests ("Missions and prologue").

## 1. Goal

Three pieces the vertical slice needs before it can be data only (Q-55, Q-56), each off unless a world uses it, so
the Ashlands run unchanged: an engine vignette player (still cards, typewriter text, fades, skip and speed, sound
cues; Q-88), a kit story layer (vignettes as data, a queue, and linear story sequences that chain cards, fights and
effects, so a new game can open on the intro and the prologue), and a kit mission board (a hub NPC whose board offers
quests by state, with map markers and checkpoint deadlines). The asset task fills the slice: the intro, the prologue
0a-0c, the hearth scene, the Doom Clock card, the Act 1 title card, the slice end card (Q-87) and the Act 1 bad-end
card, Tess's board with the slice's main missions.

## 2. Survey (develop at 487b59d)

| Area | Where | What it does today |
| --- | --- | --- |
| Dialogue conditions | `emberwake_world/src/dialogue.rs:171-186` | `flag`, `quest`, `items`, `gold`, `stat`; `requires` all, `unless` any; lock reasons `:191-243` |
| Dialogue effects | `dialogue.rs:607-722` | `set_flag`, `add_flag`, `start_quest` (giver must be the speaker, `:626-637`), `turn_in`, `give`, `take`, `open_shop`, `fight`, `recruit` (refused at `party_max`, `:680`) |
| Service choices | `dialogue.rs:402-439` | Appended to the start node: one offer per available quest whose giver is the speaker, turn-ins, the keeper's shop, farewell; lines from `data/dialogue/services.json` |
| Fights from talk | `dialogue.rs:573-579`, `:804-826` | `fight` closes the conversation; `after_fight` settles and wakes the party on a loss; no way to chain a next beat |
| Quests | `emberwake_world/src/quest.rs:84-143`, `:211-239`, `:380-426` | Templates (`reach_location`, `win_encounter`, `flag`), steps, variants (`available: false` closes a quest), `requires` (completed quests) and `requires_flags` lock, `auto_start`; statuses invalid, completed, blocked, ready, active, unavailable, available, locked |
| Quest progress | `emberwake_world/src/battle.rs:272`, `:363` | `settle` advances `win_encounter` on a win |
| Map markers | `emberwake_world/src/travel.rs:225-276`, `emberwake_gui/src/worldmap.rs:164-181` | A "!" and a tooltip line on nodes that hold an active quest's current target; nothing for quests not yet taken |
| NPC badges | `emberwake_world/src/explore.rs:305-335`, `emberwake_gui/src/region.rs:611`, `:1084-1105` | `turn_in`, `offer`, `shop`, `recruit`; the first service is the marker's badge |
| Conversation UI | `region.rs:247-308`, `:687-691`, `:1234` | The talk box (400 px dock, portrait, choices) and the shop pane live inside `RegionMap`; no dialogue surface exists off a region map |
| Battle bridge | `emberwake_gui/src/battle.rs:548`, `:723-767` | `BattleScreen::world(fight, runner)`; `settle` writes HP, calls the runner's `after_fight` (`travel`, `dialogue`, else `explore`), autosaves, pops, opens `Results` |
| New game | `emberwake_gui/src/menus.rs:59`, `:191-227`; `screens.rs:166`; `emberwake_world/src/new_game.rs:24-100` | Menu, world cards, Start, `create_for_world`, `enter_game` (always `Nav::Reset(RegionMap)`), an autosave |
| Game state | `emberwake_world/src/state.rs:456-476`, `:515-521` | `flags` (keys `[a-z0-9_.]`, values bool, int, string), `quests`, `time` (the travel clock) |
| Screens | `engine/crates/aegis_ui/src/screen.rs:85-90`, `:222-330` | `Nav` Push, Pop, Reset; `tick` for the top screen only (`shell.rs:301`), `resume` on a pop (`shell.rs:151`), `music()` (top wish wins, `shell.rs:359`) |
| Drawing, sound | `aegis_ui/src/painter.rs:116`, `:172`, `:264`; `art.rs:136`; `audio.rs:142`, `:185` | Alpha fills blend (a fade is a cover fill), `image`, `wrap`, `Art::first`; no typewriter, fade or card widget; one-shot `play`, `music(Some(""))` = silence |
| Settings | `screen.rs:109-112`; `emberwake_gui/src/screens.rs:24` | Engine keys (sound, display, `text_scale`); game keys `anim_speed`, `skip_anim`; no text speed or reduce-motion key |
| UI Gallery | `emberwake_gui/src/gallery.rs:26-55`, `:145-170` | 26 entries; screens that need a game get the sample game of the first playable world |
| World kinds | `emberwake_content/src/source.rs:29` | 13 kinds; `emberwake_world/tests/data_source.rs:25-34` strips post-freeze fields (`AFTER_GODOT`) |

## 3. Layers

| Piece | Layer | Where | Why |
| --- | --- | --- | --- |
| Card player: phases, typewriter, fades, skip, speed, cues, card drawing, `VignetteScreen` | `[engine]` | `aegis_ui/src/vignette.rs` (new) | Any game shows title cards and captions; like the timeline widget (CANON 2.1) the drawing is engine and the rules that feed it are kit |
| Vignettes as data, the play queue, seen state, tokens, conditions | `[kit hook]` | `emberwake_world/src/story.rs` (new), checks in `emberwake_content/src/check/story.rs` (new) | Any story in the genre has scenes; story-free code, moved by KIT1 |
| Story sequences (cards, fights, effects, enter, end) | `[kit hook]` | `story.rs`, `emberwake_gui/src/story.rs` (new) | A prologue, an act opening, an epilogue: a shape any campaign fills |
| Mission board | `[kit hook]` | `emberwake_world/src/board.rs`, `emberwake_gui/src/board.rs` (new) | Named in the layer rule; quests and the shop pane are its parts |
| Text speed and reduce-motion settings | `[engine]` | `screen.rs` engine defaults | Accessibility for every game (reduce motion is shared with FX1) |
| The intro, the prologue, the hearth, the cards, Tess's board, the slice quests | `[data]` | `data/worlds/sunk_ward/` | Named by the story |
| Art-bible colours of the card text | `[game code]` | `emberwake_gui/src/story.rs` constants | One line: the engine takes a palette, the game passes its art bible's hues |

The kit code stays in the game crates until KIT1 moves it (STRUCTURE.md: kit code comes when a second game shares
rules); it names no Emberwake place, person or flag.

## 4. Schema contract (schema-first: yes)

### 4.1 Kind `vignettes` (world kind, `data/worlds/<id>/vignettes.json`)

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `id`, `name` | id, string | "", "New Vignette" | standard |
| `music` | string | "" | "" keeps the music playing; `none` = silence; else a path under the asset layers' `audio/` ending `.ogg` |
| `frame` | string | `#0a0b10` | `#rrggbb`; letterbox and fade colour |
| `skippable` | bool | true | false: the Skip control is hidden and refused |
| `once` | bool | false | queued by an effect: not queued again once seen (5.2) |
| `cards` | json | [] | 1..24 cards (4.2) |
| `override`, `tags` | string, strings | "", [] | standard |

### 4.2 Cards (objects in `cards`; unknown keys refused)

| Key | Type | Default | Rule |
| --- | --- | --- | --- |
| `style` | enum | `image` | `image` (a picture area and a caption), `text` (text centred on the backdrop), `title` (title and subtitle), `line` (a speaker's portrait, name and line) |
| `image` | string | "" | `image` only; a path under the asset layers (`backgrounds/location/cinder_gate.png`); "" or a missing file draws the backdrop in the picture area |
| `top`, `bottom` | string | `#0a0b10`, = `top` | `#rrggbb`; the code-drawn backdrop, a vertical gradient |
| `ash` | bool | false | drifting ash specks over the backdrop |
| `title` | string | "" | at most 48 characters; required for `title` |
| `subtitle` | string | "" | at most 80 characters |
| `text` | string | "" | at most 360 characters, typed out; tokens 5.2; required for `text` and `line` |
| `speaker` | ref characters | "" | required for `line` |
| `fade_in_ms`, `fade_out_ms` | int | 600, 400 | 0..3000 |
| `hold_ms` | int | 0 | 0..20000; 0 waits for the player; above 0 the card moves on by itself that long after its text is out |
| `cue` | string | "" | a sound under `audio/` (`sfx/defend.wav`), played as the card starts |
| `when` | json | [] | dialogue conditions (`dialogue.rs:171`); a card whose conditions fail is left out |

### 4.3 Kind `sequences` (world kind)

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `id`, `name` | id, string | "", "New Sequence" | standard |
| `steps` | json | [] | 1..40 steps, each `{"kind", ...}` below |
| `override`, `tags` | string, strings | "", [] | standard |

| Step `kind` | Fields | Rule |
| --- | --- | --- |
| `vignette` | `vignette` (ref vignettes) | plays it (`once` does not apply inside a sequence) |
| `fight` | `encounter` (ref encounters), `on_lost`, `on_fled` (`retry`, `abort`, `continue`; default `abort`) | a world fight with the party as it is (5.3) |
| `effects` | `effects` (1..8 dialogue effect objects) | allowed: `set_flag`, `add_flag`, `start_quest`, `give`, `take`, `recruit`, `vignette` and the effects later specs register (CAMPAIGN1 meters, ROSTER1); never `fight`, `turn_in`, `open_shop`, `open_board`, `sequence` |
| `enter` | `location` (ref locations), `anchor` (string, "" = the location's entry) | last step only: the party is placed there and the map shows |
| `end` | `to` (`map` or `menu`, default `map`) | last step only; a sequence without `enter` or `end` ends as `end` `map` |

### 4.4 Kind `boards` (world kind)

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `id`, `name` | id, string | "", "New Board" | standard |
| `keeper` | ref characters | "" | required; a gone keeper (`dialogue.rs:72`) closes the board |
| `intro` | text | "" | at most 160 characters, the pane's first line |
| `requires_flags` | json | [] | `[{"key", "op"?, "value"}]`; the board is closed while any fails (the shops' rule) |
| `override`, `tags` | string, strings | "", [] | standard |

### 4.5 New quest fields (world `quests`, appended after `tags`)

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `board` | ref boards | "" | listed on that board and offered only there; needs a template; the giver must be "" or the board's keeper |
| `track` | enum | `side` | `main` or `side`; the board's sections (5.5) |
| `show_locked` | bool | false | listed on the board while locked, with the lock reason |
| `closed_text` | string | "" | at most 80 characters; when the quest is unavailable or blocked, it is listed under Closed with this line; "" = not listed |

### 4.6 World manifest field

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `start_sequence` | ref sequences | "" | runs when a new game starts in this world; "" = the map as today |

### 4.7 Dialogue effects and the service line

| Effect | Fields | Does |
| --- | --- | --- |
| `vignette` | `vignette` | queues it (5.2); the conversation goes on |
| `sequence` | `sequence` | starts it (5.3) and closes the conversation; refused while one runs |
| `open_board` | `board` | opens the board pane over the conversation, like `open_shop` |

Service line `board` (base line in `data/dialogue/services.json`, "Show me the board."; worlds may override it in
`service_lines`). The content checks know the three effects in nodes, choices and sequence `effects` steps.

### 4.8 Game state and settings

- `GameState.story`: `{"running": {"sequence": "<qid>", "step": 0}, "queue": ["<qid>"], "seen": ["<qid>"]}`, each
  key written only when not empty and the whole field only when one is; a save without it reads empty (no format
  step). Board use sets the flag `board.<board id>.opened` (true) on the first open.
- Engine settings: `text_speed` (`slow`, `normal`, `fast`, `instant`; default `normal`; 30, 60, 120 characters per
  second, or all at once) and `reduce_motion` (bool, false; FX1 needs the same key: whichever task lands first adds
  it). The settings screen's gameplay tab shows both.

## 5. Rules

### 5.1 The vignette player `[engine]`

- V1. A card runs fade in, typing, hold, fade out; then the next card; after the last the reel is done.
- V2. Typing reveals characters at the text speed; the text is wrapped once, in full (`Painter::wrap`), so a word
  never jumps lines while it types; `instant` shows it all as typing starts.
- V3. A press (click on the stage, Space, Enter) does exactly one thing: in fade in it ends the fade; in typing it
  shows the whole text (hold); in hold it starts the fade out; in fade out it ends the fade (the next card).
- V4. With `hold_ms` above 0 the fade out starts by itself `hold_ms` after the text is out; 0 waits for a press.
- V5. Skip (Esc, right click, the Skip button top right): a skippable reel is done at once, without further cues; a
  reel that is not skippable ignores it. The screen takes `ask_first`: then Skip asks "Skip this scene?" (`Cx::ask`,
  Yes the default) before it skips.
- V6. Fades are a cover fill of the frame colour over the stage: alpha 255 x (1 - t / fade) in fade in, 255 x t /
  fade in fade out; `fast` halves the fade times, `instant` makes them 0.
- V7. A card's cue plays once as the card enters fade in (`Bus::Sfx`); a skipped card plays none.
- V8. Music: `""` = no wish (the screen below decides), `none` = `Some("")` (silence), else the path.
- V9. Layout on the 640 x 360 stage: the `image` picture area is (40, 20, 560, 236), the image letterboxed to its
  aspect in the frame colour; the caption box starts at y 266, 560 wide, and grows with the text size; `title`
  centres the title at y 140 at three times the body size in the accent colour, the subtitle under it, a 200 px rule
  in the accent above and below; `text` centres its text in (80, 120, 480, 120); `line` puts a 64 px portrait left
  of the caption box and the speaker's name above the line in the accent colour. "Click to continue" shows dim at the
  bottom right in a hold that waits.
- V10. Colours come from a `Palette` (text, accent, dim) the caller passes; the engine's defaults are neutral.
- V11. Ash: 24 specks seeded by the card index, drifting down 6 px a second; never drawn under `reduce_motion`.
- V12. Determinism: the model moves only through `advance(ms)`, `press()` and `skip()`; the screen measures the
  frame time (at most 100 ms a frame) and feeds it; tests drive the model without a clock.
- V13. Agent link: `state` gives `{reel, card, cards, phase, shown, total, done}`; widgets `vignette.next` (the
  stage) and `vignette.skip`.
- V14. At every text size (TEXTFIT1) a 360-character caption fits without a cut or an overlap.
- V15. `VignetteScreen<G>` is a full screen (not an overlay) with an `on_done` callback, run when the reel ends or is
  skipped; it changes no game state itself. The UI Gallery shows it as a standalone pane (the standalone-panes rule)
  with the engine's sample reel and every world's vignettes.

### 5.2 Vignettes in the world `[kit hook]`

- W1. `story::queue(d, gs, vignette)`: a `once` vignette already seen is not queued; a queued one is not queued twice.
- W2. The game plays the queue's oldest entry when a map screen is on top with no conversation, shop, board or popup
  open; when it ends or is skipped it leaves the queue and joins `seen`.
- W3. Cards resolve when they play: `when` against the state then; tokens `{flag:KEY}` (the flag's value as text, ""
  when unset) and `{member:KEY}` (the name of the party member, away member (ENCOUNTER2-W) or world character whose
  id the flag holds); the content check refuses other tokens.
- W4. A vignette whose every card is left out by `when` is marked seen and plays nothing.
- W5. A `line` card shows the speaker's `name` and the portrait chain the talk box uses (`region.rs:1239-1243`).
- W6. Skip asks first (V5) only for a vignette not yet seen (SQ5).

### 5.3 Story sequences `[kit hook]`

- Q1. `story::start(d, gs, sequence)` is refused while a sequence runs or for an unknown ref; it sets `running` to
  step 0, then Q3.
- Q2. `story::current` gives the step at `running.step` (`vignette`, `fight`, `enter` or `end`), never `effects`.
- Q3. `effects` steps apply at once, in order, with the dialogue effect rules (the per-effect body of
  `DialogueRunner::apply`, `dialogue.rs:698-722`, factored into one shared function with no behaviour change;
  `start_quest` skips the giver check); their notes go to the log; then the next step.
- Q4. `story::step_done` (a vignette ended or was skipped): the next step, then Q3.
- Q5. `fight`: the game starts a world fight with runner `story` (`{"source": "story", "encounter": <qid>}`); the
  battle's settle calls `story::after_fight`, which runs `battle::settle` (rewards, quest progress) and then, by
  outcome: won, the next step; lost or fled by `on_lost` / `on_fled`: `retry` brings every member back to full HP,
  keeps the step and logs "Try again."; `continue` goes on; `abort` clears `running` and runs the normal loss path
  (the wake of `dialogue::after_fight`, `dialogue.rs:814-817`, and later CAMPAIGN1's Act 1 retreat).
- Q6. `enter`: `explore::enter(location, anchor)` (`explore.rs:429`), `running` cleared, the map shows
  (`Nav::Reset(RegionMap)`).
- Q7. `end` `map`: `running` cleared, `Nav::Reset(RegionMap)`; `end` `menu`: `running` cleared, autosave, the game
  in play ends, `Nav::Reset(MainMenu)`.
- Q8. Saves: `story::after_fight` moves the step before the battle's autosave, so a load resumes at the next beat; a
  save whose `running` names an unknown sequence or a step out of range drops it with a load warning; the game menu
  (save, load) cannot open while a sequence runs (no map is under it).
- Q9. A `vignette` effect inside a sequence queues as usual and plays before the next step.

### 5.4 The new-game flow (data and code)

| Beat | Data (`sunk_ward`, STORYUI1-A) | Code |
| --- | --- | --- |
| Menu, New game, Start | the world card (name, tagline, start party Kaelen) | unchanged (`menus.rs:59`, `:191-227`) |
| A fresh state | world `start_sequence` = `prologue` | `new_game::create` sets `story.running`; the autosave holds it |
| First screen | - | `enter_game`: `running` set, `Nav::Reset(StoryStage)`, else the region map as today |
| Intro vignette (four slides) | vignette `intro` | `StoryStage` plays the steps (5.3) |
| 0a The Alley (Kaelen alone) | vignette `cold_open` ("Rise", Vane's "Order 44"), fight `enc_pro_0a` (`retry`) | runner `story` |
| Meat Market walk, 0b The Rooftop | vignettes `meat_market`, `rooftop`; fight `enc_pro_0b`; effects `recruit` Vesper | - |
| 0c The Bridge | vignette `bridge`; fight `enc_pro_0c`; effects `recruit` Maren, Hollis | - |
| The dive | vignette `dive` | - |
| The hearth scene | vignette `hearth` (the scene card and the lines) | - |
| The Doom Clock appears | effects: CAMPAIGN1's meter effect, `set_flag` `act` 1; vignette `doom_clock` | CAMPAIGN1 draws the meter on the HUD (Q-46) |
| Act 1 title card | vignette `act1_title` | - |
| The hub | `enter` `sluice_undercroft` | Q6 |
| The board offers the first main mission | board `tess_board`, quest `m3_bone_mill` (the slice's first main mission, Q-84 a) | 5.5 |

### 5.5 The mission board `[kit hook]`

- B1. A board is open when its keeper is not gone and its `requires_flags` hold.
- B2. Its quests are the world lineage's quests whose `board` names it, in merged file order.
- B3. Conversation: `service_choices` (`dialogue.rs:404-413`) leave out quests with a `board`; the keeper's start
  node gets one service choice `board` (after the turn-ins, before the shop) while the board is open; it opens the
  pane (`open_board`): the runner holds `board_qid` like `shop_qid`, and its busy rule says "Close the board first.".
- B4. Badge: `npc_services` (`explore.rs:305`) lists `board` for the keeper of an open board with at least one
  available quest; the icon is the quest badge (`ui/badges/quest.png`), the hover text "New work on the board";
  `offer` counts only quests without a board.
- B5. Sections, in this order, empty ones left out: Main (track `main`, available), Jobs (track `side`, available),
  Under way (active or ready; the line is the current objective's text, or "Report to {keeper}" when ready), Locked
  (locked with `show_locked`; the line is the lock reason in `lock_reason` words), Closed (unavailable or blocked
  with a `closed_text`; the line is that text). The next main mission appears when the previous one is completed,
  through its `requires` (CANON 3.2).
- B6. A row shows the quest's name, its node (the first step's location, resolved as `travel.rs:225-256` does) and,
  when CAMPAIGN1 reports a checkpoint for it, "By day {day} ({left} left)" in Arterial Red `#e11d48`.
- B7. The detail shows the description (variants applied), the steps (done, current, to come), the rewards (the
  template's, through `hub.rs` `reward_lines` with the purse's display name, the loot pool as "Pick {n} of: ...",
  reputation), the node and the deadline line.
- B8. Accept (available quests only) runs the runner's `start_quest` path (precheck `dialogue.rs:626-637`), then
  `quest::start`; the row moves to Under way and the log says "Quest taken: {name}.". The board has no Abandon.
- B9. The first open sets `board.<id>.opened`.
- B10. Map markers (Q-30): `travel::view` gives a node `jobs` (names of available quests on opened boards whose first
  step's location resolves to it) and `deadlines` (checkpoint lines of quests targeting it); the world map draws the
  quest badge (9 px) at the pin's top left when `jobs` is not empty and adds the tooltip lines "Job: {name}" and
  "Checkpoint: by day {day}"; active targets keep today's "!" (`worldmap.rs:164-166`). Both keys are absent when
  empty.
- B11. The pane: the shop frame's size (`shop.rs:141`, 600 x 300 at Normal, grown with the text), the keeper's
  `intro` on top, the section list on the left (240 wide, scrolls with the wheel), the detail on the right, Accept and
  Close; Esc and right click close it back to the conversation (UI preferences: right click goes back).
- B12. Reporting a finished mission stays the existing turn-in at the keeper (`dialogue.rs:414-423`) (SQ3).

## 6. Interfaces with the parallel specs

| Spec | STORYUI1 needs | STORYUI1 gives |
| --- | --- | --- |
| CAMPAIGN1 | a meter effect that shows the Doom Clock (an `effects` step); `checkpoint(d, gs, quest) -> Option<(day, left)>` for B6 and B10; triggers whose effects may be `vignette` and `sequence` (the slice end on the Crucible's win, the Act 1 bad end); the purse's display name (Scrap) for B7 | the player, the two effects, the cards `slice_end` and `act1_bad_end` |
| ROSTER1 | the roster cap, so the M3 recruit of Dunstan is not refused at `party_max` (`dialogue.rs:680`) | - |
| ENCOUNTER2-W | the `crucible_stayed` flag and `party.away` for `{member:...}` | - |
| CAST1, SUNKWARD1 | character ids (founders, Vane, Tess, Bram, Aline), encounters `enc_pro_0a..0c`, `enc_evt_03`, `enc_evt_06`, locations | the sequences that run them |
| KIT1 | moves `story.rs`, `board.rs` and their checks to kit crates | story-free code |
| FX1 | the `reduce_motion` key | - |

If CAMPAIGN1's triggers lack an "encounter won" condition, the slice end starts from a "quest ready" trigger on
`m5_crucible` (its last objective is the Crucible's win).

## 7. Out of scope

Battle barks (Q-89; bosses speak through phase lines), camera shake on cards (FX1), pans, zooms and animated
sprites in cards, voice, choices inside vignettes (choices stay dialogue), generated or repeatable contracts, the
Bunks and Facility screens (Q-96), the Extinguished screen and checkpoint logic (CAMPAIGN1), ending cards (Act 3),
Act 2 content, Tess's reconnaissance (Q-91), abandoning from the board, journal changes, region-map markers for jobs
inside a location, the main menu's art.
