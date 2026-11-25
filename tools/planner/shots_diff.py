#!/usr/bin/env python3
"""SHOT-DIFF1 (PR7-72): the Python half of `tools/fc.sh shots diff [base=<sha>] [scenarios=a,b] [out=<dir>]`.

`tools/fc.sh` picks the scenarios (`select`), runs them on HEAD and, on a cache miss, on the base commit in a scratch
worktree (sharing the target dir, so only the workspace crates build again), and adds each run's shots to a flat folder
(`add`: the base's go to `target/shots_cache/<sha>/`, kept for the next diff against that commit). `report` then
compares the base shots with HEAD's (`shots_all.compare`, frame-exact by default: shots run on the shot clock, so two
builds of one commit match byte for byte) and writes into the out folder:

  index.md     the base, the counts and every changed, new and gone shot (changed: pixel counts, most first)
  sheet.png    the contact sheet: before | after | diff of each changed shot, half size, most changed first (at most
               `limit`, default 12; the rest are listed in index.md only)
  compare/     the full-size before | after | diff sheet of each changed shot
  head/        HEAD's shots (flat names, as `shots-all`)

  shots_diff.py select                        changed paths on stdin -> "none", "all" or the scenario ids (a,b)
  shots_diff.py add <bundles> <flat>          copy a run's shots into a flat folder, keeping the shots already there
  shots_diff.py report <base flat> <head flat> <out> [base=<sha>] [tolerance=0] [pixels=0] [limit=12]
  shots_diff.py none <out> [base=<sha>]       the index of a diff no shot can show (a docs-only change)

Plain Python (no Pillow), like `shots_all.py`.
"""
import shutil
import sys
from pathlib import Path

import shots_all as sa

# Paths no scenario shot reads: docs and notes, the planner's own scripts, the frozen Godot build, the agents' files.
QUIET_PREFIXES = ("docs/", ".claude/", ".agents/", "legacy/", "tools/planner/", "tools/mock_ui/")
SCENARIO_DIR = "/scenarios/"
GAP = 8
LIMIT = 12


def quiet(path):
    """Whether a change to `path` can leave every shot as it was."""
    return path.endswith(".md") or path.startswith(QUIET_PREFIXES)


def select(paths):
    """"none" (no shot can change), the ids of the changed scenario files when nothing else changed, else "all"."""
    loud = [p.strip() for p in paths if p.strip() and not quiet(p.strip())]
    if not loud:
        return "none"
    ids = []
    for p in loud:
        if SCENARIO_DIR not in p or not p.endswith(".json") or not p.startswith("games/"):
            return "all"
        name = Path(p).stem
        if name not in ids:
            ids.append(name)
    return ",".join(ids)


def add(bundles, flat):
    """Copies every shot under `bundles` into `flat` (flat names), keeping what is there; returns the names."""
    flat = Path(flat)
    flat.mkdir(parents=True, exist_ok=True)
    found = sa.shots(bundles)
    for name, p in found.items():
        shutil.copyfile(p, flat / name)
    return sorted(found)


def half(path):
    """(width, RGB rows) of a PNG at half size (every other pixel of every other row)."""
    w, bpp, rows = sa.png_rows(path)
    out = []
    for r in rows[::2]:
        line = sa.rgb(bpp, r)
        out.append(b"".join(line[i:i + 3] for i in range(0, len(line), 6)))
    return (w + 1) // 2, out


def contact(sheets, out):
    """One PNG of the `sheets` stacked at half size, a grey gap between; False when there is none."""
    parts = [half(p) for p in sheets]
    if not parts:
        return False
    width = max(w for w, _ in parts)
    gap = [bytes((40, 40, 40)) * width] * GAP
    rows = []
    for i, (w, rs) in enumerate(parts):
        if i:
            rows += gap
        rows += [r + bytes((40, 40, 40)) * (width - w) for r in rs]
    sa.write_png(out, width, rows)
    return True


def index(base, changed, new, gone, same, tolerance, pixels, shown):
    lines = ["# Shots diff", "", "- Base: `%s`" % (base or "?"),
             "- %d changed, %d new, %d gone, %d unchanged (a pixel past %d per channel; a shot past %d pixels)" % (len(changed), len(new), len(gone), same, tolerance, pixels)]
    if changed:
        lines.append("- Contact sheet: `sheet.png` (before | after | diff, the first %d changed)" % shown)
        lines += ["", "## Changed", ""] + ["- `%s` (%s)" % (n, "new size" if k is None else "%d px" % k) for n, k in changed]
    if new:
        lines += ["", "## New", ""] + ["- `%s`" % n for n in new]
    if gone:
        lines += ["", "## Gone", ""] + ["- `%s`" % n for n in gone]
    return "\n".join(lines) + "\n"


def report(base_flat, head_flat, out, base="", tolerance=0, pixels=0, limit=LIMIT):
    """Compares the base shots with HEAD's into `out` (see the module); returns (changed, new, gone, unchanged)."""
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    head = out / "head"
    if Path(head_flat).resolve() != head.resolve():
        if head.exists():
            shutil.rmtree(head)
        shutil.copytree(head_flat, head)
    found = sa.compare(base_flat, head, tolerance, pixels, None, True)
    changed, new, gone, same = found
    if (out / "compare").exists():
        shutil.rmtree(out / "compare")
    if (head / "compare").exists():
        shutil.move(str(head / "compare"), str(out / "compare"))
    if (out / "sheet.png").exists():
        (out / "sheet.png").unlink()
    sheets = [out / "compare" / n for n, k in changed if k is not None and (out / "compare" / n).exists()][:limit]
    contact(sheets, out / "sheet.png")
    (out / "index.md").write_text(index(base, changed, new, gone, same, tolerance, pixels, len(sheets)), encoding="utf-8")
    return found


def none(out, base=""):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    text = index(base, [], [], [], 0, 0, 0, 0).replace("- 0 changed", "- No scenario ran: the change touches no file a shot reads (docs only). 0 changed", 1)
    (out / "index.md").write_text(text, encoding="utf-8")


def main(argv):
    opts = dict(a.split("=", 1) for a in argv if "=" in a)
    args = [a for a in argv if "=" not in a]
    base = opts.get("base", "")
    if args[:1] == ["select"]:
        print(select(sys.stdin.read().splitlines()))
        return 0
    if len(args) == 3 and args[0] == "add":
        names = add(args[1], args[2])
        print("[shots diff] %d shot(s) into %s" % (len(names), args[2]))
        return 0
    if len(args) == 4 and args[0] == "report":
        tol, px = int(opts.get("tolerance", 0)), int(opts.get("pixels", 0))
        changed, new, gone, same = report(args[1], args[2], args[3], base, tol, px, int(opts.get("limit", LIMIT)))
        print("[shots diff] %d changed, %d new, %d gone, %d unchanged against %s; index %s"
              % (len(changed), len(new), len(gone), same, base[:10] or "the base", Path(args[3]) / "index.md"))
        for n, k in changed:
            print("  changed %s (%s)" % (n, "new size" if k is None else "%d px" % k))
        if changed:
            print("  contact sheet: %s" % (Path(args[3]) / "sheet.png"))
        return 0
    if len(args) == 2 and args[0] == "none":
        none(args[1], base)
        print("[shots diff] 0 changed: the change touches no file a shot reads (docs only); index %s" % (Path(args[1]) / "index.md"))
        return 0
    print(__doc__.split("\n\n")[-2])
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
