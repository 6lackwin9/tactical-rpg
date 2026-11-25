# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to `ENGINE-64` (all rows through 2026-11-24; `FB-01`..`FB-513`, `UN-01`..`UN-12`) | released or done | `main` / `develop` (`d5b35cd5`) | Every item answered in `docs/READY.md` section 4; the items and these status rows are in `docs/archive/FEEDBACK_CLOSED.md` (rows moved 2026-11-24, ENGINE-65). Open questions named in old rows live on as READY rows. |
| `ENGINE-65` (`TIDY3`) | Done (tools lane) | `develop` | FEEDBACK.md archived from 228 KB to about 3 KB (50 items and 171 status rows moved to FEEDBACK_CLOSED; `feedback_check.py` clean); housekeeper pass 4 over the week's engine additions written as `CLN-72`..`CLN-81` in `docs/specs/CLEANUP.md`, proposals only. FB-514 |
| `ENGINE-66` (`TIDY4`) | Built (tools lane) | `develop` | Cleanup pass 4 built (CLN-72, 73, 75 to 81; CLN-74 not taken): shared build name and ms rounding, the feel request split, launch keys `dev` and `feel` read once by `Start`, test hooks and the settings `feel` key documented. No behaviour change; shots clean; goldens unchanged. FB-515 |

## 2. Feedback Items (Append-Only)

### FB-515: ENGINE-66 TIDY4 built; cleanup pass 4 items done, no behaviour change

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui`) plus Emberwake's `dev=1` read. **Every golden
  unchanged.** Shots diff against `839fbf7b` is clean except the known `screens_menus__scene_runner`. Gate clean.
- **Spec**: `docs/READY.md` row `ENGINE-66` (`TIDY4`).
- **Done**:
  - `CLN-72`: `perf::build_name()` in the 4 places.
  - `CLN-73`: `perf::round_to(x, places)` replaces the inline closures and the bodies of `r2`/`r3`, with each report
    keeping its places (also the pad tuning's snap).
  - `CLN-75`: the headless `feel` request's held-direction steps share `Loop::hold_input`, and its time steps
    (`wait_ms`, `stall_ms`, `spike_ms`) are `Loop::clock_step`; `feel` drops from about 220 to about 170 lines.
  - `CLN-76`: `live.rs` uses `theme::FILE` and `screen_layout::LAYOUTS_DIR`.
  - `CLN-77`: `start::Start` reads `dev` (`Start::dev`) and `feel` (`Options::feel`: `feel=1`, or `--feel` through
    `feel::launch_asked`, never on the web as before) once. The shell takes `Options::feel`, and Emberwake's
    `dev_asked` takes `Start::dev`.
  - `CLN-78`: the settings `"feel"` key is kept and documented as the player's switch beside the run's `--feel`
    (`feel.rs` header, `scenarios.md`).
  - `CLN-79`: `stall_ms`/`spike_ms` and the link's `panic` are named test hooks in `scenarios.md` and
    `crash_safety.md`.
  - `CLN-80`: `controls::stick_dir` is documented as the default.
  - `CLN-81`: `Audio`'s `forget` and `remounted` share `State::drop_cached`.
  - `CLN-74`: not taken.
- **Tests**: `start.rs` (`dev` and `feel` read once), Emberwake `dev_1_shows_the_developer_surfaces_only_with_the_dev_tools`
  through `Start::dev`; all `aegis_ui` tests and `walk_feel` cover the split request.
- **Docs delta**: `engine/README.md` (Design), per CLN-77, a launch-key table: "Engine launch keys (read by
  `start::Start`): `port`, `saves`, `screen` (the game's), `dev=1` (developer surfaces and live reload in a build with the
  dev tools; `Start::dev`), `live=0` (no live reload; scenario runs), `feel=1` / `--feel` (the feel probe; the
  settings key `"feel": true` is the player's switch), `headless=WxH`, `fps`, `dpr`, `clock=step` (no-window runs)".
  `CLEANUP.md`: mark `CLN-72`, `73`, `75` to `81` done and `CLN-74` dropped.

