# Cloud Session Handover — Engine & Framework 4-Pillar Build

**Date:** 2026-11-25
**Branch:** `origin/develop` (`1c125809` + this handover)
**Mode:** Engine/framework development using `games/emberwake` as the live reference game (**game-specific content edits on hold** — shipped `games/emberwake/assets/` and `games/emberwake/data/` kept clean).

## 1. What Is Landed & Verified

### Engine & CLI (`engine/`)

- **`aegis_ui::map_fx`** (`engine/crates/aegis_ui/src/map_fx.rs`):
  - Game-agnostic per-map presentation sidecar (`map_fx/<map_id>.fx.json`) over layered VFS with live hot-reload (`ENGINE-64`).
  - Doorway-feathered per-zone floor/wall color & luminance grading (`ZoneGrade`), applied cleanly after the static floor/wall pass (before standing props/bunks/characters so sprites keep true colors), plus optional `dim_ambience` / `LightSpec` light-pool overrides.
- **`aegis align-map`** (`engine/crates/aegis_cli/src/align.rs`):
  - `refspec [stage=full|cleanplate|propped|lit]` — deterministic `<id>.refspec.json`, stage-specific `.prompt.txt`, and `.blockout.png`.
  - `check-ref [stage=...] [prev_ref=...]` — piecewise-linear grid registration, aspect fit, orthographic H/V edge ratio, wall/floor contrast, zone material/luma ordering, `floor_lighting_evenness`, inter-stage `stage_drift_non_prop` (`|L2 - L1|` + `prop_diff_mask.png`), and `stage_structure_preservation` (`L3 / L2` + `lighting_ratio_map.png`).
  - `enqueue [stage=...] [prev_ref=...] [session_ref=...] [audit=...] [dry_run=1]` — writes `artqueue.py`-compatible Drive requests and auto-syncs `<pipe>/refs/active/` to mirror **only** pending `queue/*.json` refs.
  - `layers (ref=... | ref_l1=... [ref_l2=...] [ref_l3=...])` — generates isolated 4-layer comparison sheets (`L0 Shell`, `L1 Ground`, `L2 Props |L2-L1|`, `L3 Lighting L3/L2`), stacked `layer_sheet.png`, and `layers_report.json`.
  - `score`, `patch` (`ENGINE-20` `fields`-scoped `aegis content` seed emitter with walkability BFS + wall-prop lock), and `run` (closed-loop Pass 2 `MapFx` solver).

### Reference Game Hook (`games/emberwake/`) & Queue Tooling (`tools/`)

- **`emberwake render_map`** (`games/emberwake/crates/emberwake_gui/src/region.rs`, `lib.rs`):
  - Pure CPU headless full-map PNG renderer supporting `layers=shell|ground|unlit|full`, `fx=<path|none>`, `map_override=<json>`, `view=square|tilted`.
  - `ASSETORG2` layout validator registers `map_fx/*.fx.json`.
- **`tools/portraits/artqueue.py`**:
  - Auto-syncs `refs/active/` on `enqueue`, `attach`, and `ingest` (`36/36` unit tests passing).

### Verified Prototype Results (Pre-Handover)

- **Undercroft v2 Pass 2:** Mean zone `ΔL` `13.98 -> 1.12` (`-92.0%`), `ΔRGB` `28.00 -> 6.99` (`-75.0%`); prototype sidecar archived outside shipped assets so Emberwake bunks/scenario goldens stay untouched.
- **Runoff Grate 3-Stage Zero-Upload Proof:**
  - Stage 1 (`cleanplate`, Gemini Task zero-upload): `5 PASS / 0 WARN / 0 FAIL`, `ortho_hv = 7.84`, `floor_lighting_evenness std = 10.63`.
  - Stage 2 (`propped`, Gemini Task zero-upload): `6 PASS / 0 WARN / 0 FAIL`, `stage_drift_non_prop = 1.97` ΔRGB, crisp `|L2 - L1|` prop mask.
  - Stage 3 (`lit`): `5 PASS / 0 WARN / 0 FAIL`, `edge_correlation_vs_l2 = 0.93`, clean `L3 / L2` lighting ratio map.

## 2. Cloud-Ready Image Pipeline State (`My Drive/image_pipeline/`)

- **Google Drive Plugin (`gdrive`):** Installed and OAuth-connected (`tushar27993@gmail.com`) — cloud sessions can upload `queue/*.json` + `refs/active/*.png` and download `output/*.png` directly via `gdrive` CLI without a local Drive mount.
- **Gemini `image-pipeline-executor` Skill (Updated & Verified):**
  - Resolves reference PNGs **directly from Google Drive** (`My Drive/image_pipeline/refs/active/` -> exact Drive file URL -> Google Workspace Search -> native image handle) — **zero manual browser uploads required**.
  - Quota-optimized: skips re-reading `README.md`, checks `paused.json` + `queue/*.json` first (fast-exits if empty), and reuses resolved Drive URLs across batched requests.
- **Gemini Scheduled Actions (`gemini.google.com/scheduled`):**
  - Slots 2–4 deleted; `Process Image Pipeline Queue slot 1` and `Watch Emberwake feedback slot one..four` are **Paused** (**0 idle Gemini Task burn**).
  - Toggle `slot 1` on only during active overnight batches, or trigger on-demand after confirming non-empty `queue/` via `gdrive`. Batch 5–10 maps per stage into a single Task run.
- **Pending Test Item in Drive Queue:** `sunk_ward_runoff_grate_ref_lit_q1795560970.json` is still in `My Drive/image_pipeline/queue/` (from before the quota reset) — delete or process it on the next batch.

## 3. Next Focus — The 4 Engine/Framework Pillars

1. **Pillar 1 — Gate & Dev-Loop Efficiency (`tools/fc.sh`, `tools/gate_steps.txt`)** *(Start here)*:
   - Add per-step elapsed timing output to `tools/fc.sh gate` and capture a profiled baseline on the cloud VM.
   - Run cheap non-Cargo checks (`layers`, `layouts`, `docs`, `assetlib`, `plannertests`, `tooltests`, `seeds`) **first and in parallel** for ~10s fail-fast.
   - Eliminate cold per-worktree rebuild tax (`sccache` / shared dependency cache), avoid duplicate `build` + `clippy --all-targets` compiles, and separate fast local/dev checks from full CI `shotsexact` + `newgame` while keeping full gate parity green.
2. **Pillar 2 — 3-Stage Reference & Layer-Wise Alignment Engine (`aegis_cli::align`)**:
   - Feed exact `L3 / L2` lighting ratio maps directly into the Pass 2 `MapFx` solver when staged refs are present, and derive `LightSpec` center/radius/RGB directly from `L3 / L2` peaks.
   - Add a `gdrive` CLI backend option to `aegis align-map enqueue` / `ingest` so cloud sessions push/pull `My Drive/image_pipeline` with zero local filesystem assumptions.
   - Add an `L0` water-vs-land tile consistency check to `check-ref stage=cleanplate` (catches water-basin drift at Stage 1 before Stages 2–3).
3. **Pillar 3 — Engine Presentation & Rendering (`aegis_ui`)**:
   - Generalize `MapFx` + `StaticLayers` into clean engine-owned traits/primitives in `aegis_ui` so any Aegis game gets layered offline capture, floor/wall zone grading, and light pools without per-game boilerplate.
4. **Pillar 4 — Engine/Kit Architecture & Automation (`engine/`, `kits/rpg/`)**:
   - Maintain strict 3-layer separation (`tools/fc.sh step layers`), strengthen `ENGINE-20` `fields`-scoped seed generation & validation, and move repo feedback watching off Gemini Tasks into lightweight Antigravity `/schedule` or GitHub Actions.
