"""Tests for tools/deps/deny_macros.py (PILLAR1 fast clippy macro deny scanner)."""
import tempfile
import unittest
from pathlib import Path

import deny_macros


class DenyMacrosTest(unittest.TestCase):
    def test_flags_real_macros_and_ignores_comments_and_strings(self):
        src = '''
// dbg!(x);
/* todo!("in block comment") /* nested unimplemented!() */ */
const A: &str = "dbg!(1) todo!() unimplemented!()";
const B: &str = r#"todo!("raw")"#;
const C: char = '"';
fn bad() {
    dbg!(42);
    todo!("later");
    unimplemented!();
}
'''
        macros = {"dbg": "dbg_macro", "todo": "todo", "unimplemented": "unimplemented"}
        hits = deny_macros.scan_source(src, macros)
        self.assertEqual([(m, l) for _, m, l in hits], [
            ("dbg", "dbg_macro"),
            ("todo", "todo"),
            ("unimplemented", "unimplemented"),
        ])

    def test_respects_allow_attributes(self):
        src = '''
#[allow(clippy::todo)]
fn ok_block() {
    todo!("allowed here");
}
fn caught() {
    todo!("not allowed here");
}
'''
        macros = {"todo": "todo"}
        hits = deny_macros.scan_source(src, macros)
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0][0], 7)

    def test_unknown_clippy_deny_requests_fallback(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            (p / "Cargo.toml").write_text(
                '[workspace.lints.clippy]\nunwrap_used = "deny"\n',
                encoding="utf-8",
            )
            self.assertEqual(deny_macros.main(d), 2)

    def test_clean_repo_passes(self):
        root = str(Path(__file__).resolve().parents[2])
        self.assertEqual(deny_macros.main(root), 0)


if __name__ == "__main__":
    unittest.main()
