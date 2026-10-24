# Art Style, Visual Design & Aesthetic Direction Bible

> **Category:** Art Direction, Technical Pixel Art & Visual Design System  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/specs/art_and_visual_style.md`  
> **Reference Specs:** [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md), [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md), [screen_architecture_and_separation_plan.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/screen_architecture_and_separation_plan.md)

---

## 1. Visual Theme & Core Aesthetic Pillars

*Emberwake: The Low Sleep* merges **oppressive medieval feudalism** with **eldritch cosmic dread**. The world is not a generic high-fantasy realm or standard apocalyptic wasteland; it is a decaying ecclesiastical society living upon the crust of a slumbering cosmic titan.

```text
                                [ THE THREE AESTHETIC PILLARS ]
                                               │
             ┌─────────────────────────────────┼─────────────────────────────────┐
             ▼                                 ▼                                 ▼
   [ THE SHROUD'S GLOOM ]            [ WARDED TALLOW GOLD ]            [ OSSEOUS CORRUPTION ]
   Bruised-violet twilight,          Warm, flickering tallow amber     Brittle jet-black bone spurs,
   perpetual rain-slick slate,       warding against cosmic mists;     pulsing violet veins, calcified
   towering basalt curtain walls.    the fragile light of doomed youth.carapaces, and weeping marrow.
```

### 1.1 The Visual Contrast Rule (Dichotomy of Light & Decay)
Every environment and character design embodies the central ideological struggle:
* **The Human / Cold Steel Spectrum:** Matte beaten iron, dented plate, boiled cowhide, fraying cleric wool, soot-stained linen, and warm candle-glow. Represents fragile mortality, discipline, and human dignity.
* **The Eldritch / Inoculated Spectrum:** Pitch-black petrified bone, pulsating iridescent violet ichor, calcified chalk-white spurs, and unearthly geometric fractures. Represents terrifying cosmic power purchased with bodily ruin.

---

## 2. Technical Canvas, Grid & Display Architecture

To preserve crisp pixel fidelity and ensure uniform integer pixel scaling across modern monitors, all visual rendering conforms to strict mathematical bounds:

```text
+-----------------------+-----------------------------+-------------------------------------------------------+
| DIMENSION             | PIXEL SPECIFICATION         | ENGINE RESOLUTION & SCALING CONSTRAINTS               |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Native Viewport       | $640 \times 360$ pixels     | 16:9 internal virtual resolution. Perfect integer     |
|                       |                             | scaling to 720p ($2\times$), 1080p ($3\times$),       |
|                       |                             | 1440p ($4\times$), and 4K ($6\times$). Zero distortion. |
+-----------------------+-----------------------------+-------------------------------------------------------+
| World Grid Tile       | $32 \times 32$ pixels       | Standard tactical arena unit tile.                    |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Standard Arena Size   | $12 \times 8$ or $12 \times 9$ tiles| $384 \times 288$ px arena area, leaving margins for   |
|                       |                             | timeline Gantt bars, hero cards, and HUD telemetry.  |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Unit Sprite Footprint | $32 \times 48$ pixels       | 1.5 tiles tall. Ground shadow anchors to $32 \times 32$|
|                       |                             | base tile; torso and head extend into upper tile.     |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Sprite Canvas Bounds  | $48 \times 48$ or $64 \times 64$ px | Generous transparent canvas margins preventing sprite |
|                       |                             | clipping during two-handed swings and tendril reaches.|
+-----------------------+-----------------------------+-------------------------------------------------------+
```

---

## 3. The 16-Color Curated Master Palette

The visual environment adheres to a disciplined, 16-color curated palette. Random hues are strictly prohibited; every color serves a mechanical or narrative role:

```text
+---------------------------------------------------------------------------------------------------+
| THE EMBERWAKE 16-COLOR PALETTE                                                                    |
+----+---------------+-----------+-----------------------+------------------------------------------+
| ID | COLOR NAME    | HEX CODE  | RGB TRIPLE            | STRICT SYSTEM / NARRATIVE APPLICATION    |
+----+---------------+-----------+-----------------------+------------------------------------------+
| 01 | Void Abyss    | `#0a0b10` | `rgb(10, 11, 16)`     | Deepest shadows, UI modal backdrops.     |
| 02 | Damp Basalt   | `#14171f` | `rgb(20, 23, 31)`     | Flagstone masonry, sewer vaults, walls.  |
| 03 | Wet Slate     | `#1e232d` | `rgb(30, 35, 45)`     | Walkable ground tiles, rubble outlines.  |
| 04 | Rotting Oak   | `#2c221e` | `rgb(44, 34, 30)`     | Tenement timbers, bunk frames, barrels.  |
| 05 | Scavenged Iron| `#4b5563` | `rgb(75, 85, 99)`     | Unpolished armor, chains, portcullises.  |
| 06 | Polished Steel| `#9ca3af` | `rgb(156, 163, 175)`  | Blade edges, billhook hooks, highlights. |
| 07 | Bleached Linen| `#d9cbb4` | `rgb(217, 203, 180)`  | Bandages, cleric habits, parchment text. |
| 08 | Cinder Ash    | `#6b7280` | `rgb(107, 114, 128)`  | Chimney soot, smoke zones, recovery tint.|
| 09 | Boiled Leather| `#92400e` | `rgb(146, 64, 14)`    | Brigandines, weapon grips, boot leather. |
| 10 | Tallow Amber  | `#d97706` | `rgb(217, 119, 6)`    | Warded flame, lantern radii, Poise bar.  |
| 11 | Candle Gold   | `#fbbf24` | `rgb(251, 191, 36)`   | Active Guard glow, gold coins, UI headers|
| 12 | Arterial Red  | `#e11d48` | `rgb(225, 29, 72)`    | Fast Snap attacks, Bleed, critical hits. |
| 13 | Rust Crimson  | `#991b1b` | `rgb(153, 27, 27)`    | Enemy targeting tethers, execution marks.|
| 14 | Blight Violet | `#9333ea` | `rgb(147, 51, 234)`   | Inoculated maneuvers, aberrant marrow.   |
| 15 | Resonant Orchid| `#c084fc`| `rgb(192, 132, 252)`  | Strain gauge alerts, titan tremors.      |
| 16 | Salt Cyan     | `#06b6d4` | `rgb(6, 182, 212)`    | Holy salt flushes, pure Cold Steel wards.|
+----+---------------+-----------+-----------------------+------------------------------------------+
```

### 3.1 Luminance Hierarchy & Readability Rules
To prevent visual clutter on complex battlefields:
1. **Environment Luminance Clamped $\le 25\%$:** Floor stones and walls must remain in the lower luminance bracket (`#0a0b10` to `#1e232d`) so the ground acts as an unobtrusive canvas.
2. **Character Luminance Clamped at $30\%–65\%$:** Operatives and enemies pop clearly off the floor tiles via sharp silhouette contrast.
3. **Telemetry Accents Clamped $\ge 70\%$:** Danger decals, targeting tethers, and timeline playheads use vivid saturation and high luminance to guarantee instant split-second legibility.

---

## 4. Visual Telegraphy & Decal Hierarchy

In simultaneous combat, telegraphing who is striking where and when is paramount. The visual language uses standardized telemetry symbols:

```text
+-----------------------+-----------------------------+-------------------------------------------------------+
| TELEMETRY ELEMENT     | VISUAL RENDERING SPEC       | TACTICAL PURPOSE & PLAYER FEEDBACK                    |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Ground Danger Decal   | 1-px pulsing dashed border  | Telegraphed heavy cleaves, phosphorus fire, and steam |
| (Area / Heavy)        | with impact badge `[⚠ T4]`  | vents. Units evade by stepping off before impact tick.|
+-----------------------+-----------------------------+-------------------------------------------------------+
| Entity Targeting      | Curved 1-px bezier line from| Single-target strikes (Snap thrusts, aimed bolts).    |
| Tether                | attacker weapon to victim   | Pulses faster as the impact tick approaches.          |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Timeline Playhead &   | Amber playhead bar with     | Scrubbing timeline preview displays translucent       |
| Ghost Projections     | frame-by-frame ghost units  | "ghost" silhouettes striking anticipated poses.       |
+-----------------------+-----------------------------+-------------------------------------------------------+
| Poise Fracture &      | 2-frame full white flash,   | High-impact hit-stop chime. Units snap into an        |
| Stagger Break         | timeline bar cracks in red  | exaggerated recoil pose; floating badge: `[ BREAK! ]`.|
+-----------------------+-----------------------------+-------------------------------------------------------+
| Inoculated Strain     | Bruised-violet shimmering   | Signifies heretical mutagenic attacks; leaves a faint |
| Miasma VFX            | vapor trails along limbs    | violet particle wake that drifts across the grid.     |
+-----------------------+-----------------------------+-------------------------------------------------------+
```

---

## 5. Animation Keyframing & Hit-Stop Feedback

Sprite animations prioritize punchy, readable poses over floaty interpolation:
* **The 4-Pose Maneuver Breakdown:**
  1. *Neutral / Idle:* 4-frame breathing loop at 6 FPS (subtle shoulder sway).
  2. *Windup Pose:* 1 dynamic telegraphed anticipation frame held across all windup ticks. Operative holds weapon cocked high; outline gains a faint amber shimmer.
  3. *Impact Frame:* Instantaneous explosive release (1 frame). Triggers an engine-enforced **30–60ms Hit-Stop Micro-Pause** where both attacker and target freeze in place while the screen micro-shakes (1 px).
  4. *Recovery Pose:* 2-frame follow-through showing weapon weight overbalancing the combatant.
* **Recoil & Knockback:** Units pushed by knockback slide with feet planted, leaving two white dust streaks across cobblestones.

---

## 6. The UI Design System: Glassmorphic Feudalism

The user interface balances sleek, modern responsive functionality with grimdark medieval texture:
* **Panels & Windows:** Rendered in translucent deep slate (`rgba(14, 20, 34, 0.94)`) with a subtle 1-pixel border in `#334155` and beveled iron rivet corners.
* **Typography Hierarchy:**
  - *Headers & Location Titles:* High-contrast beveled serif font in Candle Gold (`#fbbf24`).
  - *Telemetry Badges & Ticks:* Monospace bold typography (`[ T2 ]`, `[ ⚠ T5 ]`) in Arterial Red (`#e11d48`) or Salt Cyan (`#06b6d4`).
  - *Body & Narrative Lore:* Crisp, legible sans-serif in Bleached Linen (`#d9cbb4`) with generous line-height to prevent reading fatigue.
* **The Active Maneuver Deck Cards:** Styled like heavy iron-edged playing cards with weathered parchment centers. Branch A cards have steel-inlaid bevels; Branch B cards feature pulsating violet bone-inlaid borders.

---

## 7. Visual Dossiers: The Six Founders & Key Adversaries

### 7.1 The Six Founders
```text
[ KAELEN ]              [ VESPER ]              [ MAREN ]
Dented kettle helm,     Ragged chimney cowl,    Butcher leather apron,
notched billhook,       twin bodkin daggers,    bone-saw & censer crook,
rag-bound arm with      void-smoke fingertips,  pulsing black neck veins,
calcified white spurs.  unnatural long shadow.  cracked round spectacles.

[ BRAN ]                [ TORIN "SLAG-JAW" ]    [ ELYSE ]
Horn recoil pauldron,   Penal iron wrist cuffs, Brass ocular goggles,
lever-action arbalest,  slag-burned wired jaw,  wicker chemical bandolier,
grafted night-stalker   hypertrophied shoulders,vitriol-stained coat,
crimson ocular beam.    coal-shovel knuckle.    pressurized acid nozzle.
```

### 7.2 Key Adversaries
* **Captain Vane (The Watch Traitor):** Gilded, defaced castellan armor; heavy iron pavise shield emblazoned with the weeping cathedral eye; executioner's billhook.
* **Proctor Malakar (The Scalpel):** Flowing blood-splattered white silk cassock; bronze bird-beak plague mask; exposed mechanical marrow valve leaking violet steam from the throat.
* **Guild-Master Orlov (The Butcher):** Obese, leather-clad industrial cartel boss; steam-pressured lard nozzle strapped to a brass boiler on his back; barbed steel meat-hooks.
* **The Calcified Amalgam:** Amorphous horror formed from dozens of fused skeletons; jagged obsidian bone spires jutting from a writhing central mass of purple marrow muscle.

---

## 8. Audio & Soundscape Direction

Aesthetics in *Emberwake* are heavily acoustic and tactile:
* **The Cathedral Bells (The Low Sleep World Clock):** Ominous, low-frequency bell tolls reverberate through the stone streets, echoing with metallic sub-bass tremors.
* **Cold Steel Foley:** Heavy, resonant anvil chimes for parries and Active Guard; sickening wet cleaves for flesh cuts; dull leaden thuds when billhooks smash heavy plate.
* **Inoculated Foley:** Visceral tearing cartilage, squelching marrow injections, and a brittle, crystalline glass-fracture sound effect whenever an operative advances a Calcification stage.
* **Environmental Ambient Layers:** Drips of stagnant sewer runoff, the distant hum of foundry steam vents, wind howling through quarantine grates, and the rhythmic crackle of warded tallow flames.
