# Roadmap (agent-facing, high level)

Single source of truth for scope and status. Detailed per-system checklists remain in README "Roadmap". Do not delete entries: change the status and add the reason (see `docs/HISTORY.md` for deprecated ideas).

**Status values**: `done` (implemented, tested) | `balance-ready` (done and frozen: the balance agent may fine-tune it) | `in-progress` (a milestone is building it) | `approved` (milestone approved by the user, not started) | `proposed` (idea from brainstorm, not approved) | `placeholder` (folder reserved in `docs/STRUCTURE.md`, nothing built) | `deprecated` (kept for history; see HISTORY.md).

Only the user moves an item to `approved`. Only the planner (main session) sets `in-progress` and `done`. Only the user/planner set `balance-ready`.

## Systems

| # | Area | Status | Depends on | Notes |
| --- | --- | --- | --- | --- |
| S1 | Combat core: block resolver, planner, stances, AP, damage formula, statuses | balance-ready | - | `DamageCalculator` is the only formula |
| S2 | Classes, units, tiers, proficiency, passives, skill paths (six classes) | in-progress (M2c-1 roster, M2c-2a1 families/loadout and M2c-2a2 engine pieces done; M2c-2b content next) | S1, S12 | Loses balance-ready until the rework is built and re-frozen |
| S3 | Gear and inventory: requirements, tiers, burden, carry, stash | done (gear budget guard added in M2d) | S1 | Gear budget per class is an open balance lever |
| S4 | Enemy AI (roles, presets, targeting) | done (M1: kiting, no idle turns, support heals, reckless profile) | S1 | Shaman/ranger AI wastes turns: candidate fix |
| S5 | Content catalog + Content Forge | done | - | Data pipeline for S2/S3/S6 |
| S6 | World engine + World Forge (lore layer, inheritance) | done | S5 | Characters, factions, items, quests as data |
| S7 | Party and progression: persistent unit state (spend, learned skills, passives), level-up UI, XP | proposed | S2 | First gameplay-facing gap |
| S8 | Quests and lore runtime: quest state, rewards granting passives/skills/items, flags | proposed | S6, S7 | Sources of passives |
| S9 | Region map and exploration: tiles, locations, encounters triggers, interactions | placeholder | S6, S8 | README System 2 |
| S10 | World map and travel: routes, travel events, supplies | placeholder | S9 | README System 1 |
| S11 | NPCs and dialogue: dialogue trees, shops, factions reputation | placeholder | S8 | README System 9 |
| S12 | Advanced combat: combos, terrain effects, status expansion, skill upgrades | in-progress (S12-A engine done in M1: ally-target support skills, gap-close, haste/bleed/taunt; terrain and combos remain; riposte done in M2a) | S1, S2 | README System 6 |
| S13 | Save/load and game state, new-game flow, main menu | placeholder | S7 | Needed before any playable loop |
| S14 | Audio, animation and asset pipeline, asset-load tests | placeholder | - | README Test Suite 2 |
| S15 | Balance tooling and reports | done | S1, S2 | `tools/balance_sim.gd`, `docs/BALANCE_REPORT.md` |

## Proposed milestone order (planner refines; user approves each)

Superseded: the M1-first "balance pass" order (see HISTORY.md). Current order is the "Suggested build order" at the end of this file; the balance pass comes after the balance-relevant rules are frozen. Nothing below is `approved` until the user says so.

## Decisions taken from the user's answers (2026-10-15; also in DECISIONS.md)

S12 stays additive (stat inputs or a wrapper; S1 keeps balance-ready). Level-up: automatic in sims (test variants), player-allocated in game, choices flavour only. Gear budget may be broken by special items that add a playstyle or offset a con. Encounters: step counter for mobs with rare elites; elites/bosses/scripted fights placed or roaming. Lone-hero: mob-group losses acceptable when under-levelled and improving with level; bosses ~10-20% solo win at equal level, party encouraged. Passives permanent, scripted fights may grant temporary ones. One small campaign first.

## Open questions for the user

- Gear budget: exact rule (what counts, how special items are tagged and tested). Context sent 2026-10-15.
- Travel supplies: wanted, or reduced resource version? Context sent 2026-10-15.
- Support identity: replace Shaman with Priest/Paladin, keep Shaman with a support kit, or make a hybrid? Skill-coverage review sent 2026-10-15.
- Enemies in worlds: own tier tables or the one base curve? (current decision: one base curve)

## Proposed designs (brainstorm pass 2026-10-15; all `proposed`, nothing approved)

Input: the user wants remaining systems designed and built BEFORE any balance pass; only S1/S2 are balance-ready. No sims were run. The BALANCE_REPORT baseline (warrior 100%, berserker 80%, ranger 20%, shaman 0%, boss/1v3 losses) is used only to flag which systems touch balance. HISTORY.md checked: nothing below re-proposes a deprecated idea (mastery, free point allocation, per-world tiers and hard-coded content are all avoided; passives still come only from sources).

### Balance impact map (what must be frozen before the balance pass)

| System | Moves balance? | Why (baseline link) | Freeze before balance? |
| --- | --- | --- | --- |
| S3 gear budget | Yes, strongly | Warrior armor 25 / HP 314 comes from plate + kite shield; shaman has no body armor item | Yes: gear sets per class |
| S4 AI | Yes | Ranger/shaman waste turns; skews duel and 1v3 numbers | Yes: presets and kiting rules |
| S12 combat additions | Yes, strongly | New statuses, ally heals, terrain, skill upgrades change every class's power | Yes: ship the rules engine; content stays tunable |
| S7 progression | Yes | Spend pool, skill acquisition and XP curve set power at each level; duels are tested per level | Yes: curve and skill-unlock levels |
| S8 rewards | Indirect | Passives/items granted as rewards add power outside class budget | Freeze reward budget rules, not content |
| S10 supplies, S11 shops | Indirect | Gear access and consumables change real-play power | Freeze rules only; tune prices later |
| S9, S13 | No (S9 only via encounter frequency) | Plumbing / flow | No |

Conflict to resolve: S1 is `balance-ready` yet S12 extends S1. Recommendation: S12 is additive (new command/status types, no change to the `DamageCalculator` formula); if the formula must change, S1 loses `balance-ready` until re-frozen (Q1).

### P-S3 Gear budget as a design lever
- Option A, per-class gear budget (soft): define a target gear value per level (armor + power points) and validate each class's starter and best-in-slot loadout stays inside it. Cheap (validation in `ContentCatalog`); the proficiency dict stays the access gate. Risk: the budget number must be invented.
- Option B, fix by content: add missing items (shaman light robes, ranger leather) and gate plate/kite shield by proficiency level. Cheapest, no new rule; but balance stays fragile as items are added.
- Option C, armor burden tax: raise burden of plate so the warrior trades speed for armor (existing rule `Speed = 8 + DEX - burden`). Risk: warrior speed may swing too far.
- Recommend A+B (budget validation as a guard, B fills gaps now); C only if the balance pass still shows warrior dominance. Needed before balance: yes.

### P-S4 Enemy AI
- Option A, fix turn waste only: skip a turn only when no reachable target, kite to max range then attack instead of oscillating, shaman casts before moving. Small; targets the ranger/shaman baseline.
- Option B, A plus role/difficulty profiles per encounter (cautious / standard / ruthless: flee threshold, focus fire, target choice). Medium; delivers the README "difficulty profiles" item as a non-stat difficulty dial.
- Option C, perception-reactive AI (reads telegraphed plans). Big; risks unreadable outcomes for the sim.
- Recommend A now, B after S12 (support AI needs ally-target commands), C deferred. Needed before balance: A.

### P-S7 Party and progression
- Option A, XP + automatic spend: levels from XP; spend follows class weights, skills follow `skill_path`. Smallest; no player agency; fits StatRules (spend exactly 4+level-1).
- Option B, player-allocated spend at level-up inside the enforced pool, with undo until confirm. The pool stays exact and enforced, so the HISTORY row on free allocation is not revived. Medium; needs persistent member state (level, xp, spend, learned skills, passives, loadout).
- Option C, B plus skill/passive slot choice (talent-tree-lite). Large; overlaps S12 skill upgrades.
- Recommend B, built as a pure data model first (`scripts/progression/`, XP curve in `data/progression/`), UI after S13 so it persists.

### P-S8 Quests and lore runtime
- Option A, linear quest state machine: accept, steps, turn-in. Step kinds: defeat encounter, reach location, talk to NPC, have item. Rewards: XP, items, passive, skill, flag.
- Option B, A plus a global flags registry and conditions (any step, dialogue or encounter can set/test flags). Small extra; S9/S11 reuse it.
- Option C, event-bus reactive quests. Over-engineered for now.
- Recommend B. Reward rule to freeze: reward passives/items carry a cost class tied to the S3 gear budget so rewards cannot exceed class budget by much (Q4). Passives stay source-granted only (DECISIONS).

### P-S9 Exploration
- Option A, README spec as written: SAFE/DANGER tiles, danger step counter rolls encounters, roaming visible elites, exit tiles.
- Option B, no random rolls: all encounters placed/visible (authored in `encounters` data). Removes RNG from real-play balance, simpler tests.
- Option C, hybrid: placed encounters plus a step counter only in designated DANGER zones.
- Recommend C (data already marks where fights trigger). No balance impact except encounter frequency; build after S8 (flags/triggers).

### P-S10 Travel
- Option A, node graph with fixed travel cost, no events. Minimal.
- Option B, A plus road-danger roll and supplies. Adds an economy that touches balance indirectly.
- Option C, free travel between visited nodes, supplies as flavor only.
- Recommend A first, supplies (B) as a later slice if wanted (Q6).

### P-S11 NPCs and dialogue
- Option A, dialogue trees as data (speaker, line, choices, flag conditions, effects that set flags/start quests); shops as a dialogue effect with fixed stock per NPC.
- Option B, A plus faction reputation meters (relations already authored) gating dialogue, prices and quests.
- Option C, skill-check choices (stat tests). Adds randomness; defer.
- Recommend A, then B once S8 flags exist. Prices tuned after balance.

### P-S12 Advanced combat
- Option A, content-first within the current engine: ally-target commands (heal/buff), status set (bleed, poison, stun, haste, Staggered) through the existing status system, skill upgrades as per-skill levels. Support AI (S4) rides on ally targeting.
- Option B, A plus terrain rules (cover, movement cost, hazard tiles; `Terrain` metadata exists) and AoE patterns.
- Option C, A+B plus combo/flow engine (chained stance transitions, Dual Strike, weapon switch).
- Recommend A, then B as a second slice, C last and optional. Each slice keeps `DamageCalculator` as the only formula. Give the shaman a support/heal identity here (baseline 0%) rather than inflating Hex/Arcane Bolt numbers (Q7).

### P-S13 Save/load and game state
- Option A, single JSON save slot: party state (S7), flags (S8), inventory/stash, location, world id; versioned schema with a migration hook.
- Option B, multiple slots plus autosave on location change and after combat.
- Option C, A with dojo/forge sandbox saves kept separate.
- Recommend A as first slice, growing to B; build right after the S7 data model so the schema is not retrofitted; new-game flow picks world and starting party.

### Suggested build order (each step needs user approval; balance only after step 7)
1. S7 data model (state, XP curve) + S13 save schema (together they define the party snapshot).
2. S12-A (ally targeting, statuses, skill upgrades) with S4-A (turn-waste fixes, support AI); S3 gear budget validation + missing class items.
3. S8-B quest/flag runtime (rewards via budget rules).
4. S7 level-up UI + S13 menus/new-game flow.
5. S9 exploration, S11-A dialogue/shops, S10-A travel.
6. S12-B terrain/AoE, S11-B reputation, S4-B profiles.
7. Freeze S3 gear sets, S4 presets, S12 rules, S7 curve, S8 reward budget; mark balance-ready; then the balance pass (supersedes the M1-first ordering above).
Until step 7, only S1/S2 stay balance-ready and no balance work is done.

### Open questions raised by this pass
Q1 May S1 lose `balance-ready` if S12 needs a formula change? Q2 Gear budget: soft validation (A) or content-only (B)? Q3 Player-allocated level-up (S7-B) or automatic? Q4 May quest rewards exceed class budget, and by how much? Q5 Exploration encounters: placed only, step-counter only, or hybrid? Q6 Are travel supplies wanted? Q7 Should the shaman get a support/heal identity? Q8 Are lone-hero 1v3 and boss losses intended (party fights expected)? Q9 Passives permanent or temporary (existing question)? Q10 Playable loop scope: one campaign or a world sandbox?

## Refactor backlog (separate from feature milestones; never part of a builder's feature task)

| Id | Item | When | Rules |
| --- | --- | --- | --- |
| R1 | Split oversized test files: `arena_test_suite.gd` (~1400 lines, older groups: combat UI, character specs, gear/dummy, session flow, art/terrain, planning/AI) into one file per group with a small runner; later `combat_extensions_test_suite.gd` (~800) by feature and `dojo_assertions.gd` (~770) | after M2c merges | Pure refactor on its own branch and review; no behaviour or number changes; total assertion count before = after |

## Open engine items (tracked)

- AI hooks: TacticalAI ignores DISENGAGE, SWEEP (area) and cleanse-only skills (needed if enemies should use them) - schedule with the AI follow-ups or M2c-2b.
- Disengage "once per round" and "after a MOVE" are not enforced/modelled (same as plain MOVE).
- Skill-tier picker is not rendered in the planner UI yet (grouping API exists).
