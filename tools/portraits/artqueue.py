# Art queue (ARTQUEUE1, decision "Antigravity image queue", user 2026-11-02): image runs through the Antigravity image
# subagent instead of the image API. This side writes request files into the file queue `.image_pipeline/` of the main
# checkout (the same folder from every worktree) and ingests what Antigravity leaves in output/ into art_src/ with the
# house naming and a prompt record beside each image. It never calls an image API and never reads a key. The contract
# (folders, request and result files) is ART_PROMPT_TEMPLATES.md "Antigravity queue". Pillow only (via pixelize).
#   python artqueue.py request <id>... [--prompts <file> | --file icons] [--faces] [--no-ref] [--ref <png>]...
#       [--n N] [--aspect 1:1|16:9|9:16|4:3|3:4] [--route R] [--task T] [--target <path>] [--record KEY=VALUE]...
#       [--dry-run]
#       one request per id from a prompts file laid out like generate.py's (default ART2_PROMPTS.md; --faces reads
#       PORTRAITFX1_FACES.md); references as generate.py sends them (the section's `reference:` image, else Mara
#       Voss; --no-ref none), then each --ref in order
#   python artqueue.py text <name> (--text "..." | --text-file <file>) [--id ID] [--ref <png>]... [same options]
#       one request with a prompt given whole (a template prompt from ART_PROMPT_TEMPLATES.md, filled in), named
#       <name>_q<time>; the asset id is --id, else <name>
# Every request name is unique (<id>_q<time>, a second later while taken in any folder); a reference that does not
# exist refuses the request. Ingest refuses an image that is missing, corrupt, off the aspect by more than 3 % or
# under 1024 px on its long edge, and reports a request moved to failed/ with its error.txt.
#   python artqueue.py ingest [<name>...] [--dry-run]
#       every finished request (output/<name>.result.json and completed/<name>.json), or the names given
#   python artqueue.py status
#       counts per folder, failed errors, and the quota pause (ARTQUEUE2): Antigravity writes paused.json
#       {"paused_until": "<ISO UTC>", "reason", "request"} on HTTP 429 and puts the request back in queue/;
#       request still queues during a pause (and says so)
#   python artqueue.py attach
#       the refs/ file names to attach by hand in the Gemini web-app task, for the requests in queue/ (unique, sorted;
#       requests whose reference is a session reference, --record session_ref=<request name>, are left out)
# The default queue folder is the Drive folder of artqueue.json ("pipeline"; ARTQUEUE5, the Gemini web app is the
# primary route), or AEGIS_IMAGE_PIPELINE (IMAGE_PIPELINE too); it must exist. --backup picks the Antigravity folder
# (.image_pipeline of the main checkout); --pipeline <dir> any other folder (tests, a scratch run; created on use).
# request and text copy every reference into <queue>/refs/ (a different file of the same name gets a suffix) and list
# the copies in the request's references; the original paths go to metadata.record.source_refs.
import argparse
import datetime
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import time

from PIL import Image

import generate as G
import pixelize as PX

FOLDERS = ("queue", "processing", "output", "completed", "failed")
ARCHIVE = "archive"              # ours: archive/<UTC date>/{output,completed}/ of ingested requests (Antigravity never reads it)
REJECTED = "rejected"            # ours: images ingest refused, each with a .note.txt
PIXELIZED = "pixelized"          # ours: the game portraits of route portrait requests with no target
ASPECTS = {"1:1": 1.0, "16:9": 16 / 9, "9:16": 9 / 16, "4:3": 4 / 3, "3:4": 3 / 4}
ASPECT_TOL = 0.03                # relative difference of width/height from the ratio still accepted (Gemini's
                                 # 4:3 and 3:4 sizes, 1184x864 and 864x1184, are about 2.7 % off)
MIN_EDGE = 1024                  # the long edge of a full-size image (a smaller one is a preview or a partial render)
# Routes (ART_PROMPT_TEMPLATES.md "Routes by asset type"): only a portrait is pixelized at ingest; a faces sheet is cut
# by sheets.py faces and the rest is left for the asset lane.
ROUTES = ("portrait", "faces", "full_body", "story_scene", "icon", "icon_sheet", "other")
# The record's `source` and `via` follow the queue the request went through (queue_origin); `model` keeps what the result
# names (often "image-generator"). Records written before LOOK1-T2 keep what they say.
SOURCE = "antigravity image subagent"
VIA = "antigravity"
WEB_SOURCE = "gemini web app task"
WEB_VIA = "gemini_web"
BACKUP_FOLDER = ".image_pipeline"  # the Antigravity queue's folder, in the main checkout
# Record keys the ingest writes itself; a request's metadata.record cannot replace them.
RESERVED = {"model", "id", "cid", "prompt", "use_ref", "prompts_file", "n", "time", "when", "reference", "image",
            "references", "source", "via", "request", "metadata", "mark_cleared"}


def main_checkout():
    """The main checkout's root (the parent of the common .git folder), so a worktree finds the same queue."""
    try:
        out = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=G.ROOT,
                             capture_output=True, text=True, check=True).stdout.strip()
        return pathlib.Path(out).parent
    except (OSError, subprocess.CalledProcessError):
        return G.ROOT


CONFIG = pathlib.Path(__file__).resolve().parent / "artqueue.json"   # {"pipeline": "<Drive folder>"}
REFS = "refs"                    # the references of the requests, copied beside the queue so the web app can attach them
SESSION_REF = "session_ref"      # record key: the reference is an image of the web-app session, nothing to attach


def pipeline(override=None, backup=False, config=None):
    """The queue folder: --pipeline, else the Antigravity backup (--backup), else AEGIS_IMAGE_PIPELINE / IMAGE_PIPELINE,
    else artqueue.json's `pipeline` (the Drive folder). The configured one must exist (no silent fallback)."""
    if override:
        return pathlib.Path(override)
    if backup:
        return main_checkout() / BACKUP_FOLDER
    env = os.environ.get("AEGIS_IMAGE_PIPELINE") or os.environ.get("IMAGE_PIPELINE")
    where, src = env, "AEGIS_IMAGE_PIPELINE"
    if not where:
        cfg = pathlib.Path(config or CONFIG)
        try:
            where = json.loads(cfg.read_text(encoding="utf-8")).get("pipeline")
        except (OSError, ValueError, AttributeError):
            where = None
        src = cfg.name
    if not where:
        raise SystemExit(f"no image pipeline folder: set \"pipeline\" in {CONFIG}, or AEGIS_IMAGE_PIPELINE, or pass "
                         f"--pipeline / --backup")
    path = pathlib.Path(where)
    if not path.is_dir():
        raise SystemExit(f"the image pipeline folder {path} (from {src}) does not exist: is the Drive mounted? "
                         f"(--backup uses the Antigravity folder, --pipeline any other)")
    return path


def is_backup(pipe):
    """Whether a queue folder is the Antigravity one, however it was named on the command line (`.`, a trailing slash,
    another case)."""
    return pathlib.Path(pipe).resolve().name.casefold() == BACKUP_FOLDER


def queue_origin(pipe):
    """The record's (source, via) for a queue folder: the Antigravity values for the backup folder, the web-app values
    for any other."""
    return (SOURCE, VIA) if is_backup(pipe) else (WEB_SOURCE, WEB_VIA)


def queue_label(pipe):
    """The worker of a queue folder, for messages."""
    return "Antigravity" if is_backup(pipe) else "the Gemini web app"


def ensure(pipe):
    for f in FOLDERS:
        (pipe / f).mkdir(parents=True, exist_ok=True)


def write_json(path, data):
    """Writes <file>.tmp, then renames it to <file>, so a scanner (which reads *.json) never reads half a request."""
    tmp = path.with_name(f"{path.name}.tmp")
    tmp.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    os.replace(tmp, path)


def taken(pipe, name):
    return any((pipe / f / f"{name}.json").exists() for f in FOLDERS)


def request_name(pipe, cid, kind=None, now=None):
    """`<id>_q<time>` (`<id>_faces_q<time>` for a faces sheet), a second later while the name is taken anywhere."""
    now = int(time.time()) if now is None else now
    stem = f"{cid}_faces" if kind == "faces" else cid
    while taken(pipe, f"{stem}_q{now}"):
        now += 1
    return f"{stem}_q{now}"


def build_request(prompt, cid, refs=(), n=1, aspect="1:1", route="portrait", task=None, target=None, record=None):
    """The request body of the contract: the exact prompt, no style hints, absolute reference paths in order."""
    if not prompt or not prompt.strip():
        raise ValueError("empty prompt")
    if aspect not in ASPECTS:
        raise ValueError(f"aspect {aspect} is not one of {', '.join(ASPECTS)}")
    if route not in ROUTES:
        raise ValueError(f"route {route} is not one of {', '.join(ROUTES)}")
    if n < 1:
        raise ValueError("n must be at least 1")
    return {"prompt": prompt, "aspect_ratio": aspect, "style_hints": "",
            "references": [pathlib.Path(r).resolve().as_posix() for r in refs], "n": n,
            "metadata": {"asset_id": cid, "task": task, "target": target, "route": route, "record": record or {}}}


def enqueue(pipe, name, req, dry=False):
    for r in req["references"]:
        if not pathlib.Path(r).is_file():
            raise FileNotFoundError(f"no reference image {r}")
    path = pipe / "queue" / f"{name}.json"
    if req["references"]:
        req["metadata"]["record"]["source_refs"] = list(req["references"])
    if dry:
        print(f"{name}: would queue {G.rel(path)}: prompt {len(req['prompt'])} chars, n {req['n']}, "
              f"aspect {req['aspect_ratio']}, route {req['metadata']['route']}, references "
              f"{[G.rel(r) for r in req['references']] or None}")
        return path
    ensure(pipe)
    if taken(pipe, name):
        raise FileExistsError(f"request {name} exists")
    req["references"] = [store_ref(pipe, r) for r in req["references"]]
    write_json(path, req)
    print(f"{name}: queued, n {req['n']}, {len(req['references'])} reference(s)")
    return path


def store_ref(pipe, ref):
    """Copies a reference into <pipe>/refs/ and returns the stored path. The same file again is reused; a different
    file of the same name is stored as <stem>_2, _3, ... (the stored name is what the request lists)."""
    src = pathlib.Path(ref)
    dest_dir = pipe / REFS
    dest_dir.mkdir(parents=True, exist_ok=True)
    dest, k = dest_dir / src.name, 1
    while dest.exists() and dest.read_bytes() != src.read_bytes():
        k += 1
        dest = dest_dir / f"{src.stem}_{k}{src.suffix}"
    if not dest.exists():
        shutil.copyfile(src, dest)
    return dest.resolve().as_posix()


def to_attach(pipe):
    """The refs/ file names the queued requests need attached by hand: unique, sorted, session references left out."""
    names = set()
    for p in sorted((pipe / "queue").glob("*.json")) if (pipe / "queue").is_dir() else []:
        try:
            req = json.loads(p.read_text(encoding="utf-8"))
        except ValueError:
            continue
        if (req.get("metadata") or {}).get("record", {}).get(SESSION_REF):
            continue
        names.update(re.split(r"[\/]", r)[-1] for r in req.get("references") or [])
    return sorted(names)


# ---- ingest ----

def find_request(pipe, name):
    for f in ("completed", "processing", "queue", "failed"):
        p = pipe / f / f"{name}.json"
        if p.exists():
            return f, p
    return None, None


def check_image(path, aspect):
    """None when `path` is a whole PNG of the asked aspect, else the reason it is refused."""
    if not path.is_file():
        return "missing"
    try:
        with Image.open(path) as im:
            im.verify()
        with Image.open(path) as im:
            im.load(); w, h = im.size; fmt = im.format
    except Exception as e:  # noqa: BLE001 - any decoder failure is a corrupt file
        return f"corrupt ({type(e).__name__}: {str(e)[:120]})"
    if fmt != "PNG":
        return f"not a PNG ({fmt})"
    want = ASPECTS.get(aspect, 1.0)
    if h == 0 or abs(w / h - want) / want > ASPECT_TOL:
        return f"wrong aspect {w}x{h} for {aspect}"
    if max(w, h) < MIN_EDGE:
        return f"too small {w}x{h} (long edge under {MIN_EDGE})"
    return None


def expected_files(name, n):
    return [f"{name}.png"] if n == 1 else [f"{name}_{k + 1}.png" for k in range(n)]


def free_name(cid, k, n, kind, now):
    """generate.out_name, a second later while that file exists (two takes of one id ingested in the same second)."""
    while True:
        out = G.out_name(cid, k, n, kind, now)
        if not (G.SRC / out).exists():
            return out
        now += 1


def clear_model_mark(path):
    """The image model's corner star cleared where it sits on the background (pixelize.clear_mark)."""
    with Image.open(path) as im:
        im = im.convert("RGBA" if "A" in im.getbands() else "RGB")
    PX.clear_mark(im)
    im.save(path)


def portrait_out(pipe, art, target):
    """Where the pixelized portrait goes: the request's target when free, else pipeline/pixelized/<art stem>.png."""
    if target:
        t = pathlib.Path(target)
        t = t if t.is_absolute() else G.ROOT / t
        if not t.exists():
            return t
    return pipe / PIXELIZED / f"{art.stem}.png"


def failed_error(pipe, name):
    e = pipe / "failed" / f"{name}.error.txt"
    return e.read_text(encoding="utf-8", errors="replace").strip() if e.exists() else "(no error.txt)"


def report_failed(pipe, name):
    print(f"{name}: failed in {queue_label(pipe)}: {failed_error(pipe, name)[:300]}")


def failed_names(pipe):
    f = pipe / "failed"
    return sorted(p.stem for p in f.glob("*.json")) if f.is_dir() else []


def archive_dir(pipe, day=None):
    return pipe / ARCHIVE / (day or datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d"))


def archive_markers(pipe, name):
    return sorted((pipe / ARCHIVE).glob(f"*/output/{name}.ingested.json")) if (pipe / ARCHIVE).is_dir() else []


def move_unique(src, dest_dir):
    """Moves src into dest_dir, never overwriting: a name that exists gets _2, _3, ... before the extension(s)."""
    dest_dir.mkdir(parents=True, exist_ok=True)
    dest = dest_dir / src.name
    k = 1
    while dest.exists():
        k += 1
        stem, dot, ext = src.name.partition(".")
        dest = dest_dir / f"{stem}_{k}{dot}{ext}"
    shutil.move(str(src), str(dest))
    return dest


def archive_request(pipe, name, files=(), day=None):
    """Moves a finished request into archive/<day>/: its leftover images, result and marker from output/, its request
    from completed/. Returns the number of files moved."""
    base = archive_dir(pipe, day)
    moved = 0
    out = pipe / "output"
    for fname in (*files, f"{name}.result.json", f"{name}.ingested.json"):
        p = out / fname
        if p.is_file():
            move_unique(p, base / "output"); moved += 1
    req = pipe / "completed" / f"{name}.json"
    if req.is_file():
        move_unique(req, base / "completed"); moved += 1
    return moved


def archive_sweep(pipe, dry=False):
    """Archives every output/ request that has a marker (ingests from before the archive existed)."""
    out = pipe / "output"
    total = 0
    for m in sorted(out.glob("*.ingested.json")) if out.is_dir() else []:
        name = m.name[:-len(".ingested.json")]
        files = []
        try:
            d = json.loads(m.read_text(encoding="utf-8"))
            files = [e["file"] for e in (d.get("accepted") or []) + (d.get("rejected") or []) if e.get("file")]
        except (ValueError, TypeError, AttributeError):
            pass
        files = list(dict.fromkeys([*files, *(p.name for p in out.glob(f"{name}.png")),
                                    *(p.name for p in out.glob(f"{name}_[0-9]*.png"))]))
        if dry:
            n = sum(1 for f in (*files, f"{name}.result.json", f"{name}.ingested.json") if (out / f).is_file())
            n += (pipe / "completed" / f"{name}.json").is_file()
            print(f"{name}: would archive {n} files")
            total += n
        else:
            n = archive_request(pipe, name, files)
            print(f"{name}: archived {n} files")
            total += n
    return total


def ingest_one(pipe, name, dry=False, now=None):
    """Ingests output/<name>.result.json: each accepted image into art_src/ with its record (mark cleared; route
    portrait pixelized), each refused one moved to rejected/ with a note; output/<name>.ingested.json marks it done.
    Returns the marker dict (None when the request is not finished or was ingested already)."""
    out = pipe / "output"
    result_path = out / f"{name}.result.json"
    if (out / f"{name}.ingested.json").exists() or archive_markers(pipe, name):
        print(f"{name}: ingested already"); return None
    if (pipe / "failed" / f"{name}.json").exists():
        report_failed(pipe, name); return None
    if not result_path.exists():
        print(f"{name}: no result yet"); return None
    where, req_path = find_request(pipe, name)
    if where == "failed":
        report_failed(pipe, name); return None
    if where != "completed":
        print(f"{name}: request is in {where or 'no folder'}, not completed/; left"); return None
    try:
        result = json.loads(result_path.read_text(encoding="utf-8"))
    except ValueError as e:
        print(f"{name}: unreadable result ({e}); left"); return None
    req = json.loads(req_path.read_text(encoding="utf-8"))
    meta = req.get("metadata") or {}
    cid = meta.get("asset_id") or name
    route = meta.get("route") or "other"
    kind = "faces" if route == "faces" else (meta.get("record") or {}).get("kind")
    n = int(req.get("n") or 1)
    aspect = req.get("aspect_ratio") or "1:1"
    listed = [f.get("file") for f in result.get("files") or [] if f.get("file")]
    names = list(dict.fromkeys(expected_files(name, n) + [re.split(r"[\\/]", f)[-1] for f in listed]))
    now = int(time.time()) if now is None else now
    prompt_sent = result.get("prompt_sent")
    marker = {"request": name, "time": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
              "accepted": [], "rejected": []}
    refs = result.get("references", req.get("references")) or []
    for k, fname in enumerate(names):
        src = out / fname
        why = check_image(src, aspect)
        if why:
            marker["rejected"].append({"file": fname, "reason": why})
            print(f"{name}: {fname} refused: {why}")
            if not dry and src.exists():
                rej = pipe / REJECTED; rej.mkdir(parents=True, exist_ok=True)
                shutil.move(str(src), str(rej / fname))
                (rej / f"{fname}.note.txt").write_text(f"{name}: {why}\n", encoding="utf-8", newline="\n")
            continue
        dest = G.SRC / free_name(cid, k, len(names), kind, now)
        entry = {"file": fname, "art_src": G.rel(dest)}
        if not dry:
            G.SRC.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dest)
            clear_model_mark(dest)
            source, via = queue_origin(pipe)
            extra = {"references": [G.rel(r) for r in refs], "source": source, "via": via, "request": name,
                     "metadata": meta, "mark_cleared": True}
            extra.update({key: v for key, v in (meta.get("record") or {}).items() if key not in RESERVED})
            if kind:
                extra["kind"] = kind
            if prompt_sent is not None and prompt_sent != req.get("prompt"):
                extra["prompt_requested"] = req.get("prompt")
            if result.get("notes"):
                extra["notes"] = result["notes"]
            pf = (meta.get("record") or {}).get("prompts_file")
            G.record(dest, result.get("model") or via, cid,
                     prompt_sent if prompt_sent is not None else req.get("prompt"), bool(refs),
                     G.ROOT / pf if pf else None, k + 1, when=result.get("time"),
                     reference=refs[0] if refs else None, **extra)
            if route == "portrait":
                px = portrait_out(pipe, dest, meta.get("target"))
                px.parent.mkdir(parents=True, exist_ok=True)
                img, _ = PX.pixelize(str(dest), faction=PX.faction_of(str(dest)),
                                     reduce=PX.reduce_of(str(dest)))
                img.save(px)
                entry["pixelized"] = G.rel(px)
        marker["accepted"].append(entry)
        print(f"{name}: {fname} -> {entry['art_src']}" + (f", portrait {entry['pixelized']}" if "pixelized" in entry
                                                         else ""))
    if not dry:
        write_json(out / f"{name}.ingested.json", marker)
        archive_request(pipe, name, names)
    return marker


def finished(pipe):
    out = pipe / "output"
    if not out.is_dir():
        return []
    return sorted(p.name[:-len(".result.json")] for p in out.glob("*.result.json")
                  if not (out / p.name.replace(".result.json", ".ingested.json")).exists())


def status(pipe):
    """Counts per folder: requests waiting, being made, results not ingested yet, done, failed, refused images."""
    count = lambda f, pat: len(list((pipe / f).glob(pat))) if (pipe / f).is_dir() else 0
    s = {"queue": count("queue", "*.json"), "processing": count("processing", "*.json"),
         "output": len(finished(pipe)), "ingested": count("output", "*.ingested.json"),
         "completed": count("completed", "*.json"), "failed": count("failed", "*.json"),
         "rejected": count(REJECTED, "*.png"),
         "archived": len(list((pipe / ARCHIVE).glob("*/output/*.ingested.json"))) if (pipe / ARCHIVE).is_dir() else 0}
    return s


def pause_state(pipe, now=None):
    """The quota pause (ARTQUEUE2): None without paused.json, else (line, active) with a printable line."""
    path = pipe / "paused.json"
    if not path.is_file():
        return None
    try:
        d = json.loads(path.read_text(encoding="utf-8"))
        until = datetime.datetime.fromisoformat(str(d["paused_until"]).replace("Z", "+00:00"))
        if until.tzinfo is None:
            until = until.replace(tzinfo=datetime.timezone.utc)
    except (ValueError, KeyError, TypeError, OSError) as e:
        return f"paused.json unreadable ({type(e).__name__}: {e})", False
    now = now or datetime.datetime.now(datetime.timezone.utc)
    shown = until.astimezone(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M")
    reason = d.get("reason") or "no reason given"
    if until <= now:
        return f"pause expired ({shown} UTC): {reason}", False
    mins = int((until - now).total_seconds() // 60) + 1
    return f"paused until {shown} UTC ({mins} min): {reason}", True


def note_pause(pipe):
    p = pause_state(pipe)
    if p and p[1]:
        print(f"note: Antigravity is {p[0]}; requests wait in queue/ until then")


# ---- CLI ----

def common(p):
    p.add_argument("--n", type=int, default=1)
    p.add_argument("--aspect", choices=tuple(ASPECTS), default="1:1")
    p.add_argument("--route", choices=ROUTES)
    p.add_argument("--task")
    p.add_argument("--target", help="where the asset lane wants the result (portrait: the pixelized portrait)")
    p.add_argument("--ref", action="append", default=[], help="another reference image, attached after the others")
    p.add_argument("--record", action="append", default=[], metavar="KEY=VALUE",
                   help="a field for the prompt record (e.g. faction=castellan; pixelize reads faction)")
    p.add_argument("--dry-run", action="store_true")


def main(argv=None):
    ap = argparse.ArgumentParser(description="The Antigravity image queue (ARTQUEUE1).")
    ap.add_argument("--pipeline", help="the queue folder (default: artqueue.json's Drive folder)")
    ap.add_argument("--backup", action="store_true", help="the Antigravity folder (.image_pipeline of the main checkout)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("request"); r.add_argument("ids", nargs="+"); common(r)
    r.add_argument("--prompts"); r.add_argument("--file", choices=("icons",)); r.add_argument("--faces",
                                                                                            action="store_true")
    r.add_argument("--no-ref", action="store_true")
    t = sub.add_parser("text"); t.add_argument("name"); common(t)
    t.add_argument("--text"); t.add_argument("--text-file"); t.add_argument("--id")
    i = sub.add_parser("ingest"); i.add_argument("names", nargs="*"); i.add_argument("--dry-run", action="store_true")
    sub.add_parser("status")
    sub.add_parser("attach")
    ar = sub.add_parser("archive"); ar.add_argument("--dry-run", action="store_true")
    a = ap.parse_args(argv)
    pipe = pipeline(a.pipeline, a.backup)
    if a.cmd == "attach":
        names = to_attach(pipe)
        print(chr(10).join(names) if names else "nothing to attach")
        return 0
    if a.cmd == "status":
        s = status(pipe)
        print(f"{G.rel(pipe)}: " + ", ".join(f"{k} {v}" for k, v in s.items()))
        p = pause_state(pipe)
        if p:
            print(p[0])
        for e in sorted((pipe / "failed").glob("*.error.txt")) if (pipe / "failed").is_dir() else []:
            print(f"  failed {e.name[:-len('.error.txt')]}: {e.read_text(encoding='utf-8').strip()[:160]}")
        return 0
    if a.cmd == "archive":
        n = archive_sweep(pipe, a.dry_run)
        print(f"{'would archive' if a.dry_run else 'archived'} {n} files")
        return 0
    if a.cmd == "ingest":
        for name in a.names or finished(pipe):
            ingest_one(pipe, name, a.dry_run)
        if not a.names:
            for name in failed_names(pipe):
                report_failed(pipe, name)
        return 0
    if a.n < 1:
        sys.exit("--n must be at least 1")
    if not a.dry_run:
        note_pause(pipe)
    extra_refs = [pathlib.Path(x).resolve() for x in a.ref]
    fields = {}
    for kv in a.record:
        key, sep, v = kv.partition("=")
        if not sep or not key or key in RESERVED:
            sys.exit(f"--record {kv}: give KEY=VALUE, KEY not one of the record's own keys")
        fields[key] = v
    if a.cmd == "text":
        if bool(a.text) == bool(a.text_file):
            sys.exit("give one of --text or --text-file")
        prompt = a.text if a.text else pathlib.Path(a.text_file).read_text(encoding="utf-8").strip()
        req = build_request(prompt, a.id or a.name, extra_refs, a.n, a.aspect, a.route or "other", a.task,
                            a.target, {"prompts_file": G.rel(a.text_file) if a.text_file else None, **fields})
        try:
            enqueue(pipe, request_name(pipe, a.name), req, a.dry_run)
        except (FileNotFoundError, FileExistsError) as e:
            sys.exit(str(e))
        return 0
    kind = "faces" if a.faces else None
    src, styled = G.PROMPTS, None
    if a.file == "icons":
        src = G.ICONS
    if a.prompts:
        src, styled = pathlib.Path(a.prompts).resolve(), True
    if kind and styled is None and src == G.PROMPTS:
        src, styled = G.FACES, True
    if not src.is_file():
        sys.exit(f"no prompts file {G.rel(src)}")
    route = a.route or ("faces" if kind else "icon_sheet" if src == G.ICONS else "portrait")
    P, R = G.prompts(src, styled), G.references(src)
    for cid in a.ids:
        if cid not in P:
            print(f"{cid}: no prompt in {src.name}"); continue
        ref = R.get(cid)
        if kind and not a.no_ref and ref is None:
            print(f"{cid}: no reference: line in {src.name}"); continue
        refs = ([] if a.no_ref else [ref or G.REFERENCE]) + extra_refs
        record = {"prompts_file": G.rel(src), **fields}
        if kind:
            record["kind"] = kind
        req = build_request(P[cid], cid, refs, a.n, a.aspect, route, a.task, a.target, record)
        try:
            enqueue(pipe, request_name(pipe, cid, kind), req, a.dry_run)
        except (FileNotFoundError, FileExistsError) as e:
            print(f"{cid}: {e}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
