"""Unit tests for `tools/planner/seed_order.py`: bare-list seed files and `fields`-scoped field removals."""
import json
import tempfile
import unittest
from pathlib import Path

import seed_order as so


def write_json(directory, name, payload):
    p = Path(directory) / name
    p.write_text(json.dumps(payload), encoding="utf-8")
    return p


class SeedOrderExtensions(unittest.TestCase):
    def test_bare_list_seed_files_are_parsed_and_checked(self):
        with tempfile.TemporaryDirectory() as d:
            s1 = write_json(
                d,
                "s1.json",
                [{"op": "upsert", "kind": "maps", "entry": {"id": "keep", "rows": "new", "props": "p1"}}],
            )
            s2 = write_json(
                d,
                "s2.json",
                [{"op": "upsert", "kind": "maps", "entry": {"id": "keep", "rows": "old", "props": "p2"}}],
            )
            ship = {("maps", "", "keep"): {"id": "keep", "rows": "new", "props": "p2"}}
            found = so.findings(so.writes([s1, s2]), lambda k, i, w: ship.get((k, w, i)))
            self.assertEqual(found, [("s2.json", "s1.json", "maps", "", "keep", ["rows"])])

    def test_malformed_seed_root_raises_value_error(self):
        with tempfile.TemporaryDirectory() as d:
            bad_root = write_json(d, "bad.json", "not_a_seed")
            with self.assertRaises(ValueError):
                so.writes([bad_root])
            bad_steps = write_json(d, "bad_steps.json", {"steps": "not_a_list"})
            with self.assertRaises(ValueError):
                so.writes([bad_steps])

    def test_fields_removal_in_last_seed_caught_when_shipped_keeps_field(self):
        with tempfile.TemporaryDirectory() as d:
            s1 = write_json(
                d,
                "s1.json",
                {"steps": [{"op": "upsert", "kind": "things", "entry": {"id": "a", "tag": "keep_me", "cost": 1}}]},
            )
            # Last seed explicitly removes `tag` via `fields` (listed in `fields`, omitted from `entry`).
            s2 = write_json(
                d,
                "s2.json",
                {"steps": [{"op": "upsert", "kind": "things", "fields": ["tag"], "entry": {"id": "a"}}]},
            )
            ship = {("things", "", "a"): {"id": "a", "tag": "keep_me", "cost": 1}}
            found = so.findings(so.writes([s1, s2]), lambda k, i, w: ship.get((k, w, i)))
            self.assertEqual(found, [("s2.json", "s1.json", "things", "", "a", ["tag"])])

    def test_fields_removal_in_earlier_seed_caught_when_later_seed_readds_it(self):
        with tempfile.TemporaryDirectory() as d:
            s1 = write_json(
                d,
                "s1.json",
                {"steps": [{"op": "upsert", "kind": "things", "fields": ["tag"], "entry": {"id": "a"}}]},
            )
            # Later stale whole upsert brings `tag` back even though shipped data lacks it.
            s2 = write_json(
                d,
                "s2.json",
                {"steps": [{"op": "upsert", "kind": "things", "entry": {"id": "a", "tag": "stale", "cost": 2}}]},
            )
            ship = {("things", "", "a"): {"id": "a", "cost": 2}}
            found = so.findings(so.writes([s1, s2]), lambda k, i, w: ship.get((k, w, i)))
            self.assertEqual(found, [("s2.json", "s1.json", "things", "", "a", ["tag"])])

    def test_fields_removal_matching_shipped_and_omitted_defaults_produce_no_findings(self):
        with tempfile.TemporaryDirectory() as d:
            s1 = write_json(
                d,
                "s1.json",
                {"steps": [{"op": "upsert", "kind": "things", "entry": {"id": "a", "tag": "old", "system": False}}]},
            )
            s2 = write_json(
                d,
                "s2.json",
                {"steps": [{"op": "upsert", "kind": "things", "fields": ["tag"], "entry": {"id": "a"}}]},
            )
            # Shipped entry has neither `tag` (removed by s2) nor `system` (default omitted by writer).
            ship = {("things", "", "a"): {"id": "a"}}
            self.assertEqual(so.findings(so.writes([s1, s2]), lambda k, i, w: ship.get((k, w, i))), [])

            # Reversing: s2 writes `system: False` (not in ship), but no earlier seed removed `system` via `fields`.
            s3 = write_json(
                d,
                "s3.json",
                {"steps": [{"op": "upsert", "kind": "things", "entry": {"id": "a", "system": False}}]},
            )
            self.assertEqual(so.findings(so.writes([s1, s3]), lambda k, i, w: ship.get((k, w, i))), [])


if __name__ == "__main__":
    unittest.main()
