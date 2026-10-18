@echo off
rem Opens a design-review option build: run_opt.bat <a|b|c|branch> (see run.bat).
(
  call "%~dp0run.bat" opt %*
  exit /b
)
