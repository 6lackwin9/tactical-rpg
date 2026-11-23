"""ENGINE-57 (FEEL-RELEASE1): `tools/fc.sh feel`'s table. Every `feel` and `reach` step of a game's scenarios, one row
each, from the bundles a release run wrote (`<out>/<scenario id>/<WxH>/result.json`, its `perf` list).

    feel_table.py list <scenarios dir>...       the scenario files with a feel or reach step, one per line
    feel_table.py table <out> <scenario file>...   the table; exit 1 when a scenario failed or did not run

A value over the step's own budget ends in `*` (engine/docs/authoring/scenarios.md "Reading the feel table")."""
import json
import sys
from pathlib import Path

KINDS = ("feel", "reach")

# (header, budget key or None, how the value is read from a feel step's perf entry)
COLUMNS = [
    ("input", "input_ms", lambda p: p.get("to_present", {}).get("worst")),
    ("frame p50", "frame_p50_ms", lambda p: p.get("frame_time", {}).get("p50")),
    ("frame max", "frame_ms", lambda p: p.get("frame_time", {}).get("worst")),
    ("switch", None, lambda p: max((s.get("own_ms") or 0 for s in (p.get("switch") or {}).get("switches", [])), default=None)),
    ("move", "first_move_ms", lambda p: (p.get("motion") or {}).get("first_move_ms")),
    ("ui snd", "ui_sound_ms", lambda p: (p.get("audio") or {}).get("ui_sound_ms")),
    ("hit", "hit_beat_ms", lambda p: (p.get("audio") or {}).get("hit_beat_ms")),
    ("music", "music_ms", lambda p: (p.get("audio") or {}).get("music_ms")),
    ("underruns", "underruns", lambda p: (p.get("audio") or {}).get("underruns")),
]


def steps_of(scenario):
    """The feel and reach steps of a parsed scenario, in order: (kind, name, budget)."""
    return [(s["step"], s.get("name", ""), s.get("budget") or {}) for s in scenario.get("steps", []) if s.get("step") in KINDS]


def has_feel(path):
    try:
        return bool(steps_of(json.loads(Path(path).read_text(encoding="utf-8"))))
    except (OSError, ValueError):
        return False


def cell(v, budget):
    if v is None:
        return "-"
    over = budget is not None and isinstance(budget, (int, float)) and v > budget
    text = str(v) if isinstance(v, int) else f"{v:.1f}"
    return text + ("*" if over else "")


def rows_of(scenario, results):
    """The rows of one scenario: `results` are its result.json files (one per size). A step that ran gets its numbers,
    the step that failed gets the failure, the steps after it `not run`."""
    sid = scenario.get("id", "?")
    if not results:
        return [[f"{sid}/{name}", "not run"] + ["-"] * (len(COLUMNS) + 1) for _, name, _ in steps_of(scenario)]
    rows = []
    for res in results:
        size = res.get("size")
        tag = sid if len(results) == 1 else f"{sid}@{size}"
        perf = {p.get("name"): p for p in res.get("perf", []) if p.get("kind") in KINDS}
        failed = res.get("result") != "pass"
        for kind, name, budget in steps_of(scenario):
            p = perf.get(name)
            label = f"{tag}/{name}"
            if p is None:
                status = "not run"
                message = res.get("message", "")
                if failed and message.startswith(f"{kind} {name}"):
                    status = "FAIL: " + message[len(f"{kind} {name}"):].lstrip(": ")
                rows.append([label, status] + ["-"] * (len(COLUMNS) + 1))
                continue
            if kind == "reach":
                ends = p.get("dead_ends") or []
                rows.append([label, "ok"] + ["-"] * len(COLUMNS) + [str(len(ends))])
                continue
            status = "ok"
            if failed and res.get("message", "").startswith(f"feel {name}:"):
                status = "FAIL: " + res["message"][len(f"feel {name}: "):]
            rows.append([label, status] + [cell(get(p), budget.get(key) if key else None) for _, key, get in COLUMNS] + ["-"])
    return rows


def render(rows):
    head = ["scenario/step", "result"] + [c[0] for c in COLUMNS] + ["dead ends"]
    widths = [max(len(r[i]) for r in [head] + rows) for i in range(len(head))]
    widths[1] = min(widths[1], 48)
    line = lambda r: "  ".join((r[i][:48] if i == 1 else r[i]).ljust(widths[i]) if i < 2 else r[i].rjust(widths[i]) for i in range(len(head))).rstrip()
    return "\n".join([line(head), line(["-" * w for w in widths])] + [line(r) for r in rows])


def table(out, files):
    rows, bad, notes = [], 0, []
    for f in files:
        scenario = json.loads(Path(f).read_text(encoding="utf-8"))
        sid = scenario.get("id", "")
        results = [json.loads(p.read_text(encoding="utf-8")) for p in sorted((Path(out) / sid).glob("*/result.json"))]
        rows += rows_of(scenario, results)
        if not results:
            notes.append(f"{sid}: no result (it did not start)")
        for r in results:
            if r.get("result") != "pass":
                notes.append(f"{sid} {r.get('size', '')}: step {r.get('step')}: {r.get('message', '')}")
        bad += (not results) or any(r.get("result") != "pass" for r in results)
    ran = sum(1 for r in rows if r[1] != "not run")
    foot = f"{len(rows)} step(s), {ran} measured, {bad} scenario(s) failed or did not run; ms; * over the step's budget"
    return "\n".join([render(rows), "", foot] + notes), bad


def main(argv):
    if len(argv) >= 2 and argv[0] == "list":
        for d in argv[1:]:
            for f in sorted(Path(d).glob("*.json")):
                if has_feel(f):
                    print(f.as_posix())
        return 0
    if len(argv) >= 2 and argv[0] == "table":
        text, bad = table(argv[1], argv[2:])
        print(text)
        return 1 if bad else 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
