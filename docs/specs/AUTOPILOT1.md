# AUTOPILOT1: autopilot mode (draft for the user's approval)

- **Status**: **draft**, not approved. Written 2026-11-17 from the full project review (step A, user: "all recs A-E"). The mode in force stays the `CLAUDE.md` "Mode in force" line (interactive) until the user approves this in typed words; then Design records a decision row in `docs/decisions/workflow.md` and updates that line.
- **Source**: [2026-11-17 review](../audits/2026-11-17_11d320a_consistency_audit.md) "Path to automated development".

## 1. The loop

1. The user playtests and leaves notes (F8 note box or the project chat).
2. Design turns notes, audit findings and the backlog into READY rows (`docs/workflow/cloud.md` "Playtest intake", READY section 3 "Backlog order"), making reversible calls as `[PROVISIONAL]` and logging them in `DESIGN_REVIEW.md`.
3. Build takes the next row, builds it, and passes review and the gate.
4. `audit-build` checks each landing batch against its READY rows from the recorded `Last audited` sha; scenarios, replays and balance bands run as machine checks (`PR7-4`..`PR7-7`).
5. A green batch is released, and the user gets one daily digest: what changed, what to try in the next playtest, which calls were provisional.

## 2. What becomes automatic

| Class of change | Interactive (today) | Autopilot |
| --- | --- | --- |
| Bug fixes, audit findings, data fixes that match locked canon, doc sync, cleanup with no behaviour change | waits for "Build: next row" | automatic |
| Engine track rows that come with tests | not planned | automatic, in rank order |
| Reversible design calls (numbers, copy, provisional defaults) | provisional, logged | automatic and provisional, listed in the digest; the user flips any by naming it |
| Balance retunes inside the agreed bands (`PR7-6`) | Design proposes | automatic inside the bands; outside them the user decides |
| Placeholder art replaced inside the locked style | each batch asked | automatic after the style lock; one contact sheet per batch |
| `develop` -> `main` | only on "release now" | stage 1: still "release now"; stage 2: automatic when gate, audit and scenarios are green |

## 3. What still goes to the user

New systems or features and anything that changes the game's goal; changes to `[LOCKED]` canon or two locked rulings in conflict; anything that breaks saves without a migration; the art style lock and paid image-generation runs (or a budget set once); public deploys (itch.io); changes to agents, skills or this process; a failure two targeted fixes did not solve (the row is parked and reported, and the loop moves on).

## 4. Guardrails

- A cap of about `3` slices per run and a daily cap, so a bad night is small and usage stays inside the user's limits.
- A stop word ("pause") halts every routine; two failed gates in a row pause the loop automatically.
- One Build at a time through `origin/build-lease`; a matching Design lease is added before stage 1.
- Every landing names its READY row id; the audit refuses a landing that does more than its row.
- A weekly `consistency-audit` run; one digest per day in the project chat, not a message per step.

## 5. Stages and preconditions

1. **Preconditions**: review step A landed (instructions consistent, mode line, intake, backlog order); `PR7-4`..`PR7-7` landed.
2. **Stage 1, supervised (one week)**: routines bound to the Design and Build threads chain Design -> Build -> audit, capped at about `3` slices a night; releases stay on "release now"; the user reads the digest and playtests.
3. **Stage 2, full autopilot**: releases become automatic on green; the user's inputs are playtest notes and section 3.

## 6. Question for the user

Approve stage 1 as written (yes / change / not yet)? Stage 2 needs a second typed approval after the supervised week.
