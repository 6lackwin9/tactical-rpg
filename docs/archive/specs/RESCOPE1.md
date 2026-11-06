# RESCOPE1: the task queue on the Aegis engine (planner proposal, 2026-10-19)

Status: approved (user 2026-10-19: the eight PORT rows, the three closures and the order of section 6, as recommended). Written by the planner session after the transition (decision "Engine transition (user 2026-10-19)"). Planning only: no code or data changes.
(user 2026-10-19)"). Planning only: no code or data changes. `docs/TASKS.md` on this branch already carries the
re-scoped Touches and dependencies and archives the done rows; the user approves (or changes) sections 2-4 before it
lands on `develop`.

## 1. What the re-scope found

Most approved rows translate directly: the rules they build on are in the engine, checked step for step against Godot
(`engine/docs/PORT_NOTES.md`), and every Godot screen has an engine counterpart (`games/emberwake/docs/SCREENS.md`). Rows that
named `scripts/ui/...` now name the matching `engine/crates/aegis_gui/src/*.rs` screen; rows that named
`scripts/combat/...` or `scripts/content/...` name the `aegis_battle` / `aegis_world` module and
`engine/data/schema.json`; data rows go through `aegis content` with a JSON seed.

The re-scope also found Godot systems that never reached the engine. The parity suites could not catch them: they are
display, measurement or tooling layers the traces do not record. Several approved rows depend on them.

| Gap | What Godot had | Engine today | Blocks |
| --- | --- | --- | --- |
| Battle report and metrics (BREPORT1-L) | `battle_metrics.gd` (765 lines: per-unit damage, healing, kills, statuses, best moment, live round checkpoints) and `battle_report.gd` (164) | nothing; the results screen has no report table | BREPORT1-U, LEVERS1-L2 (`ward_absorbed` metric), sim metrics |
| Enemy intent tells (INTENT1-L) | `intent_rules.gd`: kind of the first action, target or aimed tile, Holding / Arriving, block range, foresight and enrage reveal | one arrow per enemy to its first attack target | MORALE1-U, PACE1-U (they extend the tells) |
| Balance sim modes | `tools/balance_sim.gd` + `tools/sim/*`: tier, levels, group, stats, party, role, sweep, campaign, world encounters, sim metrics and scenarios | `aegis sim duel` only (step for step against Godot); every mode since PORT-SIM (2026-10-20, `aegis sim <mode>`) | BAL5-R1b, PACE1-S, PACE1-B, PURSE1-B, LEVERS1-B, LEVERS1-L2 (sim priest loadout) |
| Sound and music (S14-L3, MUSIC1) | SoundBank, AudioDirector, MusicDirector, placeholder SFX | no audio at all | nothing queued, but the game is silent |
| Settings applied | text size, display mode, volumes applied live | values saved, not applied (SCREENS 03-06) | UIR rows checked "at four text sizes" (HEAL1-U, BREPORT1-U, UIR-4) |
| Keyboard focus navigation | focus moves through every menu | main menu only (SCREENS "Known gaps") | UIR-7 (controller support builds on it) |
| Playtest recording and replay (PLAYTEST1) | recorder, replayer, trail, F8 notes | none (the command API's `history` replays a battle) | PLAYTEST2, CAMP1-P (recorded run) |
| Asset manifest and registry | aliases, team overlay, manifest rows, strips planned | reads `assets/art/<key>_<variant>.png` directly (since ASSETORG1 `<category>/<key>/<variant>.png`); the manifest is not read or written | SPRITE1-L (strips) |
| Content checks of the Forge | SkillValidator, gear budget warnings, world reference rules | the writer checks schema, references and a clean reload | asset rows rely on review for the rest |

## 2. Proposed new rows (status `proposed`, for the user)

| Id | Title | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- |
| PORT-INTENT | Enemy intent tells on the engine: port `IntentRules` (kind, target / cell, Holding / Arriving, blocks, foresight and enrage reveal) and draw the tells on the battle screen; checked against a Godot export of the tells per round | logic | - | `engine/crates/aegis_battle/src/` (new `intent.rs`, `session.rs`), `aegis_host`, `aegis_gui/src/battle.rs`, `engine/parity/godot/trace.gd` (tells in the recording) |
| PORT-BREPORT | Battle metrics and report on the engine: port `BattleMetrics` and `BattleReport` (per-unit numbers, best moment, report rows); checked against a Godot export of the report for the world traces | logic | - | `engine/crates/aegis_battle/src/` (new `metrics.rs`, `report.rs`), `aegis_world/src/battle.rs`, `engine/parity/godot/trace.gd` |
| PORT-SIM | Balance sim modes on the engine: tier, levels, group, stats, party, role, sweep, campaign and world encounters, with sim metrics and scenarios (BAL2_SIM, BAL3, BAL4-ATTR); checked against Godot runs of each mode | logic (balance tooling) | PORT-BREPORT (metrics) | `engine/crates/aegis_battle/src/sim.rs` (or a new `aegis_sim` crate), `aegis_cli` (`aegis sim <mode>`), `tools/fc.sh sim` |
| PORT-AUDIO | Sound and music on the engine: sound events, the music chooser and crossfade (S14-L3, MUSIC1), volumes from settings | logic | PORT-SETTINGS | `engine/crates/aegis_gui/` (new audio module, an audio crate dependency), `assets/audio/` |
| PORT-SETTINGS | Apply the saved settings: text size, display mode, volumes | logic | - | `engine/crates/aegis_gui/src/{menus,ui,main}.rs` |
| PORT-FOCUS | Keyboard focus navigation on every screen | logic | - | `engine/crates/aegis_gui/src/ui.rs` and each screen |
| PORT-PLAYTEST | Playtest recording and replay on the engine (PLAYTEST1): record a session through the agent link and replay it | logic | - | `engine/crates/aegis_gui/src/bridge.rs`, `aegis_host`, `aegis_world` |
| PORT-CHECKS | The Forge's remaining content checks in `aegis content check` (SkillValidator rules, gear budget warnings, world reference rules) | logic | - | `engine/crates/aegis_world/src/store.rs` (or a new `check.rs`) |

## 3. Rows to close (user decision)

| Id | Proposal | Why |
| --- | --- | --- |
| CLN-17 | close as obsolete | Godot dojo suites and the Godot test-file split; those suites are frozen in `legacy/` |
| CLN-18 | close as obsolete | cleanup of the Godot Forge widgets; the engine's Forge is a new read-only viewer without them |
| CLN-13 | close as done | the README overview was written at the transition (2026-10-19); the Godot feature log is `legacy/README.md`; per-topic `docs/features/` can come back as a new row if wanted |

## 4. Rows that stay, re-scoped (Touches in `docs/TASKS.md` on this branch)

- Ready on the engine as they are (no missing dependency): UIR-3B, UIR-4, MAPLOG1-U, QUESTTRACK1-U, RCLOSE1-U,
  HEAL1-U (after PORT-SETTINGS for the text-size check), OBJ1-U, MAPNAV1-U, LEVERS1-L1, LEVERS1-L3, LEVERS1-A1,
  LEVERS1-A3, LEVERS1-U (after LEVERS1-L2 for the Ward part), PACE1-A, PURSE1-A, ASHPACE1 (spec needed), PHASEFOLD
  (idea), and the design rows KEEP1, DIFF1, DEEDS1, RESPEC1, OFFHAND1, DESERT1, COMBO-A2 (specs are engine-neutral).
- Blocked by a new port row: BREPORT1-U (PORT-BREPORT); MORALE1-U and PACE1-U (PORT-INTENT); UIR-7 (PORT-FOCUS);
  LEVERS1-L2 (PORT-BREPORT for its metric; its sim loadout part waits for PORT-SIM); PACE1-S, PACE1-B, BAL5-R1b,
  PURSE1-B, LEVERS1-B (PORT-SIM); PLAYTEST2 and CAMP1-P's recording (PORT-PLAYTEST).
- SPRITE1-A2, -L, -U: the spec names Godot's AssetRegistry; it needs an engine strip format first (frames and
  durations next to the PNGs, read by `aegis_gui/src/art.rs`), so SPRITE1-A2 waits for that spec update (design).
- Rows whose engine screen already shows part of what they ask (the builder checks before building): MAPLOG1-U (the
  region map has an event log), HEAL1-U (the hub shows party HP; the world map has Make camp).
- Godot parity: LEVERS1-L1 adds a new field used only by new data, so the traces stay valid; PHASEFOLD and
  LEVERS1-A1 change shipped encounters or skills, so they retire the parity checks they break (decision "Godot parity
  as a migration check (user 2026-10-19)").

## 5. Done rows archived

BAL3-R0, BAL3-S, BAL3-AI, BAL3-A, BAL3-R, CAMP1-B, BAL4-ATTR, MAPREAD1, RECRUIT1, ITEMUSE1, INTENT1, BOSS1, OBJ1,
RECRUIT1-B, BOSS1-B, OBJ1-B move unchanged to `docs/archive/TASKS_DONE.md`.

## 6. Suggested order (for the user to choose)

1. **Restore what the Godot build had** (the gaps of section 1, smallest first): PORT-INTENT, PORT-SETTINGS,
   PORT-BREPORT, PORT-AUDIO, PORT-FOCUS, then PORT-SIM (unblocks the balance lane), PORT-CHECKS and PORT-PLAYTEST.
2. **Data rows that are ready**, in parallel on the asset lane: PACE1-A, PURSE1-A (not alongside each other or
   ASHPACE1, as their rows say).
3. **The approved UI rows** on the engine screens, then the LEVERS chain (L1, L3, A1, A3, then L2 and U once
   PORT-BREPORT lands), then the balance runs once PORT-SIM lands.
4. **Design specs** (KEEP1, DIFF1, ...) whenever the design lane is free; they do not block anything above.

Answered (user 2026-10-19): all PORT rows approved, CLN-17 / CLN-18 closed as obsolete and CLN-13 as done, the order above as recommended (decision workflow.md "RESCOPE1 answers (user 2026-10-19)").
change them.
