# Remove a finished git worktree on Windows (git's own `worktree remove` can fail there) and its build folder.
# Git for Windows can delete the files but not read-only folders (OneDrive, old checkouts), leaving half-deleted
# worktree entries that clog the git UI. This clears the attribute first.
# Every linked worktree builds into its own Rust target folder `<base>-<worktree folder name>` (base = target-dir in
# .cargo/config.toml, the same derivation as tools/fc.sh and check_project.ps1); removing a worktree deletes that folder too.
# Usage (from the main repo, worktrees are ../tactical-rpg-<task> on claude/<task>):
#   powershell -NoProfile -File tools/wt_remove.ps1 ../tactical-rpg-<task> [claude/<task>]
#   powershell -NoProfile -File tools/wt_remove.ps1 -Targets [-WhatIf]
#   powershell -NoProfile -File tools/wt_remove.ps1 -Trim [-WhatIf]
# -Targets deletes every `<base>-*` build folder whose worktree is no longer in `git worktree list` (-WhatIf only lists);
# it never touches `<base>` itself or a folder whose worktree still exists.
# -Trim prunes superseded hashed `debug/deps/<name>-<16hex>.exe` test binaries and `debug/incremental/<crate>-<13char>`
# folders across `<base>` and `<base>-*`, keeping the newest 2 hashes per stem and anything modified in the last 60 minutes,
# while leaving all `.rlib` / `.rmeta` / `build/` files untouched so warm build caches stay warm.
param([string]$Path = "", [string]$Branch = "", [switch]$Targets, [switch]$Trim, [switch]$WhatIf)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot

$base = (Select-String -Path (Join-Path $repo ".cargo\config.toml") -Pattern '^target-dir = "(.*)"').Matches[0].Groups[1].Value
$base = $base.Replace('/', '\').TrimEnd('\')

function Remove-Tree([string]$p) {
	if (-not (Test-Path $p)) { return }
	attrib -R $p /S /D | Out-Null
	Get-ChildItem $p -Recurse -Force -ErrorAction SilentlyContinue | ForEach-Object { $_.Attributes = 'Normal' }
	# A just-finished run can hold a file for a moment: retry a few times before failing.
	for ($try = 0; $try -lt 4 -and (Test-Path $p); $try++) {
		if ($try -gt 0) { Start-Sleep -Seconds 2 }
		Remove-Item -Recurse -Force $p -ErrorAction SilentlyContinue
	}
	if (Test-Path $p) { [System.IO.Directory]::Delete($p, $true) }
}

if ($Targets) {
	$live = @(git -C $repo worktree list --porcelain | Where-Object { $_ -like 'worktree *' } |
		ForEach-Object { Split-Path -Leaf $_.Substring(9) })
	if ($LASTEXITCODE -ne 0 -or $live.Count -eq 0) { Write-Error "git worktree list failed; nothing deleted." }
	$baseLeaf = Split-Path -Leaf $base
	$stale = @(Get-ChildItem (Split-Path -Parent $base) -Directory -Force |
		Where-Object { $_.Name.StartsWith("$baseLeaf-") -and ($live -notcontains $_.Name.Substring($baseLeaf.Length + 1)) })
	if ($stale.Count -eq 0) { Write-Host "No stale build folders."; exit 0 }
	$total = 0.0
	foreach ($d in $stale) {
		$bytes = (Get-ChildItem $d.FullName -Recurse -Force -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
		$gb = [math]::Round([double]$bytes / 1GB, 2)
		$total += $gb
		if ($WhatIf) { Write-Host "would delete $($d.FullName) ($gb GB)" }
		else { Remove-Tree $d.FullName; Write-Host "deleted $($d.FullName) ($gb GB)" }
	}
	Write-Host "$($stale.Count) folder(s), $([math]::Round($total, 2)) GB"
	exit 0
}

if ($Trim) {
	$cutoff = (Get-Date).AddMinutes(-60)
	$baseLeaf = Split-Path -Leaf $base
	$dirs = @(Get-ChildItem (Split-Path -Parent $base) -Directory -Force |
		Where-Object { $_.Name -eq $baseLeaf -or $_.Name.StartsWith("$baseLeaf-") })
	$freedExe = 0.0
	$freedInc = 0.0
	foreach ($t in $dirs) {
		$deps = Join-Path $t.FullName "debug\deps"
		$tExe = 0.0
		if (Test-Path $deps) {
			$exes = Get-ChildItem $deps -Filter "*.exe" -File -ErrorAction SilentlyContinue | Where-Object { $_.BaseName -match '^(.*)-[0-9a-f]{16}$' }
			foreach ($g in ($exes | Group-Object { if ($_.BaseName -match '^(.*)-[0-9a-f]{16}$') { $matches[1] } })) {
				if ($g.Count -gt 2) {
					$old = $g.Group | Sort-Object LastWriteTime -Descending | Select-Object -Skip 2 | Where-Object { $_.LastWriteTime -lt $cutoff }
					foreach ($f in $old) {
						$len = [double]$f.Length
						if (-not $WhatIf) {
							Remove-Item -Force $f.FullName -ErrorAction SilentlyContinue
							$dFile = [System.IO.Path]::ChangeExtension($f.FullName, ".d")
							if (Test-Path $dFile) { Remove-Item -Force $dFile -ErrorAction SilentlyContinue }
						}
						if ($WhatIf -or -not (Test-Path $f.FullName)) { $tExe += $len }
					}
				}
			}
		}
		$inc = Join-Path $t.FullName "debug\incremental"
		$tInc = 0.0
		if (Test-Path $inc) {
			$incDirs = Get-ChildItem $inc -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^(.*)-[0-9a-z]{13}$' }
			foreach ($ig in ($incDirs | Group-Object { if ($_.Name -match '^(.*)-[0-9a-z]{13}$') { $matches[1] } })) {
				if ($ig.Count -gt 2) {
					$oldDirs = $ig.Group | Sort-Object LastWriteTime -Descending | Select-Object -Skip 2 | Where-Object { $_.LastWriteTime -lt $cutoff }
					foreach ($od in $oldDirs) {
						$bytes = [double](Get-ChildItem $od.FullName -Recurse -Force -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
						if (-not $WhatIf) { Remove-Tree $od.FullName }
						if ($WhatIf -or -not (Test-Path $od.FullName)) { $tInc += $bytes }
					}
				}
			}
		}
		$freedExe += $tExe
		$freedInc += $tInc
		$action = if ($WhatIf) { "would trim" } else { "trimmed" }
		Write-Host "$action $($t.Name): .exe $([math]::Round($tExe / 1GB, 2)) GB, incremental $([math]::Round($tInc / 1GB, 2)) GB"
	}
	Write-Host ("Total: {0} GB (.exe {1} GB, incremental {2} GB)" -f [math]::Round(($freedExe + $freedInc) / 1GB, 2), [math]::Round($freedExe / 1GB, 2), [math]::Round($freedInc / 1GB, 2))
	exit 0
}

if ($Path -eq "") { Write-Error "Give the worktree path, -Targets, or -Trim." }
$wt = (Resolve-Path (Join-Path $repo $Path)).Path
if ($null -ne (git -C $wt status --porcelain)) { Write-Error "Worktree $wt has uncommitted changes; commit or discard them first." }
$name = Split-Path -Leaf $wt
$meta = Join-Path $repo ".git\worktrees\$name"
foreach ($p in @($wt, $meta)) { Remove-Tree $p }
git -C $repo worktree prune
$target = "$base-$name"
if (Test-Path $target) { Remove-Tree $target; Write-Host "deleted build folder $target" }
else { Write-Host "no build folder $target" }
if ($Branch -ne "") { git -C $repo branch -d $Branch }
git -C $repo worktree list
