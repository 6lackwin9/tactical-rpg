"""Tests for feedback_check.py (SKILLS1-FB): synthetic FEEDBACK / READY / archive files in a temp folder."""
import tempfile
import unittest
from pathlib import Path

import feedback_check as fc

READY = """# Ready Queue

## 1. Ready to Build (`ready-to-build`)

| Order | Task ID | Masterplan | Layer | Summary & Spec | Acceptance Test |
| --- | --- | --- | --- | --- | --- |
| 1 | `ROW-A` | `X` | kit | a | a |
| 2 | `S1.2-S2` & `S1.2-S3` | `S1.2` | kit | b | b |

## 2. Feasibility Check (`feasibility-check`)

| Order | Sub-block / ID | Spec Audited | Audit Summary | Status |
| --- | --- | --- | --- | --- |
| 1 | `NOT-A-TASK` | x | x | x |

## 3. Held

| Wave 2 Order | Task ID | Masterplan | Summary & Dependencies | Named Golden Re-record |
| --- | --- | --- | --- | --- |
| W2-1 | `U4.2-S3..S4` | `U4.2` | c | none |

## 4. Feedback Responses

| Feedback ID | Disposition | Follow-up | Design Decision / Response | Date |
| --- | --- | --- | --- | --- |
{rows}
"""

FEEDBACK = """# Build Status and Feedback

## 2. Feedback Items (Append-Only)

### FB-01: first

- **Spec**: x
- **Status**: open
- **Finding**: one

### FB-02: second

- **Status**: open
- **Finding**: two

## 3. Notes for the user

### UN-01: a note

- **For**: user
- **Status**: open
- **Note**: three
"""

CLOSED = """# Closed Feedback Archive

## FB-00 (moved earlier)

### FB-00: old

- **Status**: closed
"""

ALL_ROWS = "\n".join([
    "| `FB-01` | `accepted` | `ROW-A` | yes | 2026-11-07 |",
    "| `FB-02` | `deferred` | `U4.2-S4` | later | 2026-11-07 |",
    "| `UN-01` | `policy` | `none` | ok | 2026-11-07 |",
])


def run(feedback=FEEDBACK, rows=ALL_ROWS, closed=CLOSED, pending_ok=False):
    return fc.check(feedback, READY.format(rows=rows), closed, pending_ok)[0]


class CheckTest(unittest.TestCase):
    def test_matching_files_pass(self):
        rep = run()
        self.assertEqual(rep.errors, [])
        self.assertEqual(rep.archivable, ["FB-01", "FB-02", "UN-01"])

    def test_missing_follow_up_task_id_fails(self):
        rep = run(rows=ALL_ROWS.replace("`ROW-A`", "`ROW-Z`"))
        self.assertEqual(len(rep.errors), 1)
        self.assertIn("ROW-Z", rep.errors[0])
        self.assertNotIn("FB-01", rep.archivable)

    def test_section_2_ids_are_not_tasks(self):
        rep = run(rows=ALL_ROWS.replace("`ROW-A`", "`NOT-A-TASK`"))
        self.assertTrue(any("NOT-A-TASK" in e for e in rep.errors))

    def test_open_item_without_row_fails_unless_pending_ok(self):
        rows = "\n".join(r for r in ALL_ROWS.splitlines() if "FB-02" not in r)
        rep = run(rows=rows)
        self.assertEqual(len(rep.errors), 1)
        self.assertIn("FB-02", rep.errors[0])
        rep = run(rows=rows, pending_ok=True)
        self.assertEqual(rep.errors, [])
        self.assertEqual(len(rep.waiting), 1)

    def test_empty_follow_up_and_bad_disposition_fail(self):
        rep = run(rows=ALL_ROWS.replace("`ROW-A`", "").replace("`policy`", "`maybe`"))
        self.assertEqual(len(rep.errors), 2)
        self.assertTrue(any("empty Follow-up" in e for e in rep.errors))
        self.assertTrue(any("maybe" in e for e in rep.errors))

    def test_range_and_ampersand_task_cells_expand(self):
        self.assertEqual(run(rows=ALL_ROWS.replace("`U4.2-S4`", "`U4.2-S3..S4`")).errors, [])
        self.assertEqual(run(rows=ALL_ROWS.replace("`ROW-A`", "`S1.2-S3`")).errors, [])

    def test_row_for_unknown_item_warns(self):
        rep = run(rows=ALL_ROWS + "\n| `FB-99` | `accepted` | `none` | ? | 2026-11-07 |")
        self.assertEqual(rep.errors, [])
        self.assertTrue(any("FB-99" in w for w in rep.warnings))
        self.assertEqual(run(rows=ALL_ROWS + "\n| `FB-00` | `accepted` | `none` | ? | 2026-11-07 |").warnings, [])

    def test_closed_item_without_row_passes(self):
        feedback = FEEDBACK.replace("- **Status**: open\n- **Finding**: two", "- **Status**: closed\n- **Finding**: two")
        rows = "\n".join(r for r in ALL_ROWS.splitlines() if "FB-02" not in r)
        self.assertEqual(run(feedback=feedback, rows=rows).errors, [])

    def test_item_without_status_is_a_parse_error(self):
        with self.assertRaises(fc.ParseError):
            run(feedback=FEEDBACK.replace("- **Status**: open\n- **Finding**: two", "- **Finding**: two"))


class ExitCodeTest(unittest.TestCase):
    """The acceptance test: exit 0 when the files match, non-zero on a missing id or a missing row."""

    def code(self, rows, feedback=FEEDBACK, *extra):
        with tempfile.TemporaryDirectory() as d:
            paths = {n: Path(d) / f"{n}.md" for n in ("fb", "ready", "closed")}
            paths["fb"].write_text(feedback, encoding="utf-8")
            paths["ready"].write_text(READY.format(rows=rows), encoding="utf-8")
            paths["closed"].write_text(CLOSED, encoding="utf-8")
            return fc.main(["check", "--feedback", str(paths["fb"]), "--ready", str(paths["ready"]),
                            "--closed", str(paths["closed"]), *extra])

    def test_match_exits_0(self):
        self.assertEqual(self.code(ALL_ROWS), 0)

    def test_missing_task_id_exits_1(self):
        self.assertEqual(self.code(ALL_ROWS.replace("`ROW-A`", "`ROW-Z`")), 1)

    def test_missing_row_exits_1_unless_pending_ok(self):
        rows = "\n".join(r for r in ALL_ROWS.splitlines() if "UN-01" not in r)
        self.assertEqual(self.code(rows), 1)
        self.assertEqual(self.code(rows, FEEDBACK, "--pending-ok"), 0)

    def test_follow_up_sub_range_passes(self):
        self.assertEqual(self.code(ALL_ROWS.replace("`U4.2-S4`", "`U4.2-S3..S4`")), 0)
        self.assertEqual(self.code(ALL_ROWS.replace("`U4.2-S4`", "`U4.2-S3..S5`")), 1)

    def test_suffixed_feedback_id_exits_2(self):
        self.assertEqual(self.code(ALL_ROWS + "\n| `FB-48a` | `accepted` | `none` | ? | 2026-11-07 |"), 2)


class ArchiveTest(unittest.TestCase):
    def test_archive_moves_answered_items_and_records_disposition(self):
        rows = "\n".join(r for r in ALL_ROWS.splitlines() if "UN-01" not in r)
        with tempfile.TemporaryDirectory() as d:
            paths = {n: Path(d) / f"{n}.md" for n in ("fb", "ready", "closed")}
            paths["fb"].write_text(FEEDBACK, encoding="utf-8")
            paths["ready"].write_text(READY.format(rows=rows), encoding="utf-8")
            paths["closed"].write_text(CLOSED, encoding="utf-8")
            code = fc.main(["archive", "--commit", "abcd1234", "--feedback", str(paths["fb"]),
                            "--ready", str(paths["ready"]), "--closed", str(paths["closed"])])
            self.assertEqual(code, 0)
            fb = paths["fb"].read_text(encoding="utf-8")
            closed = paths["closed"].read_text(encoding="utf-8")
        self.assertNotIn("### FB-01", fb)
        self.assertNotIn("### FB-02", fb)
        self.assertIn("### UN-01", fb)  # no row yet: stays
        self.assertIn("## 3. Notes for the user", fb)
        self.assertIn("- **Status**: closed: accepted (@abcd1234), follow-up `ROW-A`", closed)
        self.assertIn("- **Status**: closed: deferred (@abcd1234), follow-up `U4.2-S4`", closed)
        self.assertIn("## FB-01, FB-02 (answered in READY @abcd1234", closed)
        self.assertIn("### FB-00: old", closed)

    def test_none_follow_up_has_no_suffix(self):
        rep, items, answers = fc.check(FEEDBACK, READY.format(rows=ALL_ROWS), CLOSED, False)
        _, closed = fc.archive(FEEDBACK, CLOSED, items, answers, ["UN-01"], "abcd1234", "2026-11-08")
        self.assertIn("- **Status**: closed: policy (@abcd1234)\n", closed)

    def test_archive_refuses_on_errors(self):
        with tempfile.TemporaryDirectory() as d:
            paths = {n: Path(d) / f"{n}.md" for n in ("fb", "ready", "closed")}
            paths["fb"].write_text(FEEDBACK, encoding="utf-8")
            paths["ready"].write_text(READY.format(rows=ALL_ROWS.replace("`ROW-A`", "`ROW-Z`")), encoding="utf-8")
            paths["closed"].write_text(CLOSED, encoding="utf-8")
            code = fc.main(["archive", "--commit", "abcd1234", "--feedback", str(paths["fb"]),
                            "--ready", str(paths["ready"]), "--closed", str(paths["closed"])])
            self.assertEqual(code, 1)
            self.assertEqual(paths["fb"].read_text(encoding="utf-8"), FEEDBACK)

    def test_parse_building_rows_detects_active_section1_rows(self):
        fb = (
            "## 1. Build Status\n\n"
            "| Task / Sub-block | Stage / Status | Branch / Commit | Notes |\n"
            "| --- | --- | --- | --- |\n"
            "| Done on `develop` | done | `develop` | ok |\n"
            "| R1.1-1 `AUDIT-R1b` | building: 3 lanes | `claude/audit-r1b` | wip |\n"
            "| R1.1-2 `C02-A` | queued | - | - |\n"
            "\n## 2. Feedback Items\n"
        )
        self.assertEqual(fc.parse_building_rows(fb), ["R1.1-1 `AUDIT-R1b` (building: 3 lanes)"])


def quoting(fence: str, info: str = "markdown") -> str:
    """FEEDBACK with FB-01 quoting a whole Markdown doc (its own ## / ### headings and a Status line) in a fence."""
    quoted = "\n".join([
        f"{fence}{info}",
        "# Quoted doc",
        "",
        "## 3. Notes for the user",
        "",
        "### FB-99: not an item",
        "",
        "- **Status**: open",
        "",
        "## Tail section",
        fence,
    ])
    return FEEDBACK.replace("- **Finding**: one\n", f"- **Finding**: one\n\n{quoted}\n\n- **Recommendation**: keep\n")


class FenceTest(unittest.TestCase):
    """FB-169: headings inside a fenced code block (``` or ~~~) are not item or section boundaries."""

    def archive_fb01(self, feedback: str) -> tuple[str, str]:
        rows = "| `FB-01` | `accepted` | `ROW-A` | yes | 2026-11-07 |"
        rep, items, answers = fc.check(feedback, READY.format(rows=rows), CLOSED, True)
        self.assertEqual(rep.errors, [])
        self.assertEqual([it.id for it in items], ["FB-01", "FB-02", "UN-01"])
        self.assertEqual(rep.archivable, ["FB-01"])
        return fc.archive(feedback, CLOSED, items, answers, rep.archivable, "abcd1234", "2026-11-10")

    def test_fenced_item_is_archived_whole(self):
        for fence, info in (("```", "markdown"), ("~~~", "markdown"), ("````", ""), ("~~~~", " md")):
            with self.subTest(fence=fence):
                fb = quoting(fence, info)
                new_fb, closed = self.archive_fb01(fb)
                # the remaining FEEDBACK is the original minus FB-01: FB-02, section 3 and UN-01 untouched
                self.assertEqual(new_fb, FEEDBACK.replace(FEEDBACK[FEEDBACK.index("### FB-01"):FEEDBACK.index("### FB-02")], ""))
                for tail in ("## Tail section", "### FB-99", "- **Recommendation**: keep", "# Quoted doc"):
                    self.assertNotIn(tail, new_fb)
                    self.assertIn(tail, closed)
                # the item's own Status is rewritten; the quoted one inside the fence is not
                self.assertIn("- **Status**: closed: accepted (@abcd1234), follow-up `ROW-A`", closed)
                self.assertIn("### FB-99: not an item\n\n- **Status**: open\n", closed)
                self.assertTrue(closed.rstrip().endswith("- **Recommendation**: keep"))

    def test_shorter_or_other_char_does_not_close(self):
        # a ~~~ line inside a ``` fence, and ``` inside a ```` fence, are content, not closers
        fb = quoting("````").replace("# Quoted doc", "```\n~~~\n# Quoted doc")
        new_fb, closed = self.archive_fb01(fb)
        self.assertNotIn("## Tail section", new_fb)
        self.assertIn("## Tail section", closed)

    def test_unclosed_fence_is_a_parse_error(self):
        fb = FEEDBACK.replace("- **Finding**: one\n", "- **Finding**: one\n\n```markdown\n## Not closed\n")
        with self.assertRaises(fc.ParseError):
            fc.check(fb, READY.format(rows=ALL_ROWS), CLOSED, False)

    def test_fenced_headings_ignored_by_sections_archive_ids_and_build_rows(self):
        ready = READY.format(rows=ALL_ROWS).replace("## 2. Feasibility", "```\n## 4. Fake\n```\n\n## 2. Feasibility")
        self.assertEqual(sorted(fc.sections(ready.splitlines())), ["1", "2", "3", "4"])
        self.assertEqual(run(), fc.check(FEEDBACK, ready, CLOSED, False)[0])
        # a ### FB-77 header quoted in a fence in the archive does not count as archived: its row still warns
        rows = ALL_ROWS + "\n| `FB-77` | `policy` | `none` | ok | 2026-11-07 |"
        self.assertEqual(run(rows=rows, closed=CLOSED + "\n### FB-77: real\n\n- **Status**: closed\n").warnings, [])
        warned = run(rows=rows, closed=CLOSED + "\n~~~\n### FB-77: quoted\n~~~\n").warnings
        self.assertEqual(len(warned), 1)
        self.assertIn("FB-77 is in neither", warned[0])
        fb = ("## 1. Build Status\n\n| Task | Stage / Status |\n| --- | --- |\n"
              "```\n## 2. quoted\n```\n| R1 | building |\n\n## 2. Feedback Items\n")
        self.assertEqual(fc.parse_building_rows(fb), ["R1 (building)"])


if __name__ == "__main__":
    unittest.main()
