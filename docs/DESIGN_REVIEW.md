# Design calls for your review

The Design Session writes this file (decision "Build feedback on the recommended option (user 2026-11-10)"). When a Build Session `FB-nn` or `UN-nn` item offers options, Design applies the recommended one straight away (in `docs/READY.md` section 4 and the specs) and lists it here so you can check it later. To change one, say which row and what you want instead; Design then queues the change like any other.

Excluded (asked first, never applied alone): anything that cannot be undone (a push to `main`, deleting files or branches, releasing) and items with no recommended option.

## Open (newest first)

| Date | Item | What was asked | Applied (recommended option) | Where | Undo cost |
| --- | --- | --- | --- | --- | --- |
| 2026-11-10 | `FB-165` (WEBOPT1 limit-blocker) | The `apt` Binaryen 108 shrinks the `.wasm` (3.8 -> 3.5 MB gzip) but the optimised build no longer boots | Option A: pin a current Binaryen release on CI (sha256-checked) plus a minimum-version guard in `tools/fc.sh web` (Option B: build without reference types; Option C: drop WEBOPT1, keep only the guard) | READY Section 1 `PR4-9`, Section 4 | low (advisory CI job; nothing ships until the smoke passes) |
| 2026-11-10 | `FB-164` (web build size on CI) | CI skips `wasm-opt` (no Binaryen), so the itch.io `.wasm` stays 12.9 MB (3.8 MB gzip) | Option A: queue a small platform task to install Binaryen on CI (`PR4-9` `WEBOPT1`); Option B was to wait until load time matters | READY Section 1 `PR4-9`, Section 4 | none (not built yet) |
| 2026-11-10 | `FB-162` (dev keys on the web) | Dev keys on "debug build OR dev tools" would switch them on in desktop release builds | Option A, as built: debug build AND dev tools; the web test build has no F6/F7/F9 keys (Option B: also allow them in the web test build) | `WEBREL1.md` 2.3, READY Section 4 | low (a small follow-up adds Option B) |
| 2026-11-10 | `FB-160` (web renderer and music) | WebGPU-first or WebGL2 only; music before the first click | Option A: WebGL2 only for this release (wgpu picks WebGPU even with no adapter); the wanted music fades in at the first gesture; a failed renderer shows an on-page message (G1) | `WEBREL1.md` section 1, 2.3, 6.2; READY `PR4-6` | low (a later engine task can add WebGPU with GL fallback) |
| 2026-11-10 | `FB-158` (skill fold check) | Show the new fold error always, or only when the entry has no other error | Option A, as built: only when nothing else is reported (always would restate an error on 315 golden cases); ranks get schema defaults before the fold | READY Section 4 | low (Option B = re-record `checks_golden`) |
| 2026-11-10 | `FB-157` (Quit on the web) | A page cannot close its own tab; Quit is a no-op today | Option A: the main menu hides Quit on the web (G1); web saves write directly with no temp key | `WEBREL1.md` 6.2; READY `PR4-6` | none (not built yet) |
| 2026-11-10 | `FB-154` (PR4-1 part 2) | The approved `sim_golden` re-record turned out larger on the new baseline (rounds 3.0 -> 4.0, party hp 0.81 -> 0.98, not 0.82 -> 0.81) | Accepted as the same intended AUDIT-07 effect (the alternative was reverting to a plain level shift, FB-148 Option B) | READY Section 4, `HISTORY.md` | low (revert one commit) |
| 2026-11-10 | `FB-153` (skill content check) | A skill whose rank rows fail to fold silently skips template checks | Option A: report the fold failure as a content error; queued as `PR4-8` `SKILLCHECK-FOLD` | READY Section 1 `PR4-8`, Section 4 | none (not built yet) |
| 2026-11-10 | `FB-150` (PR4-2, AUDIT-08) | Dropping every `loadout_ranks` entry with no loadout entry would break valid saves (an empty class-default loadout keeps pinned ranks) | Option A: keep the narrower rule as built (drop only ranks the rank rules refuse) | READY Section 1 `PR4-2`, Section 4 | none (as built; Option B is a small follow-up) |
| 2026-11-10 | `FB-148` (PR4-1 part 2, AUDIT-07) | Routing the sim's `elevel=` through `raise_level` moves `sim_golden` run 11 (`0.82 -> 0.81`) | Option A: approve that one re-record as the intended effect (Option B was: drop AUDIT-07, keep `elevel=` a plain level shift) | READY Section 1 `PR4-1`, Section 4 | low (one golden line; reversible by reverting the commit) |
| 2026-11-10 | `FB-147` (WEBREL1 feasibility) | How to slice the web build after the audit (3 compile blockers, 0 golden risk) | Option A: split E1 into E1a "compiles" + E1b "runs", lint-based runtime acceptance, `fc.sh web` in G1, audio as a later bundle part, recordings and F12 shots off on web; queued as `PR4-3..PR4-7` | READY Section 1 and 2, `docs/specs/WEBREL1.md` 6.1 | low (queue order; nothing built yet) |

## Reviewed

Rows move here, with your answer and the date, once you have reviewed them.

| Date | Item | Applied | Your answer |
| --- | --- | --- | --- |
