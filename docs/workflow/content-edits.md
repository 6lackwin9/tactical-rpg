# Content edits (agent-driven)

How game content (skills, items, classes, units, passives, encounters, world data) changes now that the creation tools are read-only viewers (decision "Content edits (user 2026-10-17)", `docs/decisions/workflow.md`; tools: `games/emberwake/docs/decisions/content-forge.md` "Read-only tools"). Agents make every edit; the user controls what is wanted, approves anything new, and can undo.

## Flow

1. **Ask in plain words**, in the project chat or the matching area thread: "the archer hits too hard at range", "add a fire sword for act 1", "make the skirmisher flee earlier". The project chat routes it to the area thread (Balance for numbers, Assets & Audio for data rows, Gameplay for behaviour, Design for new ideas).
2. **The thread sorts it** as a small tweak or new content (rules below).
3. **Small tweak**: the asset-builder edits the data through `aegis content` (an upsert or a JSON seed, never raw JSON), the writer's validation and the gate catch bad values, and the change lands on `develop` like any task. The thread's reply carries a before/after table of the changed fields, plus sim numbers when the sim rule below applies.
4. **New content**: the design-brainstorm agent writes a short spec (what it is, numbers, where it appears), the thread posts it as one approval card, and building starts on approval. Data-only additions go to the asset-builder; anything needing new engine behaviour adds a logic task first.
5. **Review in the game**: the user checks results in the read-only viewers (Scene Runner) or the itch.io build after the next release, not in the files.
6. **Undo**: "undo that" means the thread reverts that task's squash commit on `develop` (`git revert`, never a force push) and says which commit it reverted; if the change already reached `main`, the revert ships with the next release.
7. **Playtest notes**: tweaks jotted in chat while playing are batched per area: the project chat forwards them to each area thread as one list, and the thread lands them as one task with one table.

## Small tweak or new content

| Small tweak (no approval card) | New content (spec + one approval card) |
| --- | --- |
| Changing values of existing rows: damage, range, cost, cooldown, HP, stats, prices, drop weights, magnitudes, AI thresholds already in data | A new item, skill, passive, class, unit, enemy, encounter, status or world location |
| Renames, descriptions, icon keys that already exist | A new mechanic, field or rule (also needs a logic task) |
| Moving an existing row between encounters, shops or tiers | Removing a shipped row, or changing what a skill or item does (its effect type, targeting or area) |
| Up to about 5 rows in one request | A request touching more than about 5 rows, or a whole family rebalance |

When unsure, treat it as new content. Design or gameplay questions always go to the user first (CLAUDE.md feedback rule).

## When a sim run is required

The balance agent runs `tools/fc.sh sim duel` before landing when a tweak changes a balance-relevant value of a unit, class, skill, item or passive used in combat: damage, healing, HP, armor, speed or movement, range, cost, cooldown, status magnitude or duration, gear budget, or AI thresholds, and puts the before/after numbers in the reply. Only the duel mode exists on the engine so far; a tweak that needs another mode (`tier`, `party`, `role`) says so and waits for that mode's port (a logic task). Text, icons, prices, drop weights and encounter placement need no sim. Systems not yet `balance-ready` (`games/emberwake/docs/ROADMAP.md`) skip the sim and say so.

## Rules

- Agents edit content only through `aegis content` (upsert, remove, `apply` of a JSON seed in `games/emberwake/data/seeds/`); the viewers never save (TOOLS-RO).
- Every save-time check the old tools ran is a writer check, a test or a gate check, so a bad value fails the gate, not the review. Checks the Godot Forge ran that the engine writer does not run yet (skill validator details, gear budget warnings, world reference rules) are listed in `engine/docs/PORT_NOTES.md` "Data writer"; reviewers check those by hand meanwhile.
- One request is one task with one before/after table; the commit lines are the usual ones (`cloud.md` "Landing a task on `develop`").
