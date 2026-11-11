# World geography and the hub: the three tiers of Cinderreach

Status: `[LOCKED]` 3-tier vertical strata on `sunk_ward` (`campaign.act` `1..3`) & persistent Sluice Undercroft facility loops (`Slab`, `Hearth`, `Basin`, `Hollows/Crypts`); `[PROVISIONAL]` Act 2-3 regional hazards (`CL-05`) & story-only forward bases; **`[PLACEHOLDER]` place/facility names & aesthetic palettes** ([CANON.md](../CANON.md) Section 2.2 wins on conflict). Related: [Sunk Ward](sunk_ward_campaign_map.md), [High Bastion](upper_bastion_campaign_map.md), [Mantle Fissure](mantle_fissure_campaign_map.md), [safehouse economy](../systems/safehouse_economy_and_triage.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. The three tiers

Cinderreach is built into the caldera of an extinct volcano whose depths hold the sleeping titan, the Low Sleep. All
three tiers live as act-gated region graphs (`campaign.act` 1-3) inside the single campaign world
`games/emberwake/data/worlds/sunk_ward/` `[data]` (`ashlands` stays the classic testbed and parity world).

```text
==========================================================================================
                          THE VERTICAL STRATA OF CINDERREACH
==========================================================================================
  [ UPPER CINDERREACH / THE HIGH BASTION ]            ACT 2: THE FIRST RUPTURE (proposal)
  |-- St. Lyra's Cathedral and the Grand Belltower
  |-- the Gilded Boulevards and the Clerisy sanctuaries
  |-- Tallow Guild reservoirs, culling stockades
  '-- proposal forward base: the Scribe's Crypt
                         |
                  [ THE SLAG GATE ]  (the Crucible, Act 1 climax)
                         |
  [ LOWER CINDERREACH / THE SUNK WARD ]               ACT 1: THE YOUNG VANGUARD (canon)
  |-- Ash-Crawl Tenements and warrens
  |-- the Fourth Kiln (Syndicate forge and bone-mill)
  |-- the Tallow Vats and the Sluice canals
  '-- the hub: the Sluice Undercroft (6 bunks)
                         :
  (Act 2 to Act 3: down through the cathedral's Reliquary Crypts and the Grand Spire Gate)
                         :
  [ THE SUB-BASALT ABYSS / THE MANTLE FISSURE ]       ACT 3: THE AWAKENING ABYSS (proposal)
  |-- the Weeping Basalt Chasm and petrified arches
  |-- the Marrow Conduits and the Chorus Spire
  |-- the High Altar and the Dreaming Core
  '-- proposal forward base: the Weeping Redoubt
==========================================================================================
```

## 2. The hub through the campaign

One hub for the whole campaign is preferred for scope: the Sluice Undercroft. The Scribe's Crypt (Act 2) and the
Weeping Redoubt (Act 3) stay `proposal` forward bases, used only if the story needs one. Bunks: 6 at the start of
Act 1 (4 filled after the prologue); Act 1 holds only the six founders, and the bunk the Crucible empties takes the
first procedural recruit at the start of Act 2 (user 2026-10-25, Q-38); 8 only through Berek's "Expanded Timber Frames"
project from Act 2, chosen by the player; a forward base inherits the count `[kit hook]` hub facilities with levels,
`[data]` project costs (Scrap, by the balance lane). Guests take no bunk until sworn in. Squad: up to 4 deployed
`[kit hook]` (exists: `PARTY_MAX`).

| Act | Base | Facilities | Bunks |
| --- | --- | --- | --- |
| Act 1 (canon) | The Sluice Undercroft | Apothecary's Slab, Tinker's Hearth, Water-Lock Basin, Dry Hollows (Isolation Crypts inside) | 6 (the founders only) |
| Act 2 (proposal) | The Sluice; optional forward base the Scribe's Crypt | as the Sluice, plus the Courier Blind | 6, 8 after the project; inherited by the forward base |
| Act 3 (proposal) | The Sluice; optional forward base the Weeping Redoubt | as the Sluice, plus the Quench-Anvil and the Ward-Lantern Rig | inherited |

Today's code: `hub.rs` `Hub` is the in-game menu, `shop.rs` `Trade` the shop, the stash and HEAL1 rest exist; facilities
with levels do not exist yet `[kit hook]`.

### 2.1 The Sluice Undercroft (canon)

An old municipal storm-drain three levels beneath the basalt cobbles of Lower Cinderreach.

1. **The Apothecary's Slab** (Chirurgeon Sabina): a granite embalming slab. Act 1: trauma surgery for wounded founders
   `[kit hook]` unit lifecycle (a wounded founder deploys at -20% max HP until a rest, user 2026-10-25, Q-31; a
   baseline the balance lane revisits). Act 2 on: graft surgery,
   Rock Salt flushes (effect open) and, after her quest, bone-scraping (-1 Calcification stage, -10% max HP for good)
   `[kit hook]` exchange recipes, `[data]`.
2. **The Tinker's Hearth** (Berek "Cinder-Lung" Dorn): a ventless charcoal forge with water baffles. Cold Steel
   re-tempering, shield reinforcement, stagger weighting and the bunk project, paid in Scrap `[kit hook]` exchange
   recipes and facility levels, `[data]`.
3. **The Water-Lock Basin** (Talia Rusk): the central pool behind a submerged portcullis with charcoal baffles; the dead-drop
   for bone-ciphers, the mission board, the launch point onto the district graph and the guest quarters `[kit hook]`
   mission board, `[data]` quests.
4. **The Dry Hollows:** six brick kiln bunks for rest and recovery `[kit hook]` HEAL1 rest (exists), 1 Warded Tallow per
   rest day `[data]`. The **Isolation Crypts** inside the Hollows hold operatives at Calcification Stage IV between
   battles (Act 2 on).

### 2.2 The Scribe's Crypt (Act 2 forward base, proposal)

Found in the flooded catacombs beneath St. Lyra's Cathedral, reached from node 08 (the Slag Gate breach).

- It does not add bunks: the count is inherited (6, or 8 after Berek's project).
- **Graft surgery:** Apex grafting opens at Sabina's Apothecary's Slab after Act 2 M2.4. Which facilities a forward base copies from the Sluice is open (see gaps).
- **The Courier Blind:** intercepts church dispatches and shows Inquisition patrol routes and culling wagon positions
  on the district map `[kit hook]` world-map markers, `[data]`.

### 2.3 The Weeping Redoubt (Act 3 forward base, proposal)

A fortified cavern hammered into the petrified marrow arches beside the Mantle Fissure (node 16).

- **The Quench-Anvil:** Berek's expedition forge, burning titan ichor to forge Black-Iron penetrators and stagger
  counterweights `[data]` recipes.
- **The Ward-Lantern Rig:** a Warded Tallow beacon that keeps the Abyss's atmospheric Strain off resting operatives
  `[kit hook]` hub facility, `[data]`.
- **Isolation Crypts:** iron cages where Stage IV operatives are restrained during bell tremors.

## 3. The three theatres

### 3.1 Act 1: the Sunk Ward (canon)

- **Tone:** industrial grimdark, drowning grime, squalor, survival.
- **Landmarks:** the Drowned Runoff Grate, the Ash-Crawl Tenements, the Fourth Kiln, the Tallow Vats, the Crucible
  Watchpost, the Slag Gate.
- **Map:** [sunk_ward_campaign_map.md](sunk_ward_campaign_map.md).

### 3.2 Act 2: the High Bastion and St. Lyra's (proposal)

- **Tone:** gothic monumentalism, oppressive wealth, magnesium glare, clerical terror, grafted corruption behind
  gilded fronts.
- **Landmarks:** the Slag Gate breach, the Gilded Boulevard, the Tallow Reservoir, the Embalming Cloisters, the
  Culling Stockades, the Grand Belltower, the Reliquary Crypts, the Grand Spire Gate.
- **Map:** [upper_bastion_campaign_map.md](upper_bastion_campaign_map.md).

### 3.3 Act 3: the Mantle Fissure (proposal)

- **Tone:** cosmic dread, petrified anatomy, pulsing marrow, deafening resonance, a hard deadline.
- **Landmarks:** the Chasm Ingress, the Weeping Basalt Chasm, the Marrow Conduits, the Chorus Spire, the Silent
  Ossuary, the High Altar, the Dreaming Core.
- **Map:** [mantle_fissure_campaign_map.md](mantle_fissure_campaign_map.md).

## 4. Battle hazards by tier

All hazards are zone effects on the battle board `[kit hook]` (zone hazards with a schedule: every round, on a given
block, or every N rounds), with names and numbers as `[data]`; the numbers are lore placeholders for the balance lane.
Strain values are points on the 0-100 meter and only apply from Act 2 (Act 1 has no Strain). A "block" is one of the
round's six blocks.

| Tier | Hazard | Effect |
| --- | --- | --- |
| Sunk Ward | Tar-bogs | -2 movement; applies Flammable |
| Sunk Ward | Searchlights | Battle effect open: stealth and accuracy are cut, so "reveals stealth, +20% accuracy" is retired (see gaps) |
| Sunk Ward | Blight fog / warded smoke | Fog: +5 Strain per round (Act 2 on). Warded smoke: blocks Strain gain inside it; its "cover" effect is open (see gaps) |
| Sunk Ward | Molten slag runoff | Bursts on block 5 of each round: 25 fire damage |
| High Bastion | Magnesium spotlights | A unit seen in the beam raises Inquisition Alert (amount open) |
| High Bastion | Curfew chimes | At the start of each round every unit loses 10 Poise unless it passes a Composure check (Composure is the derived corruption resist, CANON 2.1; how the check resolves: see gaps) |
| High Bastion | Blessed phosphor sprays | Ignite oil; 8 damage per block, ignores armour |
| High Bastion | Crumbling parapet gaps | Edge tiles: a unit knocked off drops to 0 HP (downed) |
| Mantle Fissure | Liquefied basalt flues | Entering: 20 fire damage and an armour expose (amount open) |
| Mantle Fissure | Seismic tremors | Every 3 rounds the board shakes; hazard tiles shift by 1 tile |
| Mantle Fissure | Marrow conduit veins | Standing in one: +100% Inoculated damage, +15 Strain per round |
| Mantle Fissure | Strain mists | Every operative not under a ward gains +3 Strain per round |

## 5. Gaps in this file

- Searchlight battle effect (section 4): Open; it leaned on a cut roll, so it is restated on existing levers or built as
  a reusable kit or engine system before it is used (Q-56). Smoke "cover": smoke gives cover guard in Act 1 (Q-54).
- The Composure check (section 4): Composure is decided (a derived stat, CANON 2.1); how a hazard's "Composure check"
  resolves (a threshold against the stat, a poise loss scaled by it, or a seeded roll) is open for the derived-stat spec
  and the balance lane.
- Costs of the 8-bunk project (Scrap, balance lane, Q-38); Rock Salt flush effect: open in CANON.md section 6.
