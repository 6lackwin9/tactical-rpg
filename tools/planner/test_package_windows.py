"""Tests for tools/release/package_windows.py."""
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

RELEASE_DIR = Path(__file__).resolve().parents[1] / "release"
if str(RELEASE_DIR) not in sys.path:
    sys.path.insert(0, str(RELEASE_DIR))

import package_windows as pw  # noqa: E402


class PackageWindowsTest(unittest.TestCase):
    def _make_workspace(self, root: Path) -> tuple[Path, Path]:
        (root / "engine/assets/fonts").mkdir(parents=True)
        (root / "engine/assets/fonts/mono.ttf").write_bytes(b"TTF")
        (root / "kits/rpg/assets/audio").mkdir(parents=True)
        (root / "kits/rpg/kit.json").write_text('{"id":"rpg"}\n', encoding="utf-8")
        (root / "kits/rpg/assets/audio/click.ogg").write_bytes(b"OGG")
        (root / "games/demo/data/seeds").mkdir(parents=True)
        (root / "games/demo/assets/audio").mkdir(parents=True)
        (root / "games/demo/game.json").write_text('{"id":"demo","kit":"rpg"}\n', encoding="utf-8")
        (root / "games/demo/data/catalog.json").write_text("{}\n", encoding="utf-8")
        (root / "games/demo/data/README.md").write_text("docs\n", encoding="utf-8")
        (root / "games/demo/data/seeds/dev_seed.json").write_text("{}\n", encoding="utf-8")
        (root / "games/demo/assets/audio/theme.ogg").write_bytes(b"OGG")
        bundle = root / "games/demo/web_bundle.json"
        bundle.write_text(
            json.dumps(
                {
                    "root": "../..",
                    "name": "demo",
                    "exclude": ["*.md", "games/demo/data/seeds"],
                    "parts": [
                        {
                            "file": "demo_1.pak",
                            "include": [
                                "games/demo/game.json",
                                "games/demo/data",
                                "games/demo/assets",
                                "kits/rpg/kit.json",
                                "kits/rpg/assets",
                                "engine/assets",
                            ],
                            "exclude": ["games/demo/assets/audio"],
                        },
                        {
                            "file": "demo_2.pak",
                            "include": ["games/demo/assets/audio", "kits/rpg/assets/audio"],
                        },
                    ],
                }
            ),
            encoding="utf-8",
        )
        binary = root / "target/release/demo.exe"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"MZ-DEMO-BIN")
        return binary, bundle

    def test_packages_runtime_files_and_excludes_dev_seeds_and_markdown(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            binary, bundle = self._make_workspace(root)
            out_zip = root / "out/demo-win.zip"
            info = pw.package_release(binary, bundle, out_zip, variant="release", commit="1234abcd")
            self.assertEqual(info["name"], "demo")
            self.assertEqual(info["commit"], "1234abcd")
            with zipfile.ZipFile(out_zip) as zf:
                names = set(zf.namelist())
                self.assertIn("demo.exe", names)
                self.assertIn("README.txt", names)
                self.assertIn("build.json", names)
                self.assertIn("games/demo/game.json", names)
                self.assertIn("games/demo/data/catalog.json", names)
                self.assertIn("games/demo/assets/audio/theme.ogg", names)
                self.assertIn("engine/assets/fonts/mono.ttf", names)
                self.assertNotIn("games/demo/data/README.md", names)
                self.assertNotIn("games/demo/data/seeds/dev_seed.json", names)
                meta = json.loads(zf.read("build.json").decode("utf-8"))
                self.assertEqual(meta["variant"], "release")

    def test_refuses_missing_binary(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            _, bundle = self._make_workspace(root)
            with self.assertRaises(FileNotFoundError):
                pw.package_release(root / "missing.exe", bundle, root / "out.zip")


if __name__ == "__main__":
    unittest.main()
