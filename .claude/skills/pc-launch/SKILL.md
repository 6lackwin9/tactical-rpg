---
name: pc-launch
description: Launch the game (Scene Runner) on the user's Windows PC from the latest main (or develop on request). Use when the user says "launch", "PC launch", "run it on my PC", "open the game" or asks to play/test the merged build on their machine, from any agent - a local session, a cloud thread, or the project coordinator.
---

# PC launch

Opens the Scene Runner on the user's Windows PC from the latest `main` (or `develop` when the user asks to test unreleased work). Only a session running on that PC can start it; everyone else routes the request there.

- Checkout: `C:\Users\tusha\OneDrive\Project\Godot\tactical-rpg\new-game-project` (remote `6lackwin9/tactical-rpg`).
- Launcher: `run.bat` in the checkout (user 2026-10-17: pull on launch). It fetches and fast-forwards the branch (`run.bat` = `main` in this checkout; `run.bat dev` or `run_dev.bat` = `develop` in the sibling folder `new-game-project-dev`, a git worktree created on first use, so main and dev are two builds and switching never waits for a cache rebuild), rebuilds the script cache only when the code changed, prints the branch and commit, then starts `C:\Program Files\Godot\godot.exe` on `res://scenes/scene_runner.tscn`. Offline, a failed pull, local changes or a folder on another branch: it launches the copy on disk without touching it.
- PC launch thread in the Tactical Rpg project: [Launch game on your PC](#cmsg_01TtkZhEhzkkQVCpjCh92r1EQKnptj6wj1CvUYyrAVsoDu).

## 1. On the PC (Windows, `$CLAUDE_CODE_REMOTE` empty, the checkout path exists)

Run in PowerShell:

```powershell
cd C:\Users\tusha\OneDrive\Project\Godot\tactical-rpg\new-game-project
git status --short --branch
```

- If the tree is dirty: do not pull, stash, reset or switch branches. Tell the user what is uncommitted and stop (`run.bat` would launch the old copy).
- If clean on `main` (dev runs from the `-dev` folder, which `run.bat` checks the same way; on any other branch tell the user and stop):

```powershell
Start-Process .\run.bat; Start-Sleep 8; Get-Process godot -ErrorAction SilentlyContinue | Select-Object Id, StartTime
```

  For unreleased work ("launch dev") use `Start-Process .\run.bat -ArgumentList dev`; the first dev launch creates the `-dev` folder and imports it (a few minutes). A pull that changed code makes `run.bat` rebuild the script cache first (`godot --headless --import`, about a minute), so a fresh window can take that long; a blank grey window means the cache is still stale (`scene_runner.gd` "Could not find type ..."): run that import and launch again (see `godot-known-issues`).
- A `godot` process with a fresh `StartTime` means it launched. Report the commit (`git rev-parse --short HEAD`) in one line.
- Never kill or restart other `godot` processes (`Stop-Process -Name godot` stops other agents' suites and sims). Change no code and push nothing as part of a launch.

## 2. Anywhere else (cloud container, other machine)

You cannot start a window on the PC from here. Route it:

- **Thread Claude in a cloud thread**: get the coordinator's id with `get_channel_session_id`, then `send_message` to that `session_id`: "PC launch requested: please relay `launch` to the Launch game on your PC thread." Tell the user in one line that the launch was handed to their PC.
- **Coordinator (main project window)**: `message_thread` the [Launch game on your PC](#cmsg_01TtkZhEhzkkQVCpjCh92r1EQKnptj6wj1CvUYyrAVsoDu) thread with "launch", attaching the user's message id in `context_message_ids`, then a one-line `post_message` linking that thread. If that thread has no live session, start a thread session on the PC (Remote Control) with this skill as the brief.
- **Local Claude Code session on another device**: say the PC is the only place it can launch and point the user to the project thread above.
