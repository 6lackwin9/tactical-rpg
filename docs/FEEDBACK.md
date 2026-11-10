# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to R2 rows (R1 release, R1.1, R2 Waves 1-2, R2 release) | released (`main` @98ab973a, @3cc73f2b) | `main` | Rows moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10, size). |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (`3353e62b`) | `develop` | - |
| R4 release | released: `main` @32ae1ada ([#58](https://github.com/6lackwin9/tactical-rpg/pull/58), merge commit; tree = develop @ad903f92, which adds a whitespace fix for an MD012 lint error from 92a7282b) | `main` | Ready queue empty; next: Section 3 `R3-STAGE-SEED`, `POST-R4-CLEANUP` when Design queues them. |
| WEBREL1 feasibility | done: FB-147 (read-only audit, no code) | - | Waiting on Design: E1 split (Option A) and slice order. |
| PR4-1 `R3-STAGE-SIM` | done: part 1 (`stage=N`, `55ef14d8`), part 2 (AUDIT-07, `sim_golden` run 11 re-recorded, FB-154) | `develop` | - |
| PR4-2 `POST-R4-CLEANUP` | done: phase A (`feadda4d`), phase B re-freeze (`c49194b2`, FB-152; FB-153 suggestion); item 2 stays held | `develop` | - |
| PR4-3 `WEBREL1-E1a` | done (FB-155): wasm32 check clean, `aegis_core::clock`, clock lint | `develop` | - |
| PR4-4 `WEBREL1-E1b` | done (FB-156; FB-157 suggestion): `aegis_core::vfs`, `aegis_save::store`, wasm `cfg`, fs lint | `develop` | - |
| PR4-8 `SKILLCHECK-FOLD` | done (FB-158 suggestion): fold error when nothing else is reported; ranks normalised before the fold | `develop` | Next: PR4-5 `WEBREL1-E2`. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-155 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-156. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-156: PR4-4 `WEBREL1-E1b` landed (read seam, web user store, wasm `cfg`, fs lint)

- **Spec**: READY Section 1 row PR4-4; `docs/specs/WEBREL1.md` 2.1, 2.2, 6.1 (the `PR4-4` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `aegis_core::vfs`: `read`, `read_to_string`, `exists`, `is_file`, `is_dir`, `list_dir` (`std::fs` on native, `read_dir` order kept, sorting callers still sort; on wasm a mounted `Table` of path -> bytes with derived folders and sorted listings, `vfs::key` normalisation, `mount` additive so a later bundle part such as audio can extend it; PR4-6 fills it). Routed sites, probes included: `aegis_core` 15, `aegis_content` 17, `aegis_ui` 19, `rpgkit_content` 14, `rpgkit_world` 3, `emberwake_gui` 3 (`main.rs:85-88`, `gallery.rs`). Left on `std::fs`: the content writer's writes (`aegis_content` `store.rs`, `manifest.rs`: tooling; their reads use the seam), the tuning hot-reload stamp (native-only), `aegis_core/src/scratch.rs` (test helper), `aegis_cli`, `src/bin`. PNG reads decode from memory and PNG writes encode to memory first (same bytes; a failed encode leaves no partial file). (2) `aegis_save::store`: a `Store` trait (native `Files`, unchanged; web `Keys<LocalStorage>`, keys `aegis/<game id>/<relative path>`, a refused write becomes `io::ErrorKind::StorageFull` and the usual "Can't write" save problem, the old save kept); all 26 `aegis_save` sites and the `aegis_ui/src/screen.rs` settings writer use it (`aegis_ui` now depends on `aegis_save`; `aegis_save` gains a wasm-only `web-sys` with `Window`, `Storage`); `with_store` swaps the store per thread for tests, `MemKeys` is an in-memory backend with a byte quota. Web saves are the same bytes. (3) wasm `cfg`: `bridge::start` returns None; the headless run loop is native-only; recordings are forced off on the web; Quit is a no-op on wasm (a page cannot close its own tab; FB-157). (4) Lints: the PR4-3 scanner moved to `aegis_core/tests/lint/mod.rs`; `fs_lint.rs` refuses every `std::fs` path and no-argument path probes outside the seams (allowed: `vfs.rs`, `aegis_save/src/store.rs`, `scratch.rs`; content-writer files may keep write calls only; crate `aegis_cli` excluded); both lints now skip exactly `#[cfg(not(target_arch = "wasm32"))]` items and modules (so `tuning.rs` left the clock allow-list). Tests: vfs 4, web_store 5, settings_store 2, fs_lint 5. wasm32 check of `emberwake_gui` clean. **0 golden diffs** (`checks_golden` and the store byte-for-byte resave unchanged); gate 2738 passed, 0 failed, 4 ignored, 0 warnings.
- **Recommendation**: engine/README.md `aegis_core` row: "`vfs`: the one read seam (`read`, `read_to_string`, `exists`, `is_file`, `is_dir`, `list_dir`; `std::fs` on native, a mounted `Table` on wasm, `mount`); `vfs::dev`: dev files (recordings, shots, dev-tool writes), off on the web; `tests/fs_lint.rs` refuses `std::fs` and path probes outside the seams (scanner shared with `clock_lint` in `tests/lint/`)." `aegis_save` row: "`store`: the user store trait (`Files` native, `Keys<LocalStorage>` web, keys `aegis/<game id>/<path>`; a quota refusal is a save problem), `with_store`, `MemKeys`." `aegis_ui` row: "settings go through `aegis_save::store` (depends on `aegis_save`); agent link, headless runs and recordings are native-only; Quit is a no-op on wasm." STRUCTURE: `aegis_core/src/vfs.rs`, `aegis_save/src/store.rs`; tests `aegis_core/tests/{fs_lint.rs, vfs.rs, lint/mod.rs}`, `aegis_save/tests/web_store.rs`, `aegis_ui/tests/settings_store.rs`. known-issues: "wasm: `std::fs` compiles but returns Err; read through `aegis_core::vfs` and user files through `aegis_save::store` (`fs_lint` fails otherwise)." HISTORY: none. EFFICIENCY tip (builder): total a long test log with `grep '^test result' | awk` instead of reading it.

### FB-157: PR4-4 additions beyond the WEBREL1 text (`vfs::dev`, lint pass-lists) and Quit on the web

- **Spec**: `docs/specs/WEBREL1.md` 2.1, 6.1 (PR4-4); for G1 (`PR4-6`)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (1) **`aegis_core::vfs::dev`** is a new public engine API beyond 2.1's list: F12 shots, note JPGs, recordings, frame-tuner writes and `ground::compose_file` go through it (`std::fs` on native; every call fails with `Unsupported` on the web, so a dev feature reports instead of panicking). It implements 6.1's "recordings and F12 screenshots off on web" in one named seam the lint sees, instead of `cfg` on about 10 files; `change-reviewer` recommends keeping it. (2) The fs lint passes the content writer's write calls (`WRITERS`; its reads and probes are still linted) and allows `aegis_core/src/scratch.rs` (a test helper). (3) **Quit on the web is a no-op** (the quit flag is cleared and the screen stays; the user closes the tab), which cannot trap the user; a page cannot close its own tab. (4) A web save writes a temp key then moves it over the save, so it briefly needs about twice the save's size in quota (fine under the 0.5 MB estimate; `setItem` is already atomic, so G1 could skip the temp key).
- **Recommendation**: (1)-(2) record `vfs::dev` and the two pass-lists in WEBREL1 2.1 / 6.1 (as built). (3) **Option A (recommended)**: on the web, Quit returns to the main menu (or the menu hides Quit), decided in G1; **Option B**: keep the no-op. (4) Optional G1 tidy: write web saves directly (no temp key).

### FB-158: PR4-8 `SKILLCHECK-FOLD` landed (fold error when nothing else is reported; ranks normalised before the fold)

- **Spec**: READY Section 1 row PR4-8 (`FB-153` Option A) (the `PR4-8` squash commit on `develop`, 2026-11-10)
- **Kind**: suggestion
- **Status**: open
- **Finding**: `rpgkit_content::check::skill::entry_errors` now reports a family head whose ranks do not fold as one content error, `Family '<id>': the description template is not checked because the ranks do not fold (<first fold problem>)` (test `rpgkit_content/tests/skill_fold_error.rs`, 3). Two choices beyond the row's text: (1) **the error is emitted only when the entry reports no other error**: emitting it always added a restated error to 315 of 409 `checks_golden` skill cases (for example "Heal base belongs to SUPPORT skills, not ATTACK." plus a fold error naming `heal_base`) and caught nothing new; with the rule, an entry that already fails `content check` shows its fold problem on the next check after that error is fixed, so no data set reaches `data ok` with an unchecked template (fold problems reported nowhere else: a rank-2+ unknown key, a rank name that is not the derived name, a template that differs between ranks). `change-reviewer` ruled it within the spec's intent. (2) **Ranks are filled with schema defaults before the fold** (`content("skills", o)`, as `collapse` does): `validate_entry` passes a defaulted head while sibling rows came raw, so a rank missing a schema-default key (a row saved before a field existed, the AUDIT-10 case) failed the fold with "'system' must match across ranks" — silent before, a false error once fold errors surface. **0 golden diffs** (`checks_golden` unchanged), shipped data `data ok` (0 new errors); gate 2741 passed, 0 failed, 4 ignored, 0 warnings.
- **Recommendation**: **Option A (recommended, as built)**: keep both. **Option B**: emit the fold error always (re-record `checks_golden`: 315 cases gain a restated error). HISTORY: none (0 golden diffs). STRUCTURE tests: `rpgkit_content/tests/skill_fold_error.rs`.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
