# Balance handover

Updated 2026-10-18 08:30 UTC (rotation, coordinator "Usage caps") by Balance session session_012hWXrrHGj1VxCBHAEN59mo. Read this first, then `docs/workflow/cloud.md` ("Usage caps": build lanes in economy mode, Balance holds the rotating build slot, sims over ~10 min run in the background with one wake). Agents: `balance` for runs, `logic-builder` for tools/sim, `asset-builder` for data, `change-reviewer` before each landing.

## In progress

**BAL5-SIM + BAL5-PH** on branch `claude/bal5-sim` (57354e9, pushed, not landed). All keys are coded: K1 `world=` plus character tuning via `WorldFactory.apply_character`; K2 `sweep size=3 lead=warrior`, 10 seeds default; K3 tank allies hp_end on won fights, priest heal casts / down round (small `sim_metrics.gd` edits); K4 `tier_foe=`; K5 A1 on, S2 behind `s2=on`; B2 generated boss phase `phase=on|off`; K6 PLAN SKIP lines. 7 new checks in bal2_rules and 2 in bal4_campaign, both suites pass alone; parse clean; README delta in the commit. Acceptance: jitter on + `phase=off`, 4-hero default, party L5 at 5 seeds reproduces BAL3-R exactly (100/100/80/100/80). The boss phase fires but the L5 boss cell stays 80.
Next: run the full gate once (`SUITE_TIMEOUT=900 bash tools/fc.sh gate`), run `change-reviewer`, land on develop (commit with Review/Gate/Timing/Blocked-by/AgentMap/Done lines; BAL5-SIM and BAL5-PH TASKS rows -> done), then **message Gameplay** (session_017rPg9X4j3FhvPDo3t93bwo). BREPORT1-L waits for that landing because it moves `sim_metrics.gd`.
Open question to settle in BAL5-R1: should the party/sweep defaults become trios (decision "BAL5 levers" Q1 says party targets are measured with trios)? Default: pass `comp=`/`size=3 lead=warrior` in R1 rather than change defaults.

## Queue after that

1. BAL5-R1 (balance): trio re-measure at 10 seeds (party, trio sweep, role, tier_foe), marks which BAL5 triggers fire (`docs/specs/BAL5_LEVERS.md` sections 1-7).
2. BAL5-AI (logic, `tactical_ai.gd`; Gameplay has no tactical_ai work queued), BAL5-L, BAL5-A only where triggers fire, then BAL5-R.
3. HEAL1-S (sim keys `heal=rally`, `rest=`, `extra=`; after HEAL1-L and BAL5-SIM) and HEAL1-B (campaign run; pass if arms finale 30-55 and gate >= 75). BAL5 F2 (guards at party level) triggers on the HEAL1-B finale number.

## Landed this session (develop)

BAL3-R0, BAL3-S, BAL3-AI, BAL3-A, BAL3-R + CAMP1-B, BAL4-ATTR (`sim campaign`), RECRUIT1-B / OBJ1-B / BOSS1-B + BAL4-ATTR v4 (`enc=world:<id>`, 2fd535c). Latest numbers: `docs/BALANCE_REPORT.md`; older runs: `docs/archive/BALANCE_REPORT_2026-10.md`.

## Gotchas

- Cloud arena suite takes ~5.5 min: always `SUITE_TIMEOUT=900` for the gate.
- Develop moves fast; data merges (world encounters) change sim results. Merge develop before a long run, and rerun when a merge touched `data/worlds` or `tools/sim`.
- Role L5 ~35 min, sweep 6-20 min, party/duel/tier 1-4 min (4 in parallel). Use SIM_TIMEOUT=3300 for long parallel runs.
- `pgrep -f "fc.sh gate"` matches its own wait loop; wait on the background task notification instead.
