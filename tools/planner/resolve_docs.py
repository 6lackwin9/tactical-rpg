# Resolves rebase conflicts in planner docs where two tasks touched the same table rows or inserted rows at the same
# spot: a row edited on both sides keeps every "; "-separated clause of both (HEAD's order first); rows inserted at the
# same spot are all kept. Code files are never passed here.  python resolve_docs.py <file.md> ...
import re, sys

PAT = re.compile(r"<<<<<<< [^\n]*\n(.*?)=======\n(.*?)>>>>>>> [^\n]*\n", re.S)

def merge_row(h, o):
    hc, oc = h.split(" | "), o.split(" | ")
    if len(hc) != len(oc) or hc[0] != oc[0]: return None
    out = []
    for a, b in zip(hc, oc):
        ra, rb = rank(a), rank(b)
        if a != b and (ra or rb):   # a status cell: the more advanced side wins whole (2026-10-29: a merged
            # a tie keeps the longer text: a later status note extends the earlier one (2026-10-29: "slice 1 done" lost
            # to "slice 1 building", then "slice 2 done" lost to a stale "slice 1 done" from another task's branch)
            out.append(a if (ra, len(a)) >= (rb, len(b)) else b); continue   # "approved ...; done ..." read as approved)
        at, bt = a.split("; "), b.split("; ")
        out.append(a if a == b else "; ".join(at + [t for t in bt if t not in at]))
    return " | ".join(out)

STATUS = ["idea", "proposed", "spec", "approved", "in-progress", "review", "done"]
def rank(cell):
    w = cell.strip().split(" ")[0].lower() if cell.strip() else ""
    return STATUS.index(w) + 1 if w in STATUS else 0

def res(m):
    hl, ol = m.group(1).splitlines(), m.group(2).splitlines()
    if len(hl) == len(ol):
        rows = [merge_row(a, b) for a, b in zip(hl, ol)]
        if all(rows): return "\n".join(rows) + "\n"
    # Tables whose row counts differ: merge rows with the same first cell (a task id) instead of keeping both copies
    # (2026-10-29: duplicated DIR4-R1 and WAR2H1 rows), append the rest.
    out = list(hl)
    for l in ol:
        if l in out:
            continue
        key = l.split(" | ")[0] if l.startswith("| ") else None
        j = next((k for k, x in enumerate(out) if key and x.split(" | ")[0] == key), None)
        merged = merge_row(out[j], l) if j is not None else None
        if merged:
            out[j] = merged
        else:
            out.append(l)
    return "\n".join(out) + "\n"

for p in sys.argv[1:]:
    assert p.endswith(".md"), p
    s = open(p, encoding="utf-8").read(); t = PAT.sub(res, s)
    assert "<<<<<<<" not in t and ">>>>>>>" not in t, p
    if p.replace("\\", "/").endswith("docs/TASKS.md"):
        # 2026-10-29: a merge against a side that moved rows to TASKS_DONE kept both copies, reviving 19 done rows;
        # a task id already archived as done never stays in TASKS
        done_p = p.replace("\\", "/")[: -len("TASKS.md")] + "archive/TASKS_DONE.md"
        done = set(re.findall(r"^\| ([A-Z][A-Z0-9-]*) \|", open(done_p, encoding="utf-8").read(), re.M))
        keep = [l for l in t.split("\n") if not (m := re.match(r"^\| ([A-Z][A-Z0-9-]*) \|", l)) or m.group(1) not in done]
        if len(keep) != len(t.split("\n")):
            print("dropped", len(t.split("\n")) - len(keep), "rows already in TASKS_DONE")
        # 2026-10-30: rows with the same id but different cells (an `approved` copy and a task's `done` copy) both
        # survived; one row per id, at its first position, carrying the most advanced status (the longer on a tie)
        first, out = {}, []
        for l in keep:
            m = re.match(r"^\| ([A-Z][A-Z0-9-]*) \|", l)
            if not m:
                out.append(l); continue
            if m.group(1) not in first:
                first[m.group(1)] = len(out); out.append(l); continue
            i = first[m.group(1)]
            def best(row):
                return max(((rank(c), len(c)) for c in row.split(" | ")), default=(0, 0))
            if best(l) > best(out[i]):
                out[i] = l
            print("merged duplicate row", m.group(1))
        t = "\n".join(out)
    open(p, "w", encoding="utf-8", newline="\n").write(t); print("resolved", p)
