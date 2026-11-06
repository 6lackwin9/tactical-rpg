# Specs Index and Context Rules

To prevent context bloat and outdated-spec noise in both the Design and Build sessions, files in `docs/specs/` are divided into **Tier 1 (Active Masterplan & Infrastructure)** and **Tier 2 (Pre-Masterplan Reference Library)**.

- **Build Session**: Reads **only** `docs/READY.md` and the specific spec sections named in `docs/READY.md`. Never builds directly from Tier 2 specs.
- **Design Session**: Uses `docs/specs/MASTERPLAN1.md` and `docs/sessions/<SUBBLOCK>.md` as the canonical bottom-up specs. Pulls from Tier 2 specs **only on demand** when opening the matching sub-block's `.1` design session.

## Tier 1: Active Masterplan & Engine/Kit Infrastructure

| File | Role | Status |
| --- | --- | --- |
| `MASTERPLAN1.md` | Canonical 9-block, 25-sub-block, 28-connection, 186-task master plan (all 25 `.1` specs closed in `docs/sessions/*.md`) | active (`.1` closed 2026-11-07) |
| `MASTERPLAN1_ROWS.md` | Mapping of all historical task rows onto the 186 masterplan tasks | active reference map |
| `SILOS1.md` | Silo boundaries (`WORLDFIGHT1`, `SERVICES1`, `KITIDS1`) | active (Q1-Q3 closed 2026-11-07) |
| `AGENTROLES1.md` | Lane scopes, `integration` agent, `aegis scenario`, `SKILLS1` | active (`X6`) |
| `LAYOUT720.md` | 1280 x 720 relative layout grid and canvas policy | active (`X1`) |
| `LAYOUTDATA1.md` | JSON screen layout trees (`ui/layouts/<id>.json`) and `LayoutHost` | active (`X1`) |
| `TERMS1.md` | Neutral kit term register (`content.terms`) and lore-word layer check | active (`X8`) |
| `ASSETORG2.md` | Asset layer directory layout and regeneration checks | active (`X3`) |
| `KIT1.md`, `SPLIT1.md` | Engine / Kit / Game 3-layer split contracts | done / locked (`X8`) |
| `CLEANUP.md` | Housekeeper trim candidates | active (`X7`) |

## Tier 2: Pre-Masterplan Reference Library (Input to `.1` Sessions Only)

These specs record valuable past discussions, edge cases, and partial builds from before the 2026-11-06 bottom-up Master Plan. **Do not dispatch build tasks directly from their `_TASKS.md` files**; each feeds its mapped sub-block's `.1` design session in `docs/sessions/`:

| Pre-Masterplan Spec(s) | Feeds Sub-blocks | How to Use |
| --- | --- | --- |
| `BASERULES1.md`, `_BOUND.md`, `_LIBRARY.md` | `U1-U4`, `A2` | Section 1-2 (`BASERULES1-M` metadata) is active. Formulas and `feel_flat` (`BASERULES1-F`) are **superseded** by `docs/sessions/U1_STATS.md` and `U2_CLASS.md`. Keep `_LIBRARY.md` as the candidate behaviour catalog. |
| `DERIVE1.md` | `U1`, `U3`, `U4` | Historical context on data-driven stat derivation; superseded by `U1_STATS.md`. |
| `ROSTER1.md`, `ROSTER1_TASKS.md` | `U2`, `U4`, `E2` | Roster core is built; temperament (`ROSTER1-T`) feeds `U4.1`; bunks/wounds/keepsakes feed `E2.1`. |
| `INTENT2.md`, `INTENT2_MATRIX.md` | `A1`, `A3`, `A4` | `IR21` supersedes `IR14`/`IR18`/`IR20` (baseline fields only, kit derives intent tags). Feeds `A4.1`/`A4.2` and `A3.1` (consumables). |
| `HOOKS1.md`, `HOOKS1_TASKS.md` | `S1`, `S2`, `A4`, `F4`, `P2` | Built combat hooks reference (triggers, unit meters, hazards, phase deck unlock). Feeds `.1` unification passes. |
| `RULESET1.md`, `_INTENT.md`, `_TASKS.md` | `F1-F3`, `A4`, `S2` | Built timeline/round-intent foundation. Feeds `F1-F3` finalization. |
| `INPUT1.md`, `INPUT1_TASKS.md`, `INPUT2.md` | `F2`, `F3`, `X1` | Built one-flow battle input, Maneuver dash, AP pip gauge, and playback aids. |
| `ENCOUNTER2.md`, `_C.md`, `_TASKS.md` | `F4`, `I1`, `T2` | Hold escapes, phase kills, and mid-battle talk/scripted rounds (`_C.md`). Feeds `F4.1`. |
| `CAMPAIGN1.md`, `CAMPAIGN1_TASKS.md` | `W1-W3`, `E1` | World travel modes, meters, checkpoints, and triggers. Feeds Block `W`. |
| `STORYUI1.md`, `STORYUI1_TASKS.md` | `T1`, `T2`, `X1` | Vignettes, sequences, and mission boards. Feeds Block `T`. |
| `PORTRAITFX1.md`, `_TASKS.md` | `X2`, `X3` | Portrait expressions and sidecars. |
| `REGIONGROUND1_ENGINE.md`, `_KIT.md` | `X2`, `X3`, `W1` | Ground composition and region map rendering. |
| `RESCOPE1.md` | Archive context | 2026-10-19 Godot-to-Aegis transition mapping. |
