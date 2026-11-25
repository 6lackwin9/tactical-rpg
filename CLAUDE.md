# Tactical RPG (Aegis Engine, Rust)

## Working Agreement

- **Division of labor:** The user owns vision, taste, and final calls (plain sentences, no engine jargon required). The agent owns craft, architecture, options, and execution.
- **Default 4-step loop:**
  1. **Options & clarifying questions:** Before any non-trivial change, present 2–3 concrete options in plain English (what each gives, what it costs, and a clear recommendation). Ask targeted clarifying questions when a real creative or design fork exists (suggest `/grill-me` when fleshing out a new game concept).
  2. **Autonomous execution inside scope:** Once a direction is picked, handle research, subagents, implementation, tests, and pre-merge review end-to-end without nagging on micro-decisions. Stop and ask only for genuine design forks, irreversible actions, or after two failed targeted fixes.
  3. **Concrete proof:** Report back in plain sentences with tangible results (timings, test counts, before/after shots, or playable steps) plus any `[PROVISIONAL]` calls made.
  4. **Streamline:** Record what worked vs. what felt clunky in the Experiment Log below, and drop or simplify rules that do not earn their keep.
- **Batch Asset Review (no micro-approvals):** Lock **1 Style Anchor** per asset category (maps, props, portraits, battle backdrops). Agents auto-gate and auto-reroll batches of 5–10 assets against numeric checks (`aegis align-map check-ref` / `score`), then show **one visual contact sheet** (`aegis align-map batch-sheet`) per batch for exception-only sign-off.
- **Sprite / Portrait Pipeline (`SpriteForge` Option A):** Keep `SpriteForge` as a standalone, profile-driven browser studio for interactive Style Anchor exploration; put the headless production pipeline (auto-chroma, studio keyer + despill, foot-aligned `112x96` slicer, `check-sprite` drift QA, Drive batch queue, and `batch-sheet`) in `engine/` (`aegis_cli`).
- **6-Step Main-Release Review Loop:**
  1. Agents implement and run full headless QA on `develop`, then release to `main` and trigger Web + Windows release builds (`.github/workflows/release-build.yml` & `web.yml`).
  2. The user plays and reviews **only the `main` release build** and shares feedback + feel checklist notes.
  3. Feedback is broken into scoped tasks and built on `develop`.
  4. UI/UX changes iterate via lightweight interactive mocks (`tools/mock_ui/` / HTML previews) first to lock guidance before applying to target UI, assets, or an Act-wide theme.
  5. Record workflow and development learnings in the Experiment Log below so future games inherit what worked.
  6. Once all feedback tasks land, run a consolidated QA pass on `develop` and release to `main` for the next playtest iteration.
- **Current product focus (2026-11-25):**
  - **Milestone 1:** Polished, playable **Act 1 Vertical Slice** (Windows + Web) before expanding Acts 2–3.
  - **Map rendering:** Modular engine tiles + `MapFx` zone grading & light pools driven by 3-stage AI target refs (`L1` cleanplate -> `L2` propped -> `L3` lit).

## Sessions & Parallel Lanes

- **Scoped sessions are self-contained:** A cloud or desktop session working inside a user-approved scope may edit code, tools, and docs together on a clean feature branch and fast-forward push to `origin/develop` once verified. The older single-writer Design vs. Build vs. Art session walls (`docs/READY.md`, `docs/FEEDBACK.md`, `build-lease`, `.github/workflows/feedback-watch.yml`) remain available as **optional async handoff tools** when running separate async sessions — not mandatory locks for a single scoped session.
- **Parallel lanes & subagent model tiering:**
  - In the cloud, **separate conversations** are true dedicated lanes (each gets its own isolated VM, CPU, Cargo `target/`, and branch).
  - Inside one conversation, subagents share one VM and working tree: run research, audits, and disjoint file edits in parallel, and serialize Cargo builds/tests and git merges through the coordinator.
  - **Model tiering (all at session effort):** use `flash` for read-only audits, codebase lookups, CI log triage, and doc syncs; `inherit` for scoped implementation and coordinator reviews; `pro` only for deep architectural redesigns or complex multi-crate bugs.

## Layers

Strict 3-layer separation (`docs/specs/SPLIT1.md`, enforced by `tools/fc.sh step layers`):

- **Engine (`engine/`, `aegis_*`):** Completely game-agnostic; never names any game, world, or lore word.
- **RPG Kit (`kits/rpg/`, `rpgkit_*`, `schema.json`):** Genre rules and shared RPG data contracts; names no game or class-specific lore.
- **Games (`games/<id>/`):** `games/emberwake/` (reference game: `emberwake_*`, `data/`, `assets/`, `tuning.json`, `docs/`), `games/hello/` (engine sample), `games/kit_sample/` (kit sample).
- **Frozen archive (`legacy/`):** Pre-Rust Godot 4.7 reference (`legacy/LEGACY.md`). Never edited; no build, test, or tool reads it.

## Standing Guardrails

- **Clean build & gate:** Zero compiler warnings. Run `tools/fc.sh gate` (cloud/CI; `tools/fc.sh gate fast` or `check_project.ps1 -FastLocal` for fast local loops) and `tools/fc.sh step docs` + `layers` before pushing to `origin/develop`.
- **Branches & releases:** Feature work lands on `origin/develop` via clean fast-forward/squash commits (never force-push `develop` or `main`). Release to `main` only when the user explicitly says `"release now"`.
- **Game data:** Edit `games/*/data/` only through `aegis content` (`tools/fc.sh content`) and `fields`-scoped seeds (`ENGINE-20`), never raw JSON hand-edits.
- **Missing assets & image checks:** Land features needing unbuilt art with placeholder copies replaced in place later. Whenever generating or editing an image asset, visually inspect it against the prompt and reference before accepting.
- **Known-issues & no thrashing:** Check `known-issues` first; after two failed targeted fixes, stop and present findings + options. Docs history (`docs/HISTORY.md`) is append-only; no `sed` line-range edits.

## Engine goldens

The Godot parity suites were retired 2026-11-01 (`legacy/` is not a test oracle). The engine goldens are: `aegis_core` `rng_hash`, `emberwake_rules` `rules_golden`, `emberwake_sim` `sim_golden`, `emberwake_content` `checks_golden`, `rpgkit_host` `rules_contract_golden` (`BASERULES1-M`), the `emberwake_world` journey, the store byte-for-byte resave, the timeline golden, and the `PHASEFOLD` golden. Never re-record a golden just to make a test pass — re-record only for an intended change named in the commit and `docs/HISTORY.md`.

## Key Entry Points

| Read | Purpose |
| --- | --- |
| `docs/STRUCTURE.md` | Where crates, tools, assets, and docs live by layer |
| `games/emberwake/docs/GAME_BRIEF.md` | Active rules, numbers, and system contracts |
| `docs/PRINCIPLES.md` & `docs/DESIGN_REVIEW.md` | Guiding principles and `[PROVISIONAL]` design calls |
| `games/emberwake/docs/lore/CANON.md` | Worldbuilding canon (`[LOCKED]` vs. `[PROVISIONAL]`) |
| `docs/READY.md` & `docs/FEEDBACK.md` | Optional async queue & build status handoff |
| `docs/DECISIONS.md` & `docs/HISTORY.md` | Settled decisions index and append-only tried-ideas log |

## Workflow Experiment Log

| Date | Experiment | Outcome | Decision & Why |
| --- | --- | --- | --- |
| 2026-11-25 | Strict Design vs. Build single-writer locks in every session | Felt heavy / slowed solo iteration | **Relaxed** to optional async handoff; scoped sessions now edit code + docs cleanly together |
| 2026-11-25 | Gemini Scheduled Tasks for feedback watching | Burned quota on idle polls | **Replaced** by `.github/workflows/feedback-watch.yml` (10s push/schedule check + single rolling issue) |
| 2026-11-25 | Parallel fail-fast gate + prebuilt reuse + `aegis_ui` test consolidation (54 -> 4 binaries) | Pre-build batch 191s -> 6.7s; `layouts`/`repro` <0.5s | **Kept**; expanding test-binary consolidation to `kits/rpg` and `emberwake` |
| 2026-11-25 | Read-only research subagents -> focused implementer subagent -> coordinator review before merge | Caught 3 process-global test hazards + 7 `ENGINE-20` gaps with zero collisions | **Kept** as default pattern for parallel work inside a single VM |
