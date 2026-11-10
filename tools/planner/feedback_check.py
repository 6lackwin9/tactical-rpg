#!/usr/bin/env python3
"""Checks docs/FEEDBACK.md against docs/READY.md section 4, and archives answered items (SKILLS1-FB, UN-05).

Usage (from anywhere; paths default to this checkout):
  python3 tools/planner/feedback_check.py [check] [--pending-ok]
  python3 tools/planner/feedback_check.py archive [--commit <sha>] [--dry-run]
  python3 tools/planner/feedback_check.py lease [check|claim|release] [--window-minutes 20] [--session auto|cloud|desktop] [--force] [--no-fetch]
  common options: --feedback <path> --ready <path> --closed <path>

check (exit 0 = FEEDBACK.md and READY.md match, 1 = a mismatch, 2 = a file could not be read or parsed):
  - every open FB-nn / UN-nn item in FEEDBACK.md has a row in READY.md section 4 (with --pending-ok an item with no row
    yet is listed as waiting for Design, not a failure: the build session's start-up pass);
  - every section 4 row has a Disposition of accepted, deferred or policy and a non-empty Follow-up;
  - every Follow-up other than `none` names a task id found in the Task ID column of READY.md section 1 or 3
    (a range such as `U4.2-S3..S4` also matches `U4.2-S3` and `U4.2-S4`; a follow-up range passes when every id in it
    is there); a Feedback ID cell such as `FB-48a` or `FB-04..05` is a parse error (one row per plain id);
  - a section 4 row whose item is neither in FEEDBACK.md nor in the archive is a warning.

archive: runs the check, then moves every item of FEEDBACK.md that has a valid section 4 row into
docs/archive/FEEDBACK_CLOSED.md under one new heading, its Status line rewritten to
`closed: <disposition> (@<commit>)`, plus `, follow-up <ids>` when the Follow-up is not `none`. The commit defaults to
the last commit that changed READY.md (8 characters). It refuses when the check has errors other than waiting items.
Items without a row stay. It does not commit.

Headings inside a fenced code block (``` or ~~~) are text, not item or section boundaries (FB-169), so an item quoting
Markdown moves whole; a fence in FEEDBACK.md that never closes is a parse error (exit 2).
"""
from __future__ import annotations

import argparse
import datetime
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DISPOSITIONS = ("accepted", "deferred", "policy")
ITEM_ID = re.compile(r"^(FB|UN)-\d+$")
HEADER = re.compile(r"^### ((?:FB|UN)-\d+)\b")
STATUS = re.compile(r"^- \*\*Status\*\*:\s*(.*)$")
TICKED = re.compile(r"`([^`]+)`")
RANGE = re.compile(r"^(.*-[A-Z]+)(\d+)\.\.[A-Z]*(\d+)$")  # U4.2-S3..S4 -> U4.2-S3, U4.2-S4
CLOSED_SUBJECT = re.compile(r"^Close .*squash-landed on develop", re.IGNORECASE)
BUILDING_STATUS = re.compile(r"\b(building|in-progress|in progress|running)\b", re.IGNORECASE)
FENCE = re.compile(r"^\s*(`{3,}|~{3,})(.*)$")  # a fenced code block line: ``` or ~~~ (3 or more), then an info string


class ParseError(Exception):
    pass


@dataclass
class Item:
    id: str
    status: str
    start: int  # line index of the ### header in FEEDBACK.md
    end: int  # one past the item's last line

    @property
    def open(self) -> bool:
        return not self.status.lower().startswith("closed")


@dataclass
class Answer:
    id: str
    disposition: str
    follow_up: list[str]  # task ids, or ["none"]
    line: int  # 1-based line in READY.md


@dataclass
class Report:
    errors: list[str] = field(default_factory=list)
    waiting: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    archivable: list[str] = field(default_factory=list)


def outside_fences(lines: list[str]) -> tuple[list[bool], int | None]:
    """Per line: True when it is ordinary text, False on a fence line or inside a fenced code block (FB-169), plus the
    line index of a fence still open at the end (None when every fence closes).

    CommonMark rule: a fence opens with 3 or more ` or ~ (a backtick fence's info string has no backtick) and closes
    with the same character, at least as many, and nothing but spaces after. Any indent counts, so a fence nested in a
    list item is seen too. A `## ` line inside a fence quotes Markdown; it is not a heading.
    """
    mask: list[bool] = []
    fence: tuple[str, int, int] | None = None  # (character, length, opening line index)
    for i, line in enumerate(lines):
        m = FENCE.match(line)
        if fence is None:
            if m and not (m.group(1)[0] == "`" and "`" in m.group(2)):
                fence = (m.group(1)[0], len(m.group(1)), i)
                mask.append(False)
            else:
                mask.append(True)
        else:
            mask.append(False)
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= fence[1] and not m.group(2).strip():
                fence = None
    return mask, (fence[2] if fence else None)


def read_items(lines: list[str]) -> list[Item]:
    """The ### FB-nn / UN-nn blocks: from the header to the next ### or ## header (headings inside a fenced code block
    do not count, FB-169; a fence left open is a parse error, since it would swallow every item after it)."""
    text, unclosed = outside_fences(lines)
    if unclosed is not None:
        raise ParseError(f"FEEDBACK.md:{unclosed + 1}: a fenced code block opens here and never closes")
    items: list[Item] = []
    for i, line in enumerate(lines):
        if not text[i]:
            continue
        if items and items[-1].end == len(lines) and line.startswith(("## ", "### ")):
            items[-1].end = i
        m = HEADER.match(line)
        if m:
            items.append(Item(m.group(1), "", i, len(lines)))
    for it in items:
        for k, line in enumerate(lines[it.start:it.end], it.start):
            s = STATUS.match(line) if text[k] else None
            if s:
                it.status = s.group(1).strip()
                break
        if not it.status:
            raise ParseError(f"FEEDBACK item {it.id} (line {it.start + 1}) has no '- **Status**:' line")
    # trailing blank lines belong to the gap, not the item
    for it in items:
        while it.end > it.start + 1 and not lines[it.end - 1].strip():
            it.end -= 1
    return items


def sections(lines: list[str]) -> dict[str, tuple[int, int]]:
    """READY.md '## <n>. ...' sections by number: (first line index, end index)."""
    found: list[tuple[str, int]] = []
    text = outside_fences(lines)[0]
    for i, line in enumerate(lines):
        m = re.match(r"^## (\d+)\.", line) if text[i] else None
        if m:
            found.append((m.group(1), i))
    out: dict[str, tuple[int, int]] = {}
    for k, (num, start) in enumerate(found):
        out[num] = (start, found[k + 1][1] if k + 1 < len(found) else len(lines))
    return out


def table(lines: list[str], start: int, end: int) -> tuple[list[str], list[tuple[int, list[str]]]]:
    """The first Markdown table in a range: its header cells and its rows (line index, cells)."""
    header: list[str] = []
    rows: list[tuple[int, list[str]]] = []
    for i in range(start, end):
        line = lines[i].strip()
        if not line.startswith("|"):
            if header:
                break
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if not header:
            header = cells
        elif all(re.fullmatch(r":?-+:?", c) for c in cells):
            continue
        else:
            rows.append((i, cells))
    return header, rows


def column(header: list[str], name: str, where: str) -> int:
    for k, h in enumerate(header):
        if h.lower() == name.lower():
            return k
    raise ParseError(f"READY.md {where}: no '{name}' column (columns: {', '.join(header) or 'none'})")


def expand(task: str) -> set[str]:
    out = {task}
    m = RANGE.match(task)
    if m:
        lo, hi = int(m.group(2)), int(m.group(3))
        out.update(f"{m.group(1)}{n}" for n in range(lo, hi + 1))
    return out


def task_ids(lines: list[str], secs: dict[str, tuple[int, int]]) -> set[str]:
    ids: set[str] = set()
    for num in ("1", "3"):
        if num not in secs:
            raise ParseError(f"READY.md has no section {num}")
        header, rows = table(lines, *secs[num])
        col = column(header, "Task ID", f"section {num}")
        for _, cells in rows:
            for t in TICKED.findall(cells[col] if col < len(cells) else ""):
                ids |= expand(t.strip())
    return ids


def read_answers(lines: list[str], secs: dict[str, tuple[int, int]]) -> list[Answer]:
    if "4" not in secs:
        raise ParseError("READY.md has no section 4")
    header, rows = table(lines, *secs["4"])
    c_id = column(header, "Feedback ID", "section 4")
    c_disp = column(header, "Disposition", "section 4")
    c_fu = column(header, "Follow-up", "section 4")
    out: list[Answer] = []
    for i, cells in rows:
        get = lambda c: cells[c] if c < len(cells) else ""  # noqa: E731
        ids = [t for t in TICKED.findall(get(c_id)) if ITEM_ID.match(t)]
        odd = [t for t in TICKED.findall(get(c_id)) if re.match(r"^(FB|UN)-", t) and not ITEM_ID.match(t)]
        if odd:
            raise ParseError(f"READY.md:{i + 1}: Feedback ID {', '.join(odd)} is not one plain id (write one row per `FB-nn`)")
        if not ids:
            continue
        disp = (TICKED.findall(get(c_disp)) or [get(c_disp)])[0].strip()
        fu = [t.strip() for t in TICKED.findall(get(c_fu))] or ([get(c_fu).strip()] if get(c_fu).strip() else [])
        for item_id in ids:
            out.append(Answer(item_id, disp, fu, i + 1))
    return out


def check(feedback: str, ready: str, closed: str, pending_ok: bool) -> tuple[Report, list[Item], dict[str, Answer]]:
    fb_lines = feedback.splitlines()
    rd_lines = ready.splitlines()
    items = read_items(fb_lines)
    secs = sections(rd_lines)
    tasks = task_ids(rd_lines, secs)
    answers = read_answers(rd_lines, secs)
    cl_lines = closed.splitlines()
    cl_text = outside_fences(cl_lines)[0]
    archived = {m.group(1) for m in (HEADER.match(line) for line, t in zip(cl_lines, cl_text) if t) if m}
    rep = Report()
    by_id: dict[str, Answer] = {}
    for a in answers:
        if a.id in by_id:
            rep.errors.append(f"READY.md:{a.line}: {a.id} has a second section 4 row (first on line {by_id[a.id].line})")
            continue
        by_id[a.id] = a
        bad = False
        if a.disposition not in DISPOSITIONS:
            rep.errors.append(f"READY.md:{a.line}: {a.id} Disposition '{a.disposition}' is not one of {', '.join(DISPOSITIONS)}")
            bad = True
        if not a.follow_up:
            rep.errors.append(f"READY.md:{a.line}: {a.id} has an empty Follow-up (write `none` or a task id)")
            bad = True
        elif a.follow_up != ["none"]:
            for t in a.follow_up:
                if t == "none":
                    rep.errors.append(f"READY.md:{a.line}: {a.id} Follow-up mixes `none` with task ids")
                    bad = True
                elif t not in tasks and not (expand(t) - {t} and expand(t) - {t} <= tasks):
                    rep.errors.append(f"READY.md:{a.line}: {a.id} Follow-up `{t}` is not a task id in READY.md section 1 or 3")
                    bad = True
        if bad:
            by_id[a.id] = Answer(a.id, "", [], a.line)  # invalid: never archived
    in_feedback = {it.id for it in items}
    for it in items:
        a = by_id.get(it.id)
        if a is None:
            if it.open:
                msg = f"FEEDBACK.md:{it.start + 1}: {it.id} is open and has no row in READY.md section 4"
                (rep.waiting if pending_ok else rep.errors).append(msg)
        elif a.disposition:
            rep.archivable.append(it.id)
    for item_id, a in by_id.items():
        if item_id not in in_feedback and item_id not in archived:
            rep.warnings.append(f"READY.md:{a.line}: {item_id} is in neither FEEDBACK.md nor the archive")
    return rep, items, by_id


def last_commit(path: Path) -> str:
    try:
        out = subprocess.run(["git", "log", "-1", "--format=%H", "--", str(path)], cwd=path.parent,
                             capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as e:
        raise ParseError(f"could not read the last commit of {path}: {e}") from e
    if not out:
        raise ParseError(f"{path} has no commit yet; pass --commit")
    return out[:8]


def archive(feedback: str, closed: str, items: list[Item], answers: dict[str, Answer], ids: list[str],
            commit: str, today: str) -> tuple[str, str]:
    """Returns the new FEEDBACK.md and FEEDBACK_CLOSED.md texts."""
    fb_lines = feedback.splitlines()
    text = outside_fences(fb_lines)[0]
    move = [it for it in items if it.id in ids]
    blocks: list[str] = []
    drop: set[int] = set()
    for it in move:
        a = answers[it.id]
        status = f"closed: {a.disposition} (@{commit})"
        if a.follow_up != ["none"]:
            status += ", follow-up " + ", ".join(f"`{t}`" for t in a.follow_up)
        block = fb_lines[it.start:it.end]
        # only the item's own Status line (the first one outside a fence), not a quoted one inside a code block
        k = next(k for k in range(it.start, it.end) if text[k] and STATUS.match(fb_lines[k]))
        block[k - it.start] = f"- **Status**: {status}"
        blocks.append("\n".join(block))
        # the item plus the blank lines after it
        end = it.end
        while end < len(fb_lines) and not fb_lines[end].strip():
            end += 1
        drop.update(range(it.start, end))
    kept = [line for i, line in enumerate(fb_lines) if i not in drop]
    new_fb = "\n".join(kept).rstrip("\n") + "\n"
    names = ", ".join(it.id for it in move)
    heading = f"## {names} (answered in READY @{commit}, moved {today})"
    new_closed = closed.rstrip("\n") + "\n\n" + heading + "\n\n" + "\n\n".join(blocks) + "\n"
    return new_fb, new_closed


def parse_building_rows(feedback_text: str) -> list[str]:
    """Finds rows in FEEDBACK.md Section 1 whose Stage / Status cell indicates an active build."""
    in_sec1 = False
    rows: list[str] = []
    lines = feedback_text.splitlines()
    for line, plain in zip(lines, outside_fences(lines)[0]):
        if not plain:
            continue
        if line.startswith("## 1."):
            in_sec1 = True
            continue
        if in_sec1 and line.startswith("## "):
            break
        if not in_sec1 or not line.strip().startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 2 or cells[0].lower().startswith("task") or all(set(c) <= {"-", ":"} for c in cells):
            continue
        if BUILDING_STATUS.search(cells[1]):
            rows.append(f"{cells[0]} ({cells[1]})")
    return rows


LEASE_REF = "refs/heads/build-lease"
LEASE_SUBJ = re.compile(r"^BUILD-LEASE:\s*(cloud|desktop)\b", re.IGNORECASE)


def check_lease(action: str, window_minutes: float, session: str, force: bool, no_fetch: bool) -> int:
    """Checks, claims, or releases the remote Build Session lease (`refs/heads/build-lease` + branch/FEEDBACK check)."""
    def git_out(*args: str) -> str:
        r = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
        if r.returncode != 0:
            raise ParseError(f"git {' '.join(args)} failed: {r.stderr.strip()}")
        return r.stdout

    if session == "auto":
        session = "cloud" if os.environ.get("CLAUDE_CODE_REMOTE", "").lower() == "true" else "desktop"
    if not no_fetch:
        git_out("fetch", "-p", "-q", "origin")
    now = int(time.time())

    # 1. Explicit remote lease ref (`origin/build-lease`)
    lease_owner = ""
    lease_age = 999999.0
    lease_subj = ""
    r_lease = subprocess.run(
        ["git", "log", "-1", "--format=%h|%ct|%s", "origin/build-lease"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if r_lease.returncode == 0 and r_lease.stdout.strip():
        _, l_ts, lease_subj = r_lease.stdout.strip().split("|", 2)
        lease_age = max(0.0, (now - int(l_ts)) / 60.0)
        m = LEASE_SUBJ.match(lease_subj)
        if m:
            lease_owner = m.group(1).lower()

    if action == "release":
        subprocess.run(["git", "push", "-q", "origin", f":{LEASE_REF}"], cwd=ROOT, capture_output=True)
        print(f"[build-lease] released origin/build-lease (session={session})")
        return 0

    # 2. Unmerged remote claude/* task branches
    merged = {
        b.strip()
        for b in git_out("branch", "-r", "--merged", "origin/develop").splitlines()
        if b.strip() and "->" not in b
    }
    ref_out = git_out(
        "for-each-ref",
        "--format=%(refname:short)|%(objectname:short)|%(committerdate:unix)|%(subject)",
        "refs/remotes/origin/claude/",
    )
    active_branches: list[tuple[str, str, float, str]] = []
    for raw in ref_out.splitlines():
        parts = raw.split("|", 3)
        if len(parts) < 4:
            continue
        branch, sha, ts_str, subject = parts[0], parts[1], parts[2], parts[3]
        if branch in merged or CLOSED_SUBJECT.search(subject) or branch.startswith("origin/claude/release-batch"):
            continue
        age_min = max(0.0, (now - int(ts_str)) / 60.0)
        active_branches.append((branch, sha, age_min, subject))

    # 3. origin/develop FEEDBACK.md heartbeat
    log_line = (
        git_out("log", "-1", "--format=%h|%ct|%s", "origin/develop", "--", "docs/FEEDBACK.md").strip()
        or git_out("log", "-1", "--format=%h|%ct|%s", "origin/develop").strip()
    )
    dev_sha, dev_ts, dev_subj = log_line.split("|", 2)
    dev_age = max(0.0, (now - int(dev_ts)) / 60.0)
    building = parse_building_rows(git_out("show", "origin/develop:docs/FEEDBACK.md"))

    print(f"[build-lease] session={session} | origin/develop={dev_sha} ({dev_age:.1f}m ago) '{dev_subj}'")
    if lease_owner:
        print(f"[build-lease] lease-owner={lease_owner} ({lease_age:.1f}m ago) '{lease_subj}'")
    if building:
        print(f"[build-lease] FEEDBACK.md active rows: {', '.join(building)}")
    for b, sha, age, subj in active_branches:
        print(f"[build-lease] unmerged branch: {b} ({sha}, {age:.1f}m ago) '{subj}'")

    conflicts: list[str] = []
    if lease_owner and lease_owner != session:
        conflicts.append(
            f"remote lease `origin/build-lease` is owned by `{lease_owner}` ({lease_age:.1f}m ago): {lease_subj}"
        )
    if not force and lease_owner != session:
        for b, sha, age, subj in active_branches:
            if age <= window_minutes:
                conflicts.append(f"unmerged task branch `{b}` ({sha}, {age:.1f}m ago <= {window_minutes:.0f}m): {subj}")
        if dev_age <= window_minutes and building:
            conflicts.append(
                f"origin/develop FEEDBACK.md updated {dev_age:.1f}m ago ({dev_sha}) with active build status: {', '.join(building)}"
            )

    if conflicts and not force:
        print("[build-lease] CONFLICT: another Build Session holds the lease or is active within the window:")
        for c in conflicts:
            print(f"  - {c}")
        print("[build-lease] HALT: do not build or push. Pass `lease claim --force` only when the user confirms takeover.")
        return 1

    if action == "claim":
        tree = git_out("rev-parse", "origin/develop^{tree}").strip()
        parent = git_out("rev-parse", "origin/develop").strip()
        msg = f"BUILD-LEASE: {session} claimed build lease at {datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}"
        commit = subprocess.run(
            ["git", "commit-tree", tree, "-p", parent, "-m", msg],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
        git_out("push", "-f", "-q", "origin", f"{commit}:{LEASE_REF}")
        print(f"[build-lease] claimed `origin/build-lease` for session={session} (@{commit[:8]})")
        return 0

    print("[build-lease] ok: no conflicting active Build Session.")
    return 0


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("command", nargs="?", default="check", choices=("check", "archive", "lease"))
    p.add_argument("lease_action", nargs="?", default="check", choices=("check", "claim", "release"))
    p.add_argument("--feedback", type=Path, default=ROOT / "docs/FEEDBACK.md")
    p.add_argument("--ready", type=Path, default=ROOT / "docs/READY.md")
    p.add_argument("--closed", type=Path, default=ROOT / "docs/archive/FEEDBACK_CLOSED.md")
    p.add_argument("--pending-ok", action="store_true", help="open items with no section 4 row yet are not a failure")
    p.add_argument("--commit", help="archive: the commit recorded as @<commit> (default: last commit of READY.md)")
    p.add_argument("--dry-run", action="store_true", help="archive: print what would move, change nothing")
    p.add_argument("--window-minutes", type=float, default=20.0, help="lease: activity window in minutes (default: 20)")
    p.add_argument("--session", default="auto", choices=("auto", "cloud", "desktop"))
    p.add_argument("--force", action="store_true", help="lease claim: force takeover from the other session")
    p.add_argument("--no-fetch", action="store_true", help="lease: skip git fetch -p origin")
    args = p.parse_args(argv)
    if args.command == "lease":
        try:
            return check_lease(args.lease_action, args.window_minutes, args.session, args.force, args.no_fetch)
        except ParseError as e:
            print(f"[build-lease] ERROR: {e}")
            return 2
    try:
        texts = [f.read_text(encoding="utf-8") for f in (args.feedback, args.ready, args.closed)]
        rep, items, answers = check(*texts, pending_ok=args.pending_ok or args.command == "archive")
    except (OSError, ParseError) as e:
        print(f"[feedback] ERROR: {e}")
        return 2
    for w in rep.waiting:
        print(f"[feedback] waiting for Design: {w}")
    for w in rep.warnings:
        print(f"[feedback] WARN: {w}")
    for e in rep.errors:
        print(f"[feedback] FAIL: {e}")
    if args.command == "check":
        if rep.archivable:
            print(f"[feedback] answered, ready to archive: {', '.join(rep.archivable)}")
        if rep.errors:
            return 1
        print(f"[feedback] ok: {len(items)} item(s) in FEEDBACK.md, {len(answers)} answer row(s) in READY.md section 4")
        return 0
    if rep.errors:
        print("[feedback] archive refused: fix the errors above first")
        return 1
    if not rep.archivable:
        print("[feedback] nothing to archive")
        return 0
    if not args.commit and subprocess.run(["git", "diff", "--quiet", "HEAD", "--", str(args.ready)], cwd=args.ready.parent,
                                          capture_output=True).returncode == 1:
        print("[feedback] WARN: READY.md has uncommitted edits; the recorded commit predates them (commit READY.md first, "
              "or pass --commit)")
    try:
        commit = args.commit or last_commit(args.ready)
    except ParseError as e:
        print(f"[feedback] ERROR: {e}")
        return 2
    new_fb, new_closed = archive(texts[0], texts[2], items, answers, rep.archivable, commit,
                                 datetime.date.today().isoformat())
    print(f"[feedback] {'would move' if args.dry_run else 'moved'} {', '.join(rep.archivable)} to the archive (@{commit})")
    if not args.dry_run:
        for path, text in ((args.feedback, new_fb), (args.closed, new_closed)):
            with open(path, "w", encoding="utf-8", newline="\n") as f:  # LF on Windows too (.gitattributes eol=lf)
                f.write(text)
        print("[feedback] update the 'FB-.. / UN-.. are closed' lines in FEEDBACK.md by hand if the range moved")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
