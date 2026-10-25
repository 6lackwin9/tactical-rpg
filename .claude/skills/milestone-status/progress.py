#!/usr/bin/env python3
# Task progress per milestone: reads the milestone map (games/<game>/docs/MILESTONES.md), the open queue
# (docs/TASKS.md) and the done archive (docs/archive/TASKS_DONE.md), and prints one Markdown section per milestone
# with done / in progress / queued / not started counts, broken down by system (the task id before its first '-'),
# lane and layer, plus the tasks in progress and the queued ones whose dependencies are all done.
#   python progress.py [--root DIR] [--game emberwake] [--milestone M1] [--all] [--json]
import argparse, fnmatch, json, pathlib, re, sys

STATES = ("done", "in progress", "queued", "not started")

def rows(path):
    """The task table rows of a Markdown file as dicts keyed by the header names (Id, Title, Layer, Lane, ...)."""
    out, head = [], None
    if not path.exists(): return out
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("| "): head = None if not line.strip() else head; continue
        cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
        if cells and cells[0] == "Id": head = cells; continue
        if head is None or set(line) <= set("|- "): continue
        if len(cells) < len(head): cells += [""] * (len(head) - len(cells))
        out.append(dict(zip(head, cells)))
    return out

def state_of(status):
    s = status.lower()
    if s.startswith("done") or s.startswith("covered") or s.startswith("folded"): return "done"
    if s.startswith(("in-progress", "in progress", "review")): return "in progress"
    if s.startswith("approved"): return "queued"
    if s.startswith(("spec", "idea", "proposed")): return "not started"
    return None  # obsolete, dropped, superseded: left out of the counts

def milestone_map(path):
    ms = []
    for r in rows_any(path, "Milestone"):
        pats = re.findall(r"`([^`]+)`", r.get("Tasks", ""))
        ms.append({"id": r["Milestone"], "goal": r.get("Goal", ""), "patterns": pats})
    return ms

def rows_any(path, first):
    out, head = [], None
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("| "): continue
        cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
        if cells[0] == first: head = cells; continue
        if head is None or set(line) <= set("|- "): continue
        out.append(dict(zip(head, cells)))
    return out

def assign(tid, status, ms):
    if "later than the slice" in status.lower() or re.search(r"\bM2b\b", status):
        return next((m["id"] for m in ms if m["id"] == "M2b"), None)
    for exact in (True, False):
        for m in ms:
            for p in m["patterns"]:
                if ("*" in p) != (not exact): continue
                if (p == tid) if exact else fnmatch.fnmatchcase(tid, p): return m["id"]
    return None

def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=str(pathlib.Path(__file__).resolve().parents[3]))
    ap.add_argument("--game", default="emberwake")
    ap.add_argument("--milestone"); ap.add_argument("--all", action="store_true"); ap.add_argument("--json", action="store_true")
    a = ap.parse_args(); root = pathlib.Path(a.root)
    ms = milestone_map(root / "games" / a.game / "docs" / "MILESTONES.md")
    tasks = {}
    for src in (root / "docs/archive/TASKS_DONE.md", root / "docs/TASKS.md"):  # the open queue wins on a repeated id
        for r in rows(src):
            tid = r.get("Id", "").split(" ")[0]
            if re.fullmatch(r"[A-Z][A-Z0-9]*(-[A-Za-z0-9]+)*", tid): tasks[tid] = r
    done_ids = {t for t, r in tasks.items() if state_of(r.get("Status", "")) == "done"}
    report = {}
    for tid, r in tasks.items():
        st = state_of(r.get("Status", ""))
        if st is None: continue
        mid = assign(tid, r.get("Status", ""), ms) or ("outside" if a.all else None)
        if mid is None: continue
        sysid = tid.split("-")[0]
        lane = (r.get("Lane", "") or "?").split(" ")[0].split(",")[0] or "?"
        layer = (r.get("Layer", "") or "not recorded").split(" (")[0] or "not recorded"
        deps = [d for d in re.findall(r"[A-Z][A-Z0-9]*(?:-[A-Za-z0-9]+)+|[A-Z][A-Z0-9]{2,}\d", r.get("Depends on", "")) if d in tasks]
        report.setdefault(mid, []).append({"id": tid, "state": st, "system": sysid, "lane": lane, "layer": layer,
                                           "title": r.get("Title", ""), "ready": st == "queued" and all(d in done_ids for d in deps)})
    order = [m["id"] for m in ms] + ["outside"]
    goals = {m["id"]: m["goal"] for m in ms}; goals["outside"] = "Tasks outside the slice milestones"
    pick = [m for m in order if m in report and (not a.milestone or m == a.milestone)]
    if a.json: print(json.dumps({m: report[m] for m in pick}, indent=1)); return
    for m in pick:
        ts = report[m]; n = len(ts); d = sum(t["state"] == "done" for t in ts)
        bar = "█" * round(10 * d / n) + "░" * (10 - round(10 * d / n))
        print(f"## {m}: {d} of {n} done ({round(100 * d / n)}%) {bar}\n\n{goals[m]}\n")
        print("| System | Done | In progress | Queued | Not started | Total |\n| --- | --- | --- | --- | --- | --- |")
        for sysid in sorted({t["system"] for t in ts}, key=lambda s: (-sum(t["system"] == s for t in ts), s)):
            g = [t for t in ts if t["system"] == sysid]
            print(f"| {sysid} | " + " | ".join(str(sum(t["state"] == s for t in g)) for s in STATES) + f" | {len(g)} |")
        print("| **All** | " + " | ".join(f"**{sum(t['state'] == s for t in ts)}**" for s in STATES) + f" | **{n}** |\n")
        for key, label in (("lane", "By lane"), ("layer", "By layer")):
            parts = [f"{v} {sum(t[key] == v and t['state'] == 'done' for t in ts)}/{sum(t[key] == v for t in ts)}"
                     for v in sorted({t[key] for t in ts})]
            print(f"{label} (done/total): " + ", ".join(parts) + "\n")
        prog = [t["id"] for t in ts if t["state"] == "in progress"]
        ready = [t["id"] for t in ts if t["ready"]]
        print("In progress: " + (", ".join(prog) or "none") + "\n")
        print("Ready to start (all dependencies done): " + (", ".join(ready) or "none") + "\n")

if __name__ == "__main__":
    sys.exit(main())
