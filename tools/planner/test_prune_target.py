"""TARGET-PRUNE1 (ENGINE-39): `prune_target.py` on a fake target folder: the newest executable per name stays, the older
ones and the incremental folders go, libraries and unhashed files stay, and a dry run removes nothing."""
import os
import tempfile
import time
import unittest
from pathlib import Path

import prune_target as pt


def touch(p, mtime, exe=False, size=10):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(b"x" * size)
    if exe:
        p.chmod(0o755)
    os.utime(p, (mtime, mtime))


class PruneTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        now = time.time()
        deps = t / "debug" / "deps"
        touch(deps / "scenario-0123456789abcdef", now - 300, exe=True)
        touch(deps / "scenario-fedcba9876543210", now - 10, exe=True)
        touch(deps / "sim_golden-1111111111111111", now - 50, exe=True)
        touch(deps / "libaegis_core-2222222222222222.rlib", now - 900)
        touch(deps / "libaegis_core-3333333333333333.rlib", now - 10)
        touch(deps / "scenario-0123456789abcdef.d", now - 300)
        touch(deps / "notes.txt", now - 900, exe=True)
        touch(t / "debug" / "incremental" / "x-1" / "dep-graph.bin", now - 900, size=100)
        touch(t / "release" / "deps" / "game-4444444444444444", now - 100, exe=True)
        touch(t / "release" / "deps" / "game-5555555555555555", now - 5, exe=True)
        self.t = t

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_newest_per_name_stays_and_the_rest_go(self):
        names = sorted(str(p.relative_to(self.t)) for p, _ in pt.stale(self.t))
        self.assertEqual(names, ["debug/deps/scenario-0123456789abcdef", "debug/incremental", "release/deps/game-4444444444444444"])
        n, b = pt.prune(self.t)
        self.assertEqual((n, b), (3, 120))
        left = sorted(str(p.relative_to(self.t)) for p in self.t.rglob("*") if p.is_file())
        self.assertIn("debug/deps/scenario-fedcba9876543210", left)
        self.assertIn("debug/deps/sim_golden-1111111111111111", left)
        self.assertIn("debug/deps/libaegis_core-2222222222222222.rlib", left, "libraries stay")
        self.assertIn("debug/deps/scenario-0123456789abcdef.d", left)
        self.assertIn("debug/deps/notes.txt", left)
        self.assertNotIn("debug/deps/scenario-0123456789abcdef", left)
        self.assertFalse((self.t / "debug" / "incremental").exists())

    def test_a_dry_run_removes_nothing(self):
        before = sorted(self.t.rglob("*"))
        n, _ = pt.prune(self.t, dry=True)
        self.assertEqual(n, 3)
        self.assertEqual(sorted(self.t.rglob("*")), before)

    def test_a_missing_target_is_nothing_to_do(self):
        self.assertEqual(pt.prune(self.t / "none"), (0, 0))

    def test_prunes_build_dir_layout(self):
        with tempfile.TemporaryDirectory() as d:
            t = Path(d)
            now = time.time()
            touch(t / "debug" / "build" / "aegis_ui" / "aaaa" / "out" / "ui_test-0123456789abcdef", now - 200, exe=True)
            touch(t / "debug" / "build" / "aegis_ui" / "bbbb" / "out" / "ui_test-fedcba9876543210", now - 10, exe=True)
            touch(t / "debug" / "build" / "aegis_ui" / "aaaa" / "out" / "libaegis_ui-1111111111111111.rlib", now - 200)
            names = [str(p.relative_to(t)) for p, _ in pt.stale(t)]
            self.assertEqual(names, ["debug/build/aegis_ui/aaaa/out/ui_test-0123456789abcdef"])
            self.assertEqual(pt.prune(t), (1, 10))
            self.assertTrue((t / "debug" / "build" / "aegis_ui" / "bbbb" / "out" / "ui_test-fedcba9876543210").is_file())
            self.assertTrue((t / "debug" / "build" / "aegis_ui" / "aaaa" / "out" / "libaegis_ui-1111111111111111.rlib").is_file())


if __name__ == "__main__":
    unittest.main()
