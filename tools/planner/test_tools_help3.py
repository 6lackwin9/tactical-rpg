"""Tests for TOOLS-HELP3 (ENGINE-20): `seed_order.py` (the seed order check: the last seed of an entry must not undo
an earlier one; `"fields"` steps write only their fields) and `shots_all.py`'s `crop=` and before | after | diff
sheets; no cargo run."""
import json
import tempfile
import unittest
from pathlib import Path

import seed_order as so
import shots_all as sa
from test_tools_help2 import png


def seed(d, name, *steps):
    p = Path(d) / name
    p.write_text(json.dumps({"steps": list(steps)}), encoding="utf-8")
    return p


def up(eid, fields=None, kind="maps", **kv):
    st = {"op": "upsert", "kind": kind, "entry": {"id": eid, **kv}}
    if fields is not None:
        st["fields"] = fields
    return st


class SeedOrder(unittest.TestCase):
    def test_order_by_commit_time_then_name_uncommitted_last(self):
        times = {"b.json": 5, "a.json": 5, "c.json": 1, "new.json": None}
        got = so.order([Path(n) for n in times], when=lambda p: times[Path(p).name])
        self.assertEqual([p.name for p in got], ["c.json", "a.json", "b.json", "new.json"])

    def test_a_stale_whole_upsert_undoes_the_earlier_seed(self):
        with tempfile.TemporaryDirectory() as d:
            ship = {("maps", "", "keep"): {"id": "keep", "rows": "new", "props": "p2", "name": "Keep"}}
            seeds = [seed(d, "s1.json", up("keep", rows="new", props="p1", name="Keep")),
                     seed(d, "s2.json", up("keep", rows="old", props="p2", name="Keep"), {"op": "remove", "kind": "maps", "id": "x"})]
            found = so.findings(so.writes(seeds), lambda k, i, w: ship.get((k, w, i)))
            self.assertEqual(found, [("s2.json", "s1.json", "maps", "", "keep", ["rows"])])

    def test_the_last_seed_matching_the_data_or_a_fields_step_passes(self):
        with tempfile.TemporaryDirectory() as d:
            ship = {("maps", "", "keep"): {"id": "keep", "rows": "new", "props": "p2"}}
            stored = lambda k, i, w: ship.get((k, w, i))
            ok = [seed(d, "s1.json", up("keep", rows="old", props="p1")), seed(d, "s2.json", up("keep", rows="new", props="p2"))]
            self.assertEqual(so.findings(so.writes(ok), stored), [])
            # the last seed writes only props (a fields step): it holds no rows, so it cannot undo f1's
            fl = [seed(d, "f1.json", up("keep", rows="new", props="p1")), seed(d, "f2.json", up("keep", ["props"], props="p2"))]
            self.assertEqual(so.findings(so.writes(fl), stored), [])
            # an entry no longer stored, or one seed alone, compares nothing
            self.assertEqual(so.findings(so.writes(ok), lambda k, i, w: None), [])
            self.assertEqual(so.findings(so.writes(ok[:1]), stored), [])

    def test_known_findings_file(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "known.txt"
            p.write_text("# a comment\ns2.json s1.json maps keep  # why\nbad line\n", encoding="utf-8")
            self.assertEqual(so.known_set(p), {("s2.json", "s1.json", "maps", "keep")})
            self.assertEqual(so.known_set(Path(d) / "none.txt"), set())


class ShotsCrop(unittest.TestCase):
    def test_parse_crop(self):
        self.assertEqual(sa.parse_crop("1,2,3,4"), (1, 2, 3, 4))
        self.assertIsNone(sa.parse_crop(""))
        for bad in ["1,2,3", "a,b,c,d", "0,0,0,4", "-1,0,2,2"]:
            with self.assertRaises(ValueError):
                sa.parse_crop(bad)

    def test_a_crop_counts_only_its_pixels_and_the_sheet_is_three_wide(self):
        with tempfile.TemporaryDirectory() as d:
            before, after = Path(d) / "before", Path(d) / "after"
            before.mkdir()
            after.mkdir()
            dark = [[(0, 0, 0)] * 4 for _ in range(4)]
            lit = [row[:] for row in dark]
            lit[0][0] = lit[3][3] = (255, 255, 255)
            (before / "s__a.png").write_bytes(png(4, 4, dark))
            (after / "s__a.png").write_bytes(png(4, 4, lit))
            self.assertEqual(sa.changed_pixels(before / "s__a.png", after / "s__a.png"), 2)
            self.assertEqual(sa.changed_pixels(before / "s__a.png", after / "s__a.png", crop=(2, 2, 9, 9)), 1, "clipped to the shot")
            changed = sa.compare(before, after, tolerance=8, pixels=0, crop=(0, 0, 2, 2), sheets=True)[0]
            self.assertEqual(changed, [("s__a.png", 1)])
            w, bpp, rows = sa.png_rows(after / "compare" / "s__a.png")
            self.assertEqual((w, len(rows)), (2 * 3 + sa.GAP * 2, 2), "before | after | diff of the 2x2 crop")
            diff = rows[0][(2 * 2 + sa.GAP * 2) * 3:]
            self.assertEqual(diff[:3], b"\xff\x20\x20", "the changed pixel is red")
            self.assertEqual(rows[0][(2 + sa.GAP) * 3:(2 + sa.GAP) * 3 + 3], b"\xff\xff\xff", "the after as it is")
            # collect writes each shot's crop
            b = Path(d) / "bundles" / "s" / "4x4"
            b.mkdir(parents=True)
            (b / "a.png").write_bytes(png(4, 4, lit))
            sa.collect(Path(d) / "bundles", Path(d) / "flat", crop=(3, 3, 1, 1))
            w, _, rows = sa.png_rows(Path(d) / "flat" / "crops" / "s__a.png")
            self.assertEqual((w, rows), (1, [b"\xff\xff\xff"]))


if __name__ == "__main__":
    unittest.main()
