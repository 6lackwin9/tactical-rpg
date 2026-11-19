"""Tests for GOLDEN-BISECT1 (ENGINE-14) `tools/planner/golden_bisect.py`: what moves between two recordings, by entry
label; the goldens and switches found in a synthetic workspace."""
import tempfile
import unittest
from pathlib import Path

import golden_bisect as gb


class Moved(unittest.TestCase):
    def test_entries_are_named_by_their_label(self):
        a = {"spec": "x", "fights": [{"fight": "duel 7", "steps": [1]}, {"fight": "flank 7", "steps": [2]}, {"fight": "skirmish 42", "steps": [3]}]}
        b = {"spec": "x", "fights": [{"fight": "duel 7", "steps": [1]}, {"fight": "flank 7", "steps": [9]}, {"fight": "skirmish 42", "steps": [8]}]}
        self.assertEqual(gb.moved(a, b), [("fights", 3, ["flank 7", "skirmish 42"])])
        self.assertEqual(gb.describe(gb.moved(a, b)), ["  fights: 2 of 3 moved: flank 7, skirmish 42"])

    def test_unlabelled_entries_by_index_and_plain_values(self):
        self.assertEqual(gb.moved({"runs": [[1], [2], [3]]}, {"runs": [[1], [5], [3], [4]]}), [("runs", 4, ["#2", "#4"])])
        self.assertEqual(gb.moved({"v": 1}, {"v": 2}), [("v", None, [])])
        self.assertEqual(gb.describe(gb.moved({"v": [1]}, {"v": [1]})), ["  nothing moves"])
        self.assertEqual(gb.moved({"units": [["warrior L1", 3]]}, {"units": [["warrior L1", 4]]}), [("units", 1, ["warrior L1"])])


class Repro(unittest.TestCase):
    def test_replay_verdict_and_the_game_cli(self):
        self.assertEqual(gb.replay_verdict(0, "replay ok: 40 frames"), (True, []))
        self.assertEqual(gb.replay_verdict(1, "frame 12\nframe 13 diverges: hp 30 vs 28\n"), (False, ["frame 13 diverges: hp 30 vs 28"]))
        with tempfile.TemporaryDirectory() as d:
            g = Path(d) / "g"
            g.mkdir()
            (g / "game.json").write_text('{"id": "g"}', encoding="utf-8")
            self.assertEqual(gb.game_cli(g), "g_cli")
            (g / "game.json").write_text('{"id": "g", "cli": "aegis"}', encoding="utf-8")
            self.assertEqual(gb.game_cli(g), "aegis")


class Workspace(unittest.TestCase):
    def test_goldens_and_switches_are_found(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            c = root / "games" / "g" / "crates" / "g_rules"
            (c / "tests").mkdir(parents=True)
            (c / "src").mkdir()
            (c / "Cargo.toml").write_text('[package]\nname = "g_rules"\n')
            (c / "tests" / "big_golden.rs").write_text('fn recording() -> bool { std::env::var("RECORD").is_ok() }\n')
            (c / "tests" / "plain.rs").write_text("#[test] fn t() {}\n")
            (c / "src" / "lib.rs").write_text('pub fn f() -> bool { aegis_core::switches::on("regrant") }\n')
            self.assertEqual(list(gb.goldens(root)), ["big_golden"])
            self.assertEqual(gb.find_golden("big", root), ("big_golden", c))
            self.assertEqual(gb.switches_read(root), {"regrant": ["games/g/crates/g_rules/src/lib.rs"]})
            self.assertEqual(gb.crate_name(c), "g_rules")
            with self.assertRaises(SystemExit):
                gb.find_golden("nosuch", root)


if __name__ == "__main__":
    unittest.main()
