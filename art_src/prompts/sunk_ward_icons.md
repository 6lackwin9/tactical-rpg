# Sunk Ward icon sheets: families, statuses and skills (runnable prompt file)

Art/UI batch 2026-11-15, from the asset sweep ([ASSET_PLAN.md](ASSET_PLAN.md)). These icons are byte-identical copies
of older icons today (found by hashing the asset trees), so a Sunk Ward weapon or skill shows another one's picture.
Each is replaced **in place**: same path, same pixel size. Whole-sheet prompts with no shared style block, the layout
of `ai_icons.md`; run with `tools/portraits/generate.py run <model> <sheet_id> --file art_src/prompts/sunk_ward_icons.md
--no-ref`. `cut_ai_sheets.py` needs a `SHEETS` entry per sheet before cutting (cell names in the order below; skill
and status icons cut to 64 x 64, family icons to 96 x 96).

Not run before Design confirms the skill list (Outbox `AO-07`: a shared icon may be intended for a variant skill) and
the user approves the model and the image count (`P-UI-06`).

## Sheet families_1 (`families_1`)

Three item-family base icons and two status icons, 3 by 3, with spare takes. Cells, row by row: `polearm`
(`items/polearm/base.png`, today a copy of the kit's battle axe), `censer` (`items/censer/base.png`, a copy of the
mace), `bone_saw` (`items/bone_saw/base.png`, a copy of the shortsword); `steadfast` (`statuses/steadfast.png`, a copy
of the kit's fortify), `stealth` (`statuses/stealth.png`, a copy of the disengage kind icon), spare `polearm`; spare
`censer`, spare `bone_saw`, spare `stealth`. The gear rework (`PR5-51b`) makes the polearm a one-handed phalanx spear
or short bill, so the icon shows that; its other new families (`greatsword`, `gauntlet`, `knife`, the off-hands) get
their own sheet once `PR5-51b` reports its placeholder paths (Inbox `AI-01`).

```text
A sheet of 9 pixel-art icons for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG icons. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Each tile is one icon on its own rounded-square tile filled with dark slate grey, with a subtle darker vignette. Every icon is a single object or symbol drawn large and centred, with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour, a cool light from the upper left and a thin warm amber rim light. Grimy, matte materials: dented iron, soot-stained wood, boiled leather, tarnished brass. Each reads at a glance at small size. No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a short one-handed phalanx spear with a hooked bill blade below its point, set diagonally, its shaft bound with leather; a brass censer on a chain, thin grey smoke rising from its pierced lid; a surgeon's bone saw with a toothed blade and a dark wooden grip.
Row 2: a planted boot and a kite shield braced on the ground, seen from the side, a symbol of standing firm; a hooded figure's silhouette half dissolved into dark smoke, a symbol of stealth; a second take of the short hooked spear, set upright.
Row 3: a second take of the brass censer, swinging on its chain; a second take of the bone saw, set diagonally with a spot of blood on its teeth; a second take of the stealth symbol, a pair of eyes in a dark hood.
```

## Sheets skills_1 to skills_4 (`skills_1` .. `skills_4`)

52 skill icons that share another skill's picture today (`skills/<id>.png`, 64 x 64). Three 4 by 4 sheets and one 2 by 2
sheet. The style paragraph is the same for all four; only the rows differ. Cells in order, with the icon each one
copies today in brackets.

- `skills_1`: `pommel_strike` (shield_bash), `intercepting_hook` (charge_in), `billhook_sunder` (sunder),
  `seventh_ward_bulwark` (shield_wall); `hamstring_sever` (hamstring), `ash_cloud_fade` (feint), `throat_carver`
  (assassinate), `orlovs_retribution` (lacerate); `brimstone_cauterize` (purify), `tourniquet_tonic` (greater_mend),
  `resilience_ward` (aegis), `poachers_quarrel` (barbed_arrow); `tripwire_snare` (pinning_shot), `heavy_winch_sunder`
  (deadeye), `shackle_pummel_slam` (haymaker), `slag_slam_throw` (crushing_blow).
- `skills_2`: `foundry_shatter` (shockwave), `phosphorus_pot` (firebomb), `magnesium_flash_bomb` (stunned), `choke_gas`
  (envenom); `heavy_bill_thrust` (reckless_swing), `billhook_hook` (lunge), `aimed_bolt` (aimed_shot),
  `crushing_judgment` (judgement); `iron_shackle_drag` (pinning_shot), `scourge_lash` (bleeding_cut), `martyrs_rush`
  (charge_in), `pyre_nozzle_sweep` (inferno); `brand_of_the_heretic` (rend_armor), `scalding_grease_pot`
  (glacial_burst), `boning_knife_flurry` (quick_stab), `throat_clamp_pounce` (pounce).
- `skills_3`: `burrow_ambush` (lunge), `caustic_vomit` (envenom), `bone_club_slam` (body_blow),
  `gilded_billhook_sunder` (sunder); `execution_wall` (shield_wall), `traitors_parry` (riposte),
  `sanctified_vivisection` (lacerate), `inquisitorial_parry` (riposte); `transfusion_pulse` (hex),
  `pneumatic_hammer_strike` (haymaker), `coal_shovel_jab` (offhand_jab), `bone_mill_pulverizer` (shockwave);
  `pressurized_lard_spray` (inferno), `flensing_hook_drag` (rend_armor), `scalding_cataclysm` (cyclone_kick),
  `ossified_spine_volley` (volley).
- `skills_4` (2 by 2): `tendril_flail` (sweep_kick), `ossification_bloom` (glacial_burst); `sanctified_decapitation`
  (reckless_swing), `prelates_censure` (intimidate).

Style paragraph (opens every skill sheet; say "16" and "4 by 4", or "4" and "2 by 2" for `skills_4`):

```text
A sheet of 16 pixel-art skill icons for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG ability icons. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Each tile is one icon on its own rounded-square tile filled with dark slate grey, with a subtle darker vignette. Every icon shows one action as a single bold motif drawn large and centred (a weapon or hand in motion with one clear motion arc or impact burst), with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour and one accent colour per icon: red-orange for Watch and fire, sickly yellow for the Tallow Guild, cold white for the Clerisy, blue-violet for corruption, tallow amber for the player's own cell. No faces, no full figures. Each reads at a glance at small size. No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:
```

`skills_1` rows:

```text
Row 1: a sword pommel driving forward into an impact star; a hooked bill blade catching a charging spear shaft; a billhook splitting a plate of armour in two; a wall of three overlapped Watch shields, rust-crimson, braced.
Row 2: a low blade cutting behind a boot heel, a red arc; a cloak dissolving into a puff of grey ash; a narrow dagger drawn across in a thin red line; a hooked flensing talon slashing back, a sickly yellow arc.
Row 3: a glowing brand pressed to a wound, a wisp of smoke; a strap pulled tight round an arm with a small tonic bottle; a small round shield of amber light before a chest; a crossbow quarrel with barbs along its head.
Row 4: a taut wire between two pegs catching a boot; a heavy winch crossbow bolt punching through a shield; two fists bound in iron shackles slamming down; a ladle of molten slag flung in an orange arc.
```

`skills_2` rows:

```text
Row 1: a sledge striking a floor that cracks outward in a ring; a clay pot bursting in white-hot sparks; a small tin bomb in a blinding white flash; a canister venting a low yellow-green cloud.
Row 2: a heavy bill thrust straight forward, a long straight motion line; a hooked bill blade pulling a leg, a curved arc; a crossbow bolt and a small ring of sight lines round its tip; a heavy mace falling from above under a cold white ray.
Row 3: a shackle on a chain thrown out and pulled taut; a knotted scourge mid-lash with three red streaks; a figure's shoulder and shield bursting forward through cold white light; a brass nozzle sweeping a fan of flame.
Row 4: a red-hot branding iron with a blank round head; a pot tipping scalding grease, steam rising; three quick knife cuts, three thin arcs; clawed jaws clamping shut, blue-violet.
```

`skills_3` rows:

```text
Row 1: clawed forelimbs bursting up through cracked ground; a spray of caustic yellow-green bile; a club of knotted bone coming down; a gilded billhook splitting a plate of armour, gold glint.
Row 2: a row of shields with spear points levelled over it, cold white; a blade turning another blade aside, tallow amber spark; a scalpel opening a neat red line, cold white light; a mace haft turning a blade aside, cold white spark.
Row 3: a pulse of dark red rings between two hands; a piston-driven hammer head striking, a puff of steam; a coal shovel's blade jabbing forward; two toothed mill rollers crushing bone to dust.
Row 4: a brass nozzle spraying a cone of pale burning lard; a flensing hook dragging back with a torn strip, sickly yellow; a ring of scalding steam and tallow bursting outward; a volley of pale bone spines in flight.
```

`skills_4` rows (2 by 2):

```text
Row 1: a black tendril whipping low in a wide arc, blue-violet glow; chalk-white bone crystals bursting up in a cluster, blue-violet light in the cracks.
Row 2: a heavy executioner's blade falling under a cold white ray; a raised gauntleted hand with one finger pointing down, cold white light behind it.
```

## Sheet hud_chips_2 (`hud_chips_2`)

A redo of the four 14 x 14 HUD chips (Tushar, 2026-11-15: they "look less clear"). The first sheet drew detailed
pictures that turn to mush at 14 px (gear teeth, nails and clock ticks are smaller than a pixel). This one asks for
one bold silhouette each, drawn as if on a 16 x 16 grid. Cells, row by row: `scrap`, `tallow`, `alert`; `doom_clock`,
spare `scrap` (a nut and bolt), spare `alert` (a lit lantern); spare `doom_clock` (a clock face), spare `tallow` (a
flame), spare `scrap` (a cog). Cut with the hard-grid chip cut to `games/emberwake/assets/ui/hud/<name>.png`.

```text
A sheet of 9 very simple pixel-art status icons for a game's top bar, each one designed to be shown at only 14 by 14 pixels. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF). Each tile holds one single chunky symbol that fills most of the tile, drawn as a low-resolution 16 by 16 pixel sprite scaled up with hard square pixels: one bold silhouette, a thick dark outline, at most 4 flat colours, no small details, no thin lines, no texture, no shading gradients, no glow. No magenta or pink in the symbols. No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a single fat cog wheel with six square teeth and a round hole, rust brown; a short fat candle with one big flame, pale yellow wax and an orange flame; a plain bell with a clapper, dark iron with one red highlight.
Row 2: an hourglass with red sand, brass frame; a fat hexagonal nut with a short bolt through it, rust brown; a square lantern with a lit orange window, dark iron.
Row 3: a round clock face with two thick hands and no numbers, brass with one red hand; a single large teardrop flame, orange with a yellow core; a second fat cog wheel with eight square teeth, grey iron.
```

## Sheet gear_1 (`gear_1`)

The gear pass (Tushar, 2026-11-15: "do a pass on the gears since we worked it out today and see if any dont have a
reference icon"). `docs/sessions/U3_GEAR_REWORK.md` has 15 weapon families, 7 off-hands and 3 armour weights. Checked
against `items/<family>/base.png` in the game and the kit: **eight families have no icon at all** (`greatsword`,
`gauntlet`, `knife`, `buckler`, `braces`, `ammo`, `cloak`, and `graft`, which has five items today and falls back to
the generic icon), and the new **helms** (the `HEAD` slot, one per weight) have none. The others resolve (kit or game),
and `polearm`, `censer` and `bone_saw` got their own icons today. 4 by 4; cells, row by row: `greatsword`, `gauntlet`,
`knife`, `buckler`; `braces`, `ammo`, `cloak`, `graft`; `helm_heavy`, `helm_medium`, `helm_light`, spare `ammo` (a fuel
canister); spare `greatsword`, spare `gauntlet`, spare `knife`, spare `graft`. Targets
`games/emberwake/assets/items/<family>/base.png` at 96 x 96 (scale only); the helm paths wait for `PR5-51b` (proposed
`items/<weight>/helm.png`).

```text
A sheet of 16 pixel-art item icons for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG icons. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Each tile is one icon on its own rounded-square tile filled with dark slate grey, with a subtle darker vignette. Every icon is a single piece of gear drawn large and centred, with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour, a cool light from the upper left and a thin warm amber rim light. Grimy, matte, scavenged materials of a sunless industrial city: dented iron, soot-stained wood, boiled leather, tarnished brass, grey wool. Each reads at a glance at small size. No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a long two-handed executioner's greatsword with a wide square-tipped blade, set diagonally; a heavy iron gauntlet with a brass piston along its forearm, fist clenched; a short single-edged hunting knife with a bone handle; a small round iron buckler with a central boss.
Row 2: a pair of leather forearm braces with iron splints and buckles; a leather quiver of crossbow bolts with a strap; a hooded ash-grey cloak folded over a clasp; a small surgical graft: a brass valve and tube set in a ring of pale bone.
Row 3: a heavy closed iron helm with a riveted visor; a leather hood with an iron brow band; a cloth cowl with a leather half-mask; a small brass pressurised fuel canister with a valve and a carry strap.
Row 4: a second take of the two-handed greatsword, set upright, a notched blade; a second take of the piston gauntlet, open hand; a second take of the hunting knife, in a leather sheath; a second take of the graft, a glass lens in a brass eyepiece on a bone ring.
```
