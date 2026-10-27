# Milestones

The slice's milestones (decision "Slice group (1.5)", Q-86, Q-88 in `decisions/lore.md`) and which tasks belong to each.
The skill `milestone-status` reads this table to report progress per milestone; the planner keeps it current when it
adds task rows. A task joins the first milestone whose pattern matches its id (an exact id beats a `*` pattern); rows
whose status says "later than the slice" or "M2b" join M2b; tasks that match nothing count as outside the slice.

| Milestone | Goal | Tasks |
| --- | --- | --- |
| M0 | Foundations: the kit split, engine pieces and tools the slice builds on | `KIT1-*`, `DERIVE1-*`, `STAGE1-*`, `DIR4-*`, `PLAYUI1-*`, `FX1-E`, `STORYUI1-E`, `RULESET1-E`, `CAMPAIGN1-E`, `WORLDW1`, `ASSETCHK1`, `WTCLEAN1`, `KIT1` |
| M1 | Combat feel prototype: the timeline ruleset in an arena on the prologue's alley fight ENC-PRO-0A | `RULESET1-*`, `ENCOUNTER2-*`, `HOOKS1-*`, `FX1-*`, `WINDUP1`, `STATUSFX1`, `DIR4-R1F`, `DIR4-R2`, `DIR4-R3`, `DIR4-R4`, `DIR4-HB` (moved from M0, user 2026-10-27: the DIR4-R1 pilot closes M0) |
| M2 | The vertical slice: the Sunk Ward from the onboarding to the slice end, as data on the kit | `CAMPAIGN1-*`, `ROSTER1-*`, `STORYUI1-*`, `SUNKWARD1`, `CAST1-*`, `PORTRAITFX1-*`, `STORYART1`, `CREATURE1`, `ART3`, `ART3B`, `ART3C`, `MENU2`, `LOCALGEN1` |
| M2b | The rest of Act 1 as data tasks after the slice playtest | `CAST1-A6b`, `HOOKS1-Q`, `PUPPET1`, `BIGUNIT1` |
