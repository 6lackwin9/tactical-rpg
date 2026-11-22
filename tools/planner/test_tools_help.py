"""Tests for TOOLS-HELP1 (ENGINE-15): `schema_field.py` (a field added to the one-line schema byte for byte) and
`seed_from.py` (a seed step from a stored entry with changes); no cargo run."""
import json
import unittest

import schema_field as sf
import seed_from as sd

SCHEMA = '{"content":{"items":[{"key":"id","type":"string"},{"key":"name","type":"string"}],"classes":[{"key":"id","type":"string"}]},"world":{"items":[{"key":"id","type":"string"}],"towns":[{"key":"id","type":"string","label":"Ortsname é"}]}}'


class SchemaField(unittest.TestCase):
    def test_added_last_or_after_and_the_rest_byte_for_byte(self):
        text, line = sf.add_field(SCHEMA, "classes", "tier", {"type": "int"})
        self.assertEqual(text, SCHEMA.replace('{"key":"id","type":"string"}]},"world"', '{"key":"id","type":"string"},{"key":"tier","type":"int"}]},"world"'))
        self.assertEqual(line, "added field tier in content.classes (2 fields)")
        text, _ = sf.add_field(SCHEMA, "content.items", "cost", {"key": "cost", "type": "int"}, after="id")
        self.assertIn('"items":[{"key":"id","type":"string"},{"key":"cost","type":"int"},{"key":"name"', text)
        self.assertTrue(text.endswith("}}") and "é" in text)

    def test_refusals(self):
        for args, why in [
            (("items", "x", {}), "content and world"),
            (("shops", "x", {}), "no table shops"),
            (("world.towns", "id", {}), "already has a field id"),
            (("classes", "x", {}, "nope"), "no field nope"),
            (("classes", "x", {"key": "y"}), "is not the field"),
        ]:
            with self.assertRaises(ValueError) as e:
                sf.add_field(SCHEMA, *args)
            self.assertIn(why, str(e.exception))
        with self.assertRaises(ValueError):
            sf.add_field(SCHEMA + "\n", "classes", "x", {})

    def test_replace_keeps_the_place(self):
        text, line = sf.add_field(SCHEMA, "world.towns", "id", {"type": "id"}, replace=True)
        self.assertIn('"towns":[{"key":"id","type":"id"}]', text)
        self.assertTrue(line.startswith("replaced"))


class SeedFrom(unittest.TestCase):
    def test_a_step_with_changes(self):
        entry = {"id": "w", "level": 1.0, "share": 0.25, "deck": {"active": ["a"], "size": 3.0}, "note": "x"}
        step = sd.seed_step("units", "ashlands", entry, ["level=4", "deck.active=[\"a\",\"b\"]", "name=Bold Wren", "note="])
        self.assertEqual(step, {"op": "upsert", "kind": "units", "world": "ashlands",
                                "entry": {"id": "w", "level": 4, "share": 0.25, "deck": {"active": ["a", "b"], "size": 3}, "name": "Bold Wren"}})
        self.assertEqual(list(sd.seed_step("classes", None, {"id": "c"}, [])), ["op", "kind", "entry"])
        json.dumps(step)

    def test_fields_write_only_the_named_fields(self):
        entry = {"id": "w", "level": 1, "deck": {"active": ["a"]}, "note": "x"}
        step = sd.seed_step("units", None, entry, ["deck.active=[\"b\"]", "note="], ["deck", "note"])
        self.assertEqual(step, {"op": "upsert", "kind": "units", "fields": ["deck", "note"], "entry": {"id": "w", "deck": {"active": ["b"]}}})
        for sets, fields in [(["level=2"], ["deck"]), ([], []), ([], ["id"])]:
            with self.assertRaises(ValueError):
                sd.seed_step("units", None, entry, sets, fields)

    def test_refusals(self):
        with self.assertRaises(ValueError):
            sd.seed_step("units", None, {"id": "w"}, ["level"])
        with self.assertRaises(ValueError):
            sd.seed_step("units", None, {"id": "w", "deck": 3}, ["deck.size=2"])

    def test_ids_write_one_step_per_id_in_one_call(self):
        """ENGINE-36 (2): `--ids a,b,c` gives one step per id, each with the same --set and --fields, in order."""
        rows = {"a": {"id": "a", "short_name": "A", "hp": 5}, "b": {"id": "b", "short_name": "B", "hp": 6}}
        get = lambda kind, eid, world: dict(rows[eid])
        steps = sd.steps_for("units", ["b", "a"], None, ["short_name=X"], ["short_name"], get)
        self.assertEqual([s["entry"] for s in steps], [{"id": "b", "short_name": "X"}, {"id": "a", "short_name": "X"}])
        self.assertTrue(all(s["fields"] == ["short_name"] for s in steps))
        for bad in ([], ["a", "a"]):
            with self.assertRaises(ValueError):
                sd.steps_for("units", bad, None, [], None, get)


if __name__ == "__main__":
    unittest.main()
