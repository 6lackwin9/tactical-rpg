# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to PR4 rows (R1 to R4 releases, the Post-R4 queue PR4-1..PR4-9) | released or done | `main` / `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-1 to PR5-6, PR5-8, PR5-16, PR5-17, PR5-20 to PR5-25 | done (FB-168 to FB-189, FB-194 to FB-196, answered) | `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-7, PR5-9 to PR5-13b, PR5-18, PR5-19, PR5-26 to PR5-35 | done, released to `main` in #62 (FB-190 to FB-234, answered @cf3d93b4) | `main` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-11) |
| PR5-36 `PK2-FOLLOWUP` | parts (1), (2), (4) done (FB-235; FB-236 questions): launch key `ruleset` primary, PK2 repair / reforge / forge rite / book / sim wiring, sunk_ward roster rules; 0 golden diffs. Part (3) absorbed by PR5-39 | `develop` | - |
| PR5-39 `CANON-CLEANUP` | done (FB-237): story-free engine / kit fixtures, lore_words CANON nouns and retired cast, Talia / Berek / Sabina refs and placeholder portraits, CANON display names (seed `canon1`); 0 golden diffs | `develop` | - |
| PR5-37 `CAST1-ENC` | done (FB-238; FB-239 sims and findings): 11 Act 1 sunk_ward encounters, loot, enemy characters, talk lines (seed `cast1enc`; the stale row text followed the specs); feel / battle cannot run world encounters yet; 0 golden diffs | `develop` | - |
| PR5-38 `ACT1-STORY` | done (FB-240; FB-241 soft-lock risk, missing hooks, questions): Act 1 Sunk Ward playable from New game through M5 (seed `act1story`); 0 golden diffs | `develop` | queue empty: waiting for Design (FB-235 to FB-241) |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-234 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-11). New items start at FB-235. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-235: PR5-36 `PK2-FOLLOWUP` parts (1), (2), (4) landed (part 3 held for the lore cleanup)

- **Spec**: READY Section 1 row PR5-36; answers to FB-211, FB-212 (d), FB-214 (d-f), FB-216, FB-218 (c-e), FB-220, FB-221 (f), FB-223 (a), FB-232 (the `PR5-36` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-236: PR5-36 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` 4-7; FB-232 answer (@ the `PR5-36` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: FB-232 asked for Act 1 "wounded `2` battles"; the `roster` rule has no duration field, so `wounded` lasts until the next rest (`roster::on_rest`). The duplicate check compares skill families, stricter than the row's "skill IDs". With `count` 1 rounded up, any higher-tier repair costs exactly 1 salvage, so only the Scrap scales.
- **Recommendation**: Questions, built default first: (a) wounded until rest (alternative: a `wounded_battles` rule field, a small kit row); (b) a themed weapon needs its theme's salvage instead of the tier's (alternative: both); (c) basic-weapon repair costs Scrap (alternative: free, `gold` 0 in data); (d) the forge-rite recipe needs mastery 3 (alternative: no gate); (e) repair salvage `count` 2+ if salvage should scale too.

### FB-237: PR5-39 `CANON-CLEANUP` landed (absorbs PR5-36 part 3)

- **Spec**: READY Section 1 row PR5-39; CANON 5.2-5.4, 6.1; Design sweep 60f2894f (the `PR5-39` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-238: PR5-37 `CAST1-ENC` landed (Act 1 sunk_ward encounters; T10-T14 partly measurable)

- **Spec**: READY Section 1 row PR5-37; `CAST1_ENCOUNTERS.md`, `CAST1_BOSSES.md`, `CAST1_GEAR.md` 8, `CAST1_BESTIARY.md`, `bestiary_encounters_act1.md`, CANON (the `PR5-37` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-239: PR5-37 sims and design findings

- **Spec**: READY row PR5-37 (7) CAST1-B, T10-T14; FB-233 (@ the `PR5-37` squash commit)
- **Kind**: sim
- **Status**: open
- **Finding**: `sim feel` and `aegis battle` accept only catalog encounters ("not a catalog encounter with a party", `rpgkit_sim/src/args.rs` `feel_defaults`; `emberwake_host/src/cli.rs` ~143), so the feel suite and the `thurible_smite` / `default_intent` check (FB-233) could not run. Through `aegis serve` autoplay (founders spawn at level 1 against bosses at levels 3-6, so not balance): T11 every fight loads under timeline; 0a / 0b / 0c 10/10 won in 4-6 / 2-3 / 5-6 rounds, Corvo's Gate 8 won, M4 10 won, M1 and the Interlude time out (Vane never damaged), M3 1 won 9 lost, M5 0 won (escape not played by autoplay). T14 from the logs: about 2 arena lines a round, the Pulverizer lands 10/10, push / pull 10/10, collisions 3/10. T10 / T12 / T13 need feel counters (0a 1-2 windup breaks; 0b intercepts but 0 smoke hazards; 0c 0 traps). Spec-sourced findings: M2's `tithe_captive_a` walks out the exit at [11,4] in round 1 every seed (catalog `flee_ratio` 1.0), so `cull_1` never finds a target; M1's gate-pass `key_loot {kill: castellan_footman}` needs both footmen slain but they withdraw under morale 10/10, so the quest item is effectively never paid (CAST1_GEAR line 89 calls it the objective reward). Lore notes: the Vesper "ash-sweeper brother" (`dlg_talk_m4_orlov`, from CAST1_BOSSES 3.4) is not in CANON's Vesper row; Nemeth's infusion line and talk both say "Cold steel cannot finish me."; spec loot names "Marrow Valve" / "Marrow Core" / "Stitch-Poultices" differ from the items' CANON names.
- **Recommendation**: (a) a kit row: `sim feel` and `aegis battle` accept world encounters (`world:<id>/<enc>` or `<world>:<enc>`) with the authored party and levels, then the CAST1-B pass (T10-T14, `thurible_smite`, default intents); (b) M2: the captive's `flee_ratio` 0 for this fight or move the exit; (c) M1: the gate pass as the objective reward or key loot on one unit; (d) Vesper's brother: confirm or drop; (e) the spec loot names to the CANON item names.

### FB-240: PR5-38 `ACT1-STORY` landed (Act 1 Sunk Ward playable)

- **Spec**: READY Section 1 row PR5-38; `sunk_ward_campaign_map.md`, `npcs_and_questlines.md`, STORY_GUIDE, SCREEN_ATLAS, CANON, CL-08 / CL-11 (the `PR5-38` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/act1story.json` (86 steps, idempotent; reproduces the data from the pre-row data; `act1story_alt.json` is superseded, its step folded in) and `aegis content manifest` `start_worlds ["sunk_ward"]`. `sunk_ward`: `start_party ["kaelen"]`, `start_sequence new_game` (opening vignette, 0a, Meat Market, Vesper joins, 0b, the cart, Maren and Hollis join, 0c, Debate 1, flag `debate1_seen`, enter `sluice_undercroft`); `roster` bunks 6 / max 8, keepsakes on captured / dead / lost; characters (Sabina and Berek dialogues; new `talia_rusk`, `ivo`, `widow_varka`, `bailiff_corvo`, `skerrit_bone_wright`); 4 quest templates; 8 vignettes (Debates 1-3 as line cards; Debate 3's founder lines hide the winch holder's, gender-neutral); sequences `new_game`, `seq_interlude`, `seq_after_m5` (sets `pk2.slab_open` / `pk2.hearth_open` after Debate 3: the Slab and Hearth recipes open at the Act 2 line, per the campaign map 3.5 and CANON); 11 dialogues (incl. the M2 count, the M3 / M4 recruits, the dose choice, Corvo's bribe with Rock Salt or Scrap); shops `sabina_slab` / `berek_hearth` (open after `debate1_seen`) and `varka_trade` (after `ivo_rescued`); `talia_board`; 13 quests (M1-M5, M3 / M4 after M1 + M2, M4 pays `loot_orlovs_retribution`, M5 variant `hollis_overlook` -> `enc_evt_06_overlook`; jobs Corvo and the Red Pen; the six founder side-hooks); triggers `t_m1_won`..`t_m4_won`, `t_interlude` (M3 + M4 -> the dose scene), `t_m5_won` / `t_m5_overlook_won`; 12 locations (the 2-tier diamond around the hub, `corvos_gate`, the prologue places) and 10 routes (Tier 2 after `m1_done` + `m2_done`, the convoy and flue after `interlude_done`, the Slag Gate after `m5_done`); PR5-37 encounters get their `location`. Tests: `emberwake_world/tests/act1_story.rs` (4: New game -> Prologue -> hub services; Tier 1 -> Tier 2 -> Interlude with the recruit and dose choices; M5 with the overlook via `explore::step`, the stay-behind flag and Debate 3 hiding the holder's line, the recipes opening; M5 without the overlook), `emberwake_gui` `talk::tests::the_sunk_ward_hub_opens_its_panes_and_the_world_map` (BoardPane, ShopPane, RecipePane, WorldMap), `shop_trade.rs` reads purse texts from `rpgkit_world::terms`, covers flag-gated shops and pins "Not enough Scrap.". Gate 3128 passed, 0 failed, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (fixed: Debate 3 lines, Corvo's Scrap node, next-bunk recruits, "by the next bell", "a worse Tithing", the seed order; tests drive the recruit / dose choices and the real overlook).
- **Recommendation**: STRUCTURE: `worlds/sunk_ward/{sequences,vignettes,shops,boards,quests,triggers,locations,routes}.json`, seeds `act1story.json`; the two test files. README / GAME_BRIEF: Act 1 Sunk Ward is playable from New game. Provisional numbers: Slab poultice 12, draught 20, bomb 25 (sell x0.4); Hearth prices from item value; Varka salt 25, bone shard 20, supplies 8 / day; raids 120 XP + 40 Scrap + 1 poultice; hooks 60 XP + 20 Scrap (Rock Salt hook 15, Plating hook 10); Corvo bribe Alert -20; two +5 supplies chests.

### FB-241: PR5-38 soft-lock risk, missing hooks and questions

- **Spec**: READY row PR5-38 (@ the `PR5-38` squash commit)
- **Kind**: conflict
- **Status**: open
- **Finding**: **Soft-lock (player-facing)**: a queued scene dialogue can be closed with Esc or right-click without a choice (`emberwake_gui/src/runover.rs` ~194 `pending_talk` removes it from the queue before it opens; scenes have no farewell, so `talk.rs` ~494 `Services::back` -> `SvcAct::Farewell` -> `end`; nothing re-queues it). Dismissing `dlg_int_dose` means `seq_interlude` never starts (`t_interlude` is `once`): `interlude_done` never sets and M5, the flue and the convoy stay locked for good; dismissing `dlg_post_m3` / `dlg_post_m4` loses Dunstan / Elyse and their side-hooks. Missing hooks (kit rows): nothing reads the M2 captive survivors (Maren asks "both / one / neither", exploitable); New Game ignores `start_worlds` (the Ashlands is still preselected); effects and quests cannot grant supplies (two map chests instead); items for the side-hook rewards do not exist (Hollis's Lever-Winch, the Alum Inhaler, Skerrit's cestus and quarrel recipes; the Bellows' Hearth discount has no flag-priced shop); road fights roll 15% on the new routes but no road encounters exist; nothing sets `campaign.act` 2 after M5, so `durability.from_act: 2` and mastery never switch on although the Hearth's repair / drill recipes open; no marrow skill for the dosed founder in `enc_evt_05` (PK2_GRAFTS 3); the dose is offered to all six founders plus "Nobody" (the docs: one deployed founder). Doc inconsistencies: the Corvo node id (`corvos_gate`: READY, SCREEN_ATLAS 3b, ASSETORG3; `third_ward_gate`: CAST1_ENCOUNTERS, campaign map 6); the M5 name (READY / SCREEN_ATLAS "The Drowned Sluice", the campaign map and CANON "The Crucible"); Corvo's route ungated (CANON row 115: M1's Gate Pass opens ward / Corvo routes); `vg_m5_gate` reuses `vn_hearth_stage.png` (placeholder).
- **Recommendation**: (a) soft-lock first, built default none: options (1) the GUI refuses Back on a scene with open choices, (2) Back re-queues the scene at the head of the queue (smallest change, lowest undo cost), (3) data: a default effect or a re-entry reply on Sabina for the dose; (b) `seq_after_m5` `set_act 2` (and whether salvage belongs in Act 1); (c) kit rows for the M2 survivor read, New Game `start_worlds`, supplies grants, flag-priced shops; (d) the missing reward items; (e) settle the Corvo id and the M5 name.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
