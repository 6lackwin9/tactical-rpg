# Godot 4 Workflow & Verification Rules

## 1. Static Typing & Code Standards

- All GDScript files must use strict static typing (`-> void`, `: int`, etc.).
- Use snake_case for file names and PascalCase for `class_name` definitions.
- Keep combat/game domain logic strictly decoupled in `RefCounted` classes.

## 2. File Creation, Deletion & Rename Protocol

Whenever creating new script files, deleting files, or modifying `class_name` definitions:

1. **Regenerate Class Cache**: Run `godot --headless --editor --quit-after 50` to ensure `.godot/global_script_class_cache.cfg` and UIDs are updated.
2. **Recycle the IDE language server only when safe**: never `Stop-Process -Name godot` (parallel worktrees run other agents' suites and sims as `godot` processes). Recycle the LSP only when no agent is running, or stop the editor LSP process by its PID. In a worktree, false IDE diagnostics usually come from the main checkout's class cache: trust `tools/fc.sh parse` there.
3. **Audit Diagnostics**: Run headless integration tests and verify that the active LSP reports 0 errors and 0 warnings.

## 3. Mandatory Post-Code Error Resolution Protocol

After making any code edits or additions to scripts (`*.gd`), scenes (`*.tscn`), or configuration files:

1. **Run the per-edit gate**: `tools/fc.sh parse` after each edit, `tools/fc.sh arena` / `dojo` for behaviour, and `check_project.ps1 -Fast -Quiet` (about 30 s) before handing work back. The full `check_project.ps1` (about 3 min) runs only before a milestone merge.
2. **Verify 3 Check Stages**:
   - All GDScript files must pass Godot parser syntax check (`--check-only`) with 0 errors.
   - Global class cache and imports must compile cleanly (`--editor --quit-after 50`).
   - Headless integration test suite (`test_arena.tscn`) must execute and pass 100% of assertions with exit code 0.
3. **Auto-Remediate All Current Problems**: If any script errors, warnings, markdownlint diagnostics, LSP diagnostics (IDE Problems tab), or test assertion failures occur, the agent MUST diagnose the root cause, fix the code/docs, and rerun the checks until **all current problems are completely resolved (0 errors, 0 warnings, 0 assertion failures)** before proceeding to documentation or committing.

## 4. Mandatory Documentation & Commit Protocol

Only after all current problems and verification checks have passed cleanly:

1. **Synchronize `README.md`**: builders report a docs delta and the planner applies it, so `README.md` is updated with every change to reflect:
   - Any new, extracted, or relocated files in the codebase architecture directory breakdown.
   - Newly implemented features, mechanics, or UI capabilities.
   - Updated assertion counts and test coverage in the testing section.
   - Updated roadmap status (marking completed milestones and adding future placeholders).
2. **Commit Only Once All Current Problems Are Resolved**:
   - Never commit broken code, failing tests, or unaddressed diagnostics.
   - Stage and commit code changes together with the updated `README.md` only once all current problems are 100% resolved and verified.

## 5. GDScript Warning Prevention & Safe Typing Standards

The patterns live in the skill `gdscript-typing` (`.claude/skills/gdscript-typing/SKILL.md`).

## 6. Markdown (.md) Formatting Standards (markdownlint Compliance)

Whenever creating or modifying markdown files (`README.md`, rules, docs), strictly observe the following formatting rules:

1. **Headings (MD022)**: Every heading (`#`, `##`, `###`, etc.) must be preceded by exactly 1 blank line and followed by exactly 1 blank line.
2. **Lists (MD032)**: All bulleted or numbered lists must have a blank line before the first list item and a blank line after the last list item.
3. **Table Column Style & Dividers (MD060)**:
   - Always include whitespace around delimiter pipes: `| --- | --- |` (never `|---|---|`).
   - Pad table cell contents with spaces: `| Header 1 | Header 2 |` (never `|Header 1|Header 2|`).
4. **Multiple Consecutive Blank Lines (MD012)**:
   - Do not use multiple consecutive blank lines. Maintain at most one blank line between sections, paragraphs, lists, or blocks.
5. **Fenced Code Blocks (MD031 & MD040)**:
   - Must be preceded by a blank line and followed by a blank line.
   - Inside list items, indent sub-content and maintain surrounding blank lines.
   - Every fenced code block must declare an explicit language identifier (`gdscript`, `bash`, `powershell`, `text`, etc.).
6. **Spacing & EOF (MD009, MD010, MD047)**:
   - No trailing whitespace at the end of lines.
   - No hard tabs (use spaces).
   - Every file must terminate with a single trailing newline.
7. **IDE Markdownlint Verification**:
   - Whenever any markdown file (`*.md`) is created or edited, check the IDE Problems tab to ensure **0 markdownlint warnings/errors** remain before concluding the turn or committing.
