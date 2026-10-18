@echo off
rem Opens the Scene Runner (main menu) on the latest code: launch any scene, hot-reload scripts, run test suites.
rem   run.bat        latest main (released code) in this folder
rem   run.bat dev    latest develop (finished tasks not yet released) in the sibling folder
rem                  <this folder>-dev, a git worktree created on first use; run_dev.bat does the same.
rem                  Inside the -dev folder, run.bat and run_dev.bat both open develop there.
rem   run.bat opt <a|b|c|branch>
rem                  an option build of a design review: a, b or c = the newest remote branch
rem                  claude/design-opt-<date>-<letter>; anything else is a branch name (origin/ optional).
rem                  Opens it in the sibling folder <this folder>-opt (a detached git worktree, created on
rem                  first use, switched to the branch tip on later launches unless it has local changes),
rem                  with its own saves and settings under %APPDATA%\tactical-rpg-opt. The branch name is
rem                  kept in <opt folder>\.opt_branch, so run.bat with no args inside the -opt folder
rem                  (Restart runner) reopens the same option on its newest commit; run_opt.bat does the same.
rem Scene Runner's "Restart runner" button (DEVELOPER panel) calls this file to pull.
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
  set "OPTRUN="
  set "INOPT="
  set "PD=%~dp0"
  if /i "!PD:~-5!"=="-opt\" set "INOPT=1"
  if /i "%~1"=="opt" set "OPTRUN=1"
  if defined INOPT if "%~1"=="" set "OPTRUN=1"
  if defined OPTRUN (
    set "BRANCH=opt"
    set "OB=%~2"
    if not defined OB if defined INOPT if exist "%~dp0.opt_branch" set /p OB=<"%~dp0.opt_branch"
    if not defined OB (
      echo Which option? Use run.bat opt a, b, c or a branch name.
      timeout /t 8 /nobreak >nul
      exit /b
    )
    git fetch --quiet origin 2>nul
    if errorlevel 1 echo Could not reach GitHub: using the branches you have.
    set "OBR=!OB!"
    for %%l in (a b c) do if /i "!OB!"=="%%l" (
      set "OBR="
      for /f "tokens=3" %%r in ('git for-each-ref --sort=-committerdate refs/remotes/origin/claude/design-opt-*-%%l 2^>nul') do if not defined OBR set "OBR=%%r"
      if defined OBR set "OBR=!OBR:refs/remotes/origin/=!"
    )
    if defined OBR if /i "!OBR:~0,7!"=="origin/" set "OBR=!OBR:~7!"
    if defined OBR (
      git rev-parse --verify --quiet "refs/remotes/origin/!OBR!" >nul 2>&1
      if errorlevel 1 set "OBR="
    )
    if not defined OBR (
      echo No option branch found for: !OB!
      echo Use run.bat opt a, b, c or a branch name that exists on origin.
      timeout /t 8 /nobreak >nul
      exit /b
    )
    set "OPTNAME=Option"
    if /i "!OBR:~-2!"=="-a" set "OPTNAME=Option A"
    if /i "!OBR:~-2!"=="-b" set "OPTNAME=Option B"
    if /i "!OBR:~-2!"=="-c" set "OPTNAME=Option C"
    if not defined INOPT for %%d in ("%~dp0.") do set "DIR=%%~fd-opt"
    set "OPTNEW="
    if not exist "!DIR!\project.godot" (
      echo First option launch: creating !DIR! ...
      git worktree prune 2>nul
      git worktree add --quiet --detach "!DIR!" "origin/!OBR!"
      set "OPTNEW=1"
    )
    if not exist "!DIR!\project.godot" (
      echo Could not create the option folder: not launching.
      timeout /t 8 /nobreak >nul
      exit /b
    )
  )
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
  set "BASEAPP=!APPDATA!"
  if /i "!BASEAPP:~-17!"=="\tactical-rpg-dev" set "BASEAPP=!BASEAPP:~0,-17!"
  if /i "!BASEAPP:~-17!"=="\tactical-rpg-opt" set "BASEAPP=!BASEAPP:~0,-17!"
  if "!BRANCH!"=="opt" (
    set "APPDATA=!BASEAPP!\tactical-rpg-opt"
    if not exist "!APPDATA!" mkdir "!APPDATA!"
    echo Option saves and settings: !APPDATA!
  )
  if "!BRANCH!"=="develop" (
    rem Restart runner starts this from a dev game that already has the dev APPDATA: do not nest it.
    set "APPDATA=!BASEAPP!\tactical-rpg-dev"
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
  if "!BRANCH!"=="opt" (
    if defined DIRTY (
      echo Local changes found in !DIR!: launching without switching.
      set "OLDB="
      if exist "!DIR!\.opt_branch" set /p OLDB=<"!DIR!\.opt_branch"
      if defined OLDB if not "!OLDB!"=="!OBR!" (
        echo The option folder has local changes: launching !OLDB! instead of !OBR!.
        set "OBR=!OLDB!"
        set "OPTNAME=Option"
        if /i "!OBR:~-2!"=="-a" set "OPTNAME=Option A"
        if /i "!OBR:~-2!"=="-b" set "OPTNAME=Option B"
        if /i "!OBR:~-2!"=="-c" set "OPTNAME=Option C"
      )
    ) else (
      if not defined OPTNEW (
        git switch --quiet --detach "origin/!OBR!" 2>nul
        if errorlevel 1 echo Could not switch to !OBR!: launching the copy you have.
      )
      >"!DIR!\.opt_branch" echo(!OBR!
    )
  ) else if defined DIRTY (
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
  if "!BRANCH!"=="opt" (
    echo !OPTNAME!: !OBR! at !SHA!
  ) else (
    echo Running !CUR! at !SHA! from !DIR!
  )
  start "" "!GODOT!" --path "!DIR!" res://scenes/scene_runner.tscn
  timeout /t 4 /nobreak >nul
  exit /b
)
