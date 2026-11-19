#!/usr/bin/env python3
"""GOLDEN-BISECT1 (ENGINE-14): `tools/fc.sh golden-bisect <golden> <switch> [--keep DIR]`.

Records a golden twice, with the named test-only switch off and on (`aegis_core::switches`, environment variable
`AEGIS_SWITCHES`), and lists what moves: the entries (fights, runs, units, cases) whose recording differs between the
two, plus anything the recording with the switch off already moves against the committed golden (the branch's own
drift). The committed fixtures are restored byte for byte afterwards, whatever happens; nothing is re-recorded.

<golden> is a golden test's file name under some crate's tests/ (`timeline_golden`, `sim_golden`, `rules_golden`,
`phasefold`, `journey`, `checks_golden`, `rules_report`, `rng_hash`), with or without `_golden`. The test must record
under `RECORD=1` (every engine golden does). <switch> must be read somewhere in the crates as `switches::on("<switch>")`.
`golden-bisect list` lists the goldens and the switches the code reads.

`golden-bisect repro <switch>` (TOOLS-HELP1, ENGINE-15): replays every repro recording (`games/<id>/playtests/repro/
*.json`, through the game's CLI `replay`, as `tools/fc.sh repro` does) with the switch off and on, and lists per
recording whether it matches and, when it does not, its first divergence lines.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRATE_GLOBS = ["engine/crates/*", "kits/*/crates/*", "games/*/crates/*"]
LABEL_KEYS = ("fight", "label", "argv", "line", "cmd", "run", "key", "id", "name", "unit", "case")
SWITCH_RE = re.compile(r'switches::on\("([A-Za-z0-9_.-]+)"\)')


def crates(root=ROOT):
    out = []
    for g in CRATE_GLOBS:
        out.extend(p for p in sorted(root.glob(g)) if (p / "Cargo.toml").is_file())
    return out


def goldens(root=ROOT):
    """{test name: crate dir} for every test file that records under RECORD=1."""
    found = {}
    for c in crates(root):
        for t in sorted((c / "tests").glob("*.rs")):
            text = t.read_text(encoding="utf-8", errors="replace")
            if '"RECORD"' in text:
                found[t.stem] = c
    return found


def find_golden(name, root=ROOT):
    g = goldens(root)
    for cand in (name, name + "_golden"):
        if cand in g:
            return cand, g[cand]
    raise SystemExit("golden-bisect: no golden '%s' (one of: %s)" % (name, ", ".join(sorted(g))))


def switches_read(root=ROOT):
    """{switch: [files that read it]} over every crate's src and tests."""
    out = {}
    for c in crates(root):
        for f in sorted(c.rglob("*.rs")):
            if "target" in f.parts:
                continue
            for m in SWITCH_RE.finditer(f.read_text(encoding="utf-8", errors="replace")):
                out.setdefault(m.group(1), []).append(f.relative_to(root).as_posix())
    return out


def crate_name(crate):
    m = re.search(r'^name\s*=\s*"([^"]+)"', (crate / "Cargo.toml").read_text(encoding="utf-8"), re.M)
    return m.group(1)


def label(entry, i):
    if isinstance(entry, dict):
        for k in LABEL_KEYS:
            if isinstance(entry.get(k), str):
                return entry[k]
    if isinstance(entry, list) and entry and isinstance(entry[0], str):
        return entry[0]
    return "#%d" % (i + 1)


def moved(a, b, at=""):
    """What differs between two recordings: a list of (section, total, [labels]) for each list of entries, and
    (path, None, []) for any other value that differs."""
    out = []
    if isinstance(a, dict) and isinstance(b, dict):
        for k in list(a) + [k for k in b if k not in a]:
            if a.get(k) != b.get(k):
                out.extend(moved(a.get(k), b.get(k), "%s.%s" % (at, k) if at else k))
    elif isinstance(a, list) and isinstance(b, list):
        n = max(len(a), len(b))
        labels = []
        for i in range(n):
            x = a[i] if i < len(a) else None
            y = b[i] if i < len(b) else None
            if x != y:
                labels.append(label(x if x is not None else y, i))
        out.append((at or "(root)", n, labels))
    else:
        out.append((at or "(root)", None, []))
    return out


def describe(diffs):
    lines = []
    for section, total, labels in diffs:
        if total is None:
            lines.append("  %s: differs" % section)
        else:
            lines.append("  %s: %d of %d moved: %s" % (section, len(labels), total, ", ".join(labels)))
    return lines or ["  nothing moves"]


def read(path):
    text = path.read_text(encoding="utf-8")
    try:
        return json.loads(text)
    except ValueError:
        return {"lines": text.splitlines()}


def snapshot(dirs):
    return {p: p.read_bytes() for d in dirs if d.is_dir() for p in sorted(d.rglob("*")) if p.is_file()}


def record(test, crate, switch_on, switch, fixture_dirs, cargo):
    """Runs the golden under RECORD=1 and returns {path: bytes} of the files it wrote (changed or new)."""
    before = snapshot(fixture_dirs)
    env = dict(os.environ, RECORD="1")
    env.pop("AEGIS_SWITCHES", None)
    if switch_on:
        env["AEGIS_SWITCHES"] = switch
    r = subprocess.run(cargo + ["test", "-q", "-p", crate_name(crate), "--test", test], cwd=ROOT, env=env, capture_output=True, text=True)
    after = snapshot(fixture_dirs)
    written = {p: b for p, b in after.items() if before.get(p) != b}
    if r.returncode != 0:
        sys.stderr.write(r.stdout[-3000:] + r.stderr[-3000:])
        raise RuntimeError("the golden did not record with %s %s" % (switch, "on" if switch_on else "off"))
    return before, written


def game_cli(game_dir):
    """The game's CLI program: game.json's `cli`, else `<id>_cli` (ENGINE-11, as tools/fc.sh game_cli)."""
    try:
        m = json.loads((game_dir / "game.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        m = {}
    return m.get("cli") or "%s_cli" % game_dir.name


def repro_files(root=ROOT):
    return sorted(p for p in root.glob("games/*/playtests/repro/*.json") if not p.name.endswith(".tmp"))


def replay_verdict(code, out):
    """(matches, the lines that say why not)."""
    why = [l for l in out.splitlines() if re.search(r"diverge|broken|differs|load error|panicked|refused|mismatch", l)]
    return code == 0, why[:6]


def repro(switch, readers):
    files = repro_files()
    if not files:
        print("golden-bisect repro: no recordings under games/*/playtests/repro/")
        return 0
    b = subprocess.run(["cargo", "build", "-q", "--workspace", "--bins"], cwd=ROOT)
    if b.returncode != 0:
        return 1
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug"
    print("golden-bisect repro, switch %s (read in %s)" % (switch, ", ".join(readers)))
    moved_n = 0
    for f in files:
        verdicts = {}
        for on in (False, True):
            env = dict(os.environ)
            env.pop("AEGIS_SWITCHES", None)
            if on:
                env["AEGIS_SWITCHES"] = switch
            r = subprocess.run([str(target / game_cli(f.parents[2])), "replay", str(f)], cwd=ROOT, env=env, capture_output=True, text=True)
            verdicts[on] = replay_verdict(r.returncode, r.stdout + r.stderr)
        rel = f.relative_to(ROOT)
        (off_ok, off_why), (on_ok, on_why) = verdicts[False], verdicts[True]
        if off_ok and on_ok:
            print("  %s: matches with the switch off and on" % rel)
            continue
        if off_ok and not on_ok:
            moved_n += 1
        print("  %s: switch off %s, on %s" % (rel, "matches" if off_ok else "diverges", "matches" if on_ok else "diverges"))
        for tag, why in (("off", off_why), ("on", on_why)):
            for line in why:
                print("    %s: %s" % (tag, line.strip()))
    print("  %d of %d recording(s) move with the switch" % (moved_n, len(files)))
    return 0


def main(argv):
    if argv[:1] == ["repro"]:
        if len(argv) < 2:
            print(__doc__)
            return 2
        readers = switches_read().get(argv[1])
        if not readers:
            print("golden-bisect: no code reads switch '%s'" % argv[1])
            return 2
        return repro(argv[1], readers)
    if argv[:1] == ["list"]:
        print("goldens: " + ", ".join(sorted(goldens())))
        sw = switches_read()
        print("switches: " + (", ".join("%s (%s)" % (k, ", ".join(v)) for k, v in sorted(sw.items())) or "none read yet"))
        return 0
    if len(argv) < 2:
        print(__doc__)
        return 2
    keep = None
    if "--keep" in argv:
        i = argv.index("--keep")
        keep = Path(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    test, crate = find_golden(argv[0])
    switch = argv[1]
    readers = switches_read().get(switch)
    if not readers:
        print("golden-bisect: no code reads switch '%s' (switches::on(\"%s\")); `golden-bisect list` shows those read" % (switch, switch))
        return 2
    cargo = ["cargo"]
    fixture_dirs = sorted({c / "tests" / "fixtures" for c in crates()})
    original = snapshot(fixture_dirs)
    runs = {}
    try:
        for on in (False, True):
            _, written = record(test, crate, on, switch, fixture_dirs, cargo)
            runs[on] = written
            # back to the committed fixtures before the next run, so both record from the same frozen inputs
            for p in snapshot(fixture_dirs):
                if p not in original:
                    p.unlink()
            for p, b in original.items():
                if p.read_bytes() != b:
                    p.write_bytes(b)
    finally:
        for p in snapshot(fixture_dirs):
            if p not in original:
                p.unlink()
        for p, b in original.items():
            if not p.exists() or p.read_bytes() != b:
                p.write_bytes(b)
    paths = sorted(set(runs[False]) | set(runs[True]))
    print("golden-bisect %s, switch %s (read in %s)" % (test, switch, ", ".join(readers)))
    if not paths:
        print("  nothing moves: both recordings equal the committed golden")
    for p in paths:
        rel = p.relative_to(ROOT)
        committed = read(p) if p in original else None
        off = json.loads(runs[False][p]) if p in runs[False] else committed
        on = json.loads(runs[True][p]) if p in runs[True] else committed
        print("%s, switch on against off:" % rel)
        print("\n".join(describe(moved(off, on))))
        if committed is not None and off != committed:
            print("%s, switch off against the committed golden (moves without the switch):" % rel)
            print("\n".join(describe(moved(committed, off))))
        if keep:
            keep.mkdir(parents=True, exist_ok=True)
            for tag, data in (("off", runs[False].get(p)), ("on", runs[True].get(p))):
                if data is not None:
                    (keep / ("%s.%s%s" % (p.stem, tag, p.suffix))).write_bytes(data)
    print("(committed fixtures restored)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
