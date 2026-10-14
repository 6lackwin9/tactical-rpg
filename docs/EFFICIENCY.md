# Time and token efficiency log

Living file: measure, fix, record. Add a row whenever a step feels slow or a run floods the context. Re-measure after changes. Times are wall-clock on the dev machine (2026-10-15).

## Measured steps

| Step | Cost | Verdict |
|---|---|---|
| `tools/fc.sh parse` (changed files) | 0.5 s (auto class-cache refresh only when new .gd files appear) | fine |
| `tools/fc.sh parse-all` (every script, one process) | 3 s | use for "does everything compile" |
| `tools/fc.sh arena` | 20 s | fine |
| `tools/fc.sh dojo` | 2.5 s | fine |
| `check_project.ps1` (old default) | 188 s, ~170 output lines (~9k tokens of PASS lines) | **slow + noisy**: 130+ Godot launches (one per file) |
| `check_project.ps1 -Fast -Quiet` | 32 s, ~20 output lines | pre-commit default |
| `tools/fc.sh sim duel 5` (3 seeds) | ~90 s; `tier` ~2 min; 10 seeds ~5 min | acceptable; run in background / in parallel (own sandboxes) |
| Builder agent for M1 | ~23 min, ~300k tokens, 159 tool calls | the dominant cost; see below |
| Reviewer agent | ~2.5 min, ~30k tokens | cheap, keep |
| Brainstorm agent | ~1.5 min, ~56k tokens | cheap, keep |

## Bottlenecks found and fixes

1. **Per-file parse in check_project.ps1 (~155 of 188 s).** Fixed: `tools/parse_all.gd` + `-Fast` switch (and `fc.sh parse-all`).
2. **Test output floods context (~9k tokens per full run).** Fixed: `-Quiet` prints failures and a summary only; `fc.sh` prints failures first with a log path. Rule: never print full suite output; grep the log.
3. **Patch scripts that failed silently or truncated files** (find() == -1, `\n` in `"""` strings, shell heredocs with GDScript). Cost: several retry loops. Fixed: `patch-and-seed` skill (Edit tool for text, upsert for data), known-issues rows.
4. **Test numbers hard-coded to old data** forced repeated suite-by-suite fixes after each data change. Fix: derive expectations from rules (done for new suites); when data changes broadly, print real values once with a scratch script instead of iterating through full runs.
5. **Rediscovering context** (reading files to learn rules). Fixed: `CLAUDE.md` + `docs/GAME_BRIEF.md`, STRUCTURE, DECISIONS, HISTORY. Keep them updated in the same change.
6. **Large builder tasks.** One agent doing engine + AI + data + tests is long and expensive. Mitigation: split milestones into slices of roughly one subsystem each (e.g. M1 could have been 1: ally targeting, 2: gap-close + statuses, 3: AI), brief the builder with exact file pointers from the brief instead of letting it explore, and ask it to report open questions early rather than at the end.
7. **Re-spawning agents for follow-ups** loses their context. Prefer `SendMessage` to continue the same agent for fixes to its own work.
8. **Waiting on long jobs in the foreground.** Run sims and full checks with `run_in_background` and do doc work meanwhile; parallelise independent sims (each has its own sandbox).
9. **Custom agents are not callable until the session restarts**; a fallback (general-purpose agent with the same instructions) worked. Restart after adding agent files to avoid the detour.
10. **Stale IDE diagnostics** after adding `class_name` files caused false alarms. `fc.sh parse` refreshes the class cache; trust it over the IDE panel.

## Working rules (cheap path first)

- Edit -> `tools/fc.sh parse` -> `tools/fc.sh arena` (or `dojo`) -> repeat. Run `check_project.ps1 -Fast -Quiet` only before a commit.
- Read with `grep -n` and line ranges, not whole files; use the brief to find the file first.
- Batch independent tool calls in one turn; background anything over ~30 s.
- After 2 failed fixes, ask the user instead of brute-forcing (token sink).

## Ideas not done yet
- `fc.sh suite <name>`: run a single test suite (arena takes 20 s, so low value now; revisit if it grows past ~60 s).
- Cache catalog loads in sims (minor).
- Cut `balance_sim` time by lowering tick/step overhead or fighting fewer seeds on obvious matchups (revisit when tuning).
