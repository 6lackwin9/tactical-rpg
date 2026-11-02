# Tests of the dash rows (DASH1, docs/specs/INPUT1.md MR12: a dash drawn with its own clip, front, side and back).
# Run from anywhere:
#   python tools/sprites/test_dash.py
# Every body on the humanoid rig is exported in memory; the shipped sheets are only read. Nothing is written.
import contextlib
import io
import json
import os
import re
import sys
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from PIL import Image  # noqa: E402

import classes  # noqa: E402
import export_units  # noqa: E402
import rig  # noqa: E402
import test_views  # noqa: E402
import variants  # noqa: E402
import views  # noqa: E402

ROWS = ('dash', 'dash_down', 'dash_up')
COIL, DRIVE, FLIGHT, REACH, PLANT = range(5)
APART = 0.15              # a dash frame differs from every walk frame of its view in at least this share at the token size
_cache = {}


def exported(key):
    if key not in _cache:
        with contextlib.redirect_stdout(io.StringIO()):
            _cache[key] = export_units.body(key, "all")
    return _cache[key]


def lowest(im):
    """The lowest row of frame `im` with a figure pixel."""
    return max(y for _, y in test_views.solid(im))


def game_dash_ms():
    """The battle screen's slide of a dash, read from its source."""
    path = os.path.join(HERE, "..", "..", "games", "emberwake", "crates", "emberwake_gui", "src", "battle.rs")
    with open(path, encoding="utf-8") as f:
        return int(re.search(r"const DASH_MS: u64 = (\d+);", f.read()).group(1))


class DashRows(unittest.TestCase):
    def bodies(self):
        return [k for k, _, _ in test_views.rig_bodies()]

    def test_every_humanoid_body_has_the_three_rows(self):
        self.assertTrue(self.bodies())
        for key in self.bodies():
            clips = {c["id"]: c for c in exported(key)[4]["clips"]}
            for row in ROWS:
                self.assertIn(row, clips, key)
                self.assertEqual(clips[row]["ms"], rig.DASH_MS, (key, row))
                self.assertFalse(clips[row]["loop"], (key, row))
            self.assertEqual((clips["dash_down"]["of"], clips["dash_down"]["view"]), ("dash", "down"), key)
            self.assertEqual((clips["dash_up"]["of"], clips["dash_up"]["view"]), ("dash", "up"), key)

    def test_other_bodies_have_none(self):
        humanoid = set(self.bodies())
        others = [k for k in variants.all_bodies() if k not in humanoid]
        self.assertTrue(others)
        for key in others:
            self.assertFalse(set(ROWS) & set(exported(key)[2]), key)

    def test_the_clip_runs_as_long_as_the_slide(self):
        # the battle screen plays it unstretched over its slide (play_for), so a shorter clip would hold and a longer one cut
        self.assertEqual(sum(rig.DASH_MS), game_dash_ms())

    def test_side_rows_keep_the_wind_ups_last(self):
        for key in self.bodies():
            side = [c["id"] for c in exported(key)[4]["clips"] if "of" not in c]
            self.assertEqual(side[-7:], ["dash"] + list(rig.WINDUP_ROWS), key)

    def test_the_flight_leaves_the_ground(self):
        for key in self.bodies():
            rows = exported(key)[2]
            for row in ROWS:
                self.assertLess(lowest(rows[row][FLIGHT]), lowest(rows[row][COIL]), (key, row))

    def test_it_is_not_a_walk(self):
        # the flight against every walk frame of the same view, at the battle's token size
        for key in self.bodies():
            rows = exported(key)[2]
            for row, walk in (("dash", "walk"), ("dash_down", "walk_down"), ("dash_up", "walk_up")):
                share = min(views.difference(rows[row][FLIGHT], w, size=export_token()) for w in rows[walk])
                self.assertGreaterEqual(share, APART, (key, row))

    def test_the_figure_stays_in_the_frame(self):
        # the figure and its weapon, the dust and speed lines left out (they may run off the frame's edge)
        none = lambda *a: []
        with mock.patch.object(rig, "dash_trail", none), mock.patch.object(views, "dash_trail", none):
            for key, cls, u in test_views.rig_bodies():
                carry = classes.DASH_CARRY['%s/%s' % (cls, u['grip']) if u.get('grip') else cls]
                g = views.grip_of(cls, u)
                drawn = [("dash", rig.dash(u, **carry)[0])]
                drawn += [("dash_" + v, views.dash(u, v, **views.grip_kw(g, "dash", v))[0]) for v in views.VIEWS]
                for row, frames in drawn:
                    for im in frames:
                        xs = [x for x, _ in test_views.solid(im)]
                        self.assertGreater(min(xs), 0, (key, row))
                        self.assertLess(max(xs), rig.W - 1, (key, row))

    def test_shipped_sheets_hold_the_rows_the_rig_draws(self):
        for key in self.bodies():
            layer, folder, rows, _, clips = exported(key)
            base = os.path.join(export_units.WORKSPACE, *export_units.LAYERS[layer], "units", *folder.split("/"))
            with open(os.path.join(base, "clips.json"), encoding="utf-8") as f:
                self.assertEqual(json.load(f), clips, key)
            with Image.open(os.path.join(base, "sheet.png")) as sheet:
                got = sheet.convert("RGBA")
            for r, c in enumerate(clips["clips"]):
                for i, im in enumerate(rows[c["id"]]):
                    box = (i * rig.W, r * rig.W, (i + 1) * rig.W, (r + 1) * rig.W)
                    self.assertEqual(got.crop(box).tobytes(), im.tobytes(), (key, c["id"], i))


class DashMarks(unittest.TestCase):
    def test_speed_lines_on_the_side_view_only(self):
        for _, _, u in test_views.rig_bodies():
            P = u["pal"]
            self.assertTrue(any(c == P["metal_hi"] for *_, c in rig.dash_trail(P, FLIGHT)))
            for view in views.VIEWS:
                for f in range(len(rig.DASH_MS)):
                    self.assertFalse([d for d in views.dash_trail(P, view, f) if d[3] == P["metal_hi"]], (view, f))

    def test_dust_at_the_drive_and_the_plant_on_every_view(self):
        P = classes.UNITS["warrior"]["pal"]
        for f in (DRIVE, PLANT):
            self.assertTrue(any(c == P["metal_lo"] for *_, c in rig.dash_trail(P, f)))
            for view in views.VIEWS:
                self.assertTrue(any(c == P["metal_lo"] for *_, c in views.dash_trail(P, view, f)), (view, f))

    def test_every_grip_with_views_carries_a_dash(self):
        for key, cls, u in test_views.rig_bodies():
            g = views.grip_of(cls, u)
            self.assertIn("dash", g, key)
            for view in views.VIEWS:
                self.assertIn(view, g["dash"], key)


class CheckDash(unittest.TestCase):
    """export_units.check_dash on small frames, so a broken body exits instead of shipping."""
    A, B = (10, 20, 30), (40, 50, 60)
    PAL = {"a": A, "b": B}

    def frame(self, *colours):
        im = Image.new("RGBA", (rig.W, rig.W), (0, 0, 0, 0))
        for i, c in enumerate(colours):
            im.putpixel((i, 0), c + (255,))
        return im

    def run_check(self, dash_frame, ms=None, loop=False):
        frames = {"idle": [self.frame(self.A)], "dash": [dash_frame]}
        clips = [{"id": "idle"}, {"id": "dash", "ms": list(rig.DASH_MS) if ms is None else ms, "loop": loop}]
        export_units.check_dash("test", frames, clips, self.PAL, ("a", "b"))

    def test_passes_with_no_new_slot(self):
        self.run_check(self.frame(self.A))

    def test_a_new_slot_exits(self):
        with self.assertRaises(SystemExit):
            self.run_check(self.frame(self.A, self.B))

    def test_other_timing_exits(self):
        with self.assertRaises(SystemExit):
            self.run_check(self.frame(self.A), ms=[90] * 5)

    def test_a_loop_exits(self):
        with self.assertRaises(SystemExit):
            self.run_check(self.frame(self.A), loop=True)


def export_token():
    """The token size the battle draws at, tuning.json's token_px."""
    with open(os.path.join(export_units.WORKSPACE, "games", "emberwake", "tuning.json"), encoding="utf-8") as f:
        return json.load(f)["token_px"]


if __name__ == "__main__":
    unittest.main()
