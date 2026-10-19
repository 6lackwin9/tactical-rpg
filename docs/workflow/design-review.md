# Design review cycle (Design area thread)

Set up 2026-10-18 at the user's ask (decision "Design review cycle (user 2026-10-18)" in `docs/decisions/workflow.md`). The Design area thread reviews the whole game once a batch of playable changes has landed on `develop`, reports what works and what needs improvement, and builds up to three playable option branches the user can try and mix. Shared rules: `cloud.md`; this file adds only what the cycle needs.

## When a review is due

The ledger `docs/design/OPTION_BRANCHES.md` records the `develop` commit each review covered (`Reviewed at`). A review is due when one of these holds since that commit:

1. At least **6 playable task commits** have landed: commits whose title starts with a task id and that change `engine/` or `data/` (count with `git log --oneline <reviewed-at>..origin/develop -- engine data`, then drop handover, cleanup-only and test-only commits; before the transition, 2026-10-19, the paths were `scripts`, `scenes` and `data`).
2. A roadmap system moved to `done` or `balance-ready` in `docs/ROADMAP.md`.
3. At least **3 days** have passed and at least one playable commit landed.

A review is never due while the previous one still has an option set waiting on the user; the check then repeats the waiting items to the coordinator instead (the pending list).

**First review**: due once RECRUIT1-L, INTENT1-L, MAPREAD1-L1, ITEMUSE1-L and MAPREAD1-L2 (full-screen region map) are on `develop`; CONTROLS-HELP and the other Settings work already landed. Rule 1 starts counting from that review.

## The check routine

A routine bound to the Design session fires at 12:20 and 18:20 UTC, after the 11:52 and 17:52 releases ("Design review check"). On each firing the Design thread fetches `develop`, applies the rules above, and either starts the review or ends the turn silently (`no_reply_needed`). On a session rotation the new Design session recreates the routine and the old one is disabled (the coordinator does the disable, as for the dispatch routine).

## The report

- One Markdown file, `/mnt/project-files/design-review/DESIGN_REVIEW_<YYYY-MM-DD>.md`, linked in the Design thread with a one-line reply; a copy goes to `docs/specs/DESIGN_REVIEW<n>.md` when the user answers it (as DESIGN_REVIEW1).
- Covers all of the game, one section per aspect: combat rules and AI, party and classes, progression (levels, skills, gear), items and economy, world map, travel and quests, story and dialogue, battles' pacing and difficulty (latest `docs/BALANCE_REPORT.md`), UI and readability, audio and art, saves and settings, and playtest notes (`playtests/` and notes the user sent).
- Each section: **Working** (what the changes since the last review got right, with the commit or task id), **Needs improvement** (the problem and the evidence: a sim number, a frame, a playtest note), and **Options** (2-3 ways to fix it, one-line consequence each, one recommended).
- Evidence comes from the repo agents: `design-brainstorm` writes the options, `balance` runs a sim when a claim needs numbers (on `main`, so ask for a release first if needed). Frames for UI claims come from Xvfb renders (`cloud.md` "Environment").
- The report ends with an approval list (one line per item, `approve / skip / pick A, B, C`) and the option builds below with the branch to launch for each.

## Option builds (up to 3)

**Paused (user 2026-10-18 08:29 UTC, project chat):** "limit design agent to propose ideas for now and have me review and approve those choices". Until the user lifts the pause, the Design thread builds no option branches: each review ends with proposals (approval list, 2-3 options each with a recommendation) and the user approves them before any build; approved items become TASKS rows for their area threads. The intent (Design explores ideas on its own branches) is paused, not dropped; the steps below apply again when resumed.

1. Pick the report items that are easier to judge by playing than by reading (feel, pacing, layout). Group them into at most **3 options**, each a coherent direction (for example A: faster battles, B: more tactical battles, C: a middle path). Fewer is fine.
   Building option branches needs the rotating build slot (`cloud.md` "Usage caps"): ask the coordinator for it before step 3 and hand it back when the set is built; the report itself is spec work and needs no slot. Reviews and specs run in standard mode.
2. Each option gets a branch cut from the `develop` commit the review covered: `claude/design-opt-<YYYYMMDD>-a`, `-b`, `-c`. The date keeps sets from different reviews apart.
3. The Design thread builds them with the lane agents (`logic-builder` for code, `asset-builder` for data, as in `cloud.md` "Lanes"): a mock build, not production work. Rules:
   - **One change per commit**, titled `OPT-<letter><n>: <plain description>` (for example `OPT-A2: enemies act before the party on round 1`), so any commit can be cherry-picked alone. A commit that needs an earlier one says so in the ledger (`needs A1`).
   - Data still goes through `aegis content`.
   - The build must run: `tools/fc.sh build` and the tests the change touches (`tools/fc.sh test <filter>`) pass before each push. Failing tests the mock change makes obsolete may be updated on the branch; say so in the ledger row (an option that diverges from a Godot parity test says so in the ledger; the check is retired only when the pick lands, CLAUDE.md "Godot parity"). No `change-reviewer` pass and no full gate (that happens when a pick lands).
   - Never merge an option branch into `develop` or another option branch, never force-push it.
4. After every push, add the commit to `docs/design/OPTION_BRANCHES.md` (commit hash, id, one-line description, files, needs, conflicts with). The ledger lives on `develop`; ledger updates are docs-only commits straight to `develop` (`DESIGN-LEDGER: ...`, `Review: none (ledger)`, `Gate: fc.sh docs passed (docs-only)`).
5. Rows that touch the same file as a row of another option get `conflicts with <id>` so a mix can be planned before cherry-picking.

## How the user plays an option

Through the "Launch game on your PC" thread: the user says "launch option A" (or names the branch) in any thread and the coordinator relays it. The PC session runs `run.bat opt a` (or `b`, `c`, or `run.bat opt <branch>`): `a`/`b`/`c` resolves to the newest remote `claude/design-opt-<date>-<letter>`, and the branch opens in a third folder next to the dev one, `<checkout>-opt` (a detached git worktree, switched to the branch tip on later launches unless it has local changes), with `APPDATA` pointed at `%APPDATA%\tactical-rpg-opt`, so option saves and settings never touch main or dev saves. Manual fallback: `git worktree add --detach <checkout>-opt origin/<branch>`, then run `run.bat` inside that folder (it builds and opens the engine there with that `APPDATA`; an option branch older than the switch to the engine, 2026-10-19, does not launch). The report names the branch per option and a short "what to try" list.

## Approval and landing

- UI options the user approves are preferences, not final decisions, unless the UI choice comes from the user (decision "UI choices are preferences", user 2026-10-18 08:37 UTC). Mark such TASKS rows "UI preference, not final"; his later UI review may change them.
- Report items the user approves that need no option build become TASKS rows (`approved`) for their area thread, built through the normal flow; already-approved work keeps landing on `develop` as usual.
- Option work reaches `develop` only after the user picks it, by option (`pick A`) or by ledger id (`A1, B3, C2`). The Design thread then:
  1. records the pick as a decision row in the matching `docs/decisions/<system>.md`;
  2. cuts `claude/design-pick-<date>` from the latest `develop`, cherry-picks the picked commits in ledger order (`git cherry-pick -x <hash>`), resolving `conflicts with` pairs and noting how in the ledger;
  3. runs the normal landing: `change-reviewer`, full gate, one squash commit titled `DESIGN-PICK-<date>: <ids>` with the usual `Review:`, `Gate:`, `Timing:`, `Blocked-by:`, `AgentMap:` and `Done:` lines;
  4. marks the picked rows `landed <develop commit>` and the rest `not picked` in the ledger, and sets the set's status to `closed`.
- A pick that needs real production work (tests, polish, data at scale) becomes a TASKS row for its area thread instead, with the option commits as its reference.
- Branch cleanup keeps option branches (their commits are not contained in `develop`); closed sets stay in the ledger as history.
