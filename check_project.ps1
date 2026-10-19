# ==============================================================================
# Tactical RPG - commit gate on the Aegis engine (transition step 2, 2026-10-19)
# Builds every engine crate, then runs every test: the rules, battle, world, data-loader, content-writer and Godot
# parity suites (the parity suites skip themselves when games/emberwake/parity/out is missing; tools/fc.sh parity records it
# from legacy/). Same steps as `tools/fc.sh gate`. The Godot gate of the frozen Godot copy is legacy/check_project.ps1.
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
Set-Location $projectRoot   # the Cargo workspace is at the repo root (SPLIT1-1)

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] cargo not found: install Rust (rustup) and reopen the terminal." -ForegroundColor Red
    exit 1
}

function Invoke-Cargo([string[]]$CargoArgs) {
    $out = & cmd /c "cargo $($CargoArgs -join ' ') 2>&1"
    return @{ Code = $LASTEXITCODE; Lines = @($out) }
}

Write-Host "=== AEGIS: COMMIT GATE ===" -ForegroundColor Cyan

Write-Host "[1/2] Building every crate..."
$build = Invoke-Cargo @("build", "--workspace")
if ($VerboseOutput) { $build.Lines | ForEach-Object { Write-Host $_ } }
if ($build.Code -ne 0) {
    $build.Lines | Where-Object { $_ -match "^(error|warning)|-->" } | Select-Object -First 60 | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    Write-Host "=== [FAILED] build ===" -ForegroundColor Red
    exit 1
}
if (-not $Quiet) { Write-Host "  -> build ok" -ForegroundColor Green }

Write-Host "[2/2] Running every test..."
# Windows Smart App Control sometimes blocks a freshly linked test binary (os error 4551): delete it and retry.
$test = $null
for ($try = 1; $try -le 4; $try++) {
    $test = Invoke-Cargo @("test", "--workspace", "--no-fail-fast")
    $blocked = @($test.Lines | Select-String -Pattern 'could not execute process `([^`]*\.exe)`' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Select-Object -Unique)
    if ($blocked.Count -eq 0) { break }
    Write-Host "  -> Smart App Control blocked $($blocked.Count) test binary(ies); relinking (try $try)" -ForegroundColor Yellow
    $blocked | ForEach-Object { Remove-Item -LiteralPath $_ -Force -ErrorAction SilentlyContinue }
}
if ($VerboseOutput) { $test.Lines | ForEach-Object { Write-Host $_ } }
$summary = $test.Lines | Where-Object { $_ -match "match Godot|checked|diverge|catalog rows|compared|^test result: FAILED|panicked|os error" }
$failed = ($test.Code -ne 0) -or ($test.Lines -match "os error 4551")
if ($failed) {
    $test.Lines | Where-Object { $_ -match "FAILED|panicked|^error|os error|differ|diverge" } | Select-Object -First 60 | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    Write-Host "=== [FAILED] tests ===" -ForegroundColor Red
    exit 1
}
if (-not $Quiet) { $summary | ForEach-Object { Write-Host "  $_" } }
$passed = 0
$test.Lines | Select-String -Pattern "test result: ok\. (\d+) passed" | ForEach-Object { $passed += [int]$_.Matches[0].Groups[1].Value }
Write-Host "=== [SUCCESS] ALL CHECKS PASSED: $passed tests ===" -ForegroundColor Green
exit 0
