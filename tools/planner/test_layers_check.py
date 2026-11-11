"""Tests for the layer check G3 of tools/fc.sh (`tools/fc.sh layers`, LAYER-DOWN-1): a synthetic project in a temp git
repo with a copy of fc.sh; engine and kit code (src and tests), kit schema.json and kit.json must name no lore word of a
game, any case, whole word with `_` as a separator."""
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

FC = Path(__file__).resolve().parent.parent / "fc.sh"
BASH = shutil.which("bash")
GIT = shutil.which("git")

LORE = "# test lore words\nCold Steel\nMarrow\nSt. Ives\n"


@unittest.skipUnless(BASH and GIT, "needs bash and git")
class LayersCheck(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.write("tools/fc.sh", FC.read_text(encoding="utf-8"))
        self.write("games/demo/lore_words.txt", LORE)
        self.write("games/demo/data/worlds/sunk_vale/world.json", "{}\n")
        self.write("engine/crates/core/src/lib.rs", "pub fn narrow() -> u32 { 1 }\n")
        self.write("kits/rpg/crates/k/src/lib.rs", "pub const CHANNELED_MARK: &str = \"(Channeled)\";\n")
        self.write("kits/rpg/schema.json", "{\"kinds\": {}}\n")
        subprocess.run([GIT, "init", "-q"], cwd=self.root, check=True)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, rel, text):
        p = self.root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8", newline="\n")

    def layers(self):
        r = subprocess.run([BASH, "tools/fc.sh", "layers"], cwd=self.root, capture_output=True, text=True, timeout=60)
        return r.returncode, r.stdout + r.stderr

    def assert_fails(self, rel, text, word):
        self.write(rel, text)
        code, out = self.layers()
        self.assertNotEqual(code, 0, out)
        self.assertIn(f"{rel}:1 names lore word \"{word}\" of game demo", out)

    def test_clean_tree_passes(self):
        code, out = self.layers()
        self.assertEqual(code, 0, out)
        self.assertIn("[layers] ok", out)

    def test_lowercase_lore_word_in_engine_src_fails(self):
        self.assert_fails("engine/crates/core/src/fx.rs", "let marrow = 1;\n", "marrow")

    def test_lore_word_with_underscore_in_kit_src_fails(self):
        self.assert_fails("kits/rpg/crates/k/src/deck.rs", "pub marrow_actives: bool,\n", "marrow")

    def test_phrase_with_underscore_in_kit_tests_fails(self):
        self.assert_fails("kits/rpg/crates/k/tests/t.rs", "let cold_steel_mark = 2;\n", "cold_steel")

    def test_engine_tests_are_scanned(self):
        self.assert_fails("engine/crates/core/tests/t.rs", "// the COLD STEEL path\n", "COLD STEEL")

    def test_world_id_any_case_in_kit_schema_fails(self):
        self.assert_fails("kits/rpg/kit.json", "{\"world\": \"Sunk_Vale\"}\n", "Sunk_Vale")

    def test_special_characters_are_escaped(self):
        # the old `\&` replacement turned every `.` into `&`, so a phrase with a dot never matched
        self.assert_fails("kits/rpg/crates/k/src/a.rs", "// st. ives\n", "st. ives")
        self.write("kits/rpg/crates/k/src/a.rs", "// stx ives\n")
        code, out = self.layers()
        self.assertEqual(code, 0, out)

    def test_whole_word_only_and_docs_not_scanned(self):
        self.write("kits/rpg/crates/k/src/b.rs", "let marrowbone = 1; // narrow\n")
        self.write("kits/rpg/README.md", "Marrow is a game term.\n")
        self.write("engine/README.md", "Marrow\n")
        code, out = self.layers()
        self.assertEqual(code, 0, out)


if __name__ == "__main__":
    unittest.main()
