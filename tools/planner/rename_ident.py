#!/usr/bin/env python3
"""TOOLS-HELP2 (ENGINE-16): `tools/fc.sh rename <old> <new> [<path>...] [--strings] [--comments] [--dry]`.

Renames an identifier (whole word, case sensitive) across the repository's Rust and JSON files (tracked by git, or the
paths given: files or folders). In Rust only code changes by default: string literals (plain, byte, raw `r#"..."#`),
char literals and comments (line, block, nested; doc comments too) are left alone unless `--strings` / `--comments`.
In JSON an object key or a string value that is exactly <old> is renamed (`--strings` also renames <old> as a word
inside a longer string). Never touched, only listed: `legacy/`, shipped game data (`games/*/data/`: edit it through
`tools/fc.sh content`) and test fixtures (`tests/fixtures/`: goldens are re-recorded, not edited). Prints the files
and counts, plus the hits it left (in strings, comments or skipped files); `--dry` changes nothing.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\Z")


def skipped(rel):
    parts = rel.split("/")
    if parts[0] == "legacy":
        return "legacy"
    if len(parts) > 2 and parts[0] == "games" and parts[2] == "data":
        return "game data"
    if "/tests/fixtures/" in "/" + rel:
        return "test fixture"
    return None


def rust_spans(src):
    """[(kind, text)] covering src: kind is code, string or comment."""
    out, i, n, start = [], 0, len(src), 0

    def flush(upto):
        if upto > start:
            out.append(("code", src[start:upto]))

    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            flush(i); out.append(("comment", src[i:j])); i = start = j
            continue
        if src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            flush(i); out.append(("comment", src[i:j])); i = start = j
            continue
        m = re.match(r'(?:b|c)?r(#*)"', src[i:]) if c in "bcr" and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")) else None
        if m:
            close = '"' + m.group(1)
            j = src.find(close, i + m.end())
            j = n if j < 0 else j + len(close)
            flush(i); out.append(("string", src[i:j])); i = start = j
            continue
        if c == '"' or (c in "bc" and src.startswith('"', i + 1) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_"))):
            j = i + (1 if c == '"' else 2)
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            j = min(j + 1, n)
            flush(i); out.append(("string", src[i:j])); i = start = j
            continue
        if c == "'":
            m = re.match(r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'\n])'", src[i:])
            if m:
                flush(i); out.append(("string", m.group(0))); i = start = i + m.end()
                continue
        i += 1
    flush(n)
    return out


def word(old):
    return re.compile(r"(?<![A-Za-z0-9_])%s(?![A-Za-z0-9_])" % re.escape(old))


def rename_rust(src, old, new, strings=False, comments=False):
    """(new text, renamed, left in strings, left in comments)."""
    w = word(old)
    parts, done, left_s, left_c = [], 0, 0, 0
    for kind, text in rust_spans(src):
        k = len(w.findall(text))
        if kind == "code" or (kind == "string" and strings) or (kind == "comment" and comments):
            parts.append(w.sub(new, text))
            done += k
        else:
            parts.append(text)
            if kind == "string":
                left_s += k
            else:
                left_c += k
    return "".join(parts), done, left_s, left_c


def rename_json(src, old, new, strings=False):
    """(new text, renamed, left in longer strings). Works on the string tokens of the file text, so its layout stays."""
    w = word(old)
    counts = [0, 0]

    def tok(m):
        raw = m.group(0)
        try:
            val = json.loads(raw)
        except ValueError:
            return raw
        if val == old:
            counts[0] += 1
            return json.dumps(new, ensure_ascii=False)
        k = len(w.findall(val))
        if not k:
            return raw
        if strings:
            counts[0] += k
            return '"' + w.sub(new, raw[1:-1]) + '"'
        counts[1] += k
        return raw

    out = re.sub(r'"(?:[^"\\]|\\.)*"', tok, src)
    return out, counts[0], counts[1]


def files(paths):
    if paths:
        got = []
        for p in paths:
            p = (ROOT / p).resolve()
            got += [p] if p.is_file() else [q for q in p.rglob("*") if q.suffix in (".rs", ".json")]
        return sorted(set(got))
    r = subprocess.run(["git", "ls-files", "*.rs", "*.json"], cwd=ROOT, capture_output=True, text=True, check=True)
    return [ROOT / l for l in r.stdout.splitlines() if l]


def main(argv):
    flags = {a for a in argv if a.startswith("--")}
    pos = [a for a in argv if not a.startswith("--")]
    bad = flags - {"--strings", "--comments", "--dry"}
    if len(pos) < 2 or bad:
        print(__doc__)
        return 2
    old, new, paths = pos[0], pos[1], pos[2:]
    if not IDENT.match(old) or not IDENT.match(new):
        print("rename: <old> and <new> are identifiers (letters, digits, _)")
        return 2
    strings, comments, dry = "--strings" in flags, "--comments" in flags, "--dry" in flags
    w = word(old)
    total, changed, left, untouched = 0, [], [], []
    for f in files(paths):
        try:
            src = f.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if not w.search(src):
            continue
        rel = f.relative_to(ROOT).as_posix() if f.is_relative_to(ROOT) else str(f)
        why = skipped(rel)
        if why:
            untouched.append("%s (%s, %d hit(s))" % (rel, why, len(w.findall(src))))
            continue
        if f.suffix == ".rs":
            out, k, ls, lc = rename_rust(src, old, new, strings, comments)
        else:
            out, k, ls = rename_json(src, old, new, strings)
            lc = 0
        if k:
            changed.append("%s: %d" % (rel, k))
            total += k
            if not dry:
                f.write_text(out, encoding="utf-8")
        if ls or lc:
            left.append("%s: %d in strings, %d in comments" % (rel, ls, lc))
    print("rename %s -> %s: %d hit(s) in %d file(s)%s" % (old, new, total, len(changed), " (dry run, nothing written)" if dry else ""))
    for line in changed:
        print("  " + line)
    if left:
        print("left (--strings / --comments to include):")
        for line in left:
            print("  " + line)
    if untouched:
        print("not touched:")
        for line in untouched:
            print("  " + line)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
