"""PILLAR1 (GATE-EFFICIENCY): fast static check for workspace clippy macro denies (`dbg!`, `todo!`, `unimplemented!`).

`tools/fc.sh build` runs `cargo build --workspace --all-targets`, which already enforces `[workspace.lints.rust]`
(`unsafe_code = "deny"`) and all compiler warnings. Running `cargo clippy --workspace --all-targets` again solely for
the three macro-only clippy denies re-checks hundreds of targets (~3 min on 2 vCPUs).

Exit codes:
  0: every clippy deny in Cargo.toml is covered by this scanner and all crates are clean.
  1: one or more denied macro invocations were found (file:line printed).
  2: Cargo.toml defines a clippy deny outside the macro-only set; caller must run `cargo clippy`.
"""
import glob
from pathlib import Path
import re
import sys
import tomllib

MACRO_LINTS = {
    "dbg_macro": "dbg",
    "todo": "todo",
    "unimplemented": "unimplemented",
}


def workspace_clippy_denies(root="."):
    """Returns (covered_macros, unsupported_lints) from `[workspace.lints.clippy]` in root Cargo.toml."""
    with open(Path(root) / "Cargo.toml", "rb") as f:
        manifest = tomllib.load(f)
    clippy = manifest.get("workspace", {}).get("lints", {}).get("clippy", {})
    macros = {}
    unsupported = []
    for lint, spec in clippy.items():
        level = spec.get("level") if isinstance(spec, dict) else spec
        if level != "deny":
            continue
        if lint in MACRO_LINTS:
            macros[MACRO_LINTS[lint]] = lint
        else:
            unsupported.append(lint)
    return macros, unsupported


def strip_rust_comments_and_strings(src):
    """Replaces Rust comments, strings, raw strings, and char literals with spaces while preserving line breaks."""
    out = []
    i = 0
    n = len(src)
    while i < n:
        ch = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        # Line comment
        if ch == "/" and nxt == "/":
            i += 2
            while i < n and src[i] != "\n":
                i += 1
            continue
        # Nested block comment
        if ch == "/" and nxt == "*":
            depth = 1
            i += 2
            while i < n and depth > 0:
                if src[i] == "/" and i + 1 < n and src[i + 1] == "*":
                    depth += 1
                    i += 2
                elif src[i] == "*" and i + 1 < n and src[i + 1] == "/":
                    depth -= 1
                    i += 2
                else:
                    if src[i] == "\n":
                        out.append("\n")
                    i += 1
            out.append(" ")
            continue
        # Raw string / byte raw string / c raw string: r"...", r#"..."#, br#"..."#, cr#"..."#
        if ch in ("r", "b", "c") and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            j = i
            if ch in ("b", "c") and j + 1 < n and src[j + 1] == "r":
                j += 1
            if src[j] == "r":
                k = j + 1
                while k < n and src[k] == "#":
                    k += 1
                if k < n and src[k] == '"':
                    hashes = k - (j + 1)
                    i = k + 1
                    end_pat = '"' + ("#" * hashes)
                    while i < n:
                        if src.startswith(end_pat, i):
                            i += len(end_pat)
                            break
                        if src[i] == "\n":
                            out.append("\n")
                        i += 1
                    out.append(" ")
                    continue
        # Normal / byte / c string literal
        if ch == '"' or (ch in ("b", "c") and nxt == '"' and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_"))):
            i += 2 if ch != '"' else 1
            while i < n:
                if src[i] == "\\":
                    if i + 1 < n and src[i + 1] == "\n":
                        out.append("\n")
                    i += 2
                    continue
                if src[i] == '"':
                    i += 1
                    break
                if src[i] == "\n":
                    out.append("\n")
                i += 1
            out.append(" ")
            continue
        # Char literal (distinguish from lifetime `'a`: char literal has closing `'` within escape or 1 char)
        if ch == "'" and i + 2 < n:
            if nxt == "\\" and i + 3 < n:
                end = src.find("'", i + 2)
                if end != -1 and end - i <= 12 and "\n" not in src[i:end]:
                    i = end + 1
                    out.append(" ")
                    continue
            elif nxt != "'" and nxt != "\n" and src[i + 2] == "'":
                i += 3
                out.append(" ")
                continue
        out.append(ch)
        i += 1
    return "".join(out)


def scan_source(src, macros):
    """Returns [(line_no, macro_name, lint_name)] for denied macro calls in Rust source text."""
    if not macros:
        return []
    pat = re.compile(r"\b(" + "|".join(re.escape(m) for m in sorted(macros)) + r")\s*!")
    if not pat.search(src):
        return []
    cleaned = strip_rust_comments_and_strings(src)
    if not pat.search(cleaned):
        return []
    lines = cleaned.splitlines()
    file_allowed = set()
    for m in re.finditer(r"#!\s*\[\s*(?:allow|expect)\s*\(([^)]*)\)\s*\]", cleaned):
        for name in re.findall(r"clippy::([a-z_]+)", m.group(1)):
            file_allowed.add(name)
    allow_re = re.compile(r"#\s*\[\s*(?:allow|expect)\s*\(([^)]*)\)\s*\]")
    hits = []
    next_allowed = set()
    brace_depth = 0
    scope_allows = []  # [(depth, set_of_lints)]
    for lineno, line in enumerate(lines, 1):
        for am in allow_re.finditer(line):
            for name in re.findall(r"clippy::([a-z_]+)", am.group(1)):
                next_allowed.add(name)
        active = set(file_allowed) | set(next_allowed)
        for _, ls in scope_allows:
            active |= ls
        for m in pat.finditer(line):
            macro = m.group(1)
            lint = macros[macro]
            if lint not in active:
                hits.append((lineno, macro, lint))
        opens = line.count("{")
        closes = line.count("}")
        if next_allowed:
            if opens > closes:
                scope_allows.append((brace_depth, set(next_allowed)))
                next_allowed.clear()
            elif not allow_re.search(line) and line.strip():
                next_allowed.clear()
        brace_depth = max(0, brace_depth + opens - closes)
        while scope_allows and brace_depth <= scope_allows[-1][0]:
            scope_allows.pop()
    return hits


def rust_files(root="."):
    crates = sorted(
        glob.glob(f"{root}/engine/crates/*")
        + glob.glob(f"{root}/kits/*/crates/*")
        + glob.glob(f"{root}/games/*/crates/*")
    )
    files = []
    for c in crates:
        for path in sorted(glob.glob(f"{c}/**/*.rs", recursive=True)):
            files.append(path)
    return files


def main(root="."):
    macros, unsupported = workspace_clippy_denies(root)
    if unsupported:
        print(f"[lints] non-macro clippy deny(s) {unsupported}: requires cargo clippy")
        return 2
    files = rust_files(root)
    bad = []
    for path in files:
        text = Path(path).read_text(encoding="utf-8")
        for lineno, macro, lint in scan_source(text, macros):
            bad.append(f"{path}:{lineno}: denied macro `{macro}!` (clippy::{lint})")
    for item in bad[:60]:
        print(f"[lints] {item}")
    if bad:
        print(f"[lints] FAIL: {len(bad)} denied macro call(s) across {len(files)} Rust file(s)")
        return 1
    print(f"[lints] deny set clean ({len(files)} Rust files, macros: {', '.join(sorted(macros))})")
    return 0


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:]))
