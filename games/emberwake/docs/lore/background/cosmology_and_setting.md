# Cosmology and setting: the dying world of Emberwake

Status: canon (2026-10-25, consistent with [CANON.md](../CANON.md)). Game: **Emberwake: A Scarlet Lullaby**. Names
and ages follow the [cast registry](cast_registry.md); what the Red Tithing really does is the
[central truth](central_truth.md) spoiler sheet (proposal), so this file states only what the Clerisy preaches and what
the player sees. Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]` (see CANON.md).

## 1. High concept

Emberwake takes place in an ash-choked medieval realm ruled by the **Sovereign Clerisy**, an ecclesiastical
aristocracy that holds power through feudal hierarchy and blood rites tied to a titan sleeping beneath the continental
crust.

The natural order of light, seasons and decay is broken. There is no sunlight, no seasonal rebirth and no ordinary
rot. The living work the cathedral factories and fear what seeps up from below.

## 2. Broken laws of nature

### 2.1 The Shroud

Two centuries ago an event the commoners call **the Extinguishing** put out the sun, the moon and the stars (not to be
confused with the **Extinguished** failure screen). What caused it stays a mystery. The Sovereign Clerisy was founded in
its dark as the order that holds the Tithing to pacify the titan; in secret, its founding act was the first Great
Tithing, recorded as the titan's wrath (user 2026-10-25, [CANON](../CANON.md) 3.6; the folk know none of this).

- In their place hangs **the Shroud**: a permanent bruised-violet canopy of petrified cloud, cold gloom and ash
  drifting from the highland cathedral kilns.
- Day and night no longer cycle. Labour shifts, curfews and tithes are kept by the tolling of the Clerisy's warded
  cathedral bells.
- Light comes only from torches, tallow fires and phosphorescent catacomb fungi.

### 2.2 The Low Sleep

The land rests on the calcified flesh and sleeping nervous system of an ancient titan, called in heretical texts
**the Low Sleep** or **the Slumbering Root**.

- The titan is not dead; it breathes and dreams.
- Its slow pulse causes tremors, sudden ground liquefaction (boiling tar-bogs that swallow streets) and the upward
  seepage of mutagenic black marrow into wells, coal seams and sewers.
- Near the seepages flesh warps and space bends: deep mining tunnels become shifting labyrinths.

### 2.3 Broken decomposition

Organic matter does not rot into soil.

- **Ossification:** uncremated flesh dries, blackens and hardens into brittle black bone.
- **Skinless wanderers:** a corpse buried without rites does not dissolve; the titan's marrow seeps into the sinew and
  knits it over weeks into a mindless, skinless husk.
- **The salt and sulfur rite:** the only permanent disposal is cremation with Rock Salt, crushed brimstone and Warded
  Tallow oil. The Clerisy rations salt and fuel, so poor families often sell their dead to the **Tallow Guild** or the
  **Fourth Kiln Syndicate** bone-mills to pay ancestral debts.

## 3. The generational divide

Communion draughts and decades of tallow fumes split the population by age (CANON.md section 2.4 and the glossary).

```text
AGE SPECTRUM
[ 0 ------------------------------------ 24 ]   [ 25 ---------------------------- 50+ ]
                The Unsealed                                 The Spent
  children (0-17): stakes, never fighters            compliant: decades of draughts
  18-24: the playable operatives and recruits        and tallow fumes
```

### 3.1 The Spent (25 and over)

Decades of tallow fumes, bone-mill dust and church communion draughts (laced with an ossifying sedative that
suppresses dissent) leave older people docile and stiff.

- **Body:** chalky, stiff joints, clouded eyes, brittle skin, shallow wheezing breath.
- **Mind:** fear deadened, replaced by dependence on cathedral warmth and the bell schedule.
- **Society:** the Spent police their own children, reporting runaways and heretics to the Iron Inquisition for extra
  salt vouchers, tallow candles and safety from the culling wagons.
- **The few who kept their minds:** adults who avoided communion keep their wits but calcify in body and cannot fight
  (Bram, Sister Aline, Bailiff Corvo: non-combat hub or quest NPCs). Agnes, at 44, is one of the oldest in the
  Ash-Crawl; the Spent rarely pass 45. Arch-Prelate Othemar at 64 is the visible exception (see the
  [central truth](central_truth.md)).

### 3.2 The Unsealed (under 25)

The young are the only supple, vital part of humanity.

- **Resilience:** their blood vessels are still elastic; they can exert full strength and look on aberrations without
  sinking into catatonia.
- **The coming line:** at 25 the Clerisy counts a person among the Spent. Unsealed who survive the press-gangs and the
  pens either give in to the draughts or take up heretical arms. This is a world fact, not a per-operative countdown
  (Act 1 has no countdown on any founder; whether operatives age during the campaign is open, see gaps).
- **The Red Tithing** takes Unsealed youths of any age, children raised for it first (refinement: from infancy, in the
  Clerisy's foundling houses, the Lullaby Houses; CANON 3.6); children are stakes in the story (Pip, Lucan, Ida), never
  fighters or on-screen victims.
- **The rebellion's core:** every operative of **the Severed Root** is 18-24 `[data]` (recruit ages, RECRUIT1).

## 4. Where the setting touches the systems

| Setting fact | System | Layer |
| --- | --- | --- |
| Warded Tallow keeps the dark at bay | Upkeep: 1 Warded Tallow per day of travel or rest; at 0, -2 starting AP in every battle until restocked | `[kit hook]` resource purse, `[data]` names and amounts |
| Rock Salt is rationed | Stash material (Slab flushes, bribes) | `[kit hook]` resource purse, `[data]` |
| The cell lives on salvage | Scrap is the currency (there is no gold) | `[kit hook]` resource purse, `[data]` |
| The titan's tremors and seepage | Battle hazards per region (see [world geography](world_geography_and_hub.md) section 4) | `[kit hook]` scheduled zone hazards, `[data]` numbers |
| The Red Tithing date | Doom Clock: visible from Act 1 day 1 at 42 days; relabelled "the Great Tithing" at 2.4; soft in Acts 1-2 (a lesser tithing at 0), hard in Act 3 | `[kit hook]` campaign meter, `[data]` |
| Marrow grafts and Calcification | Strain and Calcification, Act 2 on (CANON.md sections 2.2-2.3) | `[kit hook]` per-unit meter and stage track, `[data]` names |
| Bells keep the hours | Time is counted in days; there is no hour clock | `[kit hook]` travel clock (exists in `travel.rs`) |

## 5. Lexicon

These terms appear in dialogue, the quest log and item text.

- **The Shroud:** the permanent ash-choked violet sky that put out the sun.
- **The Extinguishing:** the event, two centuries ago, that put out the sky.
- **The Low Sleep:** the titan's slumber and, by extension, the titan, whose dreams bleed mutagenic black marrow.
- **The Red Tithing:** the recurring rite in which Unsealed youths are led in chains into the cathedral crypts and
  sacrificed. The Clerisy preaches that it is the lullaby that keeps the titan asleep and the continent whole. The folk
  know a yearly rite of hundreds (CANON 3.6).
- **The Great Tithing:** a Tithing of thousands, held once before at the Extinguishing and again this year; known only
  to the Clerisy's inner circle until mission 2.4.
- **Lesser tithing:** an out-of-season Tithing when the soft Doom Clock runs out in Acts 1-2.
- **The Lullaby Houses:** the Clerisy's foundling houses, a charity in the folk's eyes, where tithe-children are raised
  from infancy and sung the lullaby each night (planner refinement, CANON 3.6).
- **Warded Tallow:** rendered animal and human fat treated with herbs and brimstone; its yellow-green flame is the only
  light that repels the mist-aberrations from below.
- **Rock Salt:** rationed preserving salt for rites, flushes and bribes.
- **Scrap:** salvaged metal and goods, the cell's currency.
- **Marrow-debt:** what commoners owe the Tallow Guild and the Clerisy for salt, tallow lanterns and burial
  dispensations.
- **The Unsealed / the Spent:** everyone under 25 / everyone 25 and over.
- **Bone-mortar:** dense masonry paste of ground ossified skeletons, used to shore up the bastions of Upper
  Cinderreach.
- **The Sluice:** the runoff and sewage canals beneath the city, the insurrection's hidden arteries; the hub is the
  Sluice Undercroft.
- **Cold Steel:** the skill branch (and Bram's creed) of uncorrupted weapons, leverage and drill.
- **The Inoculation:** the forbidden surgery that grafts aberration marrow, glands or ichor into living flesh (Act 2
  on; never on anyone under 18).
- **Calcification:** the permanent hardening of soft tissue into black bone. For operatives it is a five-stage track
  (Vein-Blackening, Calloused Chitin, Sensory Rot, The Shakes, Terminal Drift) that advances only through a Flesh
  Breach or Tier III graft surgery; Sister Aline's bone-scraping is the only undo.
- **Preservationist / Flesh-Broker:** the two doctrines of the cell (Act 2 on).

## 6. Gaps in this file

- Size and frequency of the Red Tithing (hundreds, a thousand or thousands; yearly, per generation or on emergency):
  decided (user 2026-10-25, Q-04, Q-16): a yearly rite of hundreds, a Great Tithing of thousands at the Extinguishing
  and this year, lesser tithings when the soft clock runs out (CANON 3.6).
- The lexicon's "dreams bleed mutagenic black marrow" (section 5) against CANON 3.6, where the Clerisy's marrow is
  rendered from the dream-steeped remains of the tithed: whether the titan's own black marrow is a separate substance
  is story guide gap G-33.
- Whether operatives age, and what happens to one who turns 25 during the campaign: Open, see gaps.
