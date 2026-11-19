"""CI-HARDEN1 (ENGINE-4): every dependency a crate's Cargo.toml names is used in its Rust code.

A dependency counts as used when its name (dashes as underscores) appears as a word in the crate's .rs files outside
`//` comments. One that is named only to turn on a feature of a transitive crate goes in KEEP with the reason.
Exit 1 and one line per unused dependency. Plain Python 3.11+ (tomllib); run from the repository root.
"""
import glob
from pathlib import Path
import re
import sys
import tomllib

# (crate folder name, dependency): named only for a feature, never imported
KEEP = {
    ("aegis_ui", "cpal"): "turns on cpal's wasm-bindgen feature (Web Audio), which rodio does not forward",
}


def crates(root="."):
    return sorted(glob.glob(f"{root}/engine/crates/*") + glob.glob(f"{root}/kits/*/crates/*") + glob.glob(f"{root}/games/*/crates/*"))


def unused(crate):
    with open(f"{crate}/Cargo.toml", "rb") as f:
        manifest = tomllib.load(f)
    src = ""
    for path in glob.glob(f"{crate}/**/*.rs", recursive=True):
        with open(path, encoding="utf-8") as f:
            src += re.sub(r"//.*", "", f.read())
    tables = [manifest.get(k, {}) for k in ("dependencies", "dev-dependencies", "build-dependencies")]
    for target in manifest.get("target", {}).values():
        tables += [target.get(k, {}) for k in ("dependencies", "dev-dependencies", "build-dependencies")]
    name = Path(crate).name  # on Windows glob joins with a backslash: never split the path on "/"
    out = []
    for table in tables:
        for dep in table:
            if (name, dep) not in KEEP and not re.search(r"\b" + re.escape(dep.replace("-", "_")) + r"\b", src):
                out.append(f"{crate}/Cargo.toml: dependency '{dep}' is not used in the crate's code")
    return out


def main(root="."):
    problems = [p for c in crates(root) for p in unused(c)]
    for p in problems:
        print(f"[deps] {p}")
    print(f"[deps] {len(crates(root))} crates, {len(problems)} unused dependencies")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:]))
