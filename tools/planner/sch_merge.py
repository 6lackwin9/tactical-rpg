# Three-way JSON merge of kits/rpg/schema.json (single-line JSON): sch_base/ours/theirs.json in dir argv[1] ->
# sch_merged.json; lists of {"key":..} objects merge by key. Prints "conflicts: [...]".
import json, sys
S = sys.argv[1]
ld = lambda n: open(f"{S}/sch_{n}.json", encoding="utf-8").read()
raw = {n: ld(n) for n in ("base", "ours", "theirs")}
def dump(o): return json.dumps(o, ensure_ascii=False, separators=(",", ":"))
b, o, t = (json.loads(raw[n]) for n in ("base", "ours", "theirs"))
conf = []
def m(b, o, t, path):
    if o == t: return o
    if o == b: return t
    if t == b: return o
    if isinstance(o, dict) and isinstance(t, dict) and isinstance(b, dict):
        out = {}
        for k in list(o.keys()) + [k for k in t if k not in o]:
            out[k] = m(b.get(k), o.get(k), t.get(k), path + [k])
        return {k: v for k, v in out.items() if v is not None or (k in o and o[k] is None)}
    if isinstance(o, list) and isinstance(t, list) and isinstance(b, list):
        if all(isinstance(x, dict) and "key" in x for x in b + o + t):
            key = lambda x: x["key"]
            bm = {key(x): x for x in b}; om = {key(x): x for x in o}; tm = {key(x): x for x in t}
            order = [key(x) for x in t] + [key(x) for x in o if key(x) not in tm]
            res = []
            for k in order:
                v = m(bm.get(k), om.get(k), tm.get(k), path + [k])
                if v is not None: res.append(v)
            return res
    conf.append("/".join(map(str, path))); return t
r = m(b, o, t, [])
print("conflicts:", conf)
if not conf:
    open(f"{S}/sch_merged.json", "w", encoding="utf-8", newline="\n").write(dump(r) + raw["ours"][len(raw["ours"].rstrip()):])
    print("written")
