# Generate (ART2): portrait source art from the Gemini API, straight into art_src/<id>.png. Each prompt in
# games/emberwake/docs/specs/ART2_PROMPTS.md (the shared style block plus the character's block) goes out with the
# style reference image attached; pixelize.py then makes the game portrait. The key comes from the GEMINI_API_KEY
# environment variable and is sent as a header, never printed or written anywhere. Standard library only.
#   python generate.py models                      the models this key can use that make images
#   python generate.py run <model> <id>... [--n N] [--file icons] [--prompts <file>] [--no-ref]
#       N images (default 1) per id; --file icons reads ART2_ICONS.md (whole sheet prompts); --prompts reads another
#       file laid out like ART2_PROMPTS.md (a shared style block first, then one block per id; ART3_PROMPTS.md);
#       --no-ref sends no reference
# Every image written gets a prompt record beside it, <same name>.json (ART3): model, id, the full prompt sent (style
# block included), the reference image (repo-relative, null with --no-ref), the prompts file, the UTC time and the
# candidate index n. The record never holds the key or any header. The .json files stay out of Git LFS.
import base64, datetime, json, os, pathlib, re, sys, time, urllib.error, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
PROMPTS = ROOT / "games/emberwake/docs/specs/ART2_PROMPTS.md"
ICONS = ROOT / "games/emberwake/docs/specs/ART2_ICONS.md"
SRC = ROOT / "art_src"
REFERENCE = SRC / "mara_voss.png"
API = "https://generativelanguage.googleapis.com/v1beta"


def call(path, body=None):
    key = os.environ.get("GEMINI_API_KEY")
    if not key:
        sys.exit("GEMINI_API_KEY is not set")
    req = urllib.request.Request(f"{API}/{path}", data=None if body is None else json.dumps(body).encode(),
                                 headers={"x-goog-api-key": key, "Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=180) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        msg = e.read().decode(errors="replace")
        try:
            msg = json.loads(msg)["error"]["message"]
        except (ValueError, KeyError):
            pass
        raise RuntimeError(f"HTTP {e.code}: {msg[:400]}") from None


def models():
    out, token = [], ""
    while True:
        page = call("models?pageSize=200" + (f"&pageToken={token}" if token else ""))
        out += page.get("models", [])
        token = page.get("nextPageToken", "")
        if not token:
            return out


def prompts(path=PROMPTS, styled=None):
    """id -> the full prompt. ART2_PROMPTS.md (and any `styled` file): the shared style block + each character
    block; ART2_ICONS.md: each block is a whole sheet prompt."""
    text = path.read_text(encoding="utf-8")
    blocks = re.findall(r"```text\n(.*?)```", text, re.S)
    ids = re.findall(r"^## .*\(`([a-z_0-9]+)`\)", text, re.M)
    if styled if styled is not None else path == PROMPTS:
        style, rest = blocks[0].strip(), blocks[1:]
        return {i: f"{style}\n\n{b.strip()}" for i, b in zip(ids, rest)}
    return {i: b.strip() for i, b in zip(ids, blocks)}


def rel(path):
    """`path` relative to the repo (forward slashes), or as given when it is outside."""
    path = pathlib.Path(path).resolve()
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def record(image, model, cid, prompt, use_ref, prompts_file, n, when=None, **extra):
    """Writes the prompt record <image>.json beside an image (no key, no headers); `prompts_file` None = the prompt
    came from elsewhere (a backfill names it in `source`)."""
    when = when or datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    rec = {"model": model, "id": cid, "prompt": prompt, "reference": rel(REFERENCE) if use_ref else None,
           "prompts_file": rel(prompts_file) if prompts_file else None, "time": when, "n": n, **extra}
    out = pathlib.Path(image).with_suffix(".json")
    out.write_text(json.dumps(rec, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    return out


def generate(model, cid, prompt, n, use_ref=True, prompts_file=PROMPTS):
    ref = base64.b64encode(REFERENCE.read_bytes()).decode() if use_ref else None
    saved = []
    for k in range(n):
        parts = ([{"inline_data": {"mime_type": "image/png", "data": ref}}] if ref else []) + [{"text": prompt}]
        body = {"contents": [{"parts": parts}],
                "generationConfig": {"responseModalities": ["TEXT", "IMAGE"]}}
        reply = call(f"models/{model}:generateContent", body)
        parts = [p for c in reply.get("candidates", []) for p in c.get("content", {}).get("parts", [])]
        imgs = [p.get("inlineData") or p.get("inline_data") for p in parts if p.get("inlineData") or p.get("inline_data")]
        if not imgs:
            note = " ".join(p.get("text", "") for p in parts)[:300] or json.dumps(reply.get("promptFeedback", {}))[:300]
            print(f"{cid} #{k + 1}: no image ({note})")
            continue
        name = f"{cid}.png" if n == 1 and k == 0 and not (SRC / f"{cid}.png").exists() else f"{cid}_gen{int(time.time())}_{k + 1}.png"
        path = SRC / name
        path.write_bytes(base64.b64decode(imgs[0]["data"]))
        record(path, model, cid, prompt, use_ref, prompts_file, k + 1)
        saved.append(path); print(f"{cid} #{k + 1}: {path.name}")
    return saved


if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] not in ("models", "run"):
        sys.exit(__doc__ or "usage: generate.py models | run <model> <id>... [--n N]")
    if sys.argv[1] == "models":
        for m in models():
            name = m["name"].split("/", 1)[-1]
            if "image" in name or "imagen" in name:
                print(name, "|", m.get("displayName", ""), "|", ",".join(m.get("supportedGenerationMethods", [])))
    else:
        args = sys.argv[2:]; n = 1; src = PROMPTS; use_ref = True; styled = None
        if "--n" in args:
            i = args.index("--n"); n = int(args[i + 1]); del args[i:i + 2]
        if "--file" in args:
            i = args.index("--file"); src = ICONS if args[i + 1] == "icons" else pathlib.Path(args[i + 1]); del args[i:i + 2]
        if "--prompts" in args:
            i = args.index("--prompts"); src = pathlib.Path(args[i + 1]).resolve(); styled = True; del args[i:i + 2]
        if "--no-ref" in args:
            args.remove("--no-ref"); use_ref = False
        model, ids = args[0], args[1:]
        P = prompts(src, styled)
        for cid in ids:
            if cid not in P:
                print(f"{cid}: no prompt in {src.name}"); continue
            try:
                generate(model, cid, P[cid], n, use_ref, src)
            except RuntimeError as e:
                print(f"{cid}: {e}")
