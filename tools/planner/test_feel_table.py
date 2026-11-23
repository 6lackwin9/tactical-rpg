"""ENGINE-57 (FEEL-RELEASE1): feel_table.py lists the scenarios with a feel or reach step and makes one row per step from
a release run's bundles: its numbers, `*` over the step's budget, the failure on the step that failed, `not run` after."""
import json
import tempfile
import unittest
from pathlib import Path

import feel_table as ft

SCENARIO = {"id": "s", "steps": [
    {"step": "wait", "until": {"path": "screen", "eq": "x"}},
    {"step": "feel", "name": "a", "script": [], "budget": {"frame_ms": 33, "ui_sound_ms": 50}},
    {"step": "reach", "name": "r"},
    {"step": "feel", "name": "b", "script": []},
    {"step": "feel", "name": "c", "script": []},
]}

FEEL_A = {"kind": "feel", "name": "a", "to_present": {"worst": 12.0}, "frame_time": {"p50": 5.0, "worst": 40.25},
          "audio": {"ui_sound_ms": 18.0, "underruns": 0}}


def result(perf, ok=True, step="5", message=""):
    return {"size": "1280x720", "result": "pass" if ok else "fail", "step": step, "message": message, "perf": perf}


class FeelTable(unittest.TestCase):
    def test_rows_mark_over_budget_the_failure_and_the_steps_after_it(self):
        reach = {"kind": "reach", "name": "r", "dead_ends": ["x", "y"]}
        rows = ft.rows_of(SCENARIO, [result([FEEL_A, reach], ok=False, message="feel b: worst frame 50 ms over 33 ms")])
        self.assertEqual([r[0] for r in rows], ["s/a", "s/r", "s/b", "s/c"])
        a = dict(zip(["step", "result"] + [c[0] for c in ft.COLUMNS] + ["dead ends"], rows[0]))
        self.assertEqual((a["result"], a["input"], a["frame max"], a["ui snd"], a["underruns"], a["hit"]), ("ok", "12.0", "40.2*", "18.0", "0", "-"))
        self.assertEqual((rows[1][1], rows[1][-1]), ("ok", "2"), "a reach step: its dead ends")
        self.assertEqual(rows[2][1], "FAIL: worst frame 50 ms over 33 ms")
        self.assertEqual(rows[3][1], "not run")

    def test_list_and_table_read_a_run_folder(self):
        with tempfile.TemporaryDirectory() as d:
            sc = Path(d) / "scenarios"
            sc.mkdir()
            (sc / "s.json").write_text(json.dumps(SCENARIO))
            (sc / "plain.json").write_text(json.dumps({"id": "plain", "steps": [{"step": "shot", "name": "x"}]}))
            (sc / "t.json").write_text(json.dumps({"id": "t", "steps": [{"step": "feel", "name": "a", "script": []}]}))
            self.assertEqual([Path(f).name for f in sorted(sc.glob("*.json")) if ft.has_feel(f)], ["s.json", "t.json"])
            out = Path(d) / "out"
            (out / "s" / "1280x720").mkdir(parents=True)
            (out / "s" / "1280x720" / "result.json").write_text(json.dumps(result([FEEL_A])))
            text, bad = ft.table(out, [sc / "s.json", sc / "t.json"])
            self.assertEqual(bad, 1, "t did not run")
            self.assertIn("t: no result (it did not start)", text)
            self.assertIn("5 step(s), 1 measured", text)


if __name__ == "__main__":
    unittest.main()
