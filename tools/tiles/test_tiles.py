"""LOOK1-T tests: temp dirs and fixtures only."""
import os
import random
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
