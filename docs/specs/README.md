# Specs Index and Context Rules

To prevent context bloat and outdated-spec noise in both the Design and Build sessions, `docs/specs/` holds only **active Masterplan, Engine/Kit, and Cross-Cutting contracts** (`21` files). Superseded pre-Masterplan Tier 2 specs live in `docs/archive/specs/` (and completed historical Emberwake task specs live in `docs/archive/specs/emberwake/`).

- **Build Session**: Reads **only** `docs/READY.md` and the specific spec sections named in `docs/READY.md` (`docs/sessions/*.md` and active Tier 1 specs).
- **Design Session**: Uses `docs/specs/MASTERPLAN1.md` and `docs/sessions/<SUBBLOCK>.md` as the canonical bottom-up RPG Kit specs.

## Active Contracts in `docs/specs/`

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
| `BASERULES1.md`, `_BOUND.md`, `_LIBRARY.md` | Rule register metadata (`BASERULES1-M`) and candidate behaviour catalog (`_LIBRARY.md`); stat/damage formulas live in `docs/sessions/U1_STATS.md` and `U2_CLASS.md` | active reference |
| `INTENT2.md`, `INTENT2_MATRIX.md` | Round intent framework (`IR1..IR23`, updated for `IR21` kit derivation and `S1.1` Option 1C) | active (`A1`, `A3`, `A4`, `S1`) |
| `PORTRAITFX1.md`, `_TASKS.md` | Portrait expressions, sidecars, and tool checks (`tools/portraits/`) | active (`X2`, `X3`) |
| `REGIONGROUND1_ENGINE.md`, `_KIT.md` | Ground composition and region map rendering contracts | active (`X2`, `X3`, `W1`) |

## Archived Pre-Masterplan Reference Library (`docs/archive/specs/`)

Moved to `docs/archive/specs/` on `2026-11-07` after their rules and open questions were folded into `docs/sessions/*.md`:

- `DERIVE1.md`, `ROSTER1.md` (`_TASKS.md`), `HOOKS1.md` (`_TASKS.md`), `RULESET1.md` (`_INTENT.md`, `_TASKS.md`), `INPUT1.md` (`_TASKS.md`), `INPUT2.md`, `ENCOUNTER2.md` (`_C.md`, `_TASKS.md`), `CAMPAIGN1.md` (`_TASKS.md`), `STORYUI1.md` (`_TASKS.md`), `KIT1_TASKS.md`, `RESCOPE1.md`.
- Completed historical Emberwake task specs (`S9..S14`, `BAL2..5`, `DLG1`, `EXP1`, `M4`, `REP1`, `TRAV1..2`, `UI1..3`, `FORGE1_*`, `FX1`, `STAGE1`, `SPRITE1`, `STATUSFX1`, `PLAYUI1`, etc.) live in `docs/archive/specs/emberwake/`.
