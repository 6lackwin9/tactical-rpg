# Rule library and fixed-vs-tunable contract (game-agnostic)

Every kit rule option in `REGISTER` (`ruleset.rs`) is additive and off by default when unlisted (`BASERULES1-M`).

## 1. Rule metadata and enforcement

- **Additive options**: A new rule is off when unlisted in a ruleset row, and a new numeric/switch field defaults to existing behaviour. Shipped rows and frozen catalogs are never re-read differently.
- **Metadata per rule (`REGISTER`)**: Each rule declares `id`, `kind` (`Option` or `Extension`), `requires`, `conflicts`, `bundle`, `system`, `tested`, and `does`. Each `FIELD_RULES` entry links a tunable ruleset key to the rule that reads it (`needs`).
- **Validation (`Ruleset::notes()`, `check/ruleset.rs`, `BASERULES1-M2`)**:
  - An unmet `requires` where the rule would be silently inert (such as `flank` without `round_intent`) is a hard error.
  - Conflicts, fields set when their `needs` rule is off, split must-ship bundles (`BUNDLES`), and combinations outside `TESTED_SETS` emit warnings; `tools/fc.sh sim` with `ruleset=` or `rules=` prints any `RULES error:` or `RULES warning:` lines under `ARGS`, and `LOCKED` validation recursively scans nested objects and arrays.
- **One picked set per game**: A game's worlds pick one active ruleset row; `TESTED_SETS` pins the supported option combinations on a frozen catalog, and `kits/rpg/crates/rpgkit_host/tests/fixtures/rules_contract_golden.json` pins `aegis rules --contract`.

## 2. Value vs form vs new rule (the stop rule)

1. **Value (data tuning)**: A number or switch in `FIELD_RULES` or catalog data (`aegis rules --contract`). Tuned freely in `.3` and after `.6` lock without code changes.
2. **Form (fixed behaviour library)**: One of the kit's composable primitives (`U1` 4-stage pipeline, `S1` 6 status behaviours, `U4` 5 passive grant kinds). Locked at `.2`.
3. **New rule**: A mechanic that cannot be expressed as a value or combination of existing primitives. Parked in the candidates table below until a sim or mock proves it improves gameplay.

## 3. Option candidates (kit-agnostic parking table)

| Candidate | Verdict | Cost & Notes |
| --- | --- | --- |
| Stances, matchup, switch AP, `ends_in_stance`, `disrupt` | Kept (`stances`) | Ruleset switch |
| Rest on use; classic chase; Switch step | Kept (`rest_ap`, `chase`, `switch_target_step`) | Ruleset row fields |
| 5 + 3 loadout, class path default | Kept (`deck` off) | - |
| AP = skill level | Kept (`ap_cost` off) | - |
| Square grid, 10-block round | Kept (`rows`, `total_blocks`) | Board fields |
| Bodyguard intercept | Kept (`triggers` intercept) | Data |
| Temporary injury | Kept (`wounded`) | Roster state |
| Ward (absorb pool) | Candidate kit option | S after `S1.2`; a status behaviour + one read after mitigation |
| Taunt guard AI rule (Shield Wall) | Candidate | S; an AI row on `taunt` grants (`I1`) |
| Boss acts twice | Candidate | M; extra actions in a block |
| Elemental damage & per-type resist | Candidate | M; `damage_type` term + per-type resist table |
| Accuracy/evasion rolls | Idea only | M; omitted where resolver is deterministic |
