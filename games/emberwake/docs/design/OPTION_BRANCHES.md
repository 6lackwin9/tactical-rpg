# Option branches ledger

Every commit on a design option branch, so the user can pick whole options or mix single commits. Process: `docs/workflow/design-review.md`. Written by the Design area thread only; docs-only commits straight to `develop`.

## Reviews

| Review | Report | Reviewed at (`develop`) | Option set | Status |
| --- | --- | --- | --- | --- |
| DESIGN_REVIEW1 (2026-10-18) | `games/emberwake/docs/specs/DESIGN_REVIEW1.md` | before the cycle existed (no option builds) | - | closed (answered 2026-10-18 01:34) |
| DESIGN_REVIEW2 (2026-10-18) | `games/emberwake/docs/specs/DESIGN_REVIEW2.md` | a499b3f | none (option builds paused 08:29) | closed (answered 2026-10-18 08:37, all recs) |
| DESIGN_REVIEW3 (2026-10-18) | `games/emberwake/docs/specs/DESIGN_REVIEW3.md` | 8e2530a | none (option builds paused) | closed (answered 2026-10-18 12:27, all recs) |
| next | - | - | - | rule 1 counts from 8e2530a |

## Candidates for the next review

Ideas better judged by playing than by sims, to weigh for the next option set:

- Boss acts twice per round (BAL5 boss gap; `games/emberwake/docs/specs/BAL5_LEVERS.md`)
- Warrior bodyguard: takes hits meant for an adjacent ally (BAL5 tank gap)
- Priest ward / damage absorb skill (BAL5 healer gap)
- Mage self-centred cone (BAL5 AoE gap)

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
