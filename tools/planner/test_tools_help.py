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

    def test_refusals(self):
        with self.assertRaises(ValueError):
            sd.seed_step("units", None, {"id": "w"}, ["level"])
        with self.assertRaises(ValueError):
            sd.seed_step("units", None, {"id": "w", "deck": 3}, ["deck.size=2"])


if __name__ == "__main__":
    unittest.main()
