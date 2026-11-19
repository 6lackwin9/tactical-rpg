# assets/audio/ambience (Emberwake)

The game's ambience beds for the `ambience` bus (AMBIENCE1, `PR7-34`; the Sound tab's Ward Ambience volume):
`<music key>.ogg` (Ogg Vorbis), the same keys as `audio/music/`. The game plays the first key of a context's music
chain with a file (`<world>_<context>` before the kit's `<context>`, `kits/rpg/assets/audio/ambience/`), so a world,
or a location scope, gets its own bed by adding its file, no code.

All are `[PROVISIONAL synth placeholder]` beds (AI-30) written by `tools/gen_music.py --ambience` (48 s seamless loops,
RMS -26 dB, peaks under -9 dB, no melody); final beds may come from the audio pipeline later. Worlds other than the
Sunk Ward use the kit's beds until they get their own.

| Key | Plays on | Bed |
| --- | --- | --- |
| `sunk_ward_town` | the Sluice Undercroft hub | a crackling hearth (a soft roar under pops), five low wordless murmurs, a far drip |
| `sunk_ward_field` | the ward's streets above ground | wind through the rows, a far D3 bell tolling three strokes twice a loop, a loose shutter |
| `sunk_ward_travel` | the Sunk Ward world map | a lighter wind, one far toll per loop |
| `sunk_ward_danger` | the drains, ruins and camps (`ruin`, `camp`, `dungeon` maps) | a low water rush, slow drips in a wet echo, a far pipe groan three times a loop |
