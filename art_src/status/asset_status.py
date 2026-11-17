#!/usr/bin/env python3
"""One asset status table for the game (style-lock flow, project review 2026-11-17 step D).

    python art_src/status/asset_status.py [--out games/emberwake/docs/specs/ASSET_STATUS.md] [--check FILE]

Walks the game's and the kit's asset folders and writes one row per art or audio file: path, category, state
(final / placeholder / missing), how the state was decided, and the act. It replaces the status spread over the inbox,
ART_UI_STUDIO sections 6-8, ASSET_PLAN and the asset READMEs. --check FILE adds the `aegis content check` warnings
(a portrait or palette that falls back) as missing rows.

State rules, in order:
  missing      listed in MISSING below (needed by canon or a spec, no file yet), or a fallback warning from --check
  placeholder  the file was last changed by a commit in PLACEHOLDER_COMMITS (the deterministic and web-queue
               stand-in packs), it is a byte copy of another asset, it is in PROCEDURAL (gen_art.py scenes the
               review called two generations older), or its README row says `PROVISIONAL synth placeholder`
  rework       final art the lock or canon sends back (REWORK)
  final        everything else
"""
import argparse
import collections
import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
LAYERS = {"game": ROOT / "games/emberwake/assets", "kit": ROOT / "kits/rpg/assets"}
EXT = {".png", ".ogg", ".wav", ".gif"}
PLACEHOLDER_COMMITS = {
    "53ae75c": "Phase 4 U2 placeholder icons (AI-14)",
    "7ddae3f": "Wave 2 placeholder pack (AI-15)",
    "7e185b0": "web-queue portrait placeholders (AI-05, AI-06, AI-08)",
    "3ded9b4": "PR5-39 placeholder NPC portraits",
}
PROCEDURAL = [("backgrounds/battle/", "procedural backdrop (gen_art.py)"),
              ("backgrounds/story/intro_", "procedural story card (gen_art.py)"),
              ("backgrounds/story/hearth.png", "procedural story card (gen_art.py)"),
              ("backgrounds/story/meat_market.png", "procedural story card (gen_art.py)")]
# Needed by canon or an approved spec, with no file yet (inbox AI-05/AI-06, project review 2026-11-17 "Art and UI").
MISSING = [
    ("portraits/bosses/foundry_master_jessop.png", "portrait", "Act 2", "AI-06; CANON 3.2"),
    ("portraits/bosses/inquisitor_general_ruthven.png", "portrait", "Act 2", "AI-06; CANON 3.2"),
    ("portraits/bosses/chief_anatomist_lazar.png", "portrait", "Act 2", "AI-06; CANON 3.2"),
    ("portraits/bosses/warden_inquisitor_ulric.png", "portrait", "Act 3", "AI-06; CANON 3.2"),
    ("portraits/bosses/cantor_ysolde.png", "portrait", "Act 3", "AI-06; CANON 3.2"),
    ("portraits/bosses/arch_prelate_othemar.png", "portrait", "Act 3", "AI-06; CANON 3.2"),
    ("portraits/npcs/faces/sabina_kress/*.png", "face sheet", "Act 1", "AI-05"),
    ("portraits/npcs/faces/talia_rusk/*.png", "face sheet", "Act 1", "AI-05"),
    ("portraits/npcs/faces/berek_dorn/*.png", "face sheet", "Act 1", "AI-05"),
]
# Final art that the style lock or canon sends back (state `rework`).
REWORK = {f"portraits/heroes/{n}.png": "AI-04: off canon (Day 1 must be clean); re-render in the locked style"
          for n in ("kaelen", "vesper", "maren", "hollis", "dunstan", "elyse")}
ACT1 = ("sunk_ward", "ashlands", "undercroft", "tallow")


def category(rel):
    top = rel.split("/")[0]
    if top == "portraits":
        return "portrait" if "/faces/" not in rel else "face sheet"
    if top == "units":
        return "sprite sheet"
    if top in ("skills", "statuses", "items"):
        return {"skills": "skill icon", "statuses": "status icon", "items": "item icon"}[top]
    if top == "audio":
        return "music" if "/music/" in rel else "sfx"
    if top in ("tilesets", "props"):
        return "tile" if top == "tilesets" else "prop"
    return top.rstrip("s") if top in ("backgrounds",) else top


def act(rel):
    return "Act 1" if any(k in rel for k in ACT1) else "all"


def red_tile(f):
    """An icon painted full-bleed on the old dark-red tile (the I1 style)."""
    from PIL import Image
    im = Image.open(f).convert("RGBA")
    w, h = im.size
    pts = [im.getpixel((x, y)) for x, y in ((w // 10, h // 2), (w // 2, h // 10), (w - 1 - w // 10, h // 2), (w // 2, h - 1 - h // 10))]
    return sum(1 for p in pts if p[3] > 200 and p[0] > p[2] + 25 and p[0] > p[1] + 15) >= 3


def last_commits(paths):
    """path -> short sha of the last commit that changed it (one git log for the whole tree)."""
    out = subprocess.run(["git", "log", "--format=@%h", "--name-only", "--", "games/emberwake/assets", "kits/rpg/assets"],
                         cwd=ROOT, capture_output=True, text=True, check=True).stdout
    seen, sha = {}, None
    for line in out.splitlines():
        if line.startswith("@"):
            sha = line[1:8]
        elif line and line not in seen:
            seen[line] = sha
    return seen


def synth_placeholders():
    out = set()
    for layer, root in LAYERS.items():
        readme = root / "audio/music/README.md"
        if readme.exists():
            for m in re.finditer(r"^\| `([a-z0-9_]+)` .*PROVISIONAL synth placeholder", readme.read_text(encoding="utf-8"), re.M):
                out.add(f"audio/music/{m.group(1)}.ogg")
    return out


def rows(check_file=None):
    files = []
    for layer, root in LAYERS.items():
        for f in sorted(root.rglob("*")):
            if f.suffix in EXT and f.is_file():
                files.append((layer, f.relative_to(root).as_posix(), f))
    commits = last_commits(files)
    digest = collections.defaultdict(list)
    for layer, rel, f in files:
        digest[hashlib.sha1(f.read_bytes()).hexdigest()].append(f"{layer}:{rel}")
    synth = synth_placeholders()
    out = []
    for layer, rel, f in files:
        repo_rel = f.relative_to(ROOT).as_posix()
        sha = commits.get(repo_rel, "")
        state, why = "final", ""
        stem = rel.rsplit("/", 1)[-1][:-4]
        twins = [t for t in digest[hashlib.sha1(f.read_bytes()).hexdigest()] if t != f"{layer}:{rel}"
                 and f"/faces/{stem}/" not in t and "/full_body/" not in t and "/faces/" not in rel]   # a neutral face is the portrait itself
        if rel in REWORK:
            state, why = "rework", REWORK[rel]
        elif rel.startswith("units/") and rel.endswith("sheet.png"):
            state, why = "rework", "S2 lock: grim slot colours with a 3-tone ramp per slot (STYLE_BIBLE 3)"
        elif category(rel) in ("skill icon", "item icon") and red_tile(f):
            state, why = "rework", "I2 lock: painted-on-red tile, move the glyph to the slate tile (STYLE_BIBLE 4)"
        elif rel.startswith("skills/kinds/"):
            state, why = "rework", "I2 lock: flat glyph, move to the slate tile (STYLE_BIBLE 4)"
        elif sha in PLACEHOLDER_COMMITS:
            state, why = "placeholder", PLACEHOLDER_COMMITS[sha]
        elif any(rel.startswith(p) for p, _ in PROCEDURAL):
            state, why = "placeholder", next(w for p, w in PROCEDURAL if rel.startswith(p))
        elif rel in synth:
            state, why = "placeholder", "synth music placeholder"
        elif twins and category(rel) in ("portrait", "skill icon", "item icon", "status icon"):
            state, why = "placeholder", "byte copy of " + twins[0]
        out.append(dict(path=f"{layer}:{rel}", category=category(rel), state=state, why=why, source=sha, act=act(rel)))
    for rel, cat, a, why in MISSING:
        if not list(LAYERS["game"].glob(rel)):
            out.append(dict(path="game:" + rel, category=cat, state="missing", why=why, source="", act=a))
    if check_file:
        seen = set()
        for line in pathlib.Path(check_file).read_text(encoding="utf-8", errors="replace").splitlines():
            if "(test)" in line or ":tl_" in line:
                continue
            m = re.search(r"has no portrait at (\S+);", line) or re.search(r"no file (props/\S+) for world '(\w+)'", line)
            if not m or m.group(1) in seen:
                continue
            seen.add(m.group(1))
            w = re.match(r"warning: (\w+):", line)
            where = m.group(2) if m.re.groups > 1 else (w.group(1) if w else "")
            legacy = where in ("ashlands", "iron_reaches", "tidewrack")
            out.append(dict(path="game:" + m.group(1), category="prop" if m.group(1).startswith("props/") else "portrait",
                            state="missing", why="content check (falls back)" + (f"; legacy world {where}" if legacy else ""),
                            source="", act="legacy" if legacy else "?"))
    return out


def write(table, out):
    by = collections.Counter((r["category"], r["state"]) for r in table)
    cats = sorted({r["category"] for r in table})
    lines = ["# Asset status", "",
             "Generated by `art_src/status/asset_status.py` (Art/UI; do not edit by hand). One row per art or audio file of the",
             "game and kit layers, plus the assets canon or a spec needs that have no file yet. Re-run after every batch.", "",
             "## Summary", "", "| Category | Final | Rework | Placeholder | Missing |", "| --- | --- | --- | --- | --- |"]
    for c in cats:
        lines.append(f"| {c} | {by[(c, 'final')]} | {by[(c, 'rework')]} | {by[(c, 'placeholder')]} | {by[(c, 'missing')]} |")
    t = collections.Counter(r["state"] for r in table)
    lines += [f"| **all** | **{t['final']}** | **{t['rework']}** | **{t['placeholder']}** | **{t['missing']}** |", ""]
    lines += ["Every row, final files included: [ASSET_STATUS.csv](ASSET_STATUS.csv) (path, category, state, act, why, last",
              "commit). Below: what still needs work, rework and placeholders grouped by folder and reason.", ""]
    import csv, io
    buf = io.StringIO(); wr = csv.writer(buf, lineterminator="\n")
    wr.writerow(["path", "category", "state", "act", "why", "last_commit"])
    for r in table:
        wr.writerow([r["path"], r["category"], r["state"], r["act"], r["why"], r["source"]])
    pathlib.Path(out).with_suffix(".csv").write_text(buf.getvalue(), encoding="utf-8")
    sel = [r for r in table if r["state"] == "missing"]
    lines += [f"## Missing ({len(sel)})", "", "| Path | Category | Act | Why |", "| --- | --- | --- | --- |"]
    lines += [f"| `{r['path']}` | {r['category']} | {r['act']} | {r['why']} |" for r in sel] + [""]
    for state in ("rework", "placeholder"):
        sel = [r for r in table if r["state"] == state]
        grp = collections.Counter((r["path"].rsplit("/", 1)[0] + "/", r["category"], r["why"]) for r in sel)
        lines += [f"## {state.capitalize()} ({len(sel)})", "", "| Folder | Category | Files | Why |", "| --- | --- | --- | --- |"]
        lines += [f"| `{f}` | {c} | {n} | {w} |" for (f, c, w), n in sorted(grp.items())] + [""]
    pathlib.Path(out).write_text("\n".join(lines).rstrip("\n") + "\n", encoding="utf-8")
    return t


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "games/emberwake/docs/specs/ASSET_STATUS.md"))
    ap.add_argument("--check")
    ap.add_argument("--json")
    a = ap.parse_args()
    table = rows(a.check)
    print(dict(write(table, a.out)))
    if a.json:
        pathlib.Path(a.json).write_text(json.dumps(table, indent=1), encoding="utf-8")
