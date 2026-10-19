# Agent map (planner-maintained dashboard)

The harness agent list cannot be regrouped, so this file is the readable view: active runs on top, then finished runs grouped by milestone with the most recently completed milestone first. The planner updates it when it dispatches an agent and when one reports. Tokens and minutes are from each agent's own report. Commit hashes are in `git log`.

## Active now

Desktop: the planner keeps this section current. Cloud: the project's area-thread list is the live view (one standing thread per area, `docs/workflow/cloud.md` "Area threads"), so this section changes only at a handover between environments (`docs/workflow/README.md`); merged cloud tasks still get a finished-runs line.

No agents running (2026-10-19, desktop): the transition to the Aegis engine (steps 1-3) was done in one desktop engine session without lane agents; dispatch mode **interactive** (planner-ops). Before it: dispatch mode: **standard**. No agents running (wind-down 2026-10-17 for the move to a cloud instance).

Merged 2026-10-17: SKILL1, BAL1, BAL2-L (run 1 + 1b), BAL2-A, cleanup batches 1-2, CLN-1/2/4/5/10/11/15, AI2 (diagnosis), PLAN1, M4-L + M4-A, QUEST2, EXP1-L (commits 1-3) + EXP1-A, ITEM1-L (commits 1-3) + ITEM1-A, CLASS1-L + CLASS1-M, DLG1-L (commits 1-2) + DLG1-A, S14-L1 (commits 1-2). Open next: S14-L1 commit 3, S14-L2/L3, TRAV1, REP1, UI1-3, S12S/T/C, BAL3 spec (from BAL2-R), CLN-12/13/16, CLN-3b.

## Finished, by milestone (newest first)

### SPLIT1: a game-agnostic engine and the first game, Emberwake (2026-10-19, desktop)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| main session (no lane agents) | SPLIT1-1 | - | Cargo workspace at the repo root (7a8903f) |
| main session (no lane agents) | SPLIT1-2 | - | engine core `aegis_core`: RNG, hash, sort, Variant helpers, field readers |
| main session (no lane agents) | SPLIT1-4 | - | engine save framework `aegis_save`: slots, envelope, atomic write, migration runner |
| main session (no lane agents) | SPLIT1-3 | - | engine content layer `aegis_content`: file format, schema, world graph, writer over game hooks |
| main session (no lane agents) | SPLIT1-5 | - | engine UI `aegis_ui`: drawing, widget kit, art, tuning, agent link, screen stack, window shell |
| main session (no lane agents) | SPLIT1-6 | - | engine protocol `aegis_host` and shell `aegis_cli`; game command set and `aegis` binary in `emberwake_host` |
| main session (no lane agents) | SPLIT1-7 | - | game crates renamed `emberwake_*` under `games/emberwake/crates/`, `emberwake_content` carved out, GUI binary `emberwake` |
| main session (no lane agents) | SPLIT1-8 | - | `games/emberwake/` holds data, assets, schema, tuning and parity; `game.json` (title read by the window and main menu); saves and settings under the game id |
| main session (no lane agents) | SPLIT1-9 | - | Layer column in TASKS, layer check in the change-reviewer, STRUCTURE and CLAUDE.md by layer, the game's design docs in `games/emberwake/docs/` |
| main session (no lane agents) | ARENA-EXIT | - | Exit button in Test Arena battles |
| main session (no lane agents) | PARTY2-B | - | the character popup's detail pane on every stat, derived and level-up row |
| main session (no lane agents) | DOJO2-B | - | gear dojo: skills (with ranks) and pouch slots listed; arrows checked |
| main session (no lane agents) | UIKIT1 | - | engine: select with its choice list, draggable slider, info-row hover, agent `drag`; game: settings, Scene Runner, World Forge and turn-in use them |
| main session (no lane agents) | PORT-SETTINGS | - | engine owns and applies window mode, size, vsync, frame cap and text size; game: settings screen, Character popup and Settings tabs fit the text size |
| main session (no lane agents) | PORT-AUDIO | - | engine: aegis_ui::audio on rodio (buses, voice pools, crossfaded looping music, volume previews), Screen::music, levels from the settings; game: MUSIC1 chooser per screen, battle sounds from the resolver log, settings volume samples |
| main session (no lane agents) | ASSETORG1 | - | engine: art loader reads `<category>/<id>/<variant>.png` (get, token, portrait, tile); game: 296 PNGs moved to a folder per thing, call sites, layout test; tools: gen_art.py, export_units.py; READMEs and specs note the layout |
| main session (no lane agents) | UIKIT2 | - | engine: drag and drop in the kit (draggable payloads, drop targets, can_drop and accept_drop, ghost and outlines, click on release without a move, Esc cancels, agent drag onto a widget id); 5 shell tests |
| main session (no lane agents) | SKILLUI1 spec | design-brainstorm | Skills tab spec (games/emberwake/docs/specs/SKILLUI1.md): chosen rank per slot, drag list to slot, slot to slot and back, rank arrows; four questions answered by the user |
| main session (no lane agents) | GEARUI1 | - | game: Gear tab in three columns, drag and drop both ways (stash, slots, pouch), green and red stat change for a hovered or dragged item, ItemText.render (item description tokens); folds in UIR-3B |
| main session (no lane agents) | GEAR2H1 | - | game: two-handed weapon and the off hand (dojo rule) in validation, member builds and the Gear tab |
| main session (no lane agents) | UIKIT3 | - | engine: windows in the UI kit (desk, move, resize, raise, close, collapse, layouts, saved per screen, clipping, agent commands); 8 tests |
| main session (no lane agents) | PORT-DOJO | - | engine: scroll areas, nested clips, rich text; game: DojoSession (strikes, skills, dummy, rounds, stances, pouch, benchmark, gear, presets, weapon switch, allocation), PartyCarry, damage explanation, the dojo desk of windows; 12 session tests |

### Transition to the Aegis engine (2026-10-19, desktop engine session)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| main session (no lane agents) | engine battle items, data loader, transition steps 1-3 | - | `legacy/` + branch `legacy` (7fa10da), engine merged into `main` (e3f4d71), process docs switched |

### COMBAT-UI3-F: drafts fill empty blocks, hostile HP beside name, compact strip labels (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | COMBAT-UI3-F | 90k / 22 | fill, hostile HP, strip labels |
| change-reviewer | COMBAT-UI3-F | 25k / 1 | clean, 4 nits |

### PURSE1-L: encounter purses pay gold and loot on a win (save v10) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | PURSE1-L | 82k / 33 | runtime, save v10, gate green |
| change-reviewer | PURSE1-L | 24k / 1 | clean, 3 nits |

### COMBAT-UI3: combat HUD layout A (planning strip, merged enemy card, hero panel) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | COMBAT-UI3 | ~450k / 49 | built layout A |
| change-reviewer | COMBAT-UI3 | 29k / 1 | clean, Touches nit fixed |

### MORALE1-B: morale balance probe report (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | MORALE1-B | 56k / 47 min | report landed |
| change-reviewer | MORALE1-B | 17k / 1 min | 1 minor (fixed) |

### HANDOVER-ASSETS: Assets & Audio wind-down handover (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Assets & Audio thread, HANDOVER-ASSETS | HANDOVER-ASSETS | none / - | done |

### PURSE1-L (schema): encounter purse field and validation (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | PURSE1-L (schema) | 81k / 26 | schema slice, gate green |
| change-reviewer | PURSE1-L (schema) | 18k / 1 | clean, 2 nits |

### PACE1-L: two-act boss behaviour (or_round, enrage, empower, enraged intents, start buffs, boss warning) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | PACE1-L | 122k / 31 | behaviour, gate green |
| change-reviewer | PACE1-L | 32k / 1 | clean, 4 nits |

### MORALE1-A: enemy morale on ash_scouts and ash_raiders (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | MORALE1-A | 19k / 1 | seed ok |
| change-reviewer | MORALE1-A | 16k / 1 | clean |

### UIR-6: dialogue as a full-width bottom bar, shop as a popup, text scale (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | UIR-6 | 196k / 31 | built + review fixes |
| change-reviewer | UIR-6 | 79k / 3 | 2 should-fix, 6 nits |

### BAL5-R (probe): why shield_wall's fortify moved nothing (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BAL5-R probe | 60k / 15 | grant works; shield_wall rarely cast |

### PACE1-L (schema): phase enrage and or_round, empower status, pace and start_status fields (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | PACE1-L (schema) | 165k / 39 | schema slice, gate green |
| change-reviewer | PACE1-L (schema) | 62k / 2 | clean, 5 nits |

### BAL5-R: final trio rerun; shield_wall fortify moved nothing, gaps stand (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | BAL5-R | 43k / 50 | gaps stand; report corrected (claimed all cells identical) |

### MORALE1-S: world sims carry the five morale fields (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MORALE1-S | 36k / 6 | done, gate auto balance 44/0 |
| change-reviewer | MORALE1-S | 29k / 1 | clean, 4 nits |

### MAPDEV1-U: map debug tools only in debug scenes; play bars show place and supplies (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MAPDEV1-U | 213k / 98 | built region + world map, docs, gate clean |
| change-reviewer | MAPDEV1-U | 53k / 2 | clean, 3 nits |

### MORALE1-L: enemy morale in battle (losses, breaks, flee bonus, surrender, Fearless) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MORALE1-L | 149k / 37 | behaviour + review fixes, gate clean |
| change-reviewer | MORALE1-L | 57k / 2 | 0 high, 2 medium, 2 nits |

### BAL5-A: shield_wall fortify extra grant and potion price 30 (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | BAL5-A | 27k / 13 | done, gate auto 951/0 |
| change-reviewer | BAL5-A | 18k / 1 | clean, 4 nits |

### BAL5-L: extra_grants skill field (second grant on DEFEND/SUPPORT skills) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BAL5-L | 96k / 41 | done, gate auto 950/0 |
| change-reviewer | BAL5-L | 40k / 1 | clean, 3 nits |

### BAL5-S2 + POTLEDGER: support_attrition scenario and potion heals in the sim ledger (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BAL5-S2 + POTLEDGER | 100k / 25 | done (STATCRED part dropped, duplicate of 3fa4cea) |
| change-reviewer | BAL5-S2 + POTLEDGER | 30k / 1 | clean, 5 nits |

### STATCRED1-L: battle report credits each status to the unit that applied it (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | STATCRED1-L | 71k / 40 | fixed + 2 follow-ups, gate clean |
| change-reviewer | STATCRED1-L | 35k / 1 | 0 high, 1 medium, 3 nits |

### UIR-3: Character popup with Stats, Level up, Skills and Gear tabs (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | UIR-3 | 382k / 61 | built, review fixes, gate clean |
| change-reviewer | UIR-3 | 119k / 12 | 3 highs, 1 medium, 6 nits |

### BAL5-R1b: re-measure under the re-draft fix and BAL5-AI (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | BAL5-R1b | ~50k / 62 | T3, S2, potion price fire; B3 no |

### GATE-SPLIT: per-system gates, parallel CI jobs per system (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | GATE-SPLIT | 30k / 1 | PASS |

### MORALE1-L (schema): enemy morale encounter fields, validation and preview (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MORALE1-L (schema) | 82k / 22 | schema slice, gate clean |
| change-reviewer | MORALE1-L (schema) | 39k / 2 | clean, 3 nits |

### ASSETS-PREFS: sprite animation notes in the world asset preferences (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Assets & Audio thread, ASSETS-PREFS | ASSETS-PREFS | none / - | done |

### CAMPLABEL1-S: campaign sim fall label names the fight actually lost (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | CAMPLABEL1-S | 17k / 1 | clean, 3 nits |

### USAGE: 2026-10-18 11:45 usage report, delta and efficiency (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Platform & Workflow thread, usage report | USAGE | none / - | done |

### BAL5-AI: early taunt with walk-in, healer self-care, AoE friendly-fire tolerance (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BAL5-AI | 164k / 44 | done, arena 924/0 |
| change-reviewer | BAL5-AI | 51k / 1 | clean, 4 nits |

### UIR-2: game menu popup over the paused map (Party, Journal, Save, Load, Settings, Quit), new game, save/load fixes (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | UIR-2 | 424k / 54 | built + review fixes, gate clean |
| change-reviewer | UIR-2 | 70k / 2 | 0 high, 3 medium, 4 nits |

### HEAL1-B: campaign check passes (arms 46, gate 95) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | HEAL1-B | 35k / 29 | pass: arms 46, gate 95 |

### MAPKEYS1-U: WASD, held-key walking, smooth map movement, label fix (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MAPKEYS1-U | 116k / 58 | built, review fixes, gate clean |
| change-reviewer | MAPKEYS1-U | 50k / 2 | 2 mediums, 5 lows |

### BAL5-R1: trio re-measure; T2, S1 and A2 triggers fire (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | BAL5-R1 | 48k / 124 | T2, S1, A2 fire; B3 no |

### RUNNER-WINDOW: fullscreen toggle, minimize and exit in the Scene Runner (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Platform & Workflow thread, RUNNER-WINDOW | RUNNER-WINDOW | review:19k / - | done |

### BREPORT1-L: battle report data (shared BattleMetrics collector, combo signal, BattleReport, results-model API) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BREPORT1-L | 221k / 31 | built, gate clean |
| change-reviewer | BREPORT1-L | 74k / 2 | 0 high, 2 medium, 6 nits |

### BOSS1-U: boss phase banner in combat (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BOSS1-U | 67k / 82 | built, gate clean |
| change-reviewer | BOSS1-U | 28k / 1 | clean, 2 lows |

### PILL-GAME: no Scene Runner pill over the real game (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | PILL-GAME | ~20k / 9 | done |
| change-reviewer | PILL-GAME | ~14k / 1 | pass |

### HEAL1-S: campaign sim keys heal=rally, rest=, extra= (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | HEAL1-S | 49k / 11 | done, arena 872/0 |
| change-reviewer | HEAL1-S | 30k / 1 | clean, 3 nits |

### HEAL1-L: between-battle healing rules (stored HP, rally, rest places, Make camp, defeat cost) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | HEAL1-L | ~184k / 99 | schema slice + rules + 2 review fixes |
| change-reviewer | HEAL1-L | ~80k / 23 | schema clean; rules 2 medium (fixed) |

### BAL5-SIM + BAL5-PH: trio sim tooling and generated boss phase (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | BAL5-SIM + BAL5-PH | 67k / 1 | clean, 4 nits |

### HEAL1-A: rest places at warden_keep and slag_gate (message for 85ac473) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | HEAL1-A | 12k / 1 | clean |

### INTENT1-U: enemy intent chips, arrows and tell lines in combat planning (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | INTENT1-U | ~210k / 55 | built, gate clean, review fixes |
| change-reviewer | INTENT1-U | ~51k / 1 | 1 medium, 4 lows |

### ITEMUSE1-A2: remove the free class pouch potions (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | ITEMUSE1-A2 | ~16k / 11 min | 4 class pouches emptied, seed |
| change-reviewer | ITEMUSE1-A2 | ~28k / 1 min | clean, 2 logic-lane follow-ups |

### UIR-1: shared screen frame, button width classes, selected style, No focus on destructive questions (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | UIR-1 | ~70k / 32 | ScreenFrame, width classes, selected style, destructive No focus |
| change-reviewer | UIR-1 | ~40k / 11 | 0 high, 1 medium (to UIR-2), 3 low |

### CLN-3b: drop legacy skill keys and version 1 skills loading (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| CLN-3b (Platform & Workflow): legacy skill keys and v1 skills loading removed; change-reviewer | CLN-3b | ~31k / - | done |

### HEAL1 (spec approved): defeat cost per the user, decision row, build rows approved (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | HEAL1 spec + defeat rule update | ~160k / ~12 | spec approved with the user's defeat change |

### HEAL1 (spec): between-battle healing spec from BAL4-ATTR v4, TASKS rows, waiting on the user (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | HEAL1 spec | ~ / ~12 | spec written, 5 open questions |

### RECRUIT1-B, OBJ1-B, BOSS1-B: world-encounter probes; BAL4-ATTR v4 (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran logic-builder (sonnet, medium) for the sim path, balance (sonnet, low) for the runs, change-reviewer (low) | RECRUIT1-B, OBJ1-B, BOSS1-B | ~180k / - | done |

### EXPWORLD: a child world survives a trip through a connected world (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | EXPWORLD | ~35k / 16 | home world kept across connects crossings |
| change-reviewer | EXPWORLD | ~14k / 2 | 0 blockers, 3 minors (1 fixed) |

### QUESTX: quests keep tracking across connected worlds (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | QUESTX | ~69k / 26 | cross-world quest scope + review fix |
| change-reviewer | QUESTX | ~26k / 1 | 1 high (fixed), 3 nits |

### ITEMUSE1-U: battle pouch section on the Gear screen (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | ITEMUSE1-U | ~47k / 3 | pouch section on the Gear screen, party suite 6 cases pass |
| change-reviewer | ITEMUSE1-U | ~22k / 1 | clean, 4 nits |

### COMBAT-UI2-L3: tactic dropdowns and Autoplay toggle in the timeline row (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | COMBAT-UI2-L3 | ~250k / 28 | dropdowns, Autoplay toggle, 14 px row, replay fix; arena 821/0 |
| change-reviewer | COMBAT-UI2-L3 | ~29k / 1 | 1 high, 1 medium, 2 nits (all fixed) |

### ITEMUSE1-L: use pouch items in battle as a planner step (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | ITEMUSE1-L | ~184k / 57 | item command, planning, undo, write-back, AI/sim, recordings |
| change-reviewer | ITEMUSE1-L | ~52k / 1 | 1 medium (pending user), 2 nits |

### COMBAT-UI2-L2: unit hover card replaces the combat inspector (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | COMBAT-UI2-L2 | ~167k / 32 | hover card built, arena 794/0 |
| change-reviewer | COMBAT-UI2-L2 | ~17k / 1 | clean, 2 nits |

### BAL4-ATTR: campaign attrition sim and report (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran logic-builder (sonnet, medium) for the sim mode, balance (sonnet, low) for the runs, change-reviewer (low) | BAL4-ATTR | ~210k / - | done |

### ITEMUSE1-A: use_reach on the potion, focus elixir and fire bomb (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | ITEMUSE1-A | ~19k / 11 min | 3 use_reach rows, seed |
| change-reviewer | ITEMUSE1-A | ~13k / 1 min | clean, 1 nit |

### ITEMUSE1-L (schema): hero battle pouch, save v8, use_reach, item plan step (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | ITEMUSE1-L (schema) | ~131k / 30 | pouch, save v8, use_reach, ITEM step shape |
| change-reviewer | ITEMUSE1-L (schema) | ~35k / 1 | clean, 2 mediums accepted |

### ART2-A: Ashlands placeholder art pass 2 (intent and objective chips, ring tile, miss sound) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | ART2-A | ~90k / 25 min | 17 PNGs, miss.wav, generator group, seed |
| change-reviewer | ART2-A | ~14k / 1 min | clean, 1 nit |

### OBJ1-A: courtyard hold, wagon raid with Wenn Tallow, slay Grakka (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | OBJ1-A | ~31k / 1 | objectives data, wagon_raid, Wenn, text pass via seed |
| logic-builder | OBJ1-A | ~17k / 0.6 | Wenn Tallow CHARACTERS row in gen_art.py, art and manifest |
| change-reviewer | OBJ1-A | ~21k / 0.5 | 1 blocker (pyc) fixed |
| asset-builder | OBJ1-A | ~27k / 4.7 | headless acceptance check (scratch, no repo change) |

### COMBAT-UI2-L1: combat layout C (compact columns) plus UIR-5 leftovers (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet) | COMBAT-UI2-L1 | ~507k / 52 | layout C, cards, ticker, tile rule, gear path removed, party cap |
| change-reviewer | COMBAT-UI2-L1 | ~19k / 1 | clean, 1 flag (fixed) |

### OBJ1-L: battle objectives runtime, deployments, guests and verdict (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | OBJ1-L | ~197k / 40 | runtime, deployments, guests, verdict + 2 review fixes |
| change-reviewer | OBJ1-L | ~50k / 1 | 2 mediums (fixed), 3 nits |

### BOSS1-A: Crown of Nails phase on Grakka's stands (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | BOSS1-A | ~23k / 1 | phases on both Grakka stands via seed |
| change-reviewer | BOSS1-A | ~23k / 0.4 | clean, 2 nits |

### INTENT1-A: Warden's Eye passive on Bryn (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | INTENT1-A | ~23k / 1 | wardens_eye passive and bryn_hale grant via seed; origin fix by thread |
| change-reviewer | INTENT1-A | ~14k / 0.2 | clean, 1 nit |

### MAPREAD1-L2: map marker icons, labels, legend, badges, world pins and hover (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet) | MAPREAD1-L2 | ~363k / 65 | markers, labels, legend, world pins and hover, review fixes |
| change-reviewer | MAPREAD1-L2 | ~26k / 1 | clean, 3 nits (fixed) |

### RECRUIT1-A: Bryn and five story recruits, Toller muster, priest/brawler units, trio sizing (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | RECRUIT1-A | ~56k / 16 min | characters, muster, units, sizing, art |
| logic-builder | RECRUIT1-A tests | ~36k / 8 min | three tests derived from data |
| change-reviewer | RECRUIT1-A | ~30k / 1 min | clean, 3 nits |

### OBJ1-L (schema): encounter objective, deployment and guest fields (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | OBJ1-L (schema) | ~109k / 19 | objective schema + validation + preview |
| change-reviewer | OBJ1-L (schema) | ~36k / 1 | clean, 6 nits |

### BAL3-R + CAMP1-B: final BAL3 rerun, band proposal, Ashmaw finale sim (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran balance (sonnet, medium) for BAL3-R and CAMP1-B, change-reviewer (low) | BAL3-R + CAMP1-B | ~136k / - | done |

### BOSS1-L: boss phases fire in battle (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BOSS1-L | ~150k / 17 | phase runtime, 8 test groups |
| change-reviewer | BOSS1-L | ~34k / 2 | clean, 3 nits |

### MAPREAD1-A: Ashlands marker labels, world-map pins, 48x30 maps (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | MAPREAD1-A | ~27k / 8 min | labels, map_pos, map switch |
| change-reviewer | MAPREAD1-A | ~20k / 1 min | clean, 2 nits |

### BAL5 (spec): lever options for the BAL3-R balance gaps, TASKS rows, review candidates (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | BAL5 lever spec | ~113k / ~7 | spec written, 6 open questions |

### ART1-A: Ashlands placeholder art (characters, classes, icons, tiles, maps, backdrops) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | ART1-A | ~350k / 45 min | ~260 PNGs, generator, seed, 4 map templates |
| logic-builder | ART1-A tests | ~65k / 28 min | world + asset test fixes and new resolve checks |
| change-reviewer | ART1-A | ~23k / 1 min | 2 high, 3 medium, 3 nits |

### BOSS1-L (schema): encounter phases field and validation (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | BOSS1-L (schema) | ~90k / 13 | phases schema + validation |
| change-reviewer | BOSS1-L (schema) | ~38k / 1 | clean, 3 nits |

### MAPVIEW1: full-screen region and world maps (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MAPVIEW1 | ~144k / 34 | full-screen maps, camera, backdrop, map_pos pins, tests |
| change-reviewer | MAPVIEW1 | ~26k / 1 | clean, 2 nits |

### MAPREAD1-L1: marker labels and notes, marker_view, location map_pos, world-map node extras (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | MAPREAD1-L1 | ~95k / 27 | marker_view, map_pos, node extras; 1 review fix |
| change-reviewer | MAPREAD1-L1 | ~27k / 1 | 1 high fixed, 1 rejected, 3 nits |

### INTENT1-L: enemy intent read-out and the foresight passive effect (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | INTENT1-L | ~120k / 15 | intents + foresight, 7 checks |
| change-reviewer | INTENT1-L | ~20k / 1 | clean, 2 nits |

### OPTLAUNCH1: run.bat opt opens a design option branch in its own folder (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | OPTLAUNCH1 | ~34k / 12 | done, review fix applied |
| change-reviewer | OPTLAUNCH1 | ~19k / 1 | pass, 3 minor |

### BREPORT1 (spec): battle report spec and TASKS rows, waiting on the user (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | BREPORT1 spec | ~ / ~12 | spec written, 4 open questions |

### RECRUIT1-L: recruiting, hero names and the runtime party cap (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | RECRUIT1-L | ~75k / 12 | behaviour + recruit suite |
| change-reviewer | RECRUIT1-L | ~39k / 1 | clean, 3 nits |

### CONTROLS-HELP: Controls tab, Settings during play, volume fix and Mute (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | CONTROLS-HELP: Controls tab, ControlsList, in-game Settings, volume fix, Mute, tests, renders | ~159k / 57 | gate green |
| change-reviewer (low) | CONTROLS-HELP | ~18k / 4 | clean |

### RECRUIT1-L (schema): manifest party_max, party characters in start_party, recruit effect validation (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | RECRUIT1-L (schema) | ~88k / 22 | schema + validation, 3 checks |
| change-reviewer | RECRUIT1-L (schema) | ~17k / 3 | clean, 3 nits |

### MUSIC2-A + MUSIC2-L: town, danger, shop, travel and boss music, wired by location kind (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | MUSIC2-A | ~15k / 1 | five manifest rows, loop=true imports |
| logic-builder | MUSIC2-L | ~44k / 13 | kind table, shop override, tests |
| change-reviewer | MUSIC2 | ~37k / 1 | 0 blockers, 2 mediums handled |

### WORLDMAP-ENTER: click the party's own pin on the world map to enter its location (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | WORLDMAP-ENTER | ~23k / 6 | fixed, check added |
| change-reviewer | WORLDMAP-ENTER | ~13k / 1 | clean, 3 nits |

### SOUND-PREVIEW: volume sliders play a sample at the new level (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | SOUND-PREVIEW: slider samples, throttle, revert, tests | ~96k / 57 | gate green |
| change-reviewer (low) | SOUND-PREVIEW | ~16k / 3 | 1 high fixed |

### MUSIC1-A + MUSIC1-L: five generated music loops and a context-based music player (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | MUSIC1-A | ~15k / 3 | manifest rows + seed, loop=true imports |
| logic-builder | MUSIC1-L | ~140k / 20 | MusicDirector, contexts, hooks, audio tests |
| change-reviewer | MUSIC1 | ~65k / 3 | clean, 1 medium accepted |

### POI-FIX: region map shows how to use chests, people and lore, and a click works too (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | POI-FIX | ~139k / 29 | done |
| change-reviewer | POI-FIX | ~20k / 1 | clean |

### AUTO1-L (logic): battle autoplay loop, party and hero tactics, safety stops, save v7 (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | AUTO1-L (logic) | ~166k / 27 | done |
| change-reviewer | AUTO1-L (logic) | ~35k / 1 | clean |

### NOTE-BUTTON: "✎ Note" in the in-scene pill opens the playtest note box (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | NOTE-BUTTON | ~30k / 8 | done |
| change-reviewer | NOTE-BUTTON | ~14k / 1 | pass, 2 minor |

### BAL3-A: data levers (sweep_kick), elite power 1.0, sim tank loadout (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran asset-builder (sonnet, medium) for BAL3-A, logic-builder (sonnet, medium) for the code half, change-reviewer (low) | BAL3-A | ~150k / - | done |

### BATTLE-FLOW: combat transition, victory popup with XP bars, rewards, minimise and replay; COMBAT-UI2 approved (layout C) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | BATTLE-FLOW: intro transition, results popup, xp_bar, replay, guards, tests, renders | ~150k / 60 | gate green |
| change-reviewer (low) | BATTLE-FLOW | ~25k / 6 | 2 medium fixed |

### TRAV2-A: Ashlands supply sources and start supplies (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | TRAV2-A | ~48k / 1.5 | data built, checks 1-4 pass |
| change-reviewer | TRAV2-A | ~27k / 3.5 | clean, README delta |

### TRAV2-L: travel supplies are spent on the road, bought in shops and found in chests (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | TRAV2-L (behaviour) | ~75k / 25 | done |
| change-reviewer | TRAV2-L (behaviour) | ~26k / 1 | clean |

### AUTO1 (spec): battle autoplay with a tactic dropdown per timeline row (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | AUTO1 (spec) | 22k / 0.3 | clean, 3 nits |

### CAMP1-A1 + CAMP1-A2: Ashlands campaign "The Long Dry" (boss unit and story data) (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | CAMP1-A1 + CAMP1-A2 | ~146k / 23 | data built, checks 1-6 pass |
| change-reviewer | CAMP1-A1 + CAMP1-A2 | ~47k / 1.5 | clean, README high fixed |

### SETTINGS-POPUP: Settings as a tabbed popup, window changes on select, right-click back fixed (2026-10-18, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | SETTINGS-POPUP: popup panel, tabs, select-to-apply, right-click via _input, tests, renders | ~126k / 54 | gate green |
| design-brainstorm (medium) | UI-REVIEW1: review of every menu and HUD screen, options Q1-Q6, task split | 78k / 33 | approved by user |
| change-reviewer (low) | SETTINGS-POPUP + UI-REVIEW1 docs | 24k / 5 | clean |

### TRAV2-L (schema): travel supplies field, save v6, shop and chest supply fields (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | TRAV2 specs (TRAV2_L, TRAV2_A) | ~111k / 5 | done |
| logic-builder | TRAV2-L (schema) | ~93k / 8 | done |
| change-reviewer | TRAV2-L (schema) | ~32k / 1 | clean |

### CAMP1: record the user's approval of the Ashlands story spec (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | CAMP1 approval | 17k / 1 | clean |

### CAMP1 (spec): Ashlands campaign story "The Long Dry" (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | CAMP1 | 135k / 9 | spec written |
| change-reviewer | CAMP1 | 25k / 1 | 0 high, 2 nits fixed |

### BAL3-AI: AI pressure levers and minion skill ceiling (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran logic-builder (high) for BAL3-AI plus its review fixes, change-reviewer (medium) | BAL3-AI | ~330k / - | done |

### FORGE-VIEW: display-only Content Forge with an in-game skill preview per rank (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | FORGE-VIEW | ~259k / 42 | done |
| change-reviewer | FORGE-VIEW | ~22k / 1 | pass, 2 low nits |

### MENU-POLISH + DISPLAY1-FIX: menu layout and sliders; DPI-scaled window sizes (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | MENU-POLISH + DISPLAY1-FIX: menu layout, sliders, restore defaults, right-click back, DPI-scaled sizes, In use row, ui_trail fix | ~122k / 76 | gate green |
| change-reviewer (low) | MENU-POLISH + DISPLAY1-FIX | 26k / 5 | 1 medium fixed |

### REP1-L: reputation runtime, shop bands, panels, world warnings (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (opus, medium) | REP1-L commit 2 (runtime) + commit-1 review lows | ~221k / 11 | done, gate green |
| change-reviewer (sonnet, low) | REP1-L behaviour review | ~50k / 1 | PASS, 4 optional lows |

### REP1-A: reputation deltas, Hest spare stores, ember_patrol quest (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | REP1-A | 24599 / 6 | seeded quest reputation, Hest choice and ember_patrol; check 1 passed |
| change-reviewer | REP1-A | 21010 / 1 | clean, 1 style nit |

### REP1-L (schema): condition operators, rep flags, add_flag, quest reputation field (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (opus, medium) | REP1-L commit 1 (schema) | ~140k / 16 | done, gate green |
| change-reviewer (sonnet, low) | REP1-L schema review | ~28k / 1 | clean, 2 lows deferred |

### TRAV1-A: Ashlands and Iron Reaches roads, world-map exits (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | TRAV1-A | 42215 / 14 | seeded 3 routes and 2 world-map exits, acceptance 1-6 checked |
| change-reviewer | TRAV1-A | 23314 / 1 | 1 medium (README blank lines, fixed) |

### DISPLAY1: display settings (window mode, window size, vsync, frame-rate cap; web Fullscreen) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | DISPLAY1: display settings, backend, countdown, web fullscreen, display suite, renders | ~92k / 43 | gate green |
| change-reviewer (low) | DISPLAY1 | 24k / 3 | 1 minor fixed |

### PULL-RESTART: Restart runner pulls the latest code for its folder, then restarts (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | PULL-RESTART | ~30k / 18 | done, review fix applied |
| change-reviewer | PULL-RESTART | ~17k / 1 | pass, 2 minor |

### SAVE-COMPAT: sample saves for every save version, loaded by the gate (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | SAVE-COMPAT fixtures, suite, generator + review fix | ~46k / 19 | done, gate green |
| change-reviewer (sonnet, low) | SAVE-COMPAT branch review | ~24k / 1 | PASS, 2 lows |

### S12C-A: first combo skills (feint, crushing_blow, opportunist) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | S12C-A | 28037 / 17 | seeded 3 combo families, acceptance 1-5 checked |
| change-reviewer | S12C-A | 22351 / 1 | clean, 1 low (spec text) |

### SETTINGS-UI: fixed speed and text-size levels, centred settings, real checkbox, bordered menu buttons (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | SETTINGS-UI: fixed levels, centred grid, checkbox, MenuStyle, layout suite, renders | ~106k / 64 | gate green |
| design-brainstorm (medium) | DISPLAY1 spec (window mode, size, vsync, fps cap; web fullscreen) | 27k / 6 | approved by user |
| change-reviewer (low) | SETTINGS-UI + DISPLAY1 docs | 37k / 5 | clean |

### GATE-ONCE: area threads run the full gate once per task (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | GATE-ONCE | ~19k / 1 | pass, 1 fixed, 1 noted |

### SPLIT-LANDING: large logic tasks can land as a schema commit, then a behaviour commit (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | SPLIT-LANDING | ~16k / 1 | 3 medium, 2 low, fixed |

### TRAV1-L: world map, routes, road encounters and the world clock (S10) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (opus, medium) | TRAV1-L: routes schema, travel runtime and save v5, debug world map in 3 commits | ~280k / ~65 | gate green |
| change-reviewer (low) | TRAV1-L whole branch | 22k / 1 | clean |

### BAL3-S: sim levers (start jitter, role scenario scale, encounter archetypes) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran logic-builder (medium) for BAL3-S, change-reviewer (medium), logic-builder (sonnet, medium) for review fixes | BAL3-S | ~213k / - | done |

### PLATFORM-HANDOVER: fresh-session rule for area threads; Platform handover (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | PLATFORM-HANDOVER | ~17k / <1 | 1 medium fixed |

### SAVE-BACKUP: run.bat backs up saves before an update; SAVE-COMPAT queued (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | SAVE-BACKUP | ~15k / <1 | 1 medium fixed |

### DEV-USERDATA: dev game keeps its own saves and settings (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | DEV-USERDATA | ~15k / <1 | clean |

### PULL-ON-LAUNCH: run.bat pulls the latest code before opening the game; run_dev.bat for develop (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | PULL-ON-LAUNCH | ~53k / 2 | 1 blocker + 3 mediums fixed |

### WEB-PAUSE: pause web deploys until a playable build (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | WEB-PAUSE | ~19k / <1 | 2 mediums fixed |

### LOAD-SLOTS-UI: Load / Save game slot list scrolls, Back always on screen (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | LOAD-SLOTS-UI: scrolling slot list, fixed Back, follow_focus, tests, renders | ~45k / 26 | gate green |
| change-reviewer (low) | LOAD-SLOTS-UI | 15k / 5 | 1 medium fixed |

### DEPLOY-AFTER-RELEASE: start the web deploy after each release merge (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | DEPLOY-AFTER-RELEASE | ~19k / <1 | 2 mediums fixed |

### S12C-L: combos (flow moves, stance break, exploit) with previews and AI (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (opus, medium) | S12C-L: combo schema, rules, AI in 3 commits plus chip fix | ~238k / ~57 | gate green, sim unchanged |
| change-reviewer (low) | S12C-L whole branch | 43k / 1 | clean, 3 lows (1 fixed) |

### SIM-IMPORT: fc.sh rebuilds a stale class cache first, loud timeouts, sim startup watchdog (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | SIM-IMPORT | ~26k / 8 | done; per-process sim files added by the thread |
| change-reviewer | SIM-IMPORT | ~16k / <1 | 0 blockers, 4 nits |

### BAL3-R0: baseline balance rerun after the 12x9 grid and S12S-L (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| Balance thread ran balance (sonnet, medium) for BAL3-R0, change-reviewer (medium) | BAL3-R0 | ~126k / - | done |

### S12T-A: 17 terrain rows (move cost, cover, hazards) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder (sonnet, low) | S12T-A: 17 terrain rows via seed | ~23k / ~4 | rows valid, 1 test broke |
| logic-builder (opus, medium) | S12T-A: terrain test independent of shipped rows | ~7k / ~12 | gate green, sim unchanged |
| change-reviewer (low) | S12T-A whole branch | 22k / 1 | clean |

### REVIEW-F5: GameState.of replaces Variant casts, small cleanups (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | REVIEW-F5 | ~70k / 17 | done |
| change-reviewer | REVIEW-F5 | ~19k / 1 | 0 blockers, 1 medium accepted |

### S12T-L: terrain rules (catalog kind, cost-weighted paths, cover, hazards, AI) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (opus, medium) | S12T-L: terrain schema, rules, AI in 3 commits plus review fixes | ~250k / ~57 | gate green, sim unchanged |
| change-reviewer (low) | S12T-L whole branch | 42k / 1 | 1 medium, 3 lows, all fixed |

### REVIEW-F3: fx_played only for applied presets, load_file rename, miss preset (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | REVIEW-F3 remainder: fx_played for applied presets, load_file rename, miss preset | ~50k / 42 | gate green |
| change-reviewer (low) | REVIEW-F3 remainder | 21k / 3 | clean |

### TOOLS-RO: creation tools become read-only viewers; content edit process documented (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | TOOLS-RO | ~294k / 28 | done; lock icons removed on user follow-up |
| change-reviewer | TOOLS-RO | ~30k / 1 | 0 blockers, 2 nits |

### COMBAT-LAYOUT: combat side docks keep their size on enemy select (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | COMBAT-LAYOUT: fixed-width combat docks, wrapped labels, clamped preview, layout test, before/after renders | ~75k / 42 | gate green |
| change-reviewer (low) | COMBAT-LAYOUT whole branch | 21k / 3 | clean |

### REVIEW-F4: pin Butler 15.32.0 with a sha256 check (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | REVIEW-F4 butler pin | ~15k / 2 | done |
| change-reviewer | REVIEW-F4 butler pin | ~14k / <1 | no blockers |

### REVIEW-F2: menu flow review fixes, new-game and level-up confirms (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | REVIEW-F2: menu flow fixes, Q1 autosave ask, Q2 level-up ask | ~84k / 38 | gate green |
| change-reviewer (low) | REVIEW-F2 whole branch | 30k / 5 | clean |

### PC-LAUNCH-CACHE: rebuild Godot's class cache after a pull, before launching (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer | PC-LAUNCH-CACHE | ~12k / <1 | 2 fixes applied |

### REVIEW-F1: save safety (export backup swap, debug map autosave, peek versions, busy) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | REVIEW-F1: save safety findings 1-9 plus review fixes | ~103k / ~13 | gate green |
| change-reviewer (low) | REVIEW-F1 whole branch, then the fix diff | ~58k / ~1 | 2 medium fixed, clean |

### REVIEW-F4 (partial): web CI cancels only PR runs, itch timeout, pack excludes tests and... (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | REVIEW-F4 partial | ~23.5k / ~10 | done; butler pin blocked |
| change-reviewer | REVIEW-F4 partial | ~11k / <1 | no blockers |

### S14-FIX: S14-L2/L3 review follow-ups (FX settings, audio director, presenter) (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder (sonnet, medium) | S14-FIX: headless instant, AudioDirector stack, presenter distance, _live pruning, to_float, startup settings load | ~50k / ~22 | gate green |
| change-reviewer (low) | S14-FIX whole branch | 17k / 1 | clean |

### SHARED-LOGS: only Merge & Release writes AGENT_MAP, TASKS_DONE and TASK_LOG (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (Platform & Workflow thread) + change-reviewer | SHARED-LOGS: only Merge & Release writes the shared logs | 25k / 6 | landed on develop |

### BATTLE1-L battle link (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | BATTLE1-L: map and dialogue fights open the combat arena (BattleScreen), outcomes settle through after_fight, results headline, battle_link suite | n/a / 60 | PR open |

### GRID12 combat grid 12x9 (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | 12x9 grid, left/right deployment, side panels 270 px, centre marker to map POI icon, exits (11, 4); before/after screenshots in project files combat-grid/ | n/a / 60 | PR #31 |

### S12S-A status carriers (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | Five status families on class paths at level 6 (seed s12s.gd); loadouts, tier gates and duel/role sims unchanged | n/a / 30 | merged via PR |

### S14-A2 placeholder sounds (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | 13 generated SFX blips and manifest rows (seed s14_a2.gd) | n/a / 10 | merged via PR |

### Roadmap review (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | Roadmap review (project file roadmap/ROADMAP_REVIEW_2026-10-17.md); ROADMAP statuses refreshed, goals G1-G6 replace the build order, CAMP1 idea row | n/a / 20 | merged via PR |

### AREA-THREADS and BATCH-MERGE: area threads, develop branch, batched releases (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (Platform & Workflow thread) | cloud.md "Area threads", CLAUDE.md, planner and planner-ops skills, README table, decision row; daily routine now dispatches to area threads | n/a | PR #33 squash-merged |
| planner (Platform & Workflow thread) + change-reviewer | cloud.md landing and release sections, CI only on PRs into main (docs-only skipped), planner skills, README table, decision row, `develop` branch | n/a | PR #34 squash-merged; WEB-DAILY (web.yml daily at 19:07 UTC and on demand, PR build only for web export changes) landed on develop |

### S14-L1 commit 3 item art (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) + change-reviewer | ItemArt through AssetRegistry, icon_id validation and Forge picker, shortsword branch removed | n/a / 25 | merged via PR |

### REVIEW-STEP review step and retroactive reviews (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer (x7) | Retroactive static review of the 13 code PRs merged without review (#4, #6, #8, #9, #11, #13, #14, #16, #17, #19, #20, #21, #23) | 459k / 3 | 1 high (TASKS_DONE wipe, fixed in this PR), 7 medium, lows; follow-ups REVIEW-F1..F5 |
| design-brainstorm | games/emberwake/docs/specs/REVIEW_F.md from the review findings, three design questions | 79k / 5 | spec |
| planner (cloud thread) | cloud.md review step and lane-agent rule, decision rows, TASKS_DONE restored | n/a | PR #30 |

### BAL3 balance levers spec (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| cloud thread (no subagent) | games/emberwake/docs/specs/BAL3.md: sim, AI and data levers for I2-I14 with triggers, sequenced after the 12x9 grid and S12S-L | n/a / 25 | approved by the user (all recs) and merged via PR #29 |

### S14-A1 first manifest rows (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Alias and four sample overrides exported and seeded (data/seeds/s14_a1.gd); commit 2 already done by ITEM1-A | n/a / 25 | merged via PR |

### DLG1-F dialogue and shop follow-ups (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Buy count cap, sell/give gold-cap notes, confirm_turn_in pre-check, integer price oracle | n/a / 15 | merged via PR |

### BAL2-R clean balance rerun (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| cloud thread (no subagent) | party, sweep, role, duel, tier at L3/L5 on main d6f716c; BALANCE_REPORT rewritten, old rerun archived as superseded | n/a / 30 | PR squash-merged; 0 PLAN FAIL, 0 METRIC FAIL |

### UI1-L menus and save slots (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | UI3-L: read-only quest journal, journal suite; screenshots 17-19 in project files ui1/ | n/a / 20 | merged via PR |
| planner (cloud thread, no subagent) | SAVE-IO: save export / import through a file dialog, save_io suite | n/a / 15 | merged via PR |
| planner (cloud thread, no subagent) | UI2-L: party, level-up, skills, gear, battle results screens; 5 shared save slots; screenshots 10-16 in project files ui1/ | n/a / 70 | merged via PR |
| planner (cloud thread, no subagent) | UI1-A: ashlands start_party seed (warrior, ranger, rogue, mage) | n/a / 5 | merged via PR |
| planner (cloud thread, no subagent) | start_party, slots 0-3 + peek + autosave, CurrentGame, SettingsFile, MenuRouter and five screen models, main menu scene; screenshots in project files ui1/ | n/a / 60 | merged via PR |

### S14-L3 audio (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Bus layout, SettingsFile audio keys, SoundEvents, SoundBank, AudioDirector, placeholder SFX tool, audio suite | n/a / 25 | merged via PR |

### S14-L2 combat presenter (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | CombatPresenter, FxPresets, FxSettings instant mode, animation lifecycle suite | n/a / 40 | merged via PR |

### Cloud cleanup threads (2026-10-17)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| cloud thread (no subagent) | CLN-12 suite split + CLN-6 ItemCatalog part | n/a | PR #8 squash-merged; PASS lines unchanged |
| cloud thread (no subagent) | CLN-16 remove the `smart_skills = false` AI path, its tests and the sim key | n/a | PR #13 squash-merged |

### S12S-L status expansion (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Commit 1 schema: registry magnitude ranges, six status rows, magnitude validation, sources, StatusExpansion suite | 135k / 16 | PR #15 |
| change-reviewer | Commit 1 review | 63k / 2 | no blockers; popup unit and comment fixed by the thread |
| logic-builder | Commits 2-3: slow, expose/fortify armor, round-end order, granted magnitudes, chips, AI rule 8, sim regen/dot metrics, two new suites | 212k / 28 | arena 472/0, dojo 45/0 |

### DLG1-L commit 3 dialogue and shop panels (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | DialoguePanel, ShopPanel in the region map; shop return without node effects (user decision); DialoguePanelChecks | n/a / 25 | merged via PR |

### S14-F asset pipeline follow-ups (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Registry family cache cleared on content reload, suite .get() lookups and copy check, spec item refs and origin rules | n/a / 10 | merged via PR |

### SKILL1 Forge skill tab (2026-10-17)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | SKILL1-L (first run stopped at usage limit; resumed run 105k / 2) | n/a + 105k / 2 | merged `611b667` |
| asset-builder | SKILL1-A migration seed | 111k / 11 | merged (planner enabled the 7 accepted pairings) |
| change-reviewer | Review | 83k / 3.5 | 1 high (fixture), 2 medium (warnings_all, lossy save) |
| logic-builder | Review fixes | 140k / 14 | merged |

### BAL1 balance rerun (2026-10-16)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | duel L1/3/5/7, tier, levels, smart_skills A/B | 66k / 22 | report committed `80eeaf0`; open issues await user decisions |

### AI1 AI skill use

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | AI use of disengage, areas, cleanse, taunt, buffs, riposte | 205k / 20 (incl. 1 resumed fix) | merged |
| change-reviewer | Review | 36k / 1 | merged |
| design-brainstorm | SK2 unlock-rule spec | 79k / 3 | spec awaiting approval |

### M2C2B class skill content

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | All six kits, C1-A2, D1-A, party units | 92k / 8 | merged |
| logic-builder | Update 9 shipped test groups for the new kit | 75k / 6 | merged |
| change-reviewer | Review | 56k / 1.5 | merged |

### Model trial 1 (reviewer, inherited vs haiku)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer (haiku) | M2C2B-L static review | 70k / 3.5 | 1 test gap + nits |
| change-reviewer (inherited) | M2C2B-L static review | 28.5k / 1 | clean + nits |

### M3 progression and save

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Party state, XP curve, versioned save | 117k / 12 | merged |
| change-reviewer | Review | 34k / 1 | merged (rename fallback hardened) |

### M2C2B-L area statuses and defend taunt

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | X1 area statuses, X2 defend grants | 69k / 11 | merged |

### R1 test split

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Per-group suites, SuiteRegistry, TestHelpers | 56k / 12 | merged |

### Milestone planning (brainstorm in builder idle time)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | Milestone plan v2 (coarse), AI1 and M3 specs | 91k / 4 | docs, awaiting your answers |

### D1 weaken and riposte

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | D1-L weaken, magnitude, zero-power attacks, riposte data | 154k / 23 | merged (D1-A folds into M2C2B assets) |
| change-reviewer | Review of merged commit | 35k / 6 | clean |

### C1 orb and tome

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | C1-A1 gating, priest proficiency, tome retype | 23k / 2 | merged |
| logic-builder | C1-L orb/tome rules, item effects, gear bonuses | 155k / 14 | merged |
| change-reviewer | Review | 42k / 3 | merged (hand-rule gap accepted) |

### B1 rest to Tactical (first parallel pilot)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Rest to Tactical, stance constant | 47k / 10 | merged (no reviewer) |
| design-brainstorm | Specs C1, D1, M2C2B (ran in parallel) | 131k / 7 | docs/specs |

### Slice A foundation

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Basics cleanup, registry, unarmed, hands | 213k / 20 | merged |
| change-reviewer | Review | 51k / 4 | merged |

### M2c-2b design

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | Skill draft v2 | 51k / 3 | docs |
| design-brainstorm | Revision 3 proposals | 47k / 2.5 | docs |
| design-brainstorm | C1, D1, M2C2B specs | 131k / 7 | docs/specs |

### M2c-2a2 engine pieces

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Cleanse, disengage, sweep, unarmed budget | 188k / 13 | merged |
| change-reviewer | Review | 51k / 3 | merged |

### M2c-2a1 skill families

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Families, tiers, loadout | 126k / 7 | merged |
| change-reviewer | Review | 37k / 2 | found rename hazard (fixed) |

### M2c-1 classes

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Six-class roster data | 109k / 12 | merged |
| design-brainstorm | Skill draft v1 | 41k / 2 | docs |
| change-reviewer | Review | 29k / 2 | merged |

### M2b items and art

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Eight items, placeholder art keys | 68k / 4 | merged (no reviewer) |

### M2d gear budget

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | GearBudget guard | 66k / 8 | merged |
| change-reviewer | Review | 24k / 2 | merged |

### M2a riposte

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Riposte engine | 128k / 9 | merged |
| change-reviewer | Review | 33k / 2 | merged |

### M1 combat extensions

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Support skills, gap-close, haste/bleed/taunt, AI fixes | 300k / 23 | merged |
| change-reviewer | Review | 30k / 2.5 | found stale reset bug (fixed) |

### Process and roadmap

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm (general-purpose) | Roadmap proposals | 56k / 1.5 | Proposed designs in ROADMAP |

## Cost tracking (started 2026-10-16)

What can be measured:

- Subagents: each report gives total tokens, tool calls and minutes (tables above). Before 2026-10-16 every agent ran on the model the main session used (Sonnet 5.5): no agent definition sets a model, so they inherit it. From now on the Model column records the model passed on each dispatch (`sonnet`, `opus`, `haiku`, `fable` or "inherited").
- Main session (planner): I can read a remaining-token counter after each tool call. The delta between two points is logged below per milestone. It is an approximate indicator (it is not split into input, output and cached tokens, and it does not include subagent usage).
- Money: no prices are stored here. If you give a $/million-tokens figure per model, the planner can add a cost column.

| Point | Counter value | Delta since previous | Note |
| --- | --- | --- | --- |
| Baseline (R1, M2C2B-L, M3 running) | about 14,988,000 | - | start of tracking |

Model comparison plan (to pick the right model per task; change one thing at a time): run the same agent type on another model for a comparable task and compare tokens, minutes and defects found.

| Agent type | Suggested trial | What to compare |
| --- | --- | --- |
| change-reviewer | haiku vs inherited | defects found on the same diff, tokens, minutes |
| asset-builder | haiku vs inherited | validate_all clean on first try, spec deviations |
| design-brainstorm | opus vs inherited | spec completeness and number of questions raised |
| logic-builder | keep inherited; try opus on one large task only | tokens, minutes, reviewer findings |

Trial protocol (user decision 2026-10-16: compare tokens and minutes only; cost is tracked in the usage page, token efficiency stands in for cost efficiency):

1. Trial 1 (reviewer): when the next feature branch is ready, run two `change-reviewer` agents with an IDENTICAL prompt on the same worktree, one with `model: haiku` and one inherited (Sonnet 5.5). Record tokens, minutes, tool calls and the list of distinct findings (true defects, false alarms, misses relative to the union).
2. Pick the model per agent type from the results and record it in the agent definition (`model:` frontmatter) and in `docs/DECISIONS.md`.
3. Repeat for `asset-builder` (first data task) and `design-brainstorm` (next spec), one trial per type.

Trial results:

| Trial | Model | Tokens | Minutes | Real defects found | False alarms | Missed (vs union) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 reviewer (M2C2B-L static review) | haiku | 69.6k | 3.5 | 1 test gap (a double status roll would not be detected) + 4 nits (self-referential haste check, unused loop var, help-text wording, trailer ambiguity) | 0 | 0 |
| 1 reviewer (M2C2B-L static review) | Sonnet 5.5 (inherited) | 28.5k | 1.0 | 0 (all PASS) + 4 nits (rng order change, hardcoded [Riposte] log tag, outdated comment, weak seeded-repeat check) | 0 | 1 (the double-roll test gap) |

Trial 1 conclusion (2026-10-16): on a static read-only review the inherited model used 41% of haiku's tokens (28.5k vs 69.6k) and 29% of its time (1 vs 3.5 minutes), because haiku explored far more (23 tool calls vs 4). Haiku found one real test gap the inherited model missed (the area-status test could not detect a double roll); the inherited model found no gap but its other nits were of similar value. Token efficiency favours the inherited model; haiku adds some extra thoroughness at 2.4x the tokens. Decision: keep reviewers on the inherited model; use haiku only when a second, deliberately thorough pass is wanted. Next trials: asset-builder (a small data task) and design-brainstorm (opus vs inherited), one per type. Other reviewer runs for reference (inherited model): M3 34k / 1, C1-L 42k / 3, D1-L 35k / 6.

Trial 2 (2026-10-16): asset-builder on haiku, SK2-A data step (small, well specified): 38.9k tokens, 3 minutes, 12 tool calls. It completed 3 of 4 steps correctly (riposte offsets, warrior path, passive templates), followed the two-attempt rule and stopped on a real schema gap (a single-row upsert cannot change `advanced` across a family) with an exact error and a clear question instead of working around it by deleting rows. The gap was a miss of SK2-L (fixed with `ContentCatalog.upsert_many`). Verdict: haiku is fine for small data steps with a clear spec and strict stop rules (about 40% of the tokens of the last inherited asset run, though that task was larger); keep the inherited model for large content tasks until a comparable-size trial exists.
