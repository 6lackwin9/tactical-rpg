# Tests of the impact frame marks (FX1-L1; decision "Attack timing on the board" A, games/emberwake/docs/decisions/combat.md).
# Run from anywhere:
#   python tools/sprites/test_impact.py
# Every attack clip of every body marks one impact frame (rig.impact); the exporter writes it as "impact" in clips.json,
# a view row carries its side clip's; the mark is metadata only, and the shipped clips.json files hold what the
# exporter writes. Nothing is written outside a temporary folder.
import json
import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import classes  # noqa: E402
import export_units  # noqa: E402
import rig  # noqa: E402
import variants  # noqa: E402


def all_bodies():
    return list(classes.UNITS) + variants.all_bodies()


class ImpactMarks(unittest.TestCase):
    def test_every_attack_clip_marks_one_frame_and_others_none(self):
        for key in all_bodies():
            _, _, frames, _, _, _ = export_units.source(key)
            for clip, frs in frames.items():
                marked = [i for i, f in enumerate(frs) if f.info.get('impact')]
                if clip in export_units.ATTACKS:
                    self.assertEqual(len(marked), 1, "%s %s marks %s" % (key, clip, marked))
                else:
                    self.assertEqual(marked, [], "%s %s is not an attack clip" % (key, clip))

    def test_the_mark_is_metadata_only(self):
        im = rig.Image.new("RGBA", (4, 4), (1, 2, 3, 4))
        before = im.tobytes()
        self.assertIs(rig.impact(im), im)
        self.assertEqual(im.tobytes(), before)
        self.assertEqual(rig.impact_frame([rig.Image.new("RGBA", (1, 1)), im]), 1)
        self.assertIsNone(rig.impact_frame([rig.Image.new("RGBA", (1, 1))]))

    def test_clips_json_carries_the_impact_and_view_rows_copy_their_side_clip(self):
        for key in all_bodies():
            layer, folder, _, _, clips = export_units.body(key, "all")
            side = {c["id"]: c for c in clips["clips"] if "of" not in c}
            for c in clips["clips"]:
                if "of" in c:
                    self.assertEqual(c.get("impact"), side[c["of"]].get("impact"), "%s %s" % (key, c["id"]))
                elif c["id"] in export_units.ATTACKS:
                    self.assertTrue(0 <= c["impact"] < c["frames"], "%s %s" % (key, c["id"]))
            # the shipped file is the exporter's output
            path = os.path.join(export_units.WORKSPACE, *export_units.LAYERS[layer], "units", *folder.split("/"), "clips.json")
            with open(path, encoding="utf-8") as f:
                self.assertEqual(json.load(f), clips, "%s: re-export the body (tools/sprites/export_units.py)" % key)


if __name__ == "__main__":
    unittest.main()
