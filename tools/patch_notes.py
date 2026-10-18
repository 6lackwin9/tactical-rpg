"""List develop commits not yet in PATCH_NOTES.md, grouped by date (newest first).

Usage: python3 tools/patch_notes.py [ref]   (default ref: origin/develop)
Prints a draft for the patch-notes skill: the agent writes the plain-language
"In the game" lines from it and pastes the "Under the hood" list as is, then
updates the `<!-- last: <sha> -->` marker at the top of PATCH_NOTES.md.
"""
import re, subprocess, sys
from collections import OrderedDict

ref = sys.argv[1] if len(sys.argv) > 1 else 'origin/develop'
notes = open('PATCH_NOTES.md', encoding='utf-8').read()
m = re.search(r'<!-- last: ([0-9a-f]+) -->', notes)
rng = f'{m.group(1)}..{ref}' if m else ref
log = subprocess.run(['git', 'log', '--first-parent', '--format=%h%x09%cs%x09%s%x09%b%x1e', rng],
                     capture_output=True, text=True, check=True).stdout
# Commits that change no player-visible behaviour: workflow, docs, specs, handovers, metrics.
INTERNAL = re.compile(r'HANDOVER|USAGE|RELEASE|TOKENS|SPEC|APPROVE|ANSWERS|DECISION|DOCS|TASKS|ROADMAP|WORKFLOW|AGENT|METRIC|PATCH-NOTES|^Merge ', re.I)
days: 'OrderedDict[str, dict]' = OrderedDict()
head = ''
for rec in log.split('\x1e'):
    rec = rec.strip()
    if not rec:
        continue
    sha, day, subj, body = (rec.split('\t', 3) + [''])[:4]
    head = head or sha
    if subj.startswith('Merge '):
        continue
    t = re.search(r'type=(\w+)', body)
    kind = t.group(1) if t else ''
    internal = bool(INTERNAL.search(subj.split(':')[0])) or kind in ('docs', 'tool', 'spec')
    d = days.setdefault(day, {'game': [], 'hood': []})
    d['hood' if internal else 'game'].append(f'- {subj} ({sha})')
if not days:
    print('No new commits since', m.group(1) if m else 'start'); sys.exit(0)
print(f'<!-- last: {head} -->  (new marker)\n')
for day, d in days.items():
    print(f'## {day}\n')
    if d['game']:
        print('Candidates for "In the game" (rewrite in plain words, player-visible only):\n')
        print('\n'.join(d['game']) + '\n')
    if d['hood']:
        print('Under the hood:\n')
        print('\n'.join(d['hood']) + '\n')
