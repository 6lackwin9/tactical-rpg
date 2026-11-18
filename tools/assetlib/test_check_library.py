"""ASSETLIB1 section 5: the library check on fixture trees (plain Python, no Pillow)."""

import json
import struct
import sys
import tempfile
import unittest
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_library as cl  # noqa: E402


def png(path: Path, w: int, h: int) -> None:
    raw = b"".join(b"\x00" + b"\x00" * (w * 3) for _ in range(h))
    chunk = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def meta(**over):
    m = {"id": "ruined_chapel_night", "kind": "backdrops", "style": "tallow_iron", "size": [64, 36], "palette": "p3",
         "tags": ["ruin", "night", "interior"], "status": "placeholder", "source": "gemini_web", "request": "AI-20",
         "license": "generated for this project (Gemini web), owner-held"}
    m.update(over)
    return m


class LibraryCheck(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        game = self.root / "games" / "somegame"
        (game / "data" / "worlds" / "grey_harbour").mkdir(parents=True)
        (game / "lore_words.txt").write_text("# words\nCold Steel\nVorlach  # a coined name\n", encoding="utf-8")
        self.style = self.root / "kits" / "rpg" / "library" / "tallow_iron"
        self.style.mkdir(parents=True)
        (self.style / "style.meta.json").write_text(json.dumps({"style": "tallow_iron", "label": "Tallow and Iron", "palette": "p3", "notes": ""}))

    def tearDown(self):
        self.tmp.cleanup()

    def add(self, name="ruined_chapel_night", kind="backdrops", size=(64, 36), m=None, raw=None):
        png(self.style / kind / f"{name}.png", *size)
        text = raw if raw is not None else json.dumps(m if m is not None else meta())
        (self.style / kind / f"{name}.meta.json").write_text(text, encoding="utf-8")

    def problems(self):
        return cl.check(self.root)

    def assertOne(self, says):
        p = self.problems()
        self.assertEqual(len(p), 1, p)
        self.assertIn(says, p[0])

    def test_a_clean_library_and_an_absent_one_pass(self):
        self.add()
        self.add("lantern_wall", "props", (16, 32), meta(id="lantern_wall", kind="props", size=[16, 32], frames=[2, 1], status="final", palette=None, request=None))
        self.assertEqual(self.problems(), [])
        empty = tempfile.TemporaryDirectory()
        self.assertEqual(cl.check(Path(empty.name)), [])
        empty.cleanup()

    def test_a_picture_with_no_sidecar(self):
        png(self.style / "props" / "crate.png", 8, 8)
        self.assertOne("no sidecar crate.meta.json")

    def test_bad_json(self):
        self.add(raw="{nope")
        self.assertOne("not JSON")

    def test_a_wrong_kind(self):
        self.add(m=meta(kind="scenery"))
        self.assertOne("kind 'scenery' is not one of")
        self.tearDown()
        self.setUp()
        self.add(m=meta(kind="props"))
        self.assertOne("kind 'props' is not its folder 'backdrops'")

    def test_a_size_mismatch(self):
        self.add(m=meta(size=[640, 360]))
        self.assertOne("size [640, 360] but the PNG is [64, 36]")

    def test_a_bad_status_and_source(self):
        self.add(m=meta(status="draft"))
        self.assertOne("status 'draft'")
        self.tearDown()
        self.setUp()
        self.add(m=meta(source="midjourney"))
        self.assertOne("source 'midjourney'")

    def test_a_duplicate_id(self):
        self.add()
        self.add("ruined_chapel_dawn", m=meta())
        self.assertOne("id 'ruined_chapel_night' is also")

    def test_a_lore_word_in_a_tag_id_or_name(self):
        self.add(m=meta(tags=["ruin", "cold_steel"]))
        self.assertOne("tag 'cold_steel' names the game word 'cold_steel'")
        self.tearDown()
        self.setUp()
        self.add("grey_harbour_docks", m=meta(id="grey_harbour_docks"))
        p = self.problems()
        self.assertEqual(len(p), 2, p)
        self.assertTrue(all("names the game word 'grey_harbour'" in x for x in p), p)
        self.tearDown()
        self.setUp()
        self.add(m=meta(tags=["Somegame"]))
        self.assertOne("names the game word 'Somegame'")

    def test_a_plain_word_inside_a_longer_one_is_not_a_hit(self):
        self.add(m=meta(tags=["steelwork", "vorlachian"]))
        self.assertEqual(self.problems(), [])

    def test_missing_and_unknown_fields(self):
        m = meta(colour="red")
        del m["license"]
        self.add(m=m)
        p = self.problems()
        self.assertEqual(sorted(x.split(": ", 1)[1] for x in p), ["missing 'license'", "unknown key 'colour'"])

    def test_the_folders(self):
        (self.style / "scenery").mkdir()
        self.assertOne("not a kind folder")
        self.tearDown()
        self.setUp()
        (self.style / "style.meta.json").unlink()
        self.assertOne("no style.meta.json")

    def test_a_sound_has_no_pixel_size(self):
        d = self.style / "audio"
        d.mkdir()
        (d / "rain_loop.ogg").write_bytes(b"OggS")
        (d / "rain_loop.meta.json").write_text(json.dumps(meta(id="rain_loop", kind="audio", size=None)))
        self.assertEqual(self.problems(), [])
        (d / "rain_loop.meta.json").write_text(json.dumps(meta(id="rain_loop", kind="audio", size=[1, 1])))
        self.assertOne("a sound has no pixel size")

    def test_a_sidecar_with_no_asset(self):
        d = self.style / "props"
        d.mkdir()
        (d / "ghost.meta.json").write_text(json.dumps(meta(id="ghost", kind="props")))
        self.assertOne("a sidecar with no asset")

    def test_the_real_repository_library_passes(self):
        self.assertEqual(cl.check(Path(__file__).resolve().parents[2]), [])


if __name__ == "__main__":
    unittest.main()
