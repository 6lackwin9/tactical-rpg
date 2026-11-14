# AI-04, AI-06: expression sheets (runnable prompt file)

Art/UI batch 2026-11-12. Laid out for `tools/portraits/generate.py run <model> <id>... --faces --prompts art_src/prompts/ai_faces.md`, the same as `PORTRAITFX1_FACES.md`: the style block below is PORTRAITFX1's, copied unchanged; each id's `reference:` line is the picked portrait source (run the portrait first). Cut each sheet with `tools/portraits/sheets.py faces <sheet.png> <id> <hero|boss> --faction <f>`, then `face.py` for the sidecars. Cells: Neutral top-left, Hurt top-right, Angry bottom-left, Afraid bottom-right. Talia's and Sabina's sheets are already in `PORTRAITFX1_FACES.md` (run them from there). Widow Varka, Ivo, the Amalgam and the Bailiff Enforcer get no sheet in this batch.

```text
Use the attached image as the reference: the same person: face, hair, outfit, colours, framing, lighting and pixel-art style; only the expression changes.
A 2x2 grid of four pixel art character portrait variations for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. 1:1 square image. No borders, no frame, no text, no dividers, no logo. Plain flat very dark background (#120e12) behind all figures. Each variation shows the same head-and-shoulders, three-quarter view of the character below, the head the same size and in the same place in every cell. Each variation utilizes a 16-bit pixel art style with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per color, and light dithering on metal and cloth. Strong cool key light from the upper left and a warm amber tallow-lantern rim light (#d97706) from the right. Same outfit, hair and colours in all four.

Keep every accessory, scar, marking and piece of equipment exactly as in the reference in all four cells, including eyepieces, masks and jewellery.

The four variations are:
- Top-Left: Neutral: the character's usual expression (the Mood line below), mouth closed, eyes looking right.
- Top-Right: Hurt: in sharp pain: both eyes squeezed tightly shut with deep dark creases, brows pinched up and together, mouth pulled wide open in a grimace showing clenched teeth, strong dark lines around the eyes and mouth.
- Bottom-Left: Angry: a deep scowl, brows drawn down hard into a V, nostrils flared, teeth bared or jaw clenched, eyes narrowed and glaring, cheeks flushed red; unmistakably furious, clearly different from Neutral.
- Bottom-Right: Afraid: eyes wide, brows raised, mouth slightly open, head drawn in.
```

## Kaelen (`kaelen`)

reference: art_src/kaelen.png

Inbox AI-04; tier `hero`, faction `hearth`. Character lines from `FOUNDER_PORTRAITS.md` 2 (word for word).

```text
Character: Kaelen, the Vanguard. Stylization warm and readable: an honest, open face, a broad square jaw with short straw-blond stubble, a strong straight nose, heavy straight brows, large steady grey eyes, a wide mouth set in a hard, guilty line. A young man of twenty-one, an adult, tall and broad-shouldered, a soldier's athletic build. Thick, wavy straw-blond hair worn loose to his shoulders, grown out from a guardsman's crop and pushed back off his brow, one lock falling forward; bareheaded. A dented, battered iron breastplate over a quilted bleached-linen gambeson, the cathedral cross on its chest chiselled out to a rough gouged scar in the iron; worn leather straps at the shoulders. His left fist grips the long wooden haft of a notched iron billhook upright in front of his chest, the hooked blade at the right edge of the frame; his left inner wrist and forearm cross the bottom edge, wrapped in blood-spotted linen partially revealing a faded dark Lullaby-House ward-number brand on bare human skin (no bone spurs).
Mood: steadfast, guarded, quietly guilty.
Main colours: scavenged iron and steel, bleached linen with rust-crimson stains, straw-blond hair, warm amber rim.
```

## Vesper (`vesper`)

reference: art_src/vesper.png

Inbox AI-04; tier `hero`, faction `hearth`. Character lines from `FOUNDER_PORTRAITS.md` 2 (word for word).

```text
Character: Vesper, the Gutter-Stalker. Stylization warm and readable: a narrow, fox-like face, high sharp cheekbones, a small pointed nose, large dark sharply narrowed eyes with thick dark upper lids, bright irises and a catchlight each, sharp dark brows, a tight razor smirk; chimney soot smeared across both cheeks. A young woman of eighteen, an adult, small, lean and wiry. Long, straight, ink-black hair streaked with ash-grey, worn loose past her shoulders, a ragged fringe swept to the far side clear of her brows. A shredded charcoal cowl pushed down around her neck, a close-fitting jerkin of overlapping slate-grey leather shingles, sooty wool forearm wraps, and a small tin runner's whistle on a cord at her throat. Her soot-and-sealing-wax stained hand is raised beside her chin holding a slim matte-black bodkin in a reversed grip (no smoke from fingertips).
Mood: sly, feral, self-reproachful.
Main colours: void black and cinder-ash charcoal, slate grey leather, tin and steel glints.
```

## Maren (`maren`)

reference: art_src/maren.png

Inbox AI-04; tier `hero`, faction `hearth`. Character lines from `FOUNDER_PORTRAITS.md` 2 (word for word).

```text
Character: Maren, the Heretic Acolyte. Stylization warm and readable: a soft round face gone thin from strain, a strong straight nose, thick dark-red brows drawn together, large earnest dark-brown eyes behind round wire spectacles with clear lenses (one lens cracked in a thin bright line), full lips pressed firm. A young woman of twenty-three, an adult, medium build with a slight copyist's stoop. Long, thick, wavy rust-red hair worn loose past her shoulders, tucked behind her near ear; bareheaded. A torn dark slate-grey wool mortuary-scribe habit blotted with ink, a pale torn patch on the breast where St. Lyra's sigil was ripped off, a stitched brown leather embalmer's cuff at her wrist, and a clean pale human throat (no black throat veins). Her gall-ink blackened fingers grip the wooden shaft of a bronze censer crook at the right edge of the frame.
Mood: solemn, studious, quietly defiant.
Main colours: wet slate wool, rust-red hair, bronze censer amber, gall-ink black fingertips.
```

## Hollis (`hollis`)

reference: art_src/hollis.png

Inbox AI-04; tier `hero`, faction `hearth`. Character lines from `FOUNDER_PORTRAITS.md` 2 (word for word).

```text
Character: Hollis, the Arbalestier. Stylization warm and readable: a lean, wind-weathered hunter's face, high cheekbones, a long crooked nose, a sparse young dark beard along the jaw, a wry lopsided mouth. A young man of twenty, an adult, lean and rangy. Short, ragged dark-brown hair hacked unevenly with a knife, dusted with pale limestone. Both eyes are natural human hazel eyes with thick dark upper lids and bright catchlights; his left eye (far side) narrows in a hunter's squint crossed vertically by a pale visor-shrapnel scar through the brow (no animal graft eye on Day 1). A small brass focusing monocle hangs on a leather cord at his collar. A stiff riveted boiled-leather brigandine with a heavy curved pale polished horn recoil pauldron strapped over his near (right) shoulder; steel bow-arms of an oversized lever-arbalest and bone-fletched quarrels rise over his far shoulder.
Mood: watchful, wry, haunted.
Main colours: boiled leather brown, pale polished horn, limestone dust, scavenged steel and brass glint.
```

## Dunstan "Slag-Jaw" (`dunstan`)

reference: art_src/dunstan.png

Inbox AI-04; tier `hero`, faction `hearth`. Character lines from `FOUNDER_PORTRAITS.md` 2 (word for word).

```text
Character: Dunstan "Slag-Jaw", the Slag-Brute. Stylization warm and readable, pushed toward bulk: a big blocky head on a thick bull neck, heavy jutting brow, deep-set fierce eyes with thick dark upper lids and bright catchlights, broad flattened nose, wide grim mouth. A young man of twenty-two, an adult, towering and massively built; shaven scalp with dark stubble. The near (right) half of his jaw is glossy ridged slag-burn scar from chin to ear, held together by twisted black iron wire and a riveted iron jawline strap; the far half has dark stubble. Bare soot-smeared shoulders with a raised Fourth Kiln debtor brand on the near shoulder, scorched foundry apron below, huge fists raised in front of his chest locked in thick riveted iron penal cuffs with broken chain links.
Mood: defiant, smouldering, loyal.
Main colours: soot black and cinder ash, scavenged iron cuffs and wire, burn-scar pink, warm tallow rim.
```

## Elyse (`elyse`)

reference: art_src/elyse.png

Inbox AI-04; tier `hero`, faction `hearth`. Character lines from `FOUNDER_PORTRAITS.md` 2 (word for word).

```text
Character: Elyse, the Ichor Alchemist. Stylization warm and readable: a wide lively face with warm deeper brown skin, broad cheekbones, large bright eager eyes, thick arched brows, a wide crooked grin showing teeth, copper-green smudge on one cheek. A young woman of nineteen, an adult, slight and small inside an oversized coat. A big wild cloud of tight dark curls around head and shoulders with one fume-bleached pale streak, held back by cracked round brass assayer goggles pushed up on her forehead. Oversized patched dark rubbered-leather Vat coat stained copper-green and pocked with pale acid burns; wicker bandolier holding small glass phials of yellow sulfur, glowing white phosphorus, and pale-blue alum tonic; pressurized brass sprayer nozzle on a hose over her near shoulder.
Mood: bright, reckless, sharp-witted.
Main colours: glossy dark rubbered coat, copper-green stains, brass gold, sulfur yellow, phosphorus white, alum blue.
```

## Overseer Grendel (`overseer_grendel`)

reference: art_src/overseer_grendel.png

Inbox AI-06; tier `boss`, faction `guild`.

```text
Character: Overseer Grendel, Master of the Fourth Kiln's bone-mill. Stylization in between, pushed brutish: a massive thick-necked man of forty-one, a heavy undershot jaw, a flattened nose, small hard eyes squinting from heat, ash and sweat on his skin. A riveted iron furnace half-helm covers his scalp and brow, its soot-black visor ridge pushed up; huge mutton-chop sideburns caked grey-white with bone dust. A scorched black leather foundry apron over bare, heavily muscled, burn-scarred shoulders, a chain of slag clinkers across his chest. Over his near shoulder rests the brass piston cylinder and iron head of a pneumatic piston-maul, a rubber pressure hose coiling from it.
Lighting: a furnace-orange rim light from the right instead of the amber tallow rim, a sickly yellow-green fill from below.
Mood: brutal, contemptuous, sweating.
Main colours: scavenged iron, soot black, scorched leather, brass piston, bone-dust white, furnace orange.
```

## Foundry-Master Jessop (`foundry_master_jessop`)

reference: art_src/foundry_master_jessop.png

Inbox AI-06; tier `boss`, faction `guild`.

```text
Character: Foundry-Master Jessop, guildmaster of the Fourth Kiln Syndicate. Stylization in between, slick and well-fed: a fleshy face on clean rations, a neat pointed salt-and-pepper beard, a waxed moustache, a thin smug smile; round smoked-glass foundry lenses in brass frames hide his eyes, small furnace glints in them. A man of forty-three, thinning dark hair slicked back. A stiff grey-white asbestos-felt guildmaster's coat with a high collar and gilded brass trim, a heavy brass chain of office with a valve-wheel medallion, kid gloves; a rolled brass-tube blueprint case under his near arm.
Lighting: a sickly yellow-green vat rim light from the right instead of the amber tallow rim.
Mood: smooth, calculating, cold.
Main colours: asbestos grey-white, candle gold trim, brass, smoked dark lenses, copper-green stains, yellow-green rim.
```

## Inquisitor-General Ruthven (`inquisitor_general_ruthven`)

reference: art_src/inquisitor_general_ruthven.png

Inbox AI-06; tier `boss`, faction `clerisy`. Ruthven's mask hides the face: the four expressions read through head tilt, the mouth grille and the grafts' glow; accept a sheet where only those change.

```text
Character: Inquisitor-General Ruthven, the "Blind Eye", head of the Iron Inquisition. Stylization far, uncanny and faceless: his whole face is hidden behind a smooth, windowless black-iron mask with no eye holes, fixed by four large screw-bolts at the temples and jaw, a narrow breathing grille at the mouth; a deep crushed dent from a crossbow quarrel is punched into the mask's left eye-plate. Cropped iron-grey hair at the back of his skull. A tall spiked black-iron gorget, open at the front over his collarbones, where pulsing brass grafts are bolted into the skin, small glass windows in them glowing faint violet. A black-lacquered long coat with a high stiff collar. His head is tilted slightly, listening.
Lighting: a cold blue-violet phosphor rim light from the right instead of the amber tallow rim.
Mood: still, listening, merciless.
Main colours: void black lacquer, scavenged iron, brass grafts, faint blight violet glow, blue-violet rim.
```

## Chief Anatomist Lazar (`chief_anatomist_lazar`)

reference: art_src/chief_anatomist_lazar.png

Inbox AI-06; tier `boss`, faction `clerisy`.

```text
Character: Chief Anatomist Lazar of the Embalming Cloisters. Stylization far, uncanny: an unnaturally smooth, unlined, ageless face like polished wax, full unblemished cheeks, perfectly calm pale eyes, hairless brows, a faint serene smile; no visible age at all although he is old. A low gold mitre-cap. Crimson-and-pearl surgical vestments with pearl ossuary lace at the collar. Rising behind his near shoulder, a multi-jointed silver embalming armature: thin articulated metal arms ending in fine scalpels, clamps and a glass syringe, poised like a spider's legs.
Lighting: a cold candle-white key light and a violet rim light from the right instead of the amber tallow rim.
Mood: serene, curious, appallingly gentle.
Main colours: arterial red and rust crimson vestments, pearl and bleached linen, candle gold mitre-cap, polished steel armature, violet rim.
```

## Warden-Inquisitor Ulric (`warden_inquisitor_ulric`)

reference: art_src/warden_inquisitor_ulric.png

Inbox AI-06; tier `boss`, faction `castellan`.

```text
Character: Warden-Inquisitor Ulric, the prototype Saint-Warden. Stylization far, grotesque: a broad, scarred human face whose whole lower jaw has turned to fused black stone, cracked basalt veined with faint violet, merging into the riveted brass collar of a pressure suit; tired, glassy human eyes above it. A shaven scalp with a riveted brass band. A massive Fourth-Kiln brass-and-iron pressure suit with dented gilded Watch cuirass plates, pressure gauges on the chest, thick rubber conduit hoses running from his shoulders back out of frame, a thread of steam leaking at a seam.
Lighting: a hard red-orange torch rim light from the right instead of the amber tallow rim.
Mood: dulled, enduring, half-gone.
Main colours: brass gold, scavenged iron, black basalt stone, faint violet veins, scraps of rust-crimson surcoat, red-orange rim.
```

## Cantor Ysolde (`cantor_ysolde`)

reference: art_src/cantor_ysolde.png

Inbox AI-06; tier `boss`, faction `corrupted`.

```text
Character: Cantor Ysolde of the Awakening Chorus. Stylization far, eerie but beautiful: a long pale face with half-lidded rapt eyes, lips parted mid-song, thin dark brows lifted, a faint violet light inside her open mouth; long ash-pale hair falling loose, stiff with dripped wax at the ends. A woman in her forties. Layered pearl-white choir robes crusted with dripped candle wax and woven sewer reed, a high choir collar; two long bone tuning-forks rise from her shoulders like antlers, shimmering with faint violet light.
Lighting: a bioluminescent violet-and-amber glow from below and a violet rim light from the right instead of the amber tallow rim.
Mood: rapt, ecstatic, unafraid of the end.
Main colours: pearl and bleached linen, candle wax, pale bone, resonant orchid, blight violet, tallow amber.
```

## Arch-Prelate Othemar (`arch_prelate_othemar`)

reference: art_src/arch_prelate_othemar.png

Inbox AI-06; tier `boss`, faction `clerisy`.

```text
Character: Arch-Prelate Othemar, Head of the Sovereign Clerisy. Stylization far, grotesque: a shrunken, skull-thin old man of sixty-four kept alive long past his time, papery yellow skin drawn tight, sunken burning eyes, a trembling toothless mouth; the far (left) side of his skull has turned to cracked black stone creeping over his temple, and his raised left hand is petrified black. A towering cloth-of-gold cope with a collar of small carved pearl skulls, a tall jewelled mitre too large for his head. He sits sunk in a gilded reliquary throne whose carved back rises behind him; thin brass marrow tubes run from the throne's arms into the veins of his right wrist and neck.
Lighting: a cold candle-white key light and a violet rim light from the right instead of the amber tallow rim.
Mood: terrified, grasping, imperious.
Main colours: candle gold and cloth-of-gold, pearl white, black basalt stone, brass tubes, violet rim.
```
