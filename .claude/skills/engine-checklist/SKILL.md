---
name: engine-checklist
description: Short edit checklist for the Aegis engine (Rust, engine/): read, register modules, build, targeted tests, gate, warnings, data via aegis content, engine goldens, markdown lint. Use before and after every code or data edit; on any failure load known-issues.
---

# Aegis engine: edit checklist

Rule: when something fails, load `known-issues` and look the failure up there first.

## Every edit: process

1. **Before editing**: read the file's current content (it may have changed on disk). Prefer the Edit/Write tools; use a checked patch script for multi-site edits (skill `patch-and-seed`). Avoid `sed` line-range edits.
2. **Where it goes**: `docs/STRUCTURE.md`; a new module gets its `pub mod` line in the crate's `lib.rs`, a new crate its line in the root `Cargo.toml` `members`.
3. **Build**: `tools/fc.sh build` after each edit (errors and warnings first). The build must have no compiler warnings.
4. **Tests**: `tools/fc.sh test <filter>` for the behaviour you touched (a test-name filter, for example `tools/fc.sh test rules_golden` or `tools/fc.sh test store`); before handing work back, `check_project.ps1 -Fast -Quiet` or `tools/fc.sh gate` (build every crate, run every test). Use `run_in_background` for long runs; read the summary, not the whole output.
5. **Engine goldens**: a rule change is checked by the goldens (`rng_hash`, `rules_golden`, `sim_golden`, `checks_golden`, the world `journey`, the store resave, the timeline and PHASEFOLD goldens; CLAUDE.md "Engine goldens"). A divergence you did not intend is a bug; an intended change re-records the golden, named in the commit and a `docs/HISTORY.md` row; never edit a golden to pass.
6. **Data edits**: only through `aegis content` (`tools/fc.sh content upsert | remove | apply`), then `tools/fc.sh content check` must print `data ok`. Never hand-edit JSON.
7. **Markdown check**: when editing or creating `.md` files, run `tools/fc.sh docs` (size and markdownlint subset, on changed files; `--all` for every file) and, on desktop, verify 0 markdownlint warnings in the IDE Problems tab. Table delimiters need spaces around pipes (`| --- | --- |`, MD060), headings and lists need single blank lines around them (MD022, MD032), and no consecutive blank lines (MD012). Before handing back, `tools/fc.sh layers` (part of the gate) is clean: no game id, crate name or title in `engine/crates/**` or `kits/*/crates/**`.

## Debug method (cheap first)

- Reproduce headlessly before reading code: `aegis battle <encounter> seed=N log` (`cargo run -q -p emberwake_host -- ...` from the repo root), or JSON lines into `aegis serve` (`state`, `plan`, `probe`, `confirm`, `step`, `log`; list in `crates/aegis_host/src/lib.rs`). Print the actual numbers, then compare with what the test assumes. Do not guess at numbers.
- For a golden divergence, the test prints the first differing step (expected, actual): find the rule (PORT_NOTES' file table names the Godot file it was ported from) and read the change that moved it.
- GUI problems: run `emberwake` and drive it through the agent link (`aegis link '{"cmd":"ui_tree"}'`, `click`, `key`, `screenshot`; engine/README.md) instead of guessing at layout.

## Known issues

On any failure, load the skill `known-issues` (symptom / cause / fix table) before debugging, and add new recurring causes there.
