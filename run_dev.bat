@echo off
rem Opens the game on the latest develop (finished tasks not yet released to main).
(
  call "%~dp0run.bat" dev
  exit /b
)
