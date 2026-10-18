# Design handover

Written 2026-10-18 08:25 UTC by the Design session (rotation by context size, `cloud.md` "Usage caps"). Read this, then `docs/workflow/cloud.md` and `docs/workflow/design-review.md`.

## Queue

- Nothing waits on the user from Design. All cards answered (BREPORT1, BAL5, Orsa quest, HEAL1).
- Next spec: MORALE1 (enemy morale, builds on OBJ1 slay; TASKS row, `docs/specs/DESIGN_REVIEW1.md` 4.3) once OBJ1-L is on `develop`. Then the remaining DR1 design rows in `docs/TASKS.md` (for example COMBO-A2).
- First design review: due when RECRUIT1-L, INTENT1-L, MAPREAD1-L1, ITEMUSE1-L and MAPREAD1-L2 are on `develop` (`docs/design/OPTION_BRANCHES.md`). Candidate option-build ideas are listed there. Option builds need the rotating build slot from the coordinator.

## Handed off today

- BREPORT1-L to Gameplay (BREPORT1-U to UI/UX after UIR-4).
- BAL5-SIM/R1/AI/PH/L/A/R to Balance (trio re-measure first).
- HEAL1-L to Gameplay (pings Assets for HEAL1-A, UI/UX for HEAL1-U, Balance for HEAL1-S/B). Not parallel with BREPORT1-L. User changed the defeat rule: 3 supplies forced rest at the last checkpoint, else back to the last rest place (decision progression-save "HEAL1 between-battle healing").
- OPTLAUNCH1 landed (Platform): `run.bat opt a|b|c`.

## Routines

- "Design review check" trig_01C87VVsqRTJWf99ggm9vqam (12:20 and 18:20 UTC) is bound to the old Design session. The new session recreates it bound to itself (same prompt as in `design-review.md` "The check routine") and asks the coordinator to disable the old one.

## Session ids for hand-offs (2026-10-18 08:20)

Gameplay cse_017rPg9X4j3FhvPDo3t93bwo, UI/UX cse_01Rs6SLjohzJBiLs4ezyKKBN, Assets & Audio cse_01UywiZpBegbdrmR76smao3d, Balance cse_012hWXrrHGj1VxCBHAEN59mo (may have rotated), Platform cse_01BW1xnuo5QGUqFTpZpTWKyj.
