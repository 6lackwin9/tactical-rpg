# STORYUI1 tasks: assets, tests, parity, task split, docs delta and open questions

Status: proposed (spec 2026-10-25); the rules are in [STORYUI1.md](STORYUI1.md) (section numbers below are its).

## 1. Assets (STORYUI1-A, asset lane)

### 1.1 Placeholder rule

Every card and screen uses existing assets or code-drawn art (the card backdrop, 4.2): the best-matching existing
file, no new art generated (Q-81: final art in the M4 pass). Backdrop colours are art-bible hues or a stated mix of
two (`#2c1546` = Blight Violet `#9333ea` at 25% over Void Abyss `#0a0b10`; `#093641` = Salt Cyan `#06b6d4` at 25%
over Void Abyss; `#5b1f24` = Rust Crimson `#991b1b` at 50% over Wet Slate `#1e232d`). Card texts quote the canon
lore docs (`narrative/new_game_onboarding_and_intro.md` 2-4, `narrative/prologue_and_starter_squad_lore.md` 3-4);
three texts marked "placeholder" are not canon (SQ8).

### 1.2 Vignettes (world `sunk_ward`, kind `vignettes`)

| Id | music | skippable | once | Cards (style: image or backdrop; speaker; text; fade in / hold / fade out ms; cue) |
| --- | --- | --- | --- | --- |
| `intro` | "" (the menu theme goes on) | true | false | 1 image: "", backdrop `#0a0b10` to `#2c1546`, ash; "Two centuries ago the stars and the sun went out. In their place hangs the Shroud. No morning comes. Time is kept only by the cathedral bells."; 1200 / 3500 / 600. 2 image: "", backdrop `#14171f` to `#0a0b10`; "Beneath the basalt sleeps the Low Sleep, an ancient dreaming leviathan whose nightmares shake the earth and bleed marrow into the wells."; 800 / 3500 / 600. 3 image: `backgrounds/location/ruined_courtyard.png`; "The Clerisy says that if the titan wakes, the world breaks, and that the Red Tithing, sung in young blood each generation, is the lullaby that keeps it asleep. Those who live past five-and-twenty are spent, docile and blind."; 800 / 4500 / 600. 4 image: "", backdrop `#1e232d` to `#5b1f24`; "In the Seventh Ward, a garrison guard refuses to lock the cages."; 800 / 3000 / 1200 |
| `cold_open` | `none` | true | false | 1 text, backdrop `#1e232d` to `#0a0b10`; "The Alleyway of Cinders, Seventh Ward. Night. Rain."; 600 / 2000 / 400. 2 text, same backdrop; "Rise"; 400 / 0 / 300. 3 line, Vane: "Order 44, deserter. The culling cages aren't full yet. You were sworn to drive the young stock into the pens, not break your corporal's jaw."; 300 / 0 / 200. 4 line, Kaelen: "They're fifteen years old, Vane. You're herding children into the crypts."; 200 / 0 / 200. 5 line, Vane: "They're tithes for the bells, boy. Draw your steel, or take their place on the wagons."; 200 / 0 / 200. 6 text: "Vane withdraws. Two Castellan Footmen close in."; 300 / 1500 / 300 |
| `meat_market` | "" | true | false | 1 image: `backgrounds/location/ashmaw_camp.png`; "Kaelen runs through the Lower Meat Market, past cauldrons that boil carcasses and unclaimed corpses into warded tallow."; 600 / 0 / 400. 2 same image: "The Spent stand in silent queues with wooden bowls, trading curfew badges for communion draughts and cups of grey salt. None of them look up from their cups."; 400 / 0 / 400 |
| `rooftop` | "" | true | false | 1 text, backdrop `#14171f` to `#0a0b10`; "Inquisition whistles shriek. An ambush squad corners Kaelen on the Meat Market rooftops."; 400 / 0 / 300. 2 line, Vesper: "Move your feet, tin-can. Grates are open. Follow the smoke."; 200 / 0 / 300 |
| `bridge` | "" | true | false | 1 text, backdrop `#14171f` to `#1e232d`; "The canal bridge. An iron mortuary cart, and two prisoners inside it."; 400 / 0 / 300 |
| `dive` | "" | true | false | 1 line, Vesper: "Down! If they catch us on the cobblestones, we're ash!"; 300 / 0 / 200. 2 text, backdrop `#0a0b10` to `#093641`; "The four drop into the black runoff and vanish beneath the city."; 400 / 2000 / 800 |
| `hearth` | `music/town.ogg` (kit) | true | false | 1 image: `backgrounds/location/warden_keep.png`; "The Sluice Undercroft. Water drips into the Water-Lock Basin; Bram's charcoal forge glows at the Tinker's Hearth; at the Apothecary's Slab, Sister Aline wipes black ichor from her scalpels."; 1000 / 0 / 400. 2 line, Maren: "Bone spurs. Grafted, not grown: these are surgical scars. Who did this to you?". 3 line, Kaelen: "Garrison surgeons, under an Inquisition seal. They called it a trial. Nobody asked me. How long?". 4 line, Maren: "I have only read about this. If it keeps growing, four months before it locks your elbow.". 5 line, Vesper: "We don't have four months, soldier. The Red Tithing is in six weeks. The Watch is sealing every surface grate in the ward. If we sit down here in the dark, they'll flush the drains with quicklime. If we go up, we need steel, salt and targets.". 6 line, Hollis: "Steel I can use. Plans are for people who intend to grow old. None of us four will see five-and-twenty. So let's crack as many iron masks as we can before the stone takes us.". 7 line, Bram: "Enough talk. You survived the street sweep. That makes you either very lucky or very dangerous. There are six dry bunks in the Hollows behind me. Bread and salt on the shelf. Sleep while you can, young blood. When the next cathedral bell tolls, the war begins."; lines 2-7: 200 / 0 / 200 |
| `doom_clock` | "" | true | false | 1 title, backdrop `#0a0b10`; title "42 days", subtitle "until the Red Tithing"; 800 / 2500 / 600 |
| `act1_title` | `none` | true | false | 1 title, backdrop `#0a0b10` to `#14171f`; title "Act 1", subtitle "The Young Vanguard"; 1200 / 3000 / 1200 |
| `slice_end` | `none` | true | false | 1 image: `backgrounds/location/cinder_gate.png`; "Through the Slag Gate. {member:crucible_stayed} held the winch, and the Inquisition took them alive."; 1000 / 0 / 600. 2 title, backdrop `#0a0b10`; title "End of the slice", subtitle "Thank you for playing. Act 2, The First Rupture, comes later." (placeholder); 800 / 0 / 1200 |
| `act1_bad_end` | `none` | true | false | 1 text, backdrop `#1e232d` to `#5b1f24`; "Too late. The cell could not stop what came next." (placeholder); 800 / 0 / 600. 2 title, backdrop `#0a0b10`; title "The run ends here", subtitle "Load the start of Act 1 to try again." (placeholder); 800 / 0 / 800 |

Speakers are the world characters SUNKWARD1 and CAST1 create (`kaelen`, `vesper`, `maren`, `hollis`; Vane by his
CAST1 id, `vane` or `sw_vane`; `bram`); cues stay "" (no fitting sound exists, 1.5).

### 1.3 Sequences, board, quests, manifest, service line

| Kind | Id | Content |
| --- | --- | --- |
| `sequences` | `prologue` | 1 vignette `intro`; 2 vignette `cold_open`; 3 fight `enc_pro_0a`, `on_lost` `retry`; 4 vignette `meat_market`; 5 vignette `rooftop`; 6 fight `enc_pro_0b`, `retry` (Vesper as a guest, CAST1); 7 effects `recruit` `vesper`; 8 vignette `bridge`; 9 fight `enc_pro_0c`, `retry` (Maren and Hollis as guests, CAST1); 10 effects `recruit` `maren`, `recruit` `hollis`; 11 vignette `dive`; 12 vignette `hearth`; 13 effects `set_flag` `act` 1 (plus CAMPAIGN1's Doom Clock effect once it exists); 14 vignette `doom_clock`; 15 vignette `act1_title`; 16 enter `sluice_undercroft`, anchor "" |
| `sequences` | `slice_end` | 1 effects `set_flag` `roster_open` true (ROSTER1_TASKS: roster management opens after the Crucible); 2 vignette `slice_end`; 3 end `menu`. Started by CAMPAIGN1's trigger on the win of `enc_evt_06` (or `m5_crucible` turning ready) |
| `boards` | `tess_board` | name "Dead-drop board", keeper `tess`, intro "", `requires_flags` `[{"key": "act", "op": ">=", "value": 1}]` |
| `quests` | `m3_bone_mill` | name "The Bone-Mill Raid", giver `tess`, board `tess_board`, track `main`, template `reach_and_clear`, steps: reach `fourth_kiln` ("Reach the Fourth Kiln"), win `enc_evt_03` at `fourth_kiln` ("Raid the bone-mill"), `requires_flags` `act` >= 1; rewards the template's until the balance lane sets the slice's stocks (Q-41) |
| `quests` | `m5_crucible` | name "The Crucible", giver `tess`, board `tess_board`, track `main`, `requires` [`m3_bone_mill`], template `reach_and_clear`, steps: reach `crucible_watchpost`, win `enc_evt_06`; its checkpoint day is CAMPAIGN1's data |
| `quests` | the dilemma job (CAST1 CQ9) | only `board` `tess_board`, track `side`; its row and choice come with its content task |
| world manifest | `sunk_ward` | `start_sequence` `prologue` (through the manifest path SUNKWARD1 uses) |
| `services` | `board` | "Show me the board." |

Location ids follow SUNKWARD1 (`sunk_ward_campaign_map.md` 5 sketch: `sluice_undercroft`, `fourth_kiln`,
`crucible_watchpost`); Tess, Bram and Aline are SUNKWARD1's hub NPCs; Tess needs no dialogue tree (the greeting and
the service choices suffice).

### 1.4 Seed

`games/emberwake/data/seeds/storyui1.json`, steps in dependency order: the services line, vignettes, sequences, the
board, the two quests, the dilemma job's board fields, the manifest; applied with `tools/fc.sh content apply
games/emberwake/data/seeds/storyui1.json`, then `tools/fc.sh content check` clean (V, S and B checks, 3.1 below).

### 1.5 Placeholders and poor matches (for the user to approve new art or sound later)

| Need | Placeholder | Match |
| --- | --- | --- |
| Intro slide 1, the sunless violet sky with ash | code-drawn gradient and ash | fair |
| Intro slide 2, the earth's cross-section, titan bones, black veins | code-drawn gradient | poor |
| Intro slide 3, iron masks over chained youths at crypt arches | `ruined_courtyard.png` (arches only, an ember sky) | poor |
| Intro slide 4, a watch insignia in bloody mud | code-drawn gradient to a crimson mix | poor |
| Meat Market rendering station and queues | `ashmaw_camp.png` (tents and fires) | poor |
| The hearth (Undercroft vault, forge glow) | `warden_keep.png` (stone hall, glowing arch; red banners do not belong) | fair |
| Slice end, the Slag Gate | `cinder_gate.png` (a ruined gate, an ember sky) | good |
| Bell toll, rain, dripping gutters (intro, cold open) | none (silence or the menu theme) | missing |
| Hub music for the hearth | kit `music/town.ogg` | fair |
| Portraits of Vane, Tess, Bram, Aline | the kit's common portraits through the body chain (`warrior_base.png` and the like) | poor |
| Board service glyph | the quest badge `ui/badges/quest.png` (kit) | good |
| The `sunk_ward` world card art, the main menu (an orange sun in a sunless world) | today's Ashlands-era files | poor |

## 2. Tests derived from the rules

| Id | Task | Test | Rule |
| --- | --- | --- | --- |
| TE1 | E | A two-card reel walks fade in, typing, hold, fade out per card; each press moves exactly one phase | V1, V3 |
| TE2 | E | After 1000 ms of typing, `normal` shows 60 characters, `slow` 30, `fast` 120; `instant` all at once | V2 |
| TE3 | E | `hold_ms` 500: the fade out starts at 500 ms after the text is out, not earlier; 0 waits forever | V4 |
| TE4 | E | Skip ends a skippable reel with no later cue; a reel that is not skippable plays on | V5, V7 |
| TE5 | E | Cover alpha is 128 (within 1) at a fade's midpoint; `fast` halves the time, `instant` gives 0 | V6 |
| TE6 | E | A cue fires once per card on fade in; none for cards after a skip | V7 |
| TE7 | E | Music wish: "" None, `none` Some(""), a path Some(path) | V8 |
| TE8 | E | The line breaks of a half-typed caption equal those of the full caption | V2 |
| TE9 | E | A 360-character caption at the four text sizes: `text_audit` reports no cut, edge or overlap | V9, V14 |
| TE10 | E | The model never reads a clock: the same calls give the same frames | V12 |
| TS1 | S | One broken fixture per check (style, required fields, lengths, timing ranges, colours, tokens, refs, step kinds, step order, banned effects, board keeper, quest `board` without a template, giver not the keeper) fails with its message; a clean fixture passes | 4.1-4.7 |
| TS2 | S | The shipped Ashlands, Iron Reaches and Tidewrack data check as before (no new message) | parity |
| TW1 | W | A `once` vignette is not queued after it was seen; a queued one is not queued twice | W1 |
| TW2 | W | `when`-filtered cards and tokens resolve from the state at play time; an all-filtered vignette is marked seen | W3, W4 |
| TW3 | W | `start` runs the leading `effects` steps; refused while one runs | Q1, Q3 |
| TW4 | W | A won story fight moves on; lost with `retry` restores full HP and keeps the step; `abort` clears `running` and wakes the party; `continue` moves on | Q5 |
| TW5 | W | `enter` places the party at the location; `end` `menu` clears `running` | Q6, Q7 |
| TW6 | W | A save round trip keeps `story`; an empty `story` writes no key; a bad `running` is dropped with a warning | 4.8, Q8 |
| TW7 | W | New game: a world with `start_sequence` sets `running`; the Ashlands start without it | 5.4 |
| TW8 | W | Board sections by status (main, jobs, under way, locked with `show_locked`, closed with `closed_text`) | B5 |
| TW9 | W | Board quests are missing from the keeper's offers; the `board` service shows only while the board is open | B1, B3 |
| TW10 | W | Accept starts the quest; a quest whose giver is someone else is refused | B8 |
| TW11 | W | `npc_services` lists `board` only with an available board quest; `offer` ignores board quests | B4 |
| TW12 | W | `travel::view` gives `jobs` only after the board was opened, and no `jobs` or `deadlines` key when empty | B9, B10 |
| TU1 | U1 | `enter_game` with `running` shows `StoryStage`; it plays a vignette step, then starts the story fight | 5.4, Q5 |
| TU2 | U1 | A queued vignette plays when the region map ticks with nothing open, and not while a conversation is open | W2 |
| TU3 | U1 | The gallery's vignette entry draws without a game | V15 |
| TU4 | U2 | Opening the board from the keeper's service choice shows the pane; Accept on a row makes the quest active; right click returns to the conversation | B3, B8, B11 |
| TU5 | U2 | The board pane at the four text sizes: no cut or overlap | B11 |
| TU6 | U2 | The world map shows the badge and the "Job:" tooltip line on the job's node; the Ashlands map draws as before | B10 |

## 3. Content tool path, parity plan

### 3.1 Content tool path

The asset lane writes only through `tools/fc.sh content` (seed 1.4). STORYUI1-S teaches the writer the three kinds
(`source.rs` `WORLD_KINDS` from 13 to 16, `store.rs` storage), the quest and manifest fields, and the checks: V (cards
and vignettes), S (sequences and steps), B (boards and board quests), plus the three effects in `check/dialogue.rs`.
Image, cue and music paths that no asset layer holds are warnings (the asset checks' style), not errors.

### 3.2 Parity (the Ashlands untouched)

- No data change in `ashlands`, `iron_reaches` or `tidewrack`; every new field defaults to off; the new kinds are
  absent from those worlds.
- `data_source.rs`: `AFTER_GODOT` gains `("world", "quests", "board")`, `track`, `show_locked`, `closed_text` and
  `("world", "world", "start_sequence")`; the three new kinds are stripped at kind level (the mechanism DERIVE1-S
  adds for `derived_stats`; whichever task lands second reuses it). A `docs/HISTORY.md` row names them.
- Saves: `story` is written only when not empty, so `saves.rs` replays byte-identical; `journey.rs` and
  `world_traces.rs` see no new key (B10 keys and the runner's `board` view key appear only when used).
- `services.json` gains one line; the loader parity reads `legacy/data`, so it is unaffected; `checks_parity` sees
  no Ashlands change.
- `trace_parity`, `phasefold`, sim parity and replays: no battle rule changes.
- Switch-off: a world without `start_sequence`, boards or vignettes plays exactly as today.

## 4. Task split (one builder run each)

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| STORYUI1-E | Vignette player: the card model (phases, typewriter, fades, skip, cues), drawing, `VignetteScreen`, the sample reel, the `text_speed` and `reduce_motion` keys | engine | logic | - | `engine/crates/aegis_ui/src/{vignette (new),lib,screen}.rs`, `engine/crates/aegis_ui/tests/vignette.rs` (new) |
| STORYUI1-S | Story and board schema: kinds `vignettes`, `sequences`, `boards`; quest fields; `start_sequence`; effects; checks; writer; parity strips | kit | logic | - (reuses DERIVE1-S's kind strip if it landed) | `games/emberwake/schema.json`, `emberwake_content/src/{source,store}.rs`, `check/{mod,world,quest,dialogue,story (new),board (new)}.rs`, `emberwake_content/tests/story_checks.rs` (new), `emberwake_world/tests/data_source.rs` |
| STORYUI1-W | Story and board rules: queue, seen, tokens, sequences and the story fight runner, board view and accept, the board service and badge, map jobs, the `story` state field, the new game's start sequence | kit | logic | STORYUI1-S | `emberwake_world/src/{story (new),board (new),dialogue,explore,travel,state,save,new_game,lib}.rs`, `emberwake_world/tests/{story,board}.rs` (new) |
| STORYUI1-U1 | Story stage: `StoryStage`, queue playback on the maps, runner `story` in the battle settle, `enter_game`, the two settings, the gallery's vignette pane | kit (in `emberwake_gui` until KIT1) | logic | STORYUI1-E, STORYUI1-W | `emberwake_gui/src/{story (new),screens,battle,region,worldmap,menus,gallery}.rs` |
| STORYUI1-U2 | Board pane in the conversation, the service glyph and text, world-map job markers, the gallery's board pane (a fixture view) | kit (in `emberwake_gui` until KIT1) | logic | STORYUI1-U1 | `emberwake_gui/src/{board (new),region,worldmap,gallery}.rs` |
| STORYUI1-A | Slice story data: the vignettes, the `prologue` and `slice_end` sequences, Tess's board, the two board quests, the service line, `start_sequence` | game (data) | asset | STORYUI1-S, SUNKWARD1, CAST1-A6; the Doom Clock step after CAMPAIGN1's meter task; `slice_end`'s trigger after CAMPAIGN1's triggers | `games/emberwake/data/worlds/sunk_ward/{vignettes,sequences,boards,quests,world}.json`, `games/emberwake/data/dialogue/services.json`, `games/emberwake/data/seeds/storyui1.json` |

Hub files (shared with other tasks; the planner serialises them): `schema.json` and `data_source.rs` (DERIVE1-S,
RULESET1-R, ENCOUNTER2), `dialogue.rs`, `state.rs`, `save.rs` (ENCOUNTER2-W), `battle.rs` in the GUI (ENCOUNTER2-U,
STAGE1-L, FX1), `region.rs` and `worldmap.rs` (DIR4-M, CAMPAIGN1's HUD work), `gallery.rs`, `menus.rs` (FX1's
reduce-motion control). U1 and U2 both touch `region.rs`, `worldmap.rs` and `gallery.rs`, so U2 runs after U1.

## 5. Docs delta

- `docs/STRUCTURE.md`: `aegis_ui` row (`vignette.rs`, the card player), `emberwake_world` row (`story.rs`,
  `board.rs`), `emberwake_content` row (`check/story.rs`, `check/board.rs`), `emberwake_gui` row (`story.rs`,
  `board.rs`), the world kinds in the data row.
- `engine/README.md` (the vignette player and its settings), `engine/docs/PORT_NOTES.md` (`vignette.rs`: engine-only,
  no Godot source).
- `games/emberwake/docs/GAME_BRIEF.md` "where to look" (story sequences, the board), `games/emberwake/docs/SCREENS.md`
  (the story stage, the vignette screen, the board pane), `games/emberwake/data/dialogue/README.md` (the `board` line),
  the world data README (the three kinds).
- `engine/docs/authoring/`: a game-agnostic note on story beats on Aegis (vignettes, sequences, boards), per the
  "promote generic methods" rule.
- `docs/HISTORY.md`: the parity strip row (3.2).
- `games/emberwake/docs/ROADMAP.md`: rows "Vignette player" (engine), "Story sequences" and "Mission board" (kit),
  `proposed`.

## 6. Conflicts between CANON, the lore docs and the code

1. Order of the last prologue beats: CANON 3.1 has the hearth scene, the Doom Clock, then the Act 1 title card;
   `prologue_and_starter_squad_lore.md` 4 fades to the title card and then shows the clock. CANON wins (5.4).
2. Dialogue offers every available quest of its giver as its own choice (`dialogue.rs:404-413`); a board needs them
   gathered (B3).
3. `recruit` is refused at `party_max` 4 (`dialogue.rs:680`): Dunstan's M3 join fails until ROSTER1's roster cap.
4. Rewards, costs and lock reasons say "gold" and travel tooltips "supplies" (`dialogue.rs:221`, `worldmap.rs:177`);
   CANON says Scrap and Warded Tallow (CAMPAIGN1's purse names).
5. A lost world fight wakes the party at the last safe place (`explore.rs:528`); CANON's Act 1 loss is a retreat to
   the hub, all wounded, a day passing (Q-20; CAMPAIGN1, ROSTER1).
6. The world map marks only active targets (`worldmap.rs:164`); Q-30 wants board jobs on their nodes (B10).
7. No dialogue surface exists off a region map (`region.rs:1234`), so the prologue's and the hearth's lines are
   `line` cards.
8. The main menu art shows an orange sun (`backgrounds/menu/main_menu.png`) in a world whose sun went out (art only).
9. The onboarding doc says the board then offers M1, The Sweep; the slice's first main mission is M3 (Q-84 a).

## 7. Open questions

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| SQ1 | Between the prologue fights (the onboarding doc leaves "vignette or scripted walk" open) | (a) a linear sequence of cards, the fights started from it; (b) small walkable region maps (alley, market, rooftop, bridge) with triggers | a for the slice; b later if the playtest wants it |
| SQ2 | The hearth scene (tagged "dialogue or vignette player") | (a) `line` cards before the hub map appears; (b) a dialogue on the hub's region map | a: it fades to the title card as the prologue doc says |
| SQ3 | Finishing a board mission | (a) report to Tess (today's turn-in, rewards and picks; the next main mission appears after the report); (b) it completes on the win | a |
| SQ4 | Job markers on the world map | (a) after the board was opened once; (b) always; (c) only accepted jobs (today's "!") | a |
| SQ5 | Skipping a scene not yet seen | (a) ask first, Yes the default; (b) skip at once | a |
| SQ6 | Layer of the vignette player (the layer rule lists it as a kit hook) | (a) the player in the engine, its data and sequences in the kit; (b) all of it in the kit | a, like the timeline widget |
| SQ7 | Pacing | (a) the intro moves on by itself (about 30 s, as the onboarding doc says) and story lines wait for a click; (b) every card waits | a |
| SQ8 | Texts of the slice end and the Act 1 bad-end cards (no canon text) | (a) ship the placeholders of 1.2, the design lane writes the final text with the M2b data; (b) write them before the slice | a |
| SQ9 | Poor-match art and sound (1.5) | (a) placeholders until the M4 art pass (Q-81); (b) briefs now for intro slides 2-4, the Meat Market and a bell toll (image API asked first) | a |
| SQ10 | After the slice end card | (a) the main menu, the autosave kept (loading it returns to the Sunk Ward after the Crucible); (b) the main menu, the autosave rolled back to before the Crucible | a |
