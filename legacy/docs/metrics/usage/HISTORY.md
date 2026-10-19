# Project usage history

One row per usage report Tushar pastes (cumulative since project start; totals = sum of listed threads, only the current coordinator session is listed). Deltas and heavy-thread notes: `DELTA_<date>.md`.

| Report (UTC) | Threads | Tokens | Output | Cache write | Cache read | Cache hit | Delta tokens | Heaviest since previous |
|---|---|---|---|---|---|---|---|---|
| 2026-10-17 16:51 (screenshot) | 29 | 594M | 1.9M | 9.2M | 582.8M | 98% | - | - |
| 2026-10-17 20:19 | 31 | 709.6M | 2.7M | 14.4M | 692.4M | 98% | +116M | Platform 99.2M, UI1-L 69.3M |
| 2026-10-18 07:57 | 53 | 1,367M | 6.3M | 43.7M | ~1.3B | 97% | +657M | Balance +84.5M, UI/UX first session +70.7M, new Gameplay sessions ~180M |
| 2026-10-18 11:45 | 57 | 1,719M | 7.7M | 52.4M | ~1.7B | 97% | +352M | Coordinator +72.9M, Gameplay +64.5M (two sessions), Assets & Audio +53.4M (sprite art rounds), UI/UX +44.5M |
| 2026-10-18 15:59 | 63 | 1,952M | 8.8M | 61.7M | ~1.9B | 97% | +325M (old coordinator row 93.1M dropped from the list) | Merge & Release +45.9M, Gameplay +68.9M (rotation 14:37), UI/UX +77.5M (rotation 14:41), Balance +41.2M (rotation 15:37), Platform +37.4M (rotation 12:58), Design +25.7M, Assets & Audio +16.9M (Sonnet) |

2026-10-18 16:29 UTC: the standard vs economy trial ended early (project moved to interactive mode, decision "Interactive mode"); only the standard baseline exists (MODE_TRIAL.md). The Sonnet trial continues only if Assets & Audio gets requests.
