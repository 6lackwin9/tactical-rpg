# Remove a finished git worktree on Windows (git's own `worktree remove` can fail there) and its build folder.
# Git for Windows can delete the files but not read-only folders (OneDrive, old checkouts), leaving half-deleted
# worktree entries that clog the git UI. This clears the attribute first.
# Every linked worktree builds into its own Rust target folder `<base>-<worktree folder name>` (base = target-dir in
# .cargo/config.toml, the same derivation as tools/fc.sh and check_project.ps1); removing a worktree deletes that folder too.
# Usage (from the main repo, worktrees are ../tactical-rpg-<task> on claude/<task>):
#   powershell -NoProfile -File tools/wt_remove.ps1 ../tactical-rpg-<task> [claude/<task>]
#   powershell -NoProfile -File tools/wt_remove.ps1 -Targets [-WhatIf]
# -Targets deletes every `<base>-*` build folder whose worktree is no longer in `git worktree list` (-WhatIf only lists);
# it never touches `<base>` itself or a folder whose worktree still exists.
param([string]$Path = "", [string]$Branch = "", [switch]$Targets, [switch]$WhatIf)
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

if ($Path -eq "") { Write-Error "Give the worktree path, or -Targets." }
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
