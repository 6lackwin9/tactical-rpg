# Art/UI requests (inbox and outbox)

The hand-off file between the Artwork & Mock UI Session and the Design Session (decision "Art/UI request outbox and ownership", user 2026-11-10). Build never reads it; Design turns outbox rows into `docs/READY.md` items.

- **Inbox** (Design or the user writes, Art/UI works): tasks for the Art/UI session; the user adds art requests here directly (or asks the Design Session to), and the file stays in use as the standing art request queue. At its session start Art/UI reads the inbox, does the tasks, and **deletes each row once done**, logging the result in `ART_UI_STUDIO.md`.
- **Outbox** (Art/UI writes, Design triages): requests from Art/UI for Build or Design. At its session start Design queues or answers each row in `docs/READY.md` and **deletes the row once it is recorded there** (READY Section 4 keeps the answer with the request id).

Placeholders (decision "Placeholder-first assets"): when Build lands a feature with a placeholder asset (a copy at the final path), an Inbox row asks Art/UI to replace that file **in place**; keep the path, format and frame layout so nothing needs rebuilding (an integration pass checks the replacements later).

## Inbox (for Art/UI)

| Id | Task | Why / blocks | Files | Status |
| --- | --- | --- | --- | --- |
| AI-01 | Repaint the **placeholder** `warrior/2h_polearm` look (Build copies it from an existing warrior two-handed look in `PR5-18`) with real polearm art, in place (same files: clips, palettes, side and view clips), plus any other placeholder look `PR5-18` reports in `FEEDBACK.md` | The Vanguard's main weapon is now the polearm, held two-handed; the placeholder keeps the art checks green until the real art lands, and no setup changes are needed | `games/emberwake/assets/units/warrior/2h_polearm/` (and any other reported placeholder) | open |
| AI-02 | Remove `menu_theme` from `TRACKS` in `tools/gen_music.py` (line 546), or retire the script once Flow replaces the synthesized loops | Running it with no arguments overwrites the shipped Flow menu theme; the script is Art/UI's now | `tools/gen_music.py` | open |
| AI-03 | Produce the slice's story tracks: `story_opening` (the opening vignette), `prologue_tension`, `story_escape`, `story_goodbye` | `PR5-24` `STORYMUSIC1` adds the hook; the tracks are Art/UI's | `games/emberwake/assets/music/` | open |

## Outbox (for Design)

| Id | Request | Files | Status |
| --- | --- | --- | --- |
