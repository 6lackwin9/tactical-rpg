# Roadmap (agent-facing, high level)

Single source of truth for scope and status. Detailed per-system checklists remain in README "Roadmap". Do not delete entries: change the status and add the reason (see `docs/HISTORY.md` for deprecated ideas).

**Status values**: `done` (implemented, tested) | `balance-ready` (done and frozen: the balance agent may fine-tune it) | `in-progress` (a milestone is building it) | `approved` (milestone approved by the user, not started) | `proposed` (idea from brainstorm, not approved) | `placeholder` (folder reserved in `docs/STRUCTURE.md`, nothing built) | `deprecated` (kept for history; see HISTORY.md).

Only the user moves an item to `approved`. Only the planner (main session) sets `in-progress` and `done`. Only the user/planner set `balance-ready`.

## Systems

| # | Area | Status | Depends on | Notes |
|---|---|---|---|---|
| S1 | Combat core: block resolver, planner, stances, AP, damage formula, statuses | balance-ready | - | `DamageCalculator` is the only formula |
| S2 | Classes, units, tiers, proficiency, passives, skill paths (six classes) | balance-ready | S1 | Class balance is out of the 40-60% target; see BALANCE_REPORT |
| S3 | Gear and inventory: requirements, tiers, burden, carry, stash | done | S1 | Gear budget per class is an open balance lever |
| S4 | Enemy AI (roles, presets, targeting) | done | S1 | Shaman/ranger AI wastes turns: candidate fix |
| S5 | Content catalog + Content Forge | done | - | Data pipeline for S2/S3/S6 |
| S6 | World engine + World Forge (lore layer, inheritance) | done | S5 | Characters, factions, items, quests as data |
| S7 | Party and progression: persistent unit state (spend, learned skills, passives), level-up UI, XP | proposed | S2 | First gameplay-facing gap |
| S8 | Quests and lore runtime: quest state, rewards granting passives/skills/items, flags | proposed | S6, S7 | Sources of passives |
| S9 | Region map and exploration: tiles, locations, encounters triggers, interactions | placeholder | S6, S8 | README System 2 |
| S10 | World map and travel: routes, travel events, supplies | placeholder | S9 | README System 1 |
| S11 | NPCs and dialogue: dialogue trees, shops, factions reputation | placeholder | S8 | README System 9 |
| S12 | Advanced combat: combos, terrain effects, status expansion, skill upgrades | proposed | S1, S2 | README System 6 |
| S13 | Save/load and game state, new-game flow, main menu | placeholder | S7 | Needed before any playable loop |
| S14 | Audio, animation and asset pipeline, asset-load tests | placeholder | - | README Test Suite 2 |
| S15 | Balance tooling and reports | done | S1, S2 | `tools/balance_sim.gd`, `docs/BALANCE_REPORT.md` |

## Proposed milestone order (planner refines; user approves each)

1. **M1 Balance pass on S1/S2/S3/S4** (gear budget, shaman, warrior dominance) -> brings classes into 40-60%.
2. **M2 S7 Party and progression** (state model + level-up flow, data only first).
3. **M3 S13 Save/load and game state** (so S7/S8 persist).
4. **M4 S8 Quests/lore runtime**.
5. **M5 S9/S10 exploration and travel**, **M6 S11 NPCs/dialogue**, **M7 S12 advanced combat**, **M8 S14 polish**.

## Open questions for the user
- Should enemies in worlds be able to define their own tier tables, or stay on the one base curve? (current decision: one base curve)
- How should passives be earned/lost in play (permanent vs. temporary)?
- Target platform and scope of the "playable loop" (single campaign vs sandbox of worlds)?
