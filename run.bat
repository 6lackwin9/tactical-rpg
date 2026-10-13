@echo off
rem Opens the Scene Runner (main menu): launch any scene, hot-reload scripts, run test suites.
start "" "C:\Program Files\Godot\godot.exe" --path "%~dp0." res://scenes/scene_runner.tscn
