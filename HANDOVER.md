# Handover (2026-11-20)

For a fresh agent, on any platform, that has only this repository. It says what the project is, how the owner works,
where everything lives, and what is open as of develop `ba24f109` (2026-11-20). For live state, read the files it
points to, not this snapshot.

## 1. The project and the owner

- **Game**: Emberwake, a tactical RPG (timeline battles, hub, regions, story) on **Aegis**, the owner's own Rust
  engine. Owner: Tushar (GitHub `6lackwin9`). Web build of `main`: <https://6lackwin9.itch.io/tactical-rpg>.
- **Goal**: mostly automated development. Tushar's playtest feedback is the main manual input. Also welcome: work
  that strengthens the Aegis engine even if the game does not need it yet.
- **Plan now**: finish the UI mocks so every screen is playable, then Tushar plays by hand for feel, input and
  responsiveness.
- **Lore and assets**: current lore stands but may change after his story playtest. Do not lock character looks or
  portraits yet; keep backstories. Work on placeholders. Core systems should make lore revisions fast.

### How Tushar wants you to work

- Bring decisions as options **with a recommendation**. He often answers "all recs" in one typed line.
- Small calls: make the lowest-undo-cost option that fits `docs/PRINCIPLES.md`, mark it `[PROVISIONAL]`, log it in
  `docs/DESIGN_REVIEW.md`, and list major ones in `games/emberwake/docs/NEEDS_YOUR_EYES.md` ("For the next session").
- A decision recorded in the repo needs a typed message from him, not a button tap.
- Ask first only on irreversible steps, `[LOCKED]` canon conflicts, and golden re-records.
- **Never force-push `main` or `develop`.** `main` is always a clean build.
- **Releases (develop -> main) only when he says "release now".**
- Mode in force is **interactive** (`CLAUDE.md`): act on his direct requests. "Build: next row" builds one READY row;
  "Build: run queue" builds up to 3 section 1 rows in order (stop rules in `docs/specs/AUTOPILOT1.md` section 6);
  he has also asked runs to chain back to back, parking rows that need his sign-off, with a short summary per run.
- He uses speech to text (for example "grid" may mean "crit"). Keep replies short and in plain sentences.

## 2. Architecture (read `CLAUDE.md` first; it is authoritative)

| Layer | Path | Rule |
| --- | --- | --- |
| Engine | `engine/` (crates `aegis_*`) | Game-agnostic; never names a game. `engine/README.md` |
| RPG kit | `kits/rpg/` (crates `rpgkit_*`, `schema.json`) | Genre rules; names no game |
| Game | `games/emberwake/` (crates `emberwake_*`, `data/`, `assets/`, `tuning.json`, `game.json`, `docs/`) | Façades over the kit plus game code |
| Sample | `games/hello/` | Engine-only sample game |
| Legacy | `legacy/` | Frozen Godot 4.7 build. Never edit, never build, not a test oracle |

- Gate (cloud/Linux): `tools/fc.sh gate` (desktop: `check_project.ps1 -Fast -Quiet`; both run `tools/gate_steps.txt`).
  Docs check: `tools/fc.sh docs`. Both must be clean before pushing to `develop`.
- Data changes only through `aegis content` (`tools/fc.sh content`), never raw JSON edits.
- Goldens (`CLAUDE.md` "Engine goldens") are re-recorded only for an intended change, named in the commit and a
  `docs/HISTORY.md` row, and with Tushar's word. Never edit a golden to make a test pass.
- No compiler warnings. Docs history is append-only.

## 3. How work flows (roles)

Three roles, originally separate sessions; on a single-agent platform play them one at a time and keep their files
apart (`CLAUDE.md` "Two-session split"):

- **Design**: writes `docs/READY.md` (the build queue and answers to feedback), specs, decisions, `NEEDS_YOUR_EYES.md`.
  Entry: skill `design-session` (`.claude/skills/`).
- **Build**: builds `ready-to-build` rows from `docs/READY.md` section 1 in order, writes status and questions only to
  `docs/FEEDBACK.md`. Rows tagged `[lane: tools]` were a second Build lane (tools and engine). Entry: skills `planner`,
  `planner-ops`, `build-session`; agents in `.claude/agents/`.
- **Art/UI**: mocks and layouts in `tools/mock_ui/`, `art_src/`, `games/emberwake/assets/ui/layouts/*.json`; exchanges
  requests with Design via `games/emberwake/docs/specs/ART_UI_REQUESTS.md` (Inbox/Outbox). Entry: skill
  `art-ui-session`.

Branches: work on a feature branch, land on `develop` after a clean gate (cloud threads pushed straight to `develop`
or via PR), release to `main` by PR only on "release now". Environment notes: `docs/workflow/cloud.md`,
`docs/workflow/desktop.md`, `docs/workflow/README.md`.

## 4. Where things live

| What | Where |
| --- | --- |
| Build queue + Design answers | `docs/READY.md` (section 1 ready, 3 held, 4 answers) |
| Build status, questions, deltas | `docs/FEEDBACK.md` (closed items in `docs/archive/FEEDBACK_CLOSED.md`) |
| What only Tushar can judge | `games/emberwake/docs/NEEDS_YOUR_EYES.md` |
| Provisional calls for his review | `docs/DESIGN_REVIEW.md` |
| Art/UI requests | `games/emberwake/docs/specs/ART_UI_REQUESTS.md` |
| Decisions | `docs/DECISIONS.md` index; `docs/decisions/workflow.md`; `games/emberwake/docs/decisions/*.md` |
| Specs | `docs/specs/` (index `docs/specs/README.md`), game specs `games/emberwake/docs/specs/` |
| Screen contracts, mocks | `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md`, `SCREEN_ATLAS.md`, `ART_UI_STUDIO.md` |
| Lore | `games/emberwake/docs/lore/CANON.md` |
| His preferences | `games/emberwake/docs/preferences/` |
| Code map | `docs/STRUCTURE.md`, `games/emberwake/docs/GAME_BRIEF.md` |
| Audits | `docs/audits/` |
| Balance | `games/emberwake/docs/BALANCE_REPORT.md` |
| Tried and dropped ideas | `docs/HISTORY.md` |
| Known failures and fixes | skill `known-issues` |
| Patch notes | `PATCH_NOTES.md` |

Outside the repo (not reachable from a fresh platform unless he shares them): mock images and playtest reports were
kept in the old project's shared files (`art-ui/`, `playtests/`); the image pipeline is in his Google Drive.

## 5. Art rules

- **Style locked 2026-11-17**: palette P3 "Tallow and Iron", frame F2 (thin iron frame, oxblood top line), sprites S2
  (3-tone), icons I2 (slate). Main screens use the ornate frame; popups and HUD use the double line (A).
- Design every UI for **720p at 1:1**.
- **Gemini outputs are placeholders** for anything game-specific (characters, CGs, regions). Only generic kit-library
  assets (tilesets, props, skill and gear icons) may be approved straight from Gemini. Placeholders may also go into
  the kit asset library (`kits/rpg/library/`) stripped of game references. **Final character art via Flow**, later.
- Region maps: reference image from the design, never from a render; then mirror it with terrain tiles and props.
- **Image pipeline**: Google Drive `My Drive/image_pipeline/` (`queue/`, `refs/`, `output/`). He says "process image
  queue" in Gemini with everything in `refs/active` attached.
- **LFS art** is pushed from his PC checkout `C:\dev\tactical-rpg` (cloud containers could not push LFS art).
- A feature that needs missing art lands with a placeholder at the final path (decision "Placeholder-first assets").
- After generating or editing any image, view it and check it against the prompt and reference before accepting.

## 6. State on 2026-11-20

- `main` = release 2026-11-19 (PR #72, `9b221a04`). `develop` is 59 commits ahead (PR7-45..61 and Design
  answers). **No release PR is open**; release only on "release now". The release CI fix (ENGINE-20, seed order on a
  shallow clone) landed in that release.
- No open PRs; the only remote branches are `main`, `develop` and this handover branch.
- **ENGINE-12** (gate parity Windows/Linux): **closed**, the PC gate passed on develop `ec53813` (15 steps, 3952 tests).
- CI: the develop `gate` workflow is green through `97689631`. The scheduled `bands` workflow (balance bands) on
  `main` **failed** on 2026-11-20 11:36 UTC; not yet looked at.

### Open queue (`docs/READY.md` section 1, not marked done)

Main lane (game UI and content), in order: PR7-21 settings mock port, PR7-26 playtest UI fixes, PR7-15 HUD frames
and zoom, PR7-24 hub polish, PR7-23 hub layout, PR7-43 bunks layout B, PR7-25 style engine (S2 ramps, F2 portraits),
PR7-37/41 playtest fixes, PR7-42 pacing, PR7-36 style pass, PR7-40 battle screen to contract, PR7-44 and PR7-49
controls tab, PR7-46 item 3, PR7-47, PR7-48, PR7-50..53 and PR7-58 bunks and footlockers, PR7-27 VN stage, PR7-32
battle report UI, PR7-18 facility effects, PR6-12/13 Act 2 and 3 content, PR7-16, PR7-17, PR7-20 balance tune.
Tools lane: PR7-22 agent playtest driver, PR7-38/39 UI split, PR7-62 intercept limit (next), PR7-33 floating panels,
PR7-34 ambience bus, PR7-35 kit asset library, PR7-4 part 2 screenshot baselines.

Several of these may be partly built: always check the row's text and `docs/FEEDBACK.md` section 1 before starting.
Open feedback items: FB-334, FB-346, FB-354, FB-374.

### Waiting on Tushar (full list in `NEEDS_YOUR_EYES.md`)

- **Golden re-records**: auto-plan fixes (PR7-41 item 6, PT-22) and the Crucible planner fixes (PR7-60, behind a
  test switch); boss poise data (PR7-54 BU9). Recommendation: approve together.
- **Fight length** (PR7-55, FB-377/381): random fights end in 1 round vs his 3; rec two more foes per random
  encounter. Elites: keep targets, measure planning time. Bosses: ease the Crucible Overlook.
- **5-round cap on random encounters** (`[PROVISIONAL]` from his "5 or 6", decision "Random encounters end at a
  short cap") and the enrage balance misses (`ENGINE-26` options) wait for his confirmation.
- **Provisional calls to confirm**: enemy short names on foe plates (PR7-40), pacing numbers (PR7-42), FB-346
  bisect list, screenshot baseline approval (PR7-4 part 2), gamepad feel on the PC (FB-327).
- **Mocks to sign off**: battle take 22 (boss) and take 23 (timeline and playback); six screens: victory report,
  story, character, shop, notice board, saves (`screens_ai21`).
- **Party token size on maps** (FB-354): rec **D**, redraw tokens at 32 px.
- **Image queue**: say "process image queue" in Gemini.
- Older playtest checklists in `NEEDS_YOUR_EYES.md` (battle view, walking and turns, windows).

### Recent decisions (rows in `docs/decisions/` and `games/emberwake/docs/decisions/`)

- **Boss phase break ends the round** (2026-11-19): no damage carries over; a cutscene hook plays between phases
  (spec `docs/specs/BOSSUI1.md` BU15; built PR7-56).
- **Boss intent**: two rows, the selected hero and the party ("Others", every member on hover), HP, poise and status
  per block; poise bar style A; party row P1 (built PR7-54, PR7-59). Fallback if too busy in play: totals only.
- **Status shown where it acts** (take 33 with take 34 changes, `docs/specs/STATUSVIS1.md`): stagger locks blocks 1
  and 4, stun locks the first 2; status tabs are icons only, counts in the expanded view (built PR7-57).
- **Passives**: two fixed loadout slots, always on, shown as two small icons apart from the status column
  (STATUSVIS1 SV10, SV13).
- **Bunk room take 8 signed off** (2026-11-19): arch and peg board, lunette founder emblems, footlockers closed by
  default, open while editing loadout, hidden when locked; Act 2 bunk spots hold props until the upgrade (PR7-50..53).
- **UI split** (2026-11-18): screen behaviour (Build), layout files (Art/UI), theme (`docs/specs/UISPLIT1.md`).

## 7. How to resume

1. Clone `https://github.com/6lackwin9/tactical-rpg`, check out `develop`. Toolchain from `rust-toolchain.toml`.
2. Read `CLAUDE.md`, this file, `docs/READY.md` section 1, `games/emberwake/docs/NEEDS_YOUR_EYES.md`.
3. Run `tools/fc.sh gate` once to confirm a clean baseline (about 20 minutes).
4. Answer the waiting items above, then say "Build: next row" (or "run queue").

### Starter prompt (paste into the new tool)

```text
You are continuing my tactical RPG project in the repo 6lackwin9/tactical-rpg (branch develop).
Read HANDOVER.md and CLAUDE.md first; CLAUDE.md is the authoritative rule book. Then read
docs/READY.md section 1 and games/emberwake/docs/NEEDS_YOUR_EYES.md.
Work style: I mostly give playtest feedback; you drive. Bring decisions as options with a
recommendation. Make small calls yourself as [PROVISIONAL] and log them in docs/DESIGN_REVIEW.md.
Never force-push main or develop; push to develop only after tools/fc.sh gate and tools/fc.sh docs are
clean; release develop to main only when I say "release now". Never edit legacy/ or a golden to make a
test pass; data only through aegis content.
Start by telling me in a few lines: what is waiting on me, what the next READY row is, and anything
broken in CI. Then wait for my go.
```
