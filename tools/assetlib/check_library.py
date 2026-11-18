#!/usr/bin/env python3
"""ASSETLIB1 check: the kit asset library (`kits/<kit>/library/<style>/<kind>/`) and its `.meta.json` sidecars.

Usage: python3 tools/assetlib/check_library.py [--root <repo root>]
Exit 0 when every library is clean (an absent library passes), 1 on a problem, each printed as `[assetlib] <path>: ...`.
Plain Python, no Pillow: a PNG's size is read from its header. The lore words are every game's world ids, its id and the
lines of `games/<id>/lore_words.txt` (the list the layers check G3 uses), matched whole word, any case, `_` a word break
and a space matching a space or `_` (spec docs/specs/ASSETLIB1.md AL1-AL7; the kit never reads a game, this tool does).
"""

from __future__ import annotations

import argparse
import json
import re
import struct
import sys
from pathlib import Path

KINDS = ("tilesets", "props", "skill_icons", "gear_icons", "status_icons", "ui", "units", "characters", "portraits",
         "backdrops", "cgs", "audio")
STATUSES = ("placeholder", "final")
SOURCES = ("gemini_web", "flow", "hand", "generated")
REQUIRED = ("id", "kind", "style", "size", "palette", "tags", "status", "source", "request", "license")
OPTIONAL = ("frames",)
STYLE_FIELDS = ("style", "label", "palette", "notes")
SNAKE = re.compile(r"^[a-z][a-z0-9_]*$")
META = ".meta.json"
STYLE_META = "style.meta.json"
# Kinds whose files are not pictures: no pixel size to compare (AL5 `size` is pixels; a sound has none, so null).
NO_PIXELS = ("audio",)
ASSET_EXTS = (".png", ".ogg", ".wav", ".mp3")


def png_size(path: Path) -> tuple[int, int] | None:
    """The width and height in a PNG's IHDR, or None when the file is no PNG."""
    with path.open("rb") as f:
        head = f.read(24)
    if len(head) < 24 or head[:8] != b"\x89PNG\r\n\x1a\n" or head[12:16] != b"IHDR":
        return None
    return struct.unpack(">II", head[16:24])


def lore_pattern(root: Path) -> re.Pattern[str] | None:
    """One pattern of every game's id, world ids and lore words, or None when there are none."""
    words: list[str] = []
    for game in sorted((root / "games").glob("*/")):
        words.append(game.name)
        words.extend(w.name for w in sorted((game / "data" / "worlds").glob("*/")))
        lore = game / "lore_words.txt"
        if lore.is_file():
            for line in lore.read_text(encoding="utf-8").splitlines():
                line = line.split("#", 1)[0].strip()
                if line:
                    words.append(line)
    if not words:
        return None
    alts = sorted({r"[\s_]+".join(re.escape(p) for p in w.split()) for w in words}, key=len, reverse=True)
    return re.compile(r"(?:^|[^0-9a-z])(" + "|".join(alts) + r")(?:[^0-9a-z]|$)", re.IGNORECASE)


def lore_hit(pattern: re.Pattern[str] | None, text: str) -> str | None:
    if pattern is None:
        return None
    m = pattern.search(text)
    return m.group(1) if m else None


def check_style(path: Path, style: str, lore: re.Pattern[str] | None) -> list[str]:
    out = []
    try:
        meta = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        return [f"{path}: not JSON: {e}"]
    if not isinstance(meta, dict):
        return [f"{path}: not an object"]
    for k in STYLE_FIELDS:
        if k not in meta:
            out.append(f"{path}: missing '{k}'")
    for k in sorted(set(meta) - set(STYLE_FIELDS)):
        out.append(f"{path}: unknown key '{k}'")
    if meta.get("style") != style:
        out.append(f"{path}: style '{meta.get('style')}' is not its folder '{style}'")
    for k in ("style", "label"):
        hit = lore_hit(lore, str(meta.get(k, "")))
        if hit:
            out.append(f"{path}: {k} names the game word '{hit}'")
    return out


def check_meta(meta_path: Path, asset: Path | None, style: str, kind: str, lore: re.Pattern[str] | None) -> tuple[list[str], str | None]:
    """The problems of one sidecar and the id it gives (None when unreadable)."""
    out: list[str] = []
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        return [f"{meta_path}: not JSON: {e}"], None
    if not isinstance(meta, dict):
        return [f"{meta_path}: not an object"], None
    for k in REQUIRED:
        if k not in meta:
            out.append(f"{meta_path}: missing '{k}'")
    for k in sorted(set(meta) - set(REQUIRED) - set(OPTIONAL)):
        out.append(f"{meta_path}: unknown key '{k}'")
    mid = meta.get("id")
    if not isinstance(mid, str) or not SNAKE.match(mid):
        out.append(f"{meta_path}: id '{mid}' is not snake_case")
        mid = None
    if meta.get("kind") not in KINDS:
        out.append(f"{meta_path}: kind '{meta.get('kind')}' is not one of {', '.join(KINDS)}")
    elif meta["kind"] != kind:
        out.append(f"{meta_path}: kind '{meta['kind']}' is not its folder '{kind}'")
    if meta.get("style") != style:
        out.append(f"{meta_path}: style '{meta.get('style')}' is not its folder '{style}'")
    if "status" in meta and meta["status"] not in STATUSES:
        out.append(f"{meta_path}: status '{meta['status']}' is not placeholder or final")
    if "source" in meta and meta["source"] not in SOURCES:
        out.append(f"{meta_path}: source '{meta['source']}' is not one of {', '.join(SOURCES)}")
    if "palette" in meta and meta["palette"] is not None and not isinstance(meta["palette"], str):
        out.append(f"{meta_path}: palette is a palette id or null")
    if "request" in meta and meta["request"] is not None and not isinstance(meta["request"], str):
        out.append(f"{meta_path}: request is a request id or null")
    if "license" in meta and not (isinstance(meta["license"], str) and meta["license"].strip()):
        out.append(f"{meta_path}: license is the licence or provenance line")
    tags = meta.get("tags")
    if "tags" in meta and not (isinstance(tags, list) and all(isinstance(t, str) and t for t in tags)):
        out.append(f"{meta_path}: tags is a list of words")
        tags = []
    if "frames" in meta and not (isinstance(meta["frames"], list) and len(meta["frames"]) == 2 and all(isinstance(n, int) and n > 0 for n in meta["frames"])):
        out.append(f"{meta_path}: frames is [cols, rows]")
    size = meta.get("size")
    if kind in NO_PIXELS:
        if "size" in meta and size is not None:
            out.append(f"{meta_path}: a sound has no pixel size (size null)")
    elif "size" in meta:
        if not (isinstance(size, list) and len(size) == 2 and all(isinstance(n, int) and n > 0 for n in size)):
            out.append(f"{meta_path}: size is [w, h] in pixels")
        elif asset is not None and asset.suffix == ".png":
            real = png_size(asset)
            if real is None:
                out.append(f"{asset}: not a PNG")
            elif list(real) != size:
                out.append(f"{meta_path}: size {size} but the PNG is {list(real)}")
    names = [("file name", meta_path.name[: -len(META)])] + ([("id", mid)] if mid else []) + [("tag", t) for t in (tags or [])]
    for what, text in names:
        hit = lore_hit(lore, text)
        if hit:
            out.append(f"{meta_path}: {what} '{text}' names the game word '{hit}'")
    return out, mid


def check(root: Path) -> list[str]:
    lore = lore_pattern(root)
    problems: list[str] = []
    ids: dict[str, Path] = {}
    for lib in sorted(root.glob("kits/*/library")):
        if not lib.is_dir():
            continue
        for entry in sorted(lib.iterdir()):
            if entry.name.lower() == "readme.md":
                continue
            if not entry.is_dir():
                problems.append(f"{entry}: the library holds style folders only")
                continue
            style = entry.name
            if not SNAKE.match(style):
                problems.append(f"{entry}: style '{style}' is not snake_case")
            if (entry / STYLE_META).is_file():
                problems.extend(check_style(entry / STYLE_META, style, lore))
            else:
                problems.append(f"{entry}: no {STYLE_META}")
            hit = lore_hit(lore, style)
            if hit:
                problems.append(f"{entry}: style '{style}' names the game word '{hit}'")
            for kdir in sorted(entry.iterdir()):
                if kdir.name == STYLE_META:
                    continue
                if not kdir.is_dir() or kdir.name not in KINDS:
                    problems.append(f"{kdir}: not a kind folder ({', '.join(KINDS)})")
                    continue
                files = sorted(p for p in kdir.rglob("*") if p.is_file())
                metas = {p for p in files if p.name.endswith(META)}
                assets = [p for p in files if p not in metas]
                for a in assets:
                    if a.suffix.lower() not in ASSET_EXTS:
                        problems.append(f"{a}: not an asset file ({', '.join(ASSET_EXTS)})")
                        continue
                    m = a.with_name(a.stem + META)
                    if m not in metas:
                        problems.append(f"{a}: no sidecar {m.name}")
                        continue
                    metas.discard(m)
                    found, mid = check_meta(m, a, style, kdir.name, lore)
                    problems.extend(found)
                    if mid:
                        if mid in ids:
                            problems.append(f"{m}: id '{mid}' is also {ids[mid]}")
                        else:
                            ids[mid] = m
                for m in sorted(metas):
                    problems.append(f"{m}: a sidecar with no asset beside it")
    return problems


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--root", default=str(Path(__file__).resolve().parents[2]))
    a = p.parse_args(argv)
    root = Path(a.root)
    problems = check(root)
    for line in problems:
        print(f"[assetlib] {line}")
    count = sum(1 for _ in root.glob("kits/*/library/*/*/**/*.meta.json"))
    print(f"[assetlib] {'ok' if not problems else 'FAIL'}: {count} asset(s), {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
