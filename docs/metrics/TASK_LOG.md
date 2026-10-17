# Task log (time per task, waits and bottlenecks)

Owner: the Merge & Release thread. It compiles the `Timing:` and `Blocked-by:` lines from each task's squash commit on `develop` into this file at every release batch (area threads never edit it, so it never conflicts). Format and rules: `docs/workflow/cloud.md` "Task timing". Goal (user 2026-10-17): find idle time and dependencies between steps first; tokens are secondary. Rotate rows older than a month to `docs/metrics/archive/TASK_LOG_<yyyy-mm>.md`.

Work is time a thread is actively building, reviewing, gating or fixing. Wait is time the task is done or blocked and nobody works on it, split by cause: `approval` (the user), `dependency` (another task or its merge), `ci`, `merge` (conflicts with `develop`/`main` and the re-gate they force) and `release` (landed on `develop`, waiting for the batch to reach `main`).

## Bottlenecks (updated each batch; newest first)

### 2026-10-17 release #37 (10 tasks)

1. **Waits are now near zero.** Eight of ten tasks report wait 0; area threads landed on `develop` 6-34 min after starting, with no PR or CI step in between.
2. **The one real wait was a user choice:** REVIEW-F4 sat ~35 min on the Butler version pin (proxy blocks the itch download host, so the user had to pick a version and hash). Decouple: ask such questions as a card at task start, and land the unblocked part first (done here as "REVIEW-F4 (partial)").
3. **Token cost is the new outlier, not time:** TOOLS-RO ran in standard mode at ~325k tokens versus 15-160k for the economy-mode tasks.
4. **Release wait** for this batch: landed 16:58-17:50, released after 17:55, so ~5-60 min, set by the daily schedule.

### 2026-10-17 releases #35 and #36 (Merge & Release thread)

1. **The first batch cost ~40 min of release work, mostly conflicts.** Seven old per-task PRs were squash-landed on `develop` 16:15-16:29; every one conflicted in the shared docs tables (AGENT_MAP, TASKS, TASKS_DONE, ROADMAP, DECISIONS index) and one in a data manifest (fixed by re-running its seed). Two local gates (~5 min each, run in a separate worktree) and one CI run (~3 min) then took #35 to `main` at 16:46. Landed-to-main wait: ~17 min.
2. **Mid-batch landings re-run CI.** GATE-DEDUPE and WEB-DAILY landed on `develop` while #35 was open, so CI ran three times on it; each run took ~3 min. Acceptable; cheaper than a second release.
3. **On-demand release #36 took ~3 min** (one CI run, no conflicts), because the batch was small and already gated on `develop`.

**Suggested decoupling**: the shared-log suggestion below (area threads stop editing AGENT_MAP / TASKS_DONE directly) would remove nearly all release-time conflicts.

### 2026-10-17 (backfill from git, PR and thread timestamps; all splits are estimates)

1. **Waiting after the work was done was the biggest cost, not CI.** Six PRs sat 47-107 min after their last real commit: S14-A1 (~95 min), DLG1-F (~80), BAL2-R (~47), S14-A2, WEB2 and PLAYTEST1-L (still open at the first release, 60-90 min each). The cause was the per-task merge order: each PR waited for approval or for related PRs to merge first, then had to catch up with `main`.
2. **Catch-up merges with `main` repeat work.** 15 "merge main" commits across 9 PRs (S14-A1 and DLG1-F three each). Each one meant a new CI run (~2.5 min) and usually a new local gate (~3-4 min). The conflicts were almost always in the shared logs `docs/AGENT_MAP.md` and `docs/archive/TASKS_DONE.md`, which every task edits.
3. **CI time per run is small; the count of runs is the cost.** Gate job 99-161 s (median ~150 s), web build 20-40 s. 33 PRs each ran CI at least once, several 2-8 times. The web build was not the long part; the gate was. Both now run once per batch (BATCH-MERGE, WEB-DAILY).
4. **Dependency chains.** S12S-L commit 1 -> S12S-A (thread started 14:44, its work could only land after S12S-L commits 2-3 merged at 15:28); S14-A1 -> S14-L1 commit 3; UI1-L -> UI1-A -> UI2-L -> SAVE-IO -> UI3-L ran in one thread with ~4 min waits each, which worked well.

**Suggested decoupling**

- Stop area threads editing the shared logs: put the AGENT_MAP line and the TASKS_DONE row in the task's squash commit body, and let the Merge & Release thread write `docs/AGENT_MAP.md` and `docs/archive/TASKS_DONE.md` at each batch, as it does for this file. This removes most merge conflicts (cause 2).
- After merging `develop` into a task branch, re-run the local gate only when `develop` changed files the task touches or shared code; otherwise the batch CI covers it.
- Keep dependent tasks in the same area thread so they chain without waiting on another thread, and land schema slices on `develop` early (no PR wait any more) so data tasks can start.
- Queue spec approvals ahead of time (already a project rule) so `approval` waits do not stall a thread.

## Log (newest first)

Times are UTC. Start = the thread's first visible work for the task (backfill: thread start or first commit); Landed = merged to `main` before BATCH-MERGE, onto `develop` after it.

| Task | Area | Start | Landed | Work | Wait (cause) | Runs | Blocked by | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TOOLS-RO | Gameplay | 17:16 | 17:50 (develop), release #37 | ~30 | approval 0, dependency 0, ci 0, merge ~2 | gate 1 | - | mode standard, tokens ~325k |
| COMBAT-LAYOUT | Gameplay | 17:21 | 17:47 (develop), release #37 | ~26 (build ~23, review ~1, rework ~1, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~96k |
| REVIEW-F4 | Platform | 17:05 | 17:31 (develop), release #37 | ~35 incl. part 1 | approval ~10 butler choice, dependency 0, ci ~3 probes, merge 0 | gate 2 | user choice on the Butler pin (17:06-17:17) | mode economy, tokens ~65k |
| REVIEW-F2 | UI/UX | 16:59 | 17:21 (develop), release #37 | ~22 (build ~16, review ~1, rework 0, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~114k |
| PC-LAUNCH-CACHE | Platform | 17:09 | 17:20 (develop), release #37 | ~8 | approval 0, dependency 0, ci 0, merge 0 | gate 1 | - | mode economy, tokens ~20k |
| REVIEW-F1 | UI/UX | 16:58 | 17:14 (develop), release #37 | ~16 (build ~9, review ~2, rework ~4, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~161k |
| REVIEW-F4 (partial) | Platform | 17:05 | 17:06 (develop; commit says 18:05), release #37 | ~25 | approval 0, dependency 0, ci 0, merge 0, blocked ~35 on butler pin | gate 1 | butler version + sha256 (proxy blocks broth.itch.zone; user decision); REVIEW-F4 row stays open for it | mode economy, tokens ~35k |
| S14-FIX | Gameplay | 16:36 | 17:00 (develop), release #37 | ~24 (build ~18, review ~1, rework ~4, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~84k |
| SHARED-LOGS | Platform | 16:55 | 17:01 (develop), release #37 | ~6 (build 3, review 1, rework 2) | 0 | gate 0 | user decision (card answered 16:55) | mode economy, tokens 25k |
| CONTEXT-HYGIENE | Platform | 16:52 | 16:58 (develop), release #37 | ~6 (build 3, review 1, rework 2) | 0 | gate 0 | - | mode economy, tokens 15k |
| GATE-DEDUPE | Merge & Release | 16:34 | 16:38 (develop), main 16:46 (#35) | ~4 | ~8 release | gate 1 local + CI 1 | - | duplicate --import step left by S14-A1 + WEB2 |
| TIMING + BRANCH-CLEANUP | Platform | 16:35 | 16:55 (develop), main 16:52 (#36) | ~15 | ~3 merge | gate 1 | - | backfill from GitHub API; proxy blocks branch deletes, so a workflow does it |
| WEB3 | UI/UX | 16:34 | 16:53 (develop), main 16:52 (#36) | ~12 | ~4 merge, ~3 gate queue | gate 1 | patch left unpushed by the browser-build thread | built 15:30-ish, idle ~90 min (est.) |
| WEB-DAILY | Platform | 16:31 | 16:42 (develop), main 16:46 (#35) | ~12 | ~4 merge | gate 2 | - | merging develop forced the second gate |
| BATCH-MERGE | Platform | 16:06 | 16:13 (#34) | ~6 | ~4 ci | gate 1 local + CI 1 | - | |
| AREA-THREADS | Platform | 15:55 | 16:05 (#33) | ~6 | ~4 ci, ~1 merge | gate 1 local + CI 1 | - | one AGENT_MAP conflict |
| S14-L1 c3 | Assets | 15:44 | 15:56 (#32) | ~6 | ~6 ci | CI 1 | S14-A1 merge | |
| GRID12 | Gameplay | 15:02 | 16:29 (develop), main 16:46 (#35) | ~41 | ~60 release (est.) | CI 0 before batch | approval order | 1 main merge |
| REVIEW-STEP | Platform | 15:23 | 15:51 (#30) | ~20 | ~4 ci, ~4 approval (est.) | CI 1 | user approval | 1 main merge |
| BAL3 spec | Design | 15:15 | 15:39 (#29) | ~7 | ~17 approval (est.) | CI 1 | user approval | |
| S12S-A | Assets | 14:44 | 16:29 (develop), main 16:46 (#35) | ~56 | ~35 dependency, ~60 release (est.) | CI 2 | S12S-L c2-3 | 1 main merge |
| pc-launch skill | Platform | 15:15 | 15:23 (#27) | ~4 | ~4 ci | CI 1 | user approval | |
| S12S-L c2-3 | Gameplay | 14:43 | 15:28 (#26) | ~36 | ~8 ci | CI 1 | S12S-L c1 | |
| ROADMAP-GOALS | Design | 15:09 | 16:29 (develop), main 16:46 (#35) | ~10 | ~75 approval, release (est.) | CI 0 before batch | user approval | |
| BATTLE1-L | Gameplay | 15:12 | 16:29 (develop), main 16:46 (#35) | ~20 | ~65 release (est.) | CI 0 before batch | approval order | 1 main merge |
| UI3-L | UI/UX | 15:03 | 15:15 (#23) | ~8 | ~4 ci | CI 1 | SAVE-IO | |
| WEB2 | UI/UX | ~15:00 | 16:29 (develop), main 16:46 (#35) | ~26 | ~70 release, approval (est.) | CI 8 | - | 8 CI runs |
| SAVE-IO | UI/UX | 14:56 | 15:03 (#21) | ~3 | ~4 ci | CI 1 | UI2-L | |
| UI2-L | UI/UX | 14:33 | 14:56 (#20) | ~19 | ~4 ci | CI 1 | UI1-A | |
| UI1-A | Assets | 14:29 | 14:33 (#19) | ~1 | ~3 ci | CI 1 | UI1-L | |
| S14-A2 | Assets | 14:22 | 16:29 (develop), main 16:46 (#35) | ~12 | ~50 dependency, ~70 release (est.) | CI 3 | S14-A1 | 1 main merge |
| UI1-L | UI/UX | 14:02 | 14:29 (#17) | ~20 | ~7 ci | CI 1 | - | |
| S14-L3 | Gameplay | 14:10 | 14:22 (#16) | ~9 | ~3 ci | CI 1 | S14-L2 | |
| S12S-L c1 | Gameplay | 14:09 | 14:43 (#15) | ~24 | ~10 merge, ci | CI 1 | approval | 2 main merges |
| WEB1 | Platform | 13:58 | 14:36 (#14) | ~20 | ~18 approval (est.) | CI 1 | - | |
| CLN-16 | Platform | 14:00 | 14:09 (#13) | ~4 | ~4 ci | CI 1 | CLN-12 | |
| BAL2-R | Balance | 14:02 | 15:14 (#12) | ~25 | ~47 merge, approval (est.) | CI 1 | - | 2 main merges, sims ~17 min |
| DLG1-F | Gameplay | 14:01 | 15:22 (#11) | ~2 | ~80 approval, merge (est.) | CI 1 | - | 3 main merges |
| DEC-BATCH | Design | 13:58 | 14:01 (#10) | ~1 | ~3 ci | CI 1 | - | |
| S14-L2 | Gameplay | 13:36 | 14:10 (#9) | ~17 | ~17 ci, approval (est.) | CI 1 | - | |
| CLN-12 | Platform | 13:35 | 13:59 (#8) | ~17 | ~7 ci | CI 1 | - | |
| PLAYTEST1-L | Gameplay | 13:39 | 16:29 (develop), main 16:46 (#35) | ~115 | ~65 release (est.) | CI 1 | spec approval (13:50-13:55) | |
| DLG1-L c3 | Gameplay | 13:28 | 13:50 (#6) | ~18 | ~3 ci | CI 1 | - | |
| S14-A1 | Assets | 13:36 | 15:31 (#5) | ~16 | ~95 approval, merge (est.) | CI 1 | merge order | 3 main merges |
| S14-F | Assets | ~13:28 | 13:36 (#4) | ~5 | ~3 ci | CI 1 | - | |
| WF-ENV | Platform | 13:06 | 13:24 (#3) | ~15 | ~3 ci | CI 1 | - | |
