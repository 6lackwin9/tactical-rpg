# Engine Workflow & Verification Rules (Aegis, Rust)

These rules are written for the desktop IDE. In the cloud there is no IDE: the IDE Problems tab steps (2.3, the IDE part of 3.3, and 6.7) do not apply, and `tools/fc.sh docs` is the markdown check (`docs/workflow/cloud.md`). Since 2026-10-19 (transition step 3) they describe the Rust engine in `engine/`; the Godot rules they replaced are in `legacy/.agents/rules/godot_workflow.md`.

## 1. Static Typing & Code Standards

- Rust is statically typed; the build has no compiler warnings (`tools/fc.sh build` shows them). Clippy (`tools/fc.sh clippy`) is advisory: fix what new code adds, leave old lints to cleanup tasks.
- Follow the patterns in the skill `rust-conventions`: port comments name the Godot function a piece mirrors, JSON-shaped data goes through `aegis_core::json` (Godot Variant semantics), `serde_json` keeps insertion order (`shift_remove`, never `remove`).
- Game rules live in the library crates (`emberwake_rules`, `emberwake_battle`, `emberwake_world`); `emberwake_gui` only draws and calls the command API (`aegis_host`), so everything a player does is scriptable and testable without a window.

## 2. File Creation, Deletion & Rename Protocol

Whenever creating, deleting or renaming modules or crates:

1. **Register them**: a new module gets its `pub mod` line in the crate's `lib.rs`; a new crate goes into `members` in the root `Cargo.toml`; a new test file in `crates/<crate>/tests/` is found by cargo on its own.
2. **Put them where `docs/STRUCTURE.md` says**; if a path is not listed, ask the planner first.
3. **Audit Diagnostics**: build, run the tests, and verify the IDE reports 0 errors and 0 warnings (rust-analyzer; Godot language-server errors on `legacy/` are stale: close any Godot editor open on the old root).

## 3. Mandatory Post-Code Error Resolution Protocol

After making any code edits (`*.rs`, `Cargo.toml`, tools or configuration files):

1. **Run the per-edit checks**: `tools/fc.sh build` after each edit, `tools/fc.sh test <filter>` for the behaviour you touched, and `check_project.ps1 -Fast -Quiet` (or `tools/fc.sh gate`) before handing work back.
2. **Verify the check stages**:
   - Every crate builds with 0 errors and 0 compiler warnings.
   - Every test passes, including the engine goldens (never edited to pass; re-recorded only for an intended change, named in the commit and a `docs/HISTORY.md` row).
   - Data edits leave `aegis content check` clean.
3. **Auto-Remediate All Current Problems**: If any compile errors, warnings, markdownlint diagnostics, IDE diagnostics (IDE Problems tab), or test failures occur, the agent MUST diagnose the root cause, fix the code/docs, and rerun the checks until **all current problems are completely resolved (0 errors, 0 warnings, 0 test failures)** before proceeding to documentation or committing. A golden divergence caused by an intended, approved rule change is not fixed by editing the golden: the task re-records it, named in the commit and a HISTORY row (CLAUDE.md "Engine goldens").

## 4. Mandatory Documentation & Commit Protocol

Only after all current problems and verification checks have passed cleanly:

1. **Synchronize the READMEs** (`README.md` for the project, `engine/README.md` for the engine's crates, tools, tests and screens, `engine/docs/PORT_NOTES.md` for the port record): builders report a docs delta and the planner applies it, so they are updated with every change to reflect:
   - Any new, extracted, or relocated files in the codebase architecture directory breakdown.
   - Newly implemented features, mechanics, or UI capabilities.
   - Updated assertion counts and test coverage in the testing section.
   - Updated roadmap status (marking completed milestones and adding future placeholders).
2. **Commit Only Once All Current Problems Are Resolved**:
   - Never commit broken code, failing tests, or unaddressed diagnostics.
   - Stage and commit code changes together with the updated `README.md` only once all current problems are 100% resolved and verified.

## 5. Engine Code Patterns

The patterns live in the skill `rust-conventions` (`.claude/skills/rust-conventions/SKILL.md`).

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
   - Every fenced code block must declare an explicit language identifier (`rust`, `bash`, `powershell`, `text`, etc.).
6. **Spacing & EOF (MD009, MD010, MD047)**:
   - No trailing whitespace at the end of lines.
   - No hard tabs (use spaces).
   - Every file must terminate with a single trailing newline.
7. **IDE Markdownlint Verification**:
   - Whenever any markdown file (`*.md`) is created or edited, check the IDE Problems tab to ensure **0 markdownlint warnings/errors** remain before concluding the turn or committing.

## 7. Mandatory Post-Generation Image Review Protocol

Whenever generating or editing an image (`generate_image`, `artqueue.py`, or local generation):

1. **Inspect Immediately**: View the generated image and compare it side-by-side against the prompt and any reference images.
2. **Verify Prompt & Reference Fidelity**: Confirm no requested details are missing (e.g., specific props, weapons, floor planes, lighting, or view angles) and the result is not off-model or drifted from the reference image's identity, palette, or 16-bit pixel-art style.
3. **Remediate Before Shipping**: If the image has generation errors, missing details, or reference drift, re-generate or fix it before bundling or presenting it to the user.
