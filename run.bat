@echo off
rem Opens the Scene Runner (main menu) on the latest code: launch any scene, hot-reload scripts, run test suites.
rem   run.bat        latest main (released code) in this folder
rem   run.bat dev    latest develop (finished tasks not yet released) in the sibling folder
rem                  <this folder>-dev, a git worktree created on first use; run_dev.bat does the same.
rem                  Inside the -dev folder, run.bat and run_dev.bat both open develop there.
rem Two folders, so switching between main and dev never waits for a script cache rebuild.
rem The dev game also keeps its own saves and settings: Godot puts user:// under %APPDATA%,
rem so dev runs with APPDATA pointed at %APPDATA%\tactical-rpg-dev (user 2026-10-17).
rem When a pull changed the code, it first copies that game's saves (user://saves) to
rem user://save_backups/<date>-<old commit> and keeps the newest 10 (user 2026-10-17).
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
  if "!BRANCH!"=="develop" (
    set "APPDATA=!APPDATA!\tactical-rpg-dev"
    if not exist "!APPDATA!" mkdir "!APPDATA!"
    echo Dev saves and settings: !APPDATA!
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
  set "SAVES=!APPDATA!\Godot\app_userdata\rpg\saves"
  set "BACKUPS=!APPDATA!\Godot\app_userdata\rpg\save_backups"
  if defined OLD if not "!OLD!"=="!NEW!" if exist "!SAVES!\*" (
    set "STAMP="
    for /f %%t in ('powershell -NoProfile -Command "Get-Date -Format yyyyMMdd-HHmmss"') do set "STAMP=%%t"
    if not defined STAMP set "STAMP=!OLD:~0,10!"
    echo Code updated: backing up your saves to !BACKUPS!\!STAMP!-!OLD:~0,7!
    robocopy "!SAVES!" "!BACKUPS!\!STAMP!-!OLD:~0,7!" /e /njh /njs /nfl /ndl /np >nul
    for /f "skip=10 delims=" %%d in ('dir /b /ad /o-n "!BACKUPS!" 2^>nul') do rd /s /q "!BACKUPS!\%%d"
  )
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
