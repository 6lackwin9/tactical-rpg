# Specs Index and Context Rules

To prevent context bloat and outdated-spec noise in both the Design and Build sessions, `docs/specs/` holds only **active Masterplan, Engine/Kit, and Cross-Cutting contracts** (`22` specs plus this index). Completed or superseded specs live in `docs/archive/specs/` (and historical Emberwake task specs live in `docs/archive/specs/emberwake/`).

- **Build Session**: Reads **only** `docs/READY.md` and the specific spec sections named in `docs/READY.md` (`docs/sessions/*.md` and active Tier 1 specs).
- **Design Session**: Uses `docs/specs/MASTERPLAN1.md` and `docs/sessions/<SUBBLOCK>.md` as the canonical bottom-up RPG Kit specs.

## Active Contracts in `docs/specs/`

| File | Role | Status |
| --- | --- | --- |
| `MASTERPLAN1.md` | Canonical 9-block, 25-sub-block, 28-connection, 186-task master plan (all 25 `.1` specs closed in `docs/sessions/*.md`) | active (`.1` closed 2026-11-07) |
| `AUTOPILOT1.md` | Autopilot mode: the Design -> Build -> audit loop, automatic approval classes, guardrails, stages | draft (2026-11-17, awaits the user's approval) |
| `ASSETLIB1.md` | Kit asset library: generic sets by style, `.meta.json` sidecars, gate check (`PR7-35`) | active (2026-11-18) |
| `BIGUNIT1.md` | Units that fill 2 x 2 cells: footprint, grid rules, drawing, seeds (`PR7-12`) | active (2026-11-17) |
| `UISPLIT1.md` | Screen behaviour (view-models) apart from layout files and theme (`PR7-38`, `PR7-39`) | active (2026-11-18) |
| `STATUSVIS1.md` | Show a status where it acts (timeline, HP bar, numbers, grid, targeting) and plate priority by primitive (`AI-34` mock first) | draft (2026-11-19) |
| `BOSSUI1.md` | Boss bar: HP refilled per phase, phase markers and names, passive tags, icon-only status tabs, boss `poise_pips` (`AI-35` mock first) | draft (2026-11-19) |
| `MASTERPLAN1_ROWS.md` | Mapping of all historical task rows onto the 186 masterplan tasks | active reference map |
| `SILOS1.md` | Silo boundaries (`WORLDFIGHT1`, `SERVICES1`, `KITIDS1`) | active (Q1-Q3 closed 2026-11-07) |
| `AGENTROLES1.md` | Lane scopes, `integration` agent, `aegis scenario`, `SKILLS1` | active (`X6`) |
| `LAYOUT720.md` | 1280 x 720 relative layout grid and canvas policy | active (`X1`) |
| `LAYOUTDATA1.md` | JSON screen layout trees (`ui/layouts/<id>.json`) and `LayoutHost` | active (`X1`) |
| `TERMS1.md` | Neutral kit term register (`content.terms`) and lore-word layer check | active (`X8`) |
| `ASSETORG2.md`, `ASSETORG3.md` | Asset layer directory layout and regeneration checks (`ASSETORG4` archived after `PR5-56`) | active (`X3`) |
| `KIT1.md`, `SPLIT1.md` | Engine / Kit / Game 3-layer split contracts | done / locked (`X8`) |
| `CLEANUP.md` | Housekeeper trim index (historical passes archived) | active (`X7`) |
| `BASERULES1.md`, `_BOUND.md`, `_LIBRARY.md` | Rule register metadata (`BASERULES1-M`) and candidate behaviour catalog (`_LIBRARY.md`); stat/damage formulas live in `docs/sessions/U1_STATS.md` and `U2_CLASS.md` | active reference |
| `INTENT2.md`, `INTENT2_MATRIX.md` | Round intent framework (`IR1..IR23`, updated for `IR21` kit derivation and `S1.1` Option 1C) | active (`A1`, `A3`, `A4`, `S1`) |
| `PORTRAITFX1.md` | Portrait expressions, sidecars, and tool checks (`_TASKS.md` and `§§10-18` build notes archived `2026-11-15`) | active (`X2`, `X3`) |
| `REGIONGROUND1_ENGINE.md`, `_KIT.md` | Ground composition and region map rendering contracts | active (`X2`, `X3`, `W1`) |
| `WEBREL1.md` | Web (WASM) build of the game and its restricted itch.io deploy (`Post-R4`; engine seam, game entry, CI) | active (`.1` 2026-11-09; feasibility check pending) |
| `ACTS23.md` | Survey of the systems Acts 2-3 need against the code, 8 work packages (`PK1`..`PK8`), questions `Q-A1..Q-A10` | answered (2026-11-10; Channel rework 5.1 closed) |

## Archived Pre-Masterplan Reference Library (`docs/archive/specs/`)

Moved to `docs/archive/specs/`:

- `2026-11-07`: `DERIVE1.md`, `ROSTER1.md` (`_TASKS.md`), `HOOKS1.md` (`_TASKS.md`), `RULESET1.md` (`_INTENT.md`, `_TASKS.md`), `INPUT1.md` (`_TASKS.md`), `INPUT2.md`, `ENCOUNTER2.md` (`_C.md`, `_TASKS.md`), `CAMPAIGN1.md` (`_TASKS.md`), `STORYUI1.md` (`_TASKS.md`), `KIT1_TASKS.md`, `RESCOPE1.md`.
- `2026-11-15`: `PORTRAITFX1_TASKS.md`, `PORTRAITFX1_BUILD_NOTES.md`, `ASSETORG4.md`, `REVEAL1.md` (with `REVEAL1` indoor/roof/boundary rules folded into `games/emberwake/docs/specs/REGIONGROUND1.md`).
- Completed historical Emberwake task specs live in `docs/archive/specs/emberwake/`.
