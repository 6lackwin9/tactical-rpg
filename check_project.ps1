# ==============================================================================
# Tactical RPG - commit gate on the Aegis engine (transition step 2, 2026-10-19)
# Builds every crate and target (a compiler warning fails), then runs every test: the rules, battle, world, data-loader and content-writer
# suites and the engine goldens, then the layer check (engine and kit crates name no game, KIT1-G; engine and kit code name no lore word, TERMS1 G3; kit code no class id, G4), the whole docs check and the tools/tiles tests (both run by tools/fc.sh, so one copy of each). Same steps as `tools/fc.sh gate`. The Godot gate of the frozen Godot copy is legacy/check_project.ps1.
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

Write-Host "=== AEGIS: COMMIT GATE ===" -ForegroundColor Cyan

Write-Host "[1/5] Building every crate..."
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

Write-Host "[2/5] Running every test..."
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
$summary = $test.Lines | Where-Object { $_ -match "checked|diverge|compared|^test result: FAILED|panicked|os error" }
$failed = ($test.Code -ne 0) -or ($test.Lines -match "os error 4551")
if ($failed) {
    $test.Lines | Where-Object { $_ -match "FAILED|panicked|^error|os error|differ|diverge" } | Select-Object -First 60 | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    Write-Host "=== [FAILED] tests ===" -ForegroundColor Red
    exit 1
}
if (-not $Quiet) { $summary | ForEach-Object { Write-Host "  $_" } }

Write-Host "[3/5] Checking the layers..."
# KIT1-G (docs/specs/KIT1_TASKS.md 4), same check as `tools/fc.sh layers`: the engine and kit crates name no game (every
# folder under games/, whole word with `_` as a separator, any case; tracked or new files) and no engine or kit manifest
# has a path dependency into games/ (an engine manifest none into kits/ either).
$layerHits = @()
$gameIds = @(Get-ChildItem -Path (Join-Path $projectRoot "games") -Directory -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
foreach ($id in $gameIds) {
    $pattern = "(^|[^[:alnum:]])$([regex]::Replace($id, '[^A-Za-z0-9_-]', '\$0'))([^[:alnum:]]|$)"
    $hits = & git grep --untracked -n -I -i -E -e $pattern -- engine/crates 'kits/*/crates/*' 'kits/*/schema.json'
    foreach ($h in @($hits)) {
        if ($h -match '^([^:]*):(\d+):(.*)$') {
            $text = $Matches[3]; $at = [Math]::Max(0, $text.ToLower().IndexOf($id.ToLower()) - 40)
            $layerHits += "$($Matches[1]):$($Matches[2]) names game `"$id`": $($text.Substring($at, [Math]::Min(120, $text.Length - $at)))"
        }
    }
}
$manifests = @(& git ls-files --cached --others --exclude-standard -- 'engine/*Cargo.toml' 'kits/*Cargo.toml')
$gamesRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot "games")) + [IO.Path]::DirectorySeparatorChar
$kitsRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot "kits")) + [IO.Path]::DirectorySeparatorChar
foreach ($m in $manifests) {
    $dir = Split-Path -Parent (Join-Path $projectRoot $m)
    $n = 0
    foreach ($line in [IO.File]::ReadAllLines((Join-Path $projectRoot $m))) {
        $n++
        if ($line -notmatch 'path *= *"([^"]*)"') { continue }
        $p = $Matches[1]
        $full = [IO.Path]::GetFullPath($(if ([IO.Path]::IsPathRooted($p)) { $p } else { Join-Path $dir $p })) + [IO.Path]::DirectorySeparatorChar
        if ($full.StartsWith($gamesRoot, [StringComparison]::OrdinalIgnoreCase)) { $layerHits += "${m}:$n path dependency into games/: $p" }
        elseif ($m -like 'engine/*' -and $full.StartsWith($kitsRoot, [StringComparison]::OrdinalIgnoreCase)) { $layerHits += "${m}:$n engine path dependency into kits/: $p" }
    }
}
# TERMS1 G3 (docs/specs/TERMS1.md 7, LAYER-DOWN-1), same as `tools/fc.sh layers`: engine and kit code (engine and kit
# crates: src, tests and fixtures; kit schema.json and kit.json; not READMEs or docs) names no lore word of a game: its
# world ids (folders under games/<id>/data/worlds/) and the lines of games/<id>/lore_words.txt (fixed phrases, a space
# matches a space or `_`, `#` comments); any case, whole word with `_` as a separator. One git grep per game.
foreach ($id in $gameIds) {
    $gameDir = Join-Path (Join-Path $projectRoot "games") $id
    $words = @(Get-ChildItem -Path (Join-Path $gameDir "data/worlds") -Directory -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
    $loreFile = Join-Path $gameDir "lore_words.txt"
    if (Test-Path $loreFile) { $words += @(Get-Content $loreFile | ForEach-Object { ($_ -replace '#.*$', '').Trim() } | Where-Object { $_ }) }
    $lp = ($words | ForEach-Object { [regex]::Replace($_, '[\]\[\\.*^$+?(){}|/]', '\$0') -replace '\s+', '[[:space:]_]' }) -join '|'
    if (-not $lp) { continue }
    $hits = & git grep --untracked -n -I -i -E -e "(^|[^[:alnum:]])($lp)([^[:alnum:]]|$)" -- engine/crates 'kits/*/crates/*' 'kits/*/schema.json' 'kits/*/kit.json'
    $netLp = $lp.Replace('[[:space:]_]', '[\s_]')
    foreach ($h in @($hits)) {
        if ($h -match '^([^:]*):(\d+):(.*)$') {
            $file = $Matches[1]; $line = $Matches[2]; $text = $Matches[3]
            $m = [regex]::Match($text, "(?<![A-Za-z0-9])($netLp)(?![A-Za-z0-9])", 'IgnoreCase')
            $word = if ($m.Success) { $m.Groups[1].Value } else { "?" }
            $at = if ($m.Success) { [Math]::Max(0, $m.Index - 40) } else { 0 }
            $layerHits += "${file}:$line names lore word `"$word`" of game ${id}: $($text.Substring($at, [Math]::Min(120, $text.Length - $at)))"
        }
    }
}
# G4 (KITIDS1, SILOS1 T4), same as `tools/fc.sh layers`: kit code assumes no class of a game: the entry ids of
# games/<id>/data/catalog/classes.json (the writer's form: one `"id"` line per entry, three tabs in) as a whole string
# literal ("<id>"), exact case; kit src only. git grep finds the bare ids (a pattern with a quote does not survive the
# native argument passing of Windows PowerShell), then a .NET regex keeps the quoted literals.
foreach ($id in $gameIds) {
    $cfRel = "games/$id/data/catalog/classes.json"
    $cf = Join-Path $projectRoot $cfRel
    if (-not (Test-Path $cf)) { continue }
    $classIds = @(Get-Content $cf | ForEach-Object { if ($_ -match '^\t\t\t"id": "([^"]*)",?$') { $Matches[1] } })
    if ($classIds.Count -eq 0) {
        $layerHits += "${cfRel}: no class id found (expected the writer's form: one `"id`" line per entry, three tabs in)"
        continue
    }
    $cs = ($classIds | ForEach-Object { [regex]::Replace($_, '[\]\[\\.*^$+?(){}|/]', '\$0') }) -join '|'
    $hits = & git grep --untracked -n -I -E -e "($cs)" -- 'kits/*/crates/*/src/*'
    foreach ($h in @($hits)) {
        if ($h -match '^([^:]*):(\d+):(.*)$') {
            $file = $Matches[1]; $line = $Matches[2]; $text = $Matches[3]
            $m = [regex]::Match($text, "`"($cs)`"")
            if (-not $m.Success) { continue }
            $at = [Math]::Max(0, $m.Index - 40)
            $layerHits += "${file}:$line names class id $($m.Value) of game ${id}: $($text.Substring($at, [Math]::Min(120, $text.Length - $at)))"
        }
    }
}
if ($layerHits.Count -gt 0) {
    $layerHits | Select-Object -First 40 | ForEach-Object { Write-Host "  [layers] $_" -ForegroundColor Red }
    Write-Host "=== [FAILED] layers: engine and kit code must not name a game or a lore word, kit code no class id (CLAUDE.md Layers) ===" -ForegroundColor Red
    exit 1
}
if (-not $Quiet) { Write-Host "  -> layers ok: engine and kit crates name no game ($($gameIds -join ', ')); engine and kit code name no lore word, kit code no class id" -ForegroundColor Green }

$script:bash = Find-Bash
if (-not $script:bash) {
    Write-Host "=== [FAILED] no bash found next to git: the docs check and the tool tests need it ===" -ForegroundColor Red
    exit 1
}

Write-Host "[4/5] Checking every doc..."
# Same check as `tools/fc.sh docs --all`; the size warnings as one count unless -VerboseOutput.
$docs = Invoke-Fc @("docs", "--all")
$docWarnings = @($docs.Lines | Where-Object { $_ -match '^\[docs\] WARN' })
if ($VerboseOutput) { $docWarnings | ForEach-Object { Write-Host $_ } }
if ($docs.Code -ne 0) {
    $docs.Lines | Where-Object { $_ -notmatch '^\[docs\] WARN' } | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "=== [FAILED] docs ===" -ForegroundColor Red
    exit 1
}
if (-not $Quiet) {
    $docs.Lines | Where-Object { $_ -notmatch '^\[docs\] WARN' } | ForEach-Object { Write-Host "  $_" }
    if ($docWarnings.Count -gt 0) { Write-Host "  [docs] $($docWarnings.Count) size warning(s)" }
}

Write-Host "[5/5] Running the tool tests..."
# Same step as `tools/fc.sh tooltests`: it holds the time limit and kills the whole process tree on a timeout.
$tools = Invoke-Fc @("tooltests")
if ($tools.Code -ne 0) {
    $tools.Lines | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "=== [FAILED] tooltests ===" -ForegroundColor Red
    exit 1
}
# A SKIP always shows: it means these tests did not run.
if (-not $Quiet -or ($tools.Lines -match 'SKIP')) { $tools.Lines | ForEach-Object { Write-Host "  $_" -ForegroundColor $(if ($_ -match 'SKIP') { 'Yellow' } else { 'Green' }) } }

$passed = 0
$test.Lines | Select-String -Pattern "test result: ok\. (\d+) passed" | ForEach-Object { $passed += [int]$_.Matches[0].Groups[1].Value }
Write-Host "=== [SUCCESS] ALL CHECKS PASSED: $passed tests ===" -ForegroundColor Green
exit 0
