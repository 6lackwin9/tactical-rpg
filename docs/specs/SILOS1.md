# SILOS1: audit of the built systems as silos

Status: proposed
Depends on: none (BASERULES1 covers the base rules silo; only its seams are named here)
Lane: design (audit); the rows in section 6 go to logic, integration and planner
Touches: this file only

## Goal

Map every built system, how independent it is, and the few seams worth cutting so each can be tested alone, then joined (decisions "Order of work: engine and kit first", "Silos: data to tweak, options to expand"; `docs/WORK_ORDER.md`).

**Verdict.** Logic already runs alone: the crate graph is clean, and battle knows no world. Most of the tangle is in `emberwake_gui`. Two seams pay now: the world fight's start and settle (T1), and the talk, shop and board panes inside the region map (T2). An acyclic module graph is not worth chasing (T6).

Measured on `staging` at `db55a63e`; `WORK_ORDER.md` and the decisions come from the plan checkout (`45552892`). **V** = verified by script or grep; **R** = reasoning. Live lines leave out comments and `#[cfg(test)] mod x {` bodies. Cycles are strongly connected components over `crate::x` paths and `use crate::{..}` groups, folders folded into one module.

## 1. System map

**Alone**: needs no other gameplay system. **Thin**: one narrow seam. **Tangled**: reaches several systems, or cannot run without them.

| System | Layer: crates or modules | Owns | Reads from others, and how | Independence |
| --- | --- | --- | --- | --- |
| Base rules (BASERULES1) | kit: `rpgkit_data`, `rpgkit_rules` | stats, damage, gear, status, ruleset | catalog types only | alone; `rules` depends on `data` and `aegis_core` only (V) |
| Battle rules, AI | kit: `rpgkit_battle` (19,559 live) | session, resolver, hooks, encounters, `ai.rs` | calls into `rules` and `data`; no `rpgkit_world` dependency (V) | alone |
| Planning input | kit: `round_intent*`, `plan`; `rpgkit_host` | picks, ghost fill, plan queries | the same session; the screen sends through `Host` | alone |
| Battle screen | game: `battle.rs`, `battle/*` (7,344 live) | drawing, input, playback | 19 battle modules read; writes through `self.host.handle`; 8 world modules; pushes results | thin in arena and sandbox modes, tangled in `Mode::World` (T1) |
| Sim | kit: `rpgkit_sim`; game façade | balance runs | battle and rules; `SimGame` for the game's part | alone, but not on the world fight path (T1) |
| Region map and movement | kit: `explore.rs`; game: `region.rs` (1,847 live of 5,769), `mapview`, `props`, `ground` | walking, markers, camera | `explore` calls `quest`, `board`, `shop`, `campaign`, `roster`; the screen hosts talk, shop, board (T2) | tangled on screen, thin in logic |
| World map and travel | kit: `travel.rs`; game: `worldmap.rs` | graph, legs, road fights | `campaign::tick` with a `Cause`; `region::begin_fight`; `runover` | thin |
| Dialogue | kit: `dialogue.rs` (fan-out 13); game: `talk.rs` | runner, effects interpreter, conditions | every effect target, by direct call; screen draws a view `Dict` | hub by design; `talk.rs` itself thin |
| Quests | kit: `quest.rs`; game: `hub.rs` Journal | status, flags, objectives | mutual with dialogue (`dialogue` <-> `quest`, V) | treat as one silo with dialogue (Q1) |
| Story and vignettes | engine: `vignette.rs`; kit: `story.rs`; game: `story.rs` (256 live) | sequences, cards, story fights | `region::try_fight`; engine player has its sample reel | player alone; stage thin |
| Mission board | kit: `board.rs`; game: `board.rs` | views, jobs | `quest`, `dialogue` keeper lookups; has a fixture pane | thin |
| Party and roster | kit: `roster.rs`, `state.rs`; game: `party.rs` (2,059 live) | members, wounds, bunks, departures | `party.rs` reads 9 rules modules, 4 world, 2 battle by call | thin |
| Gear and equip | kit: `gear.rs`, `carry.rs`, `budget.rs`; game: `party.rs` Gear tab, `dojo.rs` | slots, training, pouch | rules only; `party.rs:1156` writes `gs.party.stash` directly (the only live write among the hub screens, V) | thin |
| Shop and economy | kit: `shop.rs`, `purse.rs`; game: `shop.rs` (417 live) | prices, trade, stock | `quest::all_hold`, `dialogue::{rep, is_gone}`, `terms`; no `campaign` (V) | thin in logic; the pane has no `region` import in live code (V) but only runs inside one (T2) |
| Progression | kit: `progression.rs` | XP, level, rest | `explore`, `travel::current_node`, `campaign::purse` for the rest rule | thin |
| Campaign | kit: `campaign/` (1,628 live); game: `runover.rs`, `hud.rs` | clock, meters, triggers, purse | `Option<Campaign>` on `GameState`; `tick(Cause)` at three call sites | thin; off in a world without it |
| Saves | engine: `aegis_save`; kit: `save.rs` (169 live), `state.rs` | slots, format | the whole `GameState`; blocks are additive | engine alone; kit thin |
| Content store and checks | engine: `aegis_content`; kit: `rpgkit_content` (15,264) | store, checks, art chains | `Hooks` trait; `rpgkit_battle` for grid and encounter parsing | alone (headless) |
| Audio | engine: `audio.rs`; game: `music.rs` | buses, tracks, the context chain | `music.rs` uses no kit module (V) | alone |
| FX and portraits | engine: `fx.rs`, `portrait_fx.rs`; kit: `battle/portrait.rs`; game: `fx.rs`, `portraits.rs` | effects, timing, cues | `fx.rs` reads only `grid` and `unit` from the kit, and `stage`, `tuning` (V) | alone (engine), thin (game) |
| Layout and UI kit | engine: `aegis_layout` (no deps), `aegis_ui` | widgets, theme, ground | none in gameplay; `theme` is thread-local state | alone |
| Agent link and scenarios | engine: `aegis_host`, `aegis_cli`, `bridge.rs`; kit: `rpgkit_host` | requests, headless runs | `Screen::state`, opaque command sets | alone; battle commands only on the kit side |

## 2. Evidence

**Crate graph (V, `Cargo.toml`).** Engine crates depend only on engine crates. `rules` -> `data`; `battle` -> `rules`; `content` -> `battle`; `world` -> `content`, `aegis_save`; `sim` and `host` on top. The GUI depends on façades and the engine, on no `rpgkit_*` crate directly. No cycle, no wrong-way edge.

**Module cycles (V).**

| Crate | Live lines | Largest cycle | Highest fan-in |
| --- | --- | --- | --- |
| `emberwake_gui` | 21,996 | 22 of 28 modules; still 14 with the dev tools and `menus`, `hub`, `hud` removed | `screens` 20, `region` 8, `portraits` 8, `hub` 8 |
| `rpgkit_world` | 8,991 | 16 of 19; outside it only `data`, `save`, `skill_text` | `state` 16, `data` 15, `campaign` 12 |
| `rpgkit_battle` | 19,559 | 15 of 26 (rule core), plus 5 (session side) | `world` 20, `grid` 19 |
| `aegis_ui` | 16,995 | 12 of 35 | `canvas` 25, `painter` 18 |

**Hub files, by live lines (V).** `region.rs` is 5,769 lines but 1,847 live: 63% is tests, so its size overstates the tangle. Its fan-out is the true measure, 17 GUI modules, the highest.

| Screen | Live | Kit modules reached |
| --- | --- | --- |
| `battle.rs`, `battle/*` | 7,344 | battle 19, world 8, rules 3, content 1 |
| `party.rs` | 2,059 | rules 9, world 4, battle 2 |
| `region.rs` | 1,847 | world 7, content 1 |
| `hub.rs`, `menus.rs` | 608, 977 | world 6 and 5, battle 1 each |

**Screens and global state (V).** The `screens.rs` header says screens "never call each other"; navigation is queued, but they construct each other directly (`hud.rs:58`). Fights all pass `region::begin_fight`, `try_fight` or `fight_now` (`worldmap.rs:189`, `story.rs:186`, `runover.rs:185`). Gameplay has no global state: statics in the GUI and kit are caches, id counters and a once-only warning set.

**Game code that should be kit (V).** `BattleScreen::world` (`battle.rs:677`), `after_fight` (`:202`), `lower_start_ap` (`:215`), see T1. All screens are game code by KIT1's KQ4(a), whose KIT2-UI has no `docs/TASKS.md` row today.

**Kit that knows too much (V).** `rpgkit_rules/src/unit.rs:924` falls back to class `"warrior"`; `rpgkit_data/src/lib.rs:98` defaults art to it; `rpgkit_sim/src/args.rs:12,52` and `roles.rs:25` fix six class ids, a comp and a class-to-role table.

**What `tools/fc.sh layers` does not catch (V).** It catches game ids in engine and kit, world ids and lore words (`lore_words.txt`, 26 lines, kept by hand) in kit only, and wrong-way path dependencies. It misses:

- Genre in the engine: `battle` keys in `aegis_host/src/playtest.rs` (18 lines), `aegis_cli`, `aegis_ui/src/recordings.rs` (`Act::Battle`), `shell.rs:377`.
- Data ids in the kit that are no lore word (the class ids above).
- Rule flow in a screen (the STRUCTURE rule "`emberwake_gui` only draws").
- World ids and lore words in the engine.

## 3. Tangles, ranked by cost to undo later

| # | Coupled | Breaks when one side changes | Smallest seam | Size | Call |
| --- | --- | --- | --- | --- | --- |
| T1 | World fight start and settle sit in a screen (`BattleScreen::world`, `after_fight`, `lower_start_ap`) | Each new fight rule lands in the GUI. Sim and host skip `field_unit`, `battle_mods`, `party_min_refusal` (V: only `rpgkit_world` and the GUI call them), and `journey.rs` draws outcomes, so no headless path plays a real world fight. Balance can differ from play (R). | Kit `battle::begin_world(d, gs, fight) -> Result<WorldFight, String>` and `battle::finish_world(d, gs, source, fight, outcome, tally)`; the screen adds art through a callback | M | **Do now**; grows with every row |
| T2 | The region map hosts talk, dialogue runner, shop, board and pending picks | Shop and dialogue cannot open elsewhere; Gallery's "shop" builds a `RegionMap`; `shop.rs` tests need one; every region task overlaps every service task | A `Services` struct (runner, shop, board, talk, `dialogue_result`), held by the region map; a thin `Screen` wrapper on a plain backdrop for the Gallery | M | **Do** at phase 2's start |
| T3 | Genre screens in one 22-module game crate | A second game rewrites 22k lines (R); moving them before LAYOUTDATA1 and LOOK1-M2 means rebasing those | KIT2-UI spec: open by screen id instead of `crate::x::new`, generic over the game's `Cx` | L | **Spec only**, in phase 4, timing Q2 |
| T4 | Kit assumes class ids | A game without `warrior` gets a blank fallback; the sim's `CLASSES` fail or mislead (R) | A content error instead of the fallback; comps and roles through `SimGame` or data; sim golden byte-equal | S | **Do**, with BASERULES1 |
| T5 | Results and much state are JSON maps: 65 of 453 `rpgkit_world` pub fns return `Dict`; `explore`, `flags`, `quests`, `shops`, `rolls` are maps; screens read `"kind"`, `"fight"`, `"pending"` by string | A renamed key compiles; goldens catch what they walk | A typed `ActionResult` for the join keys only, when a join is touched | M | **Leave**, except inside T1's new functions, which return types |
| T6 | `rpgkit_world` cycle of 16: `state` points at `campaign`, `story`, `quest`, `roster`, `progression` | Little at run time: scratch-world tests exist for board, campaign, roster, story, travel. Cutting `shop` and `board` from `dialogue`, `quest`, `explore` leaves the cycle at 16 (V, what-if run): `shop` -> `state` -> `campaign` -> `dialogue` -> `shop` | Would need `state.rs` split from behaviour | L | **Leave**; cycles are not what stops a silo here |
| T7 | Dialogue interprets every effect by direct call; the battle screen reads 19 battle modules (89 `session()` reads) | A new effect edits one match; a new rule adds a mark. Both are inherent, and battle writes already pass the command API | None | - | **Leave**; a trait would add indirection for one implementation |

A grep of `docs/HISTORY.md` finds no dropped idea on these seams; its withdrawn "~85% engine reuse" claim (2026-10-25) argues for measuring, as here.

## 4. Joins

| Pair | Class | Exploration or caller promises | Other side promises | Note |
| --- | --- | --- | --- | --- |
| Exploration -> shop (vendor), board | **easy** | Raises `interact`; an `npc` result goes to dialogue | `open_shop` carries a shop id; `close_shop` returns to the talk | **Surprise:** the path is explore -> dialogue service reply -> shop; a vendor is an NPC with a `_service`, with no `shop` marker. Still one effect, one id |
| Campaign clock <- travel, enter, battle | easy | `tick(Cause)` at three sites | merged `events`; `runover::take` shows them | **Surprise:** already event-shaped, the cleanest join in the world crate |
| FX, portraits <- battle playback; audio <- screens | easy | step events; a screen id | `fx.rs` needs only `grid`, `unit`; `music.rs` no kit module | |
| Gear -> battle | easy | `field_unit` builds the unit | battle takes a `Prebuilt` | one way |
| Save <- systems | **easy, not trivial** | A block is saved only when used | Version bump on release; `set_busy` while a screen is open; `campaign::save_refusal` | Three separate gates to know |
| Travel <-> exploration | medium | `travel::current_node`, exit targets | `explore::enter` with anchor, entry problems, the campaign tick | Rest depends on both |
| Story -> battle | medium | `FIGHT_SOURCE`, retry or abort | T1's settle | |
| Quests <-> dialogue | medium | Same silo | | Q1 |
| Rewards, roster <- battle | medium | `Tally::of_world_fight` | `settle_party`, `apply`, stay-behind `then` | Ordered: party first, then rewards |
| **Exploration -> battle** | **hard, looks easy** | "Fight" raised with `encounter` and a source string | Setup in the screen: world, party_min, ruleset, grants, wounds, upkeep AP, zone. Settle by five sources. A `debug` fork opens a popup | T1 makes it medium |
| **Campaign <-> battle** | **hard, looks easy** | `fight_ask`, `days_ask` before; `trigger::FIGHT_SOURCE`, the queue head | `battle_mods`; retreat on defeat; run-over replaces every screen | The asks are called from `region` and `worldmap` |

## 5. Testing each silo alone

`Forge` waits on CLN-71 and gets no new work (decision "No upkeep on unused debug tools"). A scenario launches `menu`, `arena` or `runner` only, with no save fixtures and no region `walk_to` until INTEG1-G (`scenarios.md`); it can reach Gallery entries by clicking through `runner` (R: not yet tried).

| Silo | Run and judge today | Placeholder needed | Tweak in data | Locked in code |
| --- | --- | --- | --- | --- |
| Battle rules, AI, planning | 29 test files, `rpgkit_host` tests, goldens, `sim` (duel, tier, group, party, role, sweep, stats, levels, `feel`) | none | catalog kinds, rulesets, `ai_tactics` | BASERULES1 says |
| Battle screen, FX | Arena scenario (`timeline_fight`), `shots`, Gallery panels, key-T timeline | Sunk Ward fight | `tuning.json` (live) | marks, layout (LAYOUTDATA1) |
| Region map | Gallery `region`, kit `explore_leader`, `ground_walking`, TURNFEEL1's debug options | **Sunk Ward map as Gallery world** | maps, markers, grounds; `walk_step_ms` is a setting | walking consts, camera |
| World map, travel | `travel_modes`, `sim deadlines`, Gallery `world_map` | none | routes, `travel_modes` | graph rules |
| Dialogue, quests | Gallery `dialogue` (first NPC only), `journey`, `npc_services`, `content` checks | none beyond the Gallery world | dialogues, quests, service lines | effect set (additive option) |
| Story, board | Gallery reel picker, `BoardSample` fixture, kit `story`, `board`, `expressions` | none | vignettes, sequences, boards | card kinds |
| Party, roster | kit `roster`, Gallery `stats`, `gear`, `level_up`, `skills` | none | roster rule rows, bunks | Tab layouts |
| Gear | Dojo (feel tool), `wear_drills`, `rules_golden` | none | families, drills, `wears_untrained` | slot model |
| Shop | `shop_trade`, Gallery `shop` through a `RegionMap` | **T2's plain backdrop** | shops, prices | `SUPPLIES_MAX` 99 |
| Campaign, progression | 4 kit test files, `sim deadlines`, `rewards`, `journey` | none | meters, triggers, checkpoints, XP curve | tick order |
| Saves, content, audio | `save_versions`, `slot_locks`, store resave; `aegis content`, `checks_golden`; volume previews | none | schema, files | save format, buses |
| Layout, UI kit | Gallery `kit`, `layout`, `frame_tuner` (feel tool), `shot_compare` | none | theme | until LAYOUTDATA1 |

**Gaps worth filling, as someone would use them:**

1. Gallery's sample game is the first sorted playable world, `ashlands` (V), while the decision names the Sunk Ward as the base. A world picker beside the reel picker: S.
2. A scratch-world kit test for shop: `shop_trade` runs on shipped data, so data work can fail a logic check (R). S.

**Not worth building now:** a scenario `launch` per screen; save fixtures (until INTEG1-S1 needs them); a dialogue node picker; per-system debug screens.

Numeric consts: GUI 152, world 29, battle 51 (V). The last two columns start the list decision "Silos" asks for; SILOCARD1 keeps it current.

## 6. What to do first

Rows to add; none is a rewrite.

| Phase | Id | Title | Layer | Lane | Dependencies | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | KITIDS1 | T4: no class id assumed in the kit; sim comps and roles from the game; sim golden unchanged | kit | logic | BASERULES1 | `rpgkit_rules/src/unit.rs`, `rpgkit_data/src/lib.rs`, `rpgkit_sim/src/{args,roles,campaign}.rs`, `emberwake_sim/src` |
| 1 | WORLDFIGHT1 | T1: `battle::begin_world`, `battle::finish_world` in the kit; the screen adds art and draws; journey golden unchanged | kit + game | logic | VISCLAR1-D (overlap) | `rpgkit_world/src/battle.rs`, tests; `emberwake_gui/src/battle.rs`, `battle/flow.rs`, `runner.rs` |
| 2 | SERVICES1 | T2: `Services` out of `RegionMap`; plain-backdrop wrapper; Gallery's `dialogue`, `shop` use it | game | logic | WORLDFIGHT1 (`start_fight` overlap) | `emberwake_gui/src/{region,talk,shop,board,gallery}.rs` |
| 2 | GALLERYWORLD1 | A world picker for the Gallery's sample game | game | logic | - | `emberwake_gui/src/gallery.rs` |
| 3 | SHOPTEST1 | Scratch-world kit test for shop prices, stock and trade | kit | logic | - | `rpgkit_world/tests/shop.rs` |
| 4 | KIT2-UI | Spec only (T3): which screens move and how; KIT1 KQ4(a) | kit + game | design | Q2 | `docs/specs/KIT2_UI.md` |
| any | SILOCARD1 | Builder reports and spec template state Tweakable and Locked per decision "Silos" | process | planner | - | `.claude/agents/logic-builder.md`, `games/emberwake/docs/specs/README.md` |
| any | SILOSCN1 | One short scenario per silo through `runner` and the Gallery; not in the gate | game | integration | INTEG1-R, GALLERYWORLD1 | `games/emberwake/scenarios/` |

**Rows that should move:**

| Row | From | To | Why |
| --- | --- | --- | --- |
| INTEG1-G | any time | head of phase 2 | The region silo needs `walk_to` |
| TURNFEEL1, LOOK1-L2, QUESTTRACK1-U | 2, 2, 3 | stay, after SERVICES1 | The first two name `region.rs` in Touches; the tracker draws on the map (R) |
| PLAYTEST2 | phase 2 | later than WORLDFIGHT1 | Needs a headless world fight; a neutral recording key (section 2) goes with it |

**Base rules seams (BASERULES1 owns the content).** `build_unit_in`, `field_unit`, `Ruleset::of` (read by `party`, `dojo`, `flow`, `input`), `derived` and `damage` as `party.rs` shows them, class ids (KITIDS1).

## Out of scope

Rewrites, traits or event buses for their own sake; an acyclic `rpgkit_world`; moving any screen now; base rules content.

## Open questions

1. **Q1. Are dialogue and quests one silo?** They call each other and share conditions and effects. Recommendation: yes, one silo for testing, the code still in two files. If no, T6-scale work appears.
2. **Q2. When do the genre screens move to the kit (KIT2-UI)?** (a) Spec at phase 4's start, move after LAYOUTDATA1 and LOOK1-M2 (the KQ4 answer); (b) move before phase 4, so screens convert once, stopping every UI row during the move. Recommendation: (a): the silo phases keep touching these screens, and the move is safest when they rest. (b) changes phase 4's order.
3. **Q3. Should the sim fight world fights through `begin_world` after WORLDFIGHT1?** It shows upkeep and wounds in balance runs but can shift sim numbers. Recommendation: not by default; a new sim mode when CAMPAIGN1 balance starts; the sim golden stays.
