# Tests of `sheets.py faces` (PORTRAITFX1-XT, spec PORTRAITFX1.md 1.7 XT2): four equal quadrants in the order neutral,
# hurt, angry, afraid; one crop for all four; four 240 px frames under <root>/<folder>/faces/<id>/; --base writes the
# neutral frame as <id>.png; the image model's mark is cleared on the whole sheet (a sheet with the mark gives the same
# frames as one without); --contact shows the portrait as it was beside the four frames; tiers and factions checked
# before anything is written. Synthetic sheets in a temp portraits root only, no shipped art.
# Run (from tools/portraits): <Python312>\python.exe -m unittest test_sheets_faces
import colorsys
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

from PIL import Image, ImageChops, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import pixelize as X  # noqa: E402
import sheets  # noqa: E402

BG = (18, 14, 18)
HUES = {"neutral": (200, 40, 40), "hurt": (40, 180, 40), "angry": (40, 60, 210), "afraid": (210, 190, 40)}
SIZE = 400   # each quadrant 200 x 200


def sheet(mark=True):
    """A 2x2 sheet on the flat dark background: one head per quadrant in its own hue, and the image model's pale star
    at the whole sheet's bottom-right (where clear_mark looks)."""
    im = Image.new("RGB", (SIZE, SIZE), BG)
    d = ImageDraw.Draw(im)
    h = SIZE // 2
    for (qx, qy), name in zip(((0, 0), (h, 0), (0, h), (h, h)), sheets.FACE_CELLS):
        c = HUES[name]
        for k in range(60):   # shaded, so the families get ramps
            t = k / 59
            d.ellipse([qx + 50 + k // 4, qy + 30 + k // 4, qx + 150 - k // 4, qy + 170 - k // 4],
                      fill=tuple(int(v * (0.55 + 0.45 * t)) for v in c))
    if mark:
        cx = cy = int(SIZE * 0.946)
        d.polygon([(cx, cy - 10), (cx + 3, cy - 3), (cx + 10, cy), (cx + 3, cy + 3), (cx, cy + 10), (cx - 3, cy + 3),
                   (cx - 10, cy), (cx - 3, cy - 3)], fill=(235, 235, 235))
    return im


def hue_of(im):
    """The mean hue (0..1) of the frame's saturated pixels."""
    rgb = im.convert("RGB"); pa = rgb.load()
    px = [pa[x, y] for y in range(rgb.height) for x in range(rgb.width)]
    hs = [colorsys.rgb_to_hls(*(v / 255 for v in p)) for p in px]
    sat = [h for h, l, s in hs if s > 0.5 and 0.2 < l < 0.8]
    return sorted(sat)[len(sat) // 2] if sat else None


class Faces(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="faces_test_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.root = self.tmp / "portraits"
        self.src = self.tmp / "tess_faces_gen1_1.png"
        sheet().save(self.src)

    def run_faces(self, **kw):
        kw.setdefault("faction", "hearth")
        return sheets.slice_faces(self.src, "tess", kw.pop("tier", "npc"), root=self.root, **kw)

    def test_four_240_frames_in_cell_order(self):
        written = self.run_faces()
        out = self.root / "npcs" / "faces" / "tess"
        self.assertEqual(written, [out / f"{n}.png" for n in sheets.FACE_CELLS])
        for name in sheets.FACE_CELLS:
            im = Image.open(out / f"{name}.png")
            self.assertEqual(im.size, (240, 240), name)
            want = colorsys.rgb_to_hls(*(v / 255 for v in HUES[name]))[0]
            got = hue_of(im)
            self.assertIsNotNone(got, name)
            self.assertLess(min(abs(got - want), 1 - abs(got - want)), 0.05, f"{name}: its own quadrant's head")
        self.assertFalse((self.root / "npcs" / "tess.png").exists(), "no base without --base")

    def test_base_writes_the_neutral_frame_as_the_portrait(self):
        written = self.run_faces(tier="heroes", base=True)
        base = self.root / "heroes" / "tess.png"
        self.assertEqual(written[-1], base)
        neutral = Image.open(self.root / "heroes" / "faces" / "tess" / "neutral.png")
        self.assertIsNone(ImageChops.difference(Image.open(base).convert("RGBA"), neutral.convert("RGBA")).getbbox())

    def test_frames_equal_pixelize_of_each_quadrant_with_the_one_crop(self):
        crop = (20, 10, 180, 190)
        self.run_faces(crop=crop)
        clean = self.tmp / "clean.png"
        sheet(mark=False).save(clean)
        for name, box in zip(sheets.FACE_CELLS, sheets.face_boxes((SIZE, SIZE), crop)):
            want, _ = X.pixelize(clean, crop=box, mark=False, faction="hearth")
            got = Image.open(self.root / "npcs" / "faces" / "tess" / f"{name}.png").convert("RGBA")
            self.assertIsNone(ImageChops.difference(got, want.convert("RGBA")).getbbox(),
                              f"{name}: pixelize of its quadrant with the crop, the mark cleared first")

    def test_mark_cleared_on_the_whole_sheet(self):
        marked, clean = sheet(), sheet(mark=False)
        self.assertIsNotNone(ImageChops.difference(marked, clean).getbbox(), "the fixture has a mark")
        self.assertIsNone(ImageChops.difference(sheets.clear_sheet(marked), clean.convert("RGBA")).getbbox(),
                          "the mark becomes background, nothing else changes")

    def test_face_boxes(self):
        self.assertEqual(sheets.face_boxes((401, 300)), [(0, 0, 200, 150), (200, 0, 400, 150), (0, 150, 200, 300),
                                                         (200, 150, 400, 300)], "equal halves, the odd column dropped")
        self.assertEqual(sheets.face_boxes((400, 400), (10, 20, 190, 180))[3], (210, 220, 390, 380))
        for bad in ((0, 0, 201, 200), (50, 0, 50, 200), (-1, 0, 100, 100)):
            with self.assertRaises(ValueError):
                sheets.face_boxes((400, 400), bad)
        self.assertEqual(sheets.parse_crop("1,2,3,4"), (1, 2, 3, 4))
        with self.assertRaises(ValueError):
            sheets.parse_crop("1,2,3")

    def test_tiers_and_factions_refused_before_writing(self):
        for tier in ("common", "monster"):
            with self.assertRaises(ValueError):
                self.run_faces(tier=tier)
        with self.assertRaises(ValueError):
            self.run_faces(faction="nobody")
        self.assertFalse(self.root.exists(), "nothing written")
        self.assertEqual({sheets.face_folder(t) for t in ("boss", "bosses")}, {"bosses"})

    def test_faction_from_the_sheet_record(self):
        self.src.with_suffix(".json").write_text(json.dumps({"kind": "faces", "faction": "clerisy"}), encoding="utf-8")
        self.assertEqual(sheets.face_faction(self.src, "no_such_id_xt"), "clerisy")
        sheets.slice_faces(self.src, "tess", "npc", root=self.root)
        want, _ = X.pixelize(self.src, crop=sheets.face_boxes((SIZE, SIZE))[0], mark=False, faction="clerisy")
        got = Image.open(self.root / "npcs" / "faces" / "tess" / "neutral.png").convert("RGBA")
        self.assertIsNone(ImageChops.difference(got, want.convert("RGBA")).getbbox(), "the clerisy backdrop")

    def test_contact_shows_the_portrait_as_it_was(self):
        folder = self.root / "npcs"; folder.mkdir(parents=True)
        old = Image.new("RGBA", (240, 240), (250, 0, 250, 255)); old.save(folder / "tess.png")
        out = self.tmp / "contact.png"
        written = self.run_faces(base=True, contact_out=out)
        self.assertEqual(written[-1], out)
        im = Image.open(out).convert("RGB")
        self.assertEqual(im.size, (8 + 5 * 248, 8 + 240 + 14 + 8))
        self.assertEqual(im.getpixel((8 + 120, 8 + 120)), (250, 0, 250), "the old portrait, not the new base")
        frame = Image.open(folder / "faces" / "tess" / "afraid.png").convert("RGB")
        self.assertEqual(im.getpixel((8 + 4 * 248 + 120, 8 + 120)), frame.getpixel((120, 120)), "afraid last")

    def test_cli(self):
        r = subprocess.run([sys.executable, str(HERE / "sheets.py"), "faces", str(self.src), "tess", "boss", "--base",
                            "--crop", "0,0,200,200", "--faction", "guild", "--root", str(self.root)],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(r.stdout.strip().splitlines()), 5, r.stdout)
        self.assertTrue((self.root / "bosses" / "tess.png").is_file())
        r = subprocess.run([sys.executable, str(HERE / "sheets.py"), "faces", str(self.src), "tess", "common",
                            "--root", str(self.root)], capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("unknown tier", r.stderr)


if __name__ == "__main__":
    unittest.main()
