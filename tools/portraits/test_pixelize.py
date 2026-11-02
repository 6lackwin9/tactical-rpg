# Tests of pixelize.py (PIXEL2): the 120 px grid default, accents that keep a family of their own (a thin violet glow
# survives the reduction, a big gold area is no accent) and the rim that lifts dark silhouette pixels. PIXEL3: the
# faction backdrop ramps, the corner and glow styles, the edge rim (own ramp, only on the outer silhouette, only as
# far as it must to stand apart from the backdrop), the smooth and mode reductions and their pick, and faction None
# with the glow and the PIXEL2 rim reproducing the PIXEL2 output (fixtures/pixelize/pixel2_bust.png, written once by
# the pre-PIXEL3 pixelize from the `bust` source below; never re-record it). Synthetic sources only, no shipped art.
# Run: C:\dev\localgen\venv\Scripts\python.exe -m unittest test_pixelize
import colorsys
import json
import pathlib
import random
import shutil
import sys
import tempfile
import unittest

from PIL import Image, ImageDraw, ImageFilter

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
        kept, ramp = X.figure(src, mark=False, rim=False, reduce="mode")
        lost, plain = X.figure(src, mark=False, accents=False, rim=False, reduce="mode")
        n_kept = sum(1 for c in pixels(kept) if near(c))
        n_lost = sum(1 for c in pixels(lost) if near(c))
        self.assertGreater(n_kept, 30, "the vein is a line of grid pixels")
        self.assertGreater(n_kept, 2 * n_lost, "without accents most of the vein is lost")
        self.assertTrue([c for c in ramp if near(c + (255,))], "an accent family in the ramps")
        self.assertFalse([c for c in plain if near(c + (255,))], "no violet family without accents")

    def test_the_rim_lifts_dark_silhouette_pixels_up_their_ramp(self):
        src = self.source("h.png", lambda d: (shaded(d, (160, 160, 800, 800), (120, 80, 60), SKIN),
                                              shaded(d, (160, 160, 800, 400), HAIR, HAIR_LIT)))
        flat, ramp = X.figure(src, mark=False, rim=False, reduce="mode")
        rim, _ = X.figure(src, mark=False, rim="lift", reduce="mode")
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


FIX = pathlib.Path(__file__).resolve().parent / "fixtures" / "pixelize"


def bust(d):
    """A bust on the flat background: shoulders cut off by the frame's bottom edge, a dark hair oval, a shaded face
    and a thin violet vein (the source the PIXEL2 fixture was made from; keep it as it is)."""
    shaded(d, (200, 600, 760, 959), (90, 60, 40), (150, 110, 80))
    d.ellipse([300, 120, 660, 620], fill=HAIR)
    shaded(d, (390, 260, 590, 560), (120, 80, 60), SKIN)
    d.line([430, 300, 560, 520], fill=VIOLET, width=3)


def patch_light(im, cx, cy, r=2):
    """The mean HLS lightness of the (2r+1)^2 patch round (cx, cy): a dithered backdrop's local tone."""
    px = im.load()
    vals = [light(px[x, y]) for y in range(cy - r, cy + r + 1) for x in range(cx - r, cx + r + 1)]
    return sum(vals) / len(vals)


class Factions(unittest.TestCase):
    """PIXEL3 (decision "Portrait backdrops and rims")."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = pathlib.Path(tempfile.mkdtemp(prefix="pixelize3_"))
        im = Image.new("RGB", (960, 960), BG)
        bust(ImageDraw.Draw(im))
        cls.src = cls.tmp / "bust.png"
        im.save(cls.src)
        # without the outline ring, so the silhouette is the one the rim saw
        cls.plain, cls.ramp = X.figure(cls.src, mark=False, rim=False, outline=False, reduce="mode")
        cls.rimmed = {f: X.figure(cls.src, mark=False, faction=f, outline=False, reduce="mode")[0] for f in ("hearth", "clerisy")}

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, True)

    def test_every_faction_has_a_dark_ramp_from_dark_to_light(self):
        self.assertEqual(set(X.FACTIONS), {"hearth", "castellan", "clerisy", "guild", "corrupted", "creature"})
        self.assertEqual(X.BACKDROPS["hearth"], X.BG, "hearth is the PIXEL2 background")
        for f, ramp in X.BACKDROPS.items():
            self.assertEqual(len(ramp), 4, f)
            ls = [light(c) for c in ramp]
            self.assertEqual(ls, sorted(ls), f"{f} runs dark to light")
            self.assertEqual(len(set(ramp)), len(ramp), f)
            self.assertLess(max(ls), 0.30, f"{f} stays a dark backdrop")
        self.assertEqual(len({tuple(r) for r in X.BACKDROPS.values()}), len(X.FACTIONS), "each faction its own ramp")

    def test_an_unknown_faction_or_backdrop_is_an_error(self):
        with self.assertRaises(ValueError):
            X.finish(self.plain, faction="elves")
        with self.assertRaises(ValueError):
            X.finish(self.plain, backdrop="halo")
        self.assertEqual(X.faction_key(None), "hearth")

    def test_corner_is_darkest_behind_the_face_and_glow_lightest(self):
        g = X.GRID; s = 240 // g
        empty = Image.new("RGBA", (g, g), (0, 0, 0, 0))
        face = (int(g * 0.58) * s, int(g * 0.40) * s)
        inset = 8 * s   # inside the border and its corner squares
        corners = [(inset, inset), (240 - inset, inset), (inset, 240 - inset), (240 - inset, 240 - inset)]
        for f, ramp in X.BACKDROPS.items():
            corner = X.finish(empty, faction=f, backdrop="corner")
            glow = X.finish(empty, faction=f, backdrop="glow")
            self.assertEqual(corner.getpixel(face)[:3], ramp[0], f"{f}: corner style is darkest at the face")
            self.assertEqual(glow.getpixel(face)[:3], ramp[-1], f"{f}: glow style is lightest at the face")
            for c in corners:
                self.assertGreater(patch_light(corner, *c), patch_light(corner, *face), f"{f}: corner {c} brighter")
                self.assertLess(patch_light(glow, *c), patch_light(glow, *face), f"{f}: glow {c} darker")
            colours = {corner.getpixel((x, y))[:3] for y in range(inset, 240 - inset, 7) for x in range(inset, 240 - inset, 7)}
            self.assertEqual(colours, set(ramp), f"{f}: the corner backdrop uses its whole ramp and nothing else")

    def test_the_default_is_the_corner_backdrop_and_the_hearth(self):
        default = X.pixelize(self.src, mark=False)[0]
        hearth = X.pixelize(self.src, mark=False, faction="hearth", backdrop="corner", rim="edge")[0]
        self.assertEqual(default.tobytes(), hearth.tobytes())

    def test_faction_none_with_the_glow_and_the_pixel2_rim_reproduces_pixel2(self):
        img, _ = X.pixelize(self.src, mark=False, faction=None, backdrop="glow", rim="lift", reduce="mode")
        old = Image.open(FIX / "pixel2_bust.png").convert("RGBA")
        self.assertEqual(img.size, old.size)
        self.assertEqual(img.tobytes(), old.tobytes())

    def test_the_edge_rim_lifts_only_outer_silhouette_pixels_in_their_own_ramp(self):
        g = X.GRID; pp = self.plain.load()
        outer = set(X.outer_silhouette(pp, g))
        self.assertFalse([p for p in outer if p[1] == g - 1 and not any(
            0 <= p[0] + dx < g and pp[p[0] + dx, p[1]][3] == 0 for dx in (-1, 1))], "the frame's edge is no silhouette")
        shades = X.PALETTE["mode"][1]; fams = len(self.ramp) - 1
        for f, fig in self.rimmed.items():
            fp = fig.load(); bg = X.BACKDROPS[f]
            changed = [(x, y) for y in range(g) for x in range(g) if fp[x, y] != pp[x, y]]
            self.assertTrue(changed, f"{f}: the dark hair gets a rim")
            self.assertTrue(set(changed) <= outer, f"{f}: never inside the figure")
            for x, y in changed:
                was, now = pp[x, y][:3], fp[x, y][:3]
                self.assertIn(now, self.ramp[:fams], f"{f}: a colour of the portrait's own ramps, no tint")
                i, j = self.ramp.index(was), self.ramp.index(now)
                self.assertEqual(i // shades, j // shades, f"{f}: the same family")
                self.assertIn(j - i, range(1, X.RIM_SHADES + 1), f"{f}: one or two shades up")
            hair = [(x, y) for x, y in changed if y < g // 2]
            self.assertTrue(hair, f"{f}: the rim runs round the hair")
            for x, y in changed:                         # only where the edge merged with the backdrop behind it
                back = X.P.backdrop_pixel(g, x, y, bg, "corner")
                self.assertGreater(X.rim_need(pp[x, y][:3], self.ramp, shades, back), 0, f"{f}: {(x, y)}")

    # A 12 x 12 figure (a block reaching the bottom edge) on one-colour backdrops, one family of 4 shades.
    FAM = [(30, 22, 18), (60, 44, 34), (100, 74, 56), (150, 112, 86), X.P.OUT]
    DARK = [(40, 32, 30)] * 4      # luma about 35: FAM[0] needs two shades, FAM[1] one, FAM[3] none
    LIGHT = [(160, 150, 140)] * 4  # luma about 151: every FAM shade but the top already stands apart

    def block(self, fill, spots=()):
        g = 12; im = Image.new("RGBA", (g, g), (0, 0, 0, 0)); px = im.load()
        for y in range(2, g):
            for x in range(2, 10):
                px[x, y] = fill + (255,)
        for p, c in spots:
            px[p] = c + (255,)
        return im, px

    def test_an_edge_over_a_dark_backdrop_lifts_and_over_a_light_one_stays(self):
        fam = self.FAM
        self.assertEqual(X.rim_need(fam[0], fam, 4, self.DARK[0]), 2)
        self.assertEqual(X.rim_need(fam[1], fam, 4, self.DARK[0]), 1)
        self.assertEqual(X.rim_need(fam[3], fam, 4, self.DARK[0]), 0, "the lit side stays")
        self.assertEqual(X.rim_need(X.P.OUT, fam, 4, self.DARK[0]), 0, "the outline colour stays")
        self.assertEqual(X.rim_need(fam[1], fam, 4, self.LIGHT[0]), 0)
        _, dp = self.block(fam[1])
        X.edge_rim(dp, 12, fam, 4, self.DARK, "corner")
        self.assertEqual(dp[2, 5][:3], fam[2], "over the dark backdrop the edge lifts one shade")
        self.assertEqual(dp[5, 5][:3], fam[1], "the inside stays")
        _, lp = self.block(fam[1])
        X.edge_rim(lp, 12, fam, 4, self.LIGHT, "corner")
        self.assertEqual([lp[x, y] for y in range(12) for x in range(12)],
                         [self.block(fam[1])[1][x, y] for y in range(12) for x in range(12)], "over the light one it stays")

    def test_a_run_is_lifted_evenly_by_its_largest_need(self):
        fam = self.FAM
        spots = [((x, 2), fam[0]) for x in range(2, 6)]                   # the top-left of the edge darker
        im, px = self.block(fam[1], spots)
        outer = X.outer_silhouette(px, 12)
        before = {p: px[p][:3] for p in outer}
        X.edge_rim(px, 12, fam, 4, self.DARK, "corner")
        lifts = {fam.index(px[p][:3]) - fam.index(before[p]) for p in outer}
        self.assertEqual(lifts, {2}, "one lift for the whole run (the darker part needs two shades)")

    def test_a_run_of_two_pixels_is_skipped(self):
        fam = self.FAM
        two = [((4, 2), fam[0]), ((5, 2), fam[0])]                        # the rest of the edge stands apart
        im, px = self.block(fam[3], two)
        X.edge_rim(px, 12, fam, 4, self.DARK, "corner")
        self.assertEqual((px[4, 2][:3], px[5, 2][:3]), (fam[0], fam[0]), "no speckles")
        im, px = self.block(fam[3], two + [((6, 2), fam[0])])
        X.edge_rim(px, 12, fam, 4, self.DARK, "corner")
        self.assertEqual({px[x, 2][:3] for x in (4, 5, 6)}, {fam[2]}, "three pixels make a run")

    def test_a_hole_inside_the_figure_gets_no_rim(self):
        g = 12; dark = (40, 30, 26)
        ramp = [(40, 30, 26), (70, 52, 40), (110, 82, 62), (150, 112, 86), X.P.OUT]
        im = Image.new("RGBA", (g, g), (0, 0, 0, 0)); px = im.load()
        for y in range(2, g):          # a block reaching the bottom edge, a clear hole in its middle
            for x in range(2, 10):
                px[x, y] = dark + (255,)
        px[5, 6] = px[6, 6] = (0, 0, 0, 0)
        X.edge_rim(px, g, ramp, 4, X.BACKDROPS["guild"], "corner")
        self.assertNotEqual(px[2, 5][:3], dark, "the outer edge gets the rim")
        for p in ((4, 6), (7, 6), (5, 5), (6, 7), (5, g - 1), (6, g - 1)):
            self.assertEqual(px[p][:3], dark, f"{p} borders the hole or the frame's edge, not the backdrop")

    def test_the_faction_comes_from_the_record_beside_the_source(self):
        src = self.tmp / "who.png"
        Image.new("RGB", (4, 4)).save(src)
        self.assertIsNone(X.faction_of(src), "no record: the default")
        (self.tmp / "who.json").write_text(json.dumps({"id": "who", "faction": "guild"}), encoding="utf-8")
        self.assertEqual(X.faction_of(src), "guild")

    def test_the_cli_takes_a_faction(self):
        out = self.tmp / "cli.png"
        X.main([str(self.src), str(out), "--faction", "castellan", "--no-mark"])
        want = X.pixelize(self.src, mark=False, faction="castellan")[0]
        self.assertEqual(Image.open(out).convert("RGBA").tobytes(), want.tobytes())
        X.main([str(self.src), str(out), "--reduce", "mode", "--no-mark"])
        want = X.pixelize(self.src, mark=False, reduce="mode")[0]
        self.assertEqual(Image.open(out).convert("RGBA").tobytes(), want.tobytes())


ART_SRC = pathlib.Path(__file__).resolve().parents[2] / "art_src"
GREY = (176, 178, 188)
RED = (230, 24, 24)


def is_red(c):
    h, l, s = colorsys.rgb_to_hls(*(v / 255 for v in c[:3]))
    return c[3] and s > 0.1 and l > 0.2 and (h < 0.08 or h > 0.9)   # reddish or pink; not the dark outline


class Reduce(unittest.TestCase):
    """PIXEL3 (user 2026-10-27): the smooth reduction for smooth sources, the mode one for pixel art, picked by grid."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="pixelize_reduce_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)

    def test_a_blocky_image_has_a_grid_and_a_smooth_one_none(self):
        rng = random.Random(7)
        small = Image.new("RGB", (120, 120)); sp = small.load()
        for y in range(120):
            for x in range(120):
                sp[x, y] = tuple(rng.randrange(256) for _ in range(3))
        blocky = small.resize((960, 960), Image.NEAREST)              # art pixels 8 source px wide
        score, period = X.grid_period(blocky)
        self.assertGreaterEqual(score, X.GRID_SCORE)
        self.assertAlmostEqual(period, 8, delta=0.5)
        smooth = Image.new("RGB", (960, 960)); d = ImageDraw.Draw(smooth)
        for r in range(680, 0, -4):                                     # a soft radial gradient with soft shapes
            d.ellipse([480 - r, 400 - r, 480 + r, 400 + r], fill=(30 + r // 6, 20 + r // 8, 40 + r // 10))
        smooth = smooth.filter(ImageFilter.GaussianBlur(6))
        self.assertLess(X.grid_period(smooth)[0], X.GRID_SCORE)
        for name, im, want in (("blocky.png", blocky, "mode"), ("smooth.png", smooth, "smooth")):
            im.save(self.tmp / name)
            self.assertEqual(X.reduce_of(self.tmp / name), want)
        self.assertEqual(X.reduce_of(self.tmp / "smooth.png", reduce="mode"), "mode", "an explicit choice wins")
        with self.assertRaises(ValueError):
            X.reduce_of(self.tmp / "smooth.png", reduce="blur")

    def test_the_smooth_reduction_keeps_a_thin_line_the_mode_one_loses(self):
        im = Image.new("RGB", (960, 960), BG); d = ImageDraw.Draw(im)
        d.rectangle([120, 120, 840, 959], fill=GREY)                     # a flat figure: one colour wins every vote
        d.line([200, 300, 760, 700], fill=RED, width=1)                 # 1 source px across about 70 grid cells
        src = self.tmp / "line.png"; im.save(src)
        mode, _ = X.figure(src, mark=False, rim=False, accents=False, reduce="mode")  # not an accent family
        smooth, ramp = X.figure(src, mark=False, rim=False, accents=False, reduce="smooth")
        n_mode = sum(1 for c in pixels(mode) if is_red(c))
        n_smooth = sum(1 for c in pixels(smooth) if is_red(c))
        self.assertGreater(n_smooth, 30, "the line is a run of reddish grid pixels")
        self.assertGreater(n_smooth, 3 * n_mode, f"the per-cell vote loses most of it ({n_smooth} vs {n_mode})")
        self.assertEqual(X.PALETTE, {"mode": (10, 4), "smooth": (14, 5)})
        self.assertEqual(len(ramp) % X.PALETTE["smooth"][1], 1, "5 shades a family, then the outline colour")

    def test_the_smooth_path_keeps_no_accent_fringe_on_the_silhouette(self):
        # a teal fringe 6 source px wide round a grey figure (a cut-out halo), and a teal gem inside: the accent hue.
        # Before the fix (plain RGB Lanczos, accents up to the edge) 63 ring pixels took the gem's accent colours.
        im = Image.new("RGB", (960, 960), BG); d = ImageDraw.Draw(im)
        d.ellipse([154, 114, 806, 1102], fill=(40, 200, 170))
        d.ellipse([160, 120, 800, 1096], fill=GREY)
        d.ellipse([470, 400, 494, 424], fill=(40, 220, 190))
        src = self.tmp / "fringe.png"; im.save(src)
        info = {}
        fig, _ = X.figure(src, mark=False, rim=False, outline=False, reduce="smooth", info=info)
        self.assertEqual(info["reduce"], "smooth")
        accent = set(info["accent_ramp"])
        self.assertTrue(accent, "the gem makes an accent family")
        fp = fig.load(); g = X.GRID
        ring = [(x, y) for y in range(g) for x in range(g) if fp[x, y][3] and X.near_silhouette(fp, g, x, y)]
        self.assertTrue(ring)
        self.assertEqual([fp[p] for p in ring if fp[p][:3] in accent], [], "no accent colour on the outer ring")
        self.assertTrue([c for c in pixels(fig) if c[3] and c[:3] in accent], "the gem inside keeps its accent")

    def test_auto_picks_mode_for_the_image_api_portrait_sources(self):
        recs = sorted(p for p in ART_SRC.glob("*.json") if "faction" in json.loads(p.read_text(encoding="utf-8")))
        checked = 0
        for rec in recs:
            r = json.loads(rec.read_text(encoding="utf-8"))
            png = rec.with_suffix(".png")
            if str(r.get("model", "")).startswith("local:") or not png.is_file():
                continue
            if r.get("via"):                                            # image-queue art takes the scale-only path, not reduce_of
                continue
            try:
                Image.open(png).verify()
            except Exception:
                continue                                                # a Git LFS pointer, not the image
            self.assertEqual(X.reduce_of(png), "mode", png.name)
            checked += 1
        if not checked:
            self.skipTest("no portrait sources (Git LFS not pulled)")


if __name__ == "__main__":
    unittest.main()
