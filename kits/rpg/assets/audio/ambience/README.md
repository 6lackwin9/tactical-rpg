# assets/audio/ambience (RPG kit)

The kit's ambience beds for the `ambience` bus (AMBIENCE1, `PR7-34`): one seamless 48 s loop per music context,
`<context>.ogg` (Ogg Vorbis), under the music keys. A game plays the first key of a context's music chain that has a
file here or in its own `audio/ambience/` (ASSETORG2 layers: the game's first), so a game or a world gets its own bed
by adding `<scope>_<context>.ogg`, no code.

All four are `[PROVISIONAL synth placeholder]` beds written by `tools/gen_music.py --ambience` (numpy and ffmpeg
libvorbis, circular buffers with no loop seam), levelled low and even (RMS -26 dB, peaks under -9 dB) to sit under the
music; no melody.

| Key | Context | Bed |
| --- | --- | --- |
| `field` | maps outdoors | wind that swells and drops, a loose shutter knocking now and then |
| `town` | a hub or town | a soft hearth roar with pops, low wordless murmurs across the room, a far drip |
| `travel` | the world map | a lighter wind, the shutter |
| `danger` | ruins, camps, dungeons | a low water rush, slow drips in a wet echo, a far pipe groan |
