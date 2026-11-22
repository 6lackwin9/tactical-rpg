# Kit Sample data

The kit's data folder layout (`catalog/` and, for a game with worlds, `worlds/`), read by `rpgkit_world::Data::open`.
Edit it only through `kit_sample_cli content` (upsert, remove, apply), never by hand. With no terrain rows the kit's
code defaults apply (the content check warns once per terrain); with no `progression/xp_curve.json` the kit's default
curve applies.
