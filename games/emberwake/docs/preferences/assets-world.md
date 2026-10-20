# World asset preferences (sprites, maps, tiles, audio, art style)

Shared art style: pixel retro, Style A (2026-10-18).
16-bit look, 48 px sprites, 16-colour palette.
Applies to every sprite, tile, icon and map.

Owner: Assets & Audio thread. Firm rules: `docs/decisions/`. Content (items, units, stats, text): [assets-content.md](assets-content.md). Format: preference (date, source).

## Art style

- The class sets the outfit and palette; the weapon type sets the animation (2026-10-18).
- Offensive stances stand upright; defensive stances hunch (2026-10-18).

## Characters

- The token is the in-game image for combat and exploration (the sprite); the portrait is separate, more detailed art (2026-10-20).
- Playable characters, important NPCs, elites and bosses each get a unique portrait; mob characters, friendly or hostile, share portraits (2026-10-20).
- Sprites may be borrowed: a class body is the default for every member, with variants inside the class (human, goblin) and minor per-character adjustments (2026-10-20).
- Items and skills fall back on a base icon per weapon type and per skill kind until they get unique art (2026-10-20).
- Layers: the engine ships generic baseline assets (terrain tiles first), the RPG kit the reusable gear, skill and unit art, the game only its named assets (Ashlands tiles, named characters); a missing asset falls back game, kit, engine (2026-10-20, decision "Asset layers and reuse groups").
- Animation set per class: idle, walk, hurt, death plus one attack per skill kind, reused per weapon family with per-skill effects layered on top; stance clips (defend, guarded idle, guarded hit) show defensive stances (2026-10-18).
- Death drops the weapon; a unit without a weapon (brawler) breaks the guard first, arms flung up, then falls back limp, starting from idle (2026-10-18).
- A guarded hit differs from a normal hit, and the defend glow shows in the guarded idle (2026-10-18).
- Portraits are detailed pixel art at the 96 px grid, 4 shades per colour family, each character in their own colours on shared shade ramps; the reference is a riveted-armour warrior bust with strong side light and dark outlines (2026-10-20, ART2).
- Unique portraits come from source art (an image model's, in the Mara Voss style and framing) run through `tools/portraits/pixelize.py`; a new character gets a new generation with Mara's portrait as the style reference, never a recolour of another character (2026-10-20).
- Recolours of one base portrait (`tools/portraits/recolor.py`: hair, armour, skin, trim) are for shared portraits only: guards, nameless soldiers, mob variants (2026-10-20).
- A character may keep several portrait variants (Mara Voss: five) for use in other contexts or a later change of feel (2026-10-20).
- The priest holds the mace raised forward with the head up, gripped a third of the way up, in robe sleeves to the wrist (2026-10-20, ART2 pick B3).

- Warrior gets a shield: animation A for the attack, B for guard and bash (2026-10-18).
- Brawler set as signed off 2026-10-18: L-shaped idle arms (elbows forward, fists bobbing in a 1-2 rhythm), walk arms slightly raised and swinging with the stride, upper-arm skin a touch darker so the arms read against the chest. Jab is short and straight at face height from the hip fist; haymaker keeps the wide hook; kick reaches chest height; the sweep spins with a motion trail near the foot only (2026-10-18).

## Process

- Asset work starts as one heavy review session with Tushar; once the base is signed off, the rest runs as a background task (2026-10-18 12:00).
- One world first: per-world art and audio wait until the Ashlands are complete (2026-10-18).
