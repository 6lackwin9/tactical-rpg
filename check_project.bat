@echo off
setlocal
cd /d "%~dp0"
echo =======================================================
echo === TACTICAL RPG: RUNNING POST-CODE ERROR CHECK    ===
echo =======================================================
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0check_project.ps1"
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Project check failed with exit code %ERRORLEVEL%.
    pause
    exit /b %ERRORLEVEL%
)
echo.
echo [OK] All checks completed successfully.
if "%~1"=="" pause
exit /b 0
