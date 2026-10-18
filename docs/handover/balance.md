# Balance handover

Updated 2026-10-18 16:35 UTC (project wind-down to interactive mode) by Balance session session_01Lc7tUeurRvdktR4Qtn5PQ9. Read this first, then `docs/workflow/cloud.md` ("Usage caps", "Landing a task") and `planner-ops` (interactive mode: work only on Tushar's request, one thing at a time). Agents: `balance` for runs, `logic-builder` for sim/tools, `asset-builder` for data, `change-reviewer` before each landing. Land with `bash tools/fc.sh gate auto` (per-system gate) plus `tools/fc.sh docs`.

## State

- Build slot: released 16:35 (coordinator told). Nothing in flight.
- Landed this session: MORALE1-B (7bda7b5). Numbers: top of `docs/BALANCE_REPORT.md`.
- MORALE1-B headline: solo warrior vs ash_scouts 20 -> 100 clear (mob band met); ash_raiders 100 clear for every duo and trio, 0.7-1.9 rounds faster; trios 1.1-1.7 rounds. DR1 levers (`morale_leader` "", `morale_break` 40, `FLEE_WAVERING`) not triggered.
- ASHPACE1 (Assets) said "first change ash_scouts one scout fewer unless MORALE1-B already lifts it to 90+": it did (100), so that change is not needed. Tell Assets/coordinator if they have not seen it.
- Older gaps (BAL5): tank won-fight allies hp 30 (< 40), priest heal share 2, brawler AoE 0.79, boss 26 / line 26 / wave 28 / elite 38, HEAL1 arms 63, stand 73.

## Queue (start only when Tushar asks)

1. PACE1-S (logic, sim pace metrics + PACE1 sim boss phase): unblocked (PACE1-L landed, develop 6e72b3b; Gameplay's hooks are in its handover). Not started. Then PACE1-B. PACE1 adds a B3 trigger: boss median under 7 rounds.
2. MORALE1-B follow-ups (proposals, need Tushar's approval as tasks): sim morale counters (`fled_shaken`, `fled_wavering`, `surrendered`, `break_by_crit_only`, `break_round`); campaign rerun with a higher SIM_TIMEOUT (both arms timed out at 900 s with 7 of 10 trios) or `comp=` per trio; question whether trios at 1.1-1.7 rounds vs ash_raiders are fine or want `morale_break` 40.
3. LEVERS1-B: waits on LEVERS1-A1/A2/A3. BAL5-R is "before"; also check warrior-row won-fight allies hp_end >= 40 at L5/L7 after LEVERS1-L3 and A3.

## Open questions for Tushar (asked in the Balance thread, no answer yet)

- Walk-in taunt (BAL5-AI T2): built on "Advance, then taunt" (still strikes if one fits). Close it with that option if he answers in words.
- Trio difficulty targets: "Re-set targets" (recommended: propose trio bands from today's numbers, no game change) / "Make fights easier" / "Leave as is". On "Re-set targets": write proposed bands into the report and ask for approval.

## Gotchas

- Balance agent reports can be wrong: check logs (summary lines, diff against the previous run) before landing a report. MORALE1-B's agent also left `until grep ...; sleep` loops waiting for a "PLAN FAIL" line that a timed-out campaign never prints: kill them, and give campaigns `timeout` above 900 s.
- Long campaigns: `campaign 5 10` per heal arm exceeds the 900 s sim timeout on a cloud container; split by `comp=`.
- Check TASKS for an owning thread before building a shared-code row (STATCRED1-L was built twice).
- Write `landed=` in Timing from `date -u` at push time, not a guess.
- The default sweep/campaign loadout is the first 5 class-path skills (`content_factory.gd:362`), not the role loadout.
- No more `landed <id> <sha>` messages to Merge & Release (Tushar 16:10, f1db72d); patch notes are written per release.
