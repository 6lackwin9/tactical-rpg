---
name: godot-known-issues
description: Known-issues table for this Godot tactical RPG (symptom, cause, fix). Load it as soon as anything fails (parse error, test FAIL, tool error, flaky run, git/worktree trouble) and before debugging from scratch; add a row for every new recurring cause.
---

# Known issues

Look the failure up here first. Only debug from scratch if nothing matches, then add the new cause as a row. Fixed or obsolete rows move to `docs/archive/KNOWN_ISSUES_ARCHIVE.md`.

| Symptom | Cause | Fix |
| --- | --- | --- |
| Markdown table formatting warning (`MD060`) | Table delimiter `\|---\|---\|` lacks padding spaces | Pad with spaces: `\| --- \| --- \|` |
| Markdown heading/list blanks (`MD022`, `MD032`) | Missing blank line immediately before/after `#` heading or list | Add exactly 1 blank line above and below |
| Markdown multiple blank lines (`MD012`) | Consecutive blank lines in `.md` file | Reduce to single blank line |
| `Identifier "StatRules/Proficiency/..." not declared`, "Could not find type" | Global class cache stale after adding a `class_name` file | Run the editor scan (step 2) |
| A script run hangs with no output | Compile errors in autoload-reachable scripts | Run `--check-only`, fix errors, retry |
| Whole file truncated after a patch script | `String.find()` returned -1 and `substr(0, -1)`/`substr(-1)` was used | Check every `find()` result before slicing; verify with `git diff --stat` |
| Patch `MISSING:` printed | `\n` inside GDScript `"""` strings is a real newline; tabs/escapes differ | Match exact text; use the Edit tool for single sites |
| `printf`/heredoc wrote a literal newline inside a string literal | Shell expanded `\n` | Fix with Edit; write GDScript via Write/Edit, not shell |
| `sed` line edits corrupt match blocks | Line numbers shifted | Never use sed ranges; use Edit |
| Unit stat/number tests fail after class/data changes | Test hard-codes old values (class budgets, speeds, power) | Print real values, derive expectations from rules (`StatRules`, `Proficiency`) not magic numbers |
| Skill missing for a class unit (`riposte` etc.) | `known_skills` = class path families unlocked by level (SK2 rule, `Combatant.tier_unlocked`); `min_proficiency` gating | Raise unit level or add `extra_skills`; check `Combatant.can_use` |
| Catalog refuses a class/unit | `StatRules`: budget 62, growth 5 (max 3/stat), spend exactly `4 + (level-1)` or empty, path rules (`ClassUnitValidator.path_errors`), loadout gear must be trained and fit the slot | Read the validation message; fix data, not the rule |
| Unit art empty/wrong | Unit `art` empty means "class art" | Use `ContentFactory.art_of(entry)` |
| Planner: friend can't enter a tile a faster friend vacates | `CombatPlan._ally_moves` evaluates in sequence-creation order | Evaluate faster units first (speed desc) |
| Speeds/tiles-per-block differ wildly between units | Speed = 8 + DEX - burden; ranger/scout are much faster than warriors | Do not assume the warrior is faster in tests; compare `speed` |
| Passive adds nothing at max level | Proficiency level clamps to 3 | Test passives on gear below level 3 or on stat/resist effects |
| `reset_round` test uses stale Combatant objects | At round 1 `reset_round()` calls `start()`, which rebuilds every Combatant | Test round-reset behaviour from round 2 (advance first) and re-fetch units after reset |
| New level-2+ catalog skill changes existing AI/test numbers | Hand-built units with empty `known_skills` know every skill | Mark new example skills `learned_only: true` or add them via `extra_skills` |
| Self-buff granted by a DEFEND skill vanishes immediately | The skill's own stance switch calls `set_stance`, which removes riposte | Grant in `on_start` after the stance switch (`StatusEffects.grant_now`) |
| (desktop) `git worktree remove` fails with "Permission denied"; half-removed entries in `.git/worktrees` | OneDrive marks folders read-only and Git for Windows cannot remove them | `powershell -NoProfile -File tools/wt_remove.ps1 ../ngp-<id> feat/<branch>`; if an entry survives, clear attributes on `.git/worktrees/<name>` and delete it, then `git worktree prune` |
| In a worktree after a rebase: `Identifier "XxxTestSuite" not declared`, arena PASS 0 | The rebase brought in new `class_name` files; that worktree's class cache is stale | Run `godot --headless --editor --quit-after 50` in that worktree |
| (desktop) IDE Problems tab shows errors in a worktree (e.g. "NO_ITEM not found") while Godot's headless parse is clean | The IDE language server reads the main checkout's class cache, not the worktree's | Trust `tools/fc.sh parse` in the worktree; the false errors clear after merge |
| `WARNING: ... invalid UID uid://... - using text path` in a worktree | The worktree's `.godot/` has no `uid_cache.bin` yet (stale import cache), harmless | Run `godot --headless --editor --quit-after 50` in that worktree |
| Grade/requires test on a shipped skill fails after a `requires` pairing is added | A dependant (greater_mend requires mend) blocks the grade change | Pick fixture families by rule (no dependants) or use a test-only sandbox family; never hard-code shipped ids |
| A content test fails once and passes on re-run | A suite forks to a fixed `user://` path instead of `TestHelpers.sandbox(tag)` | Use `TestHelpers.sandbox` + `fork` / `fork_worlds` / `cleanup` in every suite |
| Sim fight ends early with a healthy party, or a `PLAN FAIL` line | `confirm_plan` refused an invalid preset step | Read the `PLAN FAIL` line (unit, step, tile held by); check `CombatPlan._ally_moves` / `pinned_cell` |
| `check_project` fails a suite that printed no FAIL line | Any output line containing "fail" (any case) counts as a failure, e.g. a `push_warning` text | Use the quiet test hooks (`set_quiet`) or word warnings without "fail" |
| A scripted edit leaves a literal `$1` and drops the matched line | The perl replacement was built as a plain string (single quotes or a variable), so captures never interpolated | Write the replacement inside the `s///` itself, or use the Edit tool; grep for a literal `$1` after any scripted edit |
| A commit lands although `tools/fc.sh docs` printed `FAIL` | Piping `fc.sh docs` into `tail` before `&& git commit` takes the exit code of `tail` | Capture the output and test it (`out=$(tools/fc.sh docs)`, then `grep -q FAIL` on it), or run `fc.sh docs` on its own first |
| A fresh checkout (CI) fails a test on a committed PNG ("does not load as Texture2D", manifest not clean) that passes locally | `--editor --quit-after 50` can quit before the import of new image files finishes, so `.godot/imported` lacks them | Run `godot --headless --import` (the gate does since S14-A1); commit the `.png.import` files with the PNGs |
| `docs/archive/TASKS_DONE.md` (or another append-only table) shrinks to a few rows after a merge | A conflict in an append-only table was resolved by taking one side (`git show origin/main:<file> >`, "theirs" or "ours") | Keep both sides' rows; compare `wc -l` with the base before committing (DLG1-L #6 cut it to 2 lines; restored in REVIEW-STEP #30) |
