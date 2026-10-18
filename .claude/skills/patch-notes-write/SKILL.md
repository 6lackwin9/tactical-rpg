---
name: patch-notes-write
description: Add the newest develop commits to PATCH_NOTES.md (Merge & Release thread, at each release batch or when the user asks; not per landing).
---

# Write patch notes

`PATCH_NOTES.md` at the repo root lists every change on `develop`, newest day first (user 2026-10-18). The Merge & Release thread is its only writer (cloud.md "Batched release").

1. `git fetch origin develop && git checkout -B develop origin/develop`, then `python3 tools/patch_notes.py`. It prints the commits since the `<!-- last: <sha> -->` marker, grouped by day, split into game candidates and "Under the hood".
2. For today's section (`## <YYYY-MM-DD>`, created at the top if missing):
   - "In the game": one plain-language line per player-visible change, saying what a player or tester sees and where (for example "**Map movement**: walk with WASD..."). Merge with an existing line on the same feature instead of adding a second. Skip schema-only and sim-only commits.
   - "Every change that touched the game" and "Under the hood": paste the script's lines as they are.
3. Replace the marker with the new one the script printed.
4. Run `bash tools/fc.sh docs`, commit `PATCH-NOTES: <task ids>` (docs-only, `Review: none (patch notes)`, `Gate: fc.sh docs passed (docs-only)`), and push `develop`. If the push is rejected, fetch, rerun the script and repeat.
5. Run it once per release batch (11:52 and 17:52 UTC) and on request, never per landing (decision "Patch notes per release", user 2026-10-18 16:10 UTC).
