"""Lists the docs a task's change should update but does not (skill task-landing, section 1).

Run from a task worktree: python .claude/skills/task-landing/docs_needed.py [base]
base defaults to `staging` (the batched-gate branch); the diff is base...HEAD plus uncommitted changes.
Prints one line per missing doc, or "docs look complete". Advisory: a line can be justified and skipped.
"""
import re
import subprocess
import sys

base = sys.argv[1] if len(sys.argv) > 1 else "staging"


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, encoding="utf-8").stdout


changed = set(git("diff", "--name-only", f"{base}...HEAD").split())
changed |= set(git("diff", "--name-only", "HEAD").split())
changed |= {l[3:] for l in git("status", "--porcelain").splitlines() if l.startswith("??")}
added = set(git("diff", "--name-only", "--diff-filter=A", f"{base}...HEAD").split())
added |= {l[3:] for l in git("status", "--porcelain").splitlines() if l.startswith("??")}

need = []


def touched(path):
    return path in changed


def want(path, why):
    if not touched(path):
        need.append(f"{path}: {why}")


build = [p for p in changed if re.search(r"(^|/)target(-[^/]*)?/", p)]
if build:
    need.append(f"BUILD FOLDER in the change ({build[0]} ...): git rm -r --cached it before committing")

code = [p for p in changed if p.endswith((".rs", ".py")) and not p.startswith("legacy/")]
if code or any(p.startswith(("games/emberwake/data/", "kits/rpg/assets/", "games/emberwake/assets/")) for p in changed):
    want("docs/TASKS.md", "the task row's status (done / in-progress with the next slice)")
if any(p.startswith("engine/crates/") and "/src/" in p for p in changed):
    want("engine/README.md", "engine API changed: the crate's row or bullet")
    want("engine/docs/PORT_NOTES.md", "engine code changed: a `none` row for engine-only work or the ported Godot file")
if any(p.startswith("kits/rpg/crates/") and "/src/" in p for p in changed):
    want("kits/rpg/README.md", "kit code changed: one line for the hook or system")
new_src = [p for p in added if re.search(r"/(src|tests)/[^/]+\.rs$", p) or p.startswith("tools/")]
if new_src:
    want("docs/STRUCTURE.md", f"new file(s) {', '.join(sorted(new_src)[:3])}: the folder's row")
if any(p.startswith(("games/emberwake/crates/", "kits/rpg/crates/")) and "/src/" in p for p in changed):
    want("games/emberwake/docs/GAME_BRIEF.md", "game-visible system: a 'Where to look' line (skip if none is new)")
if any(p.startswith("games/emberwake/data/seeds/") for p in changed):
    want("games/emberwake/data/seeds/README.md", "a seed changed: its row")
if any(p.startswith(("kits/rpg/assets/units/", "games/emberwake/assets/units/", "tools/sprites/")) for p in changed):
    want("games/emberwake/assets/README.md", "unit art or the rig changed: the exporter or units note")
art = [p for p in changed if p.startswith("art_src/") and p.endswith((".png", ".webp"))]
for p in art:
    rec = re.sub(r"\.(png|webp)$", ".json", p)
    if rec not in changed and not git("ls-files", rec).strip():
        need.append(f"{rec}: generated art needs its prompt record")
if any("tests/fixtures/" in p and p.endswith(".json") for p in changed):
    want("docs/HISTORY.md", "a recorded golden changed: name the re-record (or say why not)")
specs = [p for p in changed if re.search(r"docs/specs/.*\.md$", p)]
if code and not specs:
    need.append("the task's spec: an 'As built' note if the build differs from the spec (skip if it matches)")
new_rows = re.findall(r"^\+\| ([A-Z][A-Z0-9]*(?:-[A-Za-z0-9]+)+) \|", git("diff", f"{base}...HEAD", "--", "docs/TASKS.md"), re.M)
old_ids = set(re.findall(r"^\| ([A-Z][A-Z0-9]*(?:-[A-Za-z0-9]+)+) \|", git("show", f"{base}:docs/TASKS.md"), re.M))
fresh = [i for i in new_rows if i not in old_ids]
if fresh:
    want("games/emberwake/docs/MILESTONES.md", f"new task row(s) {', '.join(fresh)}: add the id or pattern to a milestone")

print("\n".join(need) if need else "docs look complete")
