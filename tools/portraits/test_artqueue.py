# Tests of artqueue.py (ARTQUEUE1, decision "Antigravity image queue"): the request shape (exact prompt, references
# in order, n, aspect, metadata), the dry run, no API call and no key, the house naming with n, the ingest of good and
# bad images (missing, corrupt, wrong aspect: moved aside with a note), the prompt record fields, the model mark
# cleared, the portrait route pixelized, and an idempotent ingest. Antigravity is faked: the test moves the request to
# completed/ and writes the output images and result itself. Every queue is a temp folder (never the main checkout's
# .image_pipeline/); art_src is a temp folder too. Synthetic prompts files only.
# Run (from tools/portraits): <Python312>\python.exe -m unittest test_artqueue
import io
import json
import os
import pathlib
import shutil
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import artqueue as Q  # noqa: E402
import generate as G  # noqa: E402

STYLE = "A 2x2 grid of four portrait variations. Plain flat very dark background."
BG = (18, 14, 18)


def faces_md(ref):
    return ("# Faces\n\n```text\n" + STYLE + "\n```\n\n"
            "## Tess (`tess`)\n\nreference: `" + ref.as_posix() + "`\n\n```text\nTess, a burned ear.\n```\n\n"
            "## Bram (`bram`)\n\n```text\nBram, no reference line.\n```\n")


def portrait_png(path, size=(1024, 1024), mark=True):
    """A dark background, a coloured figure in the middle and (mark) a pale star where the model puts its mark."""
    im = Image.new("RGB", size, BG)
    w, h = size
    for y in range(h // 4, h * 3 // 4):
        for x in range(w // 3, w * 2 // 3):
            im.putpixel((x, y), (150 + (x % 3) * 20, 60, 40) if y < h // 2 else (60, 80, 140))
    if mark:
        cx, cy = int(w * 0.946), int(h * 0.946)
        for d in range(-2, 3):
            im.putpixel((cx + d, cy), (230, 230, 230)); im.putpixel((cx, cy + d), (230, 230, 230))
    im.save(path)
    return int(w * 0.946), int(h * 0.946)


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="artqueue_test_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.pipe = self.tmp / "pipe"
        self.src = self.tmp / "art_src"; self.src.mkdir()
        self.ref = self.tmp / "tess_src.png"; portrait_png(self.ref, mark=False)
        self.extra_ref = self.tmp / "style.png"; portrait_png(self.extra_ref, mark=False)
        self.md = self.tmp / "faces.md"; self.md.write_text(faces_md(self.ref), encoding="utf-8")
        old_src, old_call = G.SRC, G.call
        self.addCleanup(setattr, G, "SRC", old_src)
        self.addCleanup(setattr, G, "call", old_call)
        G.SRC = self.src

        def no_api(path, body=None):
            raise AssertionError("the queue backend called the image API")
        G.call = no_api
        self.key = os.environ.pop("GEMINI_API_KEY", None)
        self.addCleanup(lambda: self.key is not None and os.environ.__setitem__("GEMINI_API_KEY", self.key))

    def run_cli(self, *args):
        buf = io.StringIO()
        with redirect_stdout(buf):
            Q.main(["--pipeline", str(self.pipe), *args])
        return buf.getvalue()

    def queued(self):
        return sorted((self.pipe / "queue").glob("*.json"))

    def fake_antigravity(self, req_path, images, model="image-generator", prompt_sent=None):
        """Does Antigravity's part: moves the request to completed/, writes `images` {file: writer} and the result."""
        name = req_path.stem; req = json.loads(req_path.read_text(encoding="utf-8"))
        (self.pipe / "completed" / req_path.name).write_bytes(req_path.read_bytes()); req_path.unlink()
        files = []
        for fname, write in images.items():
            if write:
                write(self.pipe / "output" / fname)
            files.append({"file": f"output\\{fname}", "width": 1024, "height": 1024})
        result = {"model": model, "time": "2026-11-02T10:00:00+00:00",
                  "prompt_sent": req["prompt"] if prompt_sent is None else prompt_sent,
                  "references": req["references"], "n": req["n"], "files": files, "notes": ""}
        (self.pipe / "output" / f"{name}.result.json").write_text(json.dumps(result), encoding="utf-8")
        return name, req


class Requests(Base):
    def test_request_shape_faces(self):
        self.run_cli("request", "tess", "--faces", "--prompts", str(self.md), "--n", "2", "--ref",
                     str(self.extra_ref), "--task", "PORTRAITFX1-XA", "--record", "faction=castellan")
        [path] = self.queued()
        self.assertRegex(path.name, r"^tess_faces_q\d+\.json$")
        req = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(req["prompt"], G.prompts(self.md, True)["tess"])
        self.assertNotIn("reference:", req["prompt"])
        refs = (self.pipe / "refs").resolve()
        self.assertEqual(req["references"], [(refs / self.ref.name).as_posix(), (refs / self.extra_ref.name).as_posix()])
        self.assertTrue(all(pathlib.Path(r).is_absolute() for r in req["references"]))
        self.assertEqual((req["n"], req["aspect_ratio"], req["style_hints"]), (2, "1:1", ""))
        meta = req["metadata"]
        self.assertEqual((meta["asset_id"], meta["route"], meta["task"]), ("tess", "faces", "PORTRAITFX1-XA"))
        self.assertEqual(meta["record"]["kind"], "faces")
        self.assertEqual(meta["record"]["faction"], "castellan")
        self.assertEqual(sorted(p.name for p in self.pipe.iterdir()), sorted([*Q.FOLDERS, Q.REFS]))

    def test_faces_without_reference_line_is_skipped(self):
        out = self.run_cli("request", "bram", "--faces", "--prompts", str(self.md))
        self.assertIn("no reference: line", out)
        self.assertFalse(self.pipe.exists())

    def test_text_request(self):
        tf = self.tmp / "scene.txt"; tf.write_text("A ruined courtyard at night.\n", encoding="utf-8")
        self.run_cli("text", "courtyard_night", "--text-file", str(tf), "--aspect", "16:9", "--route",
                     "story_scene", "--ref", str(self.extra_ref))
        [path] = self.queued()
        self.assertRegex(path.name, r"^courtyard_night_q\d+\.json$")
        req = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(req["prompt"], "A ruined courtyard at night.")
        self.assertEqual((req["aspect_ratio"], req["metadata"]["route"], req["metadata"]["asset_id"]),
                         ("16:9", "story_scene", "courtyard_night"))
        self.run_cli("text", "courtyard_night", "--text", "again")   # the same name again: a new unique request
        self.assertEqual(len({p.name for p in self.queued()}), 2)
        self.assertEqual(list((self.pipe / "queue").glob("*.tmp")), [])   # written through <file>.tmp, renamed

    def test_missing_reference_refused(self):
        with self.assertRaises(SystemExit):
            self.run_cli("text", "lost", "--text", "Lost.", "--ref", str(self.tmp / "nope.png"))
        self.assertEqual(self.queued() if self.pipe.exists() else [], [])
        md = self.tmp / "gone.md"; md.write_text(faces_md(self.tmp / "gone.png"), encoding="utf-8")
        out = self.run_cli("request", "tess", "--faces", "--prompts", str(md))
        self.assertIn("no reference image", out)
        self.assertEqual(self.queued() if self.pipe.exists() else [], [])

    def test_dry_run_writes_nothing(self):
        out = self.run_cli("request", "tess", "--faces", "--prompts", str(self.md), "--dry-run")
        self.assertIn("would queue", out)
        self.assertFalse(self.pipe.exists())

    def test_build_request_rejects_bad_values(self):
        for kw in ({"aspect": "2:1"}, {"route": "poster"}, {"n": 0}):
            with self.assertRaises(ValueError):
                Q.build_request("x", "a", **kw)
        with self.assertRaises(ValueError):
            Q.build_request("  ", "a")

    def clean_env(self):
        for k in ("AEGIS_IMAGE_PIPELINE", "IMAGE_PIPELINE"):
            old = os.environ.pop(k, None)
            self.addCleanup(lambda k=k, old=old: old is not None and os.environ.__setitem__(k, old))

    def test_default_pipeline_from_config_env_and_override(self):
        self.clean_env()
        drive = self.tmp / "drive"; drive.mkdir()
        cfg = self.tmp / "cfg.json"; cfg.write_text(json.dumps({"pipeline": drive.as_posix()}), encoding="utf-8")
        self.assertEqual(Q.pipeline(config=cfg), drive)
        env_dir = self.tmp / "envdir"; env_dir.mkdir()
        os.environ["AEGIS_IMAGE_PIPELINE"] = str(env_dir)
        self.assertEqual(Q.pipeline(config=cfg), env_dir)
        other = self.tmp / "other"
        self.assertEqual(Q.pipeline(str(other), config=cfg), other)       # --pipeline wins, need not exist yet
        backup = Q.pipeline(backup=True)
        self.assertEqual(backup.name, ".image_pipeline")
        self.assertTrue((backup.parent / ".git").is_dir())   # the main checkout, not a worktree (.git is a file there)

    def test_missing_default_folder_fails_clearly(self):
        self.clean_env()
        cfg = self.tmp / "cfg.json"
        cfg.write_text(json.dumps({"pipeline": (self.tmp / "no_drive").as_posix()}), encoding="utf-8")
        with self.assertRaises(SystemExit) as cm:
            Q.pipeline(config=cfg)
        self.assertIn("does not exist", str(cm.exception))
        self.assertIn("no_drive", str(cm.exception))
        with self.assertRaises(SystemExit) as cm:
            Q.pipeline(config=self.tmp / "absent.json")
        self.assertIn("no image pipeline folder", str(cm.exception))

    def test_shipped_config_names_a_pipeline(self):
        self.assertTrue(json.loads(Q.CONFIG.read_text(encoding="utf-8"))["pipeline"])


class Refs(Base):
    def req_of(self, path):
        return json.loads(path.read_text(encoding="utf-8"))

    def test_refs_copied_and_listed(self):
        self.run_cli("text", "a", "--text", "x", "--ref", str(self.ref), "--ref", str(self.extra_ref))
        [path] = self.queued()
        req = self.req_of(path)
        self.assertEqual([pathlib.Path(r).parent for r in req["references"]], [(self.pipe / "refs").resolve()] * 2)
        self.assertEqual([pathlib.Path(r).name for r in req["references"]], [self.ref.name, self.extra_ref.name])
        for r in req["references"]:
            self.assertTrue(pathlib.Path(r).is_file())
        self.assertEqual(req["metadata"]["record"]["source_refs"],
                         [self.ref.resolve().as_posix(), self.extra_ref.resolve().as_posix()])

    def test_same_name_different_file_gets_a_suffix(self):
        other_dir = self.tmp / "o"; other_dir.mkdir()
        clash = other_dir / self.ref.name
        portrait_png(clash, size=(1024, 1024), mark=True)       # same name, different bytes
        self.run_cli("text", "a", "--text", "x", "--ref", str(self.ref))
        self.run_cli("text", "b", "--text", "x", "--ref", str(clash))
        self.run_cli("text", "c", "--text", "x", "--ref", str(self.ref))   # the same file again: reused
        names = [pathlib.Path(self.req_of(p)["references"][0]).name for p in self.queued()]
        self.assertEqual(sorted(names), sorted([self.ref.name, f"{self.ref.stem}_2.png", self.ref.name]))
        self.assertEqual(len(list((self.pipe / "refs").glob("*.png"))), 2)

    def test_dry_run_copies_nothing(self):
        self.run_cli("text", "a", "--text", "x", "--ref", str(self.ref), "--dry-run")
        self.assertFalse((self.pipe / "refs").exists())

    def test_attach_lists_unique_sorted_without_session_refs(self):
        self.run_cli("text", "a", "--text", "x", "--ref", str(self.extra_ref), "--ref", str(self.ref))
        self.run_cli("text", "b", "--text", "x", "--ref", str(self.ref))
        sess = self.tmp / "sess.png"; portrait_png(sess, mark=False)
        self.run_cli("text", "s", "--text", "x", "--ref", str(sess), "--record", "session_ref=a_q1")
        out = self.run_cli("attach").split()
        self.assertEqual(out, sorted([self.ref.name, self.extra_ref.name]))
        self.assertNotIn("sess.png", out)

    def test_attach_nothing(self):
        self.run_cli("text", "a", "--text", "x")
        self.assertIn("nothing to attach", self.run_cli("attach"))


class Ingest(Base):
    def queue_one(self, *args):
        self.run_cli(*args)
        [path] = self.queued()
        return path

    def test_portrait_n1_record_mark_and_pixelize(self):
        path = self.queue_one("text", "bryn_hale", "--text", "Bryn, a scarred sergeant.", "--route", "portrait",
                              "--ref", str(self.extra_ref), "--record", "faction=castellan")
        spot = {}
        name, req = self.fake_antigravity(path, {f"{path.stem}.png": lambda p: spot.update(xy=portrait_png(p))})
        self.run_cli("ingest")
        art = self.src / "bryn_hale.png"
        self.assertTrue(art.is_file())
        with Image.open(art) as im:
            self.assertEqual(im.convert("RGB").getpixel(spot["xy"]), BG)   # the mark cleared
        rec = json.loads(art.with_suffix(".json").read_text(encoding="utf-8"))
        self.assertEqual((rec["model"], rec["via"]), ("image-generator", "antigravity"))
        self.assertEqual(rec["id"], "bryn_hale")
        self.assertEqual(rec["prompt"], "Bryn, a scarred sergeant.")
        self.assertEqual(rec["source"], Q.SOURCE)
        self.assertEqual(rec["request"], name)
        self.assertEqual(rec["n"], 1)
        self.assertEqual(rec["time"], "2026-11-02T10:00:00+00:00")
        self.assertEqual(rec["references"], [G.rel(self.pipe / "refs" / self.extra_ref.name)])
        self.assertEqual(rec["reference"], G.rel(self.pipe / "refs" / self.extra_ref.name))
        self.assertEqual(rec["metadata"], req["metadata"])
        self.assertEqual(rec["faction"], "castellan")
        self.assertTrue(rec["mark_cleared"])
        self.assertNotIn("prompt_requested", rec)
        px = self.pipe / Q.PIXELIZED / "bryn_hale.png"
        with Image.open(px) as im:
            self.assertEqual(im.size, (240, 240))
        marker = json.loads(next((self.pipe / "archive").glob(f"*/output/{name}.ingested.json")).read_text(encoding="utf-8"))
        self.assertEqual([a["art_src"] for a in marker["accepted"]], [G.rel(art)])

    def test_portrait_target_and_existing_source(self):
        portrait_png(self.src / "bryn_hale.png")      # a first take exists: the new one is a _gen take
        target = self.tmp / "game" / "bryn_hale.png"
        path = self.queue_one("text", "bryn_q2", "--text", "Bryn.", "--id", "bryn_hale", "--route", "portrait",
                              "--target", str(target))
        self.fake_antigravity(path, {f"{path.stem}.png": portrait_png})
        self.run_cli("ingest", path.stem)
        self.assertTrue(target.is_file())
        self.assertEqual(len(list(self.src.glob("bryn_hale_gen*_1.png"))), 1)

    def test_faces_naming_with_n(self):
        path = self.queue_one("request", "tess", "--faces", "--prompts", str(self.md), "--n", "2")
        name, _ = self.fake_antigravity(path, {f"{path.stem}_1.png": portrait_png,
                                               f"{path.stem}_2.png": portrait_png})
        buf = io.StringIO()
        with redirect_stdout(buf):
            Q.ingest_one(self.pipe, name, now=1700000000)
        for k in (1, 2):
            art = self.src / f"tess_faces_gen1700000000_{k}.png"
            self.assertTrue(art.is_file(), art.name)
            rec = json.loads(art.with_suffix(".json").read_text(encoding="utf-8"))
            self.assertEqual((rec["kind"], rec["n"]), ("faces", k))
            self.assertEqual(rec["prompts_file"], G.rel(self.md))
            self.assertEqual(rec["reference"], G.rel(self.pipe / "refs" / self.ref.name))
        self.assertFalse((self.pipe / Q.PIXELIZED).exists())   # faces are cut by sheets.py, not pixelized

    def test_bad_images_moved_aside(self):
        path = self.queue_one("text", "icons_q", "--text", "Icons.", "--route", "icon_sheet", "--n", "8")
        s = path.stem
        flat = lambda size: (lambda p: Image.new("RGB", size, BG).save(p))
        name, _ = self.fake_antigravity(path, {
            f"{s}_1.png": portrait_png,
            f"{s}_2.png": None,                                      # missing
            f"{s}_3.png": lambda p: p.write_bytes(b"\x89PNG\r\n\x1a\nnot really"),   # corrupt
            f"{s}_4.png": flat((2048, 1024)),                        # wrong aspect
            f"{s}_5.png": flat((1024, 1000)),                        # 2.4 % off 1:1: accepted
            f"{s}_6.png": flat((512, 512)),                          # a preview, not full size
            f"{s}_7.png": flat((1024, 1010)),                        # 1.4 % off: accepted
            f"{s}_8.png": flat((1024, 990))})                        # 3.4 % off 1:1: refused
        buf = io.StringIO()
        with redirect_stdout(buf):
            marker = Q.ingest_one(self.pipe, name, now=1700000000)
        self.assertEqual([a["file"] for a in marker["accepted"]], [f"{s}_1.png", f"{s}_5.png", f"{s}_7.png"])
        reasons = {r["file"]: r["reason"] for r in marker["rejected"]}
        self.assertEqual(reasons[f"{s}_2.png"], "missing")
        self.assertTrue(reasons[f"{s}_3.png"].startswith("corrupt"))
        self.assertTrue(reasons[f"{s}_4.png"].startswith("wrong aspect 2048x1024"))
        self.assertTrue(reasons[f"{s}_8.png"].startswith("wrong aspect 1024x990"))
        self.assertTrue(reasons[f"{s}_6.png"].startswith("too small 512x512"))
        rej = self.pipe / Q.REJECTED
        for k in (3, 4, 6, 8):
            f = f"{s}_{k}.png"
            self.assertTrue((rej / f).is_file()); self.assertTrue((rej / f"{f}.note.txt").is_file())
            self.assertFalse((self.pipe / "output" / f).exists())
        self.assertEqual(sorted(p.name for p in self.src.glob("*.png")),
                         ["icons_q_gen1700000000_1.png", "icons_q_gen1700000000_5.png",
                          "icons_q_gen1700000000_7.png"])

    def test_gemini_sizes_pass_the_aspect_check(self):
        for aspect, size in (("4:3", (1184, 864)), ("3:4", (864, 1184)), ("16:9", (1344, 768)),
                             ("9:16", (768, 1344)), ("1:1", (1024, 1024))):
            path = self.tmp / f"g_{size[0]}x{size[1]}.png"; Image.new("RGB", size, BG).save(path)
            self.assertIsNone(Q.check_image(path, aspect), (aspect, size))
        self.assertTrue(Q.check_image(path, "4:3").startswith("wrong aspect"))
        self.assertFalse((self.pipe / Q.PIXELIZED).exists())   # only route portrait is pixelized

    def test_idempotent_and_unfinished(self):
        path = self.queue_one("text", "one_q", "--text", "One.", "--route", "other")
        name, _ = self.fake_antigravity(path, {f"{path.stem}.png": portrait_png})
        self.run_cli("ingest")
        before = sorted(p.name for p in self.src.iterdir())
        out = self.run_cli("ingest", name)
        self.assertIn("ingested already", out)
        self.run_cli("ingest")
        self.assertEqual(sorted(p.name for p in self.src.iterdir()), before)
        # a result whose request is still in processing/ is left alone
        self.run_cli("text", "two_q", "--text", "Two.")
        [req] = self.queued(); two = req.stem; req.rename(self.pipe / "processing" / req.name)
        (self.pipe / "output" / f"{two}.result.json").write_text("{}", encoding="utf-8")
        self.assertIn("not completed", self.run_cli("ingest", two))
        s = Q.status(self.pipe)
        self.assertEqual((s["queue"], s["processing"], s["output"], s["ingested"], s["completed"]), (0, 1, 1, 0, 0))
        self.assertEqual(s["archived"], 1)

    def test_prompt_changed_and_dry_run(self):
        path = self.queue_one("text", "chg_q", "--text", "Asked prompt.")
        name, _ = self.fake_antigravity(path, {f"{path.stem}.png": portrait_png}, prompt_sent="Sent prompt.")
        self.run_cli("ingest", "--dry-run")
        self.assertEqual(list(self.src.iterdir()), [])
        self.assertFalse((self.pipe / "output" / f"{name}.ingested.json").exists())
        self.run_cli("ingest")
        rec = json.loads((self.src / "chg_q.json").read_text(encoding="utf-8"))
        self.assertEqual((rec["prompt"], rec["prompt_requested"]), ("Sent prompt.", "Asked prompt."))

    def test_failed_request_reported(self):
        path = self.queue_one("text", "bad_q", "--text", "Bad.")
        (self.pipe / "failed" / path.name).write_bytes(path.read_bytes()); path.unlink()
        (self.pipe / "failed" / f"{path.stem}.error.txt").write_text("reference refused by the model\n",
                                                                     encoding="utf-8")
        out = self.run_cli("ingest")
        self.assertIn(f"{path.stem}: failed in Antigravity: reference refused by the model", out)
        self.assertIn("failed in Antigravity", self.run_cli("ingest", path.stem))
        self.assertEqual(list(self.src.iterdir()), [])
        self.assertEqual(Q.status(self.pipe)["failed"], 1)



class Archive(Base):
    def ingested(self, base="arc_q"):
        self.run_cli("text", base, "--text", "Arc.", "--route", "other")
        [path] = self.queued()
        name, _ = self.fake_antigravity(path, {f"{path.stem}.png": portrait_png})
        return name

    def day(self):
        import datetime
        return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")

    def test_ingest_archives_request(self):
        name = self.ingested()
        self.run_cli("ingest")
        arc = self.pipe / "archive" / self.day()
        self.assertEqual(sorted(p.name for p in (arc / "output").iterdir()),
                         sorted([f"{name}.png", f"{name}.result.json", f"{name}.ingested.json"]))
        self.assertEqual([p.name for p in (arc / "completed").iterdir()], [f"{name}.json"])
        for f in ("output", "completed"):
            self.assertEqual(list((self.pipe / f).iterdir()), [])
        self.assertEqual(Q.status(self.pipe)["archived"], 1)

    def test_reingest_says_ingested_already(self):
        name = self.ingested()
        self.run_cli("ingest")
        self.assertIn("ingested already", self.run_cli("ingest", name))
        self.assertEqual(Q.finished(self.pipe), [])

    def test_sweep_old_marker_and_dry_run(self):
        name = self.ingested()
        self.run_cli("ingest")
        # undo the archive move to look like an earlier ingest
        arc = self.pipe / "archive" / self.day()
        for f in (arc / "output").iterdir():
            shutil.move(str(f), str(self.pipe / "output" / f.name))
        shutil.move(str(arc / "completed" / f"{name}.json"), str(self.pipe / "completed"))
        shutil.rmtree(self.pipe / "archive")
        out = self.run_cli("archive", "--dry-run")
        self.assertIn("would archive 4", out)
        self.assertEqual(len(list((self.pipe / "output").iterdir())), 3)
        self.assertFalse((self.pipe / "archive").exists())
        self.run_cli("archive")
        self.assertEqual(list((self.pipe / "output").iterdir()), [])
        self.assertEqual(list((self.pipe / "completed").iterdir()), [])
        self.assertTrue((arc / "completed" / f"{name}.json").exists())
        self.assertEqual(Q.status(self.pipe)["archived"], 1)

    def test_ingest_dry_run_moves_nothing(self):
        name = self.ingested()
        self.run_cli("ingest", "--dry-run")
        self.assertFalse((self.pipe / "archive").exists())
        self.assertTrue((self.pipe / "completed" / f"{name}.json").exists())

    def test_name_clash_gets_suffix(self):
        d = self.pipe / "archive" / "2026-01-01" / "output"; d.mkdir(parents=True)
        (d / "a.result.json").write_text("old", encoding="utf-8")
        src = self.tmp / "a.result.json"; src.write_text("new", encoding="utf-8")
        dest = Q.move_unique(src, d)
        self.assertEqual(dest.name, "a_2.result.json")
        self.assertEqual((d / "a.result.json").read_text(encoding="utf-8"), "old")
        self.assertEqual(dest.read_text(encoding="utf-8"), "new")


class Pause(Base):
    """ARTQUEUE2: the quota pause Antigravity writes as paused.json."""
    def pause(self, text):
        self.pipe.mkdir(parents=True, exist_ok=True)
        (self.pipe / "paused.json").write_text(text, encoding="utf-8")

    def at(self, minutes):
        t = Q.datetime.datetime.now(Q.datetime.timezone.utc) + Q.datetime.timedelta(minutes=minutes)
        return t.strftime("%Y-%m-%dT%H:%M:%SZ")

    def test_absent(self):
        self.assertIsNone(Q.pause_state(self.pipe))
        self.assertNotIn("pause", self.run_cli("status"))

    def test_active(self):
        self.pause(json.dumps({"paused_until": self.at(30), "reason": "Error 429", "request": "x_q1"}))
        out = self.run_cli("status")
        self.assertRegex(out, r"paused until \S+ \S+ UTC \((30|31) min\): Error 429")
        out = self.run_cli("text", "p_q", "--text", "P.")
        self.assertIn("requests wait in queue/", out)
        self.assertEqual(len(self.queued()), 1)

    def test_expired(self):
        self.pause(json.dumps({"paused_until": self.at(-5), "reason": "Error 429"}))
        self.assertIn("pause expired", self.run_cli("status"))
        self.assertNotIn("note:", self.run_cli("text", "e_q", "--text", "E."))

    def test_malformed(self):
        for bad in ("{not json", json.dumps({"reason": "no time"}), json.dumps({"paused_until": "soon"})):
            self.pause(bad)
            self.assertIn("paused.json unreadable", self.run_cli("status"))


if __name__ == "__main__":
    unittest.main()
