# Balance handover

Updated 2026-10-18 15:40 UTC (rotation by context size) by Balance session session_017JH6C5a2uwCQHAbGFH5whk. Read this first, then `docs/workflow/cloud.md` ("Usage caps", "Landing a task": after each landing send Merge & Release `landed <id> <sha>`). Agents: `balance` for runs, `logic-builder` for sim/tools, `asset-builder` for data, `change-reviewer` before each landing. Land with `bash tools/fc.sh gate auto` (per-system gate) plus `tools/fc.sh docs`.

## State

- Build slot: released 15:30 (coordinator told). Ask the coordinator for it when a task below unblocks.
- Nothing in flight. Landed this session: BAL5-SIM/PH, HEAL1-S, BAL5-R1, HEAL1-B, BAL5-AI, CAMPLABEL1-S, BAL5-R1b, BAL5-S2 + POTLEDGER, BAL5-L, BAL5-A, MORALE1-S, BAL5-R (+ probe 7740b0e). Numbers: top of `docs/BALANCE_REPORT.md`.
- BAL5 is closed. Remaining gaps: tank won-fight allies hp 30 (< 40), priest heal share 2, brawler AoE 0.79, boss 26 / line 26 / wave 28 / elite 38 (trio era), HEAL1 arms 63, stand 73.

## Queue (all blocked on others)

1. MORALE1-B (balance, report only; spec `docs/specs/MORALE1.md` 6): waits on MORALE1-A (Assets).
2. PACE1-S (logic, sim pace metrics + PACE1 sim boss phase) then PACE1-B: wait on PACE1-L (Gameplay). PACE1 adds a B3 trigger: boss median under 7 rounds (sim prints no round count yet; PACE1-S adds it).
3. LEVERS1-B (balance; Design's spec, approved 0525027 + "Smarter Shield Wall" e0f1ae5): BAL5-R is "before"; also check warrior-row won-fight allies hp_end >= 40 at L5/L7 after LEVERS1-L3 (AI guard rule) and A3 (shield_wall in the default five).

## Open cards in this thread (Tushar)

- Walk-in taunt (BAL5-AI T2): built on "Advance, then taunt" (and still strikes if one fits). Close it with that option if he answers in words.
- Trio difficulty targets: "Re-set targets" (recommended: propose trio bands from today's numbers, no game change) / "Make fights easier" / "Leave as is". On "Re-set targets": write proposed bands into the report and ask for approval.

## Gotchas

- Balance agent reports can be wrong: BAL5-R first claimed every cell identical; check logs (`tail` summaries, diff against the previous run) before landing a report.
- Check TASKS for an owning thread before building a shared-code row (STATCRED1-L was built twice).
- Write `landed=` in Timing from `date -u` at push time, not a guess.
- The default sweep/campaign loadout is the first 5 class-path skills (`content_factory.gd:362`), not the role loadout.
