"""Tests for TOOLS-HELP2 (ENGINE-16): `rename_ident.py` (code renamed, strings and comments left unless asked; JSON keys
and exact values) and `shots_all.py` (flat names, compare by bytes); no cargo run."""
import struct
import tempfile
import unittest
import zlib
from pathlib import Path

import rename_ident as rn
import shots_all as sa

RUST = '''/// Doc: old_name here.
fn old_name(x: &'a str) -> char { // old_name in a comment
    let s = "old_name"; let r = r#"old_name "q""#; let b = b"old_name";
    /* outer /* nested old_name */ old_name */
    let c = 'o'; let q = '\\''; old_name_extra(); my_old_name(); old_name(s)
}
'''


class Rename(unittest.TestCase):
    def test_rust_code_only_by_default(self):
        out, done, ls, lc = rn.rename_rust(RUST, "old_name", "new_name")
        self.assertEqual((done, ls, lc), (2, 3, 4))
        self.assertIn("fn new_name(x: &'a str)", out)
        self.assertIn("old_name_extra(); my_old_name(); new_name(s)", out)
        self.assertIn('"old_name"', out)
        self.assertIn("/// Doc: old_name here.", out)

    def test_rust_strings_and_comments_when_asked(self):
        out, done, ls, lc = rn.rename_rust(RUST, "old_name", "new_name", strings=True, comments=True)
        self.assertEqual((done, ls, lc), (9, 0, 0))
        self.assertNotIn("old_name\"", out)
        self.assertIn("old_name_extra", out)

    def test_json_keys_and_exact_values(self):
        src = '{\n  "old_name": 1,\n  "x": "old_name",\n  "y": "the old_name rule",\n  "z": "old_names"\n}\n'
        out, done, left = rn.rename_json(src, "old_name", "new_name")
        self.assertEqual((done, left), (2, 1))
        self.assertEqual(out, src.replace('"old_name"', '"new_name"'))
        out, done, left = rn.rename_json(src, "old_name", "new_name", strings=True)
        self.assertEqual((done, left), (3, 0))
        self.assertIn('"the new_name rule"', out)

    def test_skipped_places(self):
        self.assertEqual(rn.skipped("legacy/x.json"), "legacy")
        self.assertEqual(rn.skipped("games/emberwake/data/catalog/classes.json"), "game data")
        self.assertEqual(rn.skipped("engine/crates/c/tests/fixtures/g.json"), "test fixture")
        self.assertIsNone(rn.skipped("games/emberwake/assets/ui/theme.json"))


def png(w, h, px, filt=0):
    """An RGB PNG of rows of (r, g, b); filt 0 none, 1 sub, 2 up (so the reader's filters are tried)."""
    raw, prev = b"", bytes(w * 3)
    for row in px:
        line = bytes(c for p in row for c in p)
        enc = line if filt == 0 else bytes((line[i] - (line[i - 3] if i >= 3 else 0)) & 255 for i in range(len(line))) if filt == 1 else bytes((line[i] - prev[i]) & 255 for i in range(len(line)))
        raw += bytes([filt]) + enc
        prev = line

    def chunk(k, d):
        return struct.pack(">I", len(d)) + k + d + struct.pack(">I", zlib.crc32(k + d))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")


class ShotsAll(unittest.TestCase):
    def test_pixels_past_the_tolerance(self):
        with tempfile.TemporaryDirectory() as d:
            a, b = Path(d) / "a.png", Path(d) / "b.png"
            base = [[(10, 20, 30), (40, 50, 60)], [(70, 80, 90), (100, 110, 120)]]
            a.write_bytes(png(2, 2, base, filt=1))
            b.write_bytes(png(2, 2, [[(10, 20, 30), (45, 50, 60)], [(70, 80, 99), (100, 110, 120)]], filt=2))
            self.assertEqual(sa.png_rows(a)[2], sa.png_rows(Path(d) / "a.png")[2])
            self.assertEqual(sa.changed_pixels(a, b, tolerance=4), 2)
            self.assertEqual(sa.changed_pixels(a, b, tolerance=8), 1)
            b.write_bytes(png(2, 2, base, filt=2))
            self.assertEqual(sa.changed_pixels(a, b, tolerance=0), 0)

    def test_flat_names_and_compare(self):
        with tempfile.TemporaryDirectory() as d:
            b = Path(d) / "b"
            for rel, data in [("menus/1280x720/title.png", b"1"), ("menus/1280x720/title.diff.png", b"d"), ("hub/1280x720/title.png", b"2"),
                              ("hub/1280x720/playing/frame_0001.png", b"f"), ("wide/1280x720/a.png", b"3"), ("wide/1920x1080/a.png", b"4")]:
                (b / rel).parent.mkdir(parents=True, exist_ok=True)
                (b / rel).write_bytes(data)
            names = sa.collect(b, Path(d) / "after")
            self.assertEqual(names, ["hub__title.png", "menus__title.png", "wide__a@1280x720.png", "wide__a@1920x1080.png"])
            self.assertEqual((Path(d) / "after" / "index.txt").read_text().split(), names)
            after, before = Path(d) / "after", Path(d) / "before"
            before.mkdir()
            dark, light = [[(0, 0, 0)] * 4] * 4, [[(255, 255, 255)] * 4] * 4
            (after / "hub__title.png").write_bytes(png(4, 4, dark))
            (before / "hub__title.png").write_bytes(png(4, 4, dark))
            (after / "menus__title.png").write_bytes(png(4, 4, light))
            (before / "menus__title.png").write_bytes(png(4, 4, dark))
            (before / "gone__x.png").write_bytes(png(4, 4, dark))
            (after / "wide__a@1280x720.png").write_bytes(png(4, 4, dark, filt=1))
            (before / "wide__a@1280x720.png").write_bytes(png(4, 4, dark))
            (after / "wide__a@1920x1080.png").unlink()
            changed, new, gone, same = sa.compare(before, after, tolerance=8, pixels=10)
            self.assertEqual((changed, new, gone, same), ([("menus__title.png", 16)], [], ["gone__x.png"], 2))
            self.assertEqual(sa.compare(before, after, pixels=16)[0], [])
            self.assertTrue(sa.report(changed, new, gone, same, 8, 10)[1].endswith("menus__title.png (16 px)"))

if __name__ == "__main__":
    unittest.main()
