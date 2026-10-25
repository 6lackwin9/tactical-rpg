# Art prompt templates

Reusable image-API prompt shapes in the house pixel-art style. Gemini wrote them on 2026-10-25 when the user asked it
for the prompts behind Mara Voss's portrait and her 2x2 variation sheet (`art_src/mara_voss*.png`); they name Mara as the
worked example. To reuse one, swap the character lines (name, age, features, clothing, weapon, mood, colours) and attach
the character's existing portrait as the reference. The game draws battle bodies with the code rig
(`tools/sprites/rig.py`), so the sheet template gives reference art, not shipped sprites. Records of what each run made sit
beside the images in `art_src/` (`<image>.json`); the Art Prompt Library page holds the user's tweaks and test notes.

## 2x2 portrait variations

Four design choices for one character at once. Keep the framing paragraph; change the four bullets (armour, state, age, hood, expression).

```text
A 2x2 grid of four distinct pixel art character portrait variations for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. 1:1 square image. No borders, no frame, no text, no dividers, no logo. Plain flat very dark background (#120e12) behind all figures. Each variation shows a head-and-shoulders, three-quarter view of Mara Voss, a woman in her early thirties with a strong build, square jaw, and dark copper-red hair. Each variation utilizes a 16-bit pixel art style with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per color, and light dithering on metal and cloth. Strong key light from the upper left and warm orange rim light from the right.

The four variations are:
- Top-Left: A battle-worn version, showing a larger scar across her cheek, messy loose hair, and scuffed, dirt-spattered steel armor with deep red brigandine.
- Top-Right: A heavily-armored, disciplined variant with tight-bound hair, heavily reinforced dark steel pauldrons, and a rugged leather shoulder strap over her deep red brigandine.
- Bottom-Left: An older, battle-hardened version with a visible chainmail hood (coif) framing her face, worn iron armor, and deep red brigandine.
- Bottom-Right: A survival-ready "Ember Warden" variant with looser hair, a frayed dark fur/wool collar over her steel gorget, and the warm orange glow of embers in the background.
```

## Full body from three angles

Non-animated full-body sheets, concept art or large static sprites (a party screen).

```text
2D pixel art full-body character sprite sheet for a dark fantasy tactical RPG, in the style of 16-bit SNES/GBA-era games (e.g., Tactics Ogre, Fire Emblem). Single full-body character shown from multiple angles: a front view, three-quarter front view, and side profile view, standing side-by-side on a plain, flat very dark background (#120e12) with no grid, no floor, and no text. No UI, no border, no watermark. Hand-placed pixel clusters, dark outlines, and a limited palette with light dithering.

Character: Mara Voss, Captain of the Ember Wardens. A woman in her early thirties with a strong, athletic build and dark copper-red hair tied back. She is wearing a deep red riveted brigandine with brass rivets over chainmail, worn steel pauldrons, a steel gorget, and dark steel greaves and boots. She has a sheathed sword on her hip or back and wears a dark leather belt with pouches. Ground shadows are cast beneath her feet. The style is compact but highly readable, emphasizing the deep red, steel gray, and brass color scheme in a 16-bit aesthetic.
```

## Animation sprite sheet

Combat and animation frames (idle, run, attack, hurt). Change the character lines in row 1 and the actions per row; the rig's clips are idle 8, walk 8, attack 6, hurt 4, death 6.

```text
2D pixel art animation sprite sheet for a dark fantasy tactical RPG, 16-bit SNES/GBA style (approx. 48x48 or 64x64 pixels per frame size). Showcase of a single character, Mara Voss, in a side-by-side horizontal and vertical sequence on a flat, solid dark background (#120e12) with no grids, no borders, no text, and no UI. Hand-placed pixel clusters, high-contrast dark outlines, and a limited palette of deep red, steel gray, and brass.

The sheet contains multiple rows of frame-by-frame animation cycles, from left to right:
- Row 1: Idle stance. A woman with dark copper-red hair in deep red brigandine and steel armor, hand on sword hilt, subtly breathing and looking right (4 frames).
- Row 2: Combat ready & Running/moving. Slightly crouched, sword drawn, moving forward with swinging arms and legs (6 frames).
- Row 3: Melee sword attack. Winding up, slashing the sword forward with a brief trail effect, and returning to stance (6 frames).
- Row 4: Hurt & Knockdown. Character flinching back from an impact, then falling or kneeling, showing damage, and recovering (5 frames).

Character has a tiny but highly readable silhouette with exaggerated key movement frames to preserve readability at small sizes.
```

## Props and objects

Game objects in the same style (chests, campfires, barricades, furnaces); swap the bullets for the props needed.

```text
2D pixel art game assets and prop sprites for a dark fantasy tactical RPG, in the style of 16-bit SNES/GBA-era games. A collection of object sprites shown from an isometric/three-quarter top-down perspective, arranged in rows on a flat, very dark background (#120e12). Clear dark outlines, hand-placed pixel clusters, high-contrast lighting, and a limited, earthy color palette with warm orange highlights. No text, no grid, no borders, no UI.

Sprites include:
- An "Ember Warden" campfire with burning logs, embers, and glowing orange flame animation frames.
- A heavy iron-banded wooden loot chest, shown in both closed and opened states.
- A ruined stone barricade/obstacle with moss and rubble.
- A glowing forge/embers furnace with brass accents and sparks.
- Stacked crates, weapons rack with swords, and a medieval tent.

Compact scale, highly readable, matches the 16-bit dark fantasy aesthetic.
```
