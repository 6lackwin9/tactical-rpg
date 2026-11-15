"""Unit tests for native SpriteForge spec, Emberwake canon palettes, regex trait interpreter,
and chroma-key strip/sheet composition (`tools/sprites/forge_spec.py`)."""
import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import forge_spec  # noqa: E402
import export_units  # noqa: E402
import rig  # noqa: E402


class CanonPaletteTests(unittest.TestCase):
    def test_outline_master_hues_skin_and_factions(self):
        self.assertEqual(forge_spec.EMBERWAKE_OUTLINE_RGB, (20, 17, 15))
        self.assertEqual(forge_spec.EMBERWAKE_OUTLINE_RGB, rig.OUT)
        self.assertEqual(forge_spec.EMBERWAKE_OUTLINE_HEX, "#14110F")
        self.assertEqual(len(forge_spec.EMBERWAKE_MASTER_HUES), 16)
        self.assertEqual(len({h.hex for h in forge_spec.EMBERWAKE_MASTER_HUES}), 16)
        self.assertEqual(len(forge_spec.EMBERWAKE_SKIN_RAMP), 3)
        self.assertEqual(len(forge_spec.EMBERWAKE_FACTION_SWATCHES), 6)

    def test_shade_rgb_and_two_or_three_tone_ramps_fit_slots_max(self):
        self.assertEqual(rig.SLOTS_MAX, 16)
        base = forge_spec.parse_hex_rgb("#4B5563")
        lo2, mid2 = forge_spec.shade_ramp(base, tones=2)
        lo3, mid3, hi3 = forge_spec.shade_ramp(base, tones=3)
        self.assertEqual((mid2, mid3), (base, base))
        self.assertTrue(all(a < b for a, b in zip(lo2, mid2)))
        self.assertTrue(all(a > b for a, b in zip(hi3, mid3)))
        self.assertEqual(lo2, lo3)

        for cls in forge_spec.CLASS_RIG_SLOT_ORDER:
            slots, ramps = forge_spec.build_rig_palette(
                forge_spec.FOUNDER_SPECS["kaelen"].palette,
                class_id=cls,
                tones=2,
                include_ramps=True,
            )
            self.assertLessEqual(len(slots), rig.SLOTS_MAX)
            self.assertEqual(set(ramps.keys()), set(slots.keys()))
            for ramp in ramps.values():
                self.assertEqual(len(ramp), 2)

            slots3, ramps3 = forge_spec.build_rig_palette(
                forge_spec.FOUNDER_SPECS["maren"].palette,
                class_id=cls,
                tones=3,
                include_ramps=True,
            )
            self.assertLessEqual(len(slots3), rig.SLOTS_MAX)
            for ramp in ramps3.values():
                self.assertEqual(len(ramp), 3)

    def test_founders_clean_tainted_and_faction_exports(self):
        self.assertEqual(set(forge_spec.FOUNDER_SPECS), {"kaelen", "vesper", "maren", "hollis", "dunstan", "elyse"})
        with tempfile.TemporaryDirectory() as tmp:
            founders = forge_spec.export_founder_palettes(tier="both", out_dir=tmp)
            self.assertEqual(len(founders), 12)
            for key in ("kaelen", "vesper", "maren", "hollis", "dunstan", "elyse"):
                clean = forge_spec.FOUNDER_SPECS[key]
                tainted = clean.with_taint("tainted")
                self.assertEqual(clean.taint_tier, "clean")
                self.assertEqual(tainted.taint_tier, "tainted")
                self.assertIn("(Tainted)", tainted.name)
                self.assertNotEqual(tainted.vfx_aura, "none")
                self.assertLessEqual(len(founders[key]["slots"]), rig.SLOTS_MAX)
                self.assertLessEqual(len(founders[f"{key}_tainted"]["slots"]), rig.SLOTS_MAX)
                with open(os.path.join(tmp, f"{key}_tainted.json"), encoding="utf-8") as f:
                    loaded = json.load(f)
                self.assertIn("slots", loaded)
                self.assertLessEqual(len(loaded["slots"]), rig.SLOTS_MAX)

            factions = forge_spec.export_faction_palettes(out_dir=tmp)
            self.assertEqual(len(factions), 6)
            for fname, fdata in factions.items():
                self.assertLessEqual(len(fdata["slots"]), rig.SLOTS_MAX, fname)
                self.assertTrue(os.path.isfile(os.path.join(tmp, f"{fname}.json")))


class CharacterSpecAxisTests(unittest.TestCase):
    def test_invalid_axis_rejected(self):
        with self.assertRaises(ValueError):
            forge_spec.CharacterSpec(id="bad", name="Bad", build="titanic")
        with self.assertRaises(ValueError):
            forge_spec.CharacterSpec(id="bad", name="Bad", taint_tier="cursed")
        with self.assertRaises(ValueError):
            forge_spec.CharacterSpec(id="bad", name="Bad", weapon="bazooka")


class OfflineTraitInterpreterTests(unittest.TestCase):
    def test_interpret_traits_billhook_warden(self):
        spec = forge_spec.interpret_traits(
            "Heavy grim knight warden in steel plate armor with tattered cape, tallow amber fire embers and iron billhook"
        )
        self.assertEqual(spec.build, "heavy")
        self.assertEqual(spec.archetype, "knight")
        self.assertEqual(spec.outfit, "plate-armor")
        self.assertEqual(spec.back_accessory, "tattered-cape")
        self.assertEqual(spec.weapon, "billhook")
        self.assertEqual(spec.vfx_aura, "embers")
        self.assertEqual(spec.palette.primary, "#92400E")
        self.assertEqual(spec.palette.outline, "#14110F")

    def test_interpret_traits_alchemist_sprayer(self):
        spec = forge_spec.interpret_traits(
            "Lithe alchemist wearing brass goggles and trench coat with canister manifold and salt cyan vitriol sprayer"
        )
        self.assertEqual(spec.archetype, "cyber")
        self.assertEqual(spec.build, "lithe")
        self.assertEqual(spec.headgear, "goggles")
        self.assertEqual(spec.outfit, "trench-coat")
        self.assertEqual(spec.back_accessory, "jetpack")
        self.assertEqual(spec.weapon, "sprayer")
        self.assertEqual(spec.palette.accent, "#06B6D4")

    def test_interpret_traits_bone_saw_inquisition_tainted(self):
        spec = forge_spec.interpret_traits(
            "Tainted iron inquisition prelate in iron-mask and obsidian cape carrying bone-saw with glowing eyes"
        )
        self.assertEqual(spec.taint_tier, "tainted")
        self.assertEqual(spec.headgear, "iron-mask")
        self.assertEqual(spec.eyes, "glowing")
        self.assertEqual(spec.weapon, "bone-saw")
        self.assertEqual(spec.palette.primary, "#14171F")


class ChromaKeyAndStripTests(unittest.TestCase):
    def test_chroma_key_collision_avoidance(self):
        neutral = forge_spec.CharacterPalette("#4B5563", "#991B1B", "#D97706", "#A47A5F", "#2C221E")
        self.assertEqual(forge_spec.resolve_chroma_key(neutral, "auto"), ("#00FF00", (0, 255, 0)))

        green_pal = forge_spec.CharacterPalette("#228833", "#991B1B", "#D97706", "#A47A5F", "#2C221E")
        self.assertEqual(forge_spec.resolve_chroma_key(green_pal, "auto"), ("#FF00FF", (255, 0, 255)))

        green_magenta = forge_spec.CharacterPalette("#228833", "#AA22AA", "#D97706", "#A47A5F", "#2C221E")
        self.assertEqual(forge_spec.resolve_chroma_key(green_magenta, "auto"), ("#0047FF", (0, 71, 255)))

        self.assertEqual(forge_spec.resolve_chroma_key(neutral, "blue"), ("#0047FF", (0, 71, 255)))

    def test_chroma_strip_and_sheet_render(self):
        f1 = rig.Image.new("RGBA", (rig.W, rig.W), (0, 0, 0, 0))
        f1.putpixel((20, 20), (200, 100, 50, 255))
        strip, meta = forge_spec.render_chroma_strip([f1, f1], palette=forge_spec.FOUNDER_SPECS["kaelen"])
        self.assertEqual(strip.size, (rig.W * 4, rig.W))
        self.assertEqual(meta["chroma_hex"], "#00FF00")
        self.assertEqual(strip.getpixel((0, 0)), (0, 255, 0, 255))
        self.assertEqual(strip.getpixel((20, 20)), (200, 100, 50, 255))

        rows = {
            "idle": [f1, f1],
            "walk": [f1, f1, f1, f1],
            "dash": [f1, f1],
            "attack": [f1, f1, f1],
            "windup_buff": [f1, f1],
            "defend": [f1],
            "hurt": [f1],
        }
        sheet, smeta = forge_spec.render_chroma_sheet(rows, palette={"accent": "#228833"}, cell_size=(64, 64))
        self.assertEqual(sheet.size, (8 * 64, 4 * 64))
        self.assertEqual(smeta["chroma_hex"], "#FF00FF")
        self.assertEqual(sheet.getpixel((0, 0)), (255, 0, 255, 255))

        with tempfile.TemporaryDirectory() as tmp:
            paths = export_units.chroma_review(tmp, "warrior/test", rows, {"accent": "#4B5563"}, scale=2)
            self.assertEqual(len(paths), 2)
            self.assertTrue(all(os.path.isfile(p) for p in paths))
            with rig.Image.open(paths[0]) as s:
                self.assertEqual(s.size, (4 * rig.W * 2, rig.W * 2))


class EmberwakeDrawerTests(unittest.TestCase):
    def test_all_emberwake_weapon_and_offhand_drawers_distinct(self):
        import classes
        import views

        u = dict(classes.UNITS["warrior"], offhand=None)
        rendered = {}
        for weapon in ("billhook", "greatsword_ew", "bone_saw", "censer", "cestus", "scourge", "arbalest", "sprayer", "knife"):
            self.assertIn(weapon, rig.DRAWERS)
            self.assertIn(weapon, rig.WEAPON_TIP)
            self.assertIn(weapon, views.PARTS["weapon"])
            wu = dict(u, weapon=weapon)
            rendered[weapon] = rig.pose(wu, wa=35, hand_f=(5, 4.5)).tobytes()
        self.assertEqual(len(set(rendered.values())), len(rendered))
        for part_kind, names in (("offhand", ("pavise", "orb", "braces")), ("back", ("tank",))):
            for name in names:
                self.assertIn(name, views.PARTS[part_kind])
                pu = dict(u, **{part_kind: name})
                self.assertNotEqual(rig.pose(pu, wa=35, hand_f=(5, 4.5)).tobytes(), rig.pose(u, wa=35, hand_f=(5, 4.5)).tobytes())
                self.assertNotEqual(views.pose_view(pu, "down").tobytes(), views.pose_view(u, "down").tobytes())


if __name__ == "__main__":
    unittest.main()
