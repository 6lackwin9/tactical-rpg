# Generate (ART2): portrait source art from the Gemini API, straight into art_src/<id>.png. Each prompt in
# games/emberwake/docs/specs/ART2_PROMPTS.md (the shared style block plus the character's block) goes out with the
# style reference image attached; pixelize.py then makes the game portrait. The key comes from the GEMINI_API_KEY
# environment variable and is sent as a header, never printed or written anywhere. Standard library only.
#   python generate.py models                      the models this key can use that make images
#   python generate.py run <model> <id>... [--n N] [--file icons] [--prompts <file>] [--no-ref]
#       N images (default 1) per id; --file icons reads ART2_ICONS.md (whole sheet prompts); --prompts reads another
#       file laid out like ART2_PROMPTS.md (a shared style block first, then one block per id; ART3_PROMPTS.md);
#       --no-ref sends no reference
#   python generate.py run <model> <id>... --faces [--prompts <file>] [--dry-run]
#       PORTRAITFX1 expression sheets: the prompts file (default PORTRAITFX1_FACES.md) is laid out like ART3_PROMPTS.md
#       and each id's section has a `reference: art_src/<file>.png` line (outside or inside its text block; never sent
#       as prompt text); that image goes along instead of Mara Voss, the sheet is written to
#       art_src/<id>_faces_gen<time>_<n>.png and its record gains "kind": "faces"
#   --dry-run (any run): prints each id's reference, prompt size and output name; no API call, no key needed
# Every image written gets a prompt record beside it, <same name>.json (ART3): model, id, the full prompt sent (style
# block included), the reference image (repo-relative, null with --no-ref), the prompts file, the UTC time and the
# candidate index n. The record never holds the key or any header. The .json files stay out of Git LFS.
import base64, datetime, json, os, pathlib, re, sys, time, urllib.error, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
PROMPTS = ROOT / "games/emberwake/docs/specs/ART2_PROMPTS.md"
ICONS = ROOT / "games/emberwake/docs/specs/ART2_ICONS.md"
FACES = ROOT / "games/emberwake/docs/specs/PORTRAITFX1_FACES.md"
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


REF_LINE = re.compile(r"^[ \t]*reference:[ \t]*`?([^`\n]+?)`?[ \t]*$\n?", re.M | re.I)
HEADING = re.compile(r"^## .*\(`([a-z_0-9]+)`\)", re.M)


def prompts(path=PROMPTS, styled=None):
    """id -> the full prompt: the first ```text block inside the id's own `## ... (`id`)` section (up to the next
    `#` or `##` heading). A `styled` file (ART2_PROMPTS.md and the like) prefixes the shared style block, a text block
    before the first id heading, when there is one; ART2_ICONS.md and PROPS1_PROMPTS.md blocks are whole sheet
    prompts. A `reference:` line inside a block is not prompt text (references reads it)."""
    text = path.read_text(encoding="utf-8")
    heads = list(HEADING.finditer(text))
    block = re.compile(r"```text\n(.*?)```", re.S)
    style = None
    if heads:
        pre = block.search(text, 0, heads[0].start())
        style = REF_LINE.sub("", pre.group(1)).strip() if pre else None
    use_style = styled if styled is not None else path == PROMPTS
    out = {}
    for h in heads:
        # a section runs from its heading to the next heading of the same or higher level (# or ##)
        nxt = re.search(r"^#{1,2} ", text[h.end():], re.M)
        end = h.end() + nxt.start() if nxt else len(text)
        m = block.search(text, h.end(), end)
        if not m:
            continue
        body = REF_LINE.sub("", m.group(1)).strip()
        out[h.group(1)] = f"{style}\n\n{body}" if use_style and style else body
    return out


def references(path):
    """id -> the path of its section's `reference:` image (PORTRAITFX1 faces), for the ids that have one. A section
    runs from its `## ... (`id`)` heading to the next `## ` heading; a relative path is from the repo root."""
    text = path.read_text(encoding="utf-8")
    heads = list(HEADING.finditer(text))
    out = {}
    for k, h in enumerate(heads):
        end = heads[k + 1].start() if k + 1 < len(heads) else len(text)
        nxt = re.search(r"^## ", text[h.end():end], re.M)
        m = REF_LINE.search(text, h.end(), h.end() + nxt.start() if nxt else end)
        if m:
            ref = pathlib.Path(m.group(1).strip())
            out[h.group(1)] = ref if ref.is_absolute() else ROOT / ref
    return out


def rel(path):
    """`path` relative to the repo (forward slashes), or as given when it is outside."""
    path = pathlib.Path(path).resolve()
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def record(image, model, cid, prompt, use_ref, prompts_file, n, when=None, reference=None, **extra):
    """Writes the prompt record <image>.json beside an image (no key, no headers); `prompts_file` None = the prompt
    came from elsewhere (a backfill names it in `source`); `reference` the image sent (default Mara Voss)."""
    when = when or datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    rec = {"model": model, "id": cid, "prompt": prompt, "reference": rel(reference or REFERENCE) if use_ref else None,
           "prompts_file": rel(prompts_file) if prompts_file else None, "time": when, "n": n, **extra}
    out = pathlib.Path(image).with_suffix(".json")
    out.write_text(json.dumps(rec, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    return out


def out_name(cid, k, n, kind=None, now=None):
    """The file name of candidate k (0-based): a faces sheet always `<id>_faces_gen<time>_<k+1>.png`; otherwise
    `<id>.png` for a single first image when art_src has none yet, else `<id>_gen<time>_<k+1>.png`."""
    now = int(time.time()) if now is None else now
    if kind == "faces":
        return f"{cid}_faces_gen{now}_{k + 1}.png"
    return f"{cid}.png" if n == 1 and k == 0 and not (SRC / f"{cid}.png").exists() else f"{cid}_gen{now}_{k + 1}.png"


def generate(model, cid, prompt, n, use_ref=True, prompts_file=PROMPTS, reference=None, kind=None):
    """Asks for `n` images of `prompt` (`reference`, default Mara Voss, attached unless use_ref is False), writes each
    to art_src/ with its prompt record; `kind` "faces" names them as expression sheets and records the kind."""
    reference = pathlib.Path(reference or REFERENCE)
    ref = base64.b64encode(reference.read_bytes()).decode() if use_ref else None
    extra = {"kind": kind} if kind else {}
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
        path = SRC / out_name(cid, k, n, kind)
        path.write_bytes(base64.b64decode(imgs[0]["data"]))
        record(path, model, cid, prompt, use_ref, prompts_file, k + 1, reference=reference, **extra)
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
        args = sys.argv[2:]; n = 1; src = PROMPTS; use_ref = True; styled = None; kind = None; dry = False
        if "--n" in args:
            i = args.index("--n"); n = int(args[i + 1]); del args[i:i + 2]
        if "--file" in args:
            i = args.index("--file"); src = ICONS if args[i + 1] == "icons" else pathlib.Path(args[i + 1]); del args[i:i + 2]
        if "--prompts" in args:
            i = args.index("--prompts"); src = pathlib.Path(args[i + 1]).resolve(); styled = True; del args[i:i + 2]
        if "--no-ref" in args:
            args.remove("--no-ref"); use_ref = False
        if "--faces" in args:
            args.remove("--faces"); kind = "faces"
            if styled is None and src == PROMPTS:
                src = FACES; styled = True
        if "--dry-run" in args:
            args.remove("--dry-run"); dry = True
        if not args:
            sys.exit("usage: generate.py run <model> <id>... [--n N] [--file icons] [--prompts <file>] [--no-ref] "
                     "[--faces] [--dry-run]")
        model, ids = args[0], args[1:]
        if not src.is_file():
            sys.exit(f"no prompts file {rel(src)}")
        P = prompts(src, styled); R = references(src)
        for cid in ids:
            if cid not in P:
                print(f"{cid}: no prompt in {src.name}"); continue
            ref = R.get(cid)
            if kind == "faces" and use_ref and ref is None:
                print(f"{cid}: no reference: line in {src.name}"); continue
            if use_ref and not (ref or REFERENCE).is_file():
                print(f"{cid}: no reference image {rel(ref or REFERENCE)}"); continue
            if dry:
                print(f"{cid}: reference {rel(ref or REFERENCE) if use_ref else None}, prompt {len(P[cid])} chars, "
                      f"-> art_src/{out_name(cid, 0, n, kind)}" + (f" (+{n - 1} more)" if n > 1 else ""))
                continue
            try:
                generate(model, cid, P[cid], n, use_ref, src, ref, kind)
            except RuntimeError as e:
                print(f"{cid}: {e}")
