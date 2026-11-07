---
name: design-session
description: Bootstrap and operate the Design Session in the two-session split (Design in C:\dev\tactical-rpg-dev vs Build in C:\dev\tactical-rpg-plan). Use when starting or resuming a Design Session (/design-session, "start design session") to sync origin/develop, review docs/READY.md and docs/FEEDBACK.md, apply docs deltas, answer FB-nn / UN-nn items, and maintain READY.md and specs.
---

# Design Session Start & Workflow (`C:\dev\tactical-rpg-dev`)

Operate as the **Design Session** (`C:\dev\tactical-rpg-dev`, branch `develop`) under the two-session split (`CLAUDE.md`, `docs/WORKFLOW.md`).

## 1. Ownership & Boundaries

- **Sole writer of**: `docs/READY.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/specs/*`, `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `games/emberwake/docs/MILESTONES.md`, `docs/DECISIONS.md` (`docs/decisions/*`, `games/emberwake/docs/decisions/*`), `docs/HISTORY.md`, `CLAUDE.md`, `.claude/skills/**`, `.agents/skills/**`, and shared READMEs (`README.md`, `engine/README.md`, `kits/rpg/README.md`, `docs/STRUCTURE.md`, `docs/WORKFLOW.md`, `games/emberwake/docs/GAME_BRIEF.md`, `engine/docs/PORT_NOTES.md`, `engine/docs/authoring/rule_library.md`).
- **Never edit**: `docs/FEEDBACK.md` or `docs/archive/FEEDBACK_CLOSED.md` (the Build Session is their sole writer), and never edit builder lanes (`-lane1..3`, `-stage`, `-plan`).

## 2. Startup Sequence (Run on Session Start)

1. Fetch and inspect the latest commits on `origin/develop`:
   `git -C C:\dev\tactical-rpg-dev fetch origin develop`
   `git -C C:\dev\tactical-rpg-dev log -8 --oneline origin/develop`
2. Sync `docs/FEEDBACK.md` and `docs/archive/FEEDBACK_CLOSED.md` from `origin/develop` into the working tree so Build-owned files are current.
3. Read `docs/READY.md` and `docs/FEEDBACK.md` from `origin/develop`.
4. Summarize for the user:
   - Which `READY.md` rows have landed or are currently building.
   - Any new open `FB-nn` or `UN-nn` items in `docs/FEEDBACK.md` that need Design answers, docs-delta application, or `READY.md` queue updates.
   - Proposed next actions (applying docs deltas, answering `FB-nn` in `READY.md` Section 4 with explicit `Disposition` and `Follow-up` columns, verifying with `python tools/planner/feedback_check.py`, running `/audit-build` via `build-auditor` to independently audit landed Build commits and evaluate system/decision health, or promoting Wave 2 slices from Section 3 to Section 1).

## 3. Committing & Pushing from Linked Worktree (`C:\dev\tactical-rpg-dev`)

Because `.git/worktrees/tactical-rpg-dev/index` is write-protected inside the agent sandbox, stage and push via a temporary `GIT_INDEX_FILE`, and then **restore the working tree on disk back to the local `.git` index** (Clean Pull-Ready Mode). This leaves `C:\dev\tactical-rpg-dev` 100% clean (`[behind 1]`, zero modified or untracked files) so the user's VS Code **Sync / Pull** button fast-forwards cleanly every time:

```powershell
Set-Location C:\dev\tactical-rpg-dev
git fetch -q origin develop
$env:GIT_INDEX_FILE = "$env:TEMP\dev_index"
if (Test-Path $env:GIT_INDEX_FILE) { Remove-Item $env:GIT_INDEX_FILE -Force }
git read-tree origin/develop
git add <design-owned-files>
$tree = git write-tree
$parent = git rev-parse origin/develop
$commit = "<commit message>" | git commit-tree $tree -p $parent
git push -q origin "${commit}:refs/heads/develop"
Remove-Item $env:GIT_INDEX_FILE -Force
Remove-Item Env:\GIT_INDEX_FILE
# Restore working tree to match local .git index so VS Code Sync/Pull fast-forwards with 0 conflicts
Copy-Item "C:\dev\tactical-rpg\.git\worktrees\tactical-rpg-dev\index" "$env:TEMP\restore_index" -Force
$env:GIT_INDEX_FILE = "$env:TEMP\restore_index"
git checkout-index -f -a
Remove-Item $env:GIT_INDEX_FILE -Force
Remove-Item Env:\GIT_INDEX_FILE
git clean -fd
```
