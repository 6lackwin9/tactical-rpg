# FOUNDER_PORTRAITS: portrait briefs for the six founders

Status: picks switched to the Gemini route (user 2026-11-01): the open-ended Gemini takes (no reference; R1 prompts with the Hearth line, Gemini web-app queue) are now `art_src/<id>.png`, because they match the cast's Gemini portraits; the Klein picks stay as `art_src/<id>_klein_pick.png`. Was: approved for local draft renders (user 2026-10-27): FPQ1 bareheaded (the helm on the sprite and a later variant), FPQ2 cowl down, FPQ3 graft kept in the left eye, FPQ4 all six Hearth; FPQ5-FPQ9 follow the recommendations for the drafts (the user picks from the renders; nothing is placed until then). Was: proposal (2026-10-27, design lane).
Depends on: the user's approval of this spec; the LOCALGEN3 tooling (template route, `localgen.py vary`)
Lane: asset (the planner runs the local renders; the user picks)
Touches: `art_src/<id>.json` and `art_src/<id>.png` plus their takes (`kaelen`, `vesper`, `maren`, `hollis`,
`dunstan`, `elyse`); later `games/emberwake/assets/portraits/heroes/<id>.png` (a separate placing task)

## Goal

Give each founder of the Severed Root a full portrait brief, so their portraits can be the next local-art test
(LOCALGEN3 part 2, "the six founders' base portraits as the test") and replace the Ashlands placeholders that
[CAST1_ASSETS.md](CAST1_ASSETS.md) section 2 lists (Bryn Hale, Kit Vane, Alda Wren, Tev Quill, Dagny Coal, Sella Marsh).
Each brief has a house-format "Character:" block, a plain description for Klein, a base template and lighting line,
and the props Klein may get wrong. The six must read apart from each other and from the hub cast (Tess, Bram, Aline).

## 1. Sources and what this spec adds

- Facts (age, class, background, scars, grafts): [cast_registry.md](../lore/background/cast_registry.md) founders
  table, the class files `lore/systems/class_*.md` section 4 (Hollis's graft is in his **left** socket; Kaelen's
  spurs are on his **left** forearm under linen).
- Looks already in canon, used here: the art bible
  ([art_and_visual_style.md](../lore/specs/art_and_visual_style.md) 7.1) and the "Look:" lines of
  `lore/narrative/prologue_and_starter_squad_lore.md` (Kaelen, Vesper, Maren, Hollis) and
  `lore/narrative/six_founders_and_phase_progression.md` (Dunstan, Elyse). So the founders do have a short visual brief;
  this spec turns it into full blocks and fills the gaps.
- Gear: [CAST1_FOUNDERS.md](CAST1_FOUNDERS.md) and [CAST1_GEAR.md](CAST1_GEAR.md) (item names; keepsakes in 7).
- House format and rules: the ART3B blocks in [ART3_PROMPTS.md](ART3_PROMPTS.md) and `art_src/{tess,bram,aline,
  captain_vane}.json`; [assets-world.md](../preferences/assets-world.md) (stylized, founders warm and readable, varied
  hair, large features); `decisions/lore.md` rows "Faction lighting" and "Names and asset briefs"; the Klein findings
  of the LOCALGEN3 row in `docs/TASKS.md` (template route, narrow eyes, uncommon accessories).
- Marked "(proposal)" below: every look the sources leave open (hair, eye colour, skin tone, build words, Dunstan's
  burned side and brand, Vesper's whistle). Nothing here changes a name, an age or a background.
- Not read: the user's notes on the Scarlet Lullaby Worldbook page (artifact data, outside the repo). The planner checks
  the `character~<id>` notes before rendering (FPQ10).

## 2. Shared rules for all six

R1. Gemini route (the house record, and the fallback only when Klein fails, asked first): the ART3 style block, with
its lighting sentence ("Strong cool key light from the upper left, ... far side of the face.") replaced by the Hearth
line (R3), and the large-features sentence of the base template added after the stylization line: "Large,
well-defined features that stay readable when pixelated to a small game portrait: big clearly drawn eyes with distinct
whites, iris and dark lids, strong dark brows, a large defined mouth with clear lips, a slightly oversized head, and
bold dark outlines around the eyes, brows, nose and mouth." Then the founder's "Character:" block.

R2. Klein template route (preferred, user 2026-10-27): a Klein edit of the named Gemini base
(`C:/dev/localgen/templates/<base>.png`) with the probe lead (`template_test.LEAD`, "Turn the person in this portrait
into the character below ..."), then the founder's plain description, then the Hearth line. A text-to-image run sends
the description and the Hearth line only.

R3. Lighting: all six are Hearth (decision "Faction lighting"): `Lighting: a warm amber tallow-lantern rim light from
the right, soft warm fill light, deep warm brown shadows.` (the `hearth` line of `localgen/probe/faction_test.py`).
Exceptions are FPQ4.

R4. Framing (every take): head and shoulders, body, shoulders and face turned three-quarters to the viewer's right,
eyes looking right, the head filling the frame like the cast's portraits. The near side of the face is the founder's
right (the left of the picture), as with Tess's ear.

R5. Tone: every founder is 18-24 and drawn as an adult; the blocks say "an adult".

R6. Eyes: the three founders with narrowed eyes (Vesper, Hollis, Dunstan) carry the high-contrast eye wording: thick
dark upper-lid lines, bright irises, one catchlight each.

R7. Takes: 2 seeds a founder (1 and 2, as in the faction test), pick the best overall, then at most one fix-up edit
per missing signature (section 5); every take keeps its prompt record (`model`, base template, seed, prompt).

## 3. The six briefs

### 3.1 Kaelen (`kaelen`), the Vanguard

Base template `base_man`. Lighting Hearth (FPQ4: his Castellan past could argue for the Watch's red-orange torch).
Proposals: hair colour and length, grey eyes, build words, bareheaded (FPQ1).

```text
Character: Kaelen, the Vanguard. Stylization warm and readable: an honest, open face, a broad square jaw with short straw-blond stubble, a strong straight nose, heavy straight brows, large steady grey eyes, a wide mouth set in a hard, stubborn line. A young man of twenty-one, an adult, tall and broad-shouldered, a soldier's athletic build. Thick, wavy straw-blond hair worn loose to his shoulders, grown out from a guardsman's crop and pushed back off his brow, one lock falling forward; bareheaded. A dented, battered iron breastplate over a quilted bleached-linen gambeson, the cathedral cross on its chest chiselled out to a rough gouged scar in the iron; worn leather straps at the shoulders. His left fist grips the long wooden haft of a notched billhook upright in front of his chest, the haft rising past his far shoulder and the hooked, notched iron blade at the right edge of the frame. His left forearm crosses the bottom of the frame, bound in dirty linen soaked with old blood, three short, jagged chalk-white bone spurs breaking out through the wrapping along its outer edge, large and clearly readable at small size.
Mood: steadfast, guarded, quietly ashamed.
Main colours: scavenged iron and polished steel, bleached linen with rust crimson stains, straw-blond hair, chalk-white spurs.
```

Klein description:

```text
a steadfast young guardsman of twenty-one, an adult, tall and broad-shouldered, with an honest, open face: a broad square jaw with short straw-blond stubble, a strong straight nose, heavy straight brows, large steady grey eyes, a wide mouth set in a hard line. Thick wavy straw-blond hair loose to his shoulders, pushed back off his brow, one lock falling forward. A dented iron breastplate over a quilted linen gambeson, a rough gouged scar in the iron at the centre of the chest where a cross was chiselled away. His left fist grips the long wooden haft of a notched iron billhook in front of his chest, the hooked blade beside his head at the right edge of the frame; his left forearm, wrapped in dirty blood-stained linen, crosses the bottom of the frame, three jagged chalk-white bone spurs breaking out through the wrapping. Head and shoulders, his whole body, shoulders and face turned three-quarters toward the right, looking right, his head filling most of the frame; large, clearly drawn eyes, strong dark brows, a large defined mouth, bold dark outlines. A plain, flat, empty dark background.
```

### 3.2 Vesper (`vesper`), the Gutter-Stalker

Base template `base_lean` (keeps her apart from Tess on `base_woman`). Lighting Hearth. Proposals: hair, the cowl worn
down (FPQ2), one bodkin in frame (FPQ8), the tin runner's whistle (her keepsake "Runner Whistle" is itself a
proposal, CAST1_GEAR 7). Her wandering shadow is not drawn in a bust (out of scope).

```text
Character: Vesper, the Gutter-Stalker. Stylization warm and readable: a narrow, fox-like face, high sharp cheekbones, a small pointed nose, large, dark, sharply narrowed eyes with thick dark upper lids, bright irises and a catchlight each, sharp dark brows, a wide, sly, tight-lipped smirk; chimney soot smeared across both cheeks. A young woman of eighteen, an adult, small, lean and wiry. Long, straight, ink-black hair streaked with ash-grey, worn loose and falling past her shoulders, a ragged fringe swept to the far side, clear of her brows. A shredded charcoal chimney-sweep's cowl pushed down and bunched around her neck and shoulders, the hood down. A close-fitting jerkin of small overlapping slate-grey leather shingles, sooty wool wraps on her forearms, a small tin runner's whistle on a cord at her throat. Her near hand is raised beside her chin, holding a slim matte-black bodkin in a reversed grip, the blade down along her forearm, and thin wisps of grey smoke curl up from the fingertips of that hand, clearly readable at small size.
Mood: sly, feral, vengeful.
Main colours: void black and cinder ash charcoal, slate grey, pale smoke grey, a small polished steel glint.
```

Klein description:

```text
a sly, feral young thief of eighteen, an adult, small, lean and wiry, with a narrow fox-like face: high sharp cheekbones, a small pointed nose, large dark narrowed eyes with thick dark upper-lid lines, bright irises and a single bright catchlight each, sharp dark brows, a wide tight-lipped smirk, black chimney soot smeared across both cheeks. Long, straight, ink-black hair streaked with ash-grey, loose past her shoulders, a ragged fringe swept to one side above her brows. A shredded charcoal cowl bunched around her neck and shoulders, a close-fitting jerkin of small overlapping slate-grey leather scales, a small tin whistle on a cord at her throat. Her hand is raised beside her chin holding a slim black dagger point-down, and thin wisps of grey smoke curl up from her fingertips. Head and shoulders, her whole body, shoulders and face turned three-quarters toward the right, looking right, her head filling most of the frame; large, clearly drawn eyes, strong dark brows, a large defined mouth, bold dark outlines. A plain, flat, empty dark background.
```

### 3.3 Maren (`maren`), the Heretic Acolyte

Base template `base_woman`. Lighting Hearth. Proposals: hair, eye colour, the slight stoop, the habit's colour (dark
slate, to part her from Aline's linen habit; FPQ7). Canon: cracked round spectacles, the ripped Clerisy sigil, the
censer crook, ink-stained fingers, black throat veins from marrow-ink.

```text
Character: Maren, the Heretic Acolyte. Stylization warm and readable: a soft round face with full cheeks gone thin from hunger, a strong straight nose, thick dark-red brows drawn together, large earnest dark-brown eyes behind round wire spectacles with clear lenses, one lens cracked across in a thin bright line, full lips pressed into a firm line. A young woman of twenty-three, an adult, of medium height and build, slightly stooped from years at a copying desk. Long, thick, wavy rust-red hair worn loose and falling past her shoulders, tucked behind her near ear; bareheaded, no veil. A torn scribe's habit of dark slate-grey wool, its sleeves hacked short, blotted with ink, a pale torn patch on the breast where the Clerisy sigil was ripped away; the high collar torn open at the throat. Black veins like spidery lines of ink branch up the front of her throat from the collarbone to under her jaw, pure ink-black, clearly readable at small size. Her ink-blackened fingers grip the wooden shaft of a censer crook at the edge of the frame, a small bronze censer hanging from its hook.
Mood: solemn, studious, quietly defiant.
Main colours: wet slate and damp basalt wool, rust crimson hair, tallow amber bronze, ink black.
```

Klein description:

```text
a solemn, studious young scribe of twenty-three, an adult, of medium build, with a soft round face: full cheeks gone thin, a strong straight nose, thick dark-red brows drawn together, large earnest dark-brown eyes behind round wire spectacles with clear lenses, one lens cracked across, full lips pressed into a firm line. Long, thick, wavy rust-red hair loose past her shoulders, tucked behind her ear. A torn scribe's habit of dark slate-grey wool blotted with ink, a pale torn patch on the breast, the high collar torn open at the throat. Black veins like spidery lines of ink branch up the front of her throat to her jaw. Her ink-blackened fingers grip a wooden staff at the edge of the frame, a small bronze censer hanging from its hooked top. Head and shoulders, her whole body, shoulders and face turned three-quarters toward the right, looking right, her head filling most of the frame; large, clearly drawn eyes, strong dark brows, a large defined mouth, bold dark outlines. A plain, flat, empty dark background.
```

### 3.4 Hollis (`hollis`), the Arbalestier

Base template `base_lean`. Lighting Hearth (FPQ4: a crag poacher from outside the city; kept Hearth). Proposals:
hair (self-cut, echoing the self-cut graft), the sparse beard, the crooked nose, hazel eye. Canon: the left-socket
graft with a thin red glint, the horn recoil pauldron on the right shoulder, the oversized lever arbalest, notched
bone quarrels. His keepsake monocle is not drawn (Bram owns the eyepiece look; FPQ5).

```text
Character: Hollis, the Arbalestier. Stylization warm and readable: a lean, hungry, wind-weathered face, high cheekbones, a long crooked nose, a sparse young dark beard along the jaw, a wry, lopsided mouth. A young man of twenty, an adult, lean and rangy, wiry and hard. Short, ragged dark-brown hair hacked unevenly with a knife, sticking up in tufts, pale limestone dust in it. His right eye, on the near side toward the viewer (the left of the picture), is his own: a large, narrowed hunter's squint with a thick dark upper lid, a bright hazel iris and a catchlight. His left eye, on the far side, is a crude graft: a too-large, glossy jet-black animal eye with a thin glowing red slit pupil, ringed by puckered, roughly stitched pink scar tissue, the red glint clearly readable at small size. A stiff hunting brigandine of boiled leather studded with dull rivets, a heavy curved pauldron of pale polished horn strapped over his right shoulder as a recoil pad; the steel bow arms of an oversized lever-action crossbow show over his far shoulder, with the pale fletchings of notched bone quarrels beside them.
Mood: watchful, wry, haunted.
Main colours: boiled leather and rotting oak brown, pale horn and limestone-dust bleached linen, scavenged iron, an arterial red glint.
```

Klein description:

```text
a watchful young poacher of twenty, an adult, lean and rangy, with a hungry, wind-weathered face: high cheekbones, a long crooked nose, a sparse young dark beard, a wry lopsided mouth. Short, ragged dark-brown hair hacked unevenly, sticking up in tufts, pale dust in it. His near eye is a large narrowed squint with a thick dark upper-lid line, a bright hazel iris and a single bright catchlight; his far eye is a crude graft, a too-large glossy jet-black eye with a thin glowing red slit pupil, ringed by puckered pink stitched scar tissue. A stiff boiled-leather brigandine studded with rivets, a heavy curved shoulder plate of pale polished horn strapped over his near shoulder, the steel arms of a large crossbow rising over his far shoulder beside pale bone-fletched bolts. Head and shoulders, his whole body, shoulders and face turned three-quarters toward the right, looking right, his head filling most of the frame; large, clearly drawn eyes, strong dark brows, a large defined mouth, bold dark outlines. A plain, flat, empty dark background.
```

### 3.5 Dunstan "Slag-Jaw" (`dunstan`), the Slag-Brute

Base template `base_heavy`. Lighting Hearth (FPQ4: a Fourth Kiln stoker; furnace light would argue for an orange
underlight). Proposals: the shaven scalp, the burned side (his right, the near side, so it shows), the brand on the
near shoulder (Grendel branded him; place and shape unstated). Canon: towering, soot-stained, half the jaw burned by
slag and wired with scrap, shackles riveted round scarred wrists as knuckle guards, the Foundry Apron (CAST1 gear).

```text
Character: Dunstan "Slag-Jaw", the Slag-Brute. Stylization warm and readable, pushed toward bulk: a big blocky head on a thick bull neck, a heavy jutting brow, deep-set fierce eyes with thick dark upper lids, bright irises and a catchlight each, a broad flattened nose, a wide, grim mouth. A young man of twenty-two, an adult, towering and massively built, with huge sloping shoulders and arms. A shaven scalp with dark stubble. The right half of his jaw, on the near side toward the viewer (the left of the picture), is a mass of glossy, ridged slag-burn scar from the chin to below the ear, the jawbone held together by twists of black scrap-iron wire and a riveted iron strap along the jawline, clearly readable at small size; the other half of his jaw is whole, with dark stubble. Bare, soot-smeared shoulders and arms, a raised brand scar on his near shoulder, the top of a heavy scorched leather foundry apron at the bottom of the frame. His huge fists are raised in front of his chest, the knuckles broad as coal shovels, both scarred wrists locked in thick riveted iron penal cuffs worn as knuckle guards, a few links of broken chain hanging from them.
Mood: defiant, smouldering, loyal.
Main colours: soot void black and cinder ash, scavenged iron cuffs and wire, burn-scar pink and rust crimson, warm tallow light on bare skin.
```

Klein description:

```text
a towering young foundry brute of twenty-two, an adult, massively built, with huge sloping shoulders, a thick bull neck and a big blocky head: a heavy jutting brow, deep-set fierce eyes with thick dark upper-lid lines, bright irises and a single bright catchlight each, a broad flattened nose, a wide grim mouth. A shaven scalp with dark stubble. The near half of his jaw is a mass of glossy, ridged pink burn scar from chin to ear, held together by twists of black iron wire and a riveted iron strap along the jawline; the far half has dark stubble. Bare, soot-smeared shoulders and arms, a raised brand scar on his shoulder, the top of a scorched leather apron at the bottom of the frame. His huge fists are raised in front of his chest, both wrists locked in thick riveted iron cuffs with links of broken chain hanging from them. Head and shoulders, his whole body, shoulders and face turned three-quarters toward the right, looking right, his head filling most of the frame; large, clearly drawn eyes, strong dark brows, a large defined mouth, bold dark outlines. A plain, flat, empty dark background.
```

### 3.6 Elyse (`elyse`), the Ichor Alchemist

Base template `base_woman`. Lighting Hearth (FPQ4: she worked under the Tallow Vats, but against the Guild; its
yellow-green light would mark her as the Guild's). Not in the slice; render last. Proposals: hair, the deeper skin tone
(the dark and base steps of the skin ramp; FPQ5), the grin. Canon: slight frame, cracked brass goggles on the forehead,
a wicker bandolier of sulfur and white phosphorus flasks, a vitriol-stained coat (CAST1 "Rubbered Coat"; the narrative
look says an apothecary apron, FPQ6), copper-green stains, the pressurised brass sprayer.

```text
Character: Elyse, the Ichor Alchemist. Stylization warm and readable: a wide, lively face with broad cheekbones and full cheeks, a broad nose, large bright eager eyes, thick arched brows, a wide, crooked, mischievous grin showing her teeth; a smudge of copper-green on one cheek. A young woman of nineteen, an adult, slight and small, swamped by an oversized coat; her skin in the darker tones of the skin colours. A big, wild cloud of tight dark curls loose around her head and shoulders, one streak bleached pale by fumes, held back from her face by cracked brass goggles with round lenses pushed up on her forehead. An oversized, patched rubbered leather coat, dark and glossy, stained copper-green and pocked with pale vitriol burns; a wicker bandolier across her chest holding small glass flasks of yellow sulfur and faintly glowing white phosphorus. The brass nozzle of a pressurised sprayer on a hose rises over her near shoulder.
Mood: bright, reckless, sharp-witted.
Main colours: wet slate and void black glossy coat, salt cyan copper-green stains, candle gold brass, sulfur yellow and phosphorus white accents.
```

Klein description:

```text
a bright, reckless young alchemist of nineteen, an adult, slight and small in an oversized coat, with a wide, lively face and warm brown skin: broad cheekbones, full cheeks, a broad nose, large bright eager eyes, thick arched brows, a wide crooked grin showing her teeth, a smudge of green on one cheek. A big wild cloud of tight dark curls around her head and shoulders, one pale streak in them, held back by cracked brass goggles with round lenses pushed up on her forehead. An oversized patched dark rubber-leather coat, glossy, stained copper-green and pocked with pale acid burns; a woven wicker bandolier across her chest holding small glass flasks of yellow powder and glowing white liquid. A brass hose nozzle rises over her shoulder. Head and shoulders, her whole body, shoulders and face turned three-quarters toward the right, looking right, her head filling most of the frame; large, clearly drawn eyes, strong dark brows, a large defined mouth, bold dark outlines. A plain, flat, empty dark background.
```

## 4. How the nine read apart

The six founders beside the hub cast (Vane for the billhook). No two of the nine share hair colour and length; the
silhouette hook is what reads first at 120 px.

| Who | Template | Build | Hair | Face | Silhouette hook | Palette lead | Accent |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Tess | base_woman | wiry, compact | long, wild, one loose braid; dark brown | sharp, pointed chin, freckles | braid on the shoulder, satchel strap | boiled leather brown | tallow amber |
| Bram | base_heavy | barrel chest, 30s | shaggy mane to the shoulders, bushy beard; dark brown, grey | big square, flat nose | beard and loupe | cowhide brown | amber forge glow |
| Aline | (Gemini) | tall, gaunt | cropped with a razor; soot black | long, narrow, hollow cheeks | long neck, scalpel bandolier | linen and basalt | blight violet veins |
| Kaelen | base_man | tall, athletic | wavy, loose to the shoulders; straw blond | broad square jaw, stubble | billhook blade at the right edge, bound forearm across the bottom | iron and linen | chalk-white spurs |
| Vesper | base_lean | small, wiry | long, straight, fringe; ink black, ash streaks | narrow fox face, soot cheeks | cowl bunched at the neck, raised dagger hand | charcoal, near monochrome | smoke grey |
| Maren | base_woman | medium, stooped | long, thick, wavy, loose; rust red | soft round, spectacles | round spectacles in a mass of red hair | wet slate wool | ink-black throat veins, bronze |
| Hollis | base_lean | lean, rangy | short, ragged, self-cut; dusty dark brown | weathered, crooked nose, sparse beard | horn pauldron, crossbow arms over the shoulder | leather brown and pale horn | red graft glint |
| Dunstan | base_heavy | towering, massive, 22 | shaven scalp | blocky, bull neck, wired jaw | bare shoulders, raised cuffed fists | soot and iron | burn-scar pink |
| Elyse | base_woman | slight, small | big cloud of tight curls, one pale streak; dark | wide, grinning, deeper skin | goggles in the curls, nozzle over the shoulder | glossy dark coat, cyan stains | brass, sulfur yellow, phosphorus white |

Near pairs and what parts them:

- Kaelen and Captain Vane both carry a billhook: Vane's rises behind his shoulder over gilded plate and a crimson
  tabard; Kaelen's is plain iron held in front, his plate dull, his colours iron and linen, his hair blond and long.
- Bram and Dunstan both come from the Fourth Kiln on `base_heavy`: Dunstan is younger, shaven, beardless, bare-armed,
  with the wired jaw and the cuffs; Bram has the mane, the beard, the loupe and the shirt under his apron.
- Maren and Aline are both defrocked clergy with a defaced habit and dark neck veins: Maren's habit is dark slate wool
  (Aline's linen), her face round with spectacles (Aline's long and narrow), her hair long and red (Aline's cropped and
  black), her veins pure ink-black lines up the front of the throat (Aline's faint black-violet up the side).
- Vesper and Tess are both Ash-Crawl runners in shingled leather: Vesper is charcoal and black on the lean base with
  sleek straight hair and the cowl; Tess is brown on the woman base with wild hair and a braid.

## 5. Props Klein may get wrong

Measured on Bram's loupe (LOCALGEN3): Klein turns an uncommon accessory into the nearest common object about 3 times
in 5. Recipe (R7): 2 seeds, pick the best overall, then one fix-up edit naming the object as bright and high contrast;
a reference image of the object is the fallback. Edit wording (the object and the table's fix-up line filled in):

```text
Keep the same character, face shape, hair, scars and markings, outfit, pixel-art style, framing, lighting and dark background exactly. Change only <object>: <fix-up line>.
```

| Founder | Prop | Likely drift | Must read? | Fix-up line |
| --- | --- | --- | --- | --- |
| Kaelen | Bone spurs through the forearm linen | a spiked bracer or gauntlet, or a plain bandage | yes | his left forearm, wrapped in dirty blood-stained linen, has three large jagged chalk-white bone spurs bursting out through the wrapping, bright white against the cloth |
| Kaelen | Gouged cross on the breastplate | an intact cross or a clerical emblem | yes | the centre of his breastplate shows a rough, deep gouged scar in the dark iron where an emblem was chiselled away, bright scratched metal at its edges |
| Kaelen | Notched billhook | a halberd, axe or spear | no (any hooked polearm passes) | - |
| Kaelen | Forearm at the bottom edge | cut off by the head-filling crop | yes | crop looser for Kaelen, or raise the forearm higher across his chest |
| Vesper | Smoke from the fingertips | a cigarette or a pipe, or nothing | yes | thin bright pale-grey wisps of smoke curl up from all five fingertips of her raised hand |
| Vesper | Reversed bodkin, tin whistle | a forward grip, a pendant | no | - |
| Maren | Black ink veins at the throat | a tattoo, a choker, or violet veins | yes | spidery pure ink-black veins branch up the front of her bare throat from the collarbone to under her jaw, sharp against pale skin |
| Maren | Cracked spectacle lens | an uncracked lens | no | - |
| Maren | Censer crook | a lantern on a pole (clashes with the Hearth light) or a plain staff | no (drop it if it turns into a lantern) | - |
| Hollis | Graft eye in the far socket | an eyepatch, a monocle, or a plain red eye | yes | his far eye is a glossy jet-black graft eye, too large for the socket, with a thin bright glowing red slit pupil and a ring of pink stitched scar around it |
| Hollis | Horn recoil pauldron | ram's horns on the head, or a leather pauldron | yes | a smooth, curved plate of pale polished horn, cream-white, strapped flat over his near shoulder |
| Dunstan | Wired, burned half-jaw | a muzzle or mask, braces, or a beard | yes | the near half of his jaw is bare glossy pink burn scar, crossed by bright twisted black-iron wires and one riveted iron strap along the jawline; his mouth stays visible |
| Dunstan | Penal cuffs | bracelets or bracers | no (any chunky iron cuff passes) | - |
| Elyse | Pressurised sprayer nozzle | a pistol, a horn or a pipe | no (drop it if it turns into a gun) | - |
| Elyse | Wicker bandolier | a leather bandolier | no | - |
| All | Hair length from the base template | the template's short hair kept (Maren, Vesper, Kaelen, Elyse) | yes | her hair is long, thick and loose, falling past her shoulders (per the block) |
| All | The Hearth line | a lantern drawn in the frame (seen on Tess; her test added a sentence) | yes | remove it with an edit that names a plain, flat, empty dark background |

## 6. Acceptance checks (on the 120 px pixelized takes, the base image only as a small reference)

- A1 Framing (R4): every pick faces right, three-quarters, eyes right, the head filling the frame like Tess's.
- A2 Lighting (R3): an amber rim on the right edge and warm shadows; no cold key, violet rim, red underlight or green
  cast (those mark other factions).
- A3 Signatures: every row of section 5 marked "must read" is visible at 120 px on a contact sheet of the nine.
- A4 Eyes (R6): whites, iris and dark lids are separate pixels; Vesper, Hollis and Dunstan keep a catchlight; Hollis's
  red glint is at least a 2 px bright cluster.
- A5 Apart (section 4): the hair colour and length match the table; given the six names and the sheet, a viewer can
  match each portrait.
- A6 Tone (R5): all six read as adults.
- A7 Records (R7): every take has its record beside it; the pick goes to `art_src/<id>.png` with its `.json` only after
  the user picks.

## 7. Out of scope

- Sprites and tokens (the rig, SPRITE2), the wandering shadow (an effect, not portrait art), expressions (the
  PORTRAITFX1 cells come after a pick), face sidecars, and the Act 2 Calcification variants.
- Variant portraits (Kaelen in his kettle helm, Vesper with the cowl up): later, if FPQ1 or FPQ2 says so.
- Placing the picks in `games/emberwake/assets/portraits/heroes/` and pointing the founders' art at them through
  `aegis content` (task FOUNDERPORT1-P).
- Any image-API run: the Gemini blocks are the house record and the fallback, asked first.
- Keepsake icons and item art.

## 8. Open questions for the user

- FPQ1 Kaelen's kettle helm (art bible, prologue look; also the quest item "The Deserter's Kettle Helm"): (a)
  bareheaded base portrait, the helm in his sprite and a later variant (rec: the brim hides the hair and shades the
  eyes the large-features rule needs); (b) helm on; (c) helm pushed back with the hair showing.
- FPQ2 Vesper's charcoal cowl: (a) pushed down around her neck, hair loose (rec); (b) hood up (the current
  placeholder look; hides the hair the preference wants varied).
- FPQ3 Hollis's graft is in his left socket, the far side in a right-facing portrait: (a) keep it left; the red glint
  reads in the far eye (rec); (b) move it to the right eye in the registry and class file, as Tess's ear moved.
- FPQ4 Lighting: (a) all six Hearth (rec: lighting marks the cell they belong to now); (b) a faint origin hint on
  one or more (Kaelen's Castellan torch, Dunstan's furnace underlight, Elyse's chemical green, Hollis colder).
- FPQ5 Proposed looks to confirm or change: all hair (blond Kaelen, ink-black Vesper, rust-red Maren, self-cut dark
  Hollis, shaven Dunstan, dark curls Elyse); Elyse's deeper skin from the skin ramp; Dunstan's near-side burn and the
  brand on his shoulder; Vesper's tin whistle; Hollis without his keepsake monocle.
- FPQ6 Elyse's outfit: (a) the CAST1 Rubbered Coat (rec: Bram, Aline and Dunstan already wear aprons); (b) the
  narrative's patched apothecary apron.
- FPQ7 Maren and Aline overlap (defaced habit, dark neck veins): (a) keep both, apart by colour, face, hair and vein
  placement (rec); (b) drop Maren's torn sigil patch.
- FPQ8 Vesper's bodkins: (a) one in frame, reversed (rec; CAST1 gives her one Curfew Bodkin); (b) twin bodkins as the
  art bible says.
- FPQ9 The run: (a) template route, 2 seeds each, the slice four (Kaelen, Vesper, Maren, Hollis) first, then Dunstan
  and Elyse (rec); (b) all six at once; (c) more seeds.
- FPQ10 The Worldbook notes on the six (`character~kaelen` and so on) were not read here; the planner reads them
  before rendering and folds any change into the blocks.
