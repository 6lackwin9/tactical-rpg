# Milestones

The slice's milestones (decision "Slice group (1.5)", Q-86, Q-88 in `decisions/lore.md`) and which tasks belong to each.
The skill `milestone-status` reads this table to report progress per milestone; the planner keeps it current when it
adds task rows. A task joins the first milestone whose pattern matches its id (an exact id beats a `*` pattern); rows
whose status says "later than the slice" or "M2b" join M2b; tasks that match nothing count as outside the slice.
Since 2026-10-29 (decision "Docs audit pass 2 answers" Q-A, Q-B; CLEANUP.md CLN-28) two milestones follow M2b: "Ashlands"
holds the task rows of the Ashlands goals G1-G6 (`games/emberwake/docs/ROADMAP.md` "Goals", one tracking system), and
"Later" is the backlog of the pre-slice Ashlands rows (approved or ideas, 2026-10-17 to 2026-10-22) that wait until after
the slice. The art pipeline rows of 2026-10-26/27 (local generation, pixelize, founder and mob portraits, the region
view test) joined M2; WAR2H1 (the gear look on battle tokens, a DIR4 input) joined M0. Process rows (`CLN-*`) stay outside.

| Milestone | Goal | Tasks |
| --- | --- | --- |
| M0 | Foundations: the kit split, engine pieces and tools the slice builds on | `KIT1-*`, `DERIVE1-*`, `STAGE1-*`, `DIR4-*`, `PLAYUI1-*`, `FX1-E`, `STORYUI1-E`, `RULESET1-E`, `CAMPAIGN1-E`, `WORLDW1`, `ASSETCHK1`, `WTCLEAN1`, `KIT1`, `WAR2H1` (added 2026-10-29, CLN-28) |
| M1 | Combat feel prototype: the timeline ruleset in an arena on the prologue's alley fight ENC-PRO-0A | `RULESET1-*`, `ENCOUNTER2-*`, `HOOKS1-*`, `FX1-*`, `WINDUP1`, `WINDUP1-*`, `STATUSFX1`, `STATUSFX1-*`, `DIR4-R1F`, `DIR4-R2`, `DIR4-R3`, `DIR4-R4`, `DIR4-HB` (moved from M0, user 2026-10-27: the DIR4-R1 pilot closes M0), `VISCLAR1` |
| M2 | The vertical slice: the Sunk Ward from the onboarding to the slice end, as data on the kit | `CAMPAIGN1-*`, `ROSTER1-*`, `STORYUI1-*`, `SUNKWARD1`, `CAST1-*`, `PORTRAITFX1-*`, `STORYART1`, `CREATURE1`, `ART3`, `ART3B`, `ART3C`, `MENU2`, `LOCALGEN1`; added 2026-10-29 (CLN-28): `LOCALGEN2`, `LOCALGEN3`, `PIXEL2`, `PIXEL3`, `PIXEL4`, `FOUNDERPORT1`, `FOUNDERPORT1-*`, `ALINE1-F`, `MOB1`, `ARTPROMPT1`, `EXPLOREVIEW1`, `HOOKS1-A`, `FX1-A`, `ENCOUNTER2-W`, `DIR4-R4` (moved from M1, user 2026-10-30: they need M2 data or were not asked for yet) |
| M2b | The rest of Act 1 as data tasks after the slice playtest | `CAST1-A6b`, `HOOKS1-Q`, `PUPPET1`, `BIGUNIT1` |
| Ashlands | After the slice: the Ashlands goals G1-G6 (ROADMAP "Goals", user 2026-10-17; Q-A 2026-10-29) | G1 `BATTLE1-L`, `GRID12`, `PLAYTEST1-L`, `DLG1-F`; G2 `S12S-L`, `S12S-A`, `S14-A1`, `S14-A2`, `S14-L1`, `UI3-L`, `AUTO1-L`; G3 `BAL2-R`, `BAL3`; G4 `CAMP1`, `CAMP1-A1`, `CAMP1-A2`, `CAMP1-B`, `CAMP1-P`, `TRAV1-L`, `TRAV1-A`, `REP1-L`, `REP1-A`; G5 `S12T-L`, `S12T-A`, `S12C-L`, `S12C-A`; G6 `CLN-9`, `CLN-17`, `CLN-3b`, `CLN-13` |
| Later | Backlog: the pre-slice Ashlands rows, queued or ideas, picked up after the slice (Q-B 2026-10-29) | `UIR-4`, `UIR-7`, `QUESTTRACK1-U`, `RCLOSE1-U`, `HEAL1-U`, `BREPORT1-U`, `MORALE1-U`, `OBJ1-U`, `PACE1-*`, `LEVERS1-*`, `KEEP1`, `DIFF1`, `DEEDS1`, `RESPEC1`, `OFFHAND1`, `DESERT1`, `ASHPACE1`, `COMBO-A2`, `MAPNAV1-U`, `DRAG1`, `PROPS1-A2`, `SKILLTEXT2`, `SIMKPI1`, `ICONUI1`, `ARTTOOL1`, `PLAYBACK2`, `REWARDXW1`, `LAYOUT1`, `PLAYTEST2`, `MARKERS1` |
