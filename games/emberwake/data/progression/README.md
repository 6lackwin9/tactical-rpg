# data/progression

System: S7. Status: done (M3).

`xp_curve.json`: `xp_to_next` (MAX_LEVEL - 1 positive ints) and `kill_xp` (base, per_level, tier multipliers for every `StatRules` tier, level-delta step/min/max). Validated by `XpCurve.validate`; an invalid file falls back to defaults. Starting values for BAL1. Edit the file directly (it is not a catalog kind).
