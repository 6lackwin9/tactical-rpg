# AI-04: founder Day 1 portraits (runnable prompt file)

Art/UI batch 2026-11-12, inbox row AI-04. Laid out for `tools/portraits/generate.py run <model> <id>... --n 2 --prompts art_src/prompts/ai04_founders.md`: the style block below is sent first, then the id's block. The style block is ART3's (`ART3_PROMPTS.md`) plus the `FOUNDER_PORTRAITS.md` R1 large-features clause, R3 Hearth lighting and R4/R5 framing. Each character block is copied **word for word** from the `[LOCKED]` briefs in `FOUNDER_PORTRAITS.md` section 2 (generated from that file; if a brief changes there, regenerate this file rather than editing it here). Reference: `art_src/mara_voss.png` (style only). Two seeds each (R7); pick one, then run the founder's faces sheet from `ai_faces.md`.

Check every take against `FOUNDER_PORTRAITS.md` section 3 (silhouette table) and section 4 (prop watchlist): no Day 1 taint, the eyewear only where the table gives it.

```text
Use the attached image (Mara Voss) only as the reference for pixel-art style and framing: the same pixel-art technique, level of detail, framing, lighting, outline weight and flat dark background. Do not copy her face, her hair, her build or her proportions: this is a different subject with its own stylized proportions.
Pixel art character portrait for a gothic dark fantasy tactical RPG set in a sunless church city lit only by tallow lanterns, in the style of detailed 16-bit SNES/GBA-era RPG portraits. Square 1:1 image. Head and shoulders (for a creature: head and forequarters), the head filling about half the frame, three-quarter view facing slightly to the viewer's right, eyes looking right. Strong cool key light from the upper left, a warm amber tallow-lantern rim light (#d97706) from the right edge, deep shadows on the far side of the face. Clear dark outlines, hand-placed pixel clusters, limited palette with 4 shades per colour, light dithering on metal and cloth. Muted, matte, grimy materials: dented iron, boiled leather, wool and soot-stained linen; anything corrupted is jet-black bone, chalk-white calcified growth and glowing violet marrow. Colours only from this palette and its shades: void black #0a0b10, damp basalt #14171f, wet slate #1e232d, rotting oak #2c221e, scavenged iron #4b5563, polished steel #9ca3af, bleached linen #d9cbb4, cinder ash #6b7280, boiled leather #92400e, tallow amber #d97706, candle gold #fbbf24, arterial red #e11d48, rust crimson #991b1b, blight violet #9333ea, resonant orchid #c084fc, salt cyan #06b6d4; human skin #6b4a3a / #a47a5f / #d4a988, pale and underfed. The subject reads clearly against the dark: mid-tone figure, bright accents only on small details. Plain flat very dark background (#120e12), no gradient, no scenery, nothing behind the figure. No border, no frame, no text, no logo, no watermark.
Stylized, not realistic: an expressive, larger-than-life character design with a bold, readable silhouette and pushed shapes (jaw, brow, cheekbones, nose, neck and build exaggerated to show who the character is), like a dark fantasy game's character art rather than a painted photograph. The character block says how far to push: warm and readable, in between, or far (gaunt, elongated, uncanny or grotesque).
Hair exactly as the character block describes, with its own length and style; no hood, cowl, veil or tied-back hair unless the character block asks for it.

Large, well-defined features that stay readable when pixelated to a small game portrait: big clearly drawn eyes with distinct whites, iris and dark lids, strong dark brows, a large defined mouth with clear lips, a slightly oversized head, and bold dark outlines around the eyes, brows, nose and mouth.
Lighting: a warm amber tallow-lantern rim light from the right, soft warm fill light, deep warm brown shadows.
Framing: head-and-shoulders bust turned three-quarters to the viewer's right, eyes looking right, head filling the frame; bareheaded. Day 1: a fully mortal human, only scars, soot and scavenged gear; no bone spurs, no glowing eyes, no dark veins, no smoke.
```

## Kaelen (`kaelen`)

Target: `games/emberwake/assets/portraits/heroes/kaelen.png` (pixelized from `art_src/kaelen.png`).

```text
Character: Kaelen, the Vanguard. Stylization warm and readable: an honest, open face, a broad square jaw with short straw-blond stubble, a strong straight nose, heavy straight brows, large steady grey eyes, a wide mouth set in a hard, guilty line. A young man of twenty-one, an adult, tall and broad-shouldered, a soldier's athletic build. Thick, wavy straw-blond hair worn loose to his shoulders, grown out from a guardsman's crop and pushed back off his brow, one lock falling forward; bareheaded. A dented, battered iron breastplate over a quilted bleached-linen gambeson, the cathedral cross on its chest chiselled out to a rough gouged scar in the iron; worn leather straps at the shoulders. His left fist grips the long wooden haft of a notched iron billhook upright in front of his chest, the hooked blade at the right edge of the frame; his left inner wrist and forearm cross the bottom edge, wrapped in blood-spotted linen partially revealing a faded dark Lullaby-House ward-number brand on bare human skin (no bone spurs).
Mood: steadfast, guarded, quietly guilty.
Main colours: scavenged iron and steel, bleached linen with rust-crimson stains, straw-blond hair, warm amber rim.
```

## Vesper (`vesper`)

Target: `games/emberwake/assets/portraits/heroes/vesper.png` (pixelized from `art_src/vesper.png`).

```text
Character: Vesper, the Gutter-Stalker. Stylization warm and readable: a narrow, fox-like face, high sharp cheekbones, a small pointed nose, large dark sharply narrowed eyes with thick dark upper lids, bright irises and a catchlight each, sharp dark brows, a tight razor smirk; chimney soot smeared across both cheeks. A young woman of eighteen, an adult, small, lean and wiry. Long, straight, ink-black hair streaked with ash-grey, worn loose past her shoulders, a ragged fringe swept to the far side clear of her brows. A shredded charcoal cowl pushed down around her neck, a close-fitting jerkin of overlapping slate-grey leather shingles, sooty wool forearm wraps, and a small tin runner's whistle on a cord at her throat. Her soot-and-sealing-wax stained hand is raised beside her chin holding a slim matte-black bodkin in a reversed grip (no smoke from fingertips).
Mood: sly, feral, self-reproachful.
Main colours: void black and cinder-ash charcoal, slate grey leather, tin and steel glints.
```

## Maren (`maren`)

Target: `games/emberwake/assets/portraits/heroes/maren.png` (pixelized from `art_src/maren.png`).

```text
Character: Maren, the Heretic Acolyte. Stylization warm and readable: a soft round face gone thin from strain, a strong straight nose, thick dark-red brows drawn together, large earnest dark-brown eyes behind round wire spectacles with clear lenses (one lens cracked in a thin bright line), full lips pressed firm. A young woman of twenty-three, an adult, medium build with a slight copyist's stoop. Long, thick, wavy rust-red hair worn loose past her shoulders, tucked behind her near ear; bareheaded. A torn dark slate-grey wool mortuary-scribe habit blotted with ink, a pale torn patch on the breast where St. Lyra's sigil was ripped off, a stitched brown leather embalmer's cuff at her wrist, and a clean pale human throat (no black throat veins). Her gall-ink blackened fingers grip the wooden shaft of a bronze censer crook at the right edge of the frame.
Mood: solemn, studious, quietly defiant.
Main colours: wet slate wool, rust-red hair, bronze censer amber, gall-ink black fingertips.
```

## Hollis (`hollis`)

Target: `games/emberwake/assets/portraits/heroes/hollis.png` (pixelized from `art_src/hollis.png`).

```text
Character: Hollis, the Arbalestier. Stylization warm and readable: a lean, wind-weathered hunter's face, high cheekbones, a long crooked nose, a sparse young dark beard along the jaw, a wry lopsided mouth. A young man of twenty, an adult, lean and rangy. Short, ragged dark-brown hair hacked unevenly with a knife, dusted with pale limestone. Both eyes are natural human hazel eyes with thick dark upper lids and bright catchlights; his left eye (far side) narrows in a hunter's squint crossed vertically by a pale visor-shrapnel scar through the brow (no animal graft eye on Day 1). A small brass focusing monocle hangs on a leather cord at his collar. A stiff riveted boiled-leather brigandine with a heavy curved pale polished horn recoil pauldron strapped over his near (right) shoulder; steel bow-arms of an oversized lever-arbalest and bone-fletched quarrels rise over his far shoulder.
Mood: watchful, wry, haunted.
Main colours: boiled leather brown, pale polished horn, limestone dust, scavenged steel and brass glint.
```

## Dunstan "Slag-Jaw" (`dunstan`)

Target: `games/emberwake/assets/portraits/heroes/dunstan.png` (pixelized from `art_src/dunstan.png`).

```text
Character: Dunstan "Slag-Jaw", the Slag-Brute. Stylization warm and readable, pushed toward bulk: a big blocky head on a thick bull neck, heavy jutting brow, deep-set fierce eyes with thick dark upper lids and bright catchlights, broad flattened nose, wide grim mouth. A young man of twenty-two, an adult, towering and massively built; shaven scalp with dark stubble. The near (right) half of his jaw is glossy ridged slag-burn scar from chin to ear, held together by twisted black iron wire and a riveted iron jawline strap; the far half has dark stubble. Bare soot-smeared shoulders with a raised Fourth Kiln debtor brand on the near shoulder, scorched foundry apron below, huge fists raised in front of his chest locked in thick riveted iron penal cuffs with broken chain links.
Mood: defiant, smouldering, loyal.
Main colours: soot black and cinder ash, scavenged iron cuffs and wire, burn-scar pink, warm tallow rim.
```

## Elyse (`elyse`)

Target: `games/emberwake/assets/portraits/heroes/elyse.png` (pixelized from `art_src/elyse.png`).

```text
Character: Elyse, the Ichor Alchemist. Stylization warm and readable: a wide lively face with warm deeper brown skin, broad cheekbones, large bright eager eyes, thick arched brows, a wide crooked grin showing teeth, copper-green smudge on one cheek. A young woman of nineteen, an adult, slight and small inside an oversized coat. A big wild cloud of tight dark curls around head and shoulders with one fume-bleached pale streak, held back by cracked round brass assayer goggles pushed up on her forehead. Oversized patched dark rubbered-leather Vat coat stained copper-green and pocked with pale acid burns; wicker bandolier holding small glass phials of yellow sulfur, glowing white phosphorus, and pale-blue alum tonic; pressurized brass sprayer nozzle on a hose over her near shoulder.
Mood: bright, reckless, sharp-witted.
Main colours: glossy dark rubbered coat, copper-green stains, brass gold, sulfur yellow, phosphorus white, alum blue.
```
