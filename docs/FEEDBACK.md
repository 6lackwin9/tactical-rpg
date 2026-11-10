# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | done (FB-95) | `develop` | 3 parallel lanes + FB-94 additions; checks_golden re-recorded. |
| R1.1-2 `C02-A` | done (FB-96) | `develop` | asset + logic lanes in parallel; phasefold re-recorded. |
| R1.1-3 `R1-BAL` | done (FB-99, FB-100, FB-103) | `develop` | Phase R1.1 complete. |
| READY row 14 `A2.2-S1/S2` | done (FB-102) | `develop` | - |
| READY row 16 `A4.2-K1a` + `A3.2-S1` | done (FB-105; `field_intent` gate moved to W2-4 per FB-106) | `develop` | R2-W1 batch 1 complete. |
| READY row 19 `I1.2-S1` | done (FB-104) | `develop` | - |
| READY row 15 `A1.2-S1/S2` | done (FB-107) | `develop` | - |
| READY row 17 `F1.2-S1` + `F2.2-S1` | done (FB-108) | `develop` | - |
| READY row 18 `P1.2-S1` + `P2.2-S1` | done (FB-109) | `develop` | R2 Wave 1 complete. |
| READY row 20 (R3-W1) | done (FB-111) | `develop` | R3 Wave 1 complete. |
| W2-4 (R2-W2) | done (slices 1-7) | `develop` | - |
| W2-5 (R2-W2) | done: slices 1-4 (FB-127..129, 131); open FB-128, FB-130 | `develop` | - |
| R2-W2b CLASS-BASICS | done (FB-132, FB-134, FB-135) | `develop` | - |
| R2 release | released: `main` @3cc73f2b ([#56](https://github.com/6lackwin9/tactical-rpg/pull/56), merge commit; tree = develop @f5c6568a) | `main` | - |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (`3353e62b`) | `develop` | - |
| R4 release | released: `main` @32ae1ada ([#58](https://github.com/6lackwin9/tactical-rpg/pull/58), merge commit; tree = develop @ad903f92, which adds a whitespace fix for an MD012 lint error from 92a7282b) | `main` | Ready queue empty; next: Section 3 `R3-STAGE-SEED`, `POST-R4-CLEANUP` when Design queues them. |
| WEBREL1 feasibility | done: FB-147 (read-only audit, no code) | - | Waiting on Design: E1 split (Option A) and slice order. |
| PR4-1 `R3-STAGE-SIM` | building (`logic-builder`) | `claude/optimistic-newton-ewkyqt` | Next: PR4-2 `POST-R4-CLEANUP` (items 1, 3, 4). |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-146 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-09). New items start at FB-147. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-147: WEBREL1 feasibility check (READY Section 2 row 7)

- **Spec**: `docs/specs/WEBREL1.md` section 7 (READY Section 2 row 7), audited read-only @4093aed3
- **Kind**: feasibility
- **Status**: open
- **Finding**:
  1. **Table check**: only 3 compile blockers between `emberwake_gui` and `wasm32-unknown-unknown`; `aegis_core`, `aegis_save`, `aegis_layout`, `aegis_content`, `rpgkit_rules`, `rpgkit_sim`, `emberwake_rules` and `emberwake_world` already `cargo check` clean on wasm32. Blockers: (a) `arboard 3.7.0` (the only crate in the tree that fails: no `platform::Clipboard`); (b) `aegis_ui/src/shell.rs:1456` `Pixels::new` does not exist on wasm; (c) `shell.rs:1489` `ControlFlow::WaitUntil` takes `web_time::Instant` on wasm (**missing from the table**). With those fixed, `emberwake_gui` type-checks with 0 errors. The real cost is runtime: std stubs compile but `Instant::now` / `SystemTime::now` panic and fs / env / TCP return `Err`. Rows: **file reads** live in engine and kit loaders (see 3; `emberwake_gui` runtime hits only `main.rs:85-88`, `gallery.rs:176`); **user files**: `aegis_save/src/lib.rs:29` confirmed, the table misses a second settings writer at `aegis_ui/src/screen.rs:248,285-287` plus `recordings.rs`, `aegis_host/src/playtest.rs`, `frame_tuner.rs:417-448` (dev); `rpgkit_world/src/save.rs:30` (Godot APPDATA path) has no callers; **clock**: 28 runtime sites in 14 files, including the kit (`rpgkit_world/src/new_game.rs:87` `clock_seed`: a New game without a seed would panic), the game (`emberwake_gui/src/battle/{flow.rs:365-449, playback, talks, draw}`, `dojo.rs`, `portraits.rs`) and `aegis_save/src/lib.rs:140`; `web-time 1.1.0` is already in `Cargo.lock` (via winit); **threads/TCP** confirmed (`bridge.rs:20-26`, `shell/headless.rs`, `aegis_cli/src/scenario.rs`, `aegis_core/src/scratch.rs` test helper): they compile and fail at runtime, so they need `cfg`; **process/env**: `emberwake_gui/src/main.rs:76-81` (`current_dir()?` aborts startup on web) and, **missing from the table**, `shell.rs:1380` `process::exit(0)` (Quit); **window**: `windows-sys` already `cfg(windows)` (OK); **renderer**: `Pixels::new_async` (pixels turns on wgpu `webgl` itself); **audio**: compiles, silent without cpal `wasm-bindgen`; **clipboard**: `aegis_ui/src/edit.rs:650-670` confirmed; **bundle**: the table misses `engine/assets` (1.3 MB, mostly tilesets).
  2. **Versions** (pinned): pixels 0.15.0, winit 0.30.13, wgpu 0.19.4, rodio 0.20.1, cpal 0.15.3 all support the web; **no bump needed, so no golden risk**. winit: `EventLoopExtWebSys::spawn_app` instead of `run_app` (`shell.rs:1814`) plus a canvas attribute; pixels: async init cannot run inside `resumed`, so use `wasm_bindgen_futures::spawn_local` and return the surface as a user event; rodio 0.20 does not forward cpal's `wasm-bindgen` feature, so add a wasm-only direct `cpal` dep with it. New deps: `wasm-bindgen` 0.2.129 (already locked), `wasm-bindgen-futures`, `web-sys`.
  3. **Read seam**: about 45 runtime read sites in 13 files: `aegis_core/src/assets.rs` (15; layer resolution and `read_dir`), `aegis_ui/src/theme.rs:823-1024` (6), `art.rs:71,208,430,455`, `audio.rs:117,241`, `tuning.rs:31-36` (hot reload, native only), `aegis_content/src/file.rs:21`, `world.rs:199`, `rpgkit_world/src/data.rs:51,70,80`, `rpgkit_content/src/store.rs:348,517,579`, `source.rs:58`. Many are `is_file` / `is_dir` / `read_dir` probes, so the API needs `read`, `read_to_string`, `exists` / `is_file` / `is_dir` and `list_dir`. One API covers all of them; **rules, sim and battle crates have 0 fs sites**. Kit loader code is touched (`rpgkit_world` data and new_game, `rpgkit_content`), which the spec's E1 layer list (engine only) omits.
  4. **Bundle**: raw data 1.1 MB, game assets 22 MB, kit assets 7.9 MB, engine assets 1.3 MB; gzip -9 of data + game + kit assets is 27.6 MB (data 0.10 MB, game assets 20.5 MB, kit assets 7.0 MB; xz -9 with engine assets 28.1 MB; brotli not installed, expected about the same since the 647 PNGs and 10 OGGs are already compressed). wasm: a 1.6 MB measured release build (0.35 MB gzip) is a lower bound (startup bails at `current_dir()?`, so dead code is dropped); estimate 6-12 MB raw, 2-4 MB gzip. First load about 31 MB: ~25 s at 10 Mbit/s, ~5 s at 50 Mbit/s. Deferrable: audio (kit 5.3 MB + game 4.1 MB, silent until E2 anyway, about -30%) and `backgrounds/story` (VN stages, ~1.1-1.3 MB each; 8.9 MB for all backgrounds) as a second lazy bundle.
  5. **Saves**: `aegis_save::write_envelope` (`lib.rs:165-193`) writes tab-indented sorted JSON; the largest state in the journey golden (64 states, 700 steps) is a 2.9 KB envelope. A long campaign should stay under ~50 KB a slot, so 6 slots + settings + locks < 0.5 MB, far below 5 MB even at 2x for UTF-16; saves are text, so no base64. Risk: itch.io serves HTML5 games from the shared `itch.zone` origin, so the quota may be shared, and playtest recordings with JPG shots could overflow it.
  6. **Cost** (builder runs), all slices can keep **0 golden diffs** (on native `web-time` re-exports std and the seam passes straight to `std::fs`, so `checks_golden` and the store resave keep their bytes): **E1 L** (2-3 runs: arboard gate, clock seam across engine + kit + game, read seam, user store with `aegis_save`'s 14 read / 11 write path sites mapped to keys plus `screen.rs` settings, `cfg` for bridge, headless and exit); **E2 M** (1-2 runs; needs `wasm-bindgen-cli` 0.2.129, not installed); **G1 M** (rework `main.rs:76-130` startup; `emberwake_gui` is bin-only, so add a lib/cdylib start or use the bin; smoke tests need Playwright: the container has Chromium at `/opt/pw-browsers/chromium`); **P1 S-M** (no web job in `.github/workflows` yet; install `wasm-bindgen-cli` and `butler` in CI).
- **Recommendation**: order **E1 -> E2 -> G1 -> P1**, with P1's `tools/fc.sh web` moved into G1 so G1 can be tested. **Option A (recommended)**: split E1 into **E1a "compiles"** (arboard target-gate, `aegis_core::clock` re-exporting `web_time` swapped in at the 28 sites, `shell.rs:1456/1489` behind `cfg`; small, and it makes the spec's E1 compile criterion true) and **E1b "runs"** (read seam, user store, `cfg` for TCP and exit). **Option B**: keep E1 whole. Either way: (1) give E1 a runtime acceptance, since a clean compile proves little when the std stubs compile (a lint: no `std::time` / `std::fs` outside the seams in non-test, non-CLI `src`; or a `wasm-bindgen-test`); (2) let E1 span engine, kit and game for the clock and read sites above; (3) ship audio as a later bundle part; (4) consider IndexedDB, or recordings off on web, given the shared `itch.zone` quota.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
