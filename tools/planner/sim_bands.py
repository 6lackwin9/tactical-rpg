"""Balance target bands as a failing check (PR7-6, BALANCE-BANDS1): `tools/fc.sh sim bands`.

Reads the bands from `games/emberwake/tuning.json` key `balance_bands`, runs the existing sim modes (`sim duel` with
score=binary for the class duel win %, `sim party` on the generated encounters for the clear %), prints their tables,
then one row per class / encounter (measured, band, in/out). Exit 0 when every row is inside its band, 1 when any row
is outside it (or missing from the sim output), 2 when the bands block is malformed or a sim run fails.

The block (hand-maintained, tuning.json is no content kind):
    "balance_bands": {"level": 5,
                      "duel": {"seeds": 40, "win_pct": [40, 60]},
                      "party": {"seeds": 100, "clear_pct": {"line": [80, 95], "elite": [70, 85], "boss": [30, 50]}}}
Seeds are the sim's own set (seed i = 11 + 12 i). Options: --tuning <file>, --sim "<command>" (default the CLI through
cargo), level=<n> duel_seeds=<n> party_seeds=<n> (override the block for a quicker run), quiet (only the bands table).
"""
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
TUNING = ROOT / "games" / "emberwake" / "tuning.json"
SIM = "cargo run -q -p emberwake_host -- sim"
# The generated encounters whose clear % the sim marks against a `clear_<id>` target (rpgkit_sim targets.rs)
GENERATED = ("mob", "line", "wave", "elite", "boss")


class BandsError(Exception):
    """A malformed bands block or a sim run that did not finish."""


def band(value, where):
    """[lo, hi] as two numbers 0..100 with lo <= hi."""
    ok = isinstance(value, list) and len(value) == 2 and all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in value)
    if not ok or not 0 <= value[0] <= value[1] <= 100:
        raise BandsError(f"{where} must be [lo, hi] with 0 <= lo <= hi <= 100 (got {json.dumps(value)})")
    return (value[0], value[1])


def positive(value, where):
    if not isinstance(value, int) or isinstance(value, bool) or value < 1:
        raise BandsError(f"{where} must be a positive whole number (got {json.dumps(value)})")
    return value


def parse_bands(block):
    """The checked bands block: {"level", "duel": {"seeds", "win_pct"} | None, "party": {"seeds", "clear_pct"} | None}."""
    if not isinstance(block, dict):
        raise BandsError("balance_bands must be an object")
    known = {"_doc", "level", "duel", "party"}
    extra = sorted(set(block) - known)
    if extra:
        raise BandsError(f"balance_bands: unknown key(s) {', '.join(extra)} (known: level, duel, party)")
    level = positive(block.get("level", 5), "balance_bands.level")
    if level > 30:
        raise BandsError(f"balance_bands.level must be 1..30 (got {level})")
    out = {"level": level, "duel": None, "party": None}
    duel = block.get("duel")
    if duel is not None:
        if not isinstance(duel, dict) or "win_pct" not in duel:
            raise BandsError("balance_bands.duel must be an object with win_pct")
        out["duel"] = {"seeds": positive(duel.get("seeds", 3), "balance_bands.duel.seeds"), "win_pct": band(duel["win_pct"], "balance_bands.duel.win_pct")}
    party = block.get("party")
    if party is not None:
        clear = party.get("clear_pct") if isinstance(party, dict) else None
        if not isinstance(clear, dict) or not clear:
            raise BandsError("balance_bands.party must be an object with a non-empty clear_pct")
        rows = {}
        for enc, b in clear.items():
            if enc not in GENERATED:
                raise BandsError(f"balance_bands.party.clear_pct: '{enc}' is no generated encounter ({', '.join(GENERATED)})")
            rows[enc] = band(b, f"balance_bands.party.clear_pct.{enc}")
        out["party"] = {"seeds": positive(party.get("seeds", 10), "balance_bands.party.seeds"), "clear_pct": rows}
    if out["duel"] is None and out["party"] is None:
        raise BandsError("balance_bands names no rows (duel or party)")
    return out


def load_bands(path):
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        raise BandsError(f"cannot read {path}: {e}")
    if "balance_bands" not in data:
        raise BandsError(f"{path} has no balance_bands block")
    return parse_bands(data["balance_bands"])


def pct(token):
    m = re.fullmatch(r"(\d+(?:\.\d+)?)%?", token)
    return float(m.group(1)) if m else None


def duel_rows(lines):
    """{class: win %} from a `sim duel score=binary` table: the header after the DUEL line names the classes; each row
    is the class, one cell per other class ("-" against itself) and the row average."""
    rows = {}
    for i, line in enumerate(lines):
        if not line.startswith("DUEL") or i + 1 >= len(lines):
            continue
        classes = lines[i + 1].split()[:-1]  # the last header word is "avg"
        for row in lines[i + 2:]:
            words = row.split()
            if not words or words[0] not in classes or len(words) < len(classes) + 2:
                break
            value = pct(words[len(classes) + 1])
            if value is not None:
                rows[words[0]] = value
    return rows


def party_rows(lines):
    """{encounter: clear %} from a `sim party` table (the rows under the `encounter clear% ...` header)."""
    rows = {}
    for i, line in enumerate(lines):
        words = line.split()
        if words[:2] != ["encounter", "clear%"]:
            continue
        for row in lines[i + 1:]:
            w = row.split()
            if len(w) < 2 or pct(w[1]) is None:
                if w and w[0].endswith(":"):  # a note line under a row ("  line: 2 fights skipped ...")
                    continue
                break
            rows[w[0]] = pct(w[1])
    return rows


def run_sim(sim, argv):
    cmd = shlex.split(sim) + argv
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
    except OSError as e:
        raise BandsError(f"cannot start the sim ({' '.join(cmd)}): {e}")
    lines = p.stdout.splitlines()
    if p.returncode != 0 or any(l.startswith("SIM argument error") for l in lines):
        tail = "\n".join((p.stdout + p.stderr).splitlines()[-12:])
        raise BandsError(f"sim failed (exit {p.returncode}): {' '.join(cmd)}\n{tail}")
    return lines


def fmt_band(b):
    lo, hi = b
    return f"{lo:g}-{hi:g}"


def check(bands, sim, echo=print):
    """Runs the sims of `bands`; returns the table rows [(row, measured or None, band, inside)]."""
    level, rows = bands["level"], []
    if bands["duel"]:
        d = bands["duel"]
        lines = run_sim(sim, ["duel", str(level), str(d["seeds"]), "score=binary"])
        echo("\n".join(lines))
        measured = duel_rows(lines)
        if not measured:
            raise BandsError("the duel output has no class rows")
        for cls, v in measured.items():
            rows.append((f"duel {cls}", v, d["win_pct"], d["win_pct"][0] <= v <= d["win_pct"][1]))
    if bands["party"]:
        p = bands["party"]
        for enc, b in p["clear_pct"].items():
            # one run per encounter so the sim's own band column shows this band (the CLI keeps one value per key, so
            # a second target= on one command line would replace the first)
            lines = run_sim(sim, ["party", str(level), str(p["seeds"]), f"enc={enc}", "metrics=off", f"target=clear_{enc}:{fmt_band(b)}"])
            echo("\n".join(lines))
            v = party_rows(lines).get(enc)
            rows.append((f"party {enc} clear", v, b, v is not None and b[0] <= v <= b[1]))
    return rows


def table(bands, rows):
    out = []
    d, p = bands["duel"], bands["party"]
    parts = []
    if d:
        parts.append(f"duel {d['seeds']} seeds (win % = binary score)")
    if p:
        parts.append(f"party {p['seeds']} seeds per encounter (clear %)")
    out.append(f"BANDS level {bands['level']}: " + ", ".join(parts))
    out.append(f"{'row':<20} {'measured':>9} {'band':>8}  in/out")
    for name, v, b, inside in rows:
        shown = "missing" if v is None else f"{v:g}%"
        out.append(f"{name:<20} {shown:>9} {fmt_band(b):>8}  {'in' if inside else 'OUT'}")
    bad = sum(1 for r in rows if not r[3])
    out.append(f"bands: {len(rows) - bad} in, {bad} out of {len(rows)} rows" + ("" if bad == 0 else " (FAIL)"))
    return out


def main(argv):
    tuning, sim, quiet, over = TUNING, SIM, False, {}
    args = list(argv)
    try:
        while args:
            a = args.pop(0)
            if a == "--tuning" and args:
                tuning = Path(args.pop(0))
            elif a == "--sim" and args:
                sim = args.pop(0)
            elif a == "quiet":
                quiet = True
            elif "=" in a and a.split("=", 1)[0] in ("level", "duel_seeds", "party_seeds"):
                k, v = a.split("=", 1)
                if not v.isdigit() or int(v) < 1:
                    raise BandsError(f"{k} must be a positive whole number (got '{v}')")
                over[k] = int(v)
            else:
                raise BandsError(f"unknown argument '{a}' (--tuning <file>, --sim <command>, level=, duel_seeds=, party_seeds=, quiet)")
        bands = load_bands(tuning)
        if "level" in over:
            bands["level"] = over["level"]
        for k, sec in (("duel_seeds", "duel"), ("party_seeds", "party")):
            if k in over and bands[sec]:
                bands[sec]["seeds"] = over[k]
        rows = check(bands, sim, echo=(lambda _s: None) if quiet else print)
    except BandsError as e:
        print(f"[sim bands] ERROR: {e}")
        return 2
    print("\n".join(table(bands, rows)))
    return 0 if all(r[3] for r in rows) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
