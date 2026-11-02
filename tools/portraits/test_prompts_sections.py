# Tests of generate.prompts (ARTQUEUE4): each id resolves to the first ```text block under its own heading, stopping
# at the next heading; a leading style block (before the first id heading) is prefixed only for styled files.
# Run (from tools/portraits): python -m unittest test_prompts_sections
import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import generate as G  # noqa: E402

PROPS = HERE.parents[1] / "games/emberwake/docs/specs/PROPS1_PROMPTS.md"


def section(text, cid):
    """The first text block under the heading naming `cid` (independent re-read of the file)."""
    head = text.index("(`" + cid + "`)")
    start = text.index("```text\n", head) + len("```text\n")
    return text[start:text.index("```", start)].strip()


class Sections(unittest.TestCase):
    def test_props_sheets(self):
        text = PROPS.read_text(encoding="utf-8")
        P = G.prompts(PROPS, True)
        for cid, start in (("props_kit", "A sheet of 16"), ("props_buildings", "A sheet of 4"),
                           ("props_ashlands", "A sheet of 9")):
            self.assertEqual(P[cid], section(text, cid))
            self.assertTrue(P[cid].startswith(start), cid)

    def test_style_block_and_section_end(self):
        md = ("# T\n\n```text\nSTYLE\n```\n\n## A (`a`)\n\n```text\nAAA\n```\n\n## B (`b`)\n\nno block\n\n"
              "## C (`c`)\n\n```text\nCCC\n```\n")
        with tempfile.TemporaryDirectory() as d:
            p = pathlib.Path(d) / "x.md"
            p.write_text(md, encoding="utf-8")
            self.assertEqual(G.prompts(p, True), {"a": "STYLE\n\nAAA", "c": "STYLE\n\nCCC"})
            self.assertEqual(G.prompts(p, False), {"a": "AAA", "c": "CCC"})


if __name__ == "__main__":
    unittest.main()
