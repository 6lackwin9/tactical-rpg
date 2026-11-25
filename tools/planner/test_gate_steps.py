"""Tests for GATE-PARITY1 (ENGINE-12): the cloud gate (`tools/fc.sh gate`) and the desktop gate (`check_project.ps1`)
both run the steps of tools/gate_steps.txt, so they cannot drift: every listed step maps to a check in fc.sh, fc.sh
names no step outside the list, and check_project.ps1 reads the list and runs the rest through `tools/fc.sh step`."""
import re
import shutil
import subprocess
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent.parent
ROOT = TOOLS.parent
FC = TOOLS / "fc.sh"
PS1 = ROOT / "check_project.ps1"
BASH = shutil.which("bash")


def listed():
    lines = (TOOLS / "gate_steps.txt").read_text(encoding="utf-8").splitlines()
    return [ln.split()[0] for ln in lines if ln.strip() and not ln.lstrip().startswith("#")]


def mapped():
    fc = FC.read_text(encoding="utf-8")
    table = fc[fc.index("gate_step() {"):]
    table = table[:table.index("\n}\n")]
    return re.findall(r"^    ([a-z]+)\) ", table, re.M)


class GateSteps(unittest.TestCase):
    def test_the_list_and_the_fc_table_hold_the_same_steps(self):
        steps = listed()
        self.assertEqual(len(steps), len(set(steps)), "a step listed twice")
        self.assertIn("build", steps)
        self.assertIn("test", steps)
        self.assertEqual(sorted(steps), sorted(mapped()), "tools/gate_steps.txt and fc.sh gate_step differ")

    def test_the_gate_runs_the_list_not_its_own(self):
        fc = FC.read_text(encoding="utf-8")
        gate = fc[fc.index("\ngate() {"):]
        gate = gate[:gate.index("\n}\n")]
        self.assertIn("gate_steps", gate)
        self.assertNotRegex(gate, r"\b(cargo_test|scenarios_all|layers_check|playtest_smoke)\b")

    def test_the_desktop_gate_reads_the_list_and_has_no_copy_of_a_check(self):
        ps = PS1.read_text(encoding="utf-8")
        self.assertIn("tools/gate_steps.txt", ps)
        self.assertIn('Invoke-Fc @("step", $step)', ps)
        for copy in ("git grep", "lore_words", "docs\", \"--all", "tooltests\")"):
            self.assertNotIn(copy, ps, f"check_project.ps1 keeps its own copy of a check ({copy})")

    @unittest.skipUnless(BASH, "needs bash")
    def test_an_unknown_step_fails_and_names_itself(self):
        r = subprocess.run([BASH, str(FC), "step", "nosuch"], cwd=ROOT, capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("no gate step 'nosuch'", r.stdout)
        r = subprocess.run([BASH, str(FC), "step"], cwd=ROOT, capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        for s in listed():
            self.assertIn(s, r.stdout)

    def test_prebuild_checks_run_before_build(self):
        steps = listed()
        build_idx = steps.index("build")
        for cheap in ("docs", "layers", "assetlib", "plannertests", "lints"):
            self.assertLess(steps.index(cheap), build_idx, f"{cheap} should fail-fast before build")

    @unittest.skipUnless(BASH, "needs bash")
    def test_unknown_gate_mode_is_refused(self):
        r = subprocess.run([BASH, str(FC), "gate", "bogus"], cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)
        self.assertIn("usage: tools/fc.sh gate", r.stdout)


if __name__ == "__main__":
    unittest.main()
