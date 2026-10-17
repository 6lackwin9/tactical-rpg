---
name: pc-launch
description: Launch the game (Scene Runner) on the user's Windows PC from the latest main. Use when the user says "launch", "PC launch", "run it on my PC", "open the game" or asks to play/test the merged build on their machine, from any agent - a local session, a cloud thread, or the project coordinator.
---

# PC launch

Opens the Scene Runner on the user's Windows PC from the latest `main`. Only a session running on that PC can start it; everyone else routes the request there.

- Checkout: `C:\Users\tusha\OneDrive\Project\Godot\tactical-rpg\new-game-project` (remote `6lackwin9/tactical-rpg`).
- Launcher: `run.bat` in the checkout (starts `C:\Program Files\Godot\godot.exe` on `res://scenes/scene_runner.tscn`).
- PC launch thread in the Tactical Rpg project: [Scene Runner on PC](#cmsg_01TtkZhEhzkkQVCpjCh92r1EQKnptj6wj1CvUYyrAVsoDu).

## 1. On the PC (Windows, `$CLAUDE_CODE_REMOTE` empty, the checkout path exists)

Run in PowerShell:

```powershell
cd C:\Users\tusha\OneDrive\Project\Godot\tactical-rpg\new-game-project
git status --short --branch
```

- If the tree is dirty or not on `main`: do not pull, stash, reset or switch branches. Tell the user what is uncommitted and stop.
- If clean on `main`:

```powershell
git pull --ff-only origin main; if ($?) { Start-Process .\run.bat; Start-Sleep 5; Get-Process godot -ErrorAction SilentlyContinue | Select-Object Id, StartTime }
```

- A `godot` process with a fresh `StartTime` means it launched. Report the commit (`git rev-parse --short HEAD`) in one line.
- Never kill or restart other `godot` processes (`Stop-Process -Name godot` stops other agents' suites and sims). Change no code and push nothing as part of a launch.

## 2. Anywhere else (cloud container, other machine)

You cannot start a window on the PC from here. Route it:

- **Thread Claude in a cloud thread**: `send_message` to `@parent` (visibility `sent_to_shared_agent`): "PC launch requested: please relay `launch` to the Scene Runner on PC thread." Tell the user in one line that the launch was handed to their PC.
- **Coordinator (main project window)**: `message_thread` the [Scene Runner on PC](#cmsg_01TtkZhEhzkkQVCpjCh92r1EQKnptj6wj1CvUYyrAVsoDu) thread with "launch", attaching the user's message id in `context_message_ids`, then a one-line `post_message` linking that thread. If that thread has no live session, start a thread session on the PC (Remote Control) with this skill as the brief.
- **Local Claude Code session on another device**: say the PC is the only place it can launch and point the user to the project thread above.
