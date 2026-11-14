# AI-05, AI-06, AI-08: NPC, boss and elite portraits (runnable prompt file)

Art/UI batch 2026-11-12. Laid out for `tools/portraits/generate.py run <model> <id>... --n 2 --prompts art_src/prompts/ai_portraits.md` (the same layout as `ART3_PROMPTS.md`): the style block below is ART3's, copied unchanged, and goes out before each id's block, with `art_src/mara_voss.png` as the style reference. Sources: `CANON.md` 2.1 (faction rim lights) and 3.2 (locked visual hooks). A block's `Lighting:` line replaces the style block's amber rim for non-Hearth factions; pass the same faction to `pixelize.py` (`faction` in the record) so the backdrop matches. Each section gives the target path under `games/emberwake/assets/portraits/`, the pixelize faction, and notes. Kept 1:1, so not here: `berek_dorn` (the old Bram art), `captain_vane`, `proctor_quell`, `guild_master_orlov`, `inquisitor_prelate_nemeth`.

```text
Use the attached image (Mara Voss) only as the reference for pixel-art style and framing: the same pixel-art technique, level of detail, framing, lighting, outline weight and flat dark background. Do not copy her face, her hair, her build or her proportions: this is a different subject with its own stylized proportions.
Pixel art character portrait for a gothic dark fantasy tactical RPG set in a sunless church city lit only by tallow lanterns, in the style of detailed 16-bit SNES/GBA-era RPG portraits. Square 1:1 image. Head and shoulders (for a creature: head and forequarters), the head filling about half the frame, three-quarter view facing slightly to the viewer's right, eyes looking right. Strong cool key light from the upper left, a warm amber tallow-lantern rim light (#d97706) from the right edge, deep shadows on the far side of the face. Clear dark outlines, hand-placed pixel clusters, limited palette with 4 shades per colour, light dithering on metal and cloth. Muted, matte, grimy materials: dented iron, boiled leather, wool and soot-stained linen; anything corrupted is jet-black bone, chalk-white calcified growth and glowing violet marrow. Colours only from this palette and its shades: void black #0a0b10, damp basalt #14171f, wet slate #1e232d, rotting oak #2c221e, scavenged iron #4b5563, polished steel #9ca3af, bleached linen #d9cbb4, cinder ash #6b7280, boiled leather #92400e, tallow amber #d97706, candle gold #fbbf24, arterial red #e11d48, rust crimson #991b1b, blight violet #9333ea, resonant orchid #c084fc, salt cyan #06b6d4; human skin #6b4a3a / #a47a5f / #d4a988, pale and underfed. The subject reads clearly against the dark: mid-tone figure, bright accents only on small details. Plain flat very dark background (#120e12), no gradient, no scenery, nothing behind the figure. No border, no frame, no text, no logo, no watermark.
Stylized, not realistic: an expressive, larger-than-life character design with a bold, readable silhouette and pushed shapes (jaw, brow, cheekbones, nose, neck and build exaggerated to show who the character is), like a dark fantasy game's character art rather than a painted photograph. The character block says how far to push: warm and readable, in between, or far (gaunt, elongated, uncanny or grotesque).
Hair exactly as the character block describes, with its own length and style; no hood, cowl, veil or tied-back hair unless the character block asks for it.
```

## Talia Rusk (`talia_rusk`)

Inbox AI-05. Target: `portraits/npcs/talia_rusk.png`; faction `hearth`. From `ART3_PROMPTS.md` with one change: the hair is CANON 3.2's and FOUNDER_PORTRAITS 3's **side-braid** (ART3's block says a low knot). Keep the ear clear of the braid.

```text
Character: Talia Rusk, Cipher Spymaster of the Basin. Stylization warm and readable: sharp observant eyes, weathered late-twenties lines around the mouth and brow, a small pointed chin, expressive dark brows, a dry guarded half-smile. A Spent woman of twenty-nine, lean and tough. Chestnut-brown hair worn in a single thick side-braid over her far shoulder, with stray soot-dulled strands, swept completely clear of her right ear. Her right ear, the deaf one on the near side toward the viewer (the left side of the picture), is fully visible: scarred and notched at the rim, puckered pale burn tissue running onto her jaw from an Inquisition flash-pot. Heavy grey-brown wool dispatch cloak pinned over patched leather, bone-cipher tally rods at her collar, the carved top of an ash-wood underarm crutch resting against her near shoulder.
Mood: sharp, watchful, unsentimental.
Main colours: damp wool grey-brown and boiled leather, carved pale bone rods, tallow amber rim light.
```

## Chirurgeon Sabina (`sabina_kress`)

Inbox AI-05. Target: `portraits/npcs/sabina_kress.png`; faction `hearth`. Use `ART3_PROMPTS.md`'s `sabina_kress` block unchanged (it already carries CANON 3.2's hooks); listed here so the batch is complete.

## Widow Varka (`widow_varka`)

Inbox AI-05. Target: `portraits/npcs/widow_varka.png`; faction `guild`. Age, hair and the bonnet are best guesses (CANON fixes only the waxed veil and brass salt-scales).

```text
Character: Widow Varka, Sluice Fence of the Water-Lock Basin. Stylization in between: a heavy-lidded, shrewd, fleshy face seen through a stiff black mourning veil glossy with tallow wax, which hangs from a narrow black bonnet brim to her chin, her features clearly visible through it; a thin painted mouth, a small gold hoop in her near ear. A broad-shouldered widow in her late forties, grey-black hair pinned up under the veil. A high-necked black mourning dress of oiled wool under a rubbered tallow-vat apron, a heavy ring of iron sluice keys at her collar on a chain. Her near hand holds up a small brass salt-scale, two shallow pans heaped with grey rock salt.
Lighting: the warm amber tallow rim light from the right, with a faint sickly yellow-green glint on the brass.
Mood: shrewd, mournful, transactional.
Main colours: void black and wet slate, waxed veil sheen, brass gold scales, grey rock salt, tallow amber rim.
```

## Ivo (`ivo`)

Inbox AI-05. Target: `portraits/npcs/ivo.png`; faction `hearth`. Tone rule: a child is never a fighter or a victim on screen; he is drawn as a courier, unhurt.

```text
Character: Ivo, Talia's Courier. Stylization warm and readable: a quick, bright, wary face, big dark eyes, an ash-smudged button nose, a gap-toothed half-grin. A thin boy of twelve, a non-combat courier. Shaggy dark hair sticking out under a too-big grey knitted wool cap pulled down to his brows. A patched, oversized wool coat with rolled sleeves, a canvas satchel strap across his chest, a bundle of notched bone cipher tallies tied on a cord at his chest, one tally stick held up between two fingers beside his chin.
Mood: quick, cheeky, watchful.
Main colours: damp wool grey, boiled leather brown, pale bone tallies, tallow amber rim.
```

## Overseer Grendel (`overseer_grendel`)

Inbox AI-06. Target: `portraits/bosses/overseer_grendel.png`; faction `guild`. Must not echo Dunstan (bare shoulders, chains): Grendel has the half-helm, mutton-chops and the maul; Dunstan is shaven with a wired jaw and cuffs.

```text
Character: Overseer Grendel, Master of the Fourth Kiln's bone-mill. Stylization in between, pushed brutish: a massive thick-necked man of forty-one, a heavy undershot jaw, a flattened nose, small hard eyes squinting from heat, ash and sweat on his skin. A riveted iron furnace half-helm covers his scalp and brow, its soot-black visor ridge pushed up; huge mutton-chop sideburns caked grey-white with bone dust. A scorched black leather foundry apron over bare, heavily muscled, burn-scarred shoulders, a chain of slag clinkers across his chest. Over his near shoulder rests the brass piston cylinder and iron head of a pneumatic piston-maul, a rubber pressure hose coiling from it.
Lighting: a furnace-orange rim light from the right instead of the amber tallow rim, a sickly yellow-green fill from below.
Mood: brutal, contemptuous, sweating.
Main colours: scavenged iron, soot black, scorched leather, brass piston, bone-dust white, furnace orange.
```

## Foundry-Master Jessop (`foundry_master_jessop`)

Inbox AI-06. Target: `portraits/bosses/foundry_master_jessop.png`; faction `guild`.

```text
Character: Foundry-Master Jessop, guildmaster of the Fourth Kiln Syndicate. Stylization in between, slick and well-fed: a fleshy face on clean rations, a neat pointed salt-and-pepper beard, a waxed moustache, a thin smug smile; round smoked-glass foundry lenses in brass frames hide his eyes, small furnace glints in them. A man of forty-three, thinning dark hair slicked back. A stiff grey-white asbestos-felt guildmaster's coat with a high collar and gilded brass trim, a heavy brass chain of office with a valve-wheel medallion, kid gloves; a rolled brass-tube blueprint case under his near arm.
Lighting: a sickly yellow-green vat rim light from the right instead of the amber tallow rim.
Mood: smooth, calculating, cold.
Main colours: asbestos grey-white, candle gold trim, brass, smoked dark lenses, copper-green stains, yellow-green rim.
```

## Inquisitor-General Ruthven (`inquisitor_general_ruthven`)

Inbox AI-06. Target: `portraits/bosses/inquisitor_general_ruthven.png`; faction `clerisy`. Silhouette check against `inquisitorial_purifier` (gas-mask hood) and Nemeth: Ruthven is the only bolted, eyeless flat mask.

```text
Character: Inquisitor-General Ruthven, the "Blind Eye", head of the Iron Inquisition. Stylization far, uncanny and faceless: his whole face is hidden behind a smooth, windowless black-iron mask with no eye holes, fixed by four large screw-bolts at the temples and jaw, a narrow breathing grille at the mouth; a deep crushed dent from a crossbow quarrel is punched into the mask's left eye-plate. Cropped iron-grey hair at the back of his skull. A tall spiked black-iron gorget, open at the front over his collarbones, where pulsing brass grafts are bolted into the skin, small glass windows in them glowing faint violet. A black-lacquered long coat with a high stiff collar. His head is tilted slightly, listening.
Lighting: a cold blue-violet phosphor rim light from the right instead of the amber tallow rim.
Mood: still, listening, merciless.
Main colours: void black lacquer, scavenged iron, brass grafts, faint blight violet glow, blue-violet rim.
```

## Chief Anatomist Lazar (`chief_anatomist_lazar`)

Inbox AI-06. Target: `portraits/bosses/chief_anatomist_lazar.png`; faction `clerisy`. His smooth face is the point: the opposite of gaunt Quell (`proctor_quell`).

```text
Character: Chief Anatomist Lazar of the Embalming Cloisters. Stylization far, uncanny: an unnaturally smooth, unlined, ageless face like polished wax, full unblemished cheeks, perfectly calm pale eyes, hairless brows, a faint serene smile; no visible age at all although he is old. A low gold mitre-cap. Crimson-and-pearl surgical vestments with pearl ossuary lace at the collar. Rising behind his near shoulder, a multi-jointed silver embalming armature: thin articulated metal arms ending in fine scalpels, clamps and a glass syringe, poised like a spider's legs.
Lighting: a cold candle-white key light and a violet rim light from the right instead of the amber tallow rim.
Mood: serene, curious, appallingly gentle.
Main colours: arterial red and rust crimson vestments, pearl and bleached linen, candle gold mitre-cap, polished steel armature, violet rim.
```

## Warden-Inquisitor Ulric (`warden_inquisitor_ulric`)

Inbox AI-06. Target: `portraits/bosses/warden_inquisitor_ulric.png`; faction `castellan`.

```text
Character: Warden-Inquisitor Ulric, the prototype Saint-Warden. Stylization far, grotesque: a broad, scarred human face whose whole lower jaw has turned to fused black stone, cracked basalt veined with faint violet, merging into the riveted brass collar of a pressure suit; tired, glassy human eyes above it. A shaven scalp with a riveted brass band. A massive Fourth-Kiln brass-and-iron pressure suit with dented gilded Watch cuirass plates, pressure gauges on the chest, thick rubber conduit hoses running from his shoulders back out of frame, a thread of steam leaking at a seam.
Lighting: a hard red-orange torch rim light from the right instead of the amber tallow rim.
Mood: dulled, enduring, half-gone.
Main colours: brass gold, scavenged iron, black basalt stone, faint violet veins, scraps of rust-crimson surcoat, red-orange rim.
```

## Cantor Ysolde (`cantor_ysolde`)

Inbox AI-06. Target: `portraits/bosses/cantor_ysolde.png`; faction `corrupted`. Age is a best guess. Keep the tuning-forks on the shoulders, not the head (no horns).

```text
Character: Cantor Ysolde of the Awakening Chorus. Stylization far, eerie but beautiful: a long pale face with half-lidded rapt eyes, lips parted mid-song, thin dark brows lifted, a faint violet light inside her open mouth; long ash-pale hair falling loose, stiff with dripped wax at the ends. A woman in her forties. Layered pearl-white choir robes crusted with dripped candle wax and woven sewer reed, a high choir collar; two long bone tuning-forks rise from her shoulders like antlers, shimmering with faint violet light.
Lighting: a bioluminescent violet-and-amber glow from below and a violet rim light from the right instead of the amber tallow rim.
Mood: rapt, ecstatic, unafraid of the end.
Main colours: pearl and bleached linen, candle wax, pale bone, resonant orchid, blight violet, tallow amber.
```

## Arch-Prelate Othemar (`arch_prelate_othemar`)

Inbox AI-06. Target: `portraits/bosses/arch_prelate_othemar.png`; faction `clerisy`.

```text
Character: Arch-Prelate Othemar, Head of the Sovereign Clerisy. Stylization far, grotesque: a shrunken, skull-thin old man of sixty-four kept alive long past his time, papery yellow skin drawn tight, sunken burning eyes, a trembling toothless mouth; the far (left) side of his skull has turned to cracked black stone creeping over his temple, and his raised left hand is petrified black. A towering cloth-of-gold cope with a collar of small carved pearl skulls, a tall jewelled mitre too large for his head. He sits sunk in a gilded reliquary throne whose carved back rises behind him; thin brass marrow tubes run from the throne's arms into the veins of his right wrist and neck.
Lighting: a cold candle-white key light and a violet rim light from the right instead of the amber tallow rim.
Mood: terrified, grasping, imperious.
Main colours: candle gold and cloth-of-gold, pearl white, black basalt stone, brass tubes, violet rim.
```

## The Calcified Amalgam (`calcified_amalgam`)

Inbox AI-08. Target: `portraits/bosses/calcified_amalgam.png`; faction `corrupted`. A 2x2 footprint boss: frame it as one mass that fills the square. All faces adult (tone rule).

```text
Creature: The Calcified Amalgam. Stylization grotesque, head and forequarters: several adult human bodies fused into one hulking mass by chalk-white calcified bone growth, three or four adult faces half-sunk into the white stone at different angles, mouths open, eyes blank; jet-black bone spines and tendrils split out of the mass, its cracks glowing with violet marrow light. Scraps of grey wool caught in the stone, no other clothing.
Lighting: violet marrow glow from inside the cracks and a cold rim light from the right instead of the amber tallow rim.
Mood: anguished, mindless, vast.
Main colours: chalk-white calcified growth, jet-black bone, blight violet glow, wet slate.
```

## Bailiff Enforcer (`bailiff_enforcer`)

Inbox AI-08. Target: `portraits/elites/bailiff_enforcer.png`; faction `castellan`. Must read apart from `captain_vane` and the common `warrior_base` Watch portrait: the collar ring and chain are the hook.

```text
Character: Bailiff Enforcer of the Castellan Watch, an Iron-Collar heavy. Stylization in between: a heavy brute's face with a flattened nose and close-set eyes under a dented wide-brimmed iron kettle helm, a thick neck clamped in a riveted black-iron collar ring. A dented iron cuirass over a rust-crimson surcoat; a length of heavy shackle chain with an open manacle wound over his near shoulder, the iron head of a heavy bailiff's maul resting against it.
Lighting: a hard red-orange torch rim light from the right instead of the amber tallow rim.
Mood: stolid, menacing, bored.
Main colours: scavenged iron, rust crimson, boiled leather, red-orange rim.
```
