# Assets & Audio handover

Written 2026-10-18 08:45 UTC by the seventh Assets & Audio session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md` ("Usage caps": Assets & Audio builds only while the coordinator gives it the rotating slot; build lanes run in economy mode; rotate by context size, about 120k tokens). Decision "SONNET-TRIAL" (a499b3f): this area runs on Sonnet as a trial.

## Queue

1. HEAL1-A (unblocked by HEAL1-L schema 44e0641, Gameplay 08:27): `rest: true` on ashlands:warden_keep and iron_reaches:slag_gate via seed `data/seeds/heal1.gd` (`docs/specs/HEAL1.md` section 6). Waits for the rotating slot.

The slot was handed back to the coordinator at 08:45. Queue new approved `-A` rows the coordinator or peers relay until you hold the slot again.

Live peers: Gameplay session_017rPg9X4j3FhvPDo3t93bwo, UI/UX session_01Rs6SLjohzJBiLs4ezyKKBN, Balance session_012hWXrrHGj1VxCBHAEN59mo, coordinator session_015sTvb6o2FYckV681cWHXtS.

## Landed on `develop` (this session)

- ART2-A (0218602): intent kind chips `ui/intent_<kind>` (one per `IntentRules.KINDS`), `badge_foresight`, `chip_slay`, `chip_guest`, tile `overlay_objective`, sfx `miss` (already mapped, now audible). Generator group `chips` in `tools/gen_art.py`, seed `data/seeds/art2_a.gd`. UI/UX uses them in INTENT1-U and OBJ1-U.
- ITEMUSE1-A (3048364): `use_reach` potion 1, elixir 0, bomb 3. Seed `data/seeds/itemuse1_a.gd`.
- ITEMUSE1-A2 (fda40cc): class `pouch` rows emptied (decision "No free class potion"). Seed `data/seeds/itemuse1_a2.gd`. Logic follow-ups (bal2_rules test now compares [] to [], stale comments) sent to Gameplay.

## Notes for the next session

- The Ashlands placeholder audit (`/mnt/project-files/ashlands-art/AUDIT.md`) is fully built; new art comes only from new specs or UI rows.
- The arena suite now runs past 600 s: gate with `SUITE_TIMEOUT=1200 tools/fc.sh gate > file` as a background command.
- `ContentCatalog.save_all()` renormalises `data/catalog/encounters.json`: revert that file after running a catalog seed.
- Shipped catalog rows: upsert inside `begin_system_edit()` / `end_system_edit()` with `"origin": "system"`. Run seeds with `tools/fc.sh run <seed>`; commit the seed's `.uid`.
- New art: extend `tools/gen_art.py`, `godot --headless --import`, manifest rows via a seed (see `art2_a.gd`). Never commit `tools/__pycache__/`.
- Branches to delete at the next cleanup (landed): `claude/assets-audio-9c0dia`, `claude/ITEMUSE1-A-use-reach`, `claude/ITEMUSE1-A2-class-pouch`, `claude/ASSETS-HANDOVER`.
