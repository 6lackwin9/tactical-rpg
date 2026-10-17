# Roadmap (agent-facing, high level)

Single source of truth for scope and status. Detailed per-system checklists remain in README "Roadmap". Do not delete entries: change the status and add the reason (see `docs/HISTORY.md` for deprecated ideas). Decided brainstorm options, answered questions and done plans (2026-10-15 brainstorm pass, plan v2, R1) are archived verbatim in `docs/archive/ROADMAP_2026-10.md` (CLN-11).

**Status values**: `done` (implemented, tested) | `balance-ready` (done and frozen: the balance agent may fine-tune it) | `in-progress` (a milestone is building it) | `approved` (milestone approved by the user, not started) | `proposed` (idea from brainstorm, not approved) | `placeholder` (folder reserved in `docs/STRUCTURE.md`, nothing built) | `deprecated` (kept for history; see HISTORY.md).

Only the user moves an item to `approved`. Only the planner (main session) sets `in-progress` and `done`. Only the user/planner set `balance-ready`.

## Systems

| # | Area | Status | Depends on | Notes |
| --- | --- | --- | --- | --- |
| S1 | Combat core: block resolver, planner, stances, AP, damage formula, statuses | balance-ready | - | `DamageCalculator` is the only formula |
| S2 | Classes, units, tiers, proficiency, passives, skill paths (six classes) | in-progress (M2c-1 roster, M2c-2a1 families/loadout, M2c-2a2 engine pieces, M2C2B content, SK2 level unlock and SKILL1 ranked skills done; BAL2 data pass and rerun next) | S1, S12 | Loses balance-ready until the rework is built and re-frozen |
| S3 | Gear and inventory: requirements, tiers, burden, carry, stash | done (gear budget guard added in M2d) | S1 | Gear budget per class is an open balance lever |
| S4 | Enemy AI (roles, presets, targeting) | done (M1: kiting, no idle turns, support heals, reckless profile) | S1 | Shaman/ranger AI wastes turns: candidate fix |
| S5 | Content catalog + Content Forge | done | - | Data pipeline for S2/S3/S6 |
| S6 | World engine + World Forge (lore layer, inheritance) | done | S5 | Characters, factions, items, quests as data |
| S7 | Party and progression: persistent unit state (spend, learned skills, passives), level-up UI, XP | done (M3 data model; level-up UI still open) | S2 | First gameplay-facing gap |
| S8 | Quests and lore runtime: quest state, rewards granting passives/skills/items, flags | proposed | S6, S7 | Sources of passives |
| S9 | Region map and exploration: tiles, locations, encounters triggers, interactions | placeholder | S6, S8 | README System 2; inputs below |
| S10 | World map and travel: routes, travel events, supplies | placeholder | S9 | README System 1; inputs below |
| S11 | NPCs and dialogue: dialogue trees, shops, factions reputation | placeholder | S8 | README System 9; inputs below |
| S12 | Advanced combat: combos, terrain effects, status expansion, skill upgrades | in-progress (S12-A engine done in M1: ally-target support skills, gap-close, haste/bleed/taunt; terrain and combos remain; riposte done in M2a) | S1, S2 | README System 6 |
| S13 | Save/load and game state, new-game flow, main menu | done (M3 schema and save I/O; menus still open) | S7 | Needed before any playable loop |
| S14 | Audio, animation and asset pipeline, asset-load tests | placeholder | - | README Test Suite 2 |
| S15 | Balance tooling and reports | done | S1, S2 | `tools/balance_sim.gd`, `docs/BALANCE_REPORT.md` |

## Proposed milestone order (planner refines; user approves each)

Superseded: the M1-first "balance pass" order (see HISTORY.md). Current order is the "Suggested build order" below; the balance pass comes after the balance-relevant rules are frozen. Nothing below is `approved` until the user says so.

### Suggested build order (each step needs user approval; balance only after step 7)

1. S7 data model (state, XP curve) + S13 save schema (together they define the party snapshot).
2. S12-A (ally targeting, statuses, skill upgrades) with S4-A (turn-waste fixes, support AI); S3 gear budget validation + missing class items.
3. S8-B quest/flag runtime (rewards via budget rules).
4. S7 level-up UI + S13 menus/new-game flow.
5. S9 exploration, S11-A dialogue/shops, S10-A travel.
6. S12-B terrain/AoE, S11-B reputation, S4-B profiles.
7. Freeze S3 gear sets, S4 presets, S12 rules, S7 curve, S8 reward budget; mark balance-ready; then the balance pass (supersedes the M1-first ordering above).
Until step 7, only S1/S2 stay balance-ready and no balance work is done.

State 2026-10-17 (CLN-11 note): steps 1 and 2 are done (M1, M2, M3, AI1, SK2, SKILL1); the user approved the BAL1 and BAL2 runs ahead of step 7 (DECISIONS "Milestone plan v2 picks", "BAL2 answers"). Queue: `docs/TASKS.md`.

## Inputs for placeholder systems (S8-S11; nothing approved)

Design options for S8-S11 from the 2026-10-15 brainstorm (P-S8 to P-S11 with recommendations) are in `docs/archive/ROADMAP_2026-10.md`; the settled parts are DECISIONS "Exploration encounters", "Travel supplies" and "Playable scope". The early design notes (`.context/GAME_DESIGN.md`, removed in CLN-4) add these phase-1 baselines, kept as input for the S9-S11 specs (the 10-block combat layer they sat on is deprecated, HISTORY row 1, so only the non-combat parts are kept):

- S10 travel: a node graph of regions (plains, mountains, forests, ruins, settlements) with a simple adjacency list; flat travel time per route (example 3 time units); a fixed encounter chance per travel leg, fought in the route's biome. Later additive rules: route weather, road hazards, supplies (reduced version only, DECISIONS "Travel supplies"), merchant caravans.
- S9 exploration: a 2D top-down grid per region with `Vector2i` step movement; tile zones SAFE (towns, sanctuaries: no rolls, rest, trade, dialogue, ambient NPCs with rumours, fixed special encounters) and DANGER (wilderness, dungeons, ruins: step counter triggers mob fights, visible roaming elites and bosses start a fight on touch or proximity); exit tiles on edges or portals open the world map. Later additive rules: bridges, locked gates, puzzle switches, loot chests, stealth and detection cones.
- S11 dialogue and S8 flags: a world-state key/value flag registry; static NPC prompts with branching choices; quest flags that unlock exits or trigger story events. Later additive rules: faction reputation meters, barter shops, timed world events.
- S12-C combo ideas (from `.context/COMBAT_SYSTEM.md`, concepts only, numbers belonged to the deprecated layer): flow moves (an action that ends in a different stance than its kind, so the next action skips the switch cost), a hold-stance action whose duration scales some skills, and disruptive hits that break the target's stance or interrupt a multi-block action. Check HISTORY before proposing any of them.

## Refactor backlog (separate from feature milestones; never part of a builder's feature task)

Done rows (R1) are archived in `docs/archive/ROADMAP_2026-10.md`.

| Id | Item | When | Rules |
| --- | --- | --- | --- |
| CLN-9 | approved (user 2026-10-17, DECISIONS "Cleanup pass 1 answers"): split `content_catalog.gd` (1384 lines) validators by kind into `scripts/content/rules/*` so ITEM1 and CLASS1 stop colliding on one hub file (docs/specs/CLEANUP.md#CLN-9) | after BAL2-L merges, last item of the logic cleanup batch, before ITEM1 and CLASS1 | Pure move; assertion count equal |
| CLN-10 | done 2026-10-17: `docs/DECISIONS.md` is an index; per-system files in `docs/decisions/`; superseded rows in `docs/decisions/archive.md` with "superseded by" pointers (docs/specs/CLEANUP.md#CLN-10) | done (docs only) | No row text changes; pointers in CLAUDE.md and agent files updated by the planner |
| CLN-13 | approved, parked (user 2026-10-17): README becomes an overview plus per-topic `docs/features/*` (docs/specs/CLEANUP.md#CLN-13) | next large README update | Planner owns README |

## BAL2 plan (proposed, design-brainstorm 2026-10-17; details and options in `docs/specs/BAL2_PROPOSAL.md`)

Answers the BAL1 open issues. Root causes found by probe: a PERC >= 18 range bonus turns melee units into "ranged" ones (priest L7, rogue L7, warrior L9 lose their melee power), a 4x durability spread (L5 EHP warrior 597 vs mage 146), zero CONS growth for mage/ranger/rogue, kits frozen at L5 by the default loadout rule, no L1 class skill for ranger/brawler, and a near-deterministic sim. Riposte is not the warrior lever (L5 warrior wins 100% with or without it).

| Milestone | Status | Goal | Lane split | Depends on | Balance-ready afterwards |
| --- | --- | --- | --- | --- | --- |
| BAL2-L | proposed | Sim reliability (continuous score, cap 10, smart_skills/loadout args, spend/distance/preset variants, stats and group modes; per-unit metrics, party-vs-encounter mode, role limit tests, composition sweep: docs/specs/BAL2_SIM.md), PERC range fix, unit gear sentinel `none`, elite/boss tier values | 1 logic branch, sim commit first | SKILL1-L merged (content hub files) | none |
| BAL2-A | proposed | Data pass: growth (CONS for damage classes, warrior CONS2 STR2), warrior kite shield, ranger/brawler L1 path skill, unit loadouts, goblin_shaman gear and text | 1 asset branch (seed `data/seeds/bal2.gd`) | BAL2-L sentinel commit | none |
| BAL2 | proposed | Rerun duel L1/3/5/7 on the continuous score, tiers, levels, group; one more small data round if a class is outside 40-60 | balance run on clean main | BAL2-L, BAL2-A | S2 and S4 (when every class is in bracket) |

## World systems baseline (approved: user accepted every recommendation, DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)")

Rules: `docs/specs/S9_RULES.md` (exploration, EXP1), `S10_RULES.md` (travel, TRAV1 + TRAV2 supplies idea), `S11_RULES.md` (dialogue, shops, reputation: DLG1, REP1). Build specs (status `spec` in `docs/TASKS.md`, awaiting approval): `docs/specs/EXP1_L.md` + `EXP1_A.md`, `DLG1_L.md` + `DLG1_A.md`, `TRAV1_L.md` + `TRAV1_A.md`, `REP1_L.md` + `REP1_A.md`. Progress is flags only; one shared battle bridge for exploration, road and dialogue fights; one seeded random stream per save.

| Milestone | Status | Depends on | Notes |
| --- | --- | --- | --- |
| EXP1 | approved | M4-L | region maps, step-counter encounters, patrols, location entry hooks; save v3 |
| DLG1 | done | M4-L, EXP1-L | dialogue, service choices from resolved quests, shops, gold, `gone_flags`; save v4 |
| TRAV1 | approved | EXP1-L | world map, routes, fog, road encounters, world clock; after DLG1 (shared hub files); save v5 |
| REP1 | approved | DLG1-L | reputation int flags, condition operators, price multipliers; no save bump; after TRAV1-L (shared hub files) |
| TRAV2 | idea | TRAV1, DLG1 | supplies |

EXP1, DLG1, TRAV1 and REP1 all edit `world_schema` and `world_catalog` (EXP1, DLG1 and TRAV1 also `game_state` and the save files): run the logic parts in sequence EXP1-L, DLG1-L, TRAV1-L, REP1-L, one save version bump each (v3, v4, v5; REP1 none). Each asset part may run beside the next logic part (data only).

## Core and presentation baseline (proposed, design-brainstorm 2026-10-17; awaiting user answers)

Rules and open questions: `docs/specs/S12_RULES.md` (S12S status expansion, S12T terrain, S12C combos; skill upgrades covered by SKILL1 ranks and grades), `S14_RULES.md` (AssetCatalog manifest, presenter animation, SoundBank, asset-load suite), `UI_MENUS_RULES.md` (menus, save slots, new game, party screen with level-up, journal). Mechanics only add inputs to `DamageCalculator.compute`; numbers are data fields, no balance values proposed.

| Milestone | Status | Depends on | Notes |
| --- | --- | --- | --- |
| S12S | proposed | ITEM1-L, CLASS1-L | status slice, then S12T terrain and S12C combos in sequence (shared skill and plan files) |
| S14-1 | proposed | - | AssetCatalog manifest, key resolution, placeholder icons, asset-load suite; then presenter (S14-2) and audio (S14-3) |
| UI1 | proposed | M4-L | menu router, main menu, new game, save slots; then UI2 party screen and level-up, UI3 journal |

## Open engine items (tracked)

- `ItemArt.icon_key` still reads the deprecated `shortsword` off-hand category (HISTORY row 23): cleanup candidate.
The resolved bullets (AI hooks, done in AI1) are archived in `docs/archive/ROADMAP_2026-10.md`.

- Disengage "once per round" and "after a MOVE" are not enforced/modelled (same as plain MOVE).
- Skill-tier picker is not rendered in the planner UI yet (grouping API exists).
