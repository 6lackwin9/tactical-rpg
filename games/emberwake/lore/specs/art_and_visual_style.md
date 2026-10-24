# Art & Visual Style: Technical Pixel Art & Palette Architecture

> **Category:** Art Direction & UI/UX Technical Specifications  
> **Reference File:** [master_specification.md §10](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)

---

## 1. Technical Canvas & Resolution Anchors

To ensure razor-sharp pixel rendering without distortion or non-uniform pixel scaling across modern displays, all visual assets adhere to strict geometric constraints:

* **Internal Native Viewport:** $640 \times 360$ pixels (16:9 aspect ratio).
* **Integer Display Scaling:**
  - $2\times \longrightarrow 1280 \times 720$ (720p HD)
  - $3\times \longrightarrow 1920 \times 1080$ (1080p Full HD)
  - $4\times \longrightarrow 2560 \times 1440$ (1440p QHD)
  - $6\times \longrightarrow 3840 \times 2160$ (4K UHD)
* **Grid Tile Dimensions:** $32 \times 32$ pixels per world tile.
* **Standard Tactical Map Sizes:** $12 \times 8$ or $12 \times 9$ tiles (Tight indoor sewer chokepoints: $10 \times 6$ tiles).
* **Sprite Canvas Dimensions:**
  - **Ground Footprint:** $32 \times 48$ pixels.
  - **Cell Bounding Box:** Centered within a $48 \times 48$ or $64 \times 64$ pixel sprite canvas to accommodate elongated weapon swings, polearm reaches, and protruding osseous graft extrusions without sprite clipping.

---

## 2. Palette Architecture & Contrast Hierarchy

Visual clarity is critical in a simultaneous timeline game. Color is strictly stratified across three distinct luminance tiers:

| Layer | Hex Color Codes | Purpose & Art Rules |
| :--- | :--- | :--- |
| **Environmental Base** | `#1e232d` (Damp Slate)<br>`#2b221b` (Rotting Timber)<br>`#14171c` (Deep Shadow) | Used for floors, masonry walls, grates, and inert background decor. **Luminance is strictly clamped below 25%** so that characters, decals, and hazards immediately pop to the player's eye. |
| **Character Mid-Tones** | `#8a8f9d` (Worn Iron)<br>`#9e7144` (Tanned Leather)<br>`#d9cbb4` (Tallow Cloth) | Used for character bodies, armor plating, cloaks, and weapons. Maintained between 30%–65% luminance to ensure crisp silhouette readability against dark stone floors. |
| **Signal / Threat Accents** | `#e11d48` (Arterial Red)<br>`#a855f7` (Blight Violet)<br>`#fbbf24` (Candle Gold)<br>`#06b6d4` (Salt Cyan) | **Reserved strictly for mechanics and threat telemetry:**<br>• *Arterial Red (`#e11d48`):* Fast Snap threats, bleeding wounds, critical interrupts.<br>• *Blight Violet (`#a855f7`):* Strain surges, heretical magic, void phase effects.<br>• *Candle Gold (`#fbbf24`):* Stagger meters, active guard poise, lantern radii.<br>• *Salt Cyan (`#06b6d4`):* Holy salt wards, chemical cleanses, cold steel stances. |

---

## 3. Visual Telegraphy & Decal Standards

* **Ground Danger Decals:** Ground hazard zones must use geometric outlines with pulsing 1-pixel borders and corner tick marks. The impact tick is rendered in high-contrast bold typography (`[ ⚠ T4 ]`) centered directly on the targeted tile.
* **Targeting Tethers:** Single-target strikes draw a smooth, 1-pixel curved bezier tether from the attacker's weapon node to the target's base reticle. The tether pulses faster as the impact tick approaches.
* **Stagger & Interrupt Feedback:**
  - Upon a successful interrupt, the target unit immediately flashes white for 2 frames (33ms).
  - The unit snaps into an exaggerated recoil pose with head thrown back.
  - An amber floating text badge `[ STAGGERED ]` rises 16 pixels before dissolving into black ash particles.

---

## 4. Starter Squad Visual Dossiers

The four initial heroes of The Severed Root illustrate the visual spectrum between Cold Steel armor and aberrant mutation:

```text
[ Kaelen: Vanguard ]      [ Vesper: Stalker ]       [ Maren: Acolyte ]       [ Bran: Arbalestier ]
Battered kettle helm,     Shredded soot cowl,       Leather butcher apron,   Horn recoil pauldron,
defaced breastplate,      slate leather jerkin,     bone-saw & censer crook, lever-action arbalest,
calcified arm bone spurs. shifting void fingertips. pulsing throat veins.    red glowing ocular pupil.
```

1. **Kaelen (The Vanguard):**
   - Palette: Heavy `#8a8f9d` iron plates gouged by claws, wrapped with dirtied `#d9cbb4` bandage wraps around his mutated left arm where bone spikes pierce through.
2. **Vesper (The Gutter-Stalker):**
   - Palette: Charcoal `#14171c` hooded cloak with `#9e7144` leather accents. Her twin bodkin daggers are matte black iron; her hands dissolve into `#a855f7` blight violet wisps.
3. **Maren (The Heretic Acolyte):**
   - Palette: Stained `#d9cbb4` church robes under dark blood-crusted leather apron; brass censer staff emits faint `#fbbf24` smoke; neck veins pulse with `#a855f7` violet ichor.
4. **Bran (The Arbalestier):**
   - Palette: Muted `#9e7144` hunting leathers with cold-forged iron reinforcements; his left eye emits a piercing `#e11d48` arterial red laser line across the grid when drawing his arbalest.
