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
        self.src = self.tmp / "talia_rusk_faces_gen1_1.png"
        sheet().save(self.src)

    def run_faces(self, **kw):
        kw.setdefault("faction", "hearth")
        return sheets.slice_faces(self.src, "talia_rusk", kw.pop("tier", "npc"), root=self.root, **kw)

    def test_four_240_frames_in_cell_order(self):
        written = self.run_faces()
        out = self.root / "npcs" / "faces" / "talia_rusk"
        self.assertEqual(written, [out / f"{n}.png" for n in sheets.FACE_CELLS])
        for name in sheets.FACE_CELLS:
            im = Image.open(out / f"{name}.png")
            self.assertEqual(im.size, (240, 240), name)
            want = colorsys.rgb_to_hls(*(v / 255 for v in HUES[name]))[0]
            got = hue_of(im)
            self.assertIsNotNone(got, name)
            self.assertLess(min(abs(got - want), 1 - abs(got - want)), 0.05, f"{name}: its own quadrant's head")
        self.assertFalse((self.root / "npcs" / "talia_rusk.png").exists(), "no base without --base")

    def test_base_writes_the_neutral_frame_as_the_portrait(self):
        written = self.run_faces(tier="heroes", base=True)
        base = self.root / "heroes" / "talia_rusk.png"
        self.assertEqual(written[-1], base)
        neutral = Image.open(self.root / "heroes" / "faces" / "talia_rusk" / "neutral.png")
        self.assertIsNone(ImageChops.difference(Image.open(base).convert("RGBA"), neutral.convert("RGBA")).getbbox())

    def test_frames_equal_pixelize_of_each_quadrant_with_the_one_crop(self):
        crop = (20, 10, 180, 190)
        self.run_faces(crop=crop)
        clean = self.tmp / "clean.png"
        sheet(mark=False).save(clean)
        for name, box in zip(sheets.FACE_CELLS, sheets.face_boxes((SIZE, SIZE), crop)):
            want, _ = X.pixelize(clean, crop=box, mark=False, faction="hearth")
            got = Image.open(self.root / "npcs" / "faces" / "talia_rusk" / f"{name}.png").convert("RGBA")
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
        sheets.slice_faces(self.src, "talia_rusk", "npc", root=self.root)
        want, _ = X.pixelize(self.src, crop=sheets.face_boxes((SIZE, SIZE))[0], mark=False, faction="clerisy")
        got = Image.open(self.root / "npcs" / "faces" / "talia_rusk" / "neutral.png").convert("RGBA")
        self.assertIsNone(ImageChops.difference(got, want.convert("RGBA")).getbbox(), "the clerisy backdrop")

    def test_contact_shows_the_portrait_as_it_was(self):
        folder = self.root / "npcs"; folder.mkdir(parents=True)
        old = Image.new("RGBA", (240, 240), (250, 0, 250, 255)); old.save(folder / "talia_rusk.png")
        out = self.tmp / "contact.png"
        written = self.run_faces(base=True, contact_out=out)
        self.assertEqual(written[-1], out)
        im = Image.open(out).convert("RGB")
        self.assertEqual(im.size, (8 + 5 * 248, 8 + 240 + 14 + 8))
        self.assertEqual(im.getpixel((8 + 120, 8 + 120)), (250, 0, 250), "the old portrait, not the new base")
        frame = Image.open(folder / "faces" / "talia_rusk" / "afraid.png").convert("RGB")
        self.assertEqual(im.getpixel((8 + 4 * 248 + 120, 8 + 120)), frame.getpixel((120, 120)), "afraid last")

    def test_cli(self):
        r = subprocess.run([sys.executable, str(HERE / "sheets.py"), "faces", str(self.src), "talia_rusk", "boss", "--base",
                            "--crop", "0,0,200,200", "--faction", "guild", "--root", str(self.root)],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(r.stdout.strip().splitlines()), 5, r.stdout)
        self.assertTrue((self.root / "bosses" / "talia_rusk.png").is_file())
        r = subprocess.run([sys.executable, str(HERE / "sheets.py"), "faces", str(self.src), "talia_rusk", "common",
                            "--root", str(self.root)], capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("unknown tier", r.stderr)


def figure(size=200):
    """A lopsided figure (head and a shoulder bump) on the flat background."""
    im = Image.new("RGBA", (size, size), BG + (255,))
    d = ImageDraw.Draw(im)
    d.ellipse([60, 30, 140, 120], fill=(200, 40, 40, 255))
    d.rectangle([40, 110, 150, 190], fill=(200, 40, 40, 255))
    d.ellipse([140, 120, 175, 160], fill=(200, 40, 40, 255))
    return im


class Scale(unittest.TestCase):
    """--reduce scale, --align and --cell (PORTRAITFX1-XA)."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="faces_scale_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.root = self.tmp / "portraits"
        self.src = self.tmp / "talia_rusk_faces_gen1_1.png"
        sheet().save(self.src)

    def run_faces(self, **kw):
        return sheets.slice_faces(self.src, "talia_rusk", "npc", root=self.root, faction="hearth", **kw)

    def frame(self, name):
        return Image.open(self.root / "npcs" / "faces" / "talia_rusk" / f"{name}.png").convert("RGBA")

    def test_no_palette_pass_and_frame_drawn(self):
        self.run_faces(reduce="scale")
        for name in sheets.FACE_CELLS:
            im = self.frame(name)
            self.assertEqual(im.size, (240, 240))
            colours = {c for _, c in im.getcolors(1 << 16)}
            top = HUES[name]
            # the head is shaded from 0.55 to 1.0 of its hue; its brightest ring survives a pure scale
            near = [c for c in colours if all(abs(a - b) <= 24 for a, b in zip(c[:3], top))]
            self.assertTrue(near, f"{name}: an input colour survives")
            self.assertEqual(im.getpixel((0, 0))[:3], (24, 16, 18), "the frame border")
        want = X.finish(Image.new("RGBA", (X.GRID, X.GRID), (0, 0, 0, 0)), faction="hearth")
        self.assertEqual(self.frame("neutral").getpixel((2, 120)), want.convert("RGBA").getpixel((2, 120)), "backdrop")

    def test_scale_frames_equal_the_review_recipe(self):
        self.run_faces(reduce="scale")
        im = sheets.clear_sheet(Image.open(self.src).convert("RGBA"))
        for name, box in zip(sheets.FACE_CELLS, sheets.face_boxes(im.size)):
            fig = X.smooth_reduce(X.cut_background(X.clear_corner(im.crop(box))), X.GRID)
            want = X.finish(fig, faction="hearth").convert("RGBA")
            self.assertIsNone(ImageChops.difference(self.frame(name), want).getbbox(), name)

    def test_default_is_the_pixelize_path(self):
        self.run_faces()
        first = {n: self.frame(n) for n in sheets.FACE_CELLS}
        self.run_faces(reduce="pixelize")
        clean = self.tmp / "clean.png"
        sheet(mark=False).save(clean)
        for name, box in zip(sheets.FACE_CELLS, sheets.face_boxes((SIZE, SIZE))):
            want, _ = X.pixelize(clean, crop=box, mark=False, faction="hearth")
            self.assertIsNone(ImageChops.difference(first[name], want.convert("RGBA")).getbbox(), name)
            self.assertIsNone(ImageChops.difference(self.frame(name), first[name]).getbbox(), name)

    def test_align_recovers_a_known_scale_and_shift(self):
        neutral = figure()
        cell, _ = sheets.place(neutral, 0.9, 9, -9, BG + (255,), Image.LANCZOS)
        aligned, scale, (dx, dy) = sheets.align_to(cell, neutral)
        self.assertLess(abs(scale - 1 / 0.9), 0.01)
        self.assertLessEqual(abs(dx + 10), 1)
        self.assertLessEqual(abs(dy - 10), 1)
        a = X.cut_background(aligned.copy()).getchannel("A")
        n = X.cut_background(neutral.copy()).getchannel("A")
        self.assertLess(sheets.mask_diff(a.point(lambda v: 255 if v >= 128 else 0), n.point(lambda v: 255 if v >= 128 else 0)),
                        0.02 * n.point(lambda v: 255 if v >= 128 else 0).histogram()[255])
        again = sheets.align_to(cell, neutral)
        self.assertEqual(again[1:], (scale, (dx, dy)), "deterministic")

    def test_align_through_the_sheet_reports_each_cell(self):
        lines = []
        self.run_faces(reduce="scale", align=True, report=lines.append)
        self.assertEqual([l.split(":")[0] for l in lines], ["hurt", "angry", "afraid"])
        self.assertTrue(all("scale" in l and "offset" in l for l in lines), lines)

    def test_cell_replaces_the_named_cell(self):
        edit = self.tmp / "afraid_edit.png"
        img = Image.new("RGB", (333, 333), BG)
        ImageDraw.Draw(img).ellipse([100, 60, 230, 270], fill=(250, 250, 250))
        img.save(edit)
        self.run_faces(reduce="scale", cells={"afraid": edit})
        afraid = self.frame("afraid")
        self.assertTrue(any(c[:3] == (250, 250, 250) for _, c in afraid.getcolors(1 << 16)), "the edit's colour")
        plain = self.tmp / "plain"
        sheets.slice_faces(self.src, "talia_rusk", "npc", root=plain, faction="hearth", reduce="scale")
        for name in ("neutral", "hurt", "angry"):
            other = Image.open(plain / "npcs" / "faces" / "talia_rusk" / f"{name}.png").convert("RGBA")
            self.assertIsNone(ImageChops.difference(self.frame(name), other).getbbox(), f"{name} untouched")
        with self.assertRaises(ValueError):
            self.run_faces(reduce="scale", cells={"happy": edit})

    def test_flags_need_scale(self):
        for kw in ({"align": True}, {"cells": {"hurt": self.src}}, {"reduce": "sharp"}):
            with self.assertRaises(ValueError):
                self.run_faces(**kw)
        self.assertFalse(self.root.exists())

    def test_custom_cell_names_write_the_right_files(self):
        names = ["determined", "happy", "sad", "surprised"]
        self.run_faces(reduce="scale", names=names)
        d = self.root / "npcs" / "faces" / "talia_rusk"
        self.assertEqual(sorted(p.stem for p in d.glob("*.png")), sorted(names))
        plain = self.tmp / "plain"
        sheets.slice_faces(self.src, "talia_rusk", "npc", root=plain, faction="hearth", reduce="scale")
        for new, old in zip(names, sheets.FACE_CELLS):
            other = Image.open(plain / "npcs" / "faces" / "talia_rusk" / f"{old}.png").convert("RGBA")
            self.assertIsNone(ImageChops.difference(self.frame(new), other).getbbox(), new)
        out = self.tmp / "c2.png"
        self.run_faces(reduce="scale", names=names, contact_out=out)
        self.assertTrue(out.is_file())

    def test_bad_cell_names_refused_before_writing(self):
        for names in (["a", "b", "c"], ["a", "a", "b", "c"], ["Happy", "b", "c", "d"], ["a b", "b", "c", "d"]):
            with self.assertRaises(ValueError):
                self.run_faces(reduce="scale", names=names)
        with self.assertRaises(ValueError):
            self.run_faces(reduce="scale", names=["determined", "happy", "sad", "surprised"], cells={"hurt": self.src})
        with self.assertRaises(ValueError):
            self.run_faces(reduce="scale", names=["a", "b", "c", "d"], base=True)
        self.assertFalse(self.root.exists())

    def test_align_to_recovers_a_known_scale_and_shift(self):
        neutral = figure()
        cell, _ = sheets.place(neutral, 0.9, 9, -9, BG + (255,), Image.LANCZOS)
        sheet_im = Image.new("RGB", (SIZE, SIZE), BG)
        for box in sheets.face_boxes(sheet_im.size):
            sheet_im.paste(cell.resize((box[2] - box[0], box[3] - box[1])).convert("RGB"), box[:2])
        src = self.tmp / "second.png"
        sheet_im.save(src)
        ref = self.tmp / "ref.png"
        neutral.save(ref)
        lines = []
        sheets.slice_faces(src, "talia_rusk", "npc", root=self.root, faction="hearth", reduce="scale",
                           names=["determined", "happy", "sad", "surprised"], align_ref=ref, report=lines.append)
        self.assertEqual([l.split(":")[0] for l in lines], ["determined", "happy", "sad", "surprised"])
        self.assertTrue(all(abs(float(l.split("scale ")[1].split()[0]) - 1 / 0.9) < 0.02 for l in lines), lines)
        with self.assertRaises(ValueError):
            sheets.slice_faces(src, "talia_rusk", "npc", root=self.root, reduce="scale", align_ref=self.tmp / "none.png")
        with self.assertRaises(ValueError):
            sheets.slice_faces(src, "talia_rusk", "npc", root=self.root, align_ref=ref)

    def test_contact_and_cli(self):
        out = self.tmp / "c.png"
        r = subprocess.run([sys.executable, str(HERE / "sheets.py"), "faces", str(self.src), "talia_rusk", "npc", "--reduce",
                            "scale", "--align", "--cell", f"afraid={self.src}", "--contact", str(out), "--faction",
                            "hearth", "--root", str(self.root)], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("hurt: scale", r.stdout)
        with Image.open(out) as im:
            self.assertEqual(im.size, (8 + 5 * 248, 8 + 240 + 14 + 8))


if __name__ == "__main__":
    unittest.main()
