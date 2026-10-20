# Generate (ART2): portrait source art from the Gemini API, straight into art_src/<id>.png. Each prompt in
# games/emberwake/docs/specs/ART2_PROMPTS.md (the shared style block plus the character's block) goes out with the
# style reference image attached; pixelize.py then makes the game portrait. The key comes from the GEMINI_API_KEY
# environment variable and is sent as a header, never printed or written anywhere. Standard library only.
#   python generate.py models                      the models this key can use that make images
#   python generate.py run <model> <id>... [--n N] generate N images (default 1) per character id
import base64, json, os, pathlib, re, sys, time, urllib.error, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
PROMPTS = ROOT / "games/emberwake/docs/specs/ART2_PROMPTS.md"
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


def prompts():
    """id -> the full prompt (style block + the character block) from ART2_PROMPTS.md."""
    text = PROMPTS.read_text(encoding="utf-8")
    blocks = re.findall(r"```text\n(.*?)```", text, re.S)
    style, rest = blocks[0].strip(), blocks[1:]
    ids = re.findall(r"^## .*\(`([a-z_]+)`\)", text, re.M)
    return {i: f"{style}\n\n{b.strip()}" for i, b in zip(ids, rest)}


def generate(model, cid, prompt, n):
    ref = base64.b64encode(REFERENCE.read_bytes()).decode()
    saved = []
    for k in range(n):
        body = {"contents": [{"parts": [{"inline_data": {"mime_type": "image/png", "data": ref}}, {"text": prompt}]}],
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
        args = sys.argv[2:]; n = 1
        if "--n" in args:
            i = args.index("--n"); n = int(args[i + 1]); del args[i:i + 2]
        model, ids = args[0], args[1:]
        P = prompts()
        for cid in ids:
            if cid not in P:
                print(f"{cid}: no prompt in {PROMPTS.name}"); continue
            try:
                generate(model, cid, P[cid], n)
            except RuntimeError as e:
                print(f"{cid}: {e}")
