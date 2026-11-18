import os
import tempfile
import unittest

import unused_deps


def crate(root, name, manifest, code):
    d = os.path.join(root, "engine", "crates", name)
    os.makedirs(os.path.join(d, "src"))
    with open(os.path.join(d, "Cargo.toml"), "w") as f:
        f.write(manifest)
    with open(os.path.join(d, "src", "lib.rs"), "w") as f:
        f.write(code)


class UnusedDeps(unittest.TestCase):
    def test_a_dependency_named_only_in_a_comment_is_unused(self):
        with tempfile.TemporaryDirectory() as root:
            crate(root, "a", '[package]\nname = "a"\n[dependencies]\nserde-json = "1"\nanyhow = "1"\n[target.\'cfg(windows)\'.dependencies]\nwin = "1"\n',
                  "use serde_json::Value;\n// anyhow and win are only mentioned here\n")
            got = [p for c in unused_deps.crates(root) for p in unused_deps.unused(c)]
            self.assertEqual(len(got), 2, got)
            self.assertTrue(any("'anyhow'" in p for p in got) and any("'win'" in p for p in got))
            self.assertEqual(unused_deps.main(root), 1)

    def test_the_keep_list_and_a_clean_crate_pass(self):
        with tempfile.TemporaryDirectory() as root:
            crate(root, "aegis_ui", '[package]\nname = "aegis_ui"\n[dependencies]\ncpal = "1"\n', "fn main() {}\n")
            self.assertEqual(unused_deps.main(root), 0)


if __name__ == "__main__":
    unittest.main()
