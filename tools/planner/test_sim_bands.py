"""Tests for sim_bands.py (`tools/fc.sh sim bands`, PR7-6): synthetic bands files and a fake sim that prints tables
in the shape of `aegis sim duel score=binary` and `aegis sim party`, so no cargo build or fight runs."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import sim_bands  # noqa: E402

FAKE = '''import sys
duel, party = {duel!r}, {party!r}
argv = sys.argv[1:]
if argv[0] == "duel":
    classes = list(duel)
    print("ARGS mode duel, level 5, seeds 2")
    print("DUEL level 5, hero vs hero, score of the ROW class (binary %)")
    print(" " * 10 + "".join(f"{{c:<10}}" for c in classes) + "avg")
    for c in classes:
        cells = "".join(f"{{'-' if o == c else '50%':<10}}" for o in classes)
        print(f"{{c:<10}}{{cells}}{{str(duel[c]) + '%':<10}}X HIGH (band 30-70)")
    print("metric checks: 10 over 4 fights, 0 METRIC FAIL")
elif argv[0] == "party":
    enc = [a.split("=", 1)[1] for a in argv if a.startswith("enc=")][0]
    print("PARTY level 5, comp a,b (2 of PARTY_MAX 4), 2 fights per encounter")
    print(f"{{'encounter':<16}} {{'clear%':>8}} {{'band':>7}} {{'rounds':>7}}")
    if enc in party:
        print(f"{{enc:<16}} {{str(party[enc]) + ' X':>8}} {{'70-90':>7}} {{'6.0':>7}}")
        print(f"  {{enc}}: 1 fights skipped (PLAN SKIP, jitter); the cell is over 9 fights")
    print("plan checks: 0 PLAN FAIL over 2 fights")
else:
    print("SIM argument error: unknown mode")
'''

BANDS = {"level": 5, "duel": {"seeds": 2, "win_pct": [40, 60]}, "party": {"seeds": 2, "clear_pct": {"line": [80, 95], "boss": [30, 50]}}}


class SimBands(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def run_bands(self, duel, party, bands=None):
        """The script as `tools/fc.sh sim bands` starts it, on a bands file and a fake sim; (exit code, output)."""
        fake = self.root / "fake_sim.py"
        fake.write_text(FAKE.format(duel=duel, party=party), encoding="utf-8")
        tuning = self.root / "tuning.json"
        tuning.write_text(json.dumps({"step_ms": 420, "balance_bands": bands or BANDS}), encoding="utf-8")
        p = subprocess.run([sys.executable, str(HERE / "sim_bands.py"), "--tuning", str(tuning), "--sim", f'"{sys.executable}" "{fake}"'], capture_output=True, text=True)
        return p.returncode, p.stdout

    def test_every_row_in_band_exits_0(self):
        code, out = self.run_bands({"warrior": 50, "mage": 40}, {"line": 90, "boss": 30})
        self.assertEqual(code, 0, out)
        self.assertIn("bands: 4 in, 0 out of 4 rows", out)
        self.assertRegex(out, r"duel mage\s+40%\s+40-60\s+in")
        self.assertRegex(out, r"party boss clear\s+30%\s+30-50\s+in")

    def test_a_row_outside_its_band_exits_1(self):
        code, out = self.run_bands({"warrior": 50, "mage": 61}, {"line": 90, "boss": 30})
        self.assertEqual(code, 1, out)
        self.assertRegex(out, r"duel mage\s+61%\s+40-60\s+OUT")
        self.assertIn("1 out of 4 rows (FAIL)", out)
        code, out = self.run_bands({"warrior": 50}, {"line": 100, "boss": 30})
        self.assertEqual(code, 1, out)
        self.assertRegex(out, r"party line clear\s+100%\s+80-95\s+OUT")

    def test_a_row_missing_from_the_sim_output_fails(self):
        code, out = self.run_bands({"warrior": 50}, {"line": 90})
        self.assertEqual(code, 1, out)
        self.assertRegex(out, r"party boss clear\s+missing\s+30-50\s+OUT")

    def test_a_malformed_block_or_a_failing_sim_exits_2(self):
        bad = {"level": 5, "duel": {"seeds": 2, "win_pct": [60, 40]}}
        code, out = self.run_bands({"warrior": 50}, {}, bands=bad)
        self.assertEqual(code, 2, out)
        self.assertIn("win_pct must be [lo, hi]", out)
        code, out = self.run_bands({"warrior": 50}, {}, bands={"party": {"clear_pct": {"crucible": [1, 2]}}})
        self.assertEqual(code, 2, out)
        self.assertIn("no generated encounter", out)
        code, out = self.run_bands({}, {}, bands={"duel": {"win_pct": [40, 60]}})
        self.assertEqual(code, 2, out)
        self.assertIn("no class rows", out)

    def test_the_shipped_block_is_well_formed(self):
        bands = sim_bands.load_bands(sim_bands.TUNING)
        self.assertIsNotNone(bands["duel"])
        self.assertTrue(set(bands["party"]["clear_pct"]) <= set(sim_bands.GENERATED))

    def test_block_checks(self):
        for block, msg in (
            ({"duel": {"win_pct": [40, 60]}, "extra": 1}, "unknown key"),
            ({"level": 0, "duel": {"win_pct": [40, 60]}}, "positive"),
            ({"duel": {"seeds": 0, "win_pct": [40, 60]}}, "positive"),
            ({"duel": {"win_pct": [40, 160]}}, "[lo, hi]"),
            ({"party": {"clear_pct": {}}}, "non-empty"),
            ({"_doc": "x"}, "no rows"),
        ):
            with self.assertRaises(sim_bands.BandsError) as e:
                sim_bands.parse_bands(block)
            self.assertIn(msg, str(e.exception))


if __name__ == "__main__":
    unittest.main()
