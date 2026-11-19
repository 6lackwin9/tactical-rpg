#!/usr/bin/env python3
"""TOOLS-HELP3 (ENGINE-20 item 1): `tools/fc.sh seeds [<game>] [--all]`, the seed order check (a gate step).

Every seed of `games/<game>/data/seeds/*.json` replayed in order (the order they were last committed in, ties by
name; a seed not committed yet comes last) must leave the shipped data as it is. A whole-entry upsert replaces the
stored entry, so a seed written from a stale copy of an entry undoes the fields a seed before it set. For every entry
two or more seeds upsert, the last one is compared with the shipped entry (`aegis content get`): a field it writes with
a value other than the shipped one, while an earlier seed wrote the shipped value, is a finding:
"<last> undoes <earlier> on <kind> <id>: <fields>". A step with `"fields": [...]` (`seed-from --fields`) writes only
those fields, so it undoes nothing outside them. A field the last seed leaves out is not compared (the writer's default,
or a field added later).

Findings listed in `tools/planner/seed_order_known.txt` (one `<last> <earlier> <kind> <id>` per line, `#` comments)
are reported as known and pass; any other finding fails (exit 1). Editing a seed moves it last in the order: an old seed that no
longer fits goes to `data/seeds/archive/` (never scanned) instead of being edited in place. `--all` lists every finding, known ones too.
The data is only read: the stored entries come through the game's CLI (`content get`).
"""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
KNOWN = Path(__file__).resolve().parent / "seed_order_known.txt"


def committed_at(path):
    """The time `path` was last committed (seconds); None when it never was."""
    r = subprocess.run(["git", "log", "--format=%at", "-1", "--", str(path)], cwd=ROOT, capture_output=True, text=True)
    out = r.stdout.strip()
    return int(out) if r.returncode == 0 and out else None


def order(paths, when=committed_at):
    """The seeds in replay order: by last commit time, then name; uncommitted ones last, by name."""
    big = float("inf")
    return sorted(paths, key=lambda p: ((when(p) if when(p) is not None else big), Path(p).name))


def writes(seeds):
    """{(kind, world, id): [(seed name, entry, fields or None), ...]} for every upsert, in seed order."""
    out = {}
    for path in seeds:
        try:
            steps = json.loads(Path(path).read_text(encoding="utf-8")).get("steps", [])
        except (OSError, ValueError) as e:
            raise ValueError("%s: %s" % (Path(path).name, e))
        for st in steps:
            if st.get("op", "upsert") != "upsert" or not isinstance(st.get("entry"), dict) or "kind" not in st:
                continue
            key = (st["kind"], st.get("world") or "", st["entry"].get("id", ""))
            out.setdefault(key, []).append((Path(path).name, st["entry"], st.get("fields")))
    return out


def findings(written, stored):
    """[(last, earlier, kind, world, id, fields)] for each entry the last seed's upsert undoes an earlier one on.
    `stored(kind, id, world)`: the shipped entry, None when there is none (removed later: nothing to compare)."""
    out = []
    for (kind, world, eid), ws in written.items():
        if len({w[0] for w in ws}) < 2:
            continue
        ship = stored(kind, eid, world)
        if ship is None:
            continue
        last, entry, fields = ws[-1]
        names = [f for f in entry if f != "id" and (fields is None or f in fields)]
        off = [f for f in names if f in ship and entry[f] != ship[f]]
        by = {}
        for f in off:
            for name, e, fl in reversed(ws[:-1]):
                if name != last and f in e and (fl is None or f in fl) and e[f] == ship[f]:
                    by.setdefault(name, []).append(f)
                    break
        for earlier, fs in by.items():
            out.append((last, earlier, kind, world, eid, fs))
    return out


def known_set(path=KNOWN):
    if not path.is_file():
        return set()
    rows = set()
    for ln in path.read_text(encoding="utf-8").splitlines():
        ln = ln.split("#", 1)[0].split()
        if len(ln) == 4:
            rows.add(tuple(ln))
    return rows


def cli_stored(game, keys):
    """The shipped entries of `keys` [(kind, world, id)] in one CLI call (`content get @<list>`), as a lookup."""
    wanted = [{"kind": k, "id": i, **({"world": w} if w else {})} for k, w, i in keys]
    with tempfile.TemporaryDirectory() as tmp:
        lst = Path(tmp) / "keys.json"
        lst.write_text(json.dumps(wanted), encoding="utf-8")
        cmd = ["cargo", "run", "-q", "-p", "%s_host" % game, "--", "content", "get", "@%s" % lst]
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, env={**os.environ, "RUST_BACKTRACE": "0"})
    if r.returncode != 0:
        raise ValueError("content get: %s" % (r.stderr or r.stdout).strip()[:400])
    got = json.loads(r.stdout.strip().splitlines()[-1])
    table = {k: e for k, e in zip(keys, got)}
    return lambda kind, eid, world: table.get((kind, world, eid))


def boundary():
    """The commits a shallow clone cuts its history at (none in a full clone). A seed whose last commit is one of them
    carries that commit's time, not its own (a depth-1 CI checkout: every seed the same time)."""
    r = subprocess.run(["git", "rev-parse", "--git-path", "shallow"], cwd=ROOT, capture_output=True, text=True)
    path = Path(r.stdout.strip()) if r.returncode == 0 else None
    if path is not None and not path.is_absolute():
        path = ROOT / path
    if path is None or not path.is_file():
        return set()
    return {ln.strip() for ln in path.read_text(encoding="utf-8").splitlines() if ln.strip()}


def cut_off(paths, cut=None):
    """The seeds of `paths` whose last commit is a shallow clone's boundary commit (their commit order is unknown)."""
    cut = boundary() if cut is None else cut
    if not cut:
        return []
    out = []
    for p in paths:
        r = subprocess.run(["git", "log", "--format=%H", "-1", "--", str(p)], cwd=ROOT, capture_output=True, text=True)
        if r.stdout.strip() in cut:
            out.append(Path(p).name)
    return out


def main(argv):
    show_all = "--all" in argv
    folder = ROOT / "games" / (([a for a in argv if not a.startswith("--")] or ["emberwake"])[0]) / "data" / "seeds"
    lost = cut_off(sorted(folder.glob("*.json")))
    if lost:
        print("[seeds] FAIL: a shallow clone hides the commit order of %d seed(s) (%s); check out with full history (git fetch --unshallow, or fetch-depth: 0 in CI)" % (len(lost), ", ".join(lost[:5]) + (" ..." if len(lost) > 5 else "")))
        return 1
    pos = [a for a in argv if not a.startswith("--")]
    game = pos[0] if pos else "emberwake"
    folder = ROOT / "games" / game / "data" / "seeds"
    seeds = order(sorted(folder.glob("*.json")))
    try:
        written = writes(seeds)
        multi = [k for k, ws in written.items() if len({w[0] for w in ws}) >= 2]
        found = findings(written, cli_stored(game, multi))
    except ValueError as e:
        print("[seeds] FAIL: %s" % e)
        return 1
    known = known_set()
    new = []
    for last, earlier, kind, world, eid, fs in found:
        row = (last, earlier, kind, eid)
        line = "%s undoes %s on %s %s%s: %s" % (last, earlier, kind, eid, " (%s)" % world if world else "", ", ".join(fs))
        if row in known:
            if show_all:
                print("[seeds] known: " + line)
        else:
            new.append(line)
            print("[seeds] FAIL: " + line + " (re-save the last seed from the stored entry, or write it with seed-from --fields)")
    n_known = sum(1 for f in found if (f[0], f[1], f[2], f[4]) in known)
    if new:
        return 1
    print("[seeds] ok: %d seeds replay in order to the shipped data (%d known finding(s), %s)" % (len(seeds), n_known, KNOWN.name))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
