# Assets & Audio handover

Written 2026-10-17 22:06 UTC by the first Assets & Audio session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue (in order)

1. Empty. Take the next approved `-A` task whose logic is on `develop` (BAL3-A belongs to the Balance thread).

## Update 2026-10-17 22:20 UTC (second session)

REP1-A checks 2-5 passed headlessly against the REP1-L runtime (0717392), no data fix needed: new game neutral 0, spare choice shown locked, `ember_patrol` locked; courtyard 10 (friendly), spare choice opens once (+2 potions) then locks, Hest prices 15/24/29/38, bombs 40; gate watch 20 and Horde -10; oath 35 and `ember_patrol` offered at Hest; an `ash_raiders` win makes it ready, completion 40 and `ashlands.ruins_swept`. After use the spare choice stays visible as locked (`show_locked` with `unless`), as specced. A new class (`RepRules`) needs `godot --headless --import` before a `-s` script sees it.

## Landed today on `develop` (this session)

S12C-A (802c487), TRAV1-A (53b20ac), REP1-A (b8845b1).

## Waiting on the user

- Nothing.

## Notes for the next session

- Economy mode: asset-builder sonnet low, change-reviewer low; run the reviewer while the full gate runs in the background (data-only diffs need no gate inside the review).
- `save_world` re-normalises the whole world (schema defaults, empty kind files); expected, no behaviour change.
- Combo and status families ship with `learned_only: true` (S12S-A and S12C-A specs marked); keep that for new class-path families.
- `tools/fc.sh arena <pattern>` runs the whole arena suite; the pattern only filters output.
- Branches to delete at the next cleanup (all landed): `claude/assets-audio-uiyic2`, `claude/TRAV1-A-roads`, `claude/REP1-A-reputation`.
