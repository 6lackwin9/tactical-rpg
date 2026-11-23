# kits/rpg/assets/audio/sfx

The RPG kit's combat sounds (S14; ASSETORG2-K moved them here from the game): `<key>.wav`. The interface clicks (`ui_*.wav`) are the engine's (`engine/assets/audio/sfx/`). The engine plays `sfx/<key>.wav` directly (PORT-AUDIO: battle sounds from the resolver log, playback-control clicks, volume samples); it is found over the asset layers (ASSETORG2: the game's, the kit's, the engine's `audio/`).

ART2-A: `miss.wav` (60 ms soft noise whoosh, SoundEvents key `miss`).

AI-43: `map_step.wav` (85 ms soft step on stone: a low-passed heel thud with a faint scuff and a softer toe 28 ms after, peak -20 dBFS, dark so it does not tire over a long walk; mono 22.05 kHz; the map footstep, ENGINE-56, at the Footsteps volume). One file: the map plays a single fixed path, so no variants.
