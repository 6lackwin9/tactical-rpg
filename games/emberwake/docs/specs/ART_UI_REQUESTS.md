# Art/UI requests (inbox and outbox)

The hand-off file between the Artwork & Mock UI Session and the Design Session (decision "Art/UI request outbox and ownership", user 2026-11-10). Build never reads it; Design turns outbox rows into `docs/READY.md` items.

- **Inbox** (Design writes, Art/UI works): tasks for the Art/UI session. At its session start Art/UI reads the inbox, does the tasks, and **deletes each row once done**, logging the result in `ART_UI_STUDIO.md`.
- **Outbox** (Art/UI writes, Design triages): requests from Art/UI for Build or Design. At its session start Design queues or answers each row in `docs/READY.md` and **deletes the row once it is recorded there** (READY Section 4 keeps the answer with the request id).

## Inbox (for Art/UI)

| Id | Task | Why / blocks | Files | Status |
| --- | --- | --- | --- | --- |
| AI-01 | Bake the `warrior/2h_polearm` look (clips, palettes, the base's side and view clips, as `2h_hammer`, `2h_sword`, `2h_axe`) | The class weapon lineup gives the Vanguard the polearm as its main weapon, held two-handed; `PR5-18` `WEAPONROSTER1` is parked until the look exists (`FB-185`) | `games/emberwake/assets/units/warrior/2h_polearm/` | open |
| AI-02 | Remove `menu_theme` from `TRACKS` in `tools/gen_music.py` (line 546), or retire the script once Flow replaces the synthesized loops | Running it with no arguments overwrites the shipped Flow menu theme; the script is Art/UI's now | `tools/gen_music.py` | open |
| AI-03 | Produce the slice's story tracks: `story_opening` (the opening vignette), `prologue_tension`, `story_escape`, `story_goodbye` | `PR5-24` `STORYMUSIC1` adds the hook; the tracks are Art/UI's | `games/emberwake/assets/music/` | open |

## Outbox (for Design)

| Id | Request | Files | Status |
| --- | --- | --- | --- |
