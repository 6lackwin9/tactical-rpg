# Option branches ledger

Every commit on a design option branch, so the user can pick whole options or mix single commits. Process: `docs/workflow/design-review.md`. Written by the Design area thread only; docs-only commits straight to `develop`.

## Reviews

| Review | Report | Reviewed at (`develop`) | Option set | Status |
| --- | --- | --- | --- | --- |
| DESIGN_REVIEW1 (2026-10-18) | `docs/specs/DESIGN_REVIEW1.md` | before the cycle existed (no option builds) | - | closed (answered 2026-10-18 01:34) |
| next | - | - | - | due when RECRUIT1-L, INTENT1-L, MAPREAD1-L1, ITEMUSE1-L and MAPREAD1-L2 are on `develop` |

## Option sets

None yet. Each set gets a section like this:

```text
### Set <YYYYMMDD> (from DESIGN_REVIEW<n>)

Launch: option A = claude/design-opt-<YYYYMMDD>-a, option B = ...-b, option C = ...-c

| Id | Commit | Branch | What it changes (plain words) | Files | Needs | Conflicts with | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A1 | abc1234 | -a | Enemies act before the party on round 1 | combat_session.gd | - | B2 | waiting |
```

Status values: `waiting` (on the user), `picked`, `landed <develop commit>`, `not picked`.
