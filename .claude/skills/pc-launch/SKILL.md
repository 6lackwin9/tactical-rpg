---
name: pc-launch
description: Launch the game (the engine's Scene Runner) on the user's Windows PC from the latest main (or develop on request). Use when the user says "launch", "PC launch", "run it on my PC", "open the game" or asks to play/test the merged build on their machine, from any agent - a local session, a cloud thread, or the project coordinator.
---

# PC launch

Opens the Aegis engine's Scene Runner (`aegis-gui screen=runner`) on the user's Windows PC from the latest `main` (or `develop` when the user asks to test unreleased work). Only a session running on that PC can start it; everyone else routes the request there.

- Checkout: `C:\dev\tactical-rpg` (remote `6lackwin9/tactical-rpg`; moved out of OneDrive on 2026-10-19, decision "Repository outside OneDrive"; the old OneDrive folder is no longer used).
- Launcher: `run.bat` in the checkout (user 2026-10-17: pull on launch). It fetches and fast-forwards the branch (`run.bat` = `main` in this checkout; `run.bat dev` or `run_dev.bat` = `develop` in the sibling folder `tactical-rpg-dev`, a git worktree created on first use; dev keeps its own saves and settings under `%APPDATA%\tactical-rpg-dev`; when a pull changed the code it backs up `%APPDATA%\Aegis\saves` to `%APPDATA%\Aegis\save_backups\<date>-<old commit>`, newest 10 kept), builds the engine in release (`cargo build --release -p aegis_gui -p emberwake_host`, `aegis_cli` on branches before SPLIT1-6, each folder into its own `%LOCALAPPDATA%\Temp\aegis-target[-dev|-opt]`; a first build takes a few minutes, later ones seconds), prints the branch and commit, then starts `aegis-gui.exe screen=runner` through PowerShell `Start-Process`. Offline, a failed pull, local changes or a folder on another branch: it launches the copy on disk without touching it; a failed build launches the last build.
- PC launch thread in the Tactical Rpg project: "Launch game on your PC" (thread `cmsg_01TtkZhEhzkkQVCpjCh92r1EQKnptj6wj1CvUYyrAVsoDu`).

## 1. On the PC (Windows, `$CLAUDE_CODE_REMOTE` empty, the checkout path exists)

Run in PowerShell:

```powershell
cd C:\dev\tactical-rpg
git status --short --branch
```

- If the tree is dirty: do not pull, stash, reset or switch branches. Tell the user what is uncommitted and stop (`run.bat` would launch the old copy).
- If clean on `main` (dev runs from the `-dev` folder, which `run.bat` checks the same way; on any other branch tell the user and stop):

```powershell
Start-Process .\run.bat; Start-Sleep 20; Get-Process aegis-gui -ErrorAction SilentlyContinue | Select-Object Id, StartTime
```

  For a design-review option ("launch option A/B/C" or "launch option" plus a branch name) use `Start-Process .\run.bat -ArgumentList 'opt','a'` (or `b`, `c`, a branch name): it opens the newest `claude/design-opt-<date>-<letter>` in the sibling worktree `tactical-rpg-opt` with its own saves under `%APPDATA%\tactical-rpg-opt`; an unknown option, or a branch older than the switch to the engine, prints a message and does not launch. Report the branch and commit it echoes.

  For unreleased work ("launch dev") use `Start-Process .\run.bat -ArgumentList dev`; the first dev launch creates the `-dev` folder and builds it (a few minutes). A pull that changed code rebuilds before the window opens, so allow for that before checking.

- An `aegis-gui` process with a fresh `StartTime` means it launched. Report the commit (`git rev-parse --short HEAD`) in one line. To confirm the screen, `aegis link '{"cmd":"ui_tree"}'` (the agent link on port 47800) answers with the open screen.
- Never kill processes you did not start (other agents run tests and sims). Change no code and push nothing as part of a launch.
- The Godot build (frozen in `legacy/`) is not launched by `run.bat`; if the user asks for it, open `legacy/project.godot` in Godot 4.7.2 (`"C:\Program Files\Godot\godot.exe" --path legacy`).

## 2. Anywhere else (cloud container, other machine)

You cannot start a window on the PC from here. Route it:

- **Thread Claude in a cloud thread**: get the coordinator's id with `get_channel_session_id`, then `send_message` to that `session_id`: "PC launch requested: please relay `launch` to the Launch game on your PC thread." Tell the user in one line that the launch was handed to their PC.
- **Coordinator (main project window)**: `message_thread` the "Launch game on your PC" (thread `cmsg_01TtkZhEhzkkQVCpjCh92r1EQKnptj6wj1CvUYyrAVsoDu`) thread with "launch", attaching the user's message id in `context_message_ids`, then a one-line `post_message` linking that thread. If that thread has no live session, start a thread session on the PC (Remote Control) with this skill as the brief.
- **Local Claude Code session on another device**: say the PC is the only place it can launch and point the user to the project thread above.
