"""SHOT-DIFF1 (PR7-72): shots_diff.py picks the scenarios a change can show, compares two shot folders and writes the
index and the contact sheet."""
import tempfile
import unittest
from pathlib import Path

import shots_all as sa
import shots_diff as sd
from test_tools_help2 import png


class ShotsDiff(unittest.TestCase):
    def test_select(self):
        self.assertEqual(sd.select(["docs/READY.md", "README.md", "tools/planner/x.py", ""]), "none", "docs only: no scenario")
        self.assertEqual(sd.select(["docs/FEEDBACK.md", "games/emberwake/scenarios/boss_bar.json", "games/emberwake/scenarios/vn_stage.json"]), "boss_bar,vn_stage")
        self.assertEqual(sd.select(["games/emberwake/scenarios/boss_bar.json", "engine/crates/aegis_ui/src/ui.rs"]), "all")
        self.assertEqual(sd.select(["games/emberwake/assets/ui/theme.json"]), "all")

    def test_one_changed_pixel_is_one_changed_entry_and_identical_folders_none(self):
        with tempfile.TemporaryDirectory() as d:
            base, head, out = Path(d) / "base", Path(d) / "head", Path(d) / "out"
            base.mkdir()
            head.mkdir()
            dark = [[(0, 0, 0)] * 4] * 4
            one = [row[:] for row in dark]
            one[2] = [(0, 0, 0), (0, 0, 0), (9, 0, 0), (0, 0, 0)]
            for n in ["a__x.png", "b__y.png"]:
                (base / n).write_bytes(png(4, 4, dark))
                (head / n).write_bytes(png(4, 4, dark, filt=1))
            changed, new, gone, same = sd.report(base, head, out, base="abc123")
            self.assertEqual((changed, new, gone, same), ([], [], [], 2))
            self.assertFalse((out / "sheet.png").exists(), "nothing changed, no sheet")
            self.assertIn("0 changed, 0 new, 0 gone, 2 unchanged", (out / "index.md").read_text())
            (head / "b__y.png").write_bytes(png(4, 4, one))
            changed, new, gone, same = sd.report(base, head, out, base="abc123")
            self.assertEqual((changed, same), ([("b__y.png", 1)], 1))
            w, _, rows = sa.png_rows(out / "sheet.png")
            self.assertEqual((w, len(rows)), ((4 * 3 + sa.GAP * 2 + 1) // 2, 2), "the before | after | diff sheet at half size")
            self.assertTrue((out / "compare" / "b__y.png").exists())
            self.assertIn("- `b__y.png` (1 px)", (out / "index.md").read_text())

    def test_add_keeps_what_is_there(self):
        with tempfile.TemporaryDirectory() as d:
            b1, b2, flat = Path(d) / "b1", Path(d) / "b2", Path(d) / "flat"
            for b, scen in [(b1, "one"), (b2, "two")]:
                (b / scen / "1280x720").mkdir(parents=True)
                (b / scen / "1280x720" / "s.png").write_bytes(b"x")
            sd.add(b1, flat)
            sd.add(b2, flat)
            self.assertEqual(sorted(p.name for p in flat.glob("*.png")), ["one__s.png", "two__s.png"])

    def test_a_docs_only_change_reports_none_changed(self):
        with tempfile.TemporaryDirectory() as d:
            sd.none(Path(d), base="abc123")
            text = (Path(d) / "index.md").read_text()
            self.assertIn("0 changed, 0 new, 0 gone", text)
            self.assertIn("docs only", text)


if __name__ == "__main__":
    unittest.main()
