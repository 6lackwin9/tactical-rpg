---
name: consistency-audit
description: Run a periodic whole-codebase Design-Session consistency audit across (1) all docs/specs/schema/tools vs. build, (2) three-layer boundaries (engine -> kit -> game), rule-engine purity, and Guiding Principles (docs/PRINCIPLES.md), and (3) lore/worldbuilding (CANON.md) vs. shipped JSON data. Distinct from audit-build (which audits only recent landed Build commits). Archives a dated, commit-stamped report in docs/audits/ and queues slices in docs/READY.md.
---

# Whole-Codebase Consistency Audit (`/consistency-audit`)

Use this skill in the **Design Session** for a **periodic full-repository alignment pass** at a snapshot date and commit (`@<short-sha>`).

## How `/consistency-audit` Differs from `/audit-build`

- **`/audit-build` (Incremental Commit Audit)**: Reviews **only recent landed Build commits** (`git log` range since the last batch) against their `docs/READY.md` task rows to catch slice regressions and unlogged Builder commit drift.
- **`/consistency-audit` (Whole-Codebase Snapshot Audit)**: Scans the **entire repository** end-to-end regardless of when code or docs were written, checking cross-system drift across specs, `docs/PRINCIPLES.md`, `schema.json`, `tools/`, all three layers (`engine/`, `kits/`, `games/`), and `CANON.md` / `docs/lore/` vs. `data/`, and saves a permanent dated/commit-stamped report under `docs/audits/`.

## Workflow

1. **Sync & Pin Snapshot**:
   - Fetch and rebase onto `origin/develop` (`git pull --rebase origin develop`).
   - Record the UTC date (`date -u +%Y-%m-%d`) and short commit SHA (`git rev-parse --short HEAD`).
   - Read open items in `docs/FEEDBACK.md`, `docs/PRINCIPLES.md`, and `games/emberwake/docs/lore/CANON.md` so landed `docs-delta` / `sim` notes and `[PROVISIONAL]` calls are accounted for accurately.

2. **Dispatch Three Read-Only Research Passes in Parallel**:
   - **Pillar 1 — Specs, Docs, Schema, Tooling & Principles vs. Build**:
     - Compare `docs/sessions/*.md`, `docs/specs/*.md`, `docs/PRINCIPLES.md`, `games/<id>/docs/specs/*.md`, `README.md`, `engine/README.md`, `kits/*/README.md`, `GAME_BRIEF.md`, `STRUCTURE.md`, `kits/*/schema.json`, `rpgkit_host`, `emberwake_gui`, and `tools/fc.sh` against workspace Rust crates.
     - Flag implementation bugs, incomplete cross-crate wiring (e.g. rules built in `rpgkit_rules`/`rpgkit_battle` but missing from `rpgkit_host`, `DojoSession`, or `emberwake_gui`), stale `schema.json` help/enums, retired hooks still in code, `P-*` principle drift, and outdated doc statements.
   - **Pillar 2 — Three-Layer Split & Rule-Engine Purity (`engine` -> `kits/<kit>` -> `games/<id>`, `P-ARCH-01..04`)**:
     - **Downward leaks**: Verify `engine/` has zero RPG or game/lore nouns, and `kits/<kit>/` has zero game IDs, world IDs, lore terms, hardcoded catalog unit/item/family/terrain IDs, or non-term-resolved game labels. Audit `tools/fc.sh` `layers_check` (`G1`-`G4`) and `games/<id>/lore_words.txt` for blind spots.
     - **Upward leaks (No Game-Unique Rule Engines)**: Verify `games/<id>/crates/*` implements **no standalone gameplay rules, validation, deck/loadout/pouch/squad logic, hardcoded sim matrices, or English log-string scraping** that belong in `kits/<kit>` rule engines or JSON data tables.
   - **Pillar 3 — Lore & Worldbuilding vs. Shipped Data (`CANON.md`, `P-CAMP-03`)**:
     - Cross-check `CANON.md`, `CAST1_*.md`, and `games/<id>/docs/lore/**` against `games/<id>/data/**`.
     - Separate **clear data-wiring bugs** (passive/item/unit ID typos, missing character rows, wrong `start_act` or meter `visible` flags, dead class loops) from **malleable `[PLACEHOLDER]` / `[PROVISIONAL]` lore mismatches**.

3. **Resolve Open Choices (`ask_question` Interactively, or `docs/PRINCIPLES.md` Section 0 When User Is Away)**:
   - Score every open architectural, scope, or lore/data option against **`docs/PRINCIPLES.md`** (`P-ARCH`..`P-UI`).
   - When the user is present, prompt concise `ask_question` choices with the principle-aligned `(Recommended)` option first; when running asynchronously while the user is away, apply the lowest-undo-cost principle-aligned option as `[PROVISIONAL]` and log a row in `docs/DESIGN_REVIEW.md` (stopping only on hard-stop exceptions).

4. **Archive Report, Queue Build Slices & Update Design Docs**:
   - Write the snapshot report to `docs/audits/<YYYY-MM-DD>_<short-sha>_consistency_audit.md` (and index it in `docs/audits/README.md`), keeping every `.md` file `<= 300` lines, `<= 25` KB, and `tools/md_lint.awk`-clean (wrap any `<placeholder>` in backticks for `MD033`).
   - Queue all code, `schema.json`, `tools/`, and `data/` fixes (which Design never edits directly) as ordered `ready-to-build` slices in `docs/READY.md` Section 1, and answer any open `FB-nn` items in `docs/READY.md` Section 4.
   - Apply approved Design-owned updates directly to `docs/sessions/*.md`, `docs/specs/*.md`, `CAST1_*.md`, `CANON.md`, `docs/lore/**`, `GAME_BRIEF.md`, `STRUCTURE.md`, `HISTORY.md`, and READMEs.
   - Run `python3 tools/planner/feedback_check.py` and `bash tools/fc.sh docs`, commit, and push cleanly to `origin/develop`.
