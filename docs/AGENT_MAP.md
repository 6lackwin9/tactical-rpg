# Agent map (planner-maintained dashboard)

The harness agent list cannot be regrouped, so this file is the readable view: active runs on top, finished runs grouped by milestone. The planner updates it when it dispatches an agent and when one reports. Tokens and minutes are from each agent's own report. Commit hashes are in `git log`.

## Active now

| Task | Agent | Lane | Worktree and branch | Started |
| --- | --- | --- | --- | --- |
| C1-L orb/tome schema, item effects | logic-builder | logic | `ngp-c1l` / `feat/c1l` | 2026-10-16 |
| D1-L weaken status, riposte tiers | logic-builder | logic | `ngp-d1l` / `feat/d1l` | 2026-10-16 |

Waiting to merge: C1-A1 (asset-builder, done, worktree `ngp-c1a1` / `feat/c1a1`; merges after the logic branches).

## Finished, by milestone

### Process and roadmap

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm (general-purpose) | Roadmap proposals | 56k / 1.5 | Proposed designs in ROADMAP |

### M1 combat extensions

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Support skills, gap-close, haste/bleed/taunt, AI fixes | 300k / 23 | merged |
| change-reviewer | Review | 30k / 2.5 | found stale reset bug (fixed) |

### M2a riposte

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Riposte engine | 128k / 9 | merged |
| change-reviewer | Review | 33k / 2 | merged |

### M2d gear budget

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | GearBudget guard | 66k / 8 | merged |
| change-reviewer | Review | 24k / 2 | merged |

### M2b items and art

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Eight items, placeholder art keys | 68k / 4 | merged (no reviewer) |

### M2c-1 classes

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Six-class roster data | 109k / 12 | merged |
| design-brainstorm | Skill draft v1 | 41k / 2 | docs |
| change-reviewer | Review | 29k / 2 | merged |

### M2c-2a1 skill families

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Families, tiers, loadout | 126k / 7 | merged |
| change-reviewer | Review | 37k / 2 | found rename hazard (fixed) |

### M2c-2a2 engine pieces

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Cleanse, disengage, sweep, unarmed budget | 188k / 13 | merged |
| change-reviewer | Review | 51k / 3 | merged |

### M2c-2b design

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | Skill draft v2 | 51k / 3 | docs |
| design-brainstorm | Revision 3 proposals | 47k / 2.5 | docs |
| design-brainstorm | C1, D1, M2C2B specs | 131k / 7 | docs/specs |

### Slice A foundation

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Basics cleanup, registry, unarmed, hands | 213k / 20 | merged |
| change-reviewer | Review | 51k / 4 | merged |

### B1 rest to Tactical (first parallel pilot)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Rest to Tactical, stance constant | 47k / 10 | merged (no reviewer) |
| design-brainstorm | Specs C1, D1, M2C2B (ran in parallel) | 131k / 7 | docs/specs |

### C1 orb and tome

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | C1-A1 gating, priest proficiency, tome retype | 23k / 2 | done, waiting to merge |
| logic-builder | C1-L | - | active |

### D1 weaken and riposte

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | D1-L | - | active |
