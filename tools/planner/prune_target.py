#!/usr/bin/env python3
"""TARGET-PRUNE1 (ENGINE-39): `tools/fc.sh prune [target=<dir>] [--dry]`.

Keeps the cargo target folder small: in each `<target>/<profile>/deps` it keeps only the newest test or bin
executable per name (cargo names them `<name>-<16 hex>`; a rebuild leaves the old ones behind) and removes the older
ones, then removes each `<target>/<profile>/incremental` folder (the gate builds with CARGO_INCREMENTAL=0, so they are
stale). Libraries (`.rlib`, `.rmeta`, `.so`, `.dll`, `.dylib`, `.d`) and everything else stay. `--dry` lists what it
would remove and the bytes, removing nothing. `tools/fc.sh gate` runs it first when the disk has little room left.
"""
import os
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HASHED = re.compile(r"^(?P<name>.+)-(?P<hash>[0-9a-f]{16})(?P<ext>\.exe)?$")
KEEP_EXT = {".rlib", ".rmeta", ".so", ".dll", ".dylib", ".d", ".pdb", ".lib", ".a", ".o"}


def size_of(p):
    if p.is_dir():
        return sum(f.stat().st_size for f in p.rglob("*") if f.is_file())
    return p.stat().st_size


def stale(target):
    """(path, bytes) of every file and folder `prune` removes under `target`, in a stable order."""
    out = []
    if not target.is_dir():
        return out
    for profile in sorted(p for p in target.iterdir() if p.is_dir()):
        deps = profile / "deps"
        if deps.is_dir():
            groups = {}
            for f in deps.iterdir():
                if not f.is_file() or f.suffix in KEEP_EXT:
                    continue
                m = HASHED.match(f.name)
                if not m or not (f.suffix == ".exe" or os.access(f, os.X_OK)):
                    continue
                groups.setdefault(m.group("name"), []).append(f)
            for name in sorted(groups):
                files = sorted(groups[name], key=lambda f: (f.stat().st_mtime, f.name), reverse=True)
                out.extend((f, f.stat().st_size) for f in files[1:])
        inc = profile / "incremental"
        if inc.is_dir():
            out.append((inc, size_of(inc)))
    return out


def prune(target, dry=False):
    """Removes (unless `dry`) what `stale` lists; returns (count, bytes)."""
    found = stale(target)
    for p, _ in found:
        if dry:
            continue
        if p.is_dir():
            shutil.rmtree(p, ignore_errors=True)
        else:
            try:
                p.unlink()
            except OSError:
                pass
    return len(found), sum(b for _, b in found)


def main(argv):
    target = Path(os.environ.get("CARGO_TARGET_DIR") or ROOT / "target")
    dry = False
    for a in argv:
        if a.startswith("target="):
            target = Path(a.split("=", 1)[1])
        elif a == "--dry":
            dry = True
        else:
            print(__doc__)
            return 2
    n, b = prune(target, dry)
    print("[prune] %s %d stale file(s) and folder(s), %.1f MB, under %s" % ("would remove" if dry else "removed", n, b / 1e6, target))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
