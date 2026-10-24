---
name: game-design-library
description: Top-down game authoring on Aegis - the layer rule (engine / kit hook / data / game code), a game's design library layout (CANON.md, cast registry, central truth, gaps), naming and asset briefs. Use when starting a new game, writing or reviewing a game's lore/design docs, turning a brainstorm into canon, or naming characters, units, items, skills or places.
---

# Game design library

Read `engine/docs/authoring/README.md` first; it links the three pages below.

1. **Layer rule** (`engine/docs/authoring/layer_rule.md`): tag every feature `[engine]`, `[kit hook]`, `[data]` or
   `[game code]`. Prefer a data stand-in, then a kit hook; game code needs a one-line reason.
2. **Design library** (`engine/docs/authoring/design_library.md`): `games/<id>/docs/lore/` with `CANON.md` (numbers,
   spine, endings, glossary, open questions), `background/cast_registry.md`, `background/central_truth.md`, domain docs
   with `Status:` lines, `archive/`. Every doc ends with its gaps.
3. **Naming and asset briefs** (`engine/docs/authoring/naming_and_asset_briefs.md`): stable ids, full asset briefs, names
   that carry faction and role, distinctness, one registry. The game's filled-in version is
   `games/<id>/docs/lore/specs/naming_and_asset_guide.md`.

Also: `consistency_pass.md` (brainstorm to canon), `art_bible.md` (fix sizes, palette, pipelines first),
`milestones.md` (M0-M4 and throughput per layer). When something you write for one game could serve others, add its
game-agnostic version to `engine/docs/authoring/`.

Checks before handing back:

- Every engine claim verified against the code (brainstorm docs name modules that do not exist).
- No invented design where the canon sheet leaves a gap: list the gap with 1-2 options and a recommendation.
- Repo-relative links, markdownlint-clean (`bash tools/fc.sh docs`), docs at most about 300 lines / 25 KB.
- Design or gameplay decisions go to the user (CLAUDE.md feedback rule).
