#!/usr/bin/env python3
"""TOOLS-HELP1 (ENGINE-15): `tools/fc.sh seed-from <kind> <id> [<world>] --set k=v ... [--out F] [--append]`.

Writes a seed for `aegis content apply` from a current entry with a change: reads the stored entry
(`aegis content get <kind> <id> [world=<world>]`, through the game's CLI), sets each `k=v` (a dotted key reaches
inside: `deck.active=[...]`; the value is JSON when it parses, else a plain string; `k=` with nothing removes the key,
so the writer's default applies) and prints the seed `{"steps": [{"op": "upsert", "kind", ["world",] "entry"}]}`,
or writes it to `--out F` (`--append` adds the step to F's steps). Whole floats the reader gives (`1.0`) are written
as whole numbers. It writes no data: apply the seed with `tools/fc.sh content apply F`.
"""
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def whole(v):
    if isinstance(v, float) and v.is_integer():
        return int(v)
    if isinstance(v, dict):
        return {k: whole(x) for k, x in v.items()}
    if isinstance(v, list):
        return [whole(x) for x in v]
    return v


def parse_value(raw):
    try:
        return json.loads(raw)
    except ValueError:
        return raw


def set_path(entry, key, raw):
    parts = key.split(".")
    node = entry
    for p in parts[:-1]:
        nxt = node.get(p)
        if not isinstance(nxt, dict):
            raise ValueError("%s: %s is not an object in the entry" % (key, p))
        node = nxt
    if raw == "":
        node.pop(parts[-1], None)
    else:
        node[parts[-1]] = parse_value(raw)


def seed_step(kind, world, entry, sets):
    entry = whole(entry)
    for s in sets:
        if "=" not in s:
            raise ValueError("--set takes k=v, not %r" % s)
        k, v = s.split("=", 1)
        set_path(entry, k, v)
    step = {"op": "upsert", "kind": kind}
    if world:
        step["world"] = world
    step["entry"] = entry
    return step


def stored(kind, eid, world):
    """The stored entry, through the CLI `tools/fc.sh content` runs (built on demand, so never a stale binary)."""
    cmd = ["cargo", "run", "-q", "-p", "emberwake_host", "--", "content", "get", kind, eid] + (["world=%s" % world] if world else [])
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, env={**os.environ, "RUST_BACKTRACE": "0"})
    if r.returncode != 0 or not r.stdout.strip():
        msg = (r.stderr or r.stdout).strip()
        raise ValueError(msg.splitlines()[0].removeprefix("Error: ") if msg else "content get %s %s failed" % (kind, eid))
    return json.loads(r.stdout.strip().splitlines()[-1])


def main(argv):
    pos, sets, out, append = [], [], None, False
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--set":
            sets.append(argv[i + 1])
            i += 2
        elif a == "--out":
            out, i = Path(argv[i + 1]), i + 2
        elif a == "--append":
            append, i = True, i + 1
        else:
            pos.append(a)
            i += 1
    if len(pos) not in (2, 3):
        print(__doc__)
        return 2
    kind, eid = pos[0], pos[1]
    world = pos[2] if len(pos) == 3 else None
    try:
        step = seed_step(kind, world, stored(kind, eid, world), sets)
    except ValueError as e:
        print("seed-from: %s" % e)
        return 1
    seed = {"steps": [step]}
    if out and append and out.is_file():
        old = json.loads(out.read_text(encoding="utf-8"))
        seed = {**old, "steps": old.get("steps", []) + [step]}
    text = json.dumps(seed, indent=2, ensure_ascii=False) + "\n"
    if out:
        out.write_text(text, encoding="utf-8")
        print("seed-from: %s %s%s, %d change(s), into %s (%d step(s)); apply: tools/fc.sh content apply %s" % (kind, eid, " in " + world if world else "", len(sets), out, len(seed["steps"]), out))
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
