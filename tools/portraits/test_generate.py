# Tests of generate.py (PORTRAITFX1-XT, spec PORTRAITFX1.md 1.7 XT1): a prompts block's `reference:` line (outside or
# inside the text block, never sent as prompt text), that image sent instead of Mara Voss, the faces sheet name
# art_src/<id>_faces_gen<time>_<n>.png and its record ("kind": "faces", the reference, the full prompt), and the dry
# run (no API call, no key). The API is never called: `call` is replaced by a fake that returns a tiny PNG, and SRC
# points at a temp folder. Synthetic prompts files only.
# Run (from tools/portraits): <Python312>\python.exe -m unittest test_generate
import base64
import io
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import generate  # noqa: E402

STYLE = "A 2x2 grid of four portrait variations. Plain flat very dark background."


def png_bytes(colour):
    buf = io.BytesIO()
    Image.new("RGB", (4, 4), colour).save(buf, "PNG")
    return buf.getvalue()


def faces_md(ref_a, ref_b):
    return (
        "# Faces\n\nThe style block:\n\n```text\n" + STYLE + "\n```\n\n"
        "## Tess (`tess`)\n\nreference: `" + ref_a.as_posix() + "`\n\n```text\nTess, a young adult with a burned ear.\n```\n\n"
        "## Bram (`bram`)\n\n```text\nreference: " + ref_b.as_posix() + "\nBram, a loupe over the right eye.\n```\n\n"
        "## Vane (`captain_vane`)\n\n```text\nCaptain Vane, no reference line.\n```\n")


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="gen_test_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.ref_a = self.tmp / "tess_src.png"; self.ref_a.write_bytes(png_bytes((200, 10, 10)))
        self.ref_b = self.tmp / "bram_src.png"; self.ref_b.write_bytes(png_bytes((10, 200, 10)))
        self.md = self.tmp / "faces.md"; self.md.write_text(faces_md(self.ref_a, self.ref_b), encoding="utf-8")
        self.src = self.tmp / "art_src"; self.src.mkdir()
        old_src, old_call = generate.SRC, generate.call
        self.addCleanup(setattr, generate, "SRC", old_src)
        self.addCleanup(setattr, generate, "call", old_call)
        generate.SRC = self.src
        self.sent = []

        def fake_call(path, body=None):
            self.sent.append((path, body))
            return {"candidates": [{"content": {"parts": [
                {"inlineData": {"mime_type": "image/png", "data": base64.b64encode(png_bytes((1, 2, 3))).decode()}}]}}]}
        generate.call = fake_call


class References(Base):
    def test_reference_line_outside_and_inside_the_block(self):
        refs = generate.references(self.md)
        self.assertEqual(refs, {"tess": self.ref_a, "bram": self.ref_b}, "ids without a line have no entry")

    def test_reference_line_is_not_prompt_text(self):
        P = generate.prompts(self.md, styled=True)
        self.assertEqual(set(P), {"tess", "bram", "captain_vane"})
        for cid, text in P.items():
            self.assertNotIn("reference:", text, cid)
            self.assertTrue(text.startswith(STYLE + "\n\n"), "the style block first")
        self.assertEqual(P["bram"], STYLE + "\n\nBram, a loupe over the right eye.")

    def test_relative_reference_is_from_the_repo_root(self):
        md = self.tmp / "rel.md"
        md.write_text("```text\nstyle\n```\n\n## X (`xid`)\n\nreference: art_src/xid.png\n\n```text\nx\n```\n", encoding="utf-8")
        self.assertEqual(generate.references(md), {"xid": generate.ROOT / "art_src/xid.png"})

    def test_shipped_prompt_files_unchanged_by_the_reference_strip(self):
        # no shipped prompts file has a reference: line, so their prompts are the raw blocks as before
        for path in (generate.PROMPTS, generate.ICONS):
            if path.is_file():
                text = path.read_text(encoding="utf-8")
                self.assertIsNone(generate.REF_LINE.search(text), path.name)


class Faces(Base):
    def test_faces_run_sends_the_block_reference_and_writes_sheet_and_record(self):
        P = generate.prompts(self.md, styled=True); R = generate.references(self.md)
        saved = generate.generate("gemini-x", "tess", P["tess"], 1, True, self.md, R["tess"], "faces")
        self.assertEqual(len(saved), 1)
        self.assertRegex(saved[0].name, r"^tess_faces_gen\d+_1\.png$")
        self.assertEqual(saved[0].parent, self.src)
        path, body = self.sent[0]
        self.assertEqual(path, "models/gemini-x:generateContent")
        parts = body["contents"][0]["parts"]
        self.assertEqual(base64.b64decode(parts[0]["inline_data"]["data"]), self.ref_a.read_bytes(), "the block's reference")
        self.assertEqual(parts[1]["text"], P["tess"], "the full prompt")
        rec = json.loads(saved[0].with_suffix(".json").read_text(encoding="utf-8"))
        self.assertEqual(rec["kind"], "faces")
        self.assertEqual(rec["reference"], generate.rel(self.ref_a))
        self.assertEqual(rec["prompt"], P["tess"])
        self.assertEqual((rec["id"], rec["model"], rec["n"]), ("tess", "gemini-x", 1))
        self.assertEqual(list(rec)[:7], ["model", "id", "prompt", "reference", "prompts_file", "time", "n"],
                         "generate.py's record keys first, in their order")
        self.assertFalse(any("key" in k.lower() for k in rec), "never a key")

    def test_faces_name_even_when_no_source_exists(self):
        # a plain single run would take art_src/<id>.png; a faces sheet never does
        self.assertEqual(generate.out_name("tess", 0, 1, "faces", now=7), "tess_faces_gen7_1.png")
        self.assertEqual(generate.out_name("tess", 0, 1, None, now=7), "tess.png")
        (self.src / "tess.png").write_bytes(b"x")
        self.assertEqual(generate.out_name("tess", 1, 2, None, now=7), "tess_gen7_2.png")

    def test_plain_run_record_has_no_kind_and_mara_voss(self):
        saved = generate.generate("gemini-x", "tess", "p", 1, True, self.md, reference=self.ref_a)
        rec = json.loads(saved[0].with_suffix(".json").read_text(encoding="utf-8"))
        self.assertNotIn("kind", rec)
        rec2 = json.loads(generate.record(self.tmp / "b.png", "m", "x", "p", True, None, 1).read_text(encoding="utf-8"))
        self.assertEqual(rec2["reference"], generate.rel(generate.REFERENCE), "the default reference is Mara Voss")


class DryRun(Base):
    def run_cli(self, *args):
        env = {k: v for k, v in os.environ.items() if k != "GEMINI_API_KEY"}
        return subprocess.run([sys.executable, str(HERE / "generate.py"), "run", "gemini-x", *args],
                              capture_output=True, text=True, env=env, cwd=str(self.tmp))

    def test_dry_run_names_reference_and_output_without_a_key(self):
        r = self.run_cli("tess", "bram", "captain_vane", "--faces", "--prompts", str(self.md), "--dry-run")
        self.assertEqual(r.returncode, 0, r.stderr)
        lines = r.stdout.strip().splitlines()
        self.assertEqual(len(lines), 3, r.stdout)
        self.assertIn(generate.rel(self.ref_a), lines[0])
        self.assertRegex(lines[0], r"-> art_src/tess_faces_gen\d+_1\.png")
        self.assertIn(generate.rel(self.ref_b), lines[1])
        self.assertEqual(lines[2], "captain_vane: no reference: line in faces.md")
        self.assertNotIn("GEMINI_API_KEY", r.stdout + r.stderr)


if __name__ == "__main__":
    unittest.main()
