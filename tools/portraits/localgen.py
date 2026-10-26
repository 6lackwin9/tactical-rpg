# Local generation (LOCALGEN1, LOCALGEN2): portrait drafts, edits and expression cells on this PC's own models, the art
# pipeline's local tool; generate.py (the image API) is the fallback. Two backends (--backend):
#   comfyui (default, LOCALGEN2, user 2026-10-26: FLUX.2 Klein 4B is the main local model; speed is not a concern):
#       the local ComfyUI server (--server, env LOCALGEN_COMFY_URL, default http://127.0.0.1:8188; its folder
#       LOCALGEN_HOME/comfyui; start it with C:\dev\localgen\comfyui.bat) runs FLUX.2 [klein] 4B distilled with the
#       Qwen3 4B text encoder (--clip), 4 steps, cfg 1. The graphs are the ones verified on this machine
#       (C:\dev\localgen\probe\comfy_run.py and comfy_faces.py). Standard library only (urllib).
#   openvino (LOCALGEN1): OpenVINO GenAI on the Intel NPU, the Arc GPU or the CPU, a few seconds an image, for quick
#       drafts.
# The venv, the models, the LoRA files and the compile cache stay outside the repo, in LOCALGEN_HOME (default
# C:\dev\localgen); run the tool with that venv's Python, from the repo root:
#   C:\dev\localgen\venv\Scripts\python.exe tools/portraits/localgen.py <command> ...
#
# ComfyUI backend (Klein, the default):
#   localgen.py run <id>... [--prompts FILE | --prompt TEXT] [--style TEXT] [--n N] [--seed S] [--size PX]
#               [--steps K] [--clip FILE] [--server URL] [--out DIR] [--sheet PNG] [--dry-run]
#       text to image with the whole prompt (Klein's Qwen3 encoder reads it all; nothing is cut, the record keeps a
#       token estimate): the character block first, then the shared style block (or --style) without its "Use the
#       attached image" line (a text-to-image run attaches none). The prompt is --prompt as given, else the id's
#       block and style block in --prompts FILE, else the record beside art_src/<id>.png, else the id's block and
#       style block in ART3_PROMPTS.md. Default 1024 px.
#   localgen.py vary <id> --from PNG --change TEXT [--n N] [--seed S] [--megapixels M] [same options]
#       a Klein reference edit: PNG (uploaded through /upload/image) becomes the reference latent and TEXT the whole
#       prompt ("Keep the same character ... Change only ..."); the image is scaled to M megapixels (default 1).
#   localgen.py faces <id>... [--exprs hurt angry afraid] [--src DIR] [--n N] [--seed S] [same options]
#       the PORTRAITFX1 expression cells (the PORTRAITFX1_TASKS 2.4.2 cell lines) as Klein edits of <src>/<id>.png
#       (default art_src/), written as <out>/<id>_face_<expr>_loc<time>_<n>.png.
#   Takes of run and vary go to <out>/<id>_loc<time>_<n>.png (default out: art_src/). All jobs are queued first, then
#   collected. The record: generate.py's keys (model "local:flux-2-klein-4b", reference = the source image of an
#   edit), then backend, kind (text, edit, face), unet, clip, vae, sampler, steps, cfg, size (text) or megapixels
#   (edit), seed, tokens_estimate, cut, seconds, and change (vary) or expression (faces). No server running: the
#   error says to start C:\dev\localgen\comfyui.bat. `prompt <id>` prints the Klein prompt.
#
# OpenVINO backend (--backend openvino):
#   localgen.py run <id>... [--prompts FILE | --prompt TEXT] [--n N] [--seed S] [--steps K] [--size PX]
#               [--device GPU|NPU|CPU] [--model DIR] [--guidance G] [--lora PATH:ALPHA ...] [--trigger TEXT]
#               [--style TEXT] [--out DIR] [--sheet PNG] [--dry-run]
#       text to image: N takes per id (seeds S, S+1, ...) to <out>/<id>_loc<time>_<n>.png (default out: art_src/),
#       each with its prompt record <same name>.json. The prompt is the id's block in FILE (default ART3_PROMPTS.md)
#       rebuilt subject first for the 77-token text encoder (short_prompt), or TEXT sent as given. Defaults: GPU,
#       512 px, 6 steps, seed 1, guidance 8 (LCM) or 7. --lora adds a LoRA adapter (alpha default 0.8; on the NPU in
#       static mode, whose first compile takes about 16 min); --trigger adds its trigger words to the style tail and
#       --style replaces the tail.
#   localgen.py vary <id> --from PNG --change TEXT [--strength F ...] [--prompt TEXT | --prompts FILE] [same options]
#       image to image: variants of an existing image (Image2ImagePipeline); the base prompt is the record beside PNG
#       (or the id's block in FILE, or TEXT), shortened, with the change put first. Several strengths: one take each
#       (the take's steps are about steps x strength, so raise --steps for low strengths).
#   localgen.py sheet OUT PNG... [--cols C]
#       a contact sheet: each image beside its pixelize.py game portrait (96 px grid), captioned from its record
#   localgen.py prompt <id>... [--prompts FILE] [--trigger TEXT] [--style TEXT]
#       prints the short prompt, its token count and what was cut (no model needed)
#   localgen.py lora-kohya SRC DST
#       renames a diffusers-format LoRA's keys (unet.*.lora.down.weight) to the kohya form OpenVINO GenAI reads
#
# --dry-run prints the prompt and the record each take would get and writes nothing (no OpenVINO needed). The record
# has generate.py's shape (model, id, prompt, reference, prompts_file, time, n) with model "local:<model dir name>"
# plus family, device, seed, steps, size, guidance, negative, lora, seconds and cut (the block parts left out of the
# prompt); a variant also names source_image, strength and change. Never any key: local runs need none.
# --model picks another model folder: its family (FAMILIES: LCM, SD 1.5, SDXL, FLUX, from model_index.json) sets the
# default steps, guidance and size, the negative prompt, and the prompt form (short for CLIP-only, whole for FLUX).
# Standard library and Pillow (plus openvino_genai and numpy from the local venv for generation only).
import argparse
import datetime
import importlib
import json
import os
import pathlib
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import generate  # noqa: E402  (record shape, prompts files, repo paths)

ROOT = generate.ROOT
SRC = generate.SRC
ART3 = ROOT / "games/emberwake/docs/specs/ART3_PROMPTS.md"
HOME = pathlib.Path(os.environ.get("LOCALGEN_HOME", r"C:\dev\localgen"))
MODEL = HOME / "models" / "lcm_dreamshaper_v7_int8"
CACHE = HOME / "cache"
DEVICES = ("NPU", "GPU", "CPU")

# Per model family (family(): model_index.json's pipeline class, else the folder name): the defaults, whether it takes
# a negative prompt, and whether its text encoders are CLIP only (77 tokens: the short form) or read a long prompt
# (FLUX's T5: the full prompt). FLUX.1-schnell is distilled: 4 steps and no guidance.
FAMILIES = {
    "lcm": {"steps": 6, "guidance": 8.0, "size": 512, "negative": False, "short": True},
    "sd15": {"steps": 25, "guidance": 7.0, "size": 512, "negative": True, "short": True},
    "sdxl": {"steps": 25, "guidance": 7.0, "size": 1024, "negative": True, "short": True},
    "flux": {"steps": 4, "guidance": 0.0, "size": 1024, "negative": False, "short": False},
}
PIPELINE_CLASSES = {"LatentConsistencyModelPipeline": "lcm", "StableDiffusionPipeline": "sd15",
                    "StableDiffusionXLPipeline": "sdxl", "FluxPipeline": "flux"}

# The CLIP text encoder of the SD 1.5 family (and both of SDXL's) reads 77 tokens, the start and end marks included.
TOKENS = 75
# FLUX.1-schnell's T5 encoder reads 256 tokens (its max_sequence_length).
T5_TOKENS = 256
LEAD = "head and shoulders portrait of"
# The ART3 style block cut to what a small model follows: stylized (ART3B), the framing and light, the background.
TAIL = "stylized dark fantasy character art, facing right, amber rim light, plain dark background"
NEGATIVE = "blurry, photo, 3d render, text, watermark, frame, border, hood, cowl"
# A block's sentences in this order before the rest: who it is, then its hair. A match names the subject.
WHO = re.compile(r"\b(man|woman|adult|inquisitor|corpse|hound|centipede|figure|person|in (his|her) "
                 r"(teens|twenties|thirties|forties|fifties|sixties)|of nineteen)\b", re.I)
NEGATION = re.compile(r"^(not|no|never|without|nothing|faceless)\b", re.I)


# ---------------------------------------------------------------------------------------------------- token counting

def _bytes_to_unicode():
    bs = [*range(ord("!"), ord("~") + 1), *range(ord("¡"), ord("¬") + 1), *range(ord("®"), ord("ÿ") + 1)]
    cs = bs[:]
    n = 0
    for b in range(256):
        if b not in bs:
            bs.append(b); cs.append(256 + n); n += 1
    return dict(zip(bs, map(chr, cs)))


class ClipTokens:
    """Counts CLIP BPE tokens with the model's own tokenizer files (tokenizer/vocab.json and merges.txt), the same
    split the text encoder sees; plain Python, so the count needs no model run."""
    PAT = re.compile(r"<\|startoftext\|>|<\|endoftext\|>|'s|'t|'re|'ve|'m|'ll|'d|[^\W\d_]+|\d|(?:[^\s\w]|_)+", re.I)

    def __init__(self, folder):
        lines = (pathlib.Path(folder) / "merges.txt").read_text(encoding="utf-8").split("\n")[1:]
        merges = [tuple(m.split()) for m in lines if len(m.split()) == 2]
        self.ranks = {m: i for i, m in enumerate(merges)}
        self.byte = _bytes_to_unicode()
        self.cache = {}

    def bpe(self, token):
        if token in self.cache:
            return self.cache[token]
        word = tuple(token[:-1]) + (token[-1] + "</w>",)
        while len(word) > 1:
            pairs = {(word[i], word[i + 1]) for i in range(len(word) - 1)}
            best = min(pairs, key=lambda p: self.ranks.get(p, 1 << 30))
            if best not in self.ranks:
                break
            out, i = [], 0
            while i < len(word):
                if i < len(word) - 1 and (word[i], word[i + 1]) == best:
                    out.append(word[i] + word[i + 1]); i += 2
                else:
                    out.append(word[i]); i += 1
            word = tuple(out)
        self.cache[token] = len(word)
        return len(word)

    def __call__(self, text):
        text = re.sub(r"\s+", " ", text).strip().lower()
        return sum(self.bpe("".join(self.byte[b] for b in t.encode("utf-8"))) for t in self.PAT.findall(text))


def estimate(text):
    """A token count without the tokenizer, on the high side: each word one token plus one per 4 letters past the
    first 4 (rare words split), each punctuation mark one, plus a tenth (without it, over the ART3 blocks' clauses
    the count came out 4 % below the CLIP count on average, 22 % below at worst)."""
    words = re.findall(r"[^\W\d_]+|\d", text)
    raw = sum(1 + max(0, len(w) - 4) // 4 for w in words) + len(re.findall(r"[^\s\w]", text))
    return -(-raw * 11 // 10)


def counter(model=MODEL):
    """The model's CLIP token counter when its tokenizer files are there, else the estimate."""
    folder = pathlib.Path(model) / "tokenizer"
    return ClipTokens(folder) if (folder / "merges.txt").exists() and (folder / "vocab.json").exists() else estimate


# ------------------------------------------------------------------------------------------------------ short prompt

def block_prompts(path=ART3):
    """id -> its character block in a prompts file laid out like ART2_PROMPTS.md (a shared style block first, then one
    block per `## Name (`id`)` heading), without the style block."""
    text = pathlib.Path(path).read_text(encoding="utf-8")
    blocks = [b.strip() for b in re.findall(r"```text\n(.*?)```", text, re.S)]
    ids = re.findall(r"^## .*\(`([a-z_0-9]+)`\)", text, re.M)
    if len(blocks) == len(ids) + 1:
        blocks = blocks[1:]
    return dict(zip(ids, blocks))


ARTICLE = re.compile(r"^(a|an|the|his|her|its|their|one|two|every|and|with|from|over|under)\b", re.I)


def _clauses(sentence):
    """A sentence in the pieces the prompt is filled with: split at `;` and `:`, then at commas, where a comma list of
    adjectives stays with its noun ("large, bright, alert eyes"): a piece of one or two words, or one ending in an -ed
    word before a piece without an article, joins the next piece of its segment."""
    out = []
    for seg in re.split(r"[;:]\s+|\s+-\s+", sentence):
        pieces = [c.strip(" .,") for c in seg.split(", ") if c.strip(" .,")]
        k = 0
        while k < len(pieces):
            piece = pieces[k]
            while k + 1 < len(pieces) and (len(piece.split()) <= 2 or
                                           re.search(r"ed$", piece) and not ARTICLE.match(pieces[k + 1])):
                k += 1
                piece = f"{piece}, {pieces[k]}"
            first = piece.split()[0]
            out.append(piece if len(first) > 1 and first.isupper() else piece[0].lower() + piece[1:])
            k += 1
    return out


def _sentences(text):
    return [s for s in re.split(r"(?<=[.!?])\s+(?=[A-Z\"'])", text.strip()) if s.strip(" .")]


def parse_block(block):
    """A character block (or a whole recorded prompt: the text after `Character:`) in its parts: the name, the title,
    the look sentences, the mood and the main colours."""
    i = block.find("Character:")
    body = re.sub(r"\s+", " ", block[i + len("Character:"):] if i >= 0 else block).strip()
    body = re.sub(r"\b(St|Mr|Mrs|Dr)\. ", r"\1 ", body)  # not a sentence end
    mood = re.search(r"\bMood:\s*(.*?)(?=\s*Main colou?rs:|$)", body)
    colours = re.search(r"\bMain colou?rs:\s*(.*)$", body)
    look = re.split(r"\s*\bMood:|\s*\bMain colou?rs:", body)[0].strip()
    name = title = None
    if i >= 0:
        first, _, rest = look.partition(". ")
        if first.split(",")[0].strip().istitle():  # "Name, Title." opens a block
            name, _, title = (p.strip() for p in first.partition(","))
            look = rest
    return {"name": name, "title": title or None, "look": look,
            "mood": mood.group(1).strip(" .") if mood else None,
            "colours": colours.group(1).strip(" .") if colours else None}


def short_prompt(block, count=estimate, tokens=TOKENS, change=None, trigger=None, style=TAIL):
    """A character block rebuilt subject first for a 77-token text encoder (`tokens` without the start and end marks):
    LEAD + who + the look (stylization, hair, the rest, in that order) + mood + colours, then `style` (TAIL by default)
    and a LoRA trigger, filled clause by clause, breadth first (the first clause of every part, then the second, and so
    on) while they fit; the parts keep their order. Style-first prompts made the small model draw a game scene
    instead of a portrait.
    The who part keeps its "adult" with its first clause (the tone rule; the model draws young faces). Left out,
    listed in `cut`: the name (a model cannot know it), parentheses, hex codes, negations (a CLIP prompt cannot say
    "no": "no hood" draws a hood; NEGATIVE carries the hair rule for models that take a negative prompt) and whatever
    does not fit. A `change` (a variant's edit) goes first, before LEAD. Returns (prompt, cut)."""
    p = parse_block(block)
    cut = []
    if p["name"]:
        cut.append(f"name: {p['name']}")
    look = p["look"]
    for paren in re.findall(r"\s*\(([^)]*)\)", look):
        cut.append(f"parentheses: {paren}")
    look = re.sub(r"\s*\([^)]*\)", "", look)
    look = re.sub(r"\s*#[0-9a-fA-F]{6}\b", "", look)
    look = re.sub(r"\bStylization [^:]*:\s*", "", look)
    sentences = _sentences(look)
    who = [s for s in sentences if WHO.search(s)][:1]
    hair = [s for s in sentences if re.search(r"\b(hair|bald|beard|shaved)\b", s, re.I) and s not in who][:1]
    rest = [s for s in sentences if s not in who and s not in hair]
    stylized = rest[:1] if rest and not WHO.search(rest[0]) else []
    order = who + stylized + hair + [s for s in rest if s not in stylized]
    parts = [_clauses(s) for s in order]
    if who:  # the tone rule: "adult" rides with the who clause, never cut
        w = parts[0]
        adult = [c for c in w[1:] if re.search(r"\badult\b", c, re.I)]
        if adult:
            w[0] = ", ".join([w[0]] + adult)
            parts[0] = [w[0]] + [c for c in w[1:] if c not in adult]
        elif re.search(r"\b(young|teens|nineteen)\b", w[0], re.I) and not re.search(r"\badult\b", w[0], re.I):
            w[0] = f"adult, {w[0]}"
    if p["title"] and re.match(r"(a|an)\b", p["title"]):
        parts.insert(0, [p["title"]])  # a creature's epithet says what it is
    elif p["title"]:
        cut.append(f"title: {p['title']}")
    if p["mood"] and change:
        cut.append(f"mood (the change replaces it): {p['mood']}")
    elif p["mood"]:
        parts.append([f"{m} expression" if k == 0 else m for k, m in enumerate(_clauses(p["mood"]))])
    if p["colours"]:
        parts.append(_clauses(p["colours"]))
    for part in parts:
        for j, c in enumerate(part):
            if NEGATION.match(c):
                cut.append(f"negation: {c}")
                part[j] = None
    tail = style + (f", {trigger}" if trigger else "")
    head = (f"{change}, " if change else "") + LEAD

    def text(chosen):
        look = ", ".join(c for part in parts for c in part if c and c in chosen)
        return f"{head} {look}, {tail}" if look else f"{head} {tail}"

    chosen = set()
    for rnd in range(max((len(part) for part in parts), default=0)):
        for part in parts:
            for j, c in enumerate(part):
                if not c or c in chosen or j > rnd:
                    continue
                if count(text(chosen | {c})) <= tokens:
                    chosen.add(c)
    for part in parts:
        for c in part:
            if c and c not in chosen:
                cut.append(c)
    return text(chosen), cut


def overflow(prompt, count, tokens=TOKENS):
    """The words of a free prompt past the encoder's reach (it cuts them off)."""
    words = prompt.split()
    for k in range(len(words), 0, -1):
        if count(" ".join(words[:k])) <= tokens:
            return [" ".join(words[k:])] if k < len(words) else []
    return [prompt]


def t5_estimate(text):
    """A T5 (sentencepiece) token count estimate, on the high side: each word one token plus one per 7 letters past
    the first, each punctuation mark one, plus a third (without it, over the ART3 blocks and style lines the count came
    out at 0.80 of FLUX's own tokenizer on average, 0.74 at worst; 2026-10-26)."""
    words = re.findall(r"[^\W\d_]+|\d+", text)
    raw = sum(1 + (len(w) - 1) // 7 for w in words) + len(re.findall(r"[^\s\w]", text))
    return -(-raw * 4 // 3)


def style_block(path):
    """The shared style block that opens a prompts file laid out like ART2_PROMPTS.md ("" when it has none)."""
    text = pathlib.Path(path).read_text(encoding="utf-8")
    blocks = re.findall(r"```text\n(.*?)```", text, re.S)
    ids = re.findall(r"^## .*\(`([a-z_0-9]+)`\)", text, re.M)
    return blocks[0].strip() if len(blocks) == len(ids) + 1 else ""


def full_prompt(text, style="", change=None, trigger=None, tokens=T5_TOKENS, count=t5_estimate):
    """The whole prompt for a model with a long-prompt encoder (FLUX's T5), subject first: the character block, then
    the style block (`style`, or the part of a whole recorded prompt before `Character:`) without its lines about an
    attached reference image (a local run attaches none). Subject first because the encoder still stops at `tokens`
    (256 for FLUX.1-schnell) and an ART3 style block alone is longer. A `change` goes first, a LoRA trigger last.
    Returns (prompt, cut): the reference lines and, by the estimate, what falls past the encoder's reach."""
    cut = []
    i = text.find("Character:")
    if i > 0:
        style, text = text[:i], text[i:]
    kept = []
    for line in style.strip().splitlines():
        if re.search(r"\battached\b", line, re.I):
            cut.append(f"reference line: {line[:70]}...")
        elif line.strip():
            kept.append(line.strip())
    prompt = "\n".join(p for p in [change, text.strip(), "\n".join(kept), trigger] if p)
    for rest in overflow(prompt, count, tokens):
        words = rest.split()
        cut.append(f"past about {tokens} T5 tokens (estimated): {len(words)} words from \"{' '.join(words[:8])} ...\"")
    return prompt, cut


# ------------------------------------------------------------------------------------------------------------ record

def model_name(model):
    return "local:" + pathlib.Path(model).name


def family(model):
    """The model's family (a FAMILIES key): its model_index.json pipeline class, else a guess from the folder name."""
    index = pathlib.Path(model) / "model_index.json"
    if index.exists():
        cls = json.loads(index.read_text(encoding="utf-8")).get("_class_name", "")
        if cls in PIPELINE_CLASSES:
            return PIPELINE_CLASSES[cls]
    name = pathlib.Path(model).name.lower()
    for key, fam in (("lcm", "lcm"), ("flux", "flux"), ("sdxl", "sdxl"), ("_xl", "sdxl")):
        if key in name:
            return fam
    return "sd15"


def kohya_key(key):
    """A diffusers attention-processor LoRA key (`unet.<module path>.lora.down.weight`, which OpenVINO GenAI 2026.4
    does not match and skips as "unused") in the kohya form it reads (`lora_unet_<module_path>.lora_down.weight`);
    other keys as they are."""
    m = re.match(r"^(unet|text_encoder)\.(.+)\.lora\.(down|up)\.weight$", key)
    if not m:
        return key
    prefix = "lora_unet" if m.group(1) == "unet" else "lora_te"
    return f"{prefix}_{m.group(2).replace('.', '_')}.lora_{m.group(3)}.weight"


def convert_lora(src, dst):
    """Rewrites a safetensors LoRA with kohya_key names (the tensors' bytes untouched; standard library only).
    Returns how many keys were renamed."""
    raw = pathlib.Path(src).read_bytes()
    n = int.from_bytes(raw[:8], "little")
    header = json.loads(raw[8:8 + n])
    renamed = {kohya_key(k): v for k, v in header.items()}
    changed = sum(kohya_key(k) != k for k in header)
    text = json.dumps(renamed, separators=(",", ":")).encode("utf-8")
    text += b" " * (-len(text) % 8)
    pathlib.Path(dst).write_bytes(len(text).to_bytes(8, "little") + text + raw[8 + n:])
    return changed


def parse_lora(spec):
    """`path:alpha` (alpha default 0.8; a Windows drive colon is kept) -> (path, alpha)."""
    m = re.match(r"^(.*?)(?::([-+]?\d*\.?\d+))?$", spec)
    return m.group(1), float(m.group(2)) if m.group(2) else 0.8


def lora_rel(path):
    """A LoRA path for a record without the machine: relative to the repo, or `$LOCALGEN_HOME/<path>` under HOME, else
    the file name only (records are committed beside picks)."""
    path = pathlib.Path(path).resolve()
    for base, prefix in ((ROOT, ""), (HOME.resolve(), "$LOCALGEN_HOME/")):
        try:
            return prefix + path.relative_to(base).as_posix()
        except ValueError:
            pass
    return path.name

def make_record(cid, prompt, n, *, model, device, seed, steps, size, guidance, prompts_file=None, lora=(),
                cut=(), seconds=None, when=None, source_image=None, strength=None, change=None):
    """The prompt record of one local take, in generate.py's shape plus the local settings."""
    when = when or datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    rec = {"model": model_name(model), "id": cid, "prompt": prompt, "reference": None,
           "prompts_file": generate.rel(prompts_file) if prompts_file else None, "time": when, "n": n,
           "family": family(model), "device": device, "seed": seed, "steps": steps, "size": size,
           "guidance": guidance, "negative": NEGATIVE if FAMILIES[family(model)]["negative"] else None,
           "lora": [{"path": lora_rel(p), "alpha": a} for p, a in lora] or None,
           "cut": list(cut), "seconds": seconds}
    if source_image is not None:
        rec.update({"source_image": generate.rel(source_image), "strength": strength, "change": change})
    return rec


def write_record(image, rec):
    out = pathlib.Path(image).with_suffix(".json")
    out.write_text(json.dumps(rec, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    return out


def take_path(out, cid, n, when=None):
    return pathlib.Path(out) / f"{cid}_loc{int(when or time.time())}_{n}.png"


# -------------------------------------------------------------------------------------------------------- generation

class Pipe:
    """One compiled OpenVINO GenAI pipeline (text to image or image to image) on a device, with LoRA adapters."""

    def __init__(self, kind, model, device, size, guidance, lora=()):
        # only for generation, from the local venv: the tests and --dry-run run on the plain Python without it
        g = importlib.import_module("openvino_genai")
        t0 = time.time()
        self.pipe = (g.Text2ImagePipeline if kind == "text" else g.Image2ImagePipeline)(str(model))
        self.adapters = None
        if lora:
            # the NPU takes LoRA only with static shapes (MODE_AUTO fails "to_shape was called on a dynamic shape";
            # MODE_FUSE needs f16/f32 weights, not this int8 model); its first compile with an adapter took 16 min
            mode = g.AdapterConfig.Mode.MODE_STATIC if device == "NPU" else g.AdapterConfig.Mode.MODE_AUTO
            self.adapters = g.AdapterConfig([(g.Adapter(str(p)), a) for p, a in lora], mode)
        if device == "NPU":
            self.pipe.reshape(1, size, size, guidance)  # the NPU needs static shapes
        props = {"CACHE_DIR": str(CACHE)}
        if self.adapters is not None:
            props["adapters"] = self.adapters
        self.pipe.compile(device, **props)  # cached: only the first load per model, device and size is slow
        self.load = time.time() - t0

    def generate(self, prompt, *, size, steps, seed, guidance, negative=None, image=None, strength=None,
                 max_sequence_length=None):
        kw = dict(num_inference_steps=steps, rng_seed=seed, guidance_scale=guidance)
        if negative:
            kw["negative_prompt"] = negative
        if max_sequence_length:
            kw["max_sequence_length"] = max_sequence_length  # FLUX's T5 reach
        if self.adapters is not None:
            kw["adapters"] = self.adapters
        t1 = time.time()
        if image is None:
            out = self.pipe.generate(prompt, width=size, height=size, **kw)
        else:
            np, ov = importlib.import_module("numpy"), importlib.import_module("openvino")
            arr = np.asarray(image.convert("RGB").resize((size, size)), dtype=np.uint8)[None]
            out = self.pipe.generate(prompt, ov.Tensor(arr), strength=strength, **kw)
        from PIL import Image
        return Image.fromarray(out.data[0]), time.time() - t1


# ------------------------------------------------------------------------------------------- ComfyUI backend (Klein)

COMFY_URL = os.environ.get("LOCALGEN_COMFY_URL", "http://127.0.0.1:8188")
COMFY = HOME / "comfyui"
COMFY_START = HOME / "comfyui.bat"
KLEIN = "local:flux-2-klein-4b"
KLEIN_UNET = "flux-2-klein-4b.safetensors"
# The full Qwen3 4B encoder (speed is not a concern); qwen_3_4b_fp4_flux2.safetensors is the smaller one that also works.
KLEIN_CLIP = "qwen_3_4b.safetensors"
KLEIN_VAE = "flux2-vae.safetensors"
KLEIN_STEPS = 4  # the distilled model
KLEIN_CFG = 1.0
KLEIN_SIZE = 1024
KLEIN_MEGAPIXELS = 1.0
# The PORTRAITFX1 expression cells (PORTRAITFX1_TASKS 2.4.2 lines) and the edit wording comfy_faces.py verified.
CELLS = {
    "hurt": "wincing in pain, one eye squeezed narrower, teeth gritted, head pulled back slightly",
    "angry": "brows drawn down hard, jaw set or snarling, eyes glaring",
    "afraid": "eyes wide, brows raised, mouth slightly open, head drawn in",
}
KEEP = ("Keep the same character, face shape, hair, scars and markings, outfit, pixel-art style, framing, lighting and "
        "dark background exactly. Change only the facial expression to {e}: {cell}.")


def klein_graph(prompt, *, seed, steps=KLEIN_STEPS, size=KLEIN_SIZE, prefix="localgen/take", clip=KLEIN_CLIP):
    """The ComfyUI API graph of a FLUX.2 Klein 4B text-to-image job (comfy_run.py's, verified 2026-10-26): the prompt
    encoded once, its zeroed copy as the negative, CFG 1, euler over the Flux2 scheduler, an empty Flux2 latent."""
    return {
        "1": {"class_type": "UNETLoader", "inputs": {"unet_name": KLEIN_UNET, "weight_dtype": "default"}},
        "2": {"class_type": "CLIPLoader", "inputs": {"clip_name": clip, "type": "flux2", "device": "default"}},
        "3": {"class_type": "VAELoader", "inputs": {"vae_name": KLEIN_VAE}},
        "4": {"class_type": "CLIPTextEncode", "inputs": {"text": prompt, "clip": ["2", 0]}},
        "5": {"class_type": "ConditioningZeroOut", "inputs": {"conditioning": ["4", 0]}},
        "6": {"class_type": "CFGGuider", "inputs": {"model": ["1", 0], "positive": ["4", 0], "negative": ["5", 0],
                                                    "cfg": KLEIN_CFG}},
        "7": {"class_type": "KSamplerSelect", "inputs": {"sampler_name": "euler"}},
        "8": {"class_type": "Flux2Scheduler", "inputs": {"steps": steps, "width": size, "height": size}},
        "9": {"class_type": "EmptyFlux2LatentImage", "inputs": {"width": size, "height": size, "batch_size": 1}},
        "10": {"class_type": "RandomNoise", "inputs": {"noise_seed": seed}},
        "11": {"class_type": "SamplerCustomAdvanced", "inputs": {"noise": ["10", 0], "guider": ["6", 0],
                                                                 "sampler": ["7", 0], "sigmas": ["8", 0],
                                                                 "latent_image": ["9", 0]}},
        "12": {"class_type": "VAEDecode", "inputs": {"samples": ["11", 0], "vae": ["3", 0]}},
        "13": {"class_type": "SaveImage", "inputs": {"images": ["12", 0], "filename_prefix": prefix}},
    }


def klein_edit_graph(image, prompt, *, seed, steps=KLEIN_STEPS, megapixels=KLEIN_MEGAPIXELS, prefix="localgen/edit",
                     clip=KLEIN_CLIP):
    """The Klein reference edit (comfy_run.py --edit and comfy_faces.py, the template's single-image path): `image`
    (a name in ComfyUI's input folder) scaled to `megapixels`, VAE-encoded and attached as a ReferenceLatent to both
    the prompt and its zeroed copy; the scheduler and the latent take the scaled image's size."""
    g = klein_graph(prompt, seed=seed, steps=steps, prefix=prefix, clip=clip)
    g.update({
        "20": {"class_type": "LoadImage", "inputs": {"image": image}},
        "21": {"class_type": "ImageScaleToTotalPixels", "inputs": {"image": ["20", 0], "upscale_method": "nearest-exact",
                                                                   "megapixels": megapixels, "resolution_steps": 1}},
        "22": {"class_type": "GetImageSize", "inputs": {"image": ["21", 0]}},
        "23": {"class_type": "VAEEncode", "inputs": {"pixels": ["21", 0], "vae": ["3", 0]}},
        "24": {"class_type": "ReferenceLatent", "inputs": {"conditioning": ["4", 0], "latent": ["23", 0]}},
        "25": {"class_type": "ReferenceLatent", "inputs": {"conditioning": ["5", 0], "latent": ["23", 0]}},
    })
    g["6"]["inputs"].update(positive=["24", 0], negative=["25", 0])
    for node in ("8", "9"):
        g[node]["inputs"].update(width=["22", 0], height=["22", 1])
    return g


def klein_prompt(text, style=""):
    """The whole prompt for Klein (its Qwen3 encoder reads it all): the character block first, then the style block
    (`style`, or the part of a whole recorded prompt before `Character:`) without its lines about an attached
    reference image (the whole line: the rest of it, "Do not copy her face", speaks of that image). Returns (prompt,
    cut), cut naming the dropped reference lines."""
    cut = []
    i = text.find("Character:")
    if i > 0:
        style, text = text[:i], text[i:]
    kept = []
    for line in style.strip().splitlines():
        if re.search(r"\battached image\b", line, re.I):
            cut.append(f"reference line: {line[:70]}...")
        elif line.strip():
            kept.append(line.strip())
    return "\n\n".join(p for p in (text.strip(), "\n".join(kept)) if p), cut


def face_prompt(expr):
    """The edit prompt of one PORTRAITFX1 expression cell."""
    return KEEP.format(e=expr, cell=CELLS[expr])


def klein_record(cid, prompt, n, *, kind, seed, steps=KLEIN_STEPS, clip=KLEIN_CLIP, size=None, megapixels=None,
                 prompts_file=None, reference=None, cut=(), seconds=None, when=None, change=None, expression=None):
    """The prompt record of one Klein take: generate.py's keys first (model KLEIN, reference = the edited image), then
    the backend settings."""
    when = when or datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    rec = {"model": KLEIN, "id": cid, "prompt": prompt, "reference": generate.rel(reference) if reference else None,
           "prompts_file": generate.rel(prompts_file) if prompts_file else None, "time": when, "n": n,
           "backend": "comfyui", "kind": kind, "unet": KLEIN_UNET, "clip": clip, "vae": KLEIN_VAE,
           "sampler": "euler", "steps": steps, "cfg": KLEIN_CFG}
    if reference is None:
        rec["size"] = size or KLEIN_SIZE
    else:
        rec["megapixels"] = megapixels or KLEIN_MEGAPIXELS
    rec.update(seed=seed, tokens_estimate=estimate(prompt), cut=list(cut), seconds=seconds)
    if change is not None:
        rec["change"] = change
    if expression is not None:
        rec["expression"] = expression
    return rec


def face_path(out, cid, expr, n, when=None):
    return pathlib.Path(out) / f"{cid}_face_{expr}_loc{int(when or time.time())}_{n}.png"


def multipart(fields, files):
    """A multipart/form-data body: `fields` name -> text, `files` name -> (file name, bytes). Returns (body, type)."""
    boundary = "----localgen" + uuid.uuid4().hex
    out = []
    for name, value in fields.items():
        out.append(f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"\r\n\r\n{value}\r\n'.encode())
    for name, (filename, data) in files.items():
        out.append(f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"; filename="{filename}"\r\n'
                   f"Content-Type: image/png\r\n\r\n".encode() + data + b"\r\n")
    out.append(f"--{boundary}--\r\n".encode())
    return b"".join(out), f"multipart/form-data; boundary={boundary}"


class Comfy:
    """The local ComfyUI server's HTTP API (standard library): check, upload an input image, queue a graph, wait for
    its history entry and fetch its image. `opener` (urllib.request.urlopen by default) is the one HTTP call, so tests
    fake it."""

    def __init__(self, url=COMFY_URL, home=None, opener=None, sleep=None, poll=3.0):
        self.url = url.rstrip("/")
        self.home = pathlib.Path(home or COMFY)
        self.opener = opener or urllib.request.urlopen
        self.sleep = sleep or time.sleep
        self.poll = poll

    def call(self, path, data=None, content_type=None):
        req = urllib.request.Request(self.url + path, data=data,
                                     headers={"Content-Type": content_type} if content_type else {})
        try:
            return self.opener(req, timeout=60).read()
        except urllib.error.HTTPError as e:
            raise SystemExit(f"ComfyUI rejected {path} ({e.code}): {e.read().decode('utf-8', 'replace')[:1500]}")
        except (urllib.error.URLError, OSError) as e:
            raise SystemExit(f"ComfyUI is not running at {self.url} ({getattr(e, 'reason', e)}): "
                             f"start {COMFY_START}")

    def json(self, path, body=None):
        data = None if body is None else json.dumps(body).encode("utf-8")
        return json.loads(self.call(path, data, "application/json" if data else None))

    def check(self):
        return self.json("/system_stats")

    def upload(self, path):
        """Uploads a source image to ComfyUI's input folder (as localgen_<file name>, so a user's own input of the
        same name stays); returns the name a LoadImage node takes."""
        path = pathlib.Path(path)
        body, ctype = multipart({"type": "input", "overwrite": "true"},
                                {"image": (f"localgen_{path.name}", path.read_bytes())})
        r = json.loads(self.call("/upload/image", body, ctype))
        return f"{r['subfolder']}/{r['name']}" if r.get("subfolder") else r["name"]

    def queue(self, graph):
        return self.json("/prompt", {"prompt": graph, "client_id": "localgen"})["prompt_id"]

    def wait(self, pid):
        while True:
            h = self.json(f"/history/{pid}")
            if pid in h:
                return h[pid]
            self.sleep(self.poll)

    def image(self, entry):
        """The first image of a finished job: from the ComfyUI output folder when it is there, else through /view."""
        im = next(im for o in entry.get("outputs", {}).values() for im in o.get("images", []))
        local = self.home / "output" / im.get("subfolder", "") / im["filename"]
        if local.exists():
            return local.read_bytes()
        q = urllib.parse.urlencode({"filename": im["filename"], "subfolder": im.get("subfolder", ""),
                                    "type": im.get("type", "output")})
        return self.call(f"/view?{q}")


def job_problem(entry):
    """Why a finished job has no image ("" when it has one): its status and execution error."""
    status = entry.get("status", {})
    has = any(o.get("images") for o in entry.get("outputs", {}).values())
    if status.get("status_str", "success") == "success" and has:
        return ""
    errs = [json.dumps(m[1])[:600] for m in status.get("messages", []) if m and m[0] == "execution_error"]
    return f"{status.get('status_str', 'no image')}: " + ("; ".join(errs) or "no image in its outputs")


def job_seconds(entry, fallback):
    """The run time ComfyUI recorded (execution_start to execution_success, ms), else `fallback`."""
    stamps = {m[0]: m[1].get("timestamp") for m in entry.get("status", {}).get("messages", [])
              if m and isinstance(m[1], dict)}
    if stamps.get("execution_start") and stamps.get("execution_success"):
        return round((stamps["execution_success"] - stamps["execution_start"]) / 1000, 1)
    return round(fallback, 1)


def klein_base(a, cid):
    """(prompt, cut, prompts_file) for one id of a Klein run: --prompt as given; else the id's block in --prompts FILE
    with its style block; else the record beside art_src/<id>.png; else the id's block in ART3_PROMPTS.md. --style
    replaces the style block."""
    if a.prompt:
        return a.prompt, [], None
    rec = SRC / f"{cid}.json"
    if not a.prompts and rec.exists():
        r = json.loads(rec.read_text(encoding="utf-8"))
        if r.get("prompt"):
            text = r["prompt"]
            if a.style and "Character:" in text:
                text = text[text.find("Character:"):]
            prompt, cut = klein_prompt(text, a.style or "")
            return prompt, cut, r.get("prompts_file") and ROOT / r["prompts_file"]
    path = pathlib.Path(a.prompts or ART3)
    blocks = block_prompts(path)
    if cid not in blocks:
        raise SystemExit(f"{cid}: no block in {path.name}")
    prompt, cut = klein_prompt(blocks[cid], a.style or style_block(path))
    return prompt, cut, path


def cmd_klein(a, kind):
    """run (kind text), vary (edit) and faces (face) on the ComfyUI backend: every job queued, then collected."""
    if a.lora or getattr(a, "strength", None):
        raise SystemExit("--lora and --strength are for --backend openvino (Klein edits take no strength)")
    out = pathlib.Path(a.out or SRC)
    steps = a.steps or KLEIN_STEPS
    when = int(time.time())
    jobs = []  # dicts: id, n, seed, prompt, record kwargs, source, path
    for cid in a.ids:
        if kind == "text":
            prompt, cut, pf = klein_base(a, cid)
            for k in range(a.n):
                jobs.append(dict(cid=cid, n=k + 1, seed=a.seed + k, prompt=prompt, source=None,
                                 path=take_path(out, cid, k + 1, when),
                                 rec=dict(kind="text", size=a.size or KLEIN_SIZE, prompts_file=pf, cut=cut)))
        elif kind == "edit":
            for k in range(a.n):
                jobs.append(dict(cid=cid, n=k + 1, seed=a.seed + k, prompt=a.change, source=pathlib.Path(a.source),
                                 path=take_path(out, cid, k + 1, when),
                                 rec=dict(kind="edit", megapixels=a.megapixels, change=a.change)))
        else:
            src = pathlib.Path(a.src or SRC) / f"{cid}.png"
            for e in a.exprs:
                for k in range(a.n):
                    jobs.append(dict(cid=cid, n=k + 1, seed=a.seed + k, prompt=face_prompt(e), source=src,
                                     path=face_path(out, cid, e, k + 1, when),
                                     rec=dict(kind="face", megapixels=a.megapixels, expression=e)))
    for j in jobs:
        j["rec"].update(steps=steps, clip=a.clip, reference=j["source"])
    if a.dry_run:
        for j in jobs:
            print(f"--- {j['cid']} #{j['n']} -> {j['path'].name} (about {estimate(j['prompt'])} tokens, whole)")
            print(json.dumps(klein_record(j["cid"], j["prompt"], j["n"], seed=j["seed"], **j["rec"]), indent=2,
                             ensure_ascii=False))
        return 0
    for j in jobs:
        if j["source"] is not None and not j["source"].exists():
            raise SystemExit(f"{j['source']}: no such image")
    client = Comfy(a.server)
    client.check()
    uploaded = {}
    for j in jobs:
        if j["source"] is not None and j["source"] not in uploaded:
            uploaded[j["source"]] = client.upload(j["source"])
    while any(j["path"].exists() for j in jobs):  # a second run in the same second
        when += 1
        for j in jobs:
            j["path"] = j["path"].with_name(re.sub(r"_loc\d+_", f"_loc{when}_", j["path"].name))
    for j in jobs:
        prefix = f"localgen/{j['path'].stem}"
        if j["source"] is None:
            g = klein_graph(j["prompt"], seed=j["seed"], steps=steps, size=a.size or KLEIN_SIZE, prefix=prefix,
                            clip=a.clip)
        else:
            g = klein_edit_graph(uploaded[j["source"]], j["prompt"], seed=j["seed"], steps=steps,
                                 megapixels=a.megapixels, prefix=prefix, clip=a.clip)
        j["pid"], j["t0"] = client.queue(g), time.time()
    print(f"queued {len(jobs)} job(s) on {client.url}", flush=True)
    made, failed = [], 0
    out.mkdir(parents=True, exist_ok=True)
    for j in jobs:
        entry = client.wait(j["pid"])
        problem = job_problem(entry)
        if problem:
            print(f"{j['path'].name}: FAILED {problem}", flush=True)
            failed += 1
            continue
        j["path"].write_bytes(client.image(entry))
        secs = job_seconds(entry, time.time() - j["t0"])
        write_record(j["path"], klein_record(j["cid"], j["prompt"], j["n"], seed=j["seed"], seconds=secs, **j["rec"]))
        made.append(j["path"])
        print(f"{j['path'].name}  {secs} s", flush=True)
    if made and a.sheet:
        sources = list(dict.fromkeys(j["source"] for j in jobs if j["source"] is not None))
        print("sheet", contact(a.sheet, sources + made))
    for cid, cut in {j["cid"]: j["rec"].get("cut") for j in jobs}.items():
        if cut:
            print(f"{cid}: left out of the prompt: " + "; ".join(cut))
    return 1 if failed else 0


# ------------------------------------------------------------------------------------------------------ contact sheet

def contact(out, images, cols=4, cell=240):
    """Each image beside its pixelize.py game portrait (96 px grid), a caption from its record below: one cell per
    take, `cols` takes a row."""
    from PIL import Image, ImageDraw
    import pixelize
    cap = 34
    rows = (len(images) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * cell * 2, rows * (cell + cap)), (12, 10, 12))
    draw = ImageDraw.Draw(sheet)
    for k, path in enumerate(images):
        path = pathlib.Path(path)
        x, y = (k % cols) * cell * 2, (k // cols) * (cell + cap)
        rec = path.with_suffix(".json")
        r = json.loads(rec.read_text(encoding="utf-8")) if rec.exists() else {}
        local = str(r.get("model", "")).startswith("local:")
        with Image.open(path) as im:
            sheet.paste(im.convert("RGB").resize((cell, cell)), (x, y))
        px, _ = pixelize.pixelize(str(path), 96, mark=not local)  # an image-API source has the star mark to clear
        sheet.paste(px.convert("RGB").resize((cell, cell), Image.NEAREST), (x + cell, y))
        bits = [r.get("model")] if r and not local else []
        if local:
            bits += [r.get("device") or r.get("expression") or "klein", f"seed {r.get('seed')}", f"{r.get('steps')} st"]
            if r.get("strength") is not None:
                bits.append(f"str {r['strength']}")
            for ad in r.get("lora") or []:
                bits.append(f"lora {pathlib.Path(ad['path']).stem[:12]}:{ad['alpha']}")
            if r.get("seconds") is not None:
                bits.append(f"{r['seconds']} s")
        label = f"{path.stem[:40]}\n" + "  ".join(str(b) for b in bits if b)
        draw.multiline_text((x + 4, y + cell + 2), label, fill=(220, 210, 190))
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    return out


# ------------------------------------------------------------------------------------------------------------ commands

def encoder(a):
    """(family, token counter, token limit) of the run's model: the CLIP count and 75 for the CLIP-only families
    (they take the short form), the T5 estimate and 256 for FLUX (the full prompt)."""
    fam = family(a.model)
    if FAMILIES[fam]["short"]:
        return fam, counter(a.model), TOKENS
    return fam, t5_estimate, T5_TOKENS


def base_prompt(a, cid, enc):
    """(prompt, cut, prompts_file) for one id: --prompt as given, else the record beside --from (vary), else the id's
    block in the prompts file; shortened for a CLIP-only model (short_prompt), whole for FLUX (full_prompt)."""
    fam, count, tokens = enc
    short = FAMILIES[fam]["short"]
    trigger = a.trigger
    change = getattr(a, "change", None)
    style = a.style or TAIL

    def build(text, path=None):
        if short:
            return short_prompt(text, count, change=change, trigger=trigger, style=style)
        return full_prompt(text, style_block(path) if path else "", change=change, trigger=trigger)

    if a.prompt:
        sep = ", " if short else "\n"
        text = sep.join(t for t in (change, a.prompt, trigger) if t)
        return text, overflow(text, count, tokens), None
    src = getattr(a, "source", None)
    if src and not a.prompts and pathlib.Path(src).with_suffix(".json").exists():
        rec = json.loads(pathlib.Path(src).with_suffix(".json").read_text(encoding="utf-8"))
        if rec.get("prompt"):
            prompt, cut = build(rec["prompt"])
            return prompt, cut, rec.get("prompts_file") and ROOT / rec["prompts_file"]
    path = pathlib.Path(a.prompts or ART3)
    blocks = block_prompts(path)
    if cid not in blocks:
        raise SystemExit(f"{cid}: no block in {path.name}")
    prompt, cut = build(blocks[cid], path)
    return prompt, cut, path


def settings(a):
    """The device, the model family's defaults for what was not given (steps, size, guidance) and the LoRA list."""
    device = a.device.upper()
    if device not in DEVICES:
        raise SystemExit(f"--device: one of {', '.join(DEVICES)}")
    fam = FAMILIES[family(a.model)]
    for key in ("steps", "size", "guidance"):
        if getattr(a, key) is None:
            setattr(a, key, fam[key])
    return device, [parse_lora(spec) for spec in a.lora]


def cmd_run(a, kind="text"):
    enc = encoder(a)
    fam, count, tokens = enc
    device, lora = settings(a)
    out = pathlib.Path(a.out or SRC)
    jobs = []  # (id, prompt, cut, prompts_file, seed, strength, n)
    if kind == "text":
        for cid in a.ids:
            prompt, cut, pf = base_prompt(a, cid, enc)
            jobs += [(cid, prompt, cut, pf, a.seed + k, None, k + 1) for k in range(a.n)]
    else:
        prompt, cut, pf = base_prompt(a, a.ids[0], enc)
        strengths = a.strength or [0.5]
        jobs += [(a.ids[0], prompt, cut, pf, a.seed + k, s, len(strengths) * k + j + 1)
                 for k in range(a.n) for j, s in enumerate(strengths)]
    source = None
    if kind == "vary" and not a.dry_run:
        from PIL import Image
        with Image.open(a.source) as im:
            source = im.convert("RGB")
    pipe = None if a.dry_run else Pipe(kind, a.model, device, a.size, a.guidance, lora)
    if pipe:
        print(f"loaded {model_name(a.model)} ({fam}) on {device} in {pipe.load:.1f} s")
    made = []
    for cid, prompt, cut, pf, seed, strength, n in jobs:
        kw = dict(model=a.model, device=device, seed=seed, steps=a.steps, size=a.size, guidance=a.guidance,
                  prompts_file=pf, lora=lora, cut=cut)
        if kind == "vary":
            kw.update(source_image=a.source, strength=strength, change=a.change)
        if a.dry_run:
            print(f"--- {cid} #{n} ({count(prompt)} tokens of {tokens})")
            print(json.dumps(make_record(cid, prompt, n, **kw), indent=2, ensure_ascii=False))
            continue
        img, secs = pipe.generate(prompt, size=a.size, steps=a.steps, seed=seed, guidance=a.guidance,
                                  negative=NEGATIVE if FAMILIES[fam]["negative"] else None, image=source,
                                  strength=strength, max_sequence_length=None if FAMILIES[fam]["short"] else tokens)
        out.mkdir(parents=True, exist_ok=True)
        path = take_path(out, cid, n)
        while path.exists():
            path = take_path(out, cid, n, int(path.stem.split("_loc")[1].split("_")[0]) + 1)
        img.save(path)
        write_record(path, make_record(cid, prompt, n, seconds=round(secs, 2), **kw))
        made.append(path)
        print(f"{path.name}  {secs:.1f} s")
    if made and a.sheet:
        print("sheet", contact(a.sheet, ([a.source] if kind == "vary" else []) + made))  # a variant's source first
    for cid, cut in {j[0]: j[2] for j in jobs}.items():
        if cut:
            print(f"{cid}: cut from the prompt: " + "; ".join(cut))
    return 0


def cmd_prompt(a):
    if a.backend == "comfyui":
        for cid in a.ids:
            prompt, cut, _ = klein_base(a, cid)
            print(f"{cid} (klein, whole, about {estimate(prompt)} tokens): {prompt}")
            for c in cut:
                print(f"  left out: {c}")
        return 0
    enc = encoder(a)
    fam, count, tokens = enc
    for cid in a.ids:
        prompt, cut, _ = base_prompt(a, cid, enc)
        print(f"{cid} ({fam}, {count(prompt)} tokens of {tokens}): {prompt}")
        for c in cut:
            print(f"  cut: {c}")
    return 0


def parser():
    ap = argparse.ArgumentParser(prog="localgen.py", description="Local image generation (LOCALGEN1, LOCALGEN2)")
    sub = ap.add_subparsers(dest="cmd", required=True)

    def common(p, gen=True):
        p.add_argument("--backend", choices=("comfyui", "openvino"), default="comfyui",
                       help="comfyui: FLUX.2 Klein 4B on the local ComfyUI server (default); openvino: quick drafts")
        p.add_argument("--prompts", help="prompts file (default ART3_PROMPTS.md)")
        p.add_argument("--prompt", help="the prompt as given, instead of a block")
        p.add_argument("--trigger", help="(openvino) LoRA trigger words, added to the style tail")
        p.add_argument("--style", help="the style tail of a short prompt (default TAIL); for Klein the style block")
        p.add_argument("--model", default=str(MODEL), help="(openvino) the model folder")
        if not gen:
            return
        p.add_argument("--server", default=COMFY_URL, help="(comfyui) the server URL (env LOCALGEN_COMFY_URL)")
        p.add_argument("--clip", default=KLEIN_CLIP, help="(comfyui) the Qwen3 text encoder file")
        p.add_argument("--megapixels", type=float, default=KLEIN_MEGAPIXELS, help="(comfyui) an edit's image size")
        p.add_argument("--n", type=int, default=1)
        p.add_argument("--seed", type=int, default=1)
        p.add_argument("--steps", type=int, help="default: Klein's 4, or the model family's (FAMILIES)")
        p.add_argument("--size", type=int, help="default: Klein's 1024, or the model family's (FAMILIES)")
        # GPU by default: 2026-10-26 it made a 512 px take in 0.8-1.0 s against the NPU's 2.1 s, and it compiles a new
        # size or LoRA in seconds where the NPU takes 3.5 min (a size) to 16 min (a LoRA); NPU for long fixed batches
        p.add_argument("--device", default="GPU")
        p.add_argument("--guidance", type=float, help="default: the model family's (FAMILIES)")
        p.add_argument("--lora", action="append", default=[], metavar="PATH:ALPHA")
        p.add_argument("--out", help="output folder (default art_src/)")
        p.add_argument("--sheet", help="also write a contact sheet of the takes")
        p.add_argument("--dry-run", action="store_true")

    r = sub.add_parser("run"); r.add_argument("ids", nargs="+"); common(r)
    v = sub.add_parser("vary"); v.add_argument("ids", nargs=1); common(v)
    v.add_argument("--from", dest="source", required=True)
    v.add_argument("--change", required=True)
    v.add_argument("--strength", type=float, action="append", help="(openvino)")
    f = sub.add_parser("faces"); f.add_argument("ids", nargs="+"); common(f)
    f.add_argument("--exprs", nargs="+", choices=list(CELLS), default=list(CELLS))
    f.add_argument("--src", help="folder of the source portraits <id>.png (default art_src/)")
    s = sub.add_parser("sheet"); s.add_argument("out"); s.add_argument("images", nargs="+")
    s.add_argument("--cols", type=int, default=4)
    p = sub.add_parser("prompt"); p.add_argument("ids", nargs="+"); common(p, gen=False)
    k = sub.add_parser("lora-kohya"); k.add_argument("src"); k.add_argument("dst")
    return ap


def main(argv=None):
    a = parser().parse_args(argv)
    comfy = getattr(a, "backend", None) == "comfyui"
    if a.cmd == "run":
        return cmd_klein(a, "text") if comfy else cmd_run(a)
    if a.cmd == "vary":
        if comfy:
            return cmd_klein(a, "edit")
        if any(not 0.0 < s <= 1.0 for s in a.strength or []):
            raise SystemExit("--strength: between 0 and 1 (0.3-0.7 keeps the face)")
        return cmd_run(a, "vary")
    if a.cmd == "faces":
        if not comfy:
            raise SystemExit("faces: a Klein edit, ComfyUI backend only")
        return cmd_klein(a, "face")
    if a.cmd == "sheet":
        print(contact(a.out, a.images, a.cols))
        return 0
    if a.cmd == "lora-kohya":
        print(f"{a.dst}: {convert_lora(a.src, a.dst)} keys renamed")
        return 0
    return cmd_prompt(a)


if __name__ == "__main__":
    sys.exit(main())
