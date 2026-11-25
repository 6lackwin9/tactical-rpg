#!/usr/bin/env python3
"""Packages a standalone portable Windows (or desktop) release zip for an Aegis game.

Reads the game's `web_bundle.json` as the single source of truth for runtime files (data, assets, kit, engine fonts)
so desktop and web releases never drift, adds the compiled release binary plus `README.txt` and `build.json`, verifies
required runtime entries exist, and writes a clean portable `.zip`.

Usage:
  python3 tools/release/package_windows.py \\
    --binary target/release/emberwake.exe \\
    --bundle games/emberwake/web_bundle.json \\
    --out out/release/emberwake-windows-x64.zip \\
    [--variant release|test] [--commit <sha>]
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

README_TEXT = """Emberwake — Portable Desktop Build
==================================

How to play:
1. Extract this entire zip folder anywhere on your computer (keep emberwake.exe beside the engine/, kits/, and games/ folders).
2. Double-click emberwake.exe to launch.

Controls:
- Move: Arrow keys or W A S D (or click a tile)
- Interact / Confirm: E or Enter
- Back / Menu: Esc or Right-Click
- Screenshot: F12

Saves & Settings:
- Stored automatically in your user AppData folder (not inside this zip folder), so updating to a newer release keeps your saves intact.
"""


def _excluded(rel_posix: str, patterns: list[str]) -> bool:
    name = rel_posix.rsplit("/", 1)[-1]
    for pat in patterns:
        pat_norm = pat.strip("/")
        if rel_posix == pat_norm or rel_posix.startswith(pat_norm + "/"):
            return True
        if fnmatch.fnmatch(name, pat_norm) or fnmatch.fnmatch(rel_posix, pat_norm):
            return True
    return False


def collect_runtime_files(bundle_path: Path) -> tuple[str, Path, list[tuple[Path, str]]]:
    """Returns `(name, root_dir, [(abs_src, archive_rel_posix), ...])` from a bundle manifest."""
    raw = json.loads(bundle_path.read_text(encoding="utf-8"))
    name = raw.get("name") or bundle_path.parent.name
    root = (bundle_path.parent / raw.get("root", ".")).resolve()
    global_excludes = list(raw.get("exclude", []))

    seen: set[str] = set()
    files: list[tuple[Path, str]] = []
    for part in raw.get("parts", []):
        for inc in part.get("include", []):
            src = (root / inc).resolve()
            if not src.exists():
                raise FileNotFoundError(f"bundle include missing: {inc} ({src})")
            if src.is_file():
                rel = src.relative_to(root).as_posix()
                if not _excluded(rel, global_excludes) and rel not in seen:
                    seen.add(rel)
                    files.append((src, rel))
            else:
                for p in sorted(src.rglob("*")):
                    if not p.is_file():
                        continue
                    rel = p.relative_to(root).as_posix()
                    if not _excluded(rel, global_excludes) and rel not in seen:
                        seen.add(rel)
                        files.append((p, rel))
    files.sort(key=lambda item: item[1])
    return name, root, files


def _git_short_sha(cwd: Path) -> str:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=cwd,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
        return out or "unknown"
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def package_release(
    binary: Path,
    bundle_path: Path,
    out_zip: Path,
    variant: str = "release",
    commit: str | None = None,
    target_label: str = "windows-x64",
) -> dict:
    if not binary.is_file():
        raise FileNotFoundError(f"release binary not found: {binary}")
    name, root, runtime_files = collect_runtime_files(bundle_path)
    rels = {rel for _, rel in runtime_files}
    required_prefixes = (f"games/{name}/game.json", "engine/assets/fonts/")
    for req in required_prefixes:
        if req.endswith("/"):
            if not any(r.startswith(req) for r in rels):
                raise ValueError(f"portable bundle missing required folder contents: {req}")
        elif req not in rels:
            raise ValueError(f"portable bundle missing required file: {req}")

    sha = commit or _git_short_sha(root)
    build_meta = json.dumps(
        {"name": name, "variant": variant, "target": target_label, "commit": sha},
        indent=2,
    ) + "\n"

    out_zip.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out_zip, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
        zf.write(binary, binary.name)
        zf.writestr("README.txt", README_TEXT)
        zf.writestr("build.json", build_meta)
        for src, rel in runtime_files:
            zf.write(src, rel)

    return {
        "name": name,
        "binary": binary.name,
        "variant": variant,
        "target": target_label,
        "commit": sha,
        "runtime_files": len(runtime_files),
        "zip_bytes": out_zip.stat().st_size,
        "out": str(out_zip),
    }


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description="Package a standalone portable desktop release zip.")
    p.add_argument("--binary", type=Path, required=True, help="Path to compiled release binary (e.g. emberwake.exe)")
    p.add_argument("--bundle", type=Path, default=ROOT / "games/emberwake/web_bundle.json")
    p.add_argument("--out", type=Path, default=ROOT / "out/release/emberwake-windows-x64.zip")
    p.add_argument("--variant", default="release", choices=("release", "test"))
    p.add_argument("--commit", default=None)
    p.add_argument("--target", default="windows-x64")
    args = p.parse_args(argv)
    info = package_release(args.binary, args.bundle, args.out, args.variant, args.commit, args.target)
    mb = info["zip_bytes"] / 1048576.0
    print(
        f"[release] {info['name']} ({info['variant']}, {info['target']}, @{info['commit']}): "
        f"{info['runtime_files']} runtime files -> {info['out']} ({mb:.2f} MB)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
