# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to `ENGINE-64` (all rows through 2026-11-24; `FB-01`..`FB-513`, `UN-01`..`UN-12`) | released or done | `main` / `develop` (`d5b35cd5`) | Every item answered in `docs/READY.md` section 4; the items and these status rows are in `docs/archive/FEEDBACK_CLOSED.md` (rows moved 2026-11-24, ENGINE-65). Open questions named in old rows live on as READY rows. |
| `ENGINE-65` (`TIDY3`) | Done (tools lane) | `develop` | FEEDBACK.md archived from 228 KB to about 3 KB (50 items and 171 status rows moved to FEEDBACK_CLOSED; `feedback_check.py` clean); housekeeper pass 4 over the week's engine additions written as `CLN-72`..`CLN-81` in `docs/specs/CLEANUP.md`, proposals only. FB-514 |

## 2. Feedback Items (Append-Only)

### FB-514: ENGINE-65 TIDY3 done; FEEDBACK.md archived, cleanup pass 4 proposed

- **Status**: done (tools lane), `develop`. Docs only, no code. `tools/fc.sh docs --all` and the planner tests are
  clean. The gate was not re-run: no code changed since gate97 (`d3e27428`).
- **Spec**: `docs/READY.md` row `ENGINE-65` (`TIDY3`).
- **(1) Archive**: `feedback_check.py archive` moved the 50 answered items (FB-388 to FB-513) to
  `docs/archive/FEEDBACK_CLOSED.md` (Status lines rewritten `closed: accepted (@d5b35cd5)`). The 171 status rows of
  section 1 moved there too, under "Build status rows through ENGINE-64", and one summary row replaces them.
  FEEDBACK.md went from 241,859 bytes to about 3 KB, and `feedback_check.py check` is clean (no open items, 180
  answer rows).
- **(2) Housekeeper pass 4**: `docs/specs/CLEANUP.md` "Pass 4" lists `CLN-72`..`CLN-81`, all `proposed`. I found no
  dead modules. The duplicates: the build-name string (x4), ms rounding helpers (x6), two medians (merging moves
  `walk_feel` numbers; Design call), path constants in `live.rs`, cache clearing in `Audio`. Structure: the 220-line
  headless `feel` request. Launch keys and settings: `feel` read from process args outside `Start`, `dev=1` parsed
  twice, the settings `"feel"` key no row sets (Design call). Test hooks in shipped paths: `stall_ms`/`spike_ms`, the
  release-refused `panic` (keep, document). Unused shipped helper: `controls::stick_dir`. Items that change numbers
  or API say so.
- **Docs delta**: none beyond CLEANUP.md (Design's file, written at Design's request for this row).
