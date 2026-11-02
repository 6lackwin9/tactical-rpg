"""LOOK1-T tests: temp dirs and fixtures only."""
import contextlib
import io
import os
import random
import subprocess
import sys
import tempfile
import unittest

from PIL import Image

import board
import compose
import cut
import masks
import seam


def pixels(img):
    return [img.getpixel((x, y)) for y in range(img.size[1]) for x in range(img.size[0])]


def size_of(path):
    with Image.open(path) as im:
        return im.size


def noise_tile(w, h, seed, levels=(30, 90, 150, 210)):
    rnd = random.Random(seed)
    img = Image.new("RGB", (w, h))
    for y in range(h):
        for x in range(w):
            v = rnd.choice(levels)
            img.putpixel((x, y), (v, v // 2 + 20, 255 - v))
    return img


def checker(w, h):
    img = Image.new("RGB", (w, h))
    for y in range(h):
        for x in range(w):
            v = 40 + 100 * ((x + y) % 2)
            img.putpixel((x, y), (v, v, v))
    return img


def gradient(w, h):
    img = Image.new("RGB", (w, h))
    for y in range(h):
        for x in range(w):
            img.putpixel((x, y), (x * 8, x * 8, x * 8))
    return img


class CutTest(unittest.TestCase):
    def test_recovers_scaled_fixture_with_border(self):
        src = noise_tile(32, 32, 1)
        scale = 7
        big = Image.new("RGB", (32 * scale + 2 * scale, 32 * scale + 2 * scale), (200, 200, 200))
        big.paste(src.resize((32 * scale, 32 * scale), Image.NEAREST), (scale, scale))
        self.assertEqual(cut.block_size(big), scale)
        self.assertEqual(pixels(cut.cut(big, (32, 32))), pixels(src))

    def _framed(self, art, scale, lead, trail, colour=(200, 200, 200)):
        """`art` scaled, with a border of lead/trail px (x, y) around it."""
        w, h = art.size
        big = Image.new("RGB", (w * scale + lead[0] + trail[0], h * scale + lead[1] + trail[1]), colour)
        big.paste(art.resize((w * scale, h * scale), Image.NEAREST), lead)
        return big

    def test_one_sided_border(self):
        src = noise_tile(32, 32, 11)
        big = self._framed(src, 8, (0, 0), (8, 8))
        self.assertEqual(pixels(cut.cut(big, (32, 32))), pixels(src))
        big = self._framed(src, 8, (8, 8), (0, 0))
        self.assertEqual(pixels(cut.cut(big, (32, 32))), pixels(src))

    def test_noisy_ring_is_an_error(self):
        src = noise_tile(32, 32, 12)
        big = self._framed(src, 8, (8, 8), (8, 8))
        rnd = random.Random(1)
        for i in range(big.size[0]):
            big.putpixel((i, 0), (rnd.randrange(256),) * 3)
        with self.assertRaises(ValueError):
            cut.cut(big, (32, 32))

    def test_oversize_art_is_an_error(self):
        big = noise_tile(34, 34, 13).resize((34 * 8, 34 * 8), Image.NEAREST)
        with self.assertRaises(ValueError):
            cut.cut(big, (32, 32))

    def test_malformed_size(self):
        with self.assertRaises(ValueError):
            cut.parse_size("32")
        with tempfile.TemporaryDirectory() as d:
            src = os.path.join(d, "a.png")
            noise_tile(32, 32, 3).save(src)
            self.assertEqual(cut.main([src, "--size", "abc", "--out", os.path.join(d, "o.png")]), 1)

    def test_wrong_size_is_an_error(self):
        big = noise_tile(32, 32, 2).resize((224, 224), Image.NEAREST)
        with self.assertRaises(ValueError):
            cut.cut(big, (32, 24))

    def test_cli_error_exit(self):
        with tempfile.TemporaryDirectory() as d:
            src = os.path.join(d, "a.png")
            noise_tile(32, 32, 3).resize((96, 96), Image.NEAREST).save(src)
            self.assertEqual(cut.main([src, "--size", "64x48", "--out", os.path.join(d, "o.png")]), 1)
            self.assertEqual(cut.main([src, "--size", "32x32", "--out", os.path.join(d, "o.png")]), 0)
            self.assertEqual(size_of(os.path.join(d, "o.png")), (32, 32))


def colours_of(img):
    return set(pixels(img.convert("RGB")))


def size_check(kind, img, role):
    """The result of board.checks' size line, found by its text."""
    return next(ok for text, ok in board.checks(kind, img, role) if text.startswith("size "))


class BoxTest(unittest.TestCase):
    """--reduce box and --form: the no-clean-block fixture is per-pixel noise, whose detected block is 1 px."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = tmp.name
        self.out = os.path.join(self.dir, "o.png")

    def source(self, img):
        path = os.path.join(self.dir, "src.png")
        img.save(path)
        return path

    def run_err(self, *args):
        err = io.StringIO()
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
            code = cut.main([*args, "--out", self.out])
        return code, err.getvalue()

    def test_auto_refuses_no_clean_block_and_names_box(self):
        src = noise_tile(256, 256, 21)
        self.assertEqual(cut.block_size(src), 1)
        code, err = self.run_err(self.source(src), "--size", "64x64")
        self.assertEqual(code, 1)
        self.assertIn("--reduce box", err)
        self.assertFalse(os.path.exists(self.out))
        code, _ = self.run_err(self.source(src), "--size", "64x64", "--reduce", "auto")
        self.assertEqual(code, 1)

    def test_box_reduces_a_no_clean_block_image(self):
        src = noise_tile(256, 256, 22)
        self.assertEqual(self.run_err(self.source(src), "--size", "64x64", "--reduce", "box")[0], 0)
        with Image.open(self.out) as got:
            self.assertEqual(got.size, (64, 64))
            self.assertEqual(pixels(got), pixels(src.resize((64, 64), Image.BOX)))

    def test_box_averages_each_cell(self):
        # each output cell is a 2 x 2 checker of two values: the mean is the box's result, not a sampled pixel
        got = cut.box(checker(8, 8), (4, 4))
        self.assertEqual(colours_of(got), {(90, 90, 90)})

    def test_forms_match_the_board_sizes(self):
        src = noise_tile(128, 128, 23)
        for form, (size, banded) in cut.FORMS.items():
            role = "stage" if form.startswith("stage") else "region"
            kind = "wall_face" if banded else "floor"
            got = cut.cut_form(src, form)
            self.assertEqual(got.size, size)
            self.assertIn(size, board.SIZES[role]["face" if banded else "tile"], form)
            self.assertTrue(size_check(kind, got, role), form)

    def test_every_board_size_has_a_form(self):
        have = {size for size, _ in cut.FORMS.values()}
        for role in board.SIZES.values():
            self.assertLessEqual(role["tile"] | role["face"], have)

    def test_stage_is_one_step_from_the_source(self):
        src = noise_tile(256, 256, 24)
        got = cut.cut_form(src, "stage")
        self.assertEqual(pixels(got), pixels(src.resize((64, 48), Image.BOX)))
        two_steps = src.resize((64, 64), Image.BOX).resize((64, 48), Image.BOX)
        self.assertNotEqual(pixels(got), pixels(two_steps))

    def test_region_tilted_is_one_step_from_the_source(self):
        src = noise_tile(256, 256, 34)
        self.assertEqual(pixels(cut.cut_form(src, "region_tilted")), pixels(src.resize((32, 24), Image.BOX)))

    def test_faces_are_the_centred_band_unsquashed(self):
        # the art scaled up fills the band of a 256 px square exactly; the rest is a colour the art does not have
        for form, (size, banded) in cut.FORMS.items():
            if not banded:
                continue
            scale = 256 // size[0]
            art = noise_tile(*size, 25)
            high = size[1] * scale
            src = Image.new("RGB", (256, 256), (255, 0, 255))
            src.paste(art.resize((256, high), Image.NEAREST), (0, (256 - high) // 2))
            self.assertEqual(pixels(cut.cut_form(src, form)), pixels(art), form)

    def test_stage_face_needs_the_height(self):
        with self.assertRaises(ValueError):
            cut.cut_form(noise_tile(256, 90, 26), "stage_face")   # the band would be 96 px high

    def test_square_forms_refuse_another_shape(self):
        src = noise_tile(160, 120, 27)
        for form, (_, banded) in cut.FORMS.items():
            if not banded:
                with self.assertRaises(ValueError):
                    cut.cut_form(src, form)
        self.assertEqual(cut.cut_form(noise_tile(130, 128, 28), "stage").size, (64, 48))   # a size ingest accepts

    def test_wrong_size_of_a_clean_block_does_not_suggest_box(self):
        src = noise_tile(32, 32, 35).resize((96, 96), Image.NEAREST)
        self.assertEqual(cut.block_size(src), 3)
        code, err = self.run_err(self.source(src), "--size", "64x48")
        self.assertEqual(code, 1)
        self.assertIn("block 3 px", err)
        self.assertNotIn("--reduce box", err)

    def test_smaller_source_is_refused_not_scaled_up(self):
        small = noise_tile(40, 40, 36)
        with self.assertRaises(ValueError):
            cut.box(small, (64, 64))
        with self.assertRaises(ValueError):
            cut.box(noise_tile(64, 40, 36), (64, 48))             # one axis is enough
        with self.assertRaises(ValueError):
            cut.cut_form(small, "stage")
        with self.assertRaises(ValueError):
            cut.cut_form(small, "stage_face")                     # the band is too narrow as well as too low
        path = self.source(small)
        code, err = self.run_err(path, "--size", "64x64", "--reduce", "box")
        self.assertEqual(code, 1)
        self.assertIn("does not scale up", err)
        self.assertFalse(os.path.exists(self.out))
        self.assertEqual(cut.box(noise_tile(64, 48, 37), (64, 48)).size, (64, 48))   # the same size is no upscale

    def test_bad_arguments_stop_before_the_file_is_opened(self):
        missing = os.path.join(self.dir, "no_such.png")
        code, err = self.run_err(missing, "--size", "abc")
        self.assertEqual(code, 1)
        self.assertIn("size 'abc' is not WxH", err)
        code, err = self.run_err(missing, "--form", "stage", "--colours", "300")
        self.assertEqual(code, 1)
        self.assertIn("1 to 256", err)

    def test_cli_forms_and_option_clashes(self):
        path = self.source(noise_tile(128, 128, 29))
        for form in sorted(cut.FORMS):
            self.assertEqual(self.run_err(path, "--form", form)[0], 0)
            self.assertEqual(size_of(self.out), cut.FORMS[form][0])
        self.assertEqual(self.run_err(path)[0], 1)                                       # neither --size nor --form
        self.assertEqual(self.run_err(path, "--form", "stage", "--size", "64x48")[0], 1)
        self.assertEqual(self.run_err(path, "--form", "stage", "--reduce", "box")[0], 1)
        for bad in ("0", "-1", "257"):
            code, err = self.run_err(path, "--form", "stage", "--colours", bad)
            self.assertEqual(code, 1, bad)
            self.assertIn("1 to 256", err)
        self.assertEqual(self.run_err(path, "--form", "stage", "--colours", "256")[0], 0)


class ColoursTest(unittest.TestCase):
    def rich(self, seed=31):
        return noise_tile(256, 256, seed).resize((64, 48), Image.BOX)

    def test_limit_and_own_colours_only(self):
        src = self.rich()
        self.assertGreater(len(colours_of(src)), 16)
        for n in (1, 2, 16):
            got = cut.limit_colours(src, n)
            self.assertEqual(got.size, src.size)
            self.assertLessEqual(len(colours_of(got)), n)
            self.assertLessEqual(colours_of(got), colours_of(src))

    def test_few_colours_are_unchanged(self):
        src = noise_tile(32, 32, 32)   # four levels
        self.assertEqual(pixels(cut.limit_colours(src, 16)), pixels(src))
        self.assertEqual(pixels(cut.limit_colours(src, len(colours_of(src)))), pixels(src))

    def test_no_dither(self):
        # flat areas of one colour stay flat: every pixel of a colour maps to the same pick
        src = self.rich()
        got = cut.limit_colours(src, 4)
        seen = {}
        for a, b in zip(pixels(src), pixels(got)):
            self.assertEqual(seen.setdefault(a, b), b)

    def test_deterministic(self):
        # separate processes with different string and set hashing give the same bytes
        code = ("import hashlib, cut, test_tiles; "
                "print(hashlib.sha256(cut.limit_colours(test_tiles.ColoursTest().rich(), 16).tobytes()).hexdigest())")
        here = os.path.dirname(os.path.abspath(__file__))
        digests = set()
        for seed in ("0", "1", "4242"):
            env = {**os.environ, "PYTHONHASHSEED": seed}
            done = subprocess.run([sys.executable, "-c", code], cwd=here, env=env, capture_output=True, text=True, check=True)
            digests.add(done.stdout.strip())
        self.assertEqual(len(digests), 1)
        self.assertEqual(len(digests.pop()), 64)

    def test_board_check_passes_and_cli_default_is_off(self):
        with tempfile.TemporaryDirectory() as d:
            src, out = os.path.join(d, "s.png"), os.path.join(d, "o.png")
            noise_tile(256, 256, 33).save(src)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(cut.main([src, "--form", "stage", "--out", out]), 0)
                with Image.open(out) as plain:
                    self.assertEqual(pixels(plain), pixels(self.rich(33)))
                self.assertEqual(cut.main([src, "--form", "stage", "--colours", str(board.MAX_COLOURS), "--out", out]), 0)
            with Image.open(out) as got:
                colours = dict((t.split()[0], ok) for t, ok in board.checks("floor", got, "stage"))
                self.assertTrue(colours["colours"])
                self.assertLessEqual(colours_of(got), colours_of(self.rich(33)))


class SeamTest(unittest.TestCase):
    def test_scores(self):
        self.assertLessEqual(seam.score(checker(32, 32)), 1.0)
        self.assertGreater(seam.score(gradient(32, 32)), 1.5)

    def test_fix_lowers_the_score(self):
        img = gradient(32, 32)  # the striped fixture
        before = seam.score(img)
        after = seam.score(seam.fix(img))
        self.assertGreater(before, 1.5)
        self.assertLess(after, before)


class MasksTest(unittest.TestCase):
    def test_distinct_and_quarters_follow_bits(self):
        for w, h in masks.SIZES:
            grids = [masks.mask_grid(m, w, h) for m in range(1, 15)]
            self.assertEqual(len({tuple(map(tuple, g)) for g in grids}), 14)
            centres = [(w // 4, h // 4), (3 * w // 4, h // 4), (w // 4, 3 * h // 4), (3 * w // 4, 3 * h // 4)]
            for m, g in enumerate(grids, 1):
                for bit, (x, y) in enumerate(centres):
                    self.assertEqual(g[y][x], bool(m & (1 << bit)), (w, h, m, bit))

    def test_deterministic(self):
        self.assertEqual(masks.mask_grid(6, 32, 24), masks.mask_grid(6, 32, 24))

    def test_zero_displacement_at_borders(self):
        for w, h in masks.SIZES:
            for m in range(1, 15):
                g, p = masks.mask_grid(m, w, h), masks.plain_quarters(m, w, h)
                self.assertEqual(g[0], p[0])
                self.assertEqual(g[h - 1], p[h - 1])
                self.assertEqual([r[0] for r in g], [r[0] for r in p])
                self.assertEqual([r[w - 1] for r in g], [r[w - 1] for r in p])

    def test_boundary_is_ragged(self):
        self.assertTrue(any(masks.mask_grid(m, 32, 32) != masks.plain_quarters(m, 32, 32) for m in range(1, 15)))


class ComposeTest(unittest.TestCase):
    def test_output(self):
        with tempfile.TemporaryDirectory() as d:
            up, lo = os.path.join(d, "up.png"), os.path.join(d, "lo.png")
            noise_tile(32, 24, 4).save(up)
            noise_tile(32, 24, 5, (10, 60, 110, 160)).save(lo)
            out = os.path.join(d, "floor", "over_shallow_water")
            compose.compose(up, lo, (32, 24), out)
            self.assertEqual(sorted(os.listdir(out)), [f"m{m:02d}.png" for m in range(1, 15)])
            upper, lower = Image.open(up).convert("RGBA"), Image.open(lo).convert("RGBA")
            for m in range(1, 15):
                img = Image.open(os.path.join(out, f"m{m:02d}.png")).convert("RGBA")
                self.assertEqual(img.size, (32, 24))
                self.assertTrue(all(p[3] == 255 for p in pixels(img.convert("RGBA"))))
                src = upper if m & 1 else lower
                self.assertEqual(img.convert("RGBA").getpixel((0, 0)), src.getpixel((16, 12)))

    def test_wrong_tile_size(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "t.png")
            noise_tile(32, 32, 6).save(p)
            with self.assertRaises(ValueError):
                compose.compose(p, p, (32, 24), os.path.join(d, "o"))


class BoardTest(unittest.TestCase):
    def test_board_of_a_fixture_set(self):
        with tempfile.TemporaryDirectory() as d:
            sd = os.path.join(d, "set")
            for kind, size, seed in (("floor", (32, 24), 7), ("wall", (32, 24), 8), ("shallow_water", (32, 24), 9),
                                     ("wall_face", (32, 12), 10)):
                os.makedirs(os.path.join(sd, kind))
                noise_tile(*size, seed).save(os.path.join(sd, kind, "v0.png"))
            compose.compose(os.path.join(sd, "floor"), os.path.join(sd, "shallow_water"), (32, 24),
                            os.path.join(sd, "floor", "over_shallow_water"))
            out = os.path.join(d, "out")
            board.build(sd, out)
            names = os.listdir(out)
            for n in ("index.html", "map_square.png", "map_tilted_x3.png", "pieces_floor_over_shallow_water.png"):
                self.assertIn(n, names)
            with open(os.path.join(out, "index.html"), encoding="utf-8") as f:
                self.assertIn("file:///", f.read())
            self.assertEqual(size_of(os.path.join(out, "map_square.png")), (24 * 32, 16 * 24))

    def test_vertex_mask_convention(self):
        rows = ["#.", ".."]
        at = lambda x, y: rows[y][x] if 0 <= x < 2 and 0 <= y < 2 else "."  # noqa: E731
        up = lambda c: c == "#"  # noqa: E731
        # the wall is cell (0,0): it is bit 1 at vertex (1,1), bit 2 at (0,1), bit 4 at (1,0), bit 8 at (0,0)
        self.assertEqual([board.vertex_mask(at, vx, vy, up) for vx, vy in ((1, 1), (0, 1), (1, 0), (0, 0))], [1, 2, 4, 8])

    def test_role_aware_sizes(self):
        stage, region = noise_tile(64, 48, 1), noise_tile(32, 24, 1)
        self.assertTrue(board.checks("floor", stage, "stage")[0][1])
        self.assertFalse(board.checks("floor", stage, "region")[0][1])
        self.assertFalse(board.checks("floor", region, "stage")[0][1])
        self.assertTrue(board.checks("wall_face", noise_tile(64, 24, 1), "stage")[0][1])

    def test_sample_map_shape(self):
        rows = board.load_map()
        self.assertEqual((len(rows), {len(r) for r in rows}), (16, {24}))
        self.assertEqual(set("".join(rows)), set(".#~"))


if __name__ == "__main__":
    unittest.main()
