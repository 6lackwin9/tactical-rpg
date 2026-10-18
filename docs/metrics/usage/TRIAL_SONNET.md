# Sonnet vs Opus trial (Assets & Audio)

User 2026-10-18 12:15 UTC: the Assets & Audio thread (session `cse_01V6GmFsz2ESnDgqza17YhQt`) is back on Sonnet; Platform compares tokens per landed task against the Opus Assets sessions. The report's model label is not used (it showed the session as Opus after a switch); the window comes from this log and the report deltas.

- Window start: 2026-10-18 12:15 UTC (Sonnet). Baseline: the session's row was 48.6M at the 11:45 report; the 11:45-12:15 slice ran on Opus, so the first delta overstates Sonnet slightly (noted on that row).
- First Sonnet window (08:23-10:14) is void: it mixed Sonnet and Opus and was mostly art review rounds.
- Opus baseline (efficiency table, landed asset tasks): 3.6M, 4.2M, 4.7M, 3.8M per task for data and placeholder tasks; 8.7M for the art pass; 12.3M for the music track.
- The thread adds `model=<sonnet|opus>` to each `Timing:` line while the trial runs, so tasks map to a model.

## Tasks in the window

| Landed (UTC) | Task | Kind | Model | Commit |
|---|---|---|---|---|
| 2026-10-18 12:17 | ITEMPOW1-A | data (seed) | mixed: started 11:45 on Opus, landed on Sonnet; excluded from the comparison | 02c6bf4 |

## Report deltas

| Report (UTC) | Session row | Delta since previous | Tasks landed | Tokens per task | Tokens per hour | Note |
|---|---|---|---|---|---|---|
| 2026-10-18 11:45 | 48.6M | - | - | - | - | baseline, Opus since 10:14 |
