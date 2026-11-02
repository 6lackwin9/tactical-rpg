# World asset preferences (sprites, maps, tiles, audio, art style)

Shared art style: pixel retro, Style A (2026-10-18).
16-bit look, 48 px sprites, 16-colour palette.
Applies to every sprite, tile, icon and map.

Owner: Assets & Audio thread. Firm rules: `docs/decisions/`. Content (items, units, stats, text): [assets-content.md](assets-content.md). Format: preference (date, source).

## Art style

- The class sets the outfit and palette; the weapon type sets the animation (2026-10-18).
- Offensive stances stand upright; defensive stances hunch (2026-10-18).
- Fire in generated art must look placed: flames scattered over a scene or dotted along letters read as random and awkward. Prefer a soft glow, a few embers and clean lettering; weak flames beat random ones (user 2026-11-02, title backdrop and logotype takes).

## Characters

- The token is the in-game image for combat and exploration (the sprite); the portrait is separate, more detailed art (2026-10-20).
- Playable characters, important NPCs, elites and bosses each get a unique portrait; mob characters, friendly or hostile, share portraits (2026-10-20).
- Sprites may be borrowed: a class body is the default for every member, with variants inside the class (human, goblin) and minor per-character adjustments (2026-10-20).
- Items and skills fall back on a base icon per weapon type and per skill kind until they get unique art (2026-10-20).
- Layers: the engine ships generic baseline assets (terrain tiles first), the RPG kit the reusable gear, skill and unit art, the game only its named assets (Ashlands tiles, named characters); a missing asset falls back game, kit, engine (2026-10-20, decision "Asset layers and reuse groups").
- Animation set per class: idle, walk, hurt, death plus one attack per skill kind, reused per weapon family with per-skill effects layered on top; stance clips (defend, guarded idle, guarded hit) show defensive stances (2026-10-18).
- Death drops the weapon; a unit without a weapon (brawler) breaks the guard first, arms flung up, then falls back limp, starting from idle (2026-10-18).
- A guarded hit differs from a normal hit, and the defend glow shows in the guarded idle (2026-10-18).
- Portraits are detailed pixel art at the 120 px grid (96 until PIXEL2, 2026-10-26), 4 shades per colour family, each character in their own colours on shared shade ramps; the reference is a riveted-armour warrior bust with strong side light and dark outlines (2026-10-20, ART2).
- Unique portraits come from source art (an image model's, in the Mara Voss style and framing) run through `tools/portraits/pixelize.py`; a new character gets a new generation with Mara's portrait as the style reference, never a recolour of another character (2026-10-20).
- Characters are stylized, not realistic (user 2026-10-25, Scarlet Lullaby): expressive, larger-than-life features pushed by faction. The founders and hub cast stay readable and warm; the Clerisy, the corrupted and the creatures go further (gaunt, elongated, uncanny: infused grey skin, valves, bone through skin). Hair varies in length and style for women and men alike, with long hair worn loose, braided or wild; a hood, cowl or tied-back hair only where the character's brief calls for it. The image reference (Mara Voss) sets the pixel style and framing, never the proportions.
- Recolours of one base portrait (`tools/portraits/recolor.py`: hair, armour, skin, trim) are for shared portraits only: guards, nameless soldiers, mob variants (2026-10-20).
- A character may keep several portrait variants (Mara Voss: five) for use in other contexts or a later change of feel (2026-10-20).
- The priest holds the mace raised forward with the head up, gripped a third of the way up, in robe sleeves to the wrist (2026-10-20, ART2 pick B3).

- Warrior gets a shield: animation A for the attack, B for guard and bash (2026-10-18).
- Brawler set as signed off 2026-10-18: L-shaped idle arms (elbows forward, fists bobbing in a 1-2 rhythm), walk arms slightly raised and swinging with the stride, upper-arm skin a touch darker so the arms read against the chest. Jab is short and straight at face height from the hip fist; haymaker keeps the wide hook; kick reaches chest height; the sweep spins with a motion trail near the foot only (2026-10-18).

## Process

- Asset work starts as one heavy review session with Tushar; once the base is signed off, the rest runs as a background task (2026-10-18 12:00).
- One world first: per-world art and audio wait until the Ashlands are complete (2026-10-18).
- The image API (Gemini, `tools/portraits/generate.py`) may make other assets too, but every run is agreed with Tushar first (what, how many images, which model); the agent suggests the API wherever it beats code or SVG. Small assets go several to one prompt as a sheet that a tool slices. One-off or less detailed images can go the app route instead: the agent gives the prompt, Tushar generates it in the Gemini app and saves it to `art_src/`. Colour and small pixel fixes are code, never the API (2026-10-20).
- Local generation first (user 2026-10-25): drafts and iteration run on the PC's local model (OpenVINO on the Intel NPU or Arc GPU, free, a few seconds an image; `C:\dev\localgen`, then the repo tool from LOCALGEN1); the art is pixelized to the 120 px grid anyway, so modest detail is enough. The image API is the fallback for looks the local model cannot reach, still asked first. Every local take keeps its prompt record like the API ones. Measured by LOCALGEN1 (2026-10-26): the local default is the GPU (about 0.8 s an image at 512 px); local takes are layout and colour drafts, not finals; image-to-image expressions drift the face's identity, so the image-API sheets stay for the PORTRAITFX1-XA finals. Since 2026-10-26 (user) the main local model is FLUX.2 [klein] 4B in ComfyUI on the Arc GPU (`C:/dev/localgen/comfyui.bat`, <http://127.0.0.1:8188>): it follows the full recorded prompts, and its reference edit changes an expression while keeping the face, scars and outfit; speed (about 1.5 min an image) is not a concern. Klein expression drafts are tried for the PORTRAITFX1 cast before any image-API sheet. User 2026-10-26: limit-test the local model before any image-API use; the project does not need very high quality assets, so "good enough for the game" is the bar, and the image API (Gemini) is kept for gaps the local model cannot reach after prompt rewrites, reference images and the larger Klein have been tried, never as the default. Portrait base images (user 2026-10-26): larger, well-defined facial features read better after pixelizing, even when the base looks cartoonish: big clearly drawn eyes (distinct whites, iris and dark lids), strong dark brows, a large defined mouth, a slightly oversized head, bold outlines around the features; every portrait prompt (local or image API) asks for them. Lighting is per faction (decision "Faction lighting", lore.md, 2026-10-27): each prompt's style block takes its group's lighting line instead of the single amber rim. Asset route (user 2026-10-27): every asset lane may use local generation without asking: FLUX.2 Klein 4B first (and for every edit: expressions, variants, fixes), ERNIE-Image Turbo (8B, Apache-2.0, in AI Playground's folder; about 2.5 min an image and it fills the RAM, so one model family per ComfyUI session) for heavy new images such as armour-heavy characters, props, scenes and anything with text; the image API only when neither looks right, still asked first.
- The portrait baseline (18 named characters, 12 generic bases, recolours for mob variants) stands for now; characters may be repurposed as the world fills out (2026-10-20).
- Generated art is a first-pass baseline for an alpha build others can play: do not over-polish while systems are not final; asset feedback comes after the full game runs (2026-10-21).
- Tile sets may start with the Sunk Ward before the Ashlands are complete (user 2026-11-02, decision "LOOK1 answers" Q3; lifts the 2026-10-18 "one world first" rule for tiles only).
