# Act 1 vertical slice: asset list

Working list for the Art/UI session (Tushar, 2026-11-10). The slice runs from the prologue to the card after the
Crucible: the opening vignette, 0a The Alley, the Meat Market walk, 0b The Rooftop, 0c The Bridge, the descent, the
hearth scene in the Sluice Undercroft, the hub and Sunk Ward region map, one board dilemma (Corvo's Gate), M3 The
Bone-Mill Raid (Dunstan joins) and M5 The Crucible, then the slice end card (decision "Slice purpose", Q-84 to Q-87;
`games/emberwake/docs/specs/CAST1_ENCOUNTERS.md` 2; `docs/lore/narrative/new_game_onboarding_and_intro.md`).
The rest of Act 1 (M1, M2, M4, the interlude, Elyse) is M2b, listed at the end.

Status: **done** (signed off in the new style), **have-old** (an asset exists from before the Flow style; check it
or redo it), **new** (nothing yet). The new style is the Flow character base (`ART_UI_STUDIO.md` 3.7): muted palette,
dithered shading, 1376 by 768 CGs. Canvas: 640 by 360 logical at 2x (`SCREEN_ATLAS.md`).

## 1. Characters (Flow character: portrait + three-view body)

| Character | Role in the slice | Status |
| --- | --- | --- |
| Kaelen | founder, 0a on | **done** |
| Vesper | founder, 0b on | **done** |
| Maren | founder, 0c on | **done** |
| Hollis | founder, 0c on | **done** |
| Dunstan | founder, guest in M3, joins after it | have-old (`portraits/heroes/dunstan.png`) |
| Captain Vane | "Order 44" at 0a (dialogue, not on the board) | have-old (`portraits/bosses/captain_vane.png`) |
| Bram | hub keeper, hearth scene | have-old (`portraits/npcs/bram.png`) |
| Sister Aline | hub keeper, hearth scene | have-old (`portraits/npcs/aline.png`, VN full body) |
| Tess | the mission board | have-old (`portraits/npcs/tess.png`) |
| Sergeant Corvo | Corvo's Gate dilemma (dialogue, not on the board) | new (uses `warrior_base`) |
| Overseer Grendel | M3 boss | new (uses `brawler_base`) |
| Inquisitor-Prelate Nemeth | M5 climax, takes a founder | have-old (`portraits/bosses/inquisitor_prelate_nemeth.png`) |

## 2. Enemy types (battle sprite + small portrait)

| Enemy (data id) | Where | Status |
| --- | --- | --- |
| Footman / Castellan Footman (`footman_patrol`, `castellan_footman`) | 0a, 0c, Corvo, M5 | have-old (warrior palette) |
| Watch Crossbowman (`watch_crossbowman`) | 0c, Corvo, M5 | have-old (ranger palette) |
| Mortuary Flagellant (`mortuary_flagellant`) | 0b | have-old (brawler palette) |
| Bailiff Enforcer (`bailiff_enforcer`) | Corvo, M3 | have-old (2h hammer palette) |
| Tallow Dredger (`tallow_dredger`) | M3 | have-old (`units/ranger/wenn_tallow`) |
| Inquisitorial Purifier (`inquisitorial_purifier`) | M5 | have-old (portrait; priest palette) |
| Overseer Grendel (`overseer_grendel`), boss | M3 | new look (2h hammer palette) |
| Nemeth (`inquisitor_prelate_nemeth`), boss | M5 | have-old portrait, new body |

## 3. Story CGs and vignette slides (16:9)

| # | Beat | Status |
| --- | --- | --- |
| 1-4 | Opening vignette: the Shroud, the titan, the Clerisy's claim, a guard who refuses (bell, rain, violet sky, iron masks; no Tithing number) | have-old (`backgrounds/story/intro_1..4.png`) |
| 5 | 0a: Captain Vane gives "Order 44" to Kaelen in the rain | new |
| 6 | Meat Market walk: the Spent and the rendering | have-old (`backgrounds/story/meat_market.png`) |
| 7 | 0b: Vesper on the chimney stack, searchlight | **done** (`flow/out/cg_prologue/Vesper_crouching_on_chimney_stack.jpg`) |
| 8 | 0c: Kaelen strikes the cart's padlock, Vesper's ash flask | **done** (`Kaelen_smashing_padlock_on_cart.jpg`) |
| 9 | 0c: Maren and Hollis break out of the cart | new (optional) |
| 10 | The descent: the four at the drainage grate | **done** (`Characters_escaping_through_drain_v2.jpg`) |
| 11 | Arrival: the four wash up in the Sluice Undercroft; Bram at the Tinker's Hearth, Aline at the Apothecary's Slab | have-old (`backgrounds/story/hearth.png`) |
| 12 | Corvo's Gate: the bribe or the fight (one CG, or one per choice) | new |
| 13 | M3: the bone-mill and the chain gang freed; Dunstan joins | new |
| 14 | M5: one founder holds the jammed winch at the Slag Gate as the other three escape | new (key emotional beat) |
| 15 | Slice end card after the Crucible | new |

## 4. Backgrounds

| Background | Use | Status |
| --- | --- | --- |
| Title / main menu (Sunk Ward mist) | MainMenu | **done** (`title_sunk_ward_mist_v2.jpg`) |
| VN stages: alley, rooftop, bridge, hearth | StoryStage dialogue behind full-body sprites | have-old (`backgrounds/story/vn_*_stage.png`) |
| VN stage: Meat Market | the walk between 0a and 0b | have-old (`meat_market.png`; a wide stage version is new) |
| VN stages: Third Ward gate, the bone-mill, the Crucible watchpost | dialogue before and after Corvo, M3, M5 | new |
| Hub: the Sluice Undercroft (Tinker's Hearth, Apothecary's Slab, Water-Lock Basin, the board) | Hub screen | have-old (`hearth.png`) |
| Sunk Ward region map | RegionMap with nodes 01, 03, 04, 06, 07 | new |
| Battle backdrops: `alley`, `rooftops`, `canal`, `foundry`, `watchpost` (384 by 52) | BattleScreen, all slice fights | have-old (`backgrounds/battle/`) |

## 5. Character animations

Two kinds: **stage** (full-body VN sprites at 340 px, the video pipeline, `ART_UI_STUDIO.md` 3.6) and **battle**
(unit sprites on the grid, `units/`). The stage sets get redone from the new Flow bodies (3.8).

| Character | Stage set needed | Status |
| --- | --- | --- |
| Kaelen | idle breath, draw / put away, walk, sneak, poke, slash | interim done (`art_src/vn_stage/kaelen_*_sheet.png`); redo from the Flow base |
| Vesper | idle, walk, sneak, draw daggers, dagger strike, throw (ash flask) | new |
| Maren | idle, walk, sneak, staff strike, bone-saw strike, heal / censer swing | new |
| Hollis | idle, walk, sneak, aim and shoot the arbalest, reload | new |
| Dunstan | idle, walk, main attack | new |
| NPCs (Vane, Bram, Aline, Tess, Corvo) | idle breath only (dialogue) | new |
| Bosses (Grendel, Nemeth) | idle, one signature attack | new |

Battle sprites for all five founders and every enemy type in section 2: idle, move, attack, hit, KO (the current
`units/` sheets; check against the new style before redoing).

## 6. Music and sound

The current tracks are synthesized placeholders (`tools/gen_music.py`; `assets/audio/music/`, kit loops in
`kits/rpg/assets/audio/music/`); the plan is Flow Music (`ART_UI_STUDIO.md` 3.8).

| Track | Use | Status |
| --- | --- | --- |
| Main theme | title and main menu | **done** (`menu_theme.ogg`, test 1B, whole song with a pause) |
| Lullaby theme | ending credits (a candidate for the slice end card) | **picked**: test 1A (dark acoustic) |
| Opening vignette | the four slides (bell, rain, tallow crackle) | new |
| Prologue tension | 0a Vane's order, the alley | new |
| Meat Market | the walk, the Spent | new |
| Stealth | 0b rooftops | new |
| Battle | regular fights (0a-0c, Corvo) | **done** (`sunk_ward_combat.ogg`, the Inquisition take's bass + other stems, 48 s loop); two whole-song takes in review |
| Escape sting | the descent through the grate | new |
| Hub (Sluice Undercroft) | the hearth scene and hub | **done** (`sunk_ward_town.ogg`, a cover of the main theme, seed 455684) |
| Sunk Ward exploration | the region map | new |
| Dilemma | Corvo's Gate choice | new (or reuse prologue tension) |
| Boss | Grendel, M3 | have-old (kit `boss`) |
| Climax boss | Nemeth, the Crucible | new |
| Loss / goodbye | the founder left at the winch | new |
| Slice end card | the closing card | new |
| Stingers | victory, defeat, recruit joined | new |
| Ambience (optional) | rain, distant bells, dripping undercroft, slag vents | new |

## 7. The rest of Act 1 (M2b, after the slice playtest)

- Characters: Elyse (founder, M4), Proctor Quell (M2), Guild-Master Orlov (M4), Brother Malachi (interlude);
  optional-job NPCs Pip, Agnes, Rook.
- CGs: M1 breaking Vane's sweep; M2 the caged youths and Quell's marrow valve; M4 the Tallow Vats and Elyse joining; the
  interlude (the hub floor splits on a titan vein, Aline's marrow dose).
- Backgrounds: M2 and M4 stages, the catacombs; battle backdrops `tallow_vats`, `undercroft` (have-old).
- Music: Tallow Vats, the catacombs, the interlude.
