@echo off
rem Opens the Scene Runner (main menu) on the latest code: launch any scene, hot-reload scripts, run test suites.
rem   run.bat        latest main (released code) in this folder
rem   run.bat dev    latest develop (finished tasks not yet released) in the sibling folder
rem                  <this folder>-dev, a git worktree created on first use; run_dev.bat does the same.
rem                  Inside the -dev folder, run.bat and run_dev.bat both open develop there.
rem Two folders, so switching between main and dev never waits for a script cache rebuild.
rem Before launching it fetches and fast-forwards the branch (about 1-2 s when nothing changed)
rem and rebuilds the script cache only when the code changed (about a minute). It launches the
rem copy on disk when offline, when the pull fails, when that folder has local changes or is on
rem another branch (it never stashes, resets or merges). Everything runs inside one ( ) block that
rem ends with exit /b: cmd parses it before running, so a pull that rewrites this file is safe.
setlocal EnableDelayedExpansion
(
  set "GODOT=C:\Program Files\Godot\godot.exe"
  set "DIR=%~dp0."
  cd /d "%~dp0"
  set "HERE=?"
  for /f %%b in ('git rev-parse --abbrev-ref HEAD 2^>nul') do set "HERE=%%b"
  set "BRANCH=main"
  if /i "%~1"=="dev" set "BRANCH=develop"
  if /i "%~1"=="develop" set "BRANCH=develop"
  if "!HERE!"=="develop" set "BRANCH=develop"
  if "!BRANCH!"=="develop" if not "!HERE!"=="develop" (
    for %%d in ("%~dp0.") do set "DIR=%%~fd-dev"
    if not exist "!DIR!\project.godot" (
      echo First dev launch: creating !DIR! ...
      git worktree prune 2>nul
      git fetch --quiet origin develop 2>nul
      git worktree add --quiet "!DIR!" develop 2>nul
      if errorlevel 1 git worktree add --quiet -b develop "!DIR!" origin/develop
    )
    if not exist "!DIR!\project.godot" (
      echo Could not create the dev folder: launching main instead.
      set "BRANCH=main"
      set "DIR=%~dp0."
    )
  )
  cd /d "!DIR!"
  set "OLD="
  for /f %%h in ('git rev-parse HEAD 2^>nul') do set "OLD=%%h"
  set "CUR=?"
  for /f %%b in ('git rev-parse --abbrev-ref HEAD 2^>nul') do set "CUR=%%b"
  set "DIRTY="
  for /f %%l in ('git status --porcelain --untracked-files=no 2^>nul') do set "DIRTY=1"
  if defined DIRTY (
    echo Local changes found: launching without updating.
  ) else if not "!CUR!"=="!BRANCH!" (
    echo This folder is on !CUR!, not !BRANCH!: launching without updating.
  ) else (
    git fetch --quiet origin !BRANCH! 2>nul
    if errorlevel 1 (
      echo Could not reach GitHub: launching the copy you have.
    ) else (
      git merge --ff-only --quiet origin/!BRANCH! 2>nul
      if errorlevel 1 echo Could not fast-forward !BRANCH!: launching the copy you have.
    )
  )
  set "NEW="
  for /f %%h in ('git rev-parse HEAD 2^>nul') do set "NEW=%%h"
  if not exist ".godot\global_script_class_cache.cfg" set "OLD=none"
  if not "!OLD!"=="!NEW!" (
    echo Code updated: rebuilding the script cache, about a minute...
    "!GODOT!" --headless --import --path . >nul 2>&1
  )
  set "SHA=?"
  for /f %%h in ('git rev-parse --short HEAD 2^>nul') do set "SHA=%%h"
  echo Running !CUR! at !SHA! from !DIR!
  start "" "!GODOT!" --path "!DIR!" res://scenes/scene_runner.tscn
  timeout /t 4 /nobreak >nul
  exit /b
)
