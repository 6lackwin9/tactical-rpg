# Design calls for your review

The Design Session writes this file (decision "Build feedback on the recommended option (user 2026-11-10)"). When a Build Session `FB-nn` or `UN-nn` item offers options, Design applies the recommended one straight away (in `docs/READY.md` section 4 and the specs) and lists it here so you can check it later. To change one, say which row and what you want instead; Design then queues the change like any other.

Excluded (asked first, never applied alone): anything that cannot be undone (a push to `main`, deleting files or branches, releasing) and items with no recommended option.

## Open (newest first)

| Date | Item | What was asked | Applied (recommended option) | Where | Undo cost |
| --- | --- | --- | --- | --- | --- |
| 2026-11-10 | `FB-150` (PR4-2, AUDIT-08) | Dropping every `loadout_ranks` entry with no loadout entry would break valid saves (an empty class-default loadout keeps pinned ranks) | Option A: keep the narrower rule as built (drop only ranks the rank rules refuse) | READY Section 1 `PR4-2`, Section 4 | none (as built; Option B is a small follow-up) |
| 2026-11-10 | `FB-148` (PR4-1 part 2, AUDIT-07) | Routing the sim's `elevel=` through `raise_level` moves `sim_golden` run 11 (`0.82 -> 0.81`) | Option A: approve that one re-record as the intended effect (Option B was: drop AUDIT-07, keep `elevel=` a plain level shift) | READY Section 1 `PR4-1`, Section 4 | low (one golden line; reversible by reverting the commit) |
| 2026-11-10 | `FB-147` (WEBREL1 feasibility) | How to slice the web build after the audit (3 compile blockers, 0 golden risk) | Option A: split E1 into E1a "compiles" + E1b "runs", lint-based runtime acceptance, `fc.sh web` in G1, audio as a later bundle part, recordings and F12 shots off on web; queued as `PR4-3..PR4-7` | READY Section 1 and 2, `docs/specs/WEBREL1.md` 6.1 | low (queue order; nothing built yet) |

## Reviewed

Rows move here, with your answer and the date, once you have reviewed them.

| Date | Item | Applied | Your answer |
| --- | --- | --- | --- |
