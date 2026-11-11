# Tests of face.py (PORTRAITFX1-T, checks T1-T3 of docs/specs/PORTRAITFX1_TASKS.md 1.3). Run from anywhere:
#   python tools/portraits/test_face.py
# The engine side of T1 and T2 is aegis_core/tests/face_fixtures.rs: it asserts the same fixture sidecar parses and
# the same expected_check.txt against parse_face and face_problems.
import contextlib
import io
import json
import pathlib
import shutil
import sys
import tempfile
import unittest

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import face  # noqa: E402

FIX = HERE / "fixtures" / "face"
GOOD = FIX / "portraits" / "heroes" / "good.png"
GOOD_ARGS = ["--fresh", "--eyes", "18,20,5,3", "27,20,5,3", "--glow", "24,38,6", "--lines", "30,30,45,10",
             "--idle", "point_glow:#c084fc:0.8", "glow::0.5"]


def img(path):
    """An image read whole and its file closed."""
    with Image.open(path) as im:
        im.load()
        return im.copy()


def run(*argv):
    """face.main(argv) with its output caught: (exit code, stdout lines, stderr lines)."""
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = face.main(list(map(str, argv)))
    return code, out.getvalue().splitlines(), err.getvalue().splitlines()


class Face(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="face_test_"))

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def copy(self, png=GOOD):
        dst = self.tmp / png.name
        shutil.copy(png, dst)
        return dst

    def test_t1_set_writes_the_committed_sidecar(self):
        png = self.copy()
        code, _, _ = run("set", png, *GOOD_ARGS)
        self.assertEqual(code, 0)
        written = face.face_file(png).read_text(encoding="utf-8")
        self.assertEqual(written, face.face_file(GOOD).read_text(encoding="utf-8"),
                         "the fixture is what set writes (the engine test parses it)")
        parsed, bad = face.parse_face(json.loads(written))
        self.assertEqual(bad, [])
        self.assertEqual(parsed["size"], img(png).size[0], "size from the PNG's width")
        self.assertEqual(face.face_problems(parsed, parsed["size"]), [])

    def test_t1_set_keeps_keys_it_is_not_given(self):
        png = self.copy()
        run("set", png, *GOOD_ARGS)
        code, _, _ = run("set", png, "--no-blink", "--rim", "#d97706")
        self.assertEqual(code, 0)
        side = json.loads(face.face_file(png).read_text(encoding="utf-8"))
        self.assertEqual((side["blink"], side["rim"], side["eyes"]), (False, "#d97706", [[18, 20, 5, 3], [27, 20, 5, 3]]))

    def test_t1_a_box_outside_the_inner_rect_is_refused_and_nothing_written(self):
        png = self.copy()
        code, _, err = run("set", png, "--fresh", "--eyes", "0,20,5,3")
        self.assertEqual(code, 1)
        w = img(png).size[0]
        b = face.default_border(w)
        self.assertIn(f"{png.stem}.face.json: eye box 1 [0, 20, 5, 3] must lie inside the inner rect ({b}-{w - b}).", err)
        self.assertFalse(face.face_file(png).exists())
        # A refused change leaves an existing sidecar as it was.
        run("set", png, *GOOD_ARGS)
        before = face.face_file(png).read_text(encoding="utf-8")
        code, _, _ = run("set", png, "--glow", "24,38,2")
        self.assertEqual(code, 1)
        self.assertEqual(face.face_file(png).read_text(encoding="utf-8"), before)

    def test_ex_set_writes_the_face_box_only_when_given(self):
        # PORTRAITFX1-EX: `--face` stores the box (the engine's rules: inside the inner rect, at least size / 8 a side);
        # without it the key stays out, so the sidecars written before keep their bytes.
        png = self.copy()
        run("set", png, *GOOD_ARGS)
        self.assertNotIn('"face"', face.face_file(png).read_text(encoding="utf-8"))
        w = img(png).size[0]
        b = face.default_border(w)
        side = face.tdiv(w, 8)
        code, _, _ = run("set", png, "--face", f"{b},{b},{side},{side}")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(face.face_file(png).read_text(encoding="utf-8"))["face"], [b, b, side, side])
        code, _, err = run("set", png, "--face", f"{b},{b},{side - 1},{side}")
        self.assertEqual(code, 1)
        self.assertIn(f"{png.stem}.face.json: face box width and height must be at least {side} (is {side - 1} x {side}).", err)
        code, _, _ = run("set", png, "--face", "")
        self.assertEqual(code, 0)
        self.assertNotIn('"face"', face.face_file(png).read_text(encoding="utf-8"), "\"\" clears the box")

    def test_t2_check_prints_the_committed_a11_lines(self):
        want = [l for l in (FIX / "expected_check.txt").read_text(encoding="utf-8").splitlines() if l]
        code, out, err = run("check", FIX)
        self.assertEqual(out, want)
        self.assertEqual(code, 1)
        sidecars = len(list((FIX / "portraits").rglob("*.face.json")))
        self.assertEqual(err, [f"{sidecars} sidecars, {len(want)} problems."])

    def test_t2_a_clean_layer_checks_clean(self):
        layer = self.tmp / "layer"
        (layer / "portraits" / "heroes").mkdir(parents=True)
        shutil.copy(GOOD, layer / "portraits" / "heroes" / GOOD.name)
        shutil.copy(face.face_file(GOOD), layer / "portraits" / "heroes" / face.face_file(GOOD).name)
        self.assertEqual(run("check", layer)[:2], (0, []))

    def assert_scaled(self, out, png):
        w, h = img(png).size
        self.assertEqual(img(out).size, (w * face.SCALE, h * face.SCALE))

    def test_t3_grid_is_4x_with_a_line_every_8_px(self):
        out = self.tmp / "grid.png"
        self.assertEqual(run("grid", GOOD, out)[0], 0)
        self.assert_scaled(out, GOOD)
        src = img(GOOD).convert("RGBA")
        im = img(out).convert("RGBA")
        k = face.SCALE
        # A grid line (x 8) is brightened over the source pixel; the device px beside it is not.
        y = 30 * k + 1
        self.assertGreater(sum(im.getpixel((8 * k, y))[:3]), sum(src.getpixel((8, 30))[:3]))
        self.assertEqual(im.getpixel((8 * k + 1, y)), src.getpixel((8, 30)))

    def test_t3_show_draws_the_anchors_in_their_image_px(self):
        out = self.tmp / "show.png"
        self.assertEqual(run("show", GOOD, out)[0], 0)
        self.assert_scaled(out, GOOD)
        side, _ = face.load(GOOD)
        im = img(out).convert("RGBA")
        k = face.SCALE
        for x, y, w, h in side["eyes"]:
            self.assertEqual(im.getpixel((x * k, y * k)), face.EYE, "the box's top-left device px")
            self.assertEqual(im.getpixel(((x + w) * k - 1, (y + h) * k - 1)), face.EYE, "its bottom-right")
        for x, y, r in side["glow"]:
            self.assertEqual(im.getpixel((x * k, y * k)), face.GLOW, "the glow centre")
        for x, y, deg, ln in side["lines"]:
            self.assertEqual(im.getpixel((x * k, y * k)), face.LINE, "the line root")

    def test_t3_contact_tiles_each_portrait_at_4x(self):
        out = self.tmp / "contact.png"
        plain = FIX / "portraits" / "common" / "plain.png"
        self.assertEqual(run("contact", out, GOOD, plain, "--cols", 2)[0], 0)
        w, h = img(GOOD).size
        k = face.SCALE
        sheet = img(out).convert("RGBA")
        self.assertEqual(sheet.size[0], 2 * w * k)
        self.assertGreater(sheet.size[1], h * k)
        x, y, _, _ = face.load(GOOD)[0]["eyes"][0]
        self.assertEqual(sheet.getpixel((x * k, y * k)), face.EYE, "the first tile carries its anchors")
        src = img(plain).convert("RGBA")
        self.assertEqual(sheet.getpixel((w * k + 20 * k, 20 * k)), src.getpixel((20, 20)), "no sidecar: the bare portrait")

    def test_fb206_old_names_read_as_the_new_ones_and_set_writes_the_new_ones(self):
        old = {"size": 48, "eyes": [[18, 20, 5, 3]], "veins": [[30, 30, 45, 10]],
               "idle": [{"effect": "veins"}, {"effect": "eye_glow"}, {"effect": "ash"}, {"effect": "smoke"}]}
        new = {"size": 48, "eyes": [[18, 20, 5, 3]], "lines": [[30, 30, 45, 10]],
               "idle": [{"effect": "lines"}, {"effect": "point_glow"}, {"effect": "particles_rise"}, {"effect": "drift"}]}
        self.assertEqual(face.parse_face(old), face.parse_face(new))
        self.assertEqual(face.parse_face(dict(new, veins=[]))[1],
                         ["the face sidecar holds both 'lines' and its old name 'veins' (keep 'lines')."])
        png = self.copy()
        face.face_file(png).write_text(json.dumps(old), encoding="utf-8")
        code, _, _ = run("set", png, "--veins", "28,28,90,8")
        self.assertEqual(code, 0)
        side = json.loads(face.face_file(png).read_text(encoding="utf-8"))
        self.assertNotIn("veins", side)
        self.assertEqual(side["lines"], [[28, 28, 90, 8]], "--veins is the old spelling of --lines")
        self.assertEqual([o["effect"] for o in side["idle"]], ["lines", "point_glow", "particles_rise", "drift"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
