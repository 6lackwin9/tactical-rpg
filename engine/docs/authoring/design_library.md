# A game's design library

Status: canon (user 2026-10-25). The layout every game's design library follows, in `games/<id>/docs/lore/`.

| File | Holds |
| --- | --- |
| `README.md` | The catalog of the library and a quick-reference table; process rules point to CLAUDE.md |
| `CANON.md` | The single source the other docs follow: numbers sheet, campaign spine, endings, glossary, open questions |
| `background/cast_registry.md` | Every named character and place: name, stable id, faction, role, age, background, first appearance, asset brief |
| `background/central_truth.md` | The spoiler sheet: what is really going on; options until the user picks one |
| `background/`, `narrative/`, `systems/`, `specs/` | Domain docs (world, story, rules, art and roadmap), each consistent with `CANON.md` |
| `specs/naming_and_asset_guide.md` | The game's filled-in naming styles and asset brief rules ([template](naming_and_asset_briefs.md)) |
| `archive/` | Superseded brainstorm exports, kept with an archive header (history is append-only) |

## Doc rules

- A `Status:` line on every doc: `canon` (consistent with `CANON.md`), `proposal` (later acts or systems, not locked) or
  `archive`.
- Repo-relative links only.
- One name per thing: the glossary in `CANON.md` lists the retired variants of every term.
- Every feature line tagged with its layer ([layer_rule.md](layer_rule.md)).
- Engine claims checked against the code; corrected where wrong.
- Plain statements: no self-praise or marketing tone.
- Markdownlint-clean (`bash tools/fc.sh docs`) and at most about 300 lines / 25 KB per doc: split big ones behind an
  index.
- Every doc ends with its gaps: undefined things with 1-2 options and a recommendation. Gaps are collected for the user;
  agents do not invent answers.

## The canon sheet

`CANON.md` holds, in this order:

1. Title and one-paragraph premise.
2. Numbers sheet: every gameplay constant, with its engine-scale equivalent where the engine's formulas differ.
3. Campaign spine: prologue and acts in order (missions, nodes, encounter ids, who joins, key choice, act transitions).
4. Endings and fail states, with what gates each ending.
5. Glossary: one name per term, retired variants listed.
6. Open questions: what the user still decides.
