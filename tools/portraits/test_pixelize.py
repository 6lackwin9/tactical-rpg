# Tests of pixelize.py (PIXEL2): the 120 px grid default, accents that keep a family of their own (a thin violet glow
# survives the reduction, a big gold area is no accent) and the rim that lifts dark silhouette pixels. Synthetic
# sources only, no shipped art. Run: C:\dev\localgen\venv\Scripts\python.exe -m unittest test_pixelize
import colorsys
import pathlib
import shutil
import sys
import tempfile
import unittest

from PIL import Image, ImageDraw

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import pixelize as X  # noqa: E402

BG = (18, 14, 18)
SKIN = (196, 150, 120)
GOLD = (214, 160, 40)
VIOLET = (170, 60, 240)
HAIR = (56, 40, 30)  # a dark brown, darker than RIM_DARK (outside the background's tolerance)
HAIR_LIT = (126, 88, 62)  # its lit side


def shaded(d, box, top, bottom):
    """A vertical gradient: shaded colours, as a source has, so the families get ramps from dark to light."""
    x0, y0, x1, y1 = box
    for y in range(y0, y1 + 1):
        k = (y - y0) / max(1, y1 - y0)
        d.line([x0, y, x1, y], fill=tuple(int(a + (b - a) * k) for a, b in zip(top, bottom)))


def pixels(im):
    px = im.load()
    return [px[x, y] for y in range(im.height) for x in range(im.width)]


def light(c):
    return colorsys.rgb_to_hls(*(v / 255 for v in c[:3]))[1]


class Pixelize(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="pixelize_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)

    def source(self, name, draw):
        im = Image.new("RGB", (960, 960), BG)
        draw(ImageDraw.Draw(im))
        path = self.tmp / name
        im.save(path)
        return path

    def glow_face(self, d):
        d.rectangle([160, 160, 800, 800], fill=SKIN)       # the face
        d.rectangle([160, 640, 800, 800], fill=GOLD)       # a breastplate: saturated, but a main colour
        d.line([300, 200, 520, 600], fill=VIOLET, width=3)  # a thin vein glow, 3 source px (under half a grid cell)

    def test_the_default_grid_is_120(self):
        self.assertEqual(X.GRID, 120)
        fig, _ = X.figure(self.source("a.png", self.glow_face), mark=False)
        self.assertEqual(fig.size, (X.GRID, X.GRID))

    def test_a_thin_saturated_glow_is_an_accent_and_a_gold_area_is_not(self):
        im = X.cut_background(Image.open(self.source("a.png", self.glow_face)).convert("RGBA"))
        bins = X.accent_bins(im)
        self.assertIn(X.hue_bin(X.hls_of(VIOLET)[0]), bins)
        self.assertNotIn(X.hue_bin(X.hls_of(GOLD)[0]), bins)

    def test_the_glow_survives_only_with_accents(self):
        src = self.source("a.png", self.glow_face)
        vh = X.hls_of(VIOLET)[0]
        near = lambda c: c[3] and X.saturated(c) and abs(X.hls_of(c)[0] - vh) < 1 / X.HUE_BINS * 2
        kept, ramp = X.figure(src, mark=False, rim=False)
        lost, plain = X.figure(src, mark=False, accents=False, rim=False)
        n_kept = sum(1 for c in pixels(kept) if near(c))
        n_lost = sum(1 for c in pixels(lost) if near(c))
        self.assertGreater(n_kept, 30, "the vein is a line of grid pixels")
        self.assertGreater(n_kept, 2 * n_lost, "without accents most of the vein is lost")
        self.assertTrue([c for c in ramp if near(c + (255,))], "an accent family in the ramps")
        self.assertFalse([c for c in plain if near(c + (255,))], "no violet family without accents")

    def test_the_rim_lifts_dark_silhouette_pixels_up_their_ramp(self):
        src = self.source("h.png", lambda d: (shaded(d, (160, 160, 800, 800), (120, 80, 60), SKIN),
                                              shaded(d, (160, 160, 800, 400), HAIR, HAIR_LIT)))
        flat, ramp = X.figure(src, mark=False, rim=False)
        rim, _ = X.figure(src, mark=False)
        fp, rp = flat.load(), rim.load()
        g = X.GRID
        edge = [(x, y) for y in range(g) for x in range(g) if fp[x, y][3] and fp[x, y][:3] != X.P.OUT and
                any(fp[x + dx, y + dy][3] == 0 or fp[x + dx, y + dy][:3] == X.P.OUT
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)) if 0 <= x + dx < g and 0 <= y + dy < g)]
        dark = [(x, y) for x, y in edge if light(fp[x, y]) < X.RIM_DARK]
        self.assertTrue(dark, "the hair's silhouette is dark before the rim")
        lifted = [(x, y) for x, y in dark if light(rp[x, y]) > light(fp[x, y])]
        self.assertTrue(lifted, "the rim lifts dark silhouette pixels")
        self.assertTrue(all(rp[x, y][:3] in ramp for x, y in lifted), "a lifted pixel keeps a ramp colour")
        inner = [(x, y) for y in range(g) for x in range(g) if (x, y) not in edge and fp[x, y][3]]
        self.assertTrue(all(rp[x, y] == fp[x, y] for x, y in inner), "only the silhouette changes")


if __name__ == "__main__":
    unittest.main()
