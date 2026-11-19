# ==============================================================================
# Tactical RPG - commit gate on the Aegis engine (transition step 2, 2026-10-19)
# GATE-PARITY1 (ENGINE-12): runs the steps of tools/gate_steps.txt in order, the same list `tools/fc.sh gate` runs, so the
# desktop gate and the cloud gate cannot drift. The build and the tests run here natively (a compiler warning fails the
# build; Windows Smart App Control blocks on fresh test binaries are retried); every other step runs as
# `tools/fc.sh step <name>` through the bash that comes with git, so one copy of each check exists. The Godot gate of the
# frozen Godot copy is legacy/check_project.ps1.
# ==============================================================================
param (
    [switch]$VerboseOutput,   # print the full cargo output
    [switch]$Quiet,           # print only failures and the summary
    [switch]$Fast             # accepted for compatibility: the engine gate has one speed
)

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
# A linked worktree (.git is a file) builds into its own folder, so parallel lanes never share cargo's build lock or
# overwrite each other's binaries (.cargo/config.toml names one folder for every checkout).
if (-not $env:CARGO_TARGET_DIR -and (Test-Path (Join-Path $projectRoot ".git") -PathType Leaf)) {
    $base = (Select-String -Path (Join-Path $projectRoot ".cargo\config.toml") -Pattern '^target-dir = "(.*)"').Matches[0].Groups[1].Value
    $env:CARGO_TARGET_DIR = "$base-$(Split-Path -Leaf $projectRoot)"
}
# Low-power laptop (CPU1): cargo runs below normal priority, so the machine stays usable; children inherit it.
# Only when this process was started for the gate: a run from an open terminal must not leave that terminal lowered.
if ([Environment]::CommandLine -match 'check_project') { try { (Get-Process -Id $PID).PriorityClass = 'BelowNormal' } catch {} }
Set-Location $projectRoot   # the Cargo workspace is at the repo root (SPLIT1-1)

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] cargo not found: install Rust (rustup) and reopen the terminal." -ForegroundColor Red
    exit 1
}

function Invoke-Cargo([string[]]$CargoArgs) {
    $out = & cmd /c "cargo $($CargoArgs -join ' ') 2>&1"
    return @{ Code = $LASTEXITCODE; Lines = @($out) }
}

# The bash that comes with git (tools/fc.sh steps): bin/bash.exe sets up its own PATH; a `bash` on PATH may be the WSL one.
function Find-Bash {
    $gitRoot = Split-Path (Split-Path (Split-Path (& git --exec-path)))
    foreach ($c in @("bin/bash.exe", "usr/bin/bash.exe", "bin/bash")) {
        $p = Join-Path $gitRoot $c
        if (Test-Path $p -PathType Leaf) { return $p }
    }
    $onPath = Get-Command bash -ErrorAction SilentlyContinue | Where-Object { $_.Source -notmatch 'System32' } | Select-Object -First 1
    if ($onPath) { return $onPath.Source }
    return $null
}

function Invoke-Fc([string[]]$FcArgs) {
    $global:LASTEXITCODE = 1   # a bash that did not start must not read as the last command's success
    $out = & $script:bash tools/fc.sh @FcArgs 2>&1
    return @{ Code = $global:LASTEXITCODE; Lines = @($out | ForEach-Object { "$_" }) }
}

$stepsFile = Join-Path $projectRoot "tools/gate_steps.txt"
$steps = @(Get-Content $stepsFile | Where-Object { $_ -notmatch '^\s*(#|$)' } | ForEach-Object { ($_.Trim() -split '\s+')[0] })
if ($steps.Count -eq 0) {
    Write-Host "[ERROR] no gate steps in tools/gate_steps.txt" -ForegroundColor Red
    exit 1
}
$script:bash = Find-Bash
if (-not $script:bash) {
    Write-Host "=== [FAILED] no bash found next to git: the gate steps after the tests run through tools/fc.sh ===" -ForegroundColor Red
    exit 1
}

Write-Host "=== AEGIS: COMMIT GATE ($($steps.Count) steps, tools/gate_steps.txt) ===" -ForegroundColor Cyan
$n = 0
$test = $null
foreach ($step in $steps) {
    $n++
    Write-Host "[$n/$($steps.Count)] $step..."
    switch ($step) {
        "build" {
            # Every target (tests included); a compiler warning fails the gate like an error (cargo replays cached warnings, CI1).
            $build = Invoke-Cargo @("build", "--workspace", "--all-targets")
            if ($VerboseOutput) { $build.Lines | ForEach-Object { Write-Host $_ } }
            $warned = @($build.Lines | Where-Object { $_ -match "^warning" }).Count -gt 0
            if ($build.Code -ne 0 -or $warned) {
                $build.Lines | Where-Object { $_ -match "^(error|warning)|-->" } | Select-Object -First 60 | ForEach-Object { Write-Host $_ -ForegroundColor Red }
                Write-Host "=== [FAILED] build$(if ($build.Code -eq 0) { ': compiler warnings' }) ===" -ForegroundColor Red
                exit 1
            }
            if (-not $Quiet) { Write-Host "  -> build ok" -ForegroundColor Green }
        }
        "test" {
            # Windows Smart App Control sometimes blocks a freshly linked test binary (os error 4551): delete it and retry.
            for ($try = 1; $try -le 4; $try++) {
                $test = Invoke-Cargo @("test", "--workspace", "--no-fail-fast")
                $blocked = @($test.Lines | Select-String -Pattern 'could not execute process `([^`]*\.exe)`' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Select-Object -Unique)
                if ($blocked.Count -eq 0) { break }
                Write-Host "  -> Smart App Control blocked $($blocked.Count) test binary(ies); relinking (try $try)" -ForegroundColor Yellow
                $blocked | ForEach-Object { Remove-Item -LiteralPath $_ -Force -ErrorAction SilentlyContinue }
            }
            if ($VerboseOutput) { $test.Lines | ForEach-Object { Write-Host $_ } }
            $failed = ($test.Code -ne 0) -or ($test.Lines -match "os error 4551")
            if ($failed) {
                $test.Lines | Where-Object { $_ -match "FAILED|panicked|^error|os error|differ|diverge" } | Select-Object -First 60 | ForEach-Object { Write-Host $_ -ForegroundColor Red }
                Write-Host "=== [FAILED] tests ===" -ForegroundColor Red
                exit 1
            }
            if (-not $Quiet) { $test.Lines | Where-Object { $_ -match "checked|diverge|compared|panicked|os error" } | ForEach-Object { Write-Host "  $_" } }
        }
        default {
            # One copy of each check: `tools/fc.sh step <name>` (its output; a SKIP always shows, it means a check did not run).
            $r = Invoke-Fc @("step", $step)
            if ($r.Code -ne 0) {
                $r.Lines | Select-Object -Last 60 | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
                Write-Host "=== [FAILED] $step ===" -ForegroundColor Red
                exit 1
            }
            if (-not $Quiet -or ($r.Lines -match 'SKIP')) { $r.Lines | ForEach-Object { Write-Host "  $_" -ForegroundColor $(if ($_ -match 'SKIP') { 'Yellow' } else { 'Green' }) } }
        }
    }
}

$passed = 0
if ($test) { $test.Lines | Select-String -Pattern "test result: ok\. (\d+) passed" | ForEach-Object { $passed += [int]$_.Matches[0].Groups[1].Value } }
Write-Host "=== [SUCCESS] ALL CHECKS PASSED: $passed tests ===" -ForegroundColor Green
exit 0
