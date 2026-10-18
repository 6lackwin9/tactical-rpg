# Task log (time per task, waits and bottlenecks)

Owner: the Merge & Release thread. It compiles the `Timing:` and `Blocked-by:` lines from each task's squash commit on `develop` into this file at every release batch (area threads never edit it, so it never conflicts). Format and rules: `docs/workflow/cloud.md` "Task timing". Goal (user 2026-10-17): find idle time and dependencies between steps first; tokens are secondary. Rotate rows older than a month to `docs/metrics/archive/TASK_LOG_<yyyy-mm>.md`.

Work is time a thread is actively building, reviewing, gating or fixing. Wait is time the task is done or blocked and nobody works on it, split by cause: `approval` (the user), `dependency` (another task or its merge), `ci`, `merge` (conflicts with `develop`/`main` and the re-gate they force) and `release` (landed on `develop`, waiting for the batch to reach `main`).

## Bottlenecks (updated each batch; newest first)

### 2026-10-18 midday release (124 task commits, first release since #37)

1. **Release wait dominated: up to ~18 h.** Nothing reached `main` between release #37 (2026-10-17 17:57) and this batch, so the overnight landings (S12C-L, TRAV1-L, REP1, AUTO1-L, CAMP1, MUSIC1/2, RECRUIT1, OBJ1, BOSS1, INTENT1, HEAL1, ...) waited 1-18 h on `develop`. The user tests on `develop`, so this cost no playtest time; it only delays `main` (PC `run.bat` and balance runs on `main`). Decouple: none needed while testing stays on `develop`; ask for "release now" before a balance run needs fresh code.
2. **Approval waits are the only long in-task waits:** BREPORT1 spec ~312 min and BAL5 spec ~265 min (user away overnight). Everything else waited 0-25 min. Decouple: keep queuing approvals as project-chat cards early (already the rule), and keep threads busy on approved rows meanwhile.
3. **Merges with `develop` are the main work-time leak:** UIR-2 spent 14 min on conflicts (+13 min merge wait, 4 gates), TRAV1-L 8 min (4 gates), S12C-L 6 min (6 gates), S12T-L 5 min (5 gates). These are large UI/logic tasks touching shared screens while other threads land. Decouple: land a schema or shared-file slice first (SPLIT-LANDING), and merge `develop` once right before the final gate instead of after each landing.
4. **Outside stalls:** OBJ1-L ~17 min (rate limit), COMBAT-UI2-L1 ~15 min (usage limit), INTENT1-A ~10 min (API error). Not dependency-driven; the USAGE-CAPS rules (building slots, background sims) target them.
5. **Tokens (rows with the new `tokens=` field):** logic-builder ~1.2M, change-reviewer ~0.4M, design ~0.24M, balance ~0.08M; by type ui ~755k over 3 tasks and logic ~774k over 4. UIR-2 alone was ~494k (logic 424k after 15 min of rework). Format drift: several commits used free text in `AgentMap:`/`Done:` (Balance BAL3/BAL4 rows, CLN-3b, RUNNER-WINDOW); they are logged here with minimal rows.

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
| USAGE | Platform | - | 10-18 11:49 (develop), release midday 2026-10-18 | - | - | gate - | - | type docs, tokens none |
| BAL5-AI | Balance | 10:48 | 10-18 11:44 (develop), release midday 2026-10-18 | ~50 (build ~44, review 2, rework ~4, conflicts 0) | ~5 (approval 0, dependency 0, ci 0, merge ~5) | gate 1 | BAL5-R1 (landed f3d8ac2) | mode economy, type logic, tokens logic:164k,review:51k |
| UIR-2 | Gameplay | 10:17 | 10-18 11:40 (develop), release midday 2026-10-18 | 70 (build 39, review 2, rework 15, conflicts 14) | 13 (approval 0, dependency 0, ci 0, merge 13) | gate 4 | - | mode economy, type ui, tokens logic:424k,review:70k |
| HEAL1-B | Balance | 10:48 | 10-18 11:20 (develop), release midday 2026-10-18 | ~30 (build ~29, review 0, rework 0, conflicts 0) | ~0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | HEAL1-S (landed 47ef3b4) | mode economy, type sim, tokens balance:35k |
| MAPKEYS1-U | UI/UX | 09:53 | 10-18 11:05 (develop), release midday 2026-10-18 | ~68 (build 40, review 3, rework 20, conflicts 5) | ~4 (approval 0, dependency 0, ci 0, merge 4) | gate 2 | - | mode economy, type ui, tokens logic:116k,review:50k |
| BAL5-R1 | Balance | 08:43 | 10-18 10:42 (develop), release midday 2026-10-18 | ~120 (build ~120, review 0, rework 0, conflicts 0) | ~0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | BAL5-SIM (landed c78b8ba) | mode economy, type sim, tokens balance:48k |
| RUNNER-WINDOW | Platform | - | 10-18 10:20 (develop), release midday 2026-10-18 | - | - | gate - | - | type tool, tokens review:19k |
| BREPORT1-L | Gameplay | 09:05 | 10-18 10:17 (develop), release midday 2026-10-18 | 58 (build 31, review 2, rework 3, conflicts 0, gates 22) | 14 (approval 0, dependency 0, ci 0, merge 14) | gate 3 | - | mode economy, type logic, tokens logic:221k,review:74k |
| WMAP-BLANK | Gameplay | 09:53 | 10-18 10:06 (develop), release midday 2026-10-18 | 13 (build 13, review 0, rework 0, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, type logic, tokens none |
| BOSS1-U | UI/UX | 08:42 | 10-18 10:18 (develop), release midday 2026-10-18 | ~85 (build 80, review 1, rework 0, conflicts 4) | ~10 (approval 0, dependency 0, ci 0, merge 10) | gate 1 | - | mode economy, type ui, tokens logic:67k,review:28k |
| PACE1 | Design | 09:40 | 10-18 09:42 (develop), release midday 2026-10-18 | ~2 | 0 | gate 0 | - | mode standard, type spec |
| PACE1 | Design | 09:35 | 10-18 09:46 (develop), release midday 2026-10-18 | ~11 | 0 | gate 0 | - | mode standard, type spec |
| PACE1 | Design | 09:25 | 10-18 09:40 (develop), release midday 2026-10-18 | ~15 | ~5 (approval 5) | gate 0 | user approval 09:31-09:33 | mode standard, type spec, tokens design:112k |
| PILL-GAME | Platform | 09:15 | 10-18 09:45 (develop), release midday 2026-10-18 | ~20 (build ~9, review ~1, rework 0, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~34k |
| PACE1-SPEC | Design | 09:25 | 10-18 09:36 (develop), release midday 2026-10-18 | ~11 | 0 | gate 0 | - | mode standard, type spec |
| HEAL1-S | Balance | 09:05 | 10-18 09:27 (develop), release midday 2026-10-18 | ~22 (build 11, review 1, rework 0, conflicts 0) | ~5 (approval 0, dependency 0, ci 0, merge ~5) | gate 1 | HEAL1-L (landed 0832b03) | mode economy, type sim, tokens logic:49k,review:30k |
| USAGE-TRACKING fix | Platform | 09:06 | 10-18 09:07 (develop), release midday 2026-10-18 | ~1 | 0 | gate 0 | - | mode standard, type tool, tokens none |
| MORALE1 | Design | 08:42 | 10-18 09:10 (develop), release midday 2026-10-18 | ~20 | ~8 (approval 8) | gate 0 | user approval 08:58-09:03 | mode standard, type spec, tokens design:123k |
| HEAL1-L | Gameplay | 08:13 | 10-18 09:12 (develop), release midday 2026-10-18 | ~59 (build 41, review 4, rework 10, conflicts 1) | 0 | gate 3 | - | mode standard->economy, type logic, tokens logic:184k,review:80k |
| USAGE-TRACKING | Platform | 08:48 | 10-18 08:56 (develop), release midday 2026-10-18 | ~8 | 0 | gate 0 | - | mode standard, type docs, tokens none |
| MORALE1-APPROVE | Design | 08:42 | 10-18 08:58 (develop), release midday 2026-10-18 | ~16 | 0 | gate 0 | - | mode standard |
| TOKENS-PER-AGENT | Platform | 08:46 | 10-18 08:48 (develop), release midday 2026-10-18 | ~2 | 0 | gate 0 | - | mode standard, type docs, tokens none |
| BAL5-SIM + BAL5-PH | Balance | ~07:55 | 10-18 08:43 (develop), release midday 2026-10-18 | ~40 (build ~30, review 1, rework 1, conflicts 0) | ~8 (approval 0, dependency 0, ci 0, merge ~8) | gate 2 | - | mode economy, tokens ~67k (reviewer; builder in previous session) |
| DR2-ANSWERS | Design | 08:38 | 10-18 08:46 (develop), release midday 2026-10-18 | ~8 | 0 | gate 0 | - | mode standard |
| HEAL1-A | Assets | 08:29 | 10-18 08:45 (develop), release midday 2026-10-18 | ~12 (build 3, review 1, rework 0, conflicts 0) | ~3 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens 12k, squash 85ac473 mistitled "Merge develop" |
| MORALE1-SPEC | Design | 08:24 | 10-18 08:40 (develop), release midday 2026-10-18 | ~16 | 0 | gate 0 | - | mode standard |
| DESIGN-PAUSE | Design | 08:29 | 10-18 08:31 (develop), release midday 2026-10-18 | ~2 | 0 | gate 0 | - | mode standard |
| INTENT1-U | UI/UX | 07:31 | 10-18 08:40 (develop), release midday 2026-10-18 | ~65 (build 40, review 2, rework 15, conflicts 3) | ~4 (approval 0, dependency 0, ci 0, merge 4) | gate 1 | - | mode economy, tokens ~261k |
| ASSETS-HANDOVER | Assets | 08:44 | 10-18 08:46 (develop), release midday 2026-10-18 | - | - | gate - | - |  |
| ITEMUSE1-A2 | Assets | - | 10-18 08:27 (develop), release midday 2026-10-18 | - | - | gate - | user decision (potion card) |  |
| SONNET-TRIAL | Platform | 08:22 | 10-18 08:23 (develop), release midday 2026-10-18 | ~1 | 0 | gate 0 | - | mode standard |
| USAGE-CAPS | Platform | 08:20 | 10-18 08:27 (develop), release midday 2026-10-18 | ~7 | 0 | gate 0 | - | mode standard |
| UIR-1 | Gameplay | 07:45 | 10-18 08:25 (develop), release midday 2026-10-18 | ~40 (build 25, review 2, rework 3, conflicts 0) | 0 | gate 1 | - | mode economy->standard, tokens ~125k |
| CLN-3b | Platform | 07:55 | 10-18 08:12 (develop), release midday 2026-10-18 | ~17 (build 5, review 1, rework 1, conflicts 0, gates 2 baseline+final) | 0 | gate 2 | - | mode standard, tokens ~31k |
| HEAL1 (spec approved) | Design | 07:56 | 10-18 08:12 (develop), release midday 2026-10-18 | ~16 (build ~12, rework ~4) | ~4 (approval 4) | gate 2 | user approval 08:04-08:08 | mode standard, tokens ~160k |
| HEAL1 (spec) | Design | 07:56 | 10-18 08:04 (develop), release midday 2026-10-18 | ~12 (build ~12) | 0 | gate 1 | BAL4-ATTR v4 (landed 07:51) | mode standard, tokens ~ |
| MODE-STANDARD | Platform | 07:54 | 10-18 07:56 (develop), release midday 2026-10-18 | ~2 | 0 | gate 0 | - | mode standard |
| BAL5 (spec approved) + Orsa quest answer | Design | 03:23 | 10-18 07:53 (develop), release midday 2026-10-18 | ~10 | ~265 (approval 265) | gate 1 | user approval 03:33-07:53 | mode standard, tokens ~113k |
| BREPORT1 (spec approved) | Design | 02:37 | 10-18 07:51 (develop), release midday 2026-10-18 | ~14 | ~312 (approval 312) | gate 1 | user approval 02:44-07:51 | mode standard, tokens ~104k |
| RECRUIT1-B, OBJ1-B, BOSS1-B | Balance | 06:25 | 10-18 07:51 (develop), release midday 2026-10-18 | ~80 (build ~19, review 1, rework 2, conflicts 2, sims ~55) | ~5 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | none (run from develop instead of waiting for main) | mode economy, tokens ~180k |
| EXPWORLD | Gameplay | 07:33 | 10-18 07:45 (develop), release midday 2026-10-18 | ~12 (build 10, review 1, rework 1, conflicts 0) | 0 | gate 1 | - | mode economy, tokens ~50k |
| QUESTX | Gameplay | 07:03 | 10-18 07:31 (develop), release midday 2026-10-18 | ~29 (build 18, review 1, rework 8, conflicts 0) | 0 | gate 2 | - | mode economy, tokens ~95k |
| ITEMUSE1-U | UI/UX | 07:18 | 10-18 07:30 (develop), release midday 2026-10-18 | 12 (build 3, review 1, rework 0, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~69k |
| COMBAT-UI2-L3 | UI/UX | 06:30 | 10-18 07:17 (develop), release midday 2026-10-18 | 47 (build 14, review 1, rework 11, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~279k |
| ITEMUSE1-L | Gameplay | 06:07 | 10-18 07:02 (develop), release midday 2026-10-18 | ~62 (build 45, review 1, rework 12, conflicts 4) | 0 | gate 3 | - | mode economy, tokens ~236k |
| COMBAT-UI2-L2 | UI/UX | 05:37 | 10-18 06:28 (develop), release midday 2026-10-18 | 51 (build 32, review 1, rework 0, conflicts 1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~184k |
| BAL4-ATTR | Balance | 03:54 | 10-18 06:24 (develop), release midday 2026-10-18 | ~120 (build ~25, review 1, rework ~10, conflicts 0, sims ~85) | ~25 (approval 0, dependency 25, ci 0, merge 0) | gate 3 | develop data changes mid-run (RECRUIT1-A, OBJ1-A, BOSS1-A) forced two reruns | mode economy, tokens ~210k |
| ITEMUSE1-A | Assets | - | 10-18 06:19 (develop), release midday 2026-10-18 | - | - | gate - | ITEMUSE1-L schema (ad83b98) |  |
| ITEMUSE1-L (schema) | Gameplay | 05:35 | 10-18 06:07 (develop), release midday 2026-10-18 | 32 (build 30, review 1, rework 0, conflicts 1) | 0 | gate 1 | COMBAT-UI2-L1 | mode economy, tokens ~165k |
| ART2-A | Assets | - | 10-18 06:03 (develop), release midday 2026-10-18 | - | - | gate - | - |  |
| PLATFORM-HANDOVER | Platform | 05:38 | 10-18 05:41 (develop), release midday 2026-10-18 | ~3 | 0 | gate 0 | - | mode economy |
| OBJ1-A | Assets | 05:03 | 10-18 05:36 (develop), release midday 2026-10-18 | ~33 (build 3, review 1, rework 10, conflicts 0) | ~0 (approval 0, dependency 0, ci 0, merge 0) | gate 3 | - | mode economy, tokens ~95k |
| COMBAT-UI2-L1 | UI/UX | 04:30 | 10-18 05:32 (develop), release midday 2026-10-18 | ~45 (build 37, review 1, rework 14, conflicts 1) | ~15 (approval 0, dependency 0, ci 0, merge 0, usage limit 15) | gate 3 | usage limit 04:35-04:50 | mode economy, tokens ~525k |
| OBJ1-L | Gameplay | 04:08 | 10-18 05:16 (develop), release midday 2026-10-18 | ~38 (build 31, review 1, rework 9, conflicts 0) | ~17 (approval 0, dependency 0, ci 0, merge 0, rate limit ~17) | gate 4 | - | mode economy, tokens ~247k |
| BOSS1-A | Assets | 04:53 | 10-18 05:03 (develop), release midday 2026-10-18 | ~10 (build 1, review 0.5, rework 0, conflicts 0) | ~0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~45k |
| INTENT1-A | Assets | 04:21 | 10-18 04:56 (develop), release midday 2026-10-18 | ~25 (build 5, review 1, rework 8, conflicts 2) | ~10 (approval 0, dependency 0, ci 0, merge 0, api error ~10) | gate 2 | - | mode economy, tokens ~37k |
| MAPREAD1-L2 | UI/UX | 03:16 | 10-18 04:33 (develop), release midday 2026-10-18 | ~60 (build 40, review 1, rework 25, conflicts 2) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 3 | - | mode economy, tokens ~390k |
| RECRUIT1-A | Assets | - | 10-18 04:20 (develop), release midday 2026-10-18 | - | - | gate - | RECRUIT1-L behaviour (0d6a3ae) |  |
| OBJ1-L (schema) | Gameplay | 03:46 | 10-18 04:08 (develop), release midday 2026-10-18 | 22 (build 19, review 1, rework 0, conflicts 0) | 0 | gate 1 | - | mode economy, tokens ~145k |
| BAL3-R + CAMP1-B | Balance | 01:00 | 10-18 03:53 (develop), release midday 2026-10-18 | ~165 (build ~150, review 1, rework 0, conflicts 0) | ~10 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode standard->economy, tokens ~136k |
| BOSS1-L | Gameplay | 03:20 | 10-18 03:45 (develop), release midday 2026-10-18 | 25 (build 17, review 2, rework 0, conflicts 1) | 0 | gate 2 | - | mode economy, tokens ~185k |
| MAPREAD1-A | Assets | - | 10-18 03:40 (develop), release midday 2026-10-18 | - | - | gate - | MAPREAD1-L1 (bf162bb), MAPVIEW1 (cf333ab), ART1-A (98d6da5) |  |
| BAL5 (spec) | Design | 03:23 | 10-18 03:30 (develop), release midday 2026-10-18 | ~8 (build ~8) | 0 | gate 1 | - | mode economy, tokens ~113k |
| ART1-A | Assets | - | 10-18 03:24 (develop), release midday 2026-10-18 | - | - | gate - | - |  |
| BOSS1-L (schema) | Gameplay | 02:58 | 10-18 03:20 (develop), release midday 2026-10-18 | 22 (build 13, review 1, rework 0, conflicts 2) | 0 | gate 2 | - | mode economy, tokens ~128k |
| MAPVIEW1 | UI/UX | 02:34 | 10-18 03:22 (develop), release midday 2026-10-18 | 48 (build 34, review 1, rework 0, conflicts 1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode standard->economy, tokens ~170k |
| MAPREAD1-L1 | Gameplay | 02:42 | 10-18 03:10 (develop), release midday 2026-10-18 | 28 (build 21, review 1, rework 6, conflicts 0) | 0 | gate 1 | - | mode economy, tokens ~122k |
| INTENT1-L | Gameplay | 02:33 | 10-18 02:57 (develop), release midday 2026-10-18 | 24 (build 15, review 1, rework 0, conflicts 0) | 0 | gate 2 | - | mode economy, tokens ~140k |
| OPTLAUNCH1 | Platform | 02:36 | 10-18 03:00 (develop), release midday 2026-10-18 | ~24 (build ~12, review ~1, rework ~2, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~53k |
| BREPORT1 (spec) | Design | 02:37 | 10-18 02:42 (develop), release midday 2026-10-18 | ~12 (build ~12) | 0 | gate 1 | - | mode standard, tokens ~ |
| RECRUIT1-L | Gameplay | 02:20 | 10-18 02:41 (develop), release midday 2026-10-18 | 21 (build 12, review 1, rework 0, conflicts 3) | 0 | gate 1 | - | mode economy, tokens ~114k |
| MODE-ECONOMY | Platform | 02:37 | 10-18 02:39 (develop), release midday 2026-10-18 | ~2 | 0 | gate 0 | - | mode economy |
| DESIGN-REVIEW-CYCLE | Design | 02:33 | 10-18 02:36 (develop), release midday 2026-10-18 | 4 (build 4, review 0, rework 0, conflicts 0) | 0 | gate 1 | - | mode standard, tokens 0 |
| CONTROLS-HELP | UI/UX | 01:50 | 10-18 02:38 (develop), release midday 2026-10-18 | ~45 (build ~38, review ~1, rework ~4, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~180k |
| RECRUIT1-L (schema) | Gameplay | 01:56 | 10-18 02:42 (develop), release midday 2026-10-18 | 46 (build 22, review 3, rework 0, conflicts 0) | 0 | gate 1 | - | mode economy, tokens ~105k |
| MUSIC2-A + MUSIC2-L | Assets | 01:45 | 10-18 02:13 (develop), release midday 2026-10-18 | 28 (build 18, review 2, rework 3, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode standard, tokens ~90k |
| WORLDMAP-ENTER | Gameplay | 02:03 | 10-18 02:22 (develop), release midday 2026-10-18 | 19 (build 6, review 1, rework 0, conflicts 0) | 0 | gate 1 | - | mode economy, tokens ~36k |
| SOUND-PREVIEW | UI/UX | 00:58 | 10-18 01:50 (develop), release midday 2026-10-18 | ~35 (build ~22, review ~1, rework ~6, conflicts ~1, regate ~5) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~115k |
| MUSIC1-A + MUSIC1-L | Assets | 00:59 | 10-18 01:44 (develop), release midday 2026-10-18 | 45 (build 25, review 4, rework 8, conflicts 0) | 3 (approval 3, dependency 0, ci 0, merge 0) | gate 2 | - | mode standard, tokens ~260k |
| POI-FIX | Gameplay | 00:45 | 10-18 01:35 (develop), release midday 2026-10-18 | ~42 (build 29, review 1, rework 0, conflicts 4) | ~8 (approval 0, dependency 0, ci 0, merge 8) | gate 2 | - | mode economy, tokens ~158k |
| AUTO1-L (logic) | Gameplay | 00:36 | 10-18 01:22 (develop), release midday 2026-10-18 | ~44 (build 27, review 1, rework 4, conflicts 2) | ~2 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~201k |
| NOTE-BUTTON | Platform | 00:59 | 10-18 01:15 (develop), release midday 2026-10-18 | ~16 (build ~8, review ~1, rework 0, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~44k |
| BAL3-A | Balance | 22:41 | 10-18 02:10 (develop), release midday 2026-10-18 | ~205 (build ~190, review 2, rework ~10, conflicts 2) | ~5 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode standard, tokens ~150k |
| BATTLE-FLOW | UI/UX | 00:03 | 10-18 00:58 (develop), release midday 2026-10-18 | ~50 (build ~30, review ~3, rework ~12, conflicts ~1, regate ~4) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~190k |
| TRAV2-A | Assets | 00:14 | 10-18 00:40 (develop), release midday 2026-10-18 | 12 (build 3, review 4, rework 1, conflicts 0) | 16 (approval 0, dependency 16, ci 0, merge 0) | gate 1 | merge of TRAV2-L commit 2 | mode standard, tokens ~75k |
| TRAV2-L | Gameplay | 23:58 | 10-18 00:36 (develop), release midday 2026-10-18 | ~38 (build 25, review 1, rework 0, conflicts 1) | ~0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~101k |
| AUTO1 (spec) | Design | - | 10-18 00:19 (develop), release midday 2026-10-18 | - | - | gate - | user approval 00:12-00:17 |  |
| CAMP1-A1 + CAMP1-A2 | Assets | 23:39 | 10-18 00:13 (develop), release midday 2026-10-18 | 34 (build 18, review 2, rework 0, conflicts 2) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode standard, tokens ~194k |
| SETTINGS-POPUP | UI/UX | 23:33 | 10-18 00:05 (develop), release midday 2026-10-18 | ~31 (build ~29, review ~1, rework 0, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~255k |
| TRAV2-L (schema) | Gameplay | 23:26 | 10-17 23:56 (develop), release midday 2026-10-18 | ~26 (build ~14, review 1, rework 0, conflicts 1) | ~4 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~235k |
| CAMP1 | Design | - | 10-17 23:40 (develop), release midday 2026-10-18 | - | - | gate - | user approval (spec posted ~23:36, approved 23:39 UTC) |  |
| CAMP1 (spec) | Design | - | 10-17 23:37 (develop), release midday 2026-10-18 | - | - | gate - | - |  |
| BAL3-AI | Balance | 20:36 | 10-17 22:40 (develop), release midday 2026-10-18 | ~118 (build ~83, review 2, rework ~15, conflicts 0) | ~6 (approval 0, dependency 0, ci 0, merge 0) | gate 3 | none (user card on lever C scope open since 21:58; landed on the recommended option) | mode standard, tokens ~330k |
| FORGE-VIEW | Platform | 21:28 | 10-17 22:19 (develop), release midday 2026-10-18 | ~51 (build ~42, review ~1, rework 0, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~280k |
| MENU-POLISH + DISPLAY1-FIX | UI/UX | 21:43 | 10-17 22:14 (develop), release midday 2026-10-18 | ~30 (build ~24, review ~1, rework ~5, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~150k |
| REP1-L | Gameplay | 21:52 | 10-17 22:10 (develop), release midday 2026-10-18 | 18 (build 11, review 1, rework 0, conflicts 2, gate 3) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens ~271k |
| REP1-A | Assets | 21:49 | 10-17 22:05 (develop), release midday 2026-10-18 | 16 (build 6, review 1, rework 0, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | REP1-L (schema) landed 21:49 | mode economy, tokens 45609 |
| REP1-L (schema) | Gameplay | 21:23 | 10-17 21:52 (develop), release midday 2026-10-18 | 29 (build 16, review 1, rework 0, conflicts 2, gate 9) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 3 | - | mode economy, tokens ~168k |
| TRAV1-A | Assets | 21:16 | 10-17 21:45 (develop), release midday 2026-10-18 | 29 (build 14, review 1, rework 1, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | - | mode economy, tokens 65529 |
| DISPLAY1 | UI/UX | 21:00 | 10-17 21:48 (develop), release midday 2026-10-18 | ~44 (build ~29, review ~1, rework ~7, conflicts ~1) | ~6 (approval 0, dependency 0, ci 0, merge ~6) | gate 3 | SETTINGS-UI (level mechanism), landed 21:00; user approval 19:58-20:28 | mode economy, tokens ~115k |
| PULL-RESTART | Platform | 20:58 | 10-17 21:27 (develop), release midday 2026-10-18 | ~29 (build ~18, review ~2, rework ~4, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens ~77k |
| SAVE-COMPAT | Gameplay | 20:53 | 10-17 21:23 (develop), release midday 2026-10-18 | 30 (build 19, review 1, rework 2, conflicts 0, gate 4) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 3 | - | mode economy, tokens ~70k |
| S12C-A | Assets | 20:53 | 10-17 21:17 (develop), release midday 2026-10-18 | 24 (build 17, review 1, rework 1, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens 50388 |
| SETTINGS-UI | UI/UX | 20:09 | 10-17 21:00 (develop), release midday 2026-10-18 | ~45 (build ~40, review ~1, rework 0, conflicts ~1) | ~6 (approval 0, dependency 0, ci 0, merge ~6) | gate 2 | LOAD-SLOTS-UI (shared menu files), landed 20:08 | mode economy, tokens ~170k |
| GATE-ONCE | Platform | 20:53 | 10-17 21:02 (develop), release midday 2026-10-18 | ~7 (build ~3, review ~1, rework ~2, conflicts 0) | ~2 (approval ~2 user word 20:53-20:55, dependency 0, ci 0, merge 0) | gate 0 | user's own word for the decision row (20:53-20:55) | mode economy, tokens ~19k |
| SPLIT-LANDING | Platform | 20:52 | 10-17 21:00 (develop), release midday 2026-10-18 | ~8 (build ~4, review ~1, rework ~3, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 0 | - | mode economy, tokens ~16k |
| HANDOVER-GAMEPLAY | Gameplay | 20:53 | 10-17 20:56 (develop), release midday 2026-10-18 | ~3 (build ~3, review 0, rework 0, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | - | mode economy, tokens 0 |
| TRAV1-L | Gameplay | 19:43 | 10-17 20:52 (develop), release midday 2026-10-18 | ~69 (build ~60, review ~1, rework 0, conflicts ~8) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 4 | - | mode economy, tokens ~302k |
| BAL3-S | Balance | 18:53 | 10-17 20:35 (develop), release midday 2026-10-18 | ~95 (build ~76, review 2, rework ~10, conflicts 0) | ~7 (approval ~7, dependency 0, ci 0, merge 0) | gate 3 | user decision card (ranged_dive) 19:50-20:29, work continued on the recommended option | mode standard, tokens ~213k |
| PLATFORM-HANDOVER | Platform | 20:21 | 10-17 20:42 (develop), release midday 2026-10-18 | (~8) | (approval | gate 0 | - | mode economy, tokens ~20k |
| SAVE-BACKUP | Platform | 20:21 | 10-17 20:36 (develop), release midday 2026-10-18 | (~12) | (approval | gate 1 | - | mode economy, tokens ~25k |
| DEV-USERDATA | Platform | 20:15 | 10-17 20:24 (develop), release midday 2026-10-18 | (~6) | (approval | gate 1 | - | mode economy, tokens ~20k |
| PULL-ON-LAUNCH | Platform | 20:01 | 10-17 20:15 (develop), release midday 2026-10-18 | (~12) | (approval | gate 1 | user choice on one or two folders (20:06-20:08) | mode economy, tokens ~60k |
| WEB-PAUSE | Platform | 19:58 | 10-17 20:06 (develop), release midday 2026-10-18 | (~8) | (approval | gate 1 | - | mode economy, tokens ~25k |
| LOAD-SLOTS-UI | UI/UX | 19:47 | 10-17 20:08 (develop), release midday 2026-10-18 | ~16 (build ~6, review ~1, rework ~8, conflicts 0) | ~5 (approval 0, dependency 0, ci 0, merge ~5) | gate 2 | - | mode economy, tokens ~65k |
| DEPLOY-AFTER-RELEASE | Platform | 19:32 | 10-17 20:00 (develop), release midday 2026-10-18 | (~12) | (approval | gate 1 | user choice on the deploy trigger (19:32-19:43) | mode economy, tokens ~25k |
| RELEASE-2X | Merge & Release | 19:43 | 10-17 19:47 (develop), release midday 2026-10-18 | 4 | 0 | gate 0 | - | mode economy |
| S12C-L | Gameplay | 18:40 | 10-17 19:42 (develop), release midday 2026-10-18 | ~62 (build ~50, review ~1, rework ~5, conflicts ~6) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 6 | S12T-L (landed 18:18Z) | mode economy, tokens ~281k |
| SIM-IMPORT | Platform | 18:54 | 10-17 19:07 (develop), release midday 2026-10-18 | (~13) | (approval | gate 1 | - | mode economy, tokens ~45k |
| BAL3-R0 | Balance | 17:00 | 10-17 18:53 (develop), release midday 2026-10-18 | ~113 (build ~110, review 2, rework 1, conflicts 0) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 1 | none (40 min of the build was a stale class cache hang) | mode standard, tokens ~126k |
| S12T-A | Gameplay | 18:18 | 10-17 18:40 (develop), release midday 2026-10-18 | ~22 (build ~5, review ~1, rework ~12, conflicts ~4) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 3 | S12T-L (landed 18:18Z) | mode economy, tokens ~65k |
| REVIEW-F5 | Platform | 17:59 | 10-17 18:25 (develop), release midday 2026-10-18 | (~22) | (approval | gate 2 | REVIEW-F1, REVIEW-F2, REVIEW-F3 (landed 17:58) | mode standard, tokens ~70k |
| S12T-L | Gameplay | 17:14 | 10-17 18:18 (develop), release midday 2026-10-18 | ~64 (build ~45, review ~2, rework ~10, conflicts ~5) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 5 | - | mode economy, tokens ~292k |
| REVIEW-F3 | UI/UX | 17:47 | 10-17 17:58 (develop), release midday 2026-10-18 | ~11 (build ~8, review ~1, rework 0, conflicts ~1) | 0 (approval 0, dependency 0, ci 0, merge 0) | gate 2 | REVIEW-F2 (shared menus_test_suite), landed 17:21 | mode economy, tokens ~75k |
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
