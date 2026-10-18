# Standard vs economy baseline (user 2026-10-18 12:42 UTC)

Tushar: "keep things in standard for now so we have a baseline standard consumption as per task log, then switch to economy for some hours to get that baseline". Platform compares tokens per landed task between the two windows from the usage report deltas and the `Timing:` lines (`type=`, `tokens=`), and reports it with the usage delta after the economy window. The Sonnet trial ([TRIAL_SONNET.md](TRIAL_SONNET.md)) runs through both windows.

| Window (UTC) | Mode (planner-ops "Current mode") | Report delta | Tasks landed | Tokens per landed task | Note |
|---|---|---|---|---|---|
| 2026-10-18 12:42-15:45 | standard (every lane) | | | | the 11:45 report is the start baseline; 12:42 split falls inside its first delta |
| 2026-10-18 15:45-18:45 | economy (every lane) | | | | Platform switches the line at 15:45 |

Usage reports needed: one near 15:45 and one near 18:45 (the coordinator's 3-hour reminder lands close to both).
