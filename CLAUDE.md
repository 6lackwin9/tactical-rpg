# Tactical RPG (Godot 4.7, GDScript)

Read these instead of crawling the code:
- `docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/ROADMAP.md`: what is next and open questions
- `docs/DECISIONS.md`: settled decisions (do not re-litigate)
- `docs/WORKFLOW.md`: fast checks, tooling and how to iterate
- Skill `godot-debug-checklist`: known issues table; consult BEFORE debugging

Standing rules: strict static typing; update README on every change; commit/push only when `check_project.ps1` is clean (trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`); edit content through `ContentCatalog.upsert`, never raw JSON; no sed line-range edits.

Feedback rule: look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. When you learn a new recurring failure cause, add it to the known-issues table.
