# Whole-Codebase Consistency Audits (`docs/audits/`)

Design-Session snapshot reports produced by the `/consistency-audit` skill (`.claude/skills/consistency-audit/SKILL.md`).

- **Scope vs. `/audit-build`**: `/audit-build` audits **only recent landed Build commits** against their `docs/READY.md` rows, whereas `/consistency-audit` audits the **entire repository** at a pinned date and `origin/develop` commit across **(1)** Docs/Specs/Schema/Tools vs. Build, **(2)** Three-Layer Boundaries (`engine` -> `kits/rpg` -> `games/emberwake`) & Rule-Engine Purity, and **(3)** Lore/Worldbuilding vs. Shipped Data.
- **Naming convention**: `<YYYY-MM-DD>_<short-sha>_consistency_audit.md` (`<= 300` lines, `<= 25` KB, `tools/md_lint.awk`-clean).

## Reports

| Date | Commit (`origin/develop`) | Report | Queued `READY.md` Slices |
| --- | --- | --- | --- |
| 2026-11-10 | `0de2c60` (initial pass `@0f4d5a4` + delta `@c1ca8cc..0de2c60`) | [2026-11-10_0de2c60_consistency_audit.md](2026-11-10_0de2c60_consistency_audit.md) | `PR5-26`..`PR5-34` |
