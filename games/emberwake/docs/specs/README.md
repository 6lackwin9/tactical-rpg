# Specs

One file per task: `docs/specs/<task-id>.md`, written by the `design-brainstorm` agent (or the planner), consumed by the `logic-builder`, `asset-builder` and `balance` agents. Specs are append-only history once a task is done (mark the status, do not delete). In the cleanup pass after a task is done, its spec moves to `docs/archive/specs/` with a first line `Archived <date>, task done/superseded` and the TASKS row points there; rules that later passes still cite are copied verbatim (same section numbers) into a shared spec such as `games/emberwake/docs/specs/FORGE_SHARED.md`.

## Active vs. Archived Specs (`games/emberwake/docs/specs/`)

Active files stay here (`24` specs + this index); completed specs move to `docs/archive/specs/emberwake/` (`CTX-CLEANUP1-DESIGN`, `2026-11-15`):

- **Active Art/UI & Studio (`AI-*`, `/art-ui-session`)**: `ART_UI_REQUESTS.md`, `ART_UI_STUDIO.md`, `SCREEN_ATLAS.md`, `FOUNDER_PORTRAITS.md` (stays active because `.agents/skills/art-ui-session/SKILL.md` and open `AI-04` read it directly), `PROPS1_PROMPTS.md` (stays active because `tools/portraits/test_prompts_sections.py` and `AI-13` read it), `ART_PROMPT_TEMPLATES.md`, `MUSIC1.md`.
- **Active Cast, Region, VFX & Look Specs**: `CAST1.md` + `CAST1_{ASSETS,BESTIARY,BOSSES,ENCOUNTERS,FOUNDERS,GAPS,GEAR}.md`, `TRANSITIONS1.md` (story-battle-hub transitions, `PR7-14`), `BUNKS1.md` (hub bunks and bunk talks, `PR7-13`), `REGIONGROUND1.md` (includes landed `REVEAL1` indoor/roof/boundary rules), `REGIONGROUND1_SCENE.md`, `PROPS1.md`, `LOOK1.md`, `LOOK1_TASKS.md`, `PORTRAITFX2.md`, `FORGE_SHARED.md`, `INTENT2_CASES.md`, `INTENT2_SKILLS.md`.
- **Archived in `docs/archive/specs/emberwake/` (`2026-11-15`)**: `ART1.md`, `ART2.md`, `ART2_ICONS.md`, `ART2_PROMPTS.md`, `ART3_PROMPTS.md`, `PLAYUI1_ICONS.md`, `PORTRAITFX1_FACES.md`, `DIR4.md`, `WINDUP1.md`, `REGIONGROUND1_HISTORY.md` (`§§8-12`), `LOOK1_NOTES.md` (`§§9-11`), `PORTRAITFX2_NOTES.md` (`§13`).

## Template

```markdown
# <task-id>: <title>

Status: proposed | approved | in-progress | done
Depends on: <task ids>
Lane: logic | asset | both
Touches: files or areas this task edits (e.g. `scripts/combat/status/*`, `content_schema.gd`, `data/catalog/skills.json`); the planner runs tasks in parallel only if their Touches do not overlap

## Goal
One paragraph.

## Schema contract (logic lane)
Fields, allowed values, defaults, validation rules. Schema-first: yes/no.

## Rules (logic lane)
Behaviour, formulas, edge cases, acceptance checks (rule-derived).

## Assets (asset lane)
Table of ids, numbers, links to other assets. Authoring rules used (tier defaults etc.).

## Out of scope
What not to touch.

## Open questions
Only genuine design choices for the user.
```
