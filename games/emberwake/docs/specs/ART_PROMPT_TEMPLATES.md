# Art prompt templates

Reusable image-API prompt shapes in the house pixel-art style, and the shapes for the local models
([Local short prompt](#local-short-prompt), with Klein's whole prompt and edit wording). Gemini wrote them on 2026-10-25 when the user asked it
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

## Local short prompt

### Klein: the whole prompt (the main local model)

Since LOCALGEN2 (user 2026-10-26) the main local model is FLUX.2 [klein] 4B in ComfyUI, the default backend of
`localgen.py`. It needs no short form: its Qwen3 4B text encoder reads the whole prompt (Bram's recorded prompt is
about 870 tokens by the tool's estimate, which the record keeps as `tokens_estimate`), so nothing is cut. `run` sends
the character block first, then the shared style block without its "Use the attached image" line (a text-to-image
run attaches no image, and the rest of that line speaks of it). `localgen.py prompt <id>` prints it.

An edit (`vary`, `faces`) keeps the source as a reference latent, and the change is the whole prompt: say what stays,
then the one thing that changes. The expression cells (`faces`, the cell lines of `PORTRAITFX1_TASKS.md` 2.4.2) use:

```text
Keep the same character, face shape, hair, scars and markings, outfit, pixel-art style, framing, lighting and dark background exactly. Change only the facial expression to <expression>: <cell line>.
```

### Short prompt for the CLIP models (OpenVINO drafts)

For the OpenVINO draft model (LOCALGEN1, `--backend openvino`: LCM Dreamshaper v7, SD 1.5 family). Its CLIP text
encoder reads 77 tokens, 75 of them the prompt's own, and drops the rest without a word, so the long image-API blocks
above do not fit. `localgen.py prompt <id>` builds the short form from a prompts-file block and lists what it cut;
`run` and `vary` send that form and keep the cut list in the take's record (`cut`). When a detail that makes the
character is cut (Tess's burned ear is), write the prompt by hand and pass it as `--prompt`.

The short form is only for models whose text encoders are all CLIP (LCM, SD 1.5 and SDXL; the tool reads the family
from the model folder's `model_index.json`). FLUX gets the whole block, subject first, then the style block without
its line about the attached reference. Its T5 encoder reads 256 tokens, though, and one ART3 character block nearly
fills that (Bram's is 245 by FLUX's own tokenizer), so the style block mostly falls off; the record's `cut` says how
much. The rules for the short form, from the 2026-10-25 and 2026-10-26 runs:

- Subject first, style last. A style-first prompt made the small model draw a game scene instead of a portrait.
- Who comes first, with "adult". The model draws young faces, and the tone rule (no child fighters or victims)
  needs the word. Then two or three face features, the hair, one signature detail, the lead clothing item and the mood.
- No names, no hex codes and no negations. The model cannot know a name, and hex codes waste tokens. A CLIP prompt
  cannot say "no": "no hood" draws a hood. The ART3B hair rule becomes the negative prompt (`hood, cowl`) for models
  that take one; LCM takes none.
- Keep an adjective list with its noun ("large, bright, alert eyes"), and keep one look per clause.
- The style tail is about 15 tokens and keeps the ART3B "stylized" rule. A LoRA's trigger words go after it
  (`--trigger`), and `--style` replaces it.
- For a variant (`vary`), the change goes first and replaces the mood.

```text
head and shoulders portrait of <who: age, build, adult>, <two or three face features>, <hair>, <signature detail>, <lead clothing>, <mood> expression, stylized dark fantasy character art, facing right, amber rim light, plain dark background
```

Worked example, Tess by hand (71 tokens; the automatic form spends its words on her eyes and armour and loses the
ear):

```text
head and shoulders portrait of a wiry adult woman of nineteen, sharp lively freckled face, lopsided half-smile, wild dark-brown hair in one loose braid, burn-scarred notched right ear, patched leather armour, courier satchel strap, stylized dark fantasy character art, facing right, amber rim light, plain dark background
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
