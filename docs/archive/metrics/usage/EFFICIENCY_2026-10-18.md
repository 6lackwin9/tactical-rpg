# Efficiency report, 2026-10-18 (benchmark: usage report 07:57 UTC)

Compares each thread session's usage (Tushar's project usage report) with what its commits logged (`Timing:` lines: tasks, work minutes, helper-agent tokens). Commits map to sessions exactly through their `Claude-Session` trailer; usage rows map by title, and repeated titles (later Gameplay, UI/UX, Assets & Audio sessions) are matched by session length, marked "guess". "Tasks landed" counts every commit with a `Timing:` line, including handovers and decision rows. Rerun: `python3 docs/metrics/usage/efficiency.py <repo> <report.csv> <report time>`.

## Findings

- Helper agents are a rounding error in the logs: their logged tokens are 11.7M against 872M of thread usage (1.3%). Either the usage report counts only the threads' own conversations, or the agents' `subagent_tokens` figure undercounts what they really bill (it looks like the final context size, not every step's re-read). Both readings point the same way: the residual, about 98%, is the area threads' own turns: reading files, running commands, relays and wakes, each one re-reading the thread's whole context.
- Usage per landed task by thread: Assets & Audio 3.4M to 4.2M, Design 2.4M to 3.0M, Platform 0.8M to 6.5M, UI/UX 7.6M to 18.3M, Gameplay 4.5M to 12.6M, Balance 14.0M, Merge & Release 11.3M per release batch.
- UI/UX (02:33 to 05:36) is the least efficient build session: 54.8M for 3 tasks (18.3M each), during the 04:30 rate-limit waits. Balance spends 14M per task, mostly long sim runs while its context grew for 15 hours.
- Assets & Audio missing agent counts: several Assets & Audio commits logged no `tokens=` field, so agent shares there read 0%.

## What changes now

- From 2026-10-18 08:48 (7abbb56) the `Timing:` line logs `type=` and `tokens=<agent>:<k>` per agent kind, so the next reports can split usage by task type and agent.
- Next check: with the following usage report, the delta per session minus its logged agent tokens gives each thread's own residual; a thread whose residual per task stays high is the one to slim (fewer steps, rotate sooner, or move work into agents).

## Per session

| Session (title, window UTC) | Report tokens | Tasks landed | Report tokens per task | Agent tokens logged | Agent share | Task types | Mapping |
|---|---|---|---|---|---|---|---|
| Platform & Workflow (10-17 15:55-20:29) | 111.1M | 17 | 6.5M | 0.81M | 0.7% | ? 17 | exact |
| Merge & Release (10-17 16:14-07:57) | 34.0M | 3 | 11.3M | 0.00M | 0.0% | ? 3 | exact |
| Gameplay (10-17 16:34-20:52) | 75.9M | 7 | 10.8M | 1.19M | 1.6% | ? 7 | exact |
| UI/UX (10-17 16:58-02:32) | 91.0M | 11 | 8.3M | 1.52M | 1.7% | ? 11 | exact |
| Balance (10-17 16:59-07:57) | 98.3M | 7 | 14.0M | 1.34M | 1.4% | ? 7 | exact |
| Platform & Workflow (10-17 20:29-07:50) | 45.2M | 8 | 5.7M | 0.49M | 1.1% | ? 8 | exact |
| Gameplay (10-17 20:52-22:08) | 21.2M | 3 | 7.1M | 0.51M | 2.4% | ? 3 | guess |
| Assets & Audio (10-17 20:53-22:02) | 10.2M | 3 | 3.4M | 0.00M | 0.0% | ? 3 | guess |
| Assets & Audio (10-17 22:08-22:11) | 2.7M | 0 | - | 0.00M | 0.0% | - | guess |
| Gameplay (10-17 23:25-01:35) | 44.5M | 4 | 11.1M | 0.69M | 1.6% | ? 4 | guess |
| Design (10-17 23:25-23:40) | 6.0M | 2 | 3.0M | 0.00M | 0.0% | ? 2 | exact |
| Assets & Audio (10-17 23:39-00:43) | 8.5M | 2 | 4.2M | 0.27M | 3.2% | ? 2 | guess |
| Battle autoplay tactics (10-18 00:06-00:19) | 3.5M | 1 | 3.5M | 0.00M | 0.0% | ? 1 | exact |
| Design review (10-18 00:51-02:11) | 19.2M | 0 | - | 0.00M | 0.0% | - | exact |
| Menu music track (10-18 00:59-02:17) | 24.6M | 2 | 12.3M | 0.35M | 1.4% | ? 2 | exact |
| Gameplay (10-18 01:55-03:45) | 41.8M | 7 | 6.0M | 0.83M | 2.0% | ? 7 | guess |
| Assets and art (10-18 02:24-04:26) | 26.0M | 3 | 8.7M | 0.00M | 0.0% | ? 3 | exact |
| Design reviews and option builds (10-18 02:32-07:57) | 17.0M | 7 | 2.4M | 0.49M | 2.9% | ? 7 | exact |
| UI/UX (10-18 02:33-05:36) | 54.8M | 3 | 18.3M | 1.08M | 2.0% | ? 3 | guess |
| Gameplay (10-18 03:45-07:33) | 63.1M | 5 | 12.6M | 0.89M | 1.4% | ? 5 | guess |
| Assets & Audio (10-18 04:21-05:38) | 10.7M | 3 | 3.6M | 0.18M | 1.7% | ? 3 | guess |
| UI/UX (10-18 05:37-07:30) | 22.7M | 3 | 7.6M | 0.53M | 2.3% | ? 3 | guess |
| Assets & Audio (10-18 05:37-07:57) | 14.0M | 4 | 3.5M | 0.00M | 0.0% | ? 4 | guess |
| UI/UX (10-18 07:30-07:57) | 13.3M | 1 | 13.3M | 0.26M | 2.0% | ? 1 | guess |
| Gameplay (10-18 07:32-07:57) | 8.9M | 2 | 4.5M | 0.17M | 2.0% | ? 2 | guess |
| Platform & Workflow (10-18 07:51-07:57) | 4.0M | 5 | 0.8M | 0.03M | 0.8% | ? 4, docs 1 | exact |
| Assets & Audio (10-18 08:23-07:57) | 0.0M | 1 | - | 0.01M | - | ? 1 | exact |
| Design (10-18 08:23-07:57) | 0.0M | 4 | - | 0.00M | - | ? 4 | exact |
| **Total** | 872M | 118 | | 11.7M | 1.3% | | |
