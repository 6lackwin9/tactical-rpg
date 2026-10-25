# Art and visual style

Status: canon (2026-10-25). The art bible for the Low Sleep worlds. One palette (this doc's hex values win over the
archived master export), the STAGE1 option B battle board, the rig's frames. Process rules (image API, asset layers)
follow `games/emberwake/docs/preferences/assets-world.md`; the board follows
[STAGE1.md](../../specs/STAGE1.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Theme and pillars

Emberwake: A Scarlet Lullaby joins feudal oppression with cosmic dread: a decaying church state living on the crust of
a sleeping titan.

| Pillar | Look |
| --- | --- |
| The Shroud's gloom | Bruised-violet twilight, rain-slick slate, basalt curtain walls |
| Warded tallow light | Warm, flickering amber lanterns holding back the mists; the brief light of the young |
| Osseous corruption | Jet-black bone spurs, violet veins, chalk-white calcified growths, weeping marrow |

Contrast rule: the human and Cold Steel side is matte iron, dented plate, boiled leather, wool, soot-stained linen and
candle-glow; the eldritch and Inoculated side is black petrified bone, violet ichor, white spurs and geometric
fractures.

## 2. Canvas, board and sprites

| Item | Value | Carrier | Tag |
| --- | --- | --- | --- |
| Logical canvas | 640 x 360, integer-scaled with nearest filtering (2x 720p, 3x 1080p, 4x 1440p, 6x 4K) | `aegis_ui::shell` | `[engine]` |
| Battle board | STAGE1 option B: the battle grid drawn as tilted cells 32 px wide and 24 px deep, a zone backdrop at the far edge and a front face at the near edge, a faint grid always on | STAGE1-E `[engine]` pieces, STAGE1-L `stage.rs` | `[engine]`, `[game code]` |
| Battle units | 40 px tokens (`token_px` 40 under STAGE1), contact shadows, feet-sorted with upright props (trees, stones, wall faces) | `aegis_ui::depth::DepthList` | `[engine]` |
| Rig frames | 48 x 48 frames; a "32 x 48 sprite" in the lore means the figure's bounds inside the frame (up to 32 wide and 48 tall, feet on the bottom row) | `tools/sprites` rig | `[data]` |
| Large units | Aberrations and bosses that outgrow 48 x 48 are open (gap 3) | - | - |
| Region map | Square top-down, 16 logical px tiles today (`region.rs`) | `emberwake_gui` `region.rs` | `[kit hook]` |
| District map | The world map node graph over a backdrop | `worldmap.rs` | `[kit hook]` |
| Battle HUD | COMBAT-UI3 layout A: the timeline strip at the top, party cards left, hostile cards right, status bar and log at the bottom | `emberwake_gui` `battle.rs` | `[game code]` |

Retired: "32 x 32 isometric" and a fixed "32 x 32 flat" battle grid (STAGE1 B replaces both); sprite canvases of 64 x 64.

## 3. The palette

Sixteen master hues. Each hue may have at most one dark and one light step; the steps are recorded in the rig's palette
files (`palettes/<id>.json`) when they are made (the step rule is gap 1). The skin ramp is the one extra ramp.

| Id | Name | Hex | Use |
| --- | --- | --- | --- |
| 01 | Void Abyss | `#0a0b10` | Deepest shadows, modal backdrops, UI panels (94% opacity) |
| 02 | Damp Basalt | `#14171f` | Masonry, sewer vaults, walls |
| 03 | Wet Slate | `#1e232d` | Walkable ground, rubble outlines |
| 04 | Rotting Oak | `#2c221e` | Tenement timbers, bunk frames, barrels |
| 05 | Scavenged Iron | `#4b5563` | Rough armour, chains, portcullises; UI panel borders |
| 06 | Polished Steel | `#9ca3af` | Blade edges, billhooks, highlights |
| 07 | Bleached Linen | `#d9cbb4` | Bandages, habits, parchment, body text |
| 08 | Cinder Ash | `#6b7280` | Soot, smoke zones, recovery blocks |
| 09 | Boiled Leather | `#92400e` | Brigandines, grips, boots |
| 10 | Tallow Amber | `#d97706` | Warded flame, lantern light, the poise bar |
| 11 | Candle Gold | `#fbbf24` | Active Guard glow, headers |
| 12 | Arterial Red | `#e11d48` | Snap actions, bleed, crits, impact badges |
| 13 | Rust Crimson | `#991b1b` | Enemy target lines, execution marks |
| 14 | Blight Violet | `#9333ea` | Inoculated actions, marrow |
| 15 | Resonant Orchid | `#c084fc` | Strain alerts, titan tremors |
| 16 | Salt Cyan | `#06b6d4` | Salt flushes, Cold Steel wards |

Skin ramp (design pick; the asset lane may adjust it at the first review):

| Step | Hex | Use |
| --- | --- | --- |
| Dark | `#6b4a3a` | Skin shadow |
| Base | `#a47a5f` | Skin |
| Light | `#d4a988` | Skin light under tallow |

Retired values from the master export: Worn Iron `#8a8f9d`, Leather `#9e7144`, Blight Violet `#a855f7`, Rotting Wood
`#2b221b`, Shadow `#14171c`, the UI panel `rgba(14, 20, 34, 0.94)` and border `#334155` (now hues 01 and 05).

Readability: environment luminance at most 25% (hues 01-04); characters 30-65%; telemetry accents (badges, target
lines, the playhead) at least 70%.

## 4. Telemetry and decals

| Element | Look | Purpose | Tag |
| --- | --- | --- | --- |
| Impact badge | A badge on the strip block where an impact lands (`[B4]`), the same mark on the ground zone | When a hit lands | `[engine]` |
| Ground danger zone | 1 px dashed pulsing outline on the tilted cells | Area and heavy attacks; step out before the impact | `[data]` |
| Target line | 1 px curve from attacker to victim, faster pulse as the impact nears | Single-target actions | `[game code]` |
| Poise break | 2-frame white flash, the segment cracks in red, a `BREAK` badge | The action is cancelled | `[game code]` |
| Inoculated miasma | Violet vapour along limbs, a faint particle wake | Marks heretical actions (Act 2 on) | `[data]` |

Cut (DEC-LORE-37): ghost silhouettes and multi-track scrubbing; impact badges on the existing strip come first.
Effects (projectiles, swing trails, sparks, hit-stop) are FX1's.

## 5. Animation

| Pose | Rule |
| --- | --- |
| Idle | 4-frame breathing loop |
| Windup | One anticipation frame held across the windup blocks, weapon cocked, a faint amber outline |
| Impact | One release frame; hit-stop of 30-60 ms with a 1 px shake (FX1) |
| Recovery | 2 frames of follow-through, shown on recovery blocks |
| Knockback | Slide with planted feet, two dust streaks |

The rig's clip set per class (idle, walk, hurt, death, one attack per skill kind, guard clips) stays as in the world
asset preferences.

## 6. UI look

- Panels: hue 01 at 94% opacity, a 1 px border in hue 05, riveted corners.
- Headers and place titles: a serif face in hue 11. Badges and block numbers: bold monospace in hue 12 or 16. Body
  text: hue 07. The engine's GUI font stays until a font task says otherwise.
- Deck cards: iron-edged cards with parchment centres; Cold Steel cards with steel bevels, Inoculated cards with violet
  bone borders.

## 7. Character briefs

### 7.1 Founders

| Founder | Brief |
| --- | --- |
| Kaelen | Dented kettle helm, notched billhook, rag-bound left forearm over white bone spurs |
| Vesper | Ragged chimney cowl, twin bodkin daggers, smoke at the fingertips, an unnaturally long shadow |
| Maren | Scribe's habit cut short, ink-stained fingers, censer crook, cracked round spectacles, black veins at the throat (the butcher apron and bone-saw went with the dissection background to Aline) |
| Hollis | Hunting brigandine, horn recoil pauldron, lever arbalest, a crude grafted eye with a thin red glint |
| Dunstan "Slag-Jaw" | Penal iron wrist cuffs, a slag-burned jaw wired with scrap, heavy shoulders, coal-shovel knuckles |
| Elyse | Brass goggles, wicker chemical bandolier, vitriol-stained coat, a pressurised acid nozzle |

### 7.2 Adversaries

| Name | Brief |
| --- | --- |
| Captain Vane | Gilded, defaced Castellan armour; iron pavise with the weeping cathedral eye; executioner's billhook |
| Proctor Quell | Blood-spattered white silk cassock, bronze bird-beak mask, a marrow valve venting violet steam at the throat |
| Guild-Master Orlov | Heavy leather-clad cartel boss, a brass boiler on his back feeding a lard nozzle, barbed meat hooks |
| The Calcified Amalgam | Fused skeletons around a writhing mass of violet marrow muscle, obsidian bone spires |

### 7.3 How the art is made

| Art | Route |
| --- | --- |
| Human bodies (founders, guards, inquisitors) | The rig: kit class bodies with palettes and named variants (SPRITE2) |
| Aberrations and large bosses (the Amalgam, Act 2-3 horrors) | The image API, asked each time (what, how many, which model), or a future rig for non-human bodies |
| Portraits | The ART2 pipeline (source art through `pixelize.py`, 96 px grid, 4 shades per family); new characters get a new generation, asked first |
| Tiles, props, backdrops | Code first (`gen_art.py`), image-API sheets only when asked |

## 8. Audio

- Cathedral bells: low tolls with a sub-bass tremor; the bells mark time in Act 3.
- Cold Steel foley: anvil chimes for guards, wet cleaves, dull thuds on plate.
- Inoculated foley: tearing cartilage, marrow injections, a brittle glass crack when a Calcification stage advances.
- Ambience: sewer drips, foundry steam, wind through grates, crackling tallow.

## 9. Gaps

1. The step rule. Options: (a) computed (dark = 65% value, light = 35% toward hue 07); (b) hand-picked per hue at the
   first art review. Recommendation: (a) as the default, (b) where a step reads badly.
2. One skin ramp for a varied cast. Options: (a) add two more skin ramps (light and dark); (b) keep one ramp and vary
   by recolour. Recommendation: (a), decided at the first portrait and body review.
3. Units larger than 48 x 48 (the Amalgam, the titan). Options: (a) 96 x 96 frames on the same rig scale; (b) scaled
   48 x 48 bodies. Recommendation: (a), drawn through the image API, asked first.
4. The Ashlands kit bodies use their own palette. Options: (a) the Low Sleep palette applies per world through palette
   files; (b) it replaces the kit palette. Recommendation: (a), so the sample world stays as it is.
5. The preferences say per-world art waits until the Ashlands are complete; the new worlds need placeholder art in M2.
   Recommendation: the user confirms that the Sunk Ward placeholders may start.
