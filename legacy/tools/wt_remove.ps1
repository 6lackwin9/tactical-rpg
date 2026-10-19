# Remove a finished git worktree on Windows/OneDrive (git's own `worktree remove` fails there).
# OneDrive sets the read-only attribute on folders; Git for Windows deletes the files but cannot remove the
# read-only folders, leaving half-deleted worktree entries that clog the git UI. This clears the attribute first.
# Usage (from the main repo): powershell -NoProfile -File tools/wt_remove.ps1 ../ngp-<id> [feat/<branch>]
param([Parameter(Mandatory = $true)][string]$Path, [string]$Branch = "")
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$wt = (Resolve-Path (Join-Path $repo $Path)).Path
if ($null -ne (git -C $wt status --porcelain)) { Write-Error "Worktree $wt has uncommitted changes; commit or discard them first." }
$name = Split-Path -Leaf $wt
$meta = Join-Path $repo ".git\worktrees\$name"
foreach ($p in @($wt, $meta)) {
	if (Test-Path $p) {
		attrib -R $p /S /D | Out-Null
		Get-ChildItem $p -Recurse -Force -ErrorAction SilentlyContinue | ForEach-Object { $_.Attributes = 'Normal' }
		# A just-finished Godot run can hold a file for a moment: retry a few times before failing.
		for ($try = 0; $try -lt 4 -and (Test-Path $p); $try++) {
			if ($try -gt 0) { Start-Sleep -Seconds 2 }
			Remove-Item -Recurse -Force $p -ErrorAction SilentlyContinue
		}
		if (Test-Path $p) { [System.IO.Directory]::Delete($p, $true) }
	}
}
git -C $repo worktree prune
if ($Branch -ne "") { git -C $repo branch -d $Branch }
git -C $repo worktree list
