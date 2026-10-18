"""Efficiency report: project usage report rows vs task commits (Timing lines) per thread session.

Usage: python3 efficiency.py <repo> <report.csv> <report time ISO> [out.md]
Report rows map to sessions by title; repeated titles: pins.json first, else largest row -> longest session (a guess, flagged).
"""
import csv, json, re, subprocess, sys, os, statistics as st
from datetime import datetime as D

HERE = os.path.dirname(os.path.abspath(__file__))
repo, rep, rep_t = sys.argv[1], sys.argv[2], D.fromisoformat(sys.argv[3])
out = sys.argv[4] if len(sys.argv) > 4 else None
S = json.load(open(os.path.join(HERE, 'sessions.json')))['sessions']
pins = json.load(open(os.path.join(HERE, 'pins.json'))) if os.path.exists(os.path.join(HERE, 'pins.json')) else {}
def t(x): return rep_t if x == 'now' else D.fromisoformat(x)
sess = {sid: dict(title=ti, start=t(a), end=min(t(b), rep_t)) for sid, ti, a, b in S}
rows = list(csv.DictReader(open(rep)))
by_title = {}
for r in rows: by_title.setdefault(r['title'], []).append(r)
for sid, s in sess.items(): s.update(row=None, guess=False)
for title, rs in by_title.items():
    cands = [sid for sid, s in sess.items() if s['title'] == title and s['start'] < rep_t]
    rs = sorted(rs, key=lambda r: -float(r['tokens_M']))
    for sid in [c for c in cands if c in pins]:
        r = next(r for r in rs if abs(float(r['tokens_M']) - pins[sid]) < 0.05); rs.remove(r); sess[sid]['row'] = r
    rest = sorted([c for c in cands if c not in pins], key=lambda c: -(sess[c]['end'] - sess[c]['start']).total_seconds())
    for sid, r in zip(rest, rs):
        sess[sid]['row'] = r; sess[sid]['guess'] = len(rest) > 1
log = subprocess.run(['git', '-C', repo, 'log', 'origin/develop', '--since=2026-10-17T15:00:00Z', '--format=@@%(trailers:key=Claude-Session,valueonly,separator=)%n%s%n%b'], capture_output=True, text=True).stdout
for s in sess.values(): s.update(tasks=0, agent_k=0.0, work=0, types={}, agents={})
for chunk in log.split('@@')[1:]:
    head, _, body = chunk.partition('\n')
    m = re.search(r'session_(\w+)', head)
    if not m or m.group(1) not in sess: continue
    s = sess[m.group(1)]
    for l in body.splitlines():
        if not l.startswith('Timing:'): continue
        s['tasks'] += 1
        w = re.search(r'work=~?(\d+)', l); s['work'] += int(w.group(1)) if w else 0
        ty = re.search(r'type=(\w+)', l); key = ty.group(1) if ty else '?'
        s['types'][key] = s['types'].get(key, 0) + 1
        tk = re.search(r'tokens=(\S+)', l)
        if tk:
            for part in tk.group(1).split(','):
                mm = re.match(r'(?:(\w[\w-]*):)?~?([\d.]+)k', part)
                if mm:
                    s['agent_k'] += float(mm.group(2)); a = mm.group(1) or 'all'
                    s['agents'][a] = s['agents'].get(a, 0) + float(mm.group(2))
lines = ['| Session (title, window UTC) | Report tokens | Tasks landed | Report tokens per task | Agent tokens logged | Agent share | Task types | Mapping |', '|---|---|---|---|---|---|---|---|']
tot_rep = tot_agent = 0
for sid, s in sorted(sess.items(), key=lambda x: x[1]['start']):
    if not s['row'] and not s['tasks']: continue
    rt = float(s['row']['tokens_M']) if s['row'] else 0.0
    tot_rep += rt; tot_agent += s['agent_k'] / 1000
    per = f"{rt / s['tasks']:.1f}M" if s['tasks'] and rt else '-'
    share = f"{s['agent_k'] / 1000 / rt * 100:.1f}%" if rt else '-'
    types = ', '.join(f'{k} {v}' for k, v in sorted(s['types'].items())) or '-'
    lines.append(f"| {s['title']} ({s['start']:%m-%d %H:%M}-{s['end']:%H:%M}) | {rt:.1f}M | {s['tasks']} | {per} | {s['agent_k'] / 1000:.2f}M | {share} | {types} | {'guess' if s['guess'] else 'exact'} |")
lines.append(f"| **Total** | {tot_rep:.0f}M | {sum(s['tasks'] for s in sess.values())} | | {tot_agent:.1f}M | {tot_agent / tot_rep * 100:.1f}% | | |")
text = '\n'.join(lines)
print(text)
if out:
    open(out, 'w').write(text + '\n')
