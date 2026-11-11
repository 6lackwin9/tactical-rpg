# FOUNDER_PORTRAITS: locked Day 1 portrait briefs for the six founders

Status: **`[LOCKED]` Founder visual briefs (`2026-11-10` Lore & Visual Hooks Pass)** — clean Day 1 mortal human base portraits (`0` supernatural taints in Act 1; Act 2+ corruption uses runtime `aegis_ui::portrait_fx` + Cranial/Inoculated signatures per [CANON.md](../lore/CANON.md) Section 3.1 & 3.3).
Depends on: [CANON.md](../lore/CANON.md) Sections 3.1-3.3; `LOCALGEN3` (`localgen.py vary`) / `/art-ui-session` (`ART_UI_REQUESTS.md` `AI-04`)
Lane: asset (`/art-ui-session`)
Touches: `art_src/<id>.json` and `art_src/<id>.png` (`kaelen`, `vesper`, `maren`, `hollis`, `dunstan`, `elyse`); `games/emberwake/assets/portraits/heroes/<id>.png` and `faces/<id>/*.png`

## Goal

Give each founder of **the Severed Root** a `[LOCKED]` Day 1 base portrait brief so `/art-ui-session` (`AI-04`) renders grounded, mortal Cold-Steel portraits and expression sheets (`PORTRAITFX1` / `PORTRAITFX2`) that read unmistakably apart from one another, from the visored/cowled Act 2 Recruit Cohorts, and from the three Spent Hub mentors (`Talia`, `Berek`, `Sabina`).

## 1. Shared Rules (`R1..R7`, `[LOCKED]`)

- **R1 (Gemini house block)**: Shared `ART3` 16-bit style block with the Hearth lighting sentence (`R3`) and large-features clause: *"Large, well-defined features that stay readable when pixelated to a small game portrait: big clearly drawn eyes with distinct whites, iris and dark lids, strong dark brows, a large defined mouth with clear lips, a slightly oversized head, and bold dark outlines around the eyes, brows, nose and mouth."*
- **R2 (Klein template route)**: Edit of `C:/dev/localgen/templates/<base>.png` with `template_test.LEAD`, plain Klein description, and `R3` Hearth line.
- **R3 (Hearth Faction Lighting)**: All six founders use Hearth rim light: `Lighting: a warm amber tallow-lantern rim light from the right, soft warm fill light, deep warm brown shadows.`
- **R4 (Framing)**: Head-and-shoulders bust turned three-quarters to the viewer's right, eyes looking right, head filling the frame; near side = character's right (viewer's left); barefaced (`FPQ1`/`FPQ2` settled: bareheaded Kaelen, cowl-down Vesper) so Act 2's visored/cowled Recruit Cohorts never clone them.
- **R5 (Adult Unsealed & Clean Day 1 Human Rule)**: Every founder is `18-24` ("an adult") and **100% mortal Cold-Steel human on Day 1**—occupational/regime scars, soot, and scavenged gear only. Zero bone spurs, glowing beast-eyes, black marrow throat veins, or fingertip smoke in the base portrait (`Section 7`).
- **R6 (High-contrast narrowed eyes)**: `Vesper`, `Hollis`, and `Dunstan` carry thick dark upper-lid lines, bright irises, and one catchlight per eye.
- **R7 (Takes & Fix-ups)**: 2 seeds (`1` & `2`), pick best overall, at most one targeted prop fix-up (`Section 5`), record saved alongside `art_src/<id>.json`.

## 2. The Six Locked Day 1 Briefs

### 2.1 Kaelen (`kaelen`), the Vanguard — `base_man`

Lullaby-House orphan drafted at `14` into `Capt. Vane`'s Tithe-Escort Watch; chiselled the Clerisy cross off his breastplate and broke `Cpl. Fitch`'s jaw in `Order 44`.

```text
Character: Kaelen, the Vanguard. Stylization warm and readable: an honest, open face, a broad square jaw with short straw-blond stubble, a strong straight nose, heavy straight brows, large steady grey eyes, a wide mouth set in a hard, guilty line. A young man of twenty-one, an adult, tall and broad-shouldered, a soldier's athletic build. Thick, wavy straw-blond hair worn loose to his shoulders, grown out from a guardsman's crop and pushed back off his brow, one lock falling forward; bareheaded. A dented, battered iron breastplate over a quilted bleached-linen gambeson, the cathedral cross on its chest chiselled out to a rough gouged scar in the iron; worn leather straps at the shoulders. His left fist grips the long wooden haft of a notched iron billhook upright in front of his chest, the hooked blade at the right edge of the frame; his left inner wrist and forearm cross the bottom edge, wrapped in blood-spotted linen partially revealing a faded dark Lullaby-House ward-number brand on bare human skin (no bone spurs).
Mood: steadfast, guarded, quietly guilty.
Main colours: scavenged iron and steel, bleached linen with rust-crimson stains, straw-blond hair, warm amber rim.
```

Klein description:

```text
a steadfast young guardsman of twenty-one, an adult, tall and broad-shouldered, with an honest face: a broad square jaw with short straw-blond stubble, a strong straight nose, heavy straight brows, large steady grey eyes, a wide mouth set in a hard line. Thick wavy straw-blond hair loose to his shoulders, pushed back off his brow, bareheaded. A dented iron breastplate over a quilted linen gambeson, a rough gouged scar in the iron at the centre of the chest where a cross was chiselled away. His left fist grips the wooden haft of a notched iron billhook upright in front of his chest, the hooked blade at the right edge; his left forearm crosses the bottom wrapped in blood-spotted linen showing a faded dark ward-number brand at the wrist on plain human skin. Head and shoulders turned three-quarters right, looking right; large clearly drawn eyes, strong dark brows, bold dark outlines, plain flat dark background.
```

### 2.2 Vesper (`vesper`), the Gutter-Stalker — `base_lean`

`Talia`'s fastest rooftop cipher-runner who unknowingly carried `Orlov`'s sweep map that hung her crew.

```text
Character: Vesper, the Gutter-Stalker. Stylization warm and readable: a narrow, fox-like face, high sharp cheekbones, a small pointed nose, large dark sharply narrowed eyes with thick dark upper lids, bright irises and a catchlight each, sharp dark brows, a tight razor smirk; chimney soot smeared across both cheeks. A young woman of eighteen, an adult, small, lean and wiry. Long, straight, ink-black hair streaked with ash-grey, worn loose past her shoulders, a ragged fringe swept to the far side clear of her brows. A shredded charcoal cowl pushed down around her neck, a close-fitting jerkin of overlapping slate-grey leather shingles, sooty wool forearm wraps, and a small tin runner's whistle on a cord at her throat. Her soot-and-sealing-wax stained hand is raised beside her chin holding a slim matte-black bodkin in a reversed grip (no smoke from fingertips).
Mood: sly, feral, self-reproachful.
Main colours: void black and cinder-ash charcoal, slate grey leather, tin and steel glints.
```

Klein description:

```text
a sly, feral young courier of eighteen, an adult, small, lean and wiry, with a narrow fox-like face: high sharp cheekbones, small pointed nose, large dark narrowed eyes with thick dark upper-lid lines and a bright catchlight each, sharp dark brows, a tight smirk, black chimney soot on both cheeks. Long straight ink-black hair streaked with ash-grey, loose past her shoulders, ragged side fringe, hood down. Shredded charcoal cowl bunched at her neck, close-fitting slate-grey leather-shingle jerkin, small tin whistle on a throat cord. Her soot-stained hand is raised beside her chin holding a slim matte-black dagger point-down, plain human fingers with no smoke. Head and shoulders turned three-quarters right, looking right; large clearly drawn eyes, bold dark outlines, plain flat dark background.
```

### 2.3 Maren (`maren`), the Heretic Acolyte — `base_woman`

St. Lyra's liturgical scribe and mortuary novice whose mentor was vivisected by `Proctor Quell` under `Othemar`'s seal.

```text
Character: Maren, the Heretic Acolyte. Stylization warm and readable: a soft round face gone thin from strain, a strong straight nose, thick dark-red brows drawn together, large earnest dark-brown eyes behind round wire spectacles with clear lenses (one lens cracked in a thin bright line), full lips pressed firm. A young woman of twenty-three, an adult, medium build with a slight copyist's stoop. Long, thick, wavy rust-red hair worn loose past her shoulders, tucked behind her near ear; bareheaded. A torn dark slate-grey wool mortuary-scribe habit blotted with ink, a pale torn patch on the breast where St. Lyra's sigil was ripped off, a stitched brown leather embalmer's cuff at her wrist, and a clean pale human throat (no black throat veins). Her gall-ink blackened fingers grip the wooden shaft of a bronze censer crook at the right edge of the frame.
Mood: solemn, studious, quietly defiant.
Main colours: wet slate wool, rust-red hair, bronze censer amber, gall-ink black fingertips.
```

Klein description:

```text
a solemn young scribe-novice of twenty-three, an adult, with a soft round face gone thin: strong straight nose, thick dark-red brows drawn together, large earnest dark-brown eyes behind round wire spectacles with one cracked lens, firm full lips. Long thick wavy rust-red hair loose past her shoulders, tucked behind her ear, bareheaded. Torn dark slate-grey wool habit blotted with ink, pale ripped sigil patch on the breast, clean pale throat with no veins, stitched leather embalmer cuff. Her ink-stained fingers grip a wooden crook staff with a small bronze censer hanging from its hook. Head and shoulders turned three-quarters right, looking right; large clearly drawn eyes, bold dark outlines, plain flat dark background.
```

### 2.4 Hollis (`hollis`), the Arbalestier — `base_lean`

Outer-crag poacher whose heavy quarrel punched through `Inquisitor-General Ruthven`'s visor during the midnight raid that swept sister `Wren` (`14`) toward High Bastion.

```text
Character: Hollis, the Arbalestier. Stylization warm and readable: a lean, wind-weathered hunter's face, high cheekbones, a long crooked nose, a sparse young dark beard along the jaw, a wry lopsided mouth. A young man of twenty, an adult, lean and rangy. Short, ragged dark-brown hair hacked unevenly with a knife, dusted with pale limestone. Both eyes are natural human hazel eyes with thick dark upper lids and bright catchlights; his left eye (far side) narrows in a hunter's squint crossed vertically by a pale visor-shrapnel scar through the brow (no animal graft eye on Day 1). A small brass focusing monocle hangs on a leather cord at his collar. A stiff riveted boiled-leather brigandine with a heavy curved pale polished horn recoil pauldron strapped over his near (right) shoulder; steel bow-arms of an oversized lever-arbalest and bone-fletched quarrels rise over his far shoulder.
Mood: watchful, wry, haunted.
Main colours: boiled leather brown, pale polished horn, limestone dust, scavenged steel and brass glint.
```

Klein description:

```text
a watchful young crag hunter of twenty, an adult, lean and rangy, with a weathered face: high cheekbones, crooked nose, sparse dark jaw beard, wry mouth. Short knife-cropped dusty dark-brown hair. Both eyes are natural human hazel eyes in a hunter's squint with bright catchlights, a pale vertical scar crossing his far brow; a small brass monocle hangs on a cord at his collar. Stiff riveted boiled-leather brigandine, a curved cream-white horn shoulder plate over his near shoulder, steel crossbow arms and bone-fletched bolts over his far shoulder. Head and shoulders turned three-quarters right, looking right; large clearly drawn eyes, bold dark outlines, plain flat dark background.
```

### 2.5 Dunstan "Slag-Jaw" (`dunstan`), the Slag-Brute — `base_heavy`

Debtor stoker whose jaw `Overseer Grendel` burned on a slag-ladle when Dunstan tried to pull his own Spent father off the bone-mill chute.

```text
Character: Dunstan "Slag-Jaw", the Slag-Brute. Stylization warm and readable, pushed toward bulk: a big blocky head on a thick bull neck, heavy jutting brow, deep-set fierce eyes with thick dark upper lids and bright catchlights, broad flattened nose, wide grim mouth. A young man of twenty-two, an adult, towering and massively built; shaven scalp with dark stubble. The near (right) half of his jaw is glossy ridged slag-burn scar from chin to ear, held together by twisted black iron wire and a riveted iron jawline strap; the far half has dark stubble. Bare soot-smeared shoulders with a raised Fourth Kiln debtor brand on the near shoulder, scorched foundry apron below, huge fists raised in front of his chest locked in thick riveted iron penal cuffs with broken chain links.
Mood: defiant, smouldering, loyal.
Main colours: soot black and cinder ash, scavenged iron cuffs and wire, burn-scar pink, warm tallow rim.
```

Klein description:

```text
a towering young foundry stoker of twenty-two, an adult, massively built, bull-necked with a shaven scalp: heavy brow, deep-set fierce eyes with thick dark upper lids and catchlights, broad flat nose, grim mouth. Near half of his jaw is glossy pink burn scar held together by twisted black iron wire and a riveted iron strap along the jawline; far half has dark stubble. Bare soot-smeared shoulders with a raised brand scar, top of a scorched leather apron, huge fists raised in thick riveted iron cuffs with broken chain links. Head and shoulders turned three-quarters right, looking right; large clearly drawn eyes, bold dark outlines, plain flat dark background.
```

### 2.6 Elyse (`elyse`), the Ichor Alchemist — `base_woman`

Tallow-Vat chemical assayer who tapped `Orlov`'s steam-lines to brew free tallow and an alum counter-tonic that woke her Spent mother.

```text
Character: Elyse, the Ichor Alchemist. Stylization warm and readable: a wide lively face with warm deeper brown skin, broad cheekbones, large bright eager eyes, thick arched brows, a wide crooked grin showing teeth, copper-green smudge on one cheek. A young woman of nineteen, an adult, slight and small inside an oversized coat. A big wild cloud of tight dark curls around head and shoulders with one fume-bleached pale streak, held back by cracked round brass assayer goggles pushed up on her forehead. Oversized patched dark rubbered-leather Vat coat stained copper-green and pocked with pale acid burns; wicker bandolier holding small glass phials of yellow sulfur, glowing white phosphorus, and pale-blue alum tonic; pressurized brass sprayer nozzle on a hose over her near shoulder.
Mood: bright, reckless, sharp-witted.
Main colours: glossy dark rubbered coat, copper-green stains, brass gold, sulfur yellow, phosphorus white, alum blue.
```

Klein description:

```text
a bright reckless young chemist of nineteen, an adult, slight in an oversized coat, warm brown skin: wide lively face, large eager eyes, thick arched brows, crooked grin, green smudge on cheek. Big cloud of tight dark curls with one pale bleached streak, held back by cracked round brass goggles on her forehead. Oversized glossy dark rubber-leather coat stained copper-green with pale acid marks; wicker chest bandolier with small yellow, white, and pale-blue glass phials; brass sprayer nozzle over shoulder. Head and shoulders turned three-quarters right, looking right; large clearly drawn eyes, bold dark outlines, plain flat dark background.
```

## 3. Silhouette Separation Across Founders & Hub Mentors (`[LOCKED]`)

| Who | Template | Hair | Eyewear (Unique!) | Primary Silhouette Hook (`120 px`) |
| --- | --- | --- | --- | --- |
| **Talia Rusk** (`29`, Basin) | `base_woman` | Chestnut-brown side-braid | *None* | Flash-burned right ear, wool dispatch cloak, **ash-wood crutch top & bone-cipher rods** |
| **Berek Dorn** (`33`, Hearth) | `base_heavy` | Grey-streaked dark mane & beard | **Right-eye iron loupe** | Bushy beard + strapped loupe + shirted rivet apron (`bram.png` `1:1`) |
| **Chirurgeon Sabina** (`31`, Slab) | Gemini | Silver-templed cropped black | *None* | Dark oiled-green prosector collar, **unhooked leather surgical half-mask**, pipettes & scalpels |
| **Kaelen** (`21`) | `base_man` | Wavy loose straw-blond | *None* | Gouged iron breastplate, Lullaby wrist-brand under linen, upright billhook |
| **Vesper** (`18`) | `base_lean` | Straight ink-black w/ ash streaks | *None* | Cowl bunched down, shingle jerkin, tin whistle, reversed bodkin |
| **Maren** (`23`) | `base_woman` | Wavy loose rust-red | **Cracked wire spectacles** | Round spectacles in red hair, ripped slate habit, bronze censer crook |
| **Hollis** (`20`) | `base_lean` | Knife-cropped dusty brown | *Corded collar monocle* | Scarred left squint, pale horn recoil pauldron, arbalest bow-arms |
| **Dunstan** (`22`) | `base_heavy` | Shaven scalp | *None* | Bare shoulders, iron-wired slag-burned jaw, raised chained penal cuffs |
| **Elyse** (`19`) | `base_woman` | Dark curls w/ 1 bleached streak | **Forehead brass goggles** | Goggles in curls, copper-green rubbered coat, phial bandolier & brass nozzle |

## 4. Klein Prop Watchlist & Fix-up Lines

| Founder | Prop | Likely Drift | Fix-up Line (if needed) |
| --- | --- | --- | --- |
| `kaelen` | Gouged cross & wrist brand | Intact cross or gauntlet | rough gouged scar in chest iron where an emblem was chiselled off; blood-spotted wrist linen partially showing a faded dark number tattoo on plain skin |
| `vesper` | Reversed bodkin & tin whistle | Forward dagger or pendant | slim matte-black dagger point-down beside chin in soot-smudged fingers, small tin whistle on throat cord |
| `maren` | Cracked spectacles & censer crook | Lantern on pole (drop if lantern) | round thin wire spectacles with one cracked lens; clean pale throat with no veins |
| `hollis` | Horn recoil pauldron & natural eyes | Ram horns on head or eyepatch | smooth cream-white curved horn plate strapped flat on near shoulder; both eyes natural hazel with a pale scar over far brow |
| `dunstan` | Iron-wired slag-burned right jaw | Full muzzle/mask or beard | near half of jaw is bare glossy pink burn scar crossed by twisted black iron wire and one riveted jawline strap, mouth visible |
| `elyse` | Brass sprayer nozzle | Firearm/pistol (never allow guns!) | curved brass chemical hose nozzle over shoulder (or omit if Klein draws a gun) |

## 5. Three-Tier Corruption Visuals (`Act 2+`, Not Drawn on Day 1 Base)

1. **Day 1 Base (`Tier 1`)**: Clean portraits above (`stays unchanged on Preservationist runs`).
2. **Runtime `aegis_ui::portrait_fx` + Act 2 Inoculated Keystones (`Tier 2`)**: `50+` Strain pulses `smoke`/`glow`; Stage I adds `veins`; Stage II-III adds pallor `tint` + marrow `eye_glow` + `ash`; Stage IV adds `shiver` + black-tear `mark`. Taking a founder's Act 2 Cranial/Inoculated path manifests their personal taint (`Kaelen` forearm spurs, `Vesper` fingertip ash-smoke, `Maren` black throat veins, `Hollis` left amber tapetum pin, `Dunstan` fused basalt jaw, `Elyse` crystalline geode burns).
3. **Node `3.4` Silent Ossuary (`Tier 3`)**: Bespoke Saint-Weapon (`48x48` P1 -> `96x96` `2x2` P2) for the `M5` captive only.
