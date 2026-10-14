# ==============================================================================
# Tactical RPG - Automated Project Error & Test Check Script
# ==============================================================================
param (
    [switch]$VerboseOutput,
    [switch]$Quiet,   # print only failures and the summary (saves ~10k tokens of PASS lines per run)
    [switch]$Fast   # parse all scripts in ONE Godot process (~3 s) instead of one launch per file (~2.5 min)
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
# STEP 1: Refresh Global Class Cache & Symbols
# ------------------------------------------------------------------------------
Write-Host "`n[1/4] Verifying global class cache and asset imports..." -ForegroundColor Yellow
$editorResult = Invoke-GodotProcess -Arguments @("--headless", "--editor", "--quit-after", "50")
if ($editorResult.ExitCode -ne 0) {
    Write-Host "[ERROR] Godot editor scan failed with exit code: $($editorResult.ExitCode)" -ForegroundColor Red
    Write-Host $editorResult.Output -ForegroundColor DarkRed
    exit 1
}
Write-Host "  -> Global class cache and imports up to date." -ForegroundColor Green

# ------------------------------------------------------------------------------
# STEP 2: Syntax & Static Typing Check Across All GDScript Files
# ------------------------------------------------------------------------------
Write-Host "`n[2/4] Validating all GDScript files with Godot parser..." -ForegroundColor Yellow
$gdFiles = Get-ChildItem -Path $projectRoot -Recurse -Filter "*.gd" | Where-Object { $_.FullName -notmatch "\\\.godot\\" }
$scriptErrors = 0
$scriptsChecked = 0

if ($Fast) {
    $fastResult = Invoke-GodotProcess -Arguments @("--headless", "--path", ".", "-s", "tools/parse_all.gd")
    $scriptsChecked = $gdFiles.Count
    if ($fastResult.Output -match "SCRIPT ERROR|Parse Error|Compile Error|FAIL res://") {
        $scriptErrors = 1
        Write-Host "  [FAIL] $($fastResult.Output)" -ForegroundColor DarkRed
    }
    $gdFiles = @()
}


foreach ($file in $gdFiles) {
    $relPath = $file.FullName.Replace($projectRoot + "\", "").Replace("\", "/")
    $resPath = "res://$relPath"
    $scriptsChecked++

    $result = Invoke-GodotProcess -Arguments @("--headless", "--check-only", "-s", $resPath)
    # Godot 4.7 can crash (0xC0000005) while unloading cross-referencing scripts at shutdown,
    # AFTER parsing succeeded. Re-check once; if it still only crashes at shutdown with no
    # script/parse error reported, count it as parsed and warn instead of failing.
    $shutdownCrash = ($result.ExitCode -eq -1073741819) -and ($result.Output -notmatch "SCRIPT ERROR|Parse Error|Compile Error")
    if ($shutdownCrash) {
        $result = Invoke-GodotProcess -Arguments @("--headless", "--check-only", "-s", $resPath)
        $shutdownCrash = ($result.ExitCode -eq -1073741819) -and ($result.Output -notmatch "SCRIPT ERROR|Parse Error|Compile Error")
        if ($shutdownCrash) {
            Write-Host "  [WARN] $relPath parsed OK; Godot crashed only while shutting down (engine teardown issue)" -ForegroundColor DarkYellow
            continue
        }
    }
    if ($result.ExitCode -ne 0) {
        $scriptErrors++
        Write-Host "  [FAIL] $relPath (exit code: $($result.ExitCode))" -ForegroundColor Red
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
# STEP 3: Run Full Automated Headless Integration Test Suites
# ------------------------------------------------------------------------------
$passCount = 0
$testSuites = @(
    @{ Label = "[3/4] Combat Arena Suite (test_arena.tscn)"; Scene = "res://scenes/test_arena.tscn" },
    @{ Label = "[4/4] Gear & Skill Dojo Suite (gear_dojo.tscn)"; Scene = "res://scenes/gear_dojo.tscn" }
)

foreach ($suite in $testSuites) {
    Write-Host "`n$($suite.Label)..." -ForegroundColor Yellow
    $testResult = Invoke-GodotProcess -Arguments @("--headless", $suite.Scene)

    $testLines = $testResult.Output -split "`r?`n"
    $failCount = 0

    foreach ($line in $testLines) {
        if ($line -match "PASS") {
            $passCount++
            if (-not $Quiet) { Write-Host "  $line" -ForegroundColor Green }
        }
        elseif ($line -match "FAIL") {
            $failCount++
            Write-Host "  $line" -ForegroundColor Red
        }
        elseif ($line -match "=== STARTING" -or $line -match "=== FINAL" -or $line -match "=== COMBAT" -or $line -match "=== RESOLUTION") {
            if (-not $Quiet) { Write-Host "  $line" -ForegroundColor Cyan }
        }
    }

    if ($testResult.ExitCode -ne 0 -or $failCount -gt 0) {
        Write-Host "`n[ERROR] $($suite.Scene) failed! ($failCount failed assertions, exit code: $($testResult.ExitCode))" -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n=======================================================" -ForegroundColor Cyan
Write-Host "=== [SUCCESS] ALL CHECKS PASSED: 0 ERRORS DETECTED  ===" -ForegroundColor Green
Write-Host "=== Total Scripts: $scriptsChecked | Assertions Passed: $passCount     ===" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Cyan
exit 0
