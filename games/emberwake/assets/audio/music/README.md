# assets/audio/music

The game's music (S14, MUSIC1): `<key>.ogg` (Ogg Vorbis): `menu_theme` and the world tracks; the mood loops (`combat`, `boss`, `town`, `shop`, `danger`, `travel`) are the RPG kit's (`kits/rpg/assets/audio/music/`, ASSETORG2-K). The engine plays `music/<key>.ogg` directly (PORT-AUDIO), so a new track needs only its file; it is found over the asset layers (ASSETORG2: the game's, the kit's, the engine's `audio/`).

`menu_theme` (since 2026-11-11) is a Flow Music track (Lyria 3.5), imported by `python -m tools.mock_ui.music` as the whole song with a 2.5 s pause before it restarts; its prompt and import record is `art_src/music/menu_theme.json` (ART_UI_STUDIO 3.9). `sunk_ward_town` (the hub; MUSIC1's `town` context for the Sunk Ward) is a Flow Music cover of it, record `art_src/music/sunk_ward_town.json`. `sunk_ward_combat` (MUSIC1's `combat` context for the Sunk Ward) is a 48 s section loop of a Flow Music battle take's bass and other stems, record `art_src/music/sunk_ward_combat.json`. `sunk_ward_combat_2` and `sunk_ward_combat_3` are its variants (whole songs with several stages; records beside it); they play only once the engine picks among a key's `<key>_N` variants, a request in ART_UI_STUDIO 3.9. `tools/gen_music.py` no longer has a `menu_theme` key (AI-02, 2026-11-12), so a run with no arguments cannot overwrite it.

The other loops (`ashlands`, `iron_reaches`, `tidewrack`, `combat`; MUSIC2: `town`, `danger`, `shop`, `travel`, `boss`) plus the Soot-Gothic `[PROVISIONAL synth placeholder]` tracks added via `py -3.12 tools/gen_music.py` (numpy + ffmpeg libvorbis circular buffers with zero loop seam) cover the story cues and remaining Sunk Ward contexts until Flow replacements land:

| Key | Plays on | Key / BPM | Length | Status / Notes |
| --- | --- | --- | --- | --- |
| `story_opening` | `vg_open` ("The Red Tithing") | D minor, 56 BPM (6/8) | 102.9 s | `[PROVISIONAL synth placeholder]` — solemn cradle choir pad, hand bell, low string drone, no drums (`story_tracks.txt`) |
| `prologue_tension` | Prologue steps (no vignette wired yet) | D minor, 84 BPM (4/4) | 91.4 s | `[PROVISIONAL synth placeholder]` — uneven plucked low strings, muted frame drum pulse, broken dulcimer motif, distant iron shutter |
| `story_escape` | `vg_meat_market` (escape for the drains) | D minor, 132 BPM (4/4) | 101.8 s | `[PROVISIONAL synth placeholder]` — staccato low strings, off-beat frame drums & anvil chains, double-time Scarlet Lullaby lead |
| `story_goodbye` | `vg_m5_gate`, `vg_debate_3` | D minor -> F major, 60 BPM (6/8) | 112.0 s | `[PROVISIONAL synth placeholder]` — plain music-box Scarlet Lullaby, answering low horn/viola line, distant high double on second pass |
| `sunk_ward_danger` | Sunk Ward `ruin`, `camp`, `dungeon` maps | C Phrygian, 72 BPM (4/4) | 53.3 s | `[PROVISIONAL synth placeholder]` — damp drain wash, metallic pipe drips, muted heartbeat pulse, tense low strings |
| `sunk_ward_boss` | Sunk Ward boss battles | F minor, 136 BPM (4/4) | 56.5 s | `[PROVISIONAL synth placeholder]` — organ/choir chord bed, pounding frame drums, anvil clangs, stark cathedral bell & horn calls |
| `sunk_ward_shop` | Sunk Ward shop overlays | D minor, 96 BPM (3/4) | 45.0 s | `[PROVISIONAL synth placeholder]` — candlelit sparse lute waltz, glass-vial chimes, quiet guarded whistle tune |
| `sunk_ward_travel` | Sunk Ward world / transit travel | D Dorian, 88 BPM (4/4) | 65.5 s | `[PROVISIONAL synth placeholder]` — rain-slick damp cobble pulse, horn & distant whistle soot march |

`MusicDirector` picks the track (`games/emberwake/docs/specs/MUSIC1.md`); a region gets its own track by adding `<region>.ogg` or `<scope>_<context>.ogg`.

The ambience beds that play under the music on the `ambience` bus (AMBIENCE1, AI-30) use the same keys and live beside
this folder in `audio/ambience/` (its README lists them).
