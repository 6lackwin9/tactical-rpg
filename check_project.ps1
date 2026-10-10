# ==============================================================================
# Tactical RPG - Automated Project Error & Test Check Script
# ==============================================================================
param (
    [switch]$VerboseOutput
)

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $projectRoot

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "=== TACTICAL RPG: POST-CODE ERROR & TEST CHECK      ===" -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

# Locate Godot executable
$godotExe = $null
$cmd = Get-Command "godot" -ErrorAction SilentlyContinue
if ($cmd) {
    $godotExe = $cmd.Source
}
elseif (Test-Path "C:\Program Files\Godot\godot.exe") {
    $godotExe = "C:\Program Files\Godot\godot.exe"
}

if (-not $godotExe) {
    Write-Host "[ERROR] Godot executable not found in PATH or 'C:\Program Files\Godot\godot.exe'." -ForegroundColor Red
    exit 1
}

function Invoke-GodotProcess {
    param (
        [string[]]$Arguments
    )
    $stdoutFile = [System.IO.Path]::GetTempFileName()
    $stderrFile = [System.IO.Path]::GetTempFileName()

    $process = Start-Process -FilePath $godotExe -ArgumentList $Arguments -Wait -PassThru -NoNewWindow `
        -RedirectStandardOutput $stdoutFile -RedirectStandardError $stderrFile

    $outContent = Get-Content $stdoutFile -Raw -ErrorAction SilentlyContinue
    $errContent = Get-Content $stderrFile -Raw -ErrorAction SilentlyContinue

    Remove-Item $stdoutFile, $stderrFile -Force -ErrorAction SilentlyContinue

    return [PSCustomObject]@{
        ExitCode = $process.ExitCode
        Output   = ($outContent + "`n" + $errContent).Trim()
    }
}

# ------------------------------------------------------------------------------
# STEP 1: Syntax & Static Typing Check Across All GDScript Files
# ------------------------------------------------------------------------------
Write-Host "`n[1/3] Validating all GDScript files with Godot parser..." -ForegroundColor Yellow
$gdFiles = Get-ChildItem -Path $projectRoot -Recurse -Filter "*.gd" | Where-Object { $_.FullName -notmatch "\\\.godot\\" }
$scriptErrors = 0
$scriptsChecked = 0

foreach ($file in $gdFiles) {
    $relPath = $file.FullName.Replace($projectRoot + "\", "").Replace("\", "/")
    $resPath = "res://$relPath"
    $scriptsChecked++

    $result = Invoke-GodotProcess -Arguments @("--headless", "--check-only", "-s", $resPath)
    if ($result.ExitCode -ne 0) {
        $scriptErrors++
        Write-Host "  [FAIL] $relPath" -ForegroundColor Red
        if ($result.Output) {
            Write-Host "         $($result.Output)" -ForegroundColor DarkRed
        }
    }
    else {
        if ($VerboseOutput) {
            Write-Host "  [OK]   $relPath" -ForegroundColor DarkGray
        }
    }
}

if ($scriptErrors -gt 0) {
    Write-Host "`n[ERROR] $scriptErrors script(s) failed validation!" -ForegroundColor Red
    exit 1
}
else {
    Write-Host "  -> All $scriptsChecked GDScript files passed syntax validation." -ForegroundColor Green
}

# ------------------------------------------------------------------------------
# STEP 2: Refresh Global Class Cache & Symbols
# ------------------------------------------------------------------------------
Write-Host "`n[2/3] Verifying global class cache and asset imports..." -ForegroundColor Yellow
$editorResult = Invoke-GodotProcess -Arguments @("--headless", "--editor", "--quit")
if ($editorResult.ExitCode -ne 0) {
    Write-Host "[ERROR] Godot editor scan failed with exit code: $($editorResult.ExitCode)" -ForegroundColor Red
    Write-Host $editorResult.Output -ForegroundColor DarkRed
    exit 1
}
Write-Host "  -> Global class cache and imports up to date." -ForegroundColor Green

# ------------------------------------------------------------------------------
# STEP 3: Run Full Automated Headless Integration Test Suite
# ------------------------------------------------------------------------------
Write-Host "`n[3/3] Running Headless Integration Test Suite (test_arena.tscn)..." -ForegroundColor Yellow
$testResult = Invoke-GodotProcess -Arguments @("--headless", "res://scenes/test_arena.tscn")

$testLines = $testResult.Output -split "`r?`n"
$passCount = 0
$failCount = 0

foreach ($line in $testLines) {
    if ($line -match "PASS") {
        $passCount++
        Write-Host "  $line" -ForegroundColor Green
    }
    elseif ($line -match "FAIL") {
        $failCount++
        Write-Host "  $line" -ForegroundColor Red
    }
    elseif ($line -match "=== STARTING" -or $line -match "=== FINAL" -or $line -match "=== COMBAT" -or $line -match "=== RESOLUTION") {
        Write-Host "  $line" -ForegroundColor Cyan
    }
}

if ($testResult.ExitCode -ne 0 -or $failCount -gt 0) {
    Write-Host "`n[ERROR] Integration tests failed! ($failCount failed assertions, exit code: $($testResult.ExitCode))" -ForegroundColor Red
    exit 1
}

Write-Host "`n=======================================================" -ForegroundColor Cyan
Write-Host "=== [SUCCESS] ALL CHECKS PASSED: 0 ERRORS DETECTED  ===" -ForegroundColor Green
Write-Host "=== Total Scripts: $scriptsChecked | Assertions Passed: $passCount     ===" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Cyan
exit 0
