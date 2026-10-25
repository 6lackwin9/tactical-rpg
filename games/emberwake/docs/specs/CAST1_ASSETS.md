# CAST1 assets: placeholders from existing art, and new art for the user to approve

Status: proposed (2026-10-25; CAST1-R the same day: the hook skills, hazards, vents and zones); part of
[CAST1.md](CAST1.md). User rule (2026-10-25): every unit, founder, boss, item,
skill icon, status and prop the cast needs takes the best-matching existing asset as its placeholder; recolouring an
existing rig body with a new palette file is a placeholder, not new art; every poor match is listed in section 7 for
the user to approve before any new art is made, and the image API is always asked first. Nothing here asks for art
elsewhere. Searched: `games/emberwake/assets/` and `kits/rpg/assets/` (rig bodies and palettes, portraits, item,
skill and status icons, markers) and the engine's common tiles.

Conventions: a body placeholder is `<class>/<variant>` (the rig body under `kits/rpg/assets/units/` or
`games/emberwake/assets/units/`) plus a palette file `games/emberwake/assets/units/<class>/<variant>/palettes/<id>.json`
(CAST1-A7; slot colours from the art bible's 16 hues, numbers below, and a skin ramp chosen per character, Q-78); the
unit and character rows set `art` and `palette`. Icon placeholders are copies of the named file under the new id
(`skills/<family>.png`, `items/<family>/base.png`, `statuses/<id>.png`) or the lookup's fallback; a copy is not new
art. Quality: good (reads right at token size), fair (right role, a visible difference), poor (misleading or missing).

## 1. Founders

| Entity | Placeholder | Match | What is off | Brief for new art |
| --- | --- | --- | --- | --- |
| Kaelen body | `warrior/2h_axe` + palette `kaelen` (05 iron, 13 tabard, 07 linen arm wrap) | fair | an axe head, not a notched billhook; no linen-wrapped forearm | - |
| Kaelen portrait | `games/emberwake/assets/portraits/heroes/bryn_hale.png` | fair | another man's face; plate and red tabard fit | - |
| Vesper body | `rogue/base` + palette `vesper` (08 soot, 04 timber, 12 accents) | good | no smoke at the fingertips | - |
| Vesper portrait | `games/emberwake/assets/portraits/heroes/kit_vane.png` | good | hood and mask fit; another face | - |
| Maren body | `priest/base` + palette `maren` (02 robe, 07 apron, 14 throat veins) | fair | the rig holds a staff or mace, not a censer; no veins drawn | - |
| Maren portrait | `games/emberwake/assets/portraits/heroes/alda_wren.png` | fair | a white hood; Maren is defrocked (darker cloth, ink veins) | - |
| Hollis body | `ranger/base` + palette `hollis` (09 brigandine, 05 iron) | fair | a bow, not a winch arbalest; no eye graft | - |
| Hollis portrait | `games/emberwake/assets/portraits/heroes/tev_quill.png` | fair | a green hood; no left-eye graft | - |
| Dunstan body | `brawler/base` + palette `dunstan` (09 apron, 05 shackles, 10 slag burns) | good | no wired jaw | - |
| Dunstan portrait | `games/emberwake/assets/portraits/heroes/dagny_coal.png` | fair | a headband brawler; no scrap-wired jaw | - |
| Elyse body | `ranger/wenn_tallow` + palette `elyse` (04 coat, 10 retorts) | fair | a bow in hand, no sprayer; cap and coat fit a hedge chemist | - |
| Elyse portrait | `games/emberwake/assets/portraits/heroes/sella_marsh.png` | fair | a hooded archer; no bandolier of retorts | - |

## 2. Act 1 enemies

| Entity | Placeholder | Match | What is off | Brief for new art |
| --- | --- | --- | --- | --- |
| Castellan Footman body | `warrior/1h_buckler` + palette `castellan_watch` (05 iron, 13 tabard, 09 gambeson) | good | a hand axe stands in for the hookbill | - |
| Castellan Footman portrait | `kits/rpg/assets/portraits/common/warrior_base.png` | good | - | - |
| Shield-Bearer (Vane's wave) | as the Footman, palette `castellan_watch` | good | - | - |
| Watch Crossbowman body | `ranger/base` + palette `castellan_watch` | fair | a bow, not a stirrup arbalest | - |
| Watch Crossbowman portrait | `kits/rpg/assets/portraits/common/ranger_base.png` | fair | hooded archer, no watch cloak colours | - |
| Bailiff Enforcer body | `warrior/2h_hammer` + palette `bailiff` (01 blackened plate, 13 hood) | fair | no executioner's hood | - |
| Bailiff Enforcer portrait | `kits/rpg/assets/portraits/common/warrior_base.png` | fair | a kettle helm, not a hood | - |
| Mortuary Flagellant body | `brawler/base` + palette `flagellant` (07 vestments, 12 blood, 05 wire) | fair | bare-chested fits; empty hands, no scourge | - |
| Mortuary Flagellant portrait | `kits/rpg/assets/portraits/common/brawler_base.png` | fair | no barbed wire or mortuary vestments | - |
| Purifier body | `priest/base` + palette `purifier` (05 warded iron apron, 10 canister) | fair | a robe, no bird-beak mask or nozzle | - |
| Purifier portrait | `kits/rpg/assets/portraits/common/priest_base.png` | poor | a gentle hooded priestess; the unit is a gas-masked burner | section 7, N4 |
| Tallow Dredger body | `ranger/wenn_tallow` + palette `tallow_guild` (09 grease-stained apron, 10 goggles) | fair | a bow, not a boning knife and lard pots | - |
| Tallow Dredger portrait | `games/emberwake/assets/portraits/npcs/wenn_tallow.png` | fair | no goggles | - |
| Render-Beast body | `beast/render_beast` (CREATURE1, the rig's quadruped; base palette baked in) | fair | a code creature body; was `rogue/goblin_scout` + palette `render_beast` | section 7, N1 (portrait only) |
| Render-Beast portrait | `kits/rpg/assets/portraits/common/rogue_goblin_scout.png` | poor | a goblin face | section 7, N1 |
| Ash-Crawler body | `crawler/ash_crawler` (CREATURE1, the rig's segmented crawler, with a `spit` clip; base palette baked in) | fair | a code creature body; was `rogue/goblin` + palette `ash_crawler` | section 7, N2 (portrait only) |
| Ash-Crawler portrait | `kits/rpg/assets/portraits/common/rogue_goblin.png` | poor | a goblin face | section 7, N2 |
| Calcified Husk body | `brawler/base` + palette `calcified` (01 black bone, 08 ash skin) | fair | a whole body; the jutting bone is not drawn | - |
| Calcified Husk portrait | `kits/rpg/assets/portraits/common/priest_goblin.png` | poor | a living goblin shaman, not a petrified corpse | section 7, N5 |
| Caged Unsealed (M2 captives) | `rogue/base` + palette `captive` (07 rags, 08 soot) | fair | the rig draws a hood and a dagger on an unarmed captive | - |
| Caged Unsealed portrait | `kits/rpg/assets/portraits/common/rogue_base.png` | fair | masked | - |

## 3. Bosses

| Entity | Placeholder | Match | What is off | Brief for new art |
| --- | --- | --- | --- | --- |
| Captain Vane body | `warrior/2h_axe` + palette `vane` (11 gilding, 05 plate) | fair | an axe, no pavise (the pavise is his drop) | - |
| Captain Vane portrait | `games/emberwake/assets/portraits/npcs/brask_toller.png` | fair | an older officer in plate fits; another face | - |
| Proctor Quell body | `warrior/2h_sword` + palette `quell` (07 white linen, 14 black marrow stains) | fair | a greatsword, not twin bone-saws | - |
| Proctor Quell portrait | `kits/rpg/assets/portraits/common/mage_base.png` | poor | a kindly old mage; Quell is a vivisector with a neck valve | section 7, N6 |
| Overseer Grendel body | `warrior/2h_hammer` + palette `grendel` (04 soot, 10 slag glow) | fair | plate, not a stoker's leathers | - |
| Overseer Grendel portrait | `kits/rpg/assets/portraits/common/brawler_base.png` | fair | a bald brawler fits a slaver-stoker | - |
| Orlov body | `brawler/base` + palette `orlov` (05 harness, 09 butcher leather) | fair | no lard cannon or hooks | - |
| Orlov portrait | `kits/rpg/assets/portraits/common/brawler_base.png` | poor | the same face as Grendel and the Flagellants: the vendetta boss needs his own | section 7, N7 |
| The Amalgam body | `brawler/base` + palette `amalgam` (01 bone, 14 marrow tendrils) at 1 cell | poor | one humanoid stands in for a fused mass on 2 x 2 cells (BIGUNIT1) | section 7, N3 |
| The Amalgam portrait | `kits/rpg/assets/portraits/common/brawler_goblin.png` | poor | a goblin face | section 7, N3 |
| Nemeth body | `warrior/2h_sword` + palette `nemeth` (05 mirror plate, 13 cassock, 14 infusion) | fair | no crimson cassock over the plate | - |
| Nemeth portrait | `kits/rpg/assets/portraits/common/warrior_base.png` | poor | the Footman's face: the Act 1 climax villain who returns in Act 3 needs his own | section 7, N8 |

## 4. Item icons

| Entity | Placeholder | Match | What is off |
| --- | --- | --- | --- |
| `polearm` family (billhooks, Vane's bill) | `kits/rpg/assets/items/axe/battle_axe.png` copied to `items/polearm/base.png` | fair | an axe head, no hook |
| `scourge` family | code-drawn `items/scourge/base.png` (STORYART1: a three-tailed barbed flail, ART2 icon look; was the kit mace copied) | fair | the wire tails are thin at 1x (section 7, N9) |
| `censer` family | `kits/rpg/assets/items/mace/base.png` copied to `items/censer/base.png` | fair | a mace head, not a censer on a chain |
| `bone_saw` family | `kits/rpg/assets/items/sword/shortsword.png` copied to `items/bone_saw/base.png` | fair | a blade without teeth |
| `sprayer` family | code-drawn `items/sprayer/base.png` (STORYART1: a brass tank, hose and pole nozzle; was the kit staff copied) | fair | a code placeholder until the icon pass (section 7, N9) |
| Founder and enemy gear on existing families | the family's `base.png` (dagger, crossbow, fist `knuckles.png`, hammer, axe, shield, plate, leather, cloth, tome) | good | - |
| Slag-Hammer relic | `games/emberwake/assets/items/hammer/titan_warhammer.png` (`icon_id`) | good | - |
| Gilded Pavise, Marrow Scalpel, Flensing Talon | their family's `base.png` | fair | not unique |
| Dreamer's Vertebra, the six keepsakes | `kits/rpg/assets/items/accessory/amulet.png` | fair | all share one icon |
| Stitch-Poultice, Brimstone Draught | `kits/rpg/assets/items/consumable/elixir.png` | fair | a flask, not a poultice |
| Pitch Fire-Bomb | `kits/rpg/assets/items/consumable/bomb.png` | good | - |
| Materials, quest items | `kits/rpg/assets/items/material/base.png`, `kits/rpg/assets/items/quest/base.png` | fair | generic |

## 5. Skill and status icons (copies into `games/emberwake/assets/skills/<family>.png`, `statuses/<id>.png`)

Every CAST1 skill takes a copy of an existing icon from `games/emberwake/assets/skills/`; fair where the action
differs, none poor (the kind icons in `kits/rpg/assets/skills/kinds/` are the fallback).

| Placeholder icon | Good matches | Fair matches |
| --- | --- | --- |
| `sunder.png` | `billhook_sunder`, `gilded_billhook_sunder` | - |
| `shield_wall.png` | `seventh_ward_bulwark` | `execution_wall`, `intercepting_hook` (now a Guard that covers an ally; re-copy over the `charge_in.png` copy) |
| `shield_bash.png` | `buckler_jab` (new) | `pommel_strike` |
| `charge_in.png` | - | `martyrs_rush` |
| `hamstring.png` | `hamstring_sever` | - |
| `feint.png` | - | `ash_cloud_fade` |
| `assassinate.png` | `throat_carver` | - |
| `lacerate.png` | - | `orlovs_retribution`, `sanctified_vivisection` |
| `purify.png` | `brimstone_cauterize` | - |
| `greater_mend.png` | `tourniquet_tonic` | - |
| `aegis.png` | `resilience_ward` | - |
| `barbed_arrow.png` | `poachers_quarrel` | - |
| `pinning_shot.png` | - | `tripwire_snare` (now a trap, not a shot), `iron_shackle_drag` |
| `deadeye.png` | `heavy_winch_sunder` | - |
| `haymaker.png` | `shackle_pummel_slam` | `pneumatic_hammer_strike` |
| `crushing_blow.png` | - | `slag_slam_throw` |
| `shockwave.png` | `foundry_shatter` | `bone_mill_pulverizer` |
| `firebomb.png` | `phosphorus_pot` | - |
| `stunned.png` | - | `magnesium_flash_bomb` |
| `envenom.png` | - | `choke_gas`, `caustic_vomit`, `bile_coat` (new: coats an ally's weapon) |
| `reckless_swing.png` | `sanctified_decapitation` | `heavy_bill_thrust` |
| `lunge.png` | - | `billhook_hook`, `burrow_ambush` |
| `aimed_shot.png` | `aimed_bolt` | - |
| `judgement.png` | `crushing_judgment` | - |
| `bleeding_cut.png` | - | `scourge_lash` |
| `inferno.png` | `pyre_nozzle_sweep` | `pressurized_lard_spray` |
| `rend_armor.png` | `brand_of_the_heretic` | `flensing_hook_drag` |
| `glacial_burst.png` | - | `scalding_grease_pot`, `ossification_bloom` |
| `quick_stab.png` | `boning_knife_flurry` | - |
| `pounce.png` | `throat_clamp_pounce` | - |
| `body_blow.png` | - | `bone_club_slam` |
| `riposte.png` | `traitors_parry`, `inquisitorial_parry` | - |
| `hex.png` | - | `transfusion_pulse` |
| `offhand_jab.png` | - | `coal_shovel_jab` |
| `cyclone_kick.png` | - | `scalding_cataclysm` |
| `volley.png` | `ossified_spine_volley` | - |
| `sweep_kick.png` | - | `tendril_flail` |
| `intimidate.png` | `prelates_censure` | - |

Statuses (the two RULESET1 rows CAST1 relies on): `steadfast` takes a copy of `kits/rpg/assets/statuses/fortify.png`
(fair: same family of buff); `stealth` takes `kits/rpg/assets/skills/kinds/disengage.png` (fair). Every other status
CAST1 uses has its icon in `kits/rpg/assets/statuses/` (`empower` has none today: the existing fallback stays).

## 6. Props, terrain and zones

| Entity | Placeholder | Match | What is off |
| --- | --- | --- | --- |
| The Slag Gate winch (M5 hold tile) | `kits/rpg/assets/map/markers/objective.png` under ENCOUNTER2-U's hold mark | fair | a generic marker, no winch |
| Cage wagon (M2), mortuary cart (0c), inner gate (M5), chimneys (0b), bone pillars (interlude) | `wall` tiles (`engine/assets/tilesets/common/wall`) | fair | walls, not objects |
| Crates (0a), rubble (M1, M3) | `rubble` tiles | good | - |
| Slag pools (M3), boiling vats (M4) | `lava` tiles | good / fair | the vats read as lava |
| Catwalks (M4), the canal bridge (0c) | `bridge` tiles | good | - |
| The canal (0c) | `deep_water` tiles | good | - |
| Grease (M4), violet mist (interlude) | `toxic_bog` tiles | fair | green bog, not grease or violet mist |
| Lanterns (M2), braziers (M5) | `burning_ground` tiles | fair | flat fire, no lantern or brazier |
| Slag vents (M3, an arena hazard on columns 7-8) | the battle screen's arena telegraph marks and strip badge (HOOKS1 AH3, HOOKS1-UI) on floor tiles | fair | no vent grate drawn; the marks show only where and when it strikes |
| Ground hazards (smoke, tripwire, grease, phosphorus and pitch fire, choke-gas) | the `hazards` rows' `tint` and `glyph` overlay (HOOKS1_TASKS 2.1) | fair | a tinted tile with a letter, no smoke or flame sprite; the hidden tripwire is drawn only for the party |
| Meter pips (Bile Phial, Slag Fury) | the unit card's pips (HOOKS1 UM7) | good | - |
| Rooftops (0b) | `stone` tiles | fair | flat stone, no slates |
| Zone backdrops | code-drawn `backgrounds/battle/<zone>.png` for the seven `zones` rows (STORYART1, STAGE1 7.3 format) | fair | code placeholders from the N10 briefs; section 7, N10 |
| Zone tile sets | none per zone (HOOKS1 ZN3: tiles are per terrain): the common terrain tiles above | fair | Ashlands tiles in the Sunk Ward |

## 7. For the user: new art to approve (each one asked first; the image API only after a yes)

| # | Entity | Why the placeholder is poor | Brief (naming guide fields) | Pipeline |
| --- | --- | --- | --- | --- |
| N1 | Render-Beast body and portrait | the rig draws only humanoids | Starved, hairless mastiff-sized hound; bone ridges along the spine; jaws wired with iron; low, forward-leaning silhouette that reads at 40 px; palette 07 skin, 02 shadow, 05 wire, 12 gums; posture: crouched to pounce | image-API sheet (48 x 48 frames, the rig's clip list) or a new rig body; a rig limit (non-human anatomy) |
| N2 | Ash-Crawler body and portrait | the rig draws only humanoids | Many-legged chitinous centipede about a human's length; translucent abdomen of bubbling violet acid; low and long silhouette; palette 02 chitin, 14 acid, 15 glow; posture: coiled, head raised | image-API sheet; a rig limit |
| N3 | The Calcified Amalgam body and portrait | one humanoid on 1 cell stands in for a fused mass on 2 x 2 cells | Dozens of calcified corpses fused by pulsing black marrow tendrils; a hulking mound with arms of bone; silhouette twice a man's width; palette 01 bone, 14 tendrils, 15 pulse; posture: heaving with the titan's heartbeat | 96 x 96 frames on 2 x 2 cells (Q-79, BIGUNIT1); image API, asked first; not needed by the slice |
| N4 | Purifier portrait | a gentle priestess for a gas-masked burner | Warded iron apron, a bronze bird-beak gas mask, a sulfur canister strap over the shoulder; faceless; palette 05 iron, 10 bronze, 08 soot | ART2 portrait pipeline |
| N5 | Calcified Husk portrait | a living goblin shaman for a petrified corpse | The petrified corpse of a miner or an Unsealed youth (an adult by the tone rule), black bone jutting through rotten skin, empty eyes; palette 01, 08 | ART2 portrait pipeline |
| N6 | Proctor Quell portrait | a kindly mage for the vivisector | Lean inquisitor, white linen cassock stiff with black marrow, a brass valve set in the side of his neck, thin smile; palette 07, 14, 05 | ART2 portrait pipeline |
| N7 | Orlov portrait | the same face as Grendel and the Flagellants | Obese, heavily muscled guild master in an iron-plated butcher's harness, a steam hose over the shoulder, barbed hooks at the belt; palette 05, 09, 10 | ART2 portrait pipeline |
| N8 | Nemeth portrait | the Footman's face for the Act 1 villain | Inquisitor-Prelate in mirror-polished plate over a crimson cassock, a tallow-marrow vial in the throat collar, grey skin from the infusion; cold and upright; palette 05, 13, 14 | ART2 portrait pipeline |
| N9 | Scourge and sprayer family icons | a mace and a staff mislead in the shop and loadout | Scourge: a three-tailed flail with barbed wire tails, handle wrapped in vestment cloth. Sprayer: a brass pressure tank with a hose and a pole nozzle | the icon pipeline (ART2), or the image API asked first as one small sheet |
| N10 | Battle backdrops for the seven Sunk Ward zones | a gradient stands in for every place | `alley` Cinder Alley: soot-black brick lanes, hanging linen, tallow lamps; `rooftops` Market Rooftops: wet slates, chimneys, meat hooks on lines; `canal` Canal Bridge: an iron bridge over black canal water, the mortuary cart; `foundry` The Fourth Kiln: slag pools, vent grates, chains, the bone-mill wheel; `tallow_vats` Tallow Vats: boiling vats, catwalks, grease runnels; `undercroft` Sluice Undercroft: drowned vaults, bone pillars, violet mist; `watchpost` Crucible Watchpost: the Slag Gate, the winch, braziers; all from the art bible's 16 hues | `backgrounds/battle/<id>.png` (HOOKS1 ZN1) in the STAGE1 backdrop format; M4 art (HOOKS1_TASKS 2.3), image API asked first, one zone at a time |

ART3 (2026-10-25): the N1 and N2 portraits and N4-N8 are done through the ART2 route (Gemini 3 Pro Image, Mara Voss
as the reference, `pixelize.py`; prompts in `ART3_PROMPTS.md`, a record beside every source in `art_src/`):
`portraits/common/{beast_render_beast,crawler_ash_crawler}.png`, `portraits/elites/{inquisitorial_purifier,calcified_husk}.png`
(the husk shows once its unit sets `portrait_tier` elite), `portraits/bosses/{proctor_quell,guild_master_orlov,inquisitor_prelate_nemeth}.png`.

User 2026-10-25: N1 and N2 bodies are code creature rigs (task CREATURE1, built 2026-10-25), not image-API sheets; their portraits
stay image-API, asked first. Story-scene art is on hold and code-drawn meanwhile (STORYART1, which also draws N9
and N10 as placeholders).
