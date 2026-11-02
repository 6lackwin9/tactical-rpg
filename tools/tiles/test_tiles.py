"""LOOK1-T tests: temp dirs and fixtures only."""
import contextlib
import functools
import io
import json
import math
import os
import random
import shutil
import subprocess
import sys
import tempfile
import unittest
import unittest.mock

from PIL import Image

import board
import compose
import cut
import flatten
import groundcheck
import masks
import seam


needs_composer = unittest.skipUnless(shutil.which("cargo"), "no cargo: ground_compose cannot run")


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
        src = noise_tile(256, 256, 23)
        for form, (size, banded) in cut.FORMS.items():
            role = "stage" if form.startswith("stage") else "ground" if form == "ground" else "region"
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
        path = self.source(noise_tile(256, 256, 29))
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
            done = subprocess.run([sys.executable, "-c", code], cwd=here, env=env, capture_output=True, text=True, timeout=120, check=True)
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


# REGIONGROUND1-T. Each ground fixture is a 256 x 256 master, what groundcheck sees in the pipeline.
GROUND = 256


def field_tile(fn, size=GROUND):
    """A grey-brown tile: 40 + fn(x, y) in each channel (fn in floats)."""
    data = bytearray()
    for y in range(size):
        for x in range(size):
            v = max(0, min(255, int(40 + fn(x, y) + 0.5)))
            data += bytes((v, v, v))
    return Image.frombytes("RGB", (size, size), bytes(data))


def grain(seed, amp=4):
    rnd = random.Random(seed)
    table = [rnd.randint(-amp, amp) for _ in range(GROUND * GROUND)]
    return lambda x, y: table[y * GROUND + x]


def sine_tile(amp=6, grain_amp=4):
    """One wrapped period of light and dark, over fine grain."""
    fine = grain(41, grain_amp)
    return field_tile(lambda x, y: amp * math.sin(2 * math.pi * x / GROUND) * math.sin(2 * math.pi * y / GROUND) + fine(x, y))


def stripes_tile():
    """Diagonal stripes of 2 art px, wrapped."""
    return field_tile(lambda x, y: 2.0 if ((x + y) // 8) % 2 else -2.0)


def streak_tile():
    """Short one-way streaks: slanted runs in even grain."""
    fine = grain(42, 2)
    rnd = random.Random(43)
    marks = set()
    for _ in range(90):
        x0, y0 = rnd.randrange(GROUND), rnd.randrange(GROUND)
        for i in range(48):
            for w in range(4):
                marks.add(((x0 + i + w) % GROUND, (y0 + i) % GROUND))
    return field_tile(lambda x, y: (3.0 if (x, y) in marks else 0.0) + fine(x, y))


def grid_tile():
    """A square grid of joints every 2 cells, 1 art px wide."""
    return field_tile(lambda x, y: -3.0 if (x // 4) % 32 == 0 or (y // 4) % 32 == 0 else 0.0)


def row_striped_tile():
    """Rows of two close tones, each 2 art px tall, over wrapped fine grain: no seam, plain bands."""
    fine = grain(44, 2)
    return field_tile(lambda x, y: (1.5 if ((y + 4) // 8) % 2 else -1.5) + fine(x, y))


def block_grain(seed, amp):
    """Grain at art px (4 master px), so it reaches the 64 x 64 form whole."""
    rnd = random.Random(seed)
    table = [rnd.randint(-amp, amp) for _ in range(64 * 64)]
    return lambda x, y: table[(y // 4) * 64 + x // 4]


def slabs_tile():
    """Quiet axis-lined slabs of 4 art px, two close tones over fine grain; the wrap falls inside a slab."""
    side, count = 16, GROUND // 16
    rnd = random.Random(60)
    tones = [rnd.choice((-3, 3)) for _ in range(count * count)]
    fine = grain(70, 4)
    return field_tile(lambda x, y: tones[((y + 8) % GROUND // side) * count + (x + 8) % GROUND // side] + fine(x, y))


def peak_with_bins_on_a_boundary(grid):
    """Direction's peak had a bin edge sat on each axis: what the centred bins replaced."""
    h, w = len(grid), len(grid[0])
    bins = [0.0] * groundcheck.ANGLE_BINS
    width = math.pi / groundcheck.ANGLE_BINS
    for y in range(h):
        for x in range(w):
            gx = (grid[y][(x + 1) % w] - grid[y][(x - 1) % w]) / 2
            gy = (grid[(y + 1) % h][x] - grid[(y - 1) % h][x]) / 2
            if gx or gy:
                bins[int((math.atan2(gy, gx) % math.pi) / width + 1e-9) % groundcheck.ANGLE_BINS] += gx * gx + gy * gy
    return max(bins) / (sum(bins) / groundcheck.ANGLE_BINS)


def dark_spots(centres, radius, depth, fine=None):
    """Hard dark disks on the wrapped tile, over even grain."""
    fine = fine or grain(45, 4)

    def inside(x, y):
        for cx, cy in centres:
            dx, dy = abs(x - cx), abs(y - cy)
            if min(dx, GROUND - dx) ** 2 + min(dy, GROUND - dy) ** 2 < radius * radius:
                return True
        return False

    return field_tile(lambda x, y: fine(x, y) - (depth if inside(x, y) else 0))


BLOTCHES = (
    ("one of radius 12, depth 10", ((90, 70),), 12, 10),
    ("one of radius 24, depth 12", ((90, 70),), 24, 12),
    ("three of radius 32, depth 6", ((40, 60), (150, 90), (90, 200)), 32, 6),
)


def even_tile(seed=45):
    return field_tile(grain(seed))


def tint_of(img, tint):
    """`img` with `tint` added to its channels."""
    out = bytearray(img.tobytes())
    for i in range(len(out)):
        out[i] = max(0, min(255, out[i] + tint[i % 3]))
    return Image.frombytes("RGB", img.size, bytes(out))


def fine_detail(img):
    """Mean step in L on the 64 x 64 form."""
    return groundcheck.busy(groundcheck.form_luma(img))


class GroundCutTest(unittest.TestCase):
    def test_ground_is_a_box_reduce_to_256(self):
        src = noise_tile(512, 512, 51)
        self.assertEqual(pixels(cut.cut_form(src, "ground")), pixels(src.resize((256, 256), Image.BOX)))

    def test_ground_takes_at_most_3_percent_off_square_and_no_upscale(self):
        self.assertEqual(cut.cut_form(noise_tile(520, 512, 52), "ground").size, (256, 256))
        with self.assertRaises(ValueError):
            cut.cut_form(noise_tile(600, 512, 53), "ground")
        with self.assertRaises(ValueError):
            cut.cut_form(noise_tile(128, 128, 54), "ground")

    def test_tall_face_is_the_whole_width_of_the_centred_band(self):
        art = noise_tile(128, 48, 55)
        src = Image.new("RGB", (512, 512), (255, 0, 255))
        src.paste(art.resize((512, 192), Image.NEAREST), (0, 160))
        self.assertEqual(pixels(cut.cut_form(src, "region_tall_face")), pixels(art))

    def test_cli_form_ground(self):
        with tempfile.TemporaryDirectory() as d:
            src, dst = os.path.join(d, "s.png"), os.path.join(d, "o.png")
            noise_tile(512, 512, 56).save(src)
            self.assertEqual(cut.main([src, "--form", "ground", "--out", dst]), 0)
            self.assertEqual(size_of(dst), (256, 256))


class GroundCheckTest(unittest.TestCase):
    def fails(self, img, cls="natural"):
        return groundcheck.failures(groundcheck.scores(img), cls)

    def test_even_grain_passes_every_check(self):
        self.assertEqual(self.fails(even_tile()), [])

    def test_quiet_grain_is_not_taken_for_a_direction(self):
        # rounded to 8 bits, this grain's gradients sit on four angles; the form stays in floats so it cannot fail for that
        for amp in (1, 2, 4):
            r = groundcheck.scores(field_tile(grain(46, amp)))
            self.assertLess(r["peak"], groundcheck.LIMITS["natural"]["peak"], amp)
            self.assertLess(r["coherence"], groundcheck.LIMITS["natural"]["coherence"], amp)

    def test_wrapped_sine_fails_broad_tone(self):
        self.assertIn("broad", self.fails(sine_tile()))

    def test_stripes_and_streaks_fail_both_direction_scores(self):
        for tile in (stripes_tile(), streak_tile()):
            self.assertLessEqual({"peak", "coherence"}, set(self.fails(tile)))

    def test_square_grid_fails_peak_but_has_no_one_direction(self):
        r = groundcheck.scores(grid_tile())
        self.assertGreater(r["peak"], groundcheck.LIMITS["natural"]["peak"])
        self.assertLess(r["coherence"], groundcheck.LIMITS["natural"]["coherence"])

    def test_quiet_axis_slabs_fail_peak_only_because_the_bins_sit_on_the_axes(self):
        flat = flatten.flatten(slabs_tile(), flatten.GAINS["natural"])
        limit = groundcheck.LIMITS["natural"]["peak"]
        self.assertEqual(self.fails(flat), ["peak"])
        self.assertLessEqual(peak_with_bins_on_a_boundary(groundcheck.form_luma(flat)), limit)

    def test_a_repeated_dark_blotch_fails_before_and_after_flatten(self):
        for name, centres, radius, depth in BLOTCHES:
            src = dark_spots(centres, radius, depth)
            self.assertIn("blotch", self.fails(src), name)
            out, _, _ = flatten.run(src, "natural")
            self.assertIn("blotch", self.fails(out), name)

    def test_a_lower_gain_does_not_hide_a_blotch(self):
        for name, centres, radius, depth in BLOTCHES:
            out = flatten.flatten(dark_spots(centres, radius, depth), flatten.GAIN_FLOORS["natural"])
            self.assertIn("blotch", self.fails(out), name)

    def test_even_grain_has_no_blotch_at_any_gain_or_art_px(self):
        limit = groundcheck.LIMITS["natural"]["blotch"]
        for fine in (grain(46, 1), grain(46, 8), block_grain(47, 1), block_grain(47, 4)):
            tile = field_tile(fine)
            self.assertLess(groundcheck.scores(tile)["blotch"], limit)
            self.assertLess(groundcheck.scores(flatten.flatten(tile, 0.55))["blotch"], limit)

    def test_art_px_of_8_master_px_fail_seam_by_construction(self):
        # every wrap step is a block's edge, while half the inner steps fall inside a block
        rnd = random.Random(48)
        table = [rnd.randint(-2, 2) for _ in range(32 * 32)]
        tile = field_tile(lambda x, y: table[(y // 8) * 32 + x // 8])
        self.assertGreater(groundcheck.scores(tile)["seam"], groundcheck.LIMITS["natural"]["seam"])
        self.assertLessEqual(groundcheck.scores(field_tile(block_grain(47, 2)))["seam"], groundcheck.LIMITS["natural"]["seam"])

    def test_a_size_off_the_form_warns_and_a_multiple_does_not(self):
        self.assertIsNone(groundcheck.size_warning(even_tile()))
        self.assertIsNone(groundcheck.size_warning(noise_tile(64, 64, 49)))
        self.assertIn("300x300", groundcheck.size_warning(noise_tile(300, 300, 49)))
        with tempfile.TemporaryDirectory() as d:
            good, off = os.path.join(d, "g.png"), os.path.join(d, "o.png")
            even_tile().save(good)
            even_tile().resize((300, 300), Image.BOX).save(off)
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()) as err:
                groundcheck.main(["score", good, "--class", "natural"])
                self.assertEqual(err.getvalue(), "")
                groundcheck.main(["score", off, "--class", "natural"])
            self.assertIn("whole multiple of 64", err.getvalue())

    def test_row_stripes_pass_seam_and_fail_banding(self):
        r = groundcheck.scores(row_striped_tile())
        self.assertLessEqual(r["seam"], groundcheck.LIMITS["natural"]["seam"])
        self.assertGreater(r["banding"], groundcheck.LIMITS["natural"]["banding"])

    def test_strong_colour_fails_the_colour_cap(self):
        self.assertEqual(self.fails(even_tile()), [])
        self.assertIn("colour", self.fails(tint_of(even_tile(), (14, 0, -14))))

    def test_high_contrast_fails_the_contrast_cap(self):
        loud = field_tile(block_grain(47, 30))
        self.assertLessEqual({"range", "busy"}, set(self.fails(loud, "natural")))
        self.assertLessEqual({"range", "busy"}, set(self.fails(loud, "built")))

    def test_built_limits_are_wider_on_contrast_only(self):
        n, b = groundcheck.LIMITS["natural"], groundcheck.LIMITS["built"]
        self.assertGreater(b["range"], n["range"])
        self.assertGreater(b["busy"], n["busy"])
        self.assertGreater(b["broad"], n["broad"])
        for c in ("seam", "blotch", "banding", "peak", "coherence", "colour", "lum"):
            self.assertEqual(b[c], n[c], c)

    def test_bright_fails_luminance(self):
        self.assertIn("lum", self.fails(tint_of(even_tile(), (40, 40, 40))))

    def test_a_figure_not_well_above_a_ground_warns(self):
        rnd = random.Random(48)
        figure = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        quiet = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        for y in range(4, 28):
            for x in range(4, 28):
                v = rnd.choice((10, 90, 170, 250))
                figure.putpixel((x, y), (v, v, v, 255))
                quiet.putpixel((x, y), (40, 40, 40, 255))
        ground = [("g", even_tile())]
        self.assertEqual(groundcheck.figure_warnings(ground, [("figure.png", figure)]), [])
        self.assertTrue(groundcheck.figure_warnings(ground, [("quiet.png", quiet)]))
        self.assertEqual(groundcheck.figure_warnings(ground, [(os.path.join("cobbles", "v0.png"), quiet)]), [])

    def test_cli_pair_figures_and_read(self):
        with tempfile.TemporaryDirectory() as d:
            for g, tint in (("a", (0, 0, 0)), ("b", (1, 1, 1))):
                os.makedirs(os.path.join(d, "ground", g))
                tint_of(even_tile(), tint).save(os.path.join(d, "ground", g, "v0.png"))
            frame = os.path.join(d, "frame.png")
            body_fixture().save(frame)
            ground_dir = os.path.join(d, "ground")
            with contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(groundcheck.main(["pair", ground_dir]), 1)   # one level apart, one grain: too alike
                self.assertEqual(groundcheck.main(["figures", ground_dir, "--art", frame]), 0)
                self.assertEqual(groundcheck.main(["read", frame, os.path.join(ground_dir, "a", "v0.png"), "--option", "rim"]), 0)
            self.assertIn("a / b: colour 1.7", out.getvalue())
            self.assertIn("TOO ALIKE", out.getvalue())
            self.assertIn("2 grounds", out.getvalue())
            self.assertIn("read_contrast", out.getvalue())

    def test_score_exit_code(self):
        with tempfile.TemporaryDirectory() as d:
            good, bad = os.path.join(d, "g.png"), os.path.join(d, "b.png")
            even_tile().save(good)
            stripes_tile().save(bad)
            with contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(groundcheck.main(["score", good, "--class", "natural"]), 0)
                self.assertEqual(groundcheck.main(["score", bad, "--class", "natural"]), 1)
            self.assertIn("Direction, peak", out.getvalue())


class FlattenTest(unittest.TestCase):
    def test_sine_passes_after_flatten_with_its_mean_detail_and_seam(self):
        src = sine_tile()
        self.assertIn("broad", groundcheck.failures(groundcheck.scores(src), "natural"))
        gain = flatten.GAINS["natural"]
        out = flatten.flatten(src, gain)
        after = groundcheck.scores(out)
        self.assertNotIn("broad", groundcheck.failures(after, "natural"))
        for a, b in zip(flatten.channel_means(src.tobytes()), flatten.channel_means(out.tobytes())):
            self.assertLessEqual(abs(a - b), 1)
        # fine detail: on the master, the mean step of the grain alone, against gain x the source's
        raw, flat = groundcheck.busy(groundcheck.luma_grid(src)), groundcheck.busy(groundcheck.luma_grid(out))
        self.assertAlmostEqual(flat / raw, gain, delta=0.08)
        self.assertLessEqual(seam.score(out), seam.score(src))

    def test_flatten_does_not_make_a_direction_pass(self):
        for tile in (stripes_tile(), streak_tile(), grid_tile()):
            before = groundcheck.failures(groundcheck.scores(tile), "natural")
            after = groundcheck.failures(groundcheck.scores(flatten.flatten(tile, 0.55)), "natural")
            self.assertIn("peak", before)
            self.assertIn("peak", after)

    def test_flatten_wraps_its_blur(self):
        # a clamped blur would leave a light or dark fringe along the tile's edges on a wrapped sine
        out = flatten.flatten(sine_tile(grain_amp=4), 0.55)
        self.assertLessEqual(seam.score(out), groundcheck.LIMITS["natural"]["seam"])
        low = flatten.low_of(field_tile(lambda x, y: 30 * math.sin(2 * math.pi * x / GROUND))).tobytes()
        self.assertLessEqual(abs(low[0] - low[(GROUND - 1) * 3]), 1)   # a clamped blur leaves the two far apart

    def test_isotropic_lowers_the_direction_scores_on_a_streak(self):
        src = streak_tile()
        before = groundcheck.scores(flatten.flatten(src, 0.55))
        after = groundcheck.scores(flatten.isotropic(flatten.flatten(src, 0.55)))
        self.assertLess(after["peak"], before["peak"])
        self.assertLess(after["coherence"], before["coherence"])

    def test_isotropic_is_never_the_default(self):
        src = even_tile()
        self.assertEqual(pixels(flatten.run(src, "natural")[0]), pixels(flatten.flatten(src, flatten.GAINS["natural"])))

    def test_built_steps_the_gain_down_until_the_contrast_cap_passes(self):
        loud = field_tile(block_grain(49, 9))
        self.assertIn("busy", groundcheck.failures(groundcheck.scores(flatten.flatten(loud, flatten.GAINS["built"])), "built"))
        self.assertEqual(flatten.GAIN_FLOORS["built"], 0.25)
        out, gain, bad = flatten.run(loud, "built")
        self.assertEqual(bad, [])
        self.assertLess(gain, flatten.GAINS["built"])
        self.assertGreaterEqual(gain, flatten.GAIN_FLOORS["built"])
        # one step more would not have been needed: the gain above it still fails
        above = groundcheck.failures(groundcheck.scores(flatten.flatten(loud, gain + flatten.GAIN_STEP)), "built")
        self.assertTrue(set(above) & set(flatten.GAIN_CHECKS))

    def test_natural_steps_the_gain_down_to_meet_the_caps(self):
        loud = field_tile(block_grain(49, 4))
        self.assertIn("busy", groundcheck.failures(groundcheck.scores(flatten.flatten(loud, flatten.GAINS["natural"])), "natural"))
        out, gain, bad = flatten.run(loud, "natural")
        self.assertEqual(bad, [])
        self.assertLess(gain, flatten.GAINS["natural"])
        self.assertGreaterEqual(gain, flatten.GAIN_FLOORS["natural"])
        self.assertEqual(groundcheck.failures(groundcheck.scores(out), "natural"), [])

    def test_natural_stops_at_its_floor_and_a_failure_records_the_gain(self):
        with tempfile.TemporaryDirectory() as d:
            src, dst, rec = os.path.join(d, "s.png"), os.path.join(d, "o.png"), os.path.join(d, "r.json")
            field_tile(block_grain(50, 25)).save(src)
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(flatten.main([src, "--out", dst, "--record", rec]), 1)
            self.assertIn("re-roll", err.getvalue())
            with open(rec, encoding="utf-8") as f:
                self.assertEqual(json.load(f)["flatten"]["gain"], flatten.GAIN_FLOORS["natural"])

    def test_a_stepped_gain_goes_in_the_record(self):
        with tempfile.TemporaryDirectory() as d:
            src, dst, rec = os.path.join(d, "s.png"), os.path.join(d, "o.png"), os.path.join(d, "r.json")
            loud = field_tile(block_grain(49, 4))
            loud.save(src)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(flatten.main([src, "--out", dst, "--record", rec]), 0)
            with open(rec, encoding="utf-8") as f:
                self.assertEqual(json.load(f)["flatten"]["gain"], flatten.run(loud, "natural")[1])

    def test_no_step_when_the_first_gain_passes(self):
        self.assertEqual(flatten.run(even_tile(), "natural")[1], flatten.GAINS["natural"])
        self.assertEqual(flatten.run(even_tile(), "built")[1], flatten.GAINS["built"])

    def test_built_gives_up_at_the_floor_and_exits_1(self):
        with tempfile.TemporaryDirectory() as d:
            src, dst = os.path.join(d, "s.png"), os.path.join(d, "o.png")
            field_tile(block_grain(50, 35)).save(src)
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(flatten.main([src, "--out", dst, "--class", "built"]), 1)
            self.assertIn("re-roll", err.getvalue())
            self.assertTrue(os.path.exists(dst))

    def test_explicit_gain_takes_no_steps_and_goes_in_the_record(self):
        with tempfile.TemporaryDirectory() as d:
            src, dst, rec = os.path.join(d, "s.png"), os.path.join(d, "o.png"), os.path.join(d, "s.json")
            even_tile().save(src)
            with open(rec, "w", encoding="utf-8") as f:
                json.dump({"id": "kept"}, f)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(flatten.main([src, "--out", dst, "--gain", "0.5", "--record", rec]), 0)
            with open(rec, encoding="utf-8") as f:
                record = json.load(f)
            self.assertEqual(record["id"], "kept")
            self.assertEqual(record["flatten"], {"class": "natural", "gain": 0.5, "isotropic": False})


def body_fixture():
    """A figure with a dark outline and a mid body on 48 x 48."""
    img = Image.new("RGBA", (48, 48), (0, 0, 0, 0))
    for y in range(8, 46):
        for x in range(14, 34):
            edge = x in (14, 33) or y in (8, 45)
            img.putpixel((x, y), (18, 18, 18, 255) if edge else (70, 60, 80, 255))
    return img


class ReadContrastTest(unittest.TestCase):
    def test_ring_and_band_are_the_figure_edge_and_the_two_px_around(self):
        solid, ring, band = groundcheck.ring_and_band(body_fixture())
        self.assertEqual(len(ring), 2 * 20 + 2 * 38 - 4)
        self.assertEqual(ring & band, set())
        self.assertLess(ring, solid)
        self.assertEqual(len(band), (24 * 42) - (20 * 38))

    def test_a_dark_figure_on_a_dark_ground_scores_low_and_on_a_light_one_high(self):
        dark = Image.new("RGB", (64, 64), (18, 18, 18))
        light = Image.new("RGB", (64, 64), (120, 120, 120))
        self.assertLess(groundcheck.read_contrast(body_fixture(), dark), groundcheck.read_contrast(body_fixture(), light))

    def test_the_rim_lifts_a_dark_figure_above_the_shadow(self):
        soot = Image.new("RGB", (64, 64), (30, 30, 35))
        shadow = groundcheck.read_contrast(body_fixture(), soot, "shadow")
        rim = groundcheck.read_contrast(body_fixture(), soot, "rim")
        self.assertGreater(rim, shadow)
        self.assertGreaterEqual(rim, 25)

    def test_golden_values_for_the_rust_check(self):
        # the README's fixture: the numbers a port must reproduce, with the order and sampling written there
        flat = Image.new("RGB", (64, 64), (30, 30, 35))
        checker_ground = Image.new("RGB", (64, 64))
        for y in range(64):
            for x in range(64):
                v = 60 if (x + y) % 2 else 20
                checker_ground.putpixel((x, y), (v, v, v))
        body = body_fixture()
        for ground, option, contrast, step in (
            (flat, "shadow", 11.476758, 11.443010),
            (flat, "pool", 16.708316, 16.432241),
            (flat, "rim", 51.270821, 53.059785),
            (checker_ground, "shadow", 20.833413, 20.790557),
            (checker_ground, "pool", 25.750239, 25.478856),
            (checker_ground, "rim", 58.338361, 60.013358),
        ):
            self.assertAlmostEqual(groundcheck.read_contrast(body, ground, option), contrast, places=5, msg=option)
            self.assertAlmostEqual(groundcheck.ring_step(body, ground, option), step, places=5, msg=option)

    def test_a_fully_transparent_figure_scores_zero(self):
        empty = Image.new("RGBA", (48, 48), (0, 0, 0, 0))
        ground = Image.new("RGB", (64, 64), (30, 30, 35))
        self.assertEqual(groundcheck.read_contrast(empty, ground), 0.0)
        self.assertEqual(groundcheck.ring_step(empty, ground, "rim"), 0.0)

    def test_soft_alpha_matches_the_engine_formula(self):
        self.assertEqual(groundcheck.soft_alpha(0, 0, 10, 5, 90), 90)
        self.assertEqual(groundcheck.soft_alpha(10, 0, 10, 5, 90), 0)
        self.assertEqual(groundcheck.soft_alpha(0, 6, 10, 5, 90), 0)
        values = [groundcheck.soft_alpha(x, 0, 10, 5, 90) for x in range(11)]
        self.assertEqual(values, sorted(values, reverse=True))

    def test_the_shadow_darkens_at_the_feet_only(self):
        ground = Image.new("RGB", (64, 64), (60, 60, 60))
        frame, _ = groundcheck.compose_frame(body_fixture(), ground, "shadow")
        feet = 45 + groundcheck.BAND - 1
        self.assertLess(frame[feet][13][0], 60)              # beside the feet
        self.assertEqual(frame[10][13][0], 60)               # beside the head


class GroundBoardTest(unittest.TestCase):
    def make_set(self, d):
        sd = os.path.join(d, "set")
        for g, tint in (("ash", (0, 0, 0)), ("flagstone", (12, 12, 12)), ("mud", (2, 1, 0))):
            os.makedirs(os.path.join(sd, "ground", g))
            tint_of(even_tile(), tint).save(os.path.join(sd, "ground", g, "v0.png"))
        root = os.path.join(d, "kit")
        for b in board.CLASS_BODIES:
            folder = os.path.join(root, "units", b, "base")
            os.makedirs(folder)
            body_fixture().save(os.path.join(folder, "sheet.png"))
            with open(os.path.join(folder, "clips.json"), "w", encoding="utf-8") as f:
                json.dump({"frame": [48, 48]}, f)
        for p in board.FIGURE_PROPS:
            folder = os.path.join(root, "props", "common", p)
            os.makedirs(folder)
            body_fixture().save(os.path.join(folder, "v0.png"))
        return sd, root

    @needs_composer
    def test_board_of_a_fixture_ground_set(self):
        with tempfile.TemporaryDirectory() as d:
            sd, root = self.make_set(d)
            out = os.path.join(d, "out")
            board.build_ground(sd, out, root)
            names = os.listdir(out)
            for g in ("ash", "flagstone", "mud"):
                for cells in ("16x10", "32x20"):
                    for scale in (1, 2, 3):
                        self.assertIn(f"repeat_{g}_{cells}_x{scale}.png", names)
            self.assertEqual(size_of(os.path.join(out, "repeat_ash_16x10_x1.png")), (256, 160))
            self.assertEqual(size_of(os.path.join(out, "repeat_ash_32x20_x3.png")), (1536, 960))
            for pair in ("ash_mud", "ash_flagstone", "mud_flagstone"):   # by the table's order, lower first
                self.assertIn(f"pair_{pair}.png", names)
            self.assertIn("figures_x2.png", names)
            with open(os.path.join(out, "index.html"), encoding="utf-8") as f:
                page = f.read()
            self.assertIn("file:///", page)
            self.assertIn("Direction, peak", page)
            self.assertIn("read_contrast", page)
            self.assertIn("FAIL: too alike", page)   # ash and mud are about 2 apart, on one grain
            self.assertIn("mix.png", names)

    def test_a_set_without_ground_art_is_an_error(self):
        with tempfile.TemporaryDirectory() as d:
            with contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(board.main([d, "--ground", "--out", os.path.join(d, "o")]), 1)
            self.assertIn("no <id>/v0.png", err.getvalue())

    def test_every_swatch_meets_the_colour_cap(self):
        for g, row in board.GROUND_ROWS.items():
            limit = groundcheck.limits_of(groundcheck.class_of(g), g)["colour"]
            self.assertLessEqual(groundcheck.colour_cap(Image.new("RGB", (4, 4), row[4])), limit, g)
        self.assertEqual(board.GROUND_ROWS["dirt_road"][4], "#3c3126")   # the decision's mock colours
        self.assertEqual(board.GROUND_ROWS["packed_dirt"][4], "#352c23")

    @needs_composer
    def test_a_master_off_the_form_warns_on_the_board(self):
        with tempfile.TemporaryDirectory() as d:
            sd, root = self.make_set(d)
            even_tile().resize((300, 300), Image.BOX).save(os.path.join(sd, "ground", "ash", "v0.png"))
            out = os.path.join(d, "out")
            with contextlib.redirect_stderr(io.StringIO()) as err:
                board.build_ground(sd, out, root)
            self.assertIn("ash", err.getvalue())
            with open(os.path.join(out, "index.html"), encoding="utf-8") as f:
                self.assertIn("whole multiple of 64", f.read())

    def test_every_pair_layout_is_on_the_map(self):
        cols, rows = board.PAIR_SIZE
        for title, cells in board.pair_layouts():
            self.assertTrue(cells, title)
            self.assertTrue(all(0 <= x < cols and 0 <= y < rows for x, y in cells), title)


@functools.lru_cache(maxsize=None)
def stones_tile(cells=24, joint=4, depth=6.0, seed=60):
    """Irregular stones with narrow dark joints over quiet grain, wrapped: Voronoi cells of `cells` random points."""
    rnd = random.Random(seed)
    points = [(rnd.randrange(GROUND), rnd.randrange(GROUND)) for _ in range(cells)]
    fine = grain(seed + 1, 1)

    def wrapped(x, y, p):
        dx, dy = abs(x - p[0]), abs(y - p[1])
        return math.hypot(min(dx, GROUND - dx), min(dy, GROUND - dy))

    joints = {}
    for y in range(GROUND):
        for x in range(GROUND):
            near = sorted(wrapped(x, y, p) for p in points)[:2]
            joints[(x, y)] = near[1] - near[0] < joint
    return field_tile(lambda x, y: fine(x, y) - (depth if joints[(x, y)] else 0))


def blocks_tile(block=16, amp=3, seed=61):
    """The same mean and quiet, with shapes of `block` px, wrapped."""
    rnd = random.Random(seed)
    n = GROUND // block
    table = [rnd.randint(-amp, amp) for _ in range(n * n)]
    return field_tile(lambda x, y: table[(y // block) * n + x // block])


def layout_grounds(name):
    _, _, grid = board.parse_layout(os.path.join(board.HERE, "fixtures", name))
    return sorted({g for row in grid for g in row})


class ColourSetTest(unittest.TestCase):
    def test_every_table_colour_is_reached_within_half_a_level_whatever_the_art_tint(self):
        for tint in ((0, 0, 0), (30, -10, 20)):
            src = tint_of(even_tile(), tint)
            for g, row in board.GROUND_ROWS.items():
                target = flatten.parse_colour(row[4])
                for a, b in zip(flatten.channel_means(flatten.set_mean(src, target).tobytes()), target):
                    self.assertLessEqual(abs(a - b), 0.5, (g, tint))
            target = flatten.parse_colour(board.row_of("dirt_road")[4])
            for a, b in zip(flatten.channel_means(flatten.run(src, "natural", colour=target)[0].tobytes()), target):
                self.assertLessEqual(abs(a - b), 0.5, tint)

    def test_the_art_tint_no_longer_decides_hue_or_value(self):
        target = flatten.parse_colour("#3c3126")
        a = flatten.flatten(even_tile(), 0.45, target)
        b = flatten.flatten(tint_of(even_tile(), (20, -5, 11)), 0.45, target)
        self.assertEqual(flatten.channel_means(a.tobytes()), flatten.channel_means(b.tobytes()))
        self.assertEqual(a.tobytes(), b.tobytes())

    def test_the_pattern_and_the_seam_stay(self):
        src = even_tile()
        out = flatten.set_mean(src, (70, 60, 50))
        first = out.tobytes()[0:3]
        base = src.tobytes()
        shift = [first[c] - base[c] for c in range(3)]
        raw = out.tobytes()
        self.assertTrue(all(raw[i] - base[i] == shift[i % 3] for i in range(len(raw))))
        self.assertEqual(seam.score(out), seam.score(src))
        flat, coloured = flatten.flatten(src, 0.5), flatten.flatten(src, 0.5, (28, 25, 23))
        self.assertAlmostEqual(groundcheck.busy(groundcheck.luma_grid(flat)), groundcheck.busy(groundcheck.luma_grid(coloured)), delta=0.05)
        self.assertLessEqual(seam.score(coloured), groundcheck.LIMITS["natural"]["seam"])

    def test_clipped_pixels_are_made_up_by_another_shift(self):
        src = field_tile(lambda x, y: -35 if (x // 8 + y // 8) % 2 else 25)   # levels 5 and 65
        out = flatten.set_mean(src, (12, 12, 12))    # a plain shift by -23 clips the low half at 0 and lands on 17-ish
        self.assertLessEqual(abs(flatten.channel_means(out.tobytes())[0] - 12), 0.5)

    def test_a_mean_no_shift_can_reach_stops_at_the_nearest(self):
        out = flatten.set_mean(Image.new("RGB", (8, 8), (255, 0, 9)), (0, 255, 9))
        self.assertEqual(out.getpixel((0, 0)), (0, 255, 9))

    def test_deterministic(self):
        src = even_tile()
        a = flatten.run(src, "natural", colour=(42, 38, 34))[0]
        b = flatten.run(src, "natural", colour=(42, 38, 34))[0]
        self.assertEqual(a.tobytes(), b.tobytes())

    def test_a_gain_step_scores_the_coloured_image(self):
        # broad tone is relative to the mean: art that passes as it is can fail once it is set to a dark colour
        fine = grain(62)
        src = field_tile(lambda x, y: 6 * math.sin(2 * math.pi * x / GROUND) + fine(x, y) + 40)
        _, plain, _ = flatten.run(src, "natural")
        _, dark, _ = flatten.run(src, "natural", colour=(14, 16, 24))
        self.assertLess(dark, plain)

    def test_a_run_makes_up_what_flatten_clips_at_a_dark_colour(self):
        target = (6, 6, 6)
        src = field_tile(grain(65, 20))
        self.assertGreater(flatten.channel_means(flatten.flatten(src, 1.0, target).tobytes())[0], 6.5)
        out = flatten.run(src, "natural", gain=1.0, colour=target)[0]
        self.assertLessEqual(abs(flatten.channel_means(out.tobytes())[0] - 6), 0.5)

    def test_cli_sets_the_colour_and_writes_it_to_the_record(self):
        with tempfile.TemporaryDirectory() as d:
            src, out, rec = os.path.join(d, "s.png"), os.path.join(d, "o.png"), os.path.join(d, "r.json")
            even_tile().save(src)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(flatten.main([src, "--out", out, "--colour", "#3c3126", "--record", rec]), 0)
            with Image.open(out) as im:
                for a, b in zip(flatten.channel_means(im.convert("RGB").tobytes()), (0x3c, 0x31, 0x26)):
                    self.assertLessEqual(abs(a - b), 0.5)
            with open(rec, encoding="utf-8") as f:
                self.assertEqual(json.load(f)["flatten"]["colour"], "#3c3126")

    def test_cli_refuses_a_bad_colour_and_a_huge_source(self):
        with tempfile.TemporaryDirectory() as d:
            src, out = os.path.join(d, "s.png"), os.path.join(d, "o.png")
            even_tile().save(src)
            for bad in ("zzzzzz", "12345", "1234567", "", "#12345g", "-12345"):
                with contextlib.redirect_stderr(io.StringIO()) as err:
                    self.assertEqual(flatten.main([src, "--out", out, f"--colour={bad}"]), 1, bad)
                self.assertIn("not RRGGBB", err.getvalue())
                self.assertFalse(os.path.exists(out))
            huge = os.path.join(d, "h.png")
            Image.new("RGB", (flatten.MAX_SOURCE + 1, 2)).save(huge)
            with contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(flatten.main([huge, "--out", out]), 1)
            self.assertIn("px a side", err.getvalue())


class NamedColourCapTest(unittest.TestCase):
    def brown(self, colour):
        return flatten.set_mean(even_tile(), flatten.parse_colour(colour))

    def test_the_dirts_pass_on_their_colours_and_others_do_not_pass_on_them(self):
        for g, colour in (("dirt_road", "#3c3126"), ("packed_dirt", "#352c23")):
            r = groundcheck.scores(self.brown(colour))
            self.assertNotIn("colour", groundcheck.failures(r, "natural", g))
            self.assertIn("colour", groundcheck.failures(r, "natural"))
            self.assertIn("colour", groundcheck.failures(r, "natural", "ash"))

    def test_a_named_cap_still_catches_too_strong_a_colour(self):
        strong = tint_of(self.brown("#3c3126"), (10, 0, -10))
        self.assertIn("colour", groundcheck.failures(groundcheck.scores(strong), "natural", "dirt_road"))

    def test_every_table_colour_on_grain_with_colour_noise_meets_its_cap(self):
        rnd = random.Random(64)
        noisy = Image.frombytes("RGB", (64, 64), bytes(40 + rnd.randint(-1, 1) for _ in range(64 * 64 * 3)))
        for g, row in board.GROUND_ROWS.items():
            r = groundcheck.colour_cap(flatten.set_mean(noisy, flatten.parse_colour(row[4])))
            self.assertLessEqual(r, groundcheck.limits_of(groundcheck.class_of(g), g)["colour"], g)

    def test_only_the_mock_colours_past_the_common_cap_are_named(self):
        self.assertEqual(set(groundcheck.COLOUR_CAPS), {"dirt_road", "packed_dirt", "tallow"})
        self.assertEqual(groundcheck.limits_of("natural", "ash"), groundcheck.LIMITS["natural"])

    def test_score_takes_the_ground(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "g.png")
            self.brown("#3c3126").save(path)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(groundcheck.main(["score", path, "--class", "natural", "--ground", "dirt_road"]), 0)
                self.assertEqual(groundcheck.main(["score", path, "--class", "natural"]), 1)


class PairSeparationTest(unittest.TestCase):
    def test_two_rolls_of_one_grain_a_little_apart_fail(self):
        a, b = even_tile(45), tint_of(even_tile(46), (1, 1, 1))
        r = groundcheck.pair_scores(a, b)
        self.assertLess(r["pattern"], groundcheck.MIN_PATTERN)
        self.assertFalse(r["clear"])
        self.assertTrue(groundcheck.pair_failures([("a", a), ("b", b)]))

    def test_the_mocks_colours_on_one_grain_pass_on_colour_alone(self):
        for name in ("mix_outdoor.txt", "mix_street.txt"):
            grounds = layout_grounds(name)
            tiles = [(g, flatten.set_mean(even_tile(), flatten.parse_colour(board.row_of(g)[4]))) for g in grounds]
            for a, b, r in groundcheck.pair_report(tiles):
                self.assertLess(r["pattern"], groundcheck.MIN_PATTERN, (a, b))
                self.assertGreaterEqual(r["colour"], groundcheck.MIN_PAIR_DISTANCE, (a, b))
                self.assertTrue(r["clear"], (a, b))

    def test_one_colour_passes_on_a_clearly_different_pattern_of_each_kind(self):
        even = even_tile()
        for name, other, score in (("scale", blocks_tile(), "scale"), ("busy", field_tile(grain(63, 1)), "busy"),
                                   ("direction", streak_tile(), "direction")):
            other = flatten.set_mean(other, tuple(round(v) for v in groundcheck.mean_rgb(even)))
            r = groundcheck.pair_scores(even, other)
            self.assertLess(r["colour"], groundcheck.MIN_PAIR_DISTANCE, name)
            self.assertGreaterEqual(r[score], groundcheck.MIN_PATTERN, name)
            self.assertTrue(r["clear"], name)

    def test_contrast_does_not_move_the_scale_of_shapes(self):
        loud, quiet = blocks_tile(amp=8), blocks_tile(amp=2)
        self.assertAlmostEqual(groundcheck.scale_of(groundcheck.form_luma(loud)), groundcheck.scale_of(groundcheck.form_luma(quiet)), delta=0.2)

    def test_a_flat_image_has_no_pattern_and_divides_by_nothing(self):
        flat = Image.new("RGB", (64, 64), (40, 40, 40))
        r = groundcheck.pair_scores(flat, flat)
        self.assertEqual((r["colour"], r["pattern"], r["clear"]), (0.0, 0.0, False))
        self.assertTrue(groundcheck.pair_scores(flat, even_tile())["clear"])

    def test_cli_exit_code(self):
        with tempfile.TemporaryDirectory() as d:
            for g, tile in (("a", even_tile(45)), ("b", flatten.set_mean(even_tile(46), (60, 49, 38)))):
                os.makedirs(os.path.join(d, g))
                tile.save(os.path.join(d, g, "v0.png"))
            with contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(groundcheck.main(["pair", d]), 0)
            self.assertIn("0 too alike", out.getvalue())


class StonesTest(unittest.TestCase):
    def test_stones_with_joints_count_and_even_grain_does_not(self):
        self.assertGreaterEqual(groundcheck.stones(stones_tile()), groundcheck.MIN_STONES)
        for seed in (45, 46, 47):
            self.assertLess(groundcheck.stones(even_tile(seed)), groundcheck.MIN_STONES)
        self.assertEqual(groundcheck.stones(Image.new("RGB", (64, 64), (40, 40, 40))), 0)

    def test_a_quiet_flattened_road_still_counts(self):
        out = flatten.flatten(stones_tile(), flatten.GAIN_FLOORS["built"], (62, 63, 66))
        self.assertGreaterEqual(groundcheck.stones(out), groundcheck.MIN_STONES)

    def test_only_stone_grounds_warn(self):
        plain = even_tile()
        for g in ("paved_road", "flagstone", "cobble_road"):
            self.assertIn("no joint structure", groundcheck.stone_warning(plain, g))
            self.assertIsNone(groundcheck.stone_warning(stones_tile(), g))
        for g in ("ash", "dirt_road", "iron_catwalk", None):
            self.assertIsNone(groundcheck.stone_warning(plain, g))

    def test_the_warning_reaches_score_and_pair_and_does_not_fail(self):
        with tempfile.TemporaryDirectory() as d:
            for g, tile in (("paved_road", even_tile(45)), ("ash", flatten.set_mean(even_tile(46), (10, 80, 10)))):
                os.makedirs(os.path.join(d, g))
                tile.save(os.path.join(d, g, "v0.png"))
            path = os.path.join(d, "paved_road", "v0.png")
            with contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(groundcheck.main(["score", path, "--class", "built", "--ground", "paved_road"]), 0)
                self.assertEqual(groundcheck.main(["pair", d]), 0)
            self.assertEqual(out.getvalue().count("no joint structure"), 2)


class MixBoardTest(unittest.TestCase):
    def layout(self, d, text):
        path = os.path.join(d, "l.txt")
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        return path

    def make_set(self, d, grounds):
        sd = os.path.join(d, "set")
        for g, tile in grounds.items():
            os.makedirs(os.path.join(sd, "ground", g))
            tile.save(os.path.join(sd, "ground", g, "v0.png"))
        return sd

    def test_the_fixtures_hold_the_spec_rects(self):
        cols, rows, grid = board.parse_layout(board.MIX_LAYOUT)
        self.assertEqual((cols, rows), (40, 22))
        for x, y, g in ((0, 0, "ash"), (4, 2, "flagstone"), (13, 8, "flagstone"), (5, 13, "water"), (3, 11, "mud"), (19, 15, "dirt_road"),
                        (24, 11, "packed_dirt"), (27, 3, "scorched"), (39, 21, "scrub"), (17, 3, "paved_road")):
            self.assertEqual(grid[y][x], g, (x, y))
        cols, rows, grid = board.parse_layout(os.path.join(board.HERE, "fixtures", "mix_street.txt"))
        self.assertEqual((cols, rows), (40, 22))
        for x, y, g in ((0, 0, "soot_yard"), (0, 10, "cobble_road"), (20, 17, "iron_catwalk"), (0, 17, "water"), (0, 14, "moss"), (30, 4, "tallow")):
            self.assertEqual(grid[y][x], g, (x, y))

    def test_a_bad_layout_is_refused_not_cut_to_fit(self):
        bad = {
            "size 2 2": "needs a size and a default",
            "default ash": "needs a size and a default",
            "size 0 2\ndefault ash": "off 1..",
            f"size {board.MAX_MIX_CELLS + 1} 2\ndefault ash": "off 1..",
            "size 2 2\ndefault nope": "not in the table",
            "size 2 2\ndefault ash\nnope 0-0 0-0": "not in the table",
            "size 2 2\ndefault ash\nmud 0-2 0-0": "off the",
            "size 2 2\ndefault ash\nmud 1-0 0-0": "off the",
            "size 2 2\ndefault ash\nmud 0-0": "not `size",
            "size 2 2\ndefault ash\nmud a-b 0-0": "invalid literal",
            "size 2 2\nsize 3 3\ndefault ash": "a second size",
            "size 2 2\ndefault ash\ndefault mud": "a second default",
        }
        with tempfile.TemporaryDirectory() as d:
            for text, message in bad.items():
                with self.assertRaises(ValueError) as caught:
                    board.parse_layout(self.layout(d, text))
                self.assertIn(message, str(caught.exception), text)
            path = self.layout(d, "size 2 2\ndefault ash # a comment\nmud 1-1 1-1\n")
            self.assertEqual(board.parse_layout(path)[2], [["ash", "ash"], ["ash", "mud"]])

    @needs_composer
    def test_the_picture_size_and_no_label_on_the_map(self):
        with tempfile.TemporaryDirectory() as d:
            ash, mud = (0x2a, 0x26, 0x22), (0x23, 0x1d, 0x18)
            sd = self.make_set(d, {"ash": Image.new("RGB", (256, 256), ash), "mud": Image.new("RGB", (256, 256), mud)})
            layout = self.layout(d, "size 10 6\ndefault ash\nmud 3-6 2-3\n")
            path, _ = board.build_mix(sd, os.path.join(d, "o"), layout)
            with Image.open(path) as im:
                sheet = im.convert("RGB")
            map_w, map_h = 10 * board.CELL_PX * board.MIX_SCALE, 6 * board.CELL_PX * board.MIX_SCALE
            self.assertEqual(sheet.size, (map_w + board.MIX_PANEL, map_h))
            # flat grounds under a tone of a few %: nothing bright, as a label would be
            top = max(max(sheet.getpixel((x, y))) for y in range(map_h) for x in range(map_w))
            self.assertLess(top, 60)
            panel = max(max(sheet.getpixel((x, y))) for y in range(map_h) for x in range(map_w, sheet.size[0]))
            self.assertGreater(panel, 150)

    @needs_composer
    def test_set_colours_moves_the_art_to_the_table_and_off_it_leaves_the_art(self):
        with tempfile.TemporaryDirectory() as d:
            sd = self.make_set(d, {"ash": even_tile(45), "mud": even_tile(46)})
            target = flatten.parse_colour(board.row_of("mud")[4])
            raw = dict(board.load_masters(sd))["mud"]
            set_ = dict(board.load_masters(sd, True))["mud"]
            self.assertGreater(groundcheck.pair_distance(raw, Image.new("RGB", (8, 8), target)), 10)
            self.assertLess(groundcheck.pair_distance(set_, Image.new("RGB", (8, 8), target)), 1)
            layout = self.layout(d, "size 8 6\ndefault ash\nmud 2-5 2-3\n")
            _, before = board.build_mix(sd, os.path.join(d, "a"), layout)
            _, after = board.build_mix(sd, os.path.join(d, "b"), layout, True)
            self.assertTrue(any("too alike: ash / mud" in w for w in before))
            self.assertFalse(any("too alike" in w for w in after))

    @needs_composer
    def test_the_mix_is_deterministic(self):
        with tempfile.TemporaryDirectory() as d:
            sd = self.make_set(d, {"ash": even_tile(45), "mud": even_tile(46)})
            layout = self.layout(d, "size 8 6\ndefault ash\nmud 2-5 2-3\n")
            a, _ = board.build_mix(sd, os.path.join(d, "a"), layout, True)
            b, _ = board.build_mix(sd, os.path.join(d, "b"), layout, True)
            with Image.open(a) as ia, Image.open(b) as ib:
                self.assertEqual(ia.tobytes(), ib.tobytes())

    @needs_composer
    def test_a_ground_off_the_layout_and_a_plain_road_are_named(self):
        with tempfile.TemporaryDirectory() as d:
            sd = self.make_set(d, {"ash": even_tile(45), "paved_road": flatten.set_mean(even_tile(46), (62, 63, 66)), "moss": even_tile(47)})
            layout = self.layout(d, "size 8 6\ndefault ash\npaved_road 2-5 2-3\n")
            _, warnings = board.build_mix(sd, os.path.join(d, "o"), layout)
            self.assertIn("moss: not on this layout", warnings)
            self.assertTrue(any(w.startswith("paved_road:") and "no joints" in w for w in warnings))

    @needs_composer
    def test_cli(self):
        with tempfile.TemporaryDirectory() as d:
            sd = self.make_set(d, {"ash": even_tile(45), "mud": even_tile(46)})
            layout = self.layout(d, "size 8 6\ndefault ash\nmud 2-5 2-3\n")
            out = os.path.join(d, "o")
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(board.main([sd, "--mix", "--out", out, "--layout", layout, "--set-colours"]), 0)
            self.assertEqual(os.listdir(out), ["mix.png"])
            with contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(board.main([sd, "--mix", "--out", out, "--layout", os.path.join(d, "none.txt")]), 1)
            self.assertIn("none.txt", err.getvalue())
            big = os.path.join(d, "big")
            os.makedirs(os.path.join(big, "ground", "ash"))
            Image.new("RGB", (board.MAX_MASTER + 1, 1)).save(os.path.join(big, "ground", "ash", "v0.png"))
            with contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(board.main([big, "--mix", "--out", out]), 1)
            self.assertIn("px a side", err.getvalue())


def flat_rows(**changes):
    """The table for a picture of exact colours: no tone, no patches, unless asked."""
    fields = {"tone": 3, "patches": 5, "strays": 2}
    rows = {}
    for g, row in board.GROUND_ROWS.items():
        row = list(row)
        row[3], row[5] = 0.0, 0.0
        for name, value in changes.items():
            row[fields[name]] = value
        rows[g] = tuple(row)
    return unittest.mock.patch.dict(board.GROUND_ROWS, rows)


def colour_of(ground):
    return flatten.parse_colour(board.row_of(ground)[4])


def times(colour, factor):
    return tuple(v * factor for v in colour)


def near(c, wanted):
    """`c` within one level per channel of a colour in `wanted`: the composer's rounding is its own."""
    return any(all(abs(a - b) <= 1 for a, b in zip(c, w)) for w in wanted)


def map_of(grid, tex_paths=None, wear=1.0):
    with tempfile.TemporaryDirectory() as work:
        return board.draw_ground(grid, tex_paths or {}, work, wear)


def yard(lower="ash", upper="packed_dirt", cols=20, rows=12):
    """`upper` on the right half."""
    return [[upper if x >= cols // 2 else lower for x in range(cols)] for _ in range(rows)]


class GroundJobTest(unittest.TestCase):
    GRID = [["paved_road", "ash", "water"], ["mud", "ash", "water"]]

    def test_the_legend_lists_the_maps_grounds_in_the_games_order(self):
        job = board.ground_job(self.GRID, {})
        order = ["water", "ash", "mud", "paved_road"]             # by the table's order, not by where the map first has them
        self.assertEqual(board.layer_ids(self.GRID), order)
        chars = list(job["legend"])
        self.assertEqual(chars, sorted(chars))                    # a reader that sorts keys sees the same list
        self.assertEqual(job["cells"], [chars[3] + chars[1] + chars[0], chars[2] + chars[1] + chars[0]])
        self.assertEqual([job["legend"][c]["colour"] for c in chars], [list(colour_of(g)) for g in order])

    def test_a_tie_in_order_goes_by_id_as_the_kits_plan_does(self):
        self.assertEqual(board.layer_ids([["zz", "aa", "ash"]]), ["ash", "aa", "zz"])

    def test_every_field_of_a_row_reaches_the_job(self):
        row = (40, 0.07, True, 0.09, "#102030", 0.11)
        with unittest.mock.patch.dict(board.GROUND_ROWS, {"mud": row}):
            layer = board.ground_job([["mud"]], {"mud": "a/b.png"}, 2.0)["legend"]["0"]
        self.assertEqual(layer["colour"], [0x10, 0x20, 0x30])
        self.assertAlmostEqual(layer["wear"], 0.14)
        self.assertEqual((layer["strays"], layer["tone"], layer["patches"]), (True, 0.09, 0.11))
        self.assertEqual(layer["tex"], os.path.abspath("a/b.png").replace(os.sep, "/"))

    def test_a_ground_without_art_has_no_texture(self):
        self.assertNotIn("tex", board.ground_job([["mud"]], {})["legend"]["0"])

    def test_the_cell_is_the_games_and_the_strongest_wear_stays_in_the_engines_limit(self):
        self.assertEqual(board.ground_job([["mud"]], {})["cell"], [16, 16])
        strongest = max(row[1] for row in board.GROUND_ROWS.values())
        self.assertLessEqual(strongest * board.WEAR_MAX, 0.3)

    def test_a_wear_off_its_range_or_too_many_grounds_are_refused(self):
        for wear in (-0.1, board.WEAR_MAX + 0.1, float("nan"), float("inf")):
            with self.assertRaises(ValueError) as caught:
                board.ground_job([["mud"]], {}, wear)
            self.assertIn("off 0..", str(caught.exception))
        many = [[f"g{i}" for i in range(len(board.LEGEND_CHARS) + 1)]]
        with self.assertRaises(ValueError) as caught:
            board.ground_job(many, {})
        self.assertIn("at most", str(caught.exception))


class ComposerRunTest(unittest.TestCase):
    def tearDown(self):
        board._composer = None

    def test_a_linked_worktree_builds_into_its_own_folder_as_fc_sh_does(self):
        with tempfile.TemporaryDirectory() as d:
            repo = os.path.join(d, "lane9")
            os.makedirs(os.path.join(repo, ".cargo"))
            with open(os.path.join(repo, ".cargo", "config.toml"), "w", encoding="utf-8") as f:
                f.write('[build]\ntarget-dir = "C:/t/aegis"\n')
            clean = {k: v for k, v in os.environ.items() if k != "CARGO_TARGET_DIR"}
            with unittest.mock.patch.object(board, "REPO", repo), unittest.mock.patch.dict(os.environ, clean, clear=True):
                self.assertNotIn("CARGO_TARGET_DIR", board.cargo_env())       # a main checkout, .git a folder or absent
                with open(os.path.join(repo, ".git"), "w", encoding="utf-8") as f:
                    f.write("gitdir: x\n")
                self.assertEqual(board.cargo_env()["CARGO_TARGET_DIR"], "C:/t/aegis-lane9")
                with unittest.mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "mine"}):
                    self.assertEqual(board.cargo_env()["CARGO_TARGET_DIR"], "mine")

    def test_a_failed_or_stuck_tool_is_an_error_with_its_message(self):
        failed = subprocess.CompletedProcess([], 1, "", "ground_compose: placed patch refused")
        with unittest.mock.patch("subprocess.run", return_value=failed):
            with self.assertRaises(ValueError) as caught:
                board.run_tool(["x"], "ground_compose")
        self.assertIn("placed patch refused", str(caught.exception))
        with unittest.mock.patch("subprocess.run", side_effect=subprocess.TimeoutExpired("x", 1)):
            with self.assertRaises(ValueError) as caught:
                board.run_tool(["x"], "ground_compose")
        self.assertIn("no end after", str(caught.exception))

    def test_no_cargo_is_an_error_not_a_crash(self):
        board._composer = None
        with tempfile.TemporaryDirectory() as d:
            sd = os.path.join(d, "set", "ground", "ash")
            os.makedirs(sd)
            even_tile().save(os.path.join(sd, "v0.png"))
            with unittest.mock.patch("subprocess.run", side_effect=FileNotFoundError("cargo")), contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(board.main([os.path.join(d, "set"), "--mix", "--out", os.path.join(d, "o")]), 1)
            self.assertIn("error:", err.getvalue())

    def test_cargo_names_the_composer_once_a_run(self):
        built = json.dumps({"reason": "compiler-artifact", "target": {"name": "ground_compose"}, "executable": "C:/t/ground_compose.exe"})
        other = json.dumps({"reason": "compiler-artifact", "target": {"name": "aegis_ui"}, "executable": None})
        with unittest.mock.patch.object(board, "run_tool", return_value=f"{other}\n{built}\n") as run:
            self.assertEqual(board.composer(), "C:/t/ground_compose.exe")
            self.assertEqual(board.composer(), "C:/t/ground_compose.exe")
        self.assertEqual(run.call_count, 1)
        board._composer = None
        with unittest.mock.patch.object(board, "run_tool", return_value=other):
            with self.assertRaises(ValueError):
                board.composer()

    def test_a_blocked_binary_is_relinked_and_any_other_failure_is_not(self):
        blocked = OSError("blocked")
        blocked.winerror = 4551
        with tempfile.TemporaryDirectory() as d:
            exe = os.path.join(d, "ground_compose.exe")
            open(exe, "w", encoding="utf-8").close()
            with unittest.mock.patch.object(board, "composer", return_value=exe), \
                    unittest.mock.patch.object(board, "run_tool", side_effect=[blocked, "done"]) as run:
                self.assertEqual(board.compose_file("j", "o"), "done")
            self.assertEqual(run.call_count, 2)
            self.assertFalse(os.path.exists(exe))
            open(exe, "w", encoding="utf-8").close()
            with unittest.mock.patch.object(board, "composer", return_value=exe), \
                    unittest.mock.patch.object(board, "run_tool", side_effect=OSError("other")):
                with self.assertRaises(OSError):
                    board.compose_file("j", "o")
            self.assertTrue(os.path.exists(exe))


@needs_composer
class ComposedMixTest(unittest.TestCase):
    def test_the_boards_map_is_the_composers_on_a_job_in_the_games_order(self):
        with tempfile.TemporaryDirectory() as d:
            sd = os.path.join(d, "set")
            tiles = {"ash": even_tile(45), "mud": even_tile(46)}
            paths = {}
            for g, tile in tiles.items():
                os.makedirs(os.path.join(sd, "ground", g))
                paths[g] = os.path.join(sd, "ground", g, "v0.png").replace(os.sep, "/")
                tile.save(paths[g])
            layout = os.path.join(d, "l.txt")
            with open(layout, "w", encoding="utf-8") as f:
                f.write("size 10 6\ndefault ash\nmud 3-6 2-3\n")
            sheet_path, _ = board.build_mix(sd, os.path.join(d, "o"), layout)
            with Image.open(sheet_path) as im:
                w, h = 10 * board.CELL_PX, 6 * board.CELL_PX
                the_map = im.convert("RGB").crop((0, 0, w * board.MIX_SCALE, h * board.MIX_SCALE))

            def by_hand(names):
                layers = {}
                for char, g in names:
                    _, wear, strays, tone, _, patches = board.row_of(g)
                    layers[char] = {"tex": paths[g], "wear": wear, "strays": strays, "tone": tone, "patches": patches}
                cells = ["aaaaaaaaaa"] * 2 + ["aaammmmaaa"] * 2 + ["aaaaaaaaaa"] * 2
                job_path = os.path.join(d, "job.json")
                with open(job_path, "w", encoding="utf-8") as f:
                    json.dump({"seed": board.SEED, "cell": [16, 16], "cells": cells, "legend": layers}, f)
                out = os.path.join(d, "by_hand.png")
                done = subprocess.run([board.composer(), job_path, out], capture_output=True, check=True)
                self.assertEqual(done.returncode, 0)
                with Image.open(out) as im:
                    return im.convert("RGB")

            direct = by_hand((("a", "ash"), ("m", "mud")))
            self.assertEqual(the_map.resize((w, h), Image.NEAREST).tobytes(), direct.tobytes())
            # the legend's order is part of the picture: a texture's offset comes from its place
            other = by_hand((("m", "mud"), ("a", "ash")))
            self.assertNotEqual(direct.tobytes(), other.tobytes())

    def test_each_cell_shows_its_own_ground_and_only_grounds_are_on_the_map(self):
        grid = [["ash"] * 8 for _ in range(6)]
        for y in range(2, 4):
            for x in range(1, 3):
                grid[y][x] = "flagstone"
            for x in range(5, 7):
                grid[y][x] = "dirt_road"
        c = {g: colour_of(g) for g in ("ash", "flagstone", "dirt_road")}
        with flat_rows():
            img = map_of(grid)
            wear_road, wear_flag = board.row_of("dirt_road")[1], board.row_of("flagstone")[1]
        px = img.load()
        self.assertEqual(img.size, (8 * board.CELL_PX, 6 * board.CELL_PX))
        for y, row in enumerate(grid):
            for x, g in enumerate(row):
                self.assertTrue(near(px[x * board.CELL_PX + 8, y * board.CELL_PX + 8], [c[g]]), (x, y))
        allowed = []
        for g, colour in c.items():
            allowed += [colour, times(colour, 0.95), times(colour, 0.92)]      # the ground, a bite in it, a stray of it
            allowed += [times(colour, 1 - wear_road), times(colour, 1 - wear_flag)]
        seen = {px[x, y] for y in range(img.size[1]) for x in range(img.size[0])}
        self.assertEqual([s for s in seen if not near(s, allowed)], [])
        strays = [s for s in seen if near(s, [times(c["flagstone"], 0.92)])]
        self.assertFalse(strays)                                               # the table has strays off (the approved scene); the engine pins cover them on

    def test_a_ground_off_the_built_table_throws_no_strays(self):
        with flat_rows():
            img = map_of(yard("ash", "dirt_road"))
        road = times(colour_of("dirt_road"), 0.92)
        seen = {img.getpixel((x, y)) for y in range(img.size[1]) for x in range(img.size[0])}
        self.assertFalse([s for s in seen if near(s, [road]) and not near(s, [colour_of("dirt_road")])])

    def test_a_ground_without_art_draws_its_rows_colour(self):
        grid = [["ash"] * 6 for _ in range(6)]
        grid[3][3] = grid[3][2] = grid[2][3] = grid[2][2] = "mud"
        with flat_rows():
            img = map_of(grid)
        self.assertTrue(near(img.getpixel((8, 8)), [colour_of("ash")]))
        self.assertTrue(near(img.getpixel((3 * board.CELL_PX, 3 * board.CELL_PX)), [colour_of("mud")]))

    def test_wear_scales_the_darkening_of_the_lower_ground_only(self):
        # a bright lower ground, so a level is small against the wear; a bite is the lower ground x .95 at every scale
        rows = {"ash": (10, 0.03, False, 0.0, "#c8c8c8", 0.0), "packed_dirt": (50, 0.05, False, 0.0, "#402010", 0.0)}
        lower = (200, 200, 200)
        with unittest.mock.patch.dict(board.GROUND_ROWS, rows):
            for scale in (0.0, 2.0, 3.0):
                img = map_of(yard(), wear=scale)
                greys = {c[0] for c in img.getdata() if c[0] == c[1] == c[2]}
                self.assertAlmostEqual(min(greys), 200 * min(0.95, 1 - 0.05 * scale), delta=1, msg=str(scale))
                self.assertEqual(max(greys), lower[0])

    def test_the_tables_patches_reach_the_picture(self):
        grid = [["ash"] * 30 for _ in range(20)]
        with flat_rows():
            img = map_of(grid)
        self.assertEqual({img.getpixel((x, y)) for y in range(0, img.size[1], 3) for x in range(0, img.size[0], 3)}, {colour_of("ash")})
        with flat_rows(patches=0.2):
            img = map_of(grid)
        self.assertTrue(any(img.getpixel((x, y))[0] < colour_of("ash")[0] - 1 for y in range(img.size[1]) for x in range(img.size[0])))

    def test_a_pair_holds_its_two_grounds_and_its_edge_wanders_within_the_rule(self):
        cells = dict(board.pair_layouts())["straight edge"]
        with flat_rows(), tempfile.TemporaryDirectory() as work:
            sheet = board.pair_sheet("ash", "packed_dirt", {}, [set(cells)], work)
        w, h = board.PAIR_SIZE[0] * board.CELL_PX, board.PAIR_SIZE[1] * board.CELL_PX
        self.assertEqual(sheet.size, (w, h))
        upper = colour_of("packed_dirt")
        boundary = min(x for x, _ in cells) * board.CELL_PX
        off = 0
        for y in range(h):
            first = next(x for x in range(w) if near(sheet.getpixel((x, y)), [upper]))
            self.assertLessEqual(abs(first - boundary), 6)
            off += first != boundary
        self.assertGreater(off, h // 2)

    def test_the_second_panel_is_the_second_layout(self):
        layouts = [set(cells) for _, cells in board.pair_layouts()][:2]
        with flat_rows(), tempfile.TemporaryDirectory() as work:
            sheet = board.pair_sheet("ash", "packed_dirt", {}, layouts, work)
            alone = board.pair_sheet("ash", "packed_dirt", {}, layouts[1:], work)
        w, h = board.PAIR_SIZE[0] * board.CELL_PX, board.PAIR_SIZE[1] * board.CELL_PX
        self.assertEqual(sheet.crop((w + board.CELL_PX // 2, 0, 2 * w + board.CELL_PX // 2, h)).tobytes(), alone.tobytes())


def prop_art(colour, size=(8, 16)):
    return Image.new("RGBA", size, colour)


class PropsTest(unittest.TestCase):
    def root(self, d, name="kit"):
        return os.path.join(d, name)

    def add_prop(self, root, folder, art, version=0):
        path = os.path.join(root, "props", *folder.split("/"))
        os.makedirs(path, exist_ok=True)
        art.save(os.path.join(path, f"v{version}.png"))

    def add_unit(self, root, folder, sheet, frame):
        path = os.path.join(root, "units", *folder.split("/"))
        os.makedirs(path)
        sheet.save(os.path.join(path, "sheet.png"))
        with open(os.path.join(path, "clips.json"), "w", encoding="utf-8") as f:
            json.dump({"frame": frame}, f)

    def write(self, d, text):
        path = os.path.join(d, "props.txt")
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        return path

    def test_props_come_back_by_feet_a_tie_in_the_files_order(self):
        with tempfile.TemporaryDirectory() as d:
            root = self.root(d)
            for n, c in (("a", (1, 0, 0, 255)), ("b", (2, 0, 0, 255)), ("c", (3, 0, 0, 255))):
                self.add_prop(root, f"common/{n}", prop_art(c))
            items = board.parse_props(self.write(d, "prop common/a 1 5 # far in the file\nprop common/b 1 2\n\n# note\nprop common/c 1.5 5\n"), 10, 10, [root])
        self.assertEqual([item[0].getpixel((0, 0))[0] for item in items], [2, 1, 3])
        self.assertEqual([(item[1], item[2]) for item in items], [(1, 2), (1, 5), (1.5, 5)])

    def test_a_prop_is_its_lowest_version_and_a_unit_frame_zero_and_the_game_goes_first(self):
        with tempfile.TemporaryDirectory() as d:
            game, kit = self.root(d, "game"), self.root(d, "kit")
            self.add_prop(kit, "common/p", prop_art((9, 0, 0, 255)), 2)
            self.add_prop(kit, "common/p", prop_art((8, 0, 0, 255)), 10)
            self.add_prop(kit, "common/both", prop_art((7, 0, 0, 255)))
            self.add_prop(game, "common/both", prop_art((6, 0, 0, 255)))
            sheet = Image.new("RGBA", (96, 48), (5, 0, 0, 255))
            sheet.paste((4, 0, 0, 255), (48, 0, 96, 48))
            self.add_unit(kit, "warrior/base", sheet, [48, 48])
            text = "prop common/p 1 1\nprop common/both 1 2\nunit warrior/base 1 3\n"
            arts = [item[0] for item in board.parse_props(self.write(d, text), 10, 10, [game, kit])]
        self.assertEqual(arts[0].getpixel((0, 0))[0], 9)
        self.assertEqual(arts[1].getpixel((0, 0))[0], 6)
        self.assertEqual((arts[2].size, arts[2].getpixel((47, 0))[0]), ((48, 48), 5))

    def test_a_bad_props_file_is_refused_not_cut_to_fit(self):
        bad = {
            "prop common/a 1": "not `prop",
            "thing common/a 1 1": "not `prop",
            "prop ../../secret 1 1": "not a folder name",
            "prop common/a x 1": "could not convert",
            "prop common/a nan 1": "off the",
            "prop common/a 1 inf": "off the",
            "prop common/a 11 1": "off the",
            "prop common/a 1 -1": "off the",
            "prop common/none 1 1": "no prop common/none",
            "unit common/a 1 1": "no unit common/a",
        }
        with tempfile.TemporaryDirectory() as d:
            root = self.root(d)
            self.add_prop(root, "common/a", prop_art((1, 0, 0, 255)))
            for text, message in bad.items():
                with self.assertRaises(ValueError) as caught:
                    board.parse_props(self.write(d, text), 10, 10, [root])
                self.assertIn(message, str(caught.exception), text)
                self.assertIn("props.txt:1", str(caught.exception), text)
            many = "prop common/a 1 1\n" * (board.MAX_PROPS + 1)
            with self.assertRaises(ValueError) as caught:
                board.parse_props(self.write(d, many), 10, 10, [root])
            self.assertIn("at most", str(caught.exception))
            self.add_prop(root, "common/big", Image.new("RGBA", (board.MAX_MASTER + 1, 1)))
            with self.assertRaises(ValueError) as caught:
                board.parse_props(self.write(d, "prop common/big 1 1"), 10, 10, [root])
            self.assertIn("px a side", str(caught.exception))

    def test_feet_stand_at_the_arts_bottom_centre_and_the_front_one_covers(self):
        ground = Image.new("RGB", (64, 64), (10, 10, 10))
        red, blue = (200, 0, 0, 255), (0, 0, 200, 255)
        out = board.draw_props(ground, [(prop_art(red), 1.0, 1.0), (prop_art(blue), 1.25, 1.0)], 32)
        self.assertEqual(out.size, (64, 64))
        self.assertEqual(out.getpixel((28, 31))[:3], red[:3])                  # foot at 32, 32: art 28-35 by 16-31
        self.assertEqual(out.getpixel((36, 31))[:3], blue[:3])                 # the later one, 36-43
        self.assertEqual(out.getpixel((28, 32)), (10, 10, 10))                 # nothing under the feet
        self.assertEqual(out.getpixel((28, 15)), (10, 10, 10))
        self.assertEqual(out.getpixel((27, 31)), (10, 10, 10))
        over = board.draw_props(ground, [(prop_art(red), 1.0, 1.0), (prop_art(blue), 1.0, 1.0)], 32)
        self.assertEqual(over.getpixel((30, 20))[:3], blue[:3])

    def test_a_clear_pixel_shows_the_ground_and_art_past_a_side_is_cut(self):
        ground = Image.new("RGB", (32, 32), (10, 10, 10))
        art = prop_art((200, 0, 0, 255), (16, 40))
        art.putpixel((8, 39), (0, 0, 0, 0))
        out = board.draw_props(ground, [(art, 0.0, 1.0)], 32)                  # past the left and the top
        self.assertEqual(out.getpixel((0, 31)), (10, 10, 10))                  # art's (8, 39)
        self.assertEqual(out.getpixel((1, 31)), (200, 0, 0))
        self.assertEqual(out.getpixel((7, 0)), (200, 0, 0))
        self.assertEqual(out.getpixel((8, 0)), (10, 10, 10))
        out = board.draw_props(ground, [(art, 1.0, 1.0)], 32)                  # past the right
        self.assertEqual(out.getpixel((31, 0)), (200, 0, 0))
        self.assertEqual(out.getpixel((23, 0)), (10, 10, 10))

    def test_the_outdoor_example_is_real_art_on_its_map_and_nothing_stands_in_water(self):
        path = os.path.join(board.HERE, "fixtures", "mix_outdoor_props.txt")
        cols, rows, grid = board.parse_layout(board.MIX_LAYOUT)
        items = board.parse_props(path, cols, rows)
        self.assertGreater(len(items), 0)
        with open(path, encoding="utf-8") as f:
            lines = [line.split() for line in f if line.strip() and not line.startswith("#")]
        self.assertEqual(len(items), len(lines))
        self.assertTrue(any(line[0] == "unit" for line in lines))
        for line in lines:
            x, y = float(line[2]), float(line[3])
            self.assertNotEqual(grid[min(int(y), rows - 1)][min(int(x), cols - 1)], "water", line)

    @needs_composer
    def test_the_mix_draws_them_over_the_map_and_counts_them(self):
        with tempfile.TemporaryDirectory() as d:
            sd = os.path.join(d, "set", "ground", "ash")
            os.makedirs(sd)
            flatten.set_mean(even_tile(45), colour_of("ash")).save(os.path.join(sd, "v0.png"))
            layout = os.path.join(d, "l.txt")
            with open(layout, "w", encoding="utf-8") as f:
                f.write("size 6 4\ndefault ash\n")
            root = self.root(d)
            self.add_prop(root, "common/a", prop_art((200, 0, 0, 255)))
            props = self.write(d, "prop common/a 2 2\n")
            with unittest.mock.patch.object(board, "ART_ROOTS", (root,)):
                bare, _ = board.build_mix(os.path.join(d, "set"), os.path.join(d, "a"), layout)
                dressed, _ = board.build_mix(os.path.join(d, "set"), os.path.join(d, "b"), layout, False, 1.0, props)
            with Image.open(bare) as a, Image.open(dressed) as b:
                a, b = a.convert("RGB"), b.convert("RGB")
            map_w, map_h = 6 * board.CELL_PX * board.MIX_SCALE, 4 * board.CELL_PX * board.MIX_SCALE
            self.assertEqual(a.size, b.size)
            changed = [(x, y) for y in range(map_h) for x in range(map_w) if a.getpixel((x, y)) != b.getpixel((x, y))]
            self.assertEqual((min(changed), max(changed)), ((60, 48), (67, 63)))
            self.assertEqual(len(changed), 8 * 16)
            self.assertEqual(b.getpixel((60, 48)), (200, 0, 0))
            self.assertNotEqual(a.crop((map_w, 0, a.size[0], map_h)).tobytes(), b.crop((map_w, 0, b.size[0], map_h)).tobytes())

    def test_the_command_line_refuses_a_bad_wear_or_props_file(self):
        with tempfile.TemporaryDirectory() as d:
            sd = os.path.join(d, "set", "ground", "ash")
            os.makedirs(sd)
            even_tile(45).save(os.path.join(sd, "v0.png"))
            out = os.path.join(d, "o")
            for args, message in ((["--props", os.path.join(d, "none.txt")], "none.txt"), (["--wear", "nan"], "off 0..")):
                with unittest.mock.patch.object(board, "composer", return_value="unused"), contextlib.redirect_stderr(io.StringIO()) as err:
                    self.assertEqual(board.main([os.path.join(d, "set"), "--mix", "--out", out] + args), 1, args)
                self.assertIn(message, err.getvalue(), args)
            self.assertFalse(os.path.exists(out))


if __name__ == "__main__":
    unittest.main()
