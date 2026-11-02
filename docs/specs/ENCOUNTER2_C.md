# ENCOUNTER2-C: mid-battle dialogue and scripted rounds

Status: proposed (spec 2026-11-01, from decision "INPUT1 answers, part 2" Q12 in
`games/emberwake/docs/decisions/combat.md` and the TASKS row ENCOUNTER2-C, M2); part of [ENCOUNTER2.md](ENCOUNTER2.md)
Depends on: ENCOUNTER2-U (done) for the talk part; INPUT1-U (the `Pick` API, INPUT1_TASKS 1.2) for scripted rounds;
CAST1-A6 (the `enc_pro_0a` row) for the data part
Lane: logic first (kit schema and rules, then the game screen), then asset (the prologue's opening)
Touches: by slice, section 8
Paths: `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`.

As built, C1 (2026-11-01; sections 3.1, 3.2, 5.2): `encounter.rs` `TALK_EFFECT`, `TalkCue`, `Phases::mark_block`,
`event_names`, `check_events`, `has_talk`; `session.rs` `talks()`, `units_named`, `check_phase_events` (after the hp_below
check at each block end); the host `state` key `talks`; checks `check/encounter.rs` `event_trigger_errors`,
`talk_effects`, `world_talk_problems` (called from `check/world.rs`) and `check/dialogue.rs` `battle_talk_errors` (node
and choice effects). Choices the spec left open: a `round` trigger may name round 1 (the check's floor moved from 2 to
1: "round must be 1..N"; needed by PT1 and section 10; the hp_below `or_round` floor stays 2; checks parity narrowed,
HISTORY 2026-11-01); the poise-rule check of a `broken` trigger runs for world encounters only (a catalog encounter
plays classic unless the Arena picks a ruleset, so it is not checked there; without poise it never fires); the texts not given in 5.2 are "Phase '`{p}`': block must be
0..`{n}`.", "Phase '`{p}`': trigger unit '`{u}`' is not in this encounter.", "Phase '`{p}`': trigger hero '`{id}`' is not
in this encounter's party.", "Phase '`{p}`': count must be 1..`{n}`.", "Phase '`{p}`': one talk per phase.", "Phase
'`{p}`': a talk needs a dialogue." and the world ref error for an unknown speaker ("Phase '`{p}`': talk speaker: ...");
a downed / broken unit name may also be a guest ref. The loader parity needed no list change in C1 (both `phases` rows
were in HELP_CHANGED already; only its comment names C1); `script` joins AFTER_GODOT with C3. Tests:
`rpgkit_battle/tests/battle_talk.rs`, `rpgkit_content/tests/battle_talk_checks.rs`, `rpgkit_host/tests/battle_talk.rs`.

As built, C2 (2026-11-01; section 3.3): `emberwake_gui/src/talk.rs` holds the box (`talk_layout`, `draw_talk` over a
given game state with an action map `TalkAct`, `talk_key`, `follow_node`, `talk_state`; the region map's look, ids and
tests unchanged, its `draw_talk` a wrapper). `battle.rs`, `battle/talks.rs`: `Action::Talk`, `BattleTalk`, `talk_tick` (each new cue once
`impacts_done`, i.e. after its step's impacts and their gap), `open_talk`, `talk_reply`, `auto_plan` (autoplay's
preset and Confirm held while a talk is open or due, sent when the last closes), `speaker_unit` (a unit on the field
whose `hero_key` is the speaker's qualified id, resolves to it, or is the member who plays it), `TALK_VEIL` (alpha 89),
`preview_talk` (the UI Gallery entry "Battle talk"). Choices the spec left open: the agent link's `talk` also carries
`portraits` (the box's portrait state) and `speaker_unit`; `replies` are the rows as drawn (`{id, text, locked}`,
Continue included); the box pushes a `talk.backdrop` widget so a board hover card does not show through it; Esc and
right click are swallowed too; agent-link host requests (`command`) are not blocked (DT3: the host never waits); the
Arena's scratch state is made at the first talk and dropped at the next battle start (Restart included); a playback
(viewing or take-over) opens the replayed session's cues from the first. Tests: `battle/tests/talk_screen_tests.rs` (on a
scratch world: DT5, DT6, DT8-DT11 and the text audit at the four sizes), `gallery.rs`
`the_battle_talk_entry_shows_a_talk_over_the_battle`.

As built, C3 (2026-11-01; sections 4, 5.1, 5.3): `rpgkit_battle/src/script.rs` (`Script::read` over the row, lenient:
what does not parse is left out; `RoundScript`, `ScriptEntry`, `ScriptPick`, `ScriptTarget`, `UnitName`, `unit_of`,
`plan_entries`, `rest_others`); `session.rs` `round_script` (the current round's script, None without `round_intent`),
`round_locked`, `scripted_rounds`, `script_fallbacks`, `script_party` and the free `pick_on` (SR4's one pick function:
`Session::pick` and both sides of a script call it); `schedule_enemy_ai` plans the enemy entries into the enemy plan,
then `ai::plan_enemy_intents_except` (the AI skips the scripted units, sees their plans, never re-picks or trims them).
SR3's fill is a per-unit override `RoundState.fill` read by `round_intent_plan` `build_picks` (one line; cleared at the
round start), so the player's Auto-fill is not read. `others: auto` is the GH2 a draft with the `balanced` tactic
(`plan_apply_preset_all`, which skips members holding picks); `others: rest` is one Rest pick with the Rest fill. A
locked round: `confirm_plan` at its start; `auto_draft`, the presets and a second `confirm_plan` leave it alone (the
sims and autoplay play it as scripted). Host: the 11 requests of SR5 refused before any other check (nothing recorded);
`state` gains `script` {round, lock, banner} while a round plays scripted (for C4's banner). Choices the spec left
open: SR11's reasons are "is not on the field", "is not in this battle", "has no target: `{t}` is not on the field",
"has no skill '`{s}`'" and "cannot plan its picks: `{refusal}`" (the refusal's full stop dropped); a pick a later
member's rebuild drops (GH5) fails its entry too; the cells' actor is the failed unit (the encounter for SR8's), each
with a `[Script] ...` combat log line; the metrics skip Script cells and read both counts from the session at the
fight's end (round 1 is planned before a sim attaches its metrics); a locked round whose confirm fails logs a fallback
like a failed entry (decision "ENCOUNTER2-C3 answers" (2): one `Script` cell "Script: round `{n}` cannot be locked:
`{reason}`; it plays as a guided round.", its `[Script]` line, counted in `script_fallbacks`; `script::lock_fallback_line`)
and plays guided (no check reaches it: the hold escape's locked round 1 is refused; test
`a_locked_round_that_cannot_be_confirmed_falls_back_to_a_guided_round` builds the row without the check); `others:
auto` drafts with `balanced` until ENCOUNTER2-C4 brings the member's own tactic (the same decision, (1)); the data
re-save was accepted (3); `replan_enemies` (the sim's jitter) plans a scripted
round again whole, its cells and counts once. Check texts beyond 5.1: "Script must be a list of round scripts.",
"Script round must be 1..`{n}`.", "Script lock must be true or false.", "Script others is auto or rest.", "A script
banner is text of at most 40 characters.", "A round script holds 1 to 16 entries.", "Script round: unknown key
'`{k}`'.", "Script entry: unknown key '`{k}`'.", "Script fill is auto or rest.", 'A script target is "self", a unit or
[x, y].', "`{unit}`: items are party picks."; a unit row with no skill list (it knows every skill in battle) and the
basics (the catalog's and the ruleset rows') pass the "has no skill" check. `check/world.rs` calls `script_problems`
(one line). Parity: `script` joins AFTER_GODOT in `emberwake_world/tests/data_source.rs`; every golden unchanged (no
shipped encounter has a script); the shipped encounter files re-saved in the writer's form (`"script": []` only). Tests: `rpgkit_battle/tests/script_round.rs`, `rpgkit_content/tests/
battle_script_checks.rs`, `rpgkit_host/tests/script_round.rs`.

As built, C4 (2026-11-01; section 8's C4 row, decision "ENCOUNTER2-C3 answers" (1)): `emberwake_gui/src/battle.rs`:
`Action::Skip`, `locked`, `can_skip`, `round_over` (the round-end message and the auto-advance, shared by the playback
and Skip), `cutscene` (the locked round whose playback the screen started), `send_tactics`. A locked round: Skip
(`battle.skip`, tip "Resolves the rest of this scripted round at once.") takes the Confirm slot while the round still
plays; Confirm (button, Enter, an agent-link click) does nothing; the tactic menus, Clear, the hotkeys, Self and the board
clicks are off as outside planning; the playback starts by itself at the first tick (after a round-start talk; the
message is the banner plus "(Skip resolves the round.)", or "Round `{n}`: the round plays out (Skip resolves it)."
without one) and, at the round's end, the next round starts after `round_pause_ms` without autoplay (not while
watching a recording); autoplay drafts nothing in it. The banner (`battle.banner`, gold on a card, at the board's top
edge, centred, cut to the board's width) shows in every round that plays scripted, locked or guided; a guided round
keeps planning (no Skip, no playback of its own). Agent link: the screen state's `script` {round, lock, banner, skip}
while a round plays scripted. Tactics: the kit's `script::Tactics` {party, members by hero key} on
`EncounterDef::tactics` (a caller input like `plan_reveal`, never read from the row; none = `balanced`; guests stay
`balanced`), `Session::set_tactics` (the scripted rounds still to plan; the current plan is left alone); host
request `tactics {party?, members?}` (recorded, allowed in a locked round, ids of `AUTO_IDS`) and the `new` key
`tactics`. The screen sends `tactics` after a party or hero tactic pick only in a battle with a script under the round
intent (other battles record no new request; members sorted by hero key), and the Arena's `new` carries
`tactics.party` when the party tactic is not Balanced and the encounter has a script under a round-intent ruleset
(`arena_scripted`; a world fight's screen starts on Balanced). Choices the spec left open: Skip clears a Block-mode hold;
Skip shows the round-end message as the playback would. Tests: `battle/tests/script_screen_tests.rs` (locked controls,
banner and Skip, the playback by itself and the auto-advance, Skip's `resolve` with the same log as the host's,
the guided round, the party tactic's draft), `rpgkit_host/tests/script_round.rs`
(`the_games_tactics_draft_the_heroes_with_no_entry`, `the_tactics_request_drafts_the_later_scripted_rounds_and_replays`).

History: `docs/HISTORY.md` row "Hand-scripted combat layer (10-block rounds, scripted commands)" (deprecated before
2026-10: "Scripted battles could not express planning, AP and AI; double maintenance"). That reason does not hold here:
a scripted round is not a second combat layer. It is data that feeds the same picks the player makes (INPUT1's `pick`)
and the same resolver, for a few authored rounds; planning, AP and the AI stay the only path (section 4, SR4). Row
"Dialogue as a full-width bottom bar (UIR-6)" (replaced by PLAYUI1-D's 400 px box): the battle overlay reuses that
box. Lore decision "Slice group (1.5)" Q-89 (barks after the slice, bosses speak through phase `line`s) stands: a talk
is a scene with a pause, not a bark, and phase lines are unchanged.

## 1. Goal

Two kit hooks, both off unless an encounter names them, so classic and the timeline goldens stay byte-identical. A
phase effect `talk` opens a world dialogue tree over the battle screen; the fight waits until it closes. Encounter
phases fire it (today's triggers, plus `round` at a block, `downed` and `broken`). An encounter field `script` plans
both sides for given rounds; the player's input is locked and the round plays out as a cutscene. The first user is
the prologue's opening in the Alley (`enc_pro_0a`, CAST1_ENCOUNTERS 2).

## 2. Survey (develop at 955e0d1; names confirmed, not crawled)

| Area | Where | Today |
| --- | --- | --- |
| Phases | `rpgkit_battle/src/encounter.rs` (`PhaseDef`, `parse_phases`, `Phases::mark_round`, `check_block_end`) | Triggers `hp_below`, `round`, `arrival` (and `stand`, `spared` reserved); effects line, wave, status, ai, enrage, deck, shake, kill |
| GUI-only effect | `rpgkit_data::SHAKE_EFFECT` | Parsed and kept for the camera; "no rule reads it": the pattern `talk` follows |
| Phase timing | `session.rs` `prepare_round` (round start, before planning), `on_block_start` | Due phases fire at a round start or a block start |
| Enemy plans | `session.rs` `schedule_enemy_ai`; `ai::plan_enemy_intents` | Under `round_intent` the AI sets each enemy's intent and target, then plans it |
| Party plans | `round_intent_plan` (`plan_unit`, `replan`; after INPUT1-U `pick(w, plan, c, at, picks)`) | The member's picks, then the ghost fill |
| Unit refs | `Session::units_by_ref`, `hero_keys` | Phases name enemy-side units by ref (the first active one) |
| Dialogue | `rpgkit_world/src/dialogue.rs` `DialogueRunner::open_scene(d, gs, dialogue, speaker)` | A scene with no services; trees resolve in the game state's world |
| Talk box | `emberwake_gui/src/region.rs` `TalkBox`, `talk_layout`, `draw_talk` (private) | PLAYUI1-D's 400 px box, centred at the bottom |
| Battle screen | `emberwake_gui/src/battle.rs` `BattleScreen` (`phases_seen`, `playing`, `auto`, `arena_world`) | Steps the session per request; the Arena may have no game state |

## 3. Mid-battle dialogue `[kit hook]` (rules) and `[game code]` (overlay)

### 3.1 The `talk` effect (kit)

- DT1. Phase effect `{"type": "talk", "dialogue": ref, "speaker": ref}`: when its phase fires, the session appends a
  talk cue `TalkCue { phase, dialogue, speaker, round, block }` (`block` 0 = the round start) to `Session::talks()`.
  No rule reads it: HP, AP, plans, the RNG, the log cells and the outcome are exactly as without the effect.
- DT2. A phase may hold one `talk`, in any position among its effects; the other effects apply as today, in list order.
- DT3. Sims, autoplay, the host and the Dojo never wait for a talk. The host `state` reply carries `talks` (the cue
  list) only when the encounter has a talk effect, so every other state reply is unchanged.
- DT4. Talks live in world encounters only (dialogue trees are world rows): the check refuses a talk in a catalog
  encounter.

### 3.2 Phase triggers for events (kit; any phase effect may use them)

- PT1. `round` gains `block` (int, 0 default): 0 fires at the round start as today; 1 to `total_blocks` at the start
  of that block. A phase due at the start of round 1 fires before round 1 is planned (`prepare_round`).
- PT2. `{"kind": "downed", "unit": name, "count": 1}`: due at the end of the block in which the `count`-th unit of
  `unit` is down (0 HP; killed by any cause, phase kills included); fires at the next block start, or at the next round
  start after the last block. `unit` is an encounter ref (every unit built from it counts), `party` (any hero) or
  `party:<id>` (section 4.2).
- PT3. `{"kind": "broken", "unit": name}`: due at the end of the block in which a unit of `unit` breaks (RULESET1 P7);
  fires as PT2. It never fires when the battle's ruleset has no `poise` rule.
- PT4. A phase that is due when the battle ends never fires (the end check comes first, as today). Groups (ENCOUNTER2
  P2) and the phase cap (4 per encounter) apply unchanged.

### 3.3 The overlay (game)

- DT5. The battle screen opens each new cue of `Session::talks()` once, in order, after the impacts of the step that
  fired it have played (FX1 rule F flushes first). While it is open: no `step`, `resolve`, `advance` or planning
  request is sent, autoplay waits, hover cards stay, every board click and battle hotkey is swallowed.
- DT6. Opening: `DialogueRunner::open_scene(d, gs, dialogue, speaker)`. A world fight uses the game in play; the
  Arena with a world uses a scratch game state of that world (`world_id` set, nothing else) and drops it after the
  battle. A scene that cannot open (unknown ref, speaker gone) is skipped; its reason goes to the message bar.
- DT7. The box is the region map's talk box, moved to a shared module `emberwake_gui/src/talk.rs` (layout, drawing,
  keys, portraits and expressions as PLAYUI1-D and PORTRAITFX); `region.rs` calls it (its look does not change). The
  battle draws a 35 % black veil over the board and panels and the box above it, centred at the bottom.
- DT8. A speaker on the field: while a node shows, the token whose hero key (or world character) is the node's speaker
  wears the gold selection ring.
- DT9. Replies work as on the region map. Effects: a world fight applies them to the game in play at once; the Arena to
  its scratch state. Only `set_flag` and `add_flag` are allowed in a battle talk (checked, section 5.2).
- DT10. Closing returns to where the cue found the battle: planning, or the playback, which resumes after
  `impact_gap_ms`. Viewing a recorded session shows the talks too (they are not requests: nothing to record).
- DT11. Agent link: the battle `state` gains `talk` (`{dialogue, node, speaker, text, replies}` while open, absent
  otherwise); the box keeps the region map's widget ids, so an agent clicks through as there.

## 4. Scripted rounds `[kit hook]`

### 4.1 Rules

- SR1. Encounter field `script`: a list of round scripts `{round, lock, others, banner, entries}` (schema 5.1). A round
  with no script plays as today.
- SR2. Planning a scripted round (at its round start, after `prepare_round`'s phases): first each scripted enemy entry
  in list order, then the AI for the other enemies (it sees the scripted plans as it sees earlier AI plans); then each
  scripted party entry in list order through INPUT1's `pick` (the entry's picks at once, atomic, GH5), then the
  members with no entry by `others`: `auto` (the member's auto plan, `plan_unit`, as a member with no pick, GH2) or
  `rest` (Rest for the whole round).
- SR3. Fill: an entry's `fill` decides the blocks after its picks: `auto` (the planner's fill: the follow-up order,
  then the deck order, as GH1 with Auto-fill on) or `rest`. The player's "Auto-fill the rest" setting is not read.
- SR4. Enemy entries are planned by the same rules as a party member's picks (one function for both sides; builder's
  choice how), so a script can only plan what a player or the AI could plan: AP, ranges, moves (one regular move,
  INPUT1 Q4), Maneuver positions and windups all apply. A script plans; it never forces a hit, a miss or a roll.
- SR5. `lock` true (default): the plan is confirmed at the round start. The host refuses `preset`, `plan`,
  `add_step`, `undo`, `clear`, `round_intent`, `manual`, `fill`, `pick`, `unpick` and `autofill` with "Round `{n}` is
  scripted."; `confirm` answers ok; `step`, `resolve`, `advance` and `autoplay` work as today.
- SR6. `lock` false (a guided round): the enemies' entries are planned and fixed; the party's entries become the
  members' first picks, which the player may keep, change or undo; the player confirms as usual.
- SR7. Recordings: a locked round records no planning request; replaying a recording replays the script from the data.
- SR8. Under a ruleset without `round_intent` (classic, or the Arena's classic pick) the script is ignored and the
  battle start logs one event cell: "Scripted rounds need the round intent rules; they play unscripted."
- SR9. Metrics: `BattleMetrics` counts `scripted_rounds` and `script_fallbacks` (the sim's report prints both only
  when not zero).

### 4.2 Naming units, skills, targets and moves

- SN1. A unit name is one of: `ref` (the first unit built from that encounter ref: an enemy, a phase-wave unit or a
  guest, as phases name units today); `ref#n` (the n-th unit built from it, 1-based, in the order `Session::units_by_ref`
  lists them, which is the build order of `enemies`, guests and phase waves; ids never hold `#`); `party:<id>` (the hero whose hero key is `<id>`: a
  member's character id in a world fight, the party unit ref in a catalog fight); `party#n` (the n-th deployed hero,
  1-4, in party order).
- SN2. `skill`: a skill family id; the unit's own rank of it is used (`round_intent_plan::family_skill`).
- SN3. `target`: `"self"` (default), a unit name (SN1) or a tile `[x, y]`; it is the round target (INPUT1 IF1).
- SN4. `picks` (1-4, in order; INPUT1's `Pick` shapes): `{"skill": family}` with optional `"kind"` (`engage`,
  `disengage`, `utility`); `{"item": id}` (party only, from the member's pouch); `"rest"`; `{"move": [x, y]}`
  ("Move here", only as the first pick); `{"step_after": [x, y]}` (the after-move). A move then a skill is two picks.

### 4.3 Invalid steps

- SR10. Static mistakes are content-check errors (5.3): the battle never meets them in shipped data.
- SR11. At planning time an entry fails when its unit is not on the field (down, withdrawn, reserved, not deployed), a
  unit-name target is not on the field, or its picks are refused (the refusal text of INPUT1 IF6 and ET3: AP, range,
  path, not offered). The failed unit plays its own round: an enemy by its AI, a hero by its auto plan (whatever
  `others` says). One event cell per failure, subtype `Script`: "Script: `{unit}` `{reason}`; it plays its own
  round." A locked round stays locked; later entries still apply.
- SR12. During the playback a scripted entry fails as any planned entry does (a target that died earlier, a blocked
  path, a protected flank): the resolver's rules, no script rule.

## 5. Schema contract (`world.encounters` and `content.encounters`; schema-first: yes)

### 5.1 Fields

| Key | Type | Default | Validation (error text) |
| --- | --- | --- | --- |
| `script` | json | [] | empty, or 1-3 round scripts ("At most 3 scripted rounds.") |

Round script keys:

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `round` | int | - | 1 to `max_rounds`, unique ("Round `{n}` is scripted twice.") |
| `lock` | bool | true | a locked round 1 is refused with a hold escape ("A hold escape picks its holder before round 1: script round 2 or later.") |
| `others` | enum | "auto" | `auto`, `rest` |
| `banner` | string | "" | at most 40 characters (shown above the board in a locked round) |
| `entries` | json | [] | 1-16 entries; one per unit and round ("`{unit}` has two entries in round `{n}`.") |

Entry keys: `unit` (SN1; "Script unit '`{u}`' is not in this encounter."; `party:<id>` must be in the encounter's
`party`: "Script hero '`{id}`' is not in this encounter's party."), `target` (SN3, default "self"; a tile on the
grid: "Script tile `{c}` is off the grid."), `picks` (SN4, 1-4: "Script picks are 1-4 of skill, item, rest, move,
step_after."; an unknown family: "Unknown skill '`{s}`'."; an enemy-side unit whose unit row and class path hold no
such family: "`{unit}` has no skill '`{s}`'."; an unknown item; `move` not first: "A move is the first pick."), `fill`
(enum `auto`, `rest`; default `auto`).

### 5.2 Phase keys (inside `phases`; the field stays json, its help names the new keys)

- `talk` effect keys `type, dialogue, speaker`: `dialogue` resolves in the encounter's world ("Unknown dialogue
  '`{d}`'."); `speaker` "" or a character there; at most one per phase; in a catalog encounter: "A talk needs a world
  encounter (dialogue trees live in worlds)."; a node effect other than `set_flag` or `add_flag` in that tree: "Battle
  talk '`{d}`' node '`{n}`': effect '`{e}`' cannot run in a battle."
- Trigger `round` key `block` (0 to `total_blocks`); trigger `downed` keys `kind, unit, count` (`unit` SN1 without
  `#n`, or `party`; `count` 1 to the number of units the name covers); trigger `broken` keys `kind, unit` (a world
  encounter whose ruleset lacks `poise`: "A broken trigger needs the poise rule.").

### 5.3 Content tool and the Forge

`tools/fc.sh content upsert encounters @file.json world=<id>` (or a seed through `content apply`), then
`tools/fc.sh content check`; dialogue trees through `content upsert dialogues @file.json world=<id>`. The Content
Forge edits `script` and `phases` as raw JSON (no form work, as ENCOUNTER2 6). A script in a catalog encounter is
allowed (it plays only when the Arena picks the timeline ruleset; the check adds the warning "Plays scripted only
under the timeline ruleset.").

## 6. Tests (rule-derived; scratch data in the kit crates' `tests/`)

| Test | Checks |
| --- | --- |
| `rpgkit_battle/tests/battle_talk.rs` | DT1: a round-1 talk is in `talks()` before round 1 is planned; a `round` + `block` 3 talk at block 3's start; the same battle with and without the talk effect gives identical `log_lines`, HP journal and outcome |
| same | PT2: `downed` fires at the next block start after the first fall, `count` 2 after the second, after a last-block fall at the next round start; PT4: never after the battle's last kill; PT3: `broken` under poise, never without |
| `rpgkit_battle/tests/script_round.rs` | SR2, SR5: a locked round is confirmed at its start; the host refuses `pick` with "Round 1 is scripted."; its schedule equals the one the same `pick` requests build by hand (same log) |
| same | SR11: a downed target, an unowned family and an unpayable pick each give one `Script` cell and the unit's own round; the round stays locked; SR6: a guided round keeps the enemies' entries and lets the player unpick |
| same | SR3: `fill` auto and rest; the Auto-fill setting off changes nothing; SR8: classic ignores the script with one cell; SN1: `ref#2`, `party:<id>`, `party#1` |
| same | SR7, SR9: autoplay and a recording replay play the scripted round; the metrics count it |
| `rpgkit_content/tests/battle_script_checks.rs` | Every error text of 5.1 and 5.2, one per mistake |
| `emberwake_gui` (battle.rs tests) | DT5: with a cue open the screen sends no step and autoplay waits; DT10: closing resumes; DT11: the agent link shows `talk`; a locked round hides the planning controls (C4) |
| text audit | The battle talk box and the banner at 1280 x 720 and the four text sizes; the region map's talk box unchanged |

## 7. Parity

No classic or timeline golden, recorded battle trace or replay has a script, a talk or the new triggers, so all stay
byte-identical (checked by the existing suites, run unchanged). The loader parity (`emberwake_world/tests/
data_source.rs`) adds `script` to AFTER_GODOT and the new `phases` help text to HELP_CHANGED; no recorded expectation
changes. HISTORY rows wanted: section 9.

## 8. Slices and task rows (the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| ENCOUNTER2-C1 | Phase `talk` effect, triggers `round.block`, `downed`, `broken`; `Session::talks`; checks | kit | logic | ENCOUNTER2-U, KIT1-N | `rpgkit_battle/src/{encounter,session}.rs`; `rpgkit_content/src/check/{encounter,dialogue}.rs`; `kits/rpg/schema.json` (help); `rpgkit_host/src/lib.rs` (`talks`); `emberwake_world/tests/data_source.rs`; tests |
| ENCOUNTER2-C2 | Battle talk overlay: shared `talk.rs`, pause, veil, speaker ring, agent link | game | logic | ENCOUNTER2-C1 | `emberwake_gui/src/{talk (new),region,battle,gallery}.rs` |
| ENCOUNTER2-C3 | Scripted rounds: `script` field, planning, lock, fallbacks, metrics, checks | kit | logic | ENCOUNTER2-C1, INPUT1-U | `rpgkit_battle/src/{script (new),session,round_intent_plan,ai,log,metrics,lib}.rs`; `rpgkit_host/src/lib.rs`; `rpgkit_content/src/check/encounter.rs`; `kits/rpg/schema.json`; `data_source.rs`; tests |
| ENCOUNTER2-C4 | Cutscene screen: locked controls, banner, Skip (sends `resolve`), auto-advance | game | logic | ENCOUNTER2-C2, ENCOUNTER2-C3 | `emberwake_gui/src/battle.rs` |
| ENCOUNTER2-CA | The prologue's scripted opening (`enc_pro_0a`, section 10) | game (data) | asset | ENCOUNTER2-C4, CAST1-A6 | `data/worlds/sunk_ward/{encounters,dialogues}.json` |

C2 and C4 touch `battle.rs` as INPUT1-U, -T, -PB and -AP do: they run one at a time. C4's screen (game): in a locked
round the strip, the skill menu, the hotkeys and Confirm are hidden or disabled; the banner (if set) sits above the
board; a Skip button (`battle.skip`) resolves the rest of the round (same result, no animation); after the round the
next one starts on its own after the usual round-end pause.

## 9. Docs delta, ROADMAP and HISTORY rows (for the planner)

- GAME_BRIEF: the `talk` effect, the three triggers, scripted rounds (one line each, pointer here).
- STRUCTURE: `rpgkit_battle/src/script.rs`, `emberwake_gui/src/talk.rs`.
- `engine/docs/authoring/combat_hooks.md`: game-agnostic sections "Battle talk" and "Scripted rounds".
- ENCOUNTER2 ROADMAP row: notes gain "C (mid-battle dialogue, scripted rounds; M2)".
- HISTORY: (1) "Hand-scripted combat layer" is revisited by scripted rounds: data that feeds the planner's picks for a
  few rounds, not a parallel layer (the old reason, no planning, AP or AI, does not apply); (2) "Godot parity over the
  encounter schema" narrowed: `script` joins AFTER_GODOT, the `phases` help joins HELP_CHANGED.

## 10. Data: the prologue's opening (ENCOUNTER2-CA; FP, draft lines for the user)

Added to CAST1-A6's `enc_pro_0a` row (everything else as CAST1_ENCOUNTERS 2). The lines are drafts (Q-C6).

| Key | Value |
| --- | --- |
| `script` | `[{"round": 1, "lock": true, "others": "auto", "banner": "", "entries": [E1, E2, E3]}]` |
| E1 | `{"unit": "footman_patrol", "target": "party:kaelen", "picks": [{"skill": "heavy_bill_thrust"}], "fill": "auto"}` |
| E2 | `{"unit": "footman_patrol#2", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "rest"}` |
| E3 | `{"unit": "party:kaelen", "target": "footman_patrol", "picks": [{"skill": "pommel_strike"}], "fill": "auto"}` |
| `phases` | `opening` (trigger `round` 1, block 0; effects `talk` `dlg_pro_0a_opening`, speaker `kaelen`); `tremor` (as CAST1, then a second effect `talk` `dlg_pro_0a_your_turn`, speaker `kaelen`); `first_down` (trigger `downed` `footman_patrol` count 1; effect `talk` `dlg_pro_0a_first_down`, speaker `kaelen`) |

Dialogue trees (`sunk_ward` `dialogues`; `speaker` `kaelen`, `services` false, node ids `a`, `b`):

| Tree | Node | Expression | Text | Then |
| --- | --- | --- | --- | --- |
| `dlg_pro_0a_opening` | a | angry | "Two of them. The big one is hauling his bill back for a full thrust." | next b |
| | b | neutral | "Slow. A quick blow to the jaw breaks that before it lands." | end |
| `dlg_pro_0a_your_turn` | a | neutral | "Watch their shoulders. When one winds up, strike first." | end |
| `dlg_pro_0a_first_down` | a | hurt | "One down. The other will not wait." | end |

Acceptance: `content check` clean; in the Arena (`sunk_ward`, timeline, seed 1337) and `aegis battle` the round-1
log shows Kaelen's Pommel Strike breaking Footman 1's Heavy Bill Thrust and no `Script` cell. If the break does not
land, the asset builder may add one `{"move": [x, y]}` as E3's first pick and reports; nothing else without asking.

## 11. Out of scope

Battle barks (Q-89, after the slice); talk effects from unit triggers (HOOKS1 8.1); dialogue effects that change the
battle (a surrender, a recruit mid-fight); forced hits or rolls; scripted placement or teleports; scripts under the
classic ruleset; camera moves and pans; a "skip battle dialogue" setting; Content Forge forms for `script`.

## 12. Open questions (recommended option first)

- Q-C1. Guided rounds (`lock` false, SR6): (a) keep them (enemies scripted, the player plans; cheap, tutorial-friendly);
  (b) locked rounds only. Recommend (a).
- Q-C2. Replies in a battle talk: (a) allowed, `set_flag` and `add_flag` effects only; (b) linear scenes only.
  Recommend (a).
- Q-C3. Spoken lines in the combat log: (a) no, the talk box only (phase `line`s already reach the log); (b) each line
  as a log row. Recommend (a).
- Q-C4. `fill` default: (a) `auto` (fewer keys to author); (b) `rest` (only what is written plays). Recommend (a).
- Q-C5. Skip in a locked round: (a) a Skip button that resolves the round; (b) no skip. Recommend (a).
- Q-C6. The opening's content (section 10): (a) round 1 as a locked demonstration of the interrupt, a talk before it,
  a talk at round 2 handing over, one at the first kill, the draft lines revised by the user; (b) round 1 as a guided
  round (the player finds the interrupt). Recommend (a), lines to the user.
- Q-C7. The phase cap of 4 per encounter: (a) unchanged (a talk shares a phase); (b) raise to 6 for talks. Recommend
  (a) until data needs more.
