# World asset preferences (sprites, maps, tiles, audio, art style)

Shared art style: pixel retro, Style A (2026-10-18).
16-bit look, 48 px sprites, 16-colour palette.
Applies to every sprite, tile, icon and map.

Owner: Assets & Audio thread. Firm rules: `docs/decisions/`. Content (items, units, stats, text): [assets-content.md](assets-content.md). Format: preference (date, source).

## Art style

- The class sets the outfit and palette; the weapon type sets the animation (2026-10-18).
- Offensive stances stand upright; defensive stances hunch (2026-10-18).

## Characters

- Animation set per class: idle, walk, hurt, death plus one attack per skill kind, reused per weapon family with per-skill effects layered on top; stance clips (defend, guarded idle, guarded hit) show defensive stances (2026-10-18).
- Death drops the weapon; a unit without a weapon (brawler) breaks the guard first, arms flung up, then falls back limp, starting from idle (2026-10-18).
- A guarded hit differs from a normal hit, and the defend glow shows in the guarded idle (2026-10-18).

- Warrior gets a shield: animation A for the attack, B for guard and bash (2026-10-18).
- Brawler set as signed off 2026-10-18: L-shaped idle arms (elbows forward, fists bobbing in a 1-2 rhythm), walk arms slightly raised and swinging with the stride, upper-arm skin a touch darker so the arms read against the chest. Jab is short and straight at face height from the hip fist; haymaker keeps the wide hook; kick reaches chest height; the sweep spins with a motion trail near the foot only (2026-10-18).

## Process

- Asset work starts as one heavy review session with Tushar; once the base is signed off, the rest runs as a background task (2026-10-18 12:00).
- One world first: per-world art and audio wait until the Ashlands are complete (2026-10-18).
