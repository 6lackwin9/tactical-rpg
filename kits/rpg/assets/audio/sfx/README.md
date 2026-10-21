# kits/rpg/assets/audio/sfx

The RPG kit's combat sounds (S14; ASSETORG2-K moved them here from the game): `<key>.wav`. The interface clicks (`ui_*.wav`) are the engine's (`engine/assets/audio/sfx/`). The engine plays `sfx/<key>.wav` directly (PORT-AUDIO: battle sounds from the resolver log, playback-control clicks, volume samples); it is found over the asset layers (ASSETORG2: the game's, the kit's, the engine's `audio/`).

ART2-A: `miss.wav` (60 ms soft noise whoosh, SoundEvents key `miss`).
