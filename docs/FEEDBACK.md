# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to PR4 rows (R1 to R4 releases, the Post-R4 queue PR4-1..PR4-9) | released or done | `main` / `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-1 to PR5-6, PR5-8, PR5-16, PR5-17, PR5-20 to PR5-22 | done (FB-168 to FB-189, answered) | `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-7 `PK1-S3` | done (FB-190 data; FB-191 balance run, 6 findings for Design) | `develop` | -
| PR5-18 `WEAPONROSTER1` | done (FB-192; FB-193 sims and questions): class lineup, default loadouts, placeholder `warrior/2h_polearm`, FB-188 rulings; `phasefold_golden` re-recorded | `develop` | -
| PR5-25 `SKILL-UNMARK-TURNED` | done (FB-194): a SUPPORT skill's `unmark` reaches and calms a turned party member; the item `unmark` under round_intent too; 0 golden diffs | `develop` | -
| PR5-23 `MUSICVAR1` | done (FB-195): music variants per request (seeded, never the last pick), Ogg loop points; 0 golden diffs | `develop` | -
| PR5-24 `STORYMUSIC1` | done (FB-196): `music` on vignettes and sequence steps, keys from `game.json` `music_keys`; 0 golden diffs | `develop` | next: PR5-26, PR5-27, PK2 rows

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-195 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-196. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-196: PR5-24 `STORYMUSIC1` landed (story steps and vignettes ask for music; the game's `music_keys` in `game.json`)

- **Spec**: READY Section 1 row PR5-24; ART_UI_STUDIO 3.9 request 3 (the `PR5-24` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: A vignette's `music` and a new optional `music` on every sequence step kind read the same way (`rpgkit_data` `MusicAsk`, `music_ask`): `""` keeps the music, `none` is silence, a path ending `.ogg` plays as written, any other value is a music key. The content check (`check/story.rs` `music_error`) accepts a key the game lists in `game.json` **`music_keys`** (new optional manifest key, read like the purse caps: `source::MUSIC_KEYS`, in `MANIFEST_KEYS`, `music_keys_problems` refuses a non-list, a non-id or a key twice; `View::music_keys`) or a key whose `audio/music/<key>.ogg` the asset layers hold; an unknown or non-id key is one error, a missing `.ogg` path a warning. Emberwake's `game.json` lists its 9 MUSIC1 contexts and the story keys `story_opening`, `prologue_tension`, `story_escape`, `story_goodbye` (set with `tools/fc.sh content manifest`); the kit names none of them (review M1: an earlier draft held both lists as kit constants). The game resolves a key through its music chain (`music::game_track`: region-scoped file first, then `<key>.ogg`; MUSICVAR1 variants apply); with no track the music goes on. On the story stage a step's music becomes the stage's music when the stage reaches it, before its vignettes (`StoryStage::take_step_music`); an `effects` step's music carries to the next step the stage plays (`rpgkit_world::story::step_music`: the nearest one wins); a vignette's own music still wins while it plays; a `retry` fight does not re-apply it. Tests: `rpgkit_world/tests/story_music.rs` (2), `rpgkit_content/tests/story_checks.rs` (+2: the manifest key, keys or tracks), `emberwake_gui` story test (+1, with `game.json`'s list holding every `music.rs` context). Gate 2931 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (M1 layer: fixed by the manifest key; L1 a tautological test expectation: fixed).
- **Recommendation**: confirm the new `game.json` key. GAME_BRIEF (manifest keys / story music): "`game.json` `music_keys` (optional, STORYMUSIC1): the music keys a vignette's or a sequence step's `music` may name without a track: Emberwake's 9 MUSIC1 contexts and `story_opening`, `prologue_tension`, `story_escape`, `story_goodbye`; set with `aegis content manifest`; without it only a shipped `audio/music/<key>.ogg` counts. A step's music holds from that step on on the story stage (an effects step's from the next step played); a vignette's own music wins while it plays." STRUCTURE: test `kits/rpg/crates/rpgkit_world/tests/story_music.rs`. Art/UI (music README, ART_UI_STUDIO 3.9): story tracks go in `games/emberwake/assets/audio/music/story_opening.ogg` (and the other three), or a region-scoped `<region>_story_opening.ogg` (AI-03 names `assets/music/`, but the player reads `assets/audio/music/`); data wiring sets `music` on the vignette row or the step. `kits/rpg/README.md`: done. Notes (built defaults): loading a save mid-sequence starts from the menu theme plus the current step's own or carried music (music from an earlier visible step is not restored); `music` on `enter` and `end` steps has no audible effect, and on a `fight` step it is the music after the fight; the story keys have no parent context (no track: the music goes on).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
