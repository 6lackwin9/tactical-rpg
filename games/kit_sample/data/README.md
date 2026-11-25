# Kit Sample data

The kit's data folder layout (`catalog/` and, for a game with worlds, `worlds/`), read by `rpgkit_world::Data::open`.
Edit it only through `kit_sample_cli content` (upsert, remove, apply), never by hand. A terrain type with no row takes the
kit's code defaults quietly (ENGINE-43); with no `progression/xp_curve.json` the kit's default
curve applies.
