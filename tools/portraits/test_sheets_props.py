# Tests of `sheets.py props --reduce scale` (PROPS1-A2): art that is already pixel art is reduced by a smooth scale with
# no palette or outline step; the default path is unchanged. Synthetic sheets in temp dirs only.
# Run (from tools/portraits): <Python312>\python.exe -m unittest test_sheets_props
import pathlib
import sys
import tempfile
import unittest

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import sheets  # noqa: E402

KEY = (255, 0, 255)
COLOUR = (60, 120, 200)


def sheet(sheet_id="props_ashlands", side=900):
    """A sheet of magenta tiles split by black gutters, each holding a flat-coloured block."""
    title, what, n, cells = sheets.PROP_SHEETS[sheet_id]
    im = Image.new("RGB", (side, side), (0, 0, 0))
    d = ImageDraw.Draw(im)
    t = side // n
    for i in range(n * n):
        x, y = (i % n) * t, (i // n) * t
        d.rectangle([x + 4, y + 4, x + t - 5, y + t - 5], fill=KEY)
        d.rectangle([x + t // 4, y + t // 4, x + 3 * t // 4, y + t - 12], fill=COLOUR)
    return im, cells


class PropsReduce(unittest.TestCase):
    def cut(self, **kw):
        im, cells = sheet()
        tmp = pathlib.Path(tempfile.mkdtemp())
        src = tmp / "s.png"
        im.save(src)
        saved = sheets.slice_props(src, "props_ashlands", tmp / "root", tmp / "spares", **kw)
        return cells, saved

    def test_scale_keeps_input_colour_and_size(self):
        cells, saved = self.cut(reduce="scale")
        self.assertEqual(len(saved), len(cells))
        for cell, (_, dest, _) in zip(cells, saved):
            img = Image.open(dest).convert("RGBA")
            self.assertEqual(img.size, tuple(cell[4]))
            colours = {p[:3] for p in img.get_flattened_data() if p[3]}
            self.assertEqual(colours, {COLOUR})     # no outline, no palette change

    def test_default_has_outline_and_matches_explicit_palette(self):
        _, a = self.cut()
        _, b = self.cut(reduce="palette")
        for (_, p, _), (_, q, _) in zip(a, b):
            self.assertEqual(Image.open(p).tobytes(), Image.open(q).tobytes())
        cols = {px[:3] for px in Image.open(a[0][1]).convert("RGBA").get_flattened_data() if px[3]}
        self.assertGreater(len(cols), 1)             # the dark outline


if __name__ == "__main__":
    unittest.main()
