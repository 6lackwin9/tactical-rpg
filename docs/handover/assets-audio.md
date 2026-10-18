# Assets & Audio handover

Written 2026-10-18 05:40 UTC by the sixth Assets & Audio session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Dispatch mode is economy (asset-builder low/sonnet, change-reviewer low).

## Queue (in order)

1. ITEMUSE1-A (`use_reach` on the three consumables) after Gameplay's ITEMUSE1-L lands (Gameplay pings).
2. Then any new `-A` rows the coordinator relays.

Live peers: Gameplay session_01HQvENsjr5dFTBZp6L6GkPU, UI/UX session_01EFGiKxfL6TjsoGEDnQG5hc, Balance session_012hWXrrHGj1VxCBHAEN59mo.

## Landed on `develop` (this session)

- INTENT1-A (4205bc1): passive `wardens_eye` (foresight) on Bryn. Seed `data/seeds/intent1_a.gd`.
- BOSS1-A (1303031): Crown of Nails on both Ashmaw stands, Empty Gorge line on the barred one. Seed `data/seeds/boss1.gd`.
- OBJ1-A (bda1aa0): courtyard hold, `wagon_raid` with Wenn Tallow (ambush, protect), slay Grakka on both stands, Wenn placeholder art. Seed `data/seeds/obj1.gd`.

## Notes for the next session

- New shipped catalog rows: upsert inside `ContentCatalog.begin_system_edit()` / `end_system_edit()` with `"origin": "system"` (known-issues row; origin_test_suite fails otherwise).
- Run seeds with `tools/fc.sh run <seed>` (plain `godot -s` lacks the art options); `save_world` writes schema defaults onto untouched rows, which is fine.
- Ashlands encounter `party` lists stay Mara/Tev: the campaign party comes from `start_party`, and tidewrack re-skins those two in `world_test_suite`.
- wagon_raid's ambush puts 1 of its 3 raiders behind the party (roster rule); the spec acceptance said two. Told Gameplay.
- A new character needs a `CHARACTERS` row in `tools/gen_art.py` (logic-builder), then `gen_art.py characters`, `godot --headless --import`, `tools/fc.sh run data/seeds/art1_a.gd`. Never commit `tools/__pycache__/`.
- Earlier notes hold: gate with `SUITE_TIMEOUT=600 tools/fc.sh gate`; never `pkill -f godot`; world seeds use `WorldCatalog.save_world(<world>)`.
- Branches to delete at the next cleanup (landed): `claude/INTENT1-A-wardens-eye`, `claude/BOSS1-A-crown-of-nails`, `claude/OBJ1-A-objectives`, `claude/project-thread-0lygm1`.
