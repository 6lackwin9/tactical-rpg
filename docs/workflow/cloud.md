# Cloud workflow (Claude project threads)

Use this when Claude runs in a cloud container (`$CLAUDE_CODE_REMOTE` is `true`). Shared rules: `CLAUDE.md`, skills `planner` and `planner-ops`. Differences from the desktop and the switching checklist: [README.md](README.md). Decided by the user 2026-10-17 (decisions "Cloud workflow" and "Environment split" in `docs/decisions/workflow.md`).

## Session threads (since 2026-11-17)

The project runs the two-session split (decision "Two-session split: Design and Build", 2026-11-07) plus the Artwork & Mock UI Session as standing project threads (user 2026-11-17, project decision card "Reorganise project threads to match the Design / Build / Art-UI split"). They replace the October area threads (Gameplay, UI/UX, Balance, Assets & Audio, Platform & Workflow, Merge & Release), which are retired.

| Thread | Starts with | Owns (sole writer) | Agents it runs |
| --- | --- | --- | --- |
| Design | skill `design-session` | `docs/READY.md`, specs (`docs/specs/*`, `docs/sessions/*`, `MASTERPLAN1.md`), decisions, `TASKS.md`, ROADMAP, MILESTONES, `HISTORY.md`, `PRINCIPLES.md`, `DESIGN_REVIEW.md`, audits, shared READMEs, `CLAUDE.md` and these workflow docs; agent and skill changes (with the user's approval) | `design-brainstorm`, `build-auditor` (skills `audit-build`, `consistency-audit`), `housekeeper` |
| Build | skill `build-session` | code, data, tests and sims on `claude/<task>` branches squashed onto `develop`; `docs/FEEDBACK.md` and `docs/archive/FEEDBACK_CLOSED.md`; crate-local READMEs; release PRs and `PATCH_NOTES.md` | `logic-builder`, `asset-builder`, `balance`, `change-reviewer` |
| Art/UI | skill `art-ui-session` | `tools/mock_ui/**`, `art_src/vn_stage/**`, `ART_UI_STUDIO.md`, `SCREEN_ATLAS.md`, `ART_UI_REQUESTS.md` (Inbox read, Outbox written), `tools/gen_music.py` and its audio, `assets/**/README.md` | `art-ui-studio` |
| PC launch | skill `pc-launch` | nothing in the repo | none |

- **Project chat (coordinator)**: the front desk. The user asks there; it routes each ask to one of the threads above and relays follow-ups. It does not edit the repo, takes no turn on routine thread pings, and is driven mainly through skills (decision "Coordinator and assets cutbacks").
- **Routing**: a new idea, a rule or numbers question, a playtest note that is not covered by a READY row, agent/skill/workflow changes -> Design. "Build the next row", a bug in a landed READY row, a balance run, a release, patch notes -> Build. Screens, poses, VN staging, music -> Art/UI. "Launch" -> PC launch.
- **Handover between threads is the repo, not chat**: READY (Design -> Build), FEEDBACK (Build -> Design), `ART_UI_REQUESTS.md` (Art/UI <-> Design). In interactive mode no thread messages another to start work: the user (or the coordinator on the user's ask) says "Build: next row" after Design has queued it.
- **One Build at a time**: the `origin/build-lease` branch names the live Build Session (`cloud` or `desktop`); `python3 tools/planner/feedback_check.py lease check` runs before every resume and push, and a session whose lease moved halts. One Design writer at a time too: while the Design thread is open, do not run a desktop Design Session on the same files (no lease exists yet).
- **New threads need the user's approval.** The coordinator opens a session thread when its first ask arrives, and asks before opening any other thread.
- Decision requests go to the user in the thread (a decision card when offered): question, options with one-line consequences, one recommendation. Work continues on the recommended option unless the step cannot be undone. Repo-recorded decisions cite a typed user message.
- Failure alerts: a thread reply that leads with what is blocked and what the thread is doing about it.
- Dispatch mode: the user types `mode <name>` in the project chat. In interactive mode (planner-ops "Interactive rules") threads act only on the user's direct requests, one at a time, and scheduled routines stay paused.

## One task at a time, one branch per task (Build)

1. The Build thread takes `docs/READY.md` section 1 rows in order (or the rows the user names). Each task gets its own branch (`claude/<task-id>-<slug>`, or the branch the thread was given), cut from the latest `develop`, and lands on `develop` as one squash commit. Feature branches are free. No sibling worktrees.
2. Subagents work in the thread's checkout and never commit. The thread commits after each green `tools/fc.sh` step and pushes, so a recycled container or a usage limit loses nothing.
3. The thread never edits Design-owned docs (table above); task status, `FB-nn` / `UN-nn` questions, sim results and docs deltas go to `docs/FEEDBACK.md`, in the task commit.
4. `engine` (the branch the Aegis engine was built on) was fast-forwarded into `main` on 2026-10-19 (decision "Engine merged into main") and is gone from origin; `legacy` (the frozen Godot build) is never touched unless the user says so.

## Lanes inside a thread

The thread is the planner (lane table: skill `planner`). It writes its own owned docs itself and hands every other lane to that lane's agent:

| Work | Agent | Thread |
| --- | --- | --- |
| engine, kit or game code, schema, validation, tools, tests, and fixes to them (review fixes included) | `logic-builder` | Build |
| catalog or world data and its seed | `asset-builder` | Build |
| simulator runs and `games/emberwake/docs/BALANCE_REPORT.md` | `balance` | Build |
| review of the whole branch before it lands | `change-reviewer` | Build |
| a new or changed spec, design options | `design-brainstorm` | Design |
| audit of landed Build commits, or a whole-repo consistency audit | `build-auditor` | Design |
| a cleanup pass (after a milestone, or on request) | `housekeeper` | Design |
| mock UI, animation lab, VN stage, music | `art-ui-studio` | Art/UI |

1. Only these eight agents (`.claude/agents/`) are dispatched for lane work; generic agent types are not. Effort and model come from the dispatch-mode table (skill `planner-ops`).
2. Work none of them covers goes to the user as a proposal (Design thread): extend an agent or add one. Any new or changed agent or skill needs the user's approval before it merges.
3. A thread that does a lane's work itself has broken the lane rule; "no subagent" is valid only for a docs-only task in the thread's own docs. The `AgentMap:` lines name each agent the thread ran.

## Landing a task on `develop` (Build thread, no PR)

`develop` is the long-lived integration branch (created once from `main`; if `git ls-remote origin develop` is empty, the Build thread recreates it from `main`); `main` only moves through the batched PR below, so CI runs once per batch instead of once per task (decision "Batched merges").

1. Review: run the `change-reviewer` agent on the whole task branch (`git diff origin/develop...HEAD` plus any uncommitted change). Fix its blockers (re-run it if the fix is more than a few lines). A docs-only diff still gets the docs profile of the review.
2. Gate: `tools/fc.sh gate` (builds every engine crate and runs every test; `auto` is accepted and runs everything, since the per-system arena split of the Godot build is gone) and `tools/fc.sh docs`. There is no IDE Problems rule in the cloud. A docs-only diff (Markdown outside `.claude/`, no code, data, skills, agents or workflows) needs only `tools/fc.sh docs`; the release PR runs the full gate. Gate once (decision "Gate once per task"): while fixing review findings or test failures, run only what the change touches (`tools/fc.sh build`, then `tools/fc.sh test <filter>`); run the full gate once, when the task is ready to land. In the cloud this replaces the planner skill's re-gate after each post-gate change.
3. Approved means: the task is a `ready-to-build` row in `docs/READY.md` section 1 (or an `AUDIT-nn` fix row there), or the user asked for it in the thread in typed words. Only approved work lands on `develop`.
4. Land it: `git fetch origin develop`, merge `origin/develop` into the task branch (resolve conflicts there), re-run the full gate only if `develop` brought changes to files the task touches or to shared code they use (otherwise the release CI covers it), then `git checkout develop && git merge --squash <task-branch> && git commit` and `git push origin develop`. If the push is rejected because `develop` moved, fetch and repeat from the merge step. Landing while a release PR is open is allowed: the new commit joins that batch and CI re-runs on it.
   Split landing (speed review 2026-10-17): when other tasks depend on a large logic task (a data `-A` task on its schema, a later logic task on its rules), land it in two squash commits, each reviewed and gated with its own `Review:` and `Gate:` lines: first the schema, validation and their tests (`<ID> (schema): ...`), then the behaviour (`<ID>: ...`). The schema commit counts as done for any `Depends on` that names only the schema, so the dependent task (and the daily dispatch) can start from `develop` as soon as it lands. Each commit carries its own `Timing:`, `Blocked-by:` and `AgentMap:` lines (`TASK_LOG.md` gets one row per commit, the schema row marked `(schema)`); only the second carries the `Done:` line and deletes the TASKS row.
5. The squash commit title starts with the task id; its body says what changed and ends with the `Review:` line, for example `Review: change-reviewer clean` or `Review: change-reviewer 1 high (fixed), 2 nits accepted`, and `Gate: fc.sh gate passed` (docs-only: `Gate: fc.sh docs passed (docs-only)`). No task lands without both lines. It also carries a `Timing:` and a `Blocked-by:` line ("Task timing" below), one `AgentMap: | <agent> | <task> | <tokens> / <min> | <outcome> |` line per agent run (the `docs/AGENT_MAP.md` row), and one `Done: | <id> | <title> | <lane> | <depends on> | done | <spec> | <touches> |` line (the `docs/archive/TASKS_DONE.md` row).
6. Not approved: a small change waits on its pushed task branch and goes to the user as a question in the thread; a large one waits for the user's feedback before work starts. Rows in `spec` or `idea` status and any design question that comes up mid-task go to the user.
7. Patch notes: the Build thread writes `PATCH_NOTES.md` once per release and when the user asks (decision "Patch notes per release").
8. Never push to `main`. After pushing the squash commit to `origin/develop`, fast-forward the task branch to a merge commit recording `origin/develop` (`git push origin $(git commit-tree "origin/develop^{tree}" -p origin/<task-branch> -p origin/develop -m "Close <task-branch> (squash-landed on develop)"):refs/heads/<task-branch>`) so `git branch -r --merged origin/develop` and the `branch-cleanup` workflow see it as merged and delete it automatically after the 6-hour age window ("Branch cleanup" below).

## Release to `main` (Build thread)

1. Batch: when the user says "release now" (skill `release-now` in the project chat), if `develop` is ahead of `main`. The twice-daily routines (decision "Twice-daily releases") are retired with the Merge & Release thread; the Build thread releases between tasks, never mid-landing.
2. Check each task commit in `git log origin/main..origin/develop`: task id in the title, `Review:` and `Gate:` lines in the body, task row `approved` or a decision row. A commit that fails the check is reverted on `develop` (`git revert`, pushed) and fixed in this thread. A reverted task lands again as a new squash commit of its fixed branch onto the latest `develop` (merging the old branch again would be a no-op for git). Then write the logs in one commit on `develop`, pushed before step 3 so they ship in the batch: the `AgentMap:` lines into `docs/AGENT_MAP.md` (one section per task, newest first), the `Done:` lines into `docs/archive/TASKS_DONE.md`, and the `Timing:` and `Blocked-by:` lines into `docs/metrics/TASK_LOG.md` (one row per task; release wait = landed to this commit) with a refreshed Bottlenecks section (biggest waits, the dependency chains behind them, how to decouple them). The Build thread is the only writer of those three files and of `PATCH_NOTES.md`, so they never conflict. At each batch, and when the user asks, it runs the `patch-notes-write` skill: `python3 tools/patch_notes.py` lists the commits since the marker, it writes plain-language "In the game" lines and the commit list for the day, moves the marker and pushes (`PATCH-NOTES: ...`, docs-only).
3. Pin the batch (Merge & Release 2026-10-18, release #38; decision "CI on release PRs and main only", 2026-11-11): `gate.yml` runs only on PRs into `main`, pushes to `main`, and `workflow_dispatch` (never on `develop` pushes, since every landing runs `tools/fc.sh gate` locally first). Push the current `origin/develop` sha to the release branch `claude/release-batch-vmj4rj` (or open `develop` -> `main` directly when no further landings are in flight) so later `develop` landings do not restart an open release PR's CI; they go out in the next batch. Open one PR -> `main` titled `Release <date>: <task ids>`; body: one line per task with its review verdict, plus anything waiting on the user. CI (`gate.yml`) runs on this PR; `gate` is a required check on `main` (branch protection: PR required, no force push or deletion), so it always runs on the release PR, docs-only batches included.
4. CI green: merge with a **merge commit** (not squash, so `develop` and `main` keep a shared history and the next batch has no conflicts). The push to `main` runs the gate again; web deploy follows `docs/specs/WEBREL1.md` ([web.md](web.md)).
5. CI red: find the task commit that broke it (the failing suite names its area), revert it on `develop`, push, and move the release branch to the new `develop` sha (CI re-runs on the PR), then fix it on a task branch and land it again.
6. Hotfix to `main` without `develop` only with the user's word; after any change that reaches `main` outside a release PR, merge `main` back into `develop` right away so the next batch does not conflict.
7. The `balance` agent keeps running on `origin/main`, so it measures released code only; ask for a release first when a balance run needs work still on `develop`.

## Context hygiene (token use)

Cost grows with the thread's context size times its turns (user dashboard 2026-10-17: ~98% of tokens were cache reads), so keep the thread's own context small:

- Run the gate, sims and suites with output redirected to a file and read only the summary or failures (`tail`, `grep FAIL`), never the full log.
- Hand lane work to the repo agents so file reads and build output stay in their context; the thread keeps only their reports.
- Batch related commands into one tool call, and read files by section instead of whole.
- Docs-only tasks skip the local gate ("Landing a task on `develop`" step 2).

### Fresh sessions (user 2026-10-17, "Restart often")

Rotate by context size, not task count (user 2026-10-18, usage cutbacks): when the thread's context passes about 120k tokens (`get_session` with no id, `external_metadata.context_usage.used_tokens`; else a compaction notice), a thread stops at the next task boundary (nothing uncommitted, no builder running) and hands over. A fresh session costs 4M to 13M tokens to start, so a thread with a small context keeps going whatever its task count:

1. The repo is the handover: Build records queue position, in-flight branch and open questions in `docs/FEEDBACK.md` section 1; Design keeps `docs/READY.md` and `docs/DESIGN_REVIEW.md` current; Art/UI keeps `ART_UI_STUDIO.md` and the `ART_UI_REQUESTS.md` Outbox current. No `docs/handover/` file is needed (the old `docs/handover/build.md` is superseded by FEEDBACK).
2. Commit and push that update (docs-only, `Gate: fc.sh docs passed (docs-only)`).
3. Tell the coordinator the thread is ready to rotate; it starts a fresh session for that thread under a new project post and resolves the old one. The new session starts from its session skill.

### Usage caps (user 2026-10-18, usage cutbacks 1-4)

The user hits the 5-hour and weekly usage limits, so usage is spread out:

1. Only the Build thread builds (one Build Session at a time, lease above); Design and Art/UI do not count against usage slots.
2. Dispatch mode is split: economy for build lanes, standard for specs, design reviews and the final gate (planner-ops "Current mode").
3. Long sims (more than about 10 minutes) run in the background (`balance` agent or a background command) and the thread ends its turn; the completion wakes it once. No polling with sleeps or monitors: every wake re-reads the whole context, and a wake after the prompt cache expired re-writes it.
4. Rotation goes by context size (above).
5. Asset work (user 2026-10-18): the user reviews options in one heavy session with the Assets & Audio thread; once the base is signed off, the rest becomes a task that runs in the background. Batch feedback into fewer, bigger messages where possible.

## Task timing

Goal (user 2026-10-17): see where time goes, mainly idle time and dependence between steps; tokens are secondary. The Build thread notes timestamps as it works (`date -u +%H:%M` at each step) and puts two lines in the task's squash commit body:

- `Timing: start=<HH:MM>Z landed=<HH:MM>Z work=<min> (build <min>, review <min>, rework <min>, conflicts <min>) wait=<min> (approval <min>, dependency <min>, ci <min>, merge <min>) gates=<count> mode=<dispatch mode> agents=<agents run> type=<logic|ui|asset|sim|spec|docs|tool> tokens=<agent>:<k>,...` (one entry per agent kind, summed over its runs, from each Agent result's `subagent_tokens`, for example `tokens=logic:120k,asset:40k,review:20k`; user 2026-10-18: isolates which task types and agents drive token use)
- `Blocked-by: <task id, "user approval HH:MM-HH:MM", "merge of <id>", or none>`

Work is time the thread is building, reviewing, gating or fixing; wait is time the task is ready or blocked and nobody works on it, by cause. Estimates are marked `~`. The coordinator's `time-review` and `usage-report` skills compare these lines with the user's project usage reports (thread totals minus agent tokens = the thread's own residual) in `docs/metrics/usage/` (README there). The Build thread adds the release wait and compiles the lines into `docs/metrics/TASK_LOG.md` at each batch.

## Branch cleanup

Cloud sessions cannot delete remote branches (the proxy refuses ref deletes). `.github/workflows/branch-cleanup.yml` runs daily at 19:37 UTC and on request (**Actions > branch-cleanup > Run workflow**, optional dry run): it deletes every branch except `main`, `develop` and `engine` that has no open PR (this keeps the release branch while its PR is open), has had no commit for 6 hours, and is either the head of a merged PR or already contained in `develop` (squash-landed branches closed with `git commit-tree "origin/develop^{tree}" -p origin/<branch> -p origin/develop` per step 8 above are detected as ancestors of `develop`; any older unclosed squash branch whose files later changed on `develop` can be closed the same way or deleted by name with `also_delete`); branches with work not on `develop` are kept and listed in the run summary. Do not turn on the repository setting "Automatically delete head branches": the release PR's head is `develop`, so it would be deleted at every release.

## Scheduled routines

None run (user 2026-10-18 16:28 UTC, interactive mode). The October routines (daily planner pass via Platform & Workflow, Design review check, midday and daily release batches via Merge & Release) were bound to the retired area threads and stay disabled; they are not re-enabled by `mode standard`. A routine for the session threads (for example a daily "Build: next READY row" pass) is set up only when the user asks, bound to the current Build or Design thread, and follows the same approval rule as threads.

## Environment

- Rust stable and PowerShell 7 come from the environment setup script (copy: `/mnt/project-files/cloud-setup/environment-setup-script.sh` in the project files; it may still install only PowerShell (and Godot, no longer used): then the repo hook installs Rust with rustup); `.claude/hooks/session-start.sh` installs what is missing (including the ALSA headers `libasound2-dev` and `pkg-config` when `pkg-config --exists alsa` fails; if the hook did not run in a container, `apt-get install -y libasound2-dev` fixes `alsa-sys`), sets `RUSTUP_TOOLCHAIN=stable` and `CARGO_TARGET_DIR` (when running a parallel worktree lane inside a cloud container, pass `CARGO_TARGET_DIR=<worktree>/target` inline per command since the sandbox refuses `export`). `github.com`, `sh.rustup.rs` and `static.rust-lang.org` must stay allowed in the environment's network policy.
- Web build: none on the engine yet; the Godot web export (`web.yml`, paused since 2026-10-17) was removed with the Godot build (it stays on the `legacy` branch, [web.md](web.md)).
- No interactive play. For UI work, run `emberwake` under Xvfb and use its agent link's `screenshot` (`xvfb-run -a -s "-screen 0 1280x720x24" "$CARGO_TARGET_DIR/release/emberwake" screen=runner`, then `aegis link '{"cmd":"screenshot","path":"out.png"}'`; not yet tried in a cloud container) and attach the frames to the thread reply. Playtesting feel, input and audio stays on desktop.
