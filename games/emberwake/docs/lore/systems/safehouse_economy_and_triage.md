# Safehouse economy and triage

Status: canon for the resources, upkeep, facility names, bunk counts and campaign meters ([CANON](../CANON.md) 2.4-2.6,
glossary); proposal for facility functions from Act 2, costs and screens. Related: [roster](roster_expansion_and_lifecycle.md),
[items](items_gear_and_progression.md), [screens](screen_architecture_and_separation_plan.md),
[hub geography](../background/world_geography_and_hub.md).

The cell's haven is the Sluice Undercroft, an old storm drain three levels under Lower Cinderreach; it stays the hub for
the whole campaign (the Scribe's Crypt and the Weeping Redoubt are `proposal` forward bases in Acts 2-3). Missions take
the cell out through the Water-Lock Basin; spoils come back to the stash; facilities turn spoils into gear, surgery and
room.

## 1. Resources

| Resource | Kind | Use | Comes from | Layer |
| --- | --- | --- | --- | --- |
| Scrap | currency (replaces gold) | trade, forge costs, bribes | encounter purses, foundry and Watch spoils, sales to Madame Sallow | `[kit hook]` resource purse; `[data]` name |
| Warded Tallow | upkeep | 1 per day of travel or rest for the whole cell; at 0 every battle starts at -2 AP until restocked | Tallow Guild shipments, Inquisitor stores, the vats (ENC-EVT-02, -04), ENC-ACT2-03 (+20) | `[kit hook]` resource purse |
| Rock Salt | stash material | Slab flushes (effect: CANON open question), bribes | mortuary carts, cathedral salt cellars, bone-mills | `[data]` `material` item |
| Aberrant Organs | stash materials | graft surgery (Act 2 on); Apex organs are Tier III | aberrations, elite Inquisitors, bosses | `[data]` `material` items |

No other upkeep: the per-operative, per-leg, Tallow-and-Salt and per-rest versions, the "nightmare" afflictions at zero
Tallow and the "Leviathan's Whispers" rest penalty are dropped. Item-level detail: [items](items_gear_and_progression.md) 5.

Engine today: the party purse has one currency (`PartyState.gold`, cap 9,999,999) and one upkeep (`supplies`, start
10, max 99; a route leg spends its `time` in supplies; camp costs 3 on world-map nodes; with no supplies a road fight
is 1.5x as likely; HEAL1, `emberwake_world::travel`). Scrap and Warded Tallow map onto those two slots; renaming them
and the zero-Tallow AP penalty are the purse hook `[kit hook]`. Rock Salt and organs need nothing new: the stash, the
`items` dialogue condition and the `take` effect handle them.

## 2. Facilities (one name each)

| Facility | Keeper | Act 1 | Act 2 on (proposal) | Layer |
| --- | --- | --- | --- | --- |
| Apothecary's Slab | Sister Aline Vance | treats the wounded (rules open); the ENC-EVT-05 marrow dose | graft surgery (organs, raises the Strain floor 0-50), Rock Salt flushes, bone-scraping (-1 Calcification stage, -10% max HP, after her quest) | `[kit hook]` facility + exchange recipes |
| Tinker's Hearth | Bram "Iron-Lung" Karr | forges and reinforces gear for Scrap and materials | the "Expanded Timber Frames" project (bunks 6 -> 8, player's choice, costs open) | `[kit hook]` facility with levels + exchange recipes |
| Dry Hollows | - | six bunks, rest (1 day, 1 Warded Tallow, full HP) | eight bunks after the project | `[kit hook]` bunks |
| Isolation Crypts (in the Dry Hollows) | - | none | Stage IV operatives sleep here | `[kit hook]` |
| Water-Lock Basin | Tess Morn | the launch point (to the district map), Tess's dead-drop board | guest quarters | `[kit hook]` mission board |

### 2.1 The Apothecary's Slab

A brick cistern turned illegal surgery and dissection theatre: bone saws, sulfur pans, tallow candles. Aline is the
Flesh-Broker voice.

- Graft installation (Act 2 on, proposal): spend an Aberrant Organ to install an Inoculated skill; each graft raises the
  operative's Strain floor (CANON: floor 0-50; the per-graft step is open). Graft slots (Cranial, Thoracic, Forearms,
  Visceral) are an Act 2 proposal. Tier III surgery advances Calcification (CANON 2.3). No graft on anyone under 18.
- Rock Salt flush: spends Rock Salt; what it does is a CANON open question (the old "halts Calcification" and "stabilises
  Stage IV" claims wait on it).
- Bone-scraping: the only Calcification undo (CANON 2.3).
- Screen (proposal): the shop's three-column Trade layout (`emberwake_gui::shop::Trade`) as organs / commitment /
  patient; the anatomical silhouette panel is new `[game code]`.

### 2.2 The Tinker's Hearth

A dry drain chamber with a charcoal forge, anvils and racks of scavenged garrison weapons. Bram is the Preservationist
voice.

- Forging (proposal): Scrap and materials into weapons and armour of the Cold Steel families (billhooks, bodkins,
  lever-arbalests, knuckles); armour reinforcement raises armour rating and armour poise.
- Weight tuning (proposal): more poise damage on a weapon's skills for a slower speed tier.
- Drills that teach Cold Steel passives: how skills are learned is the
  [skill acquisition](skill_acquisition_and_loadout_systems.md) doc's call.
- Projects: "Expanded Timber Frames" (bunks 6 -> 8). Facility levels are the kit shape; which other levels exist: Open,
  see gaps.

### 2.3 The Dry Hollows

A raised, vaulted brick gallery with six timber bunk frames under wool blankets; a brass brazier burns yellow-green
Warded Tallow against the drain mists.

- Rest: 1 day and 1 Warded Tallow; heals to full (a rest place, HEAL1). Strain needs no rest: it resets to the floor
  after every battle (CANON 2.2).
- Bunks, guests, quarantine and lifecycle: [roster](roster_expansion_and_lifecycle.md) 3, 4, 7.

### 2.4 The Water-Lock Basin

A drowned iron sluice gate and spillway, the central pool where bone ciphers are fished out of the upper city's flues.

- Tess's dead-drop board: the mission board (CANON 3.2): it offers the next main mission when the previous one is done,
  plus optional board work (CANON 3.2). Stand-in `[data]`: Tess is an NPC whose dialogue offers quests (`start_quest`)
  behind flag conditions.
- Launch: leaving through the Basin opens the district map (world map).
- Squad: pick up to four (the Character screen's active toggle) and their loadouts before leaving.
- Alert: bribes in Scrap or Rock Salt can lower the Inquisition Alert (amounts and other inputs open).
- Guests (Act 2 on): lodge here without a bunk; examples Ferrand the Clock-Smith (+10% re-tempering speed at the Hearth)
  and Sister Clemence (-20% Rock Salt cost of Slab flushes).

## 3. Campaign meters `[kit hook]`

Meters with thresholds and triggers; shown on the map HUD bar next to Scrap and Tallow (the bar today shows the place,
gold and supplies, `emberwake_gui::hud`).

| Meter | Canon value |
| --- | --- |
| Doom Clock | the Red Tithing calendar in days, 42 at Act 1 day 1; soft in Acts 1-2 (no fail), hard in Act 3 (fail if it reaches 0 before node 21 falls). A route leg costs 1 day (stealth +1), a rest 1 day, a battle none; the days run on the engine's travel clock (`gs.time`, route `time` 1 = 1 day) |
| Inquisition Alert | 0-100: 0-30 Guarded, 31-70 Suspicious, 71-99 Lockdown, 100 Breach. No siege in Act 1; in Acts 2-3 Breach starts a siege of the hub, and a lost siege costs facilities and forces a relocation, never a Game Over |

Alert bands, what the player sees (proposal): Guarded, curfew patrols; Suspicious, armoured checkpoints at the district
gates and crossbowmen on the roofs; Lockdown, hunter packs with Render-Beasts and lantern sweeps (roamer markers on the
region maps; no sight cones, which are cut); Breach, the siege (Acts 2-3).

## 4. Kit hooks and the first-playable stand-ins

| Kit hook | What it is | Data-only stand-in for the first playable |
| --- | --- | --- |
| Resource purse | named currency and upkeep with zero-effects | none: gold and supplies show their engine names until the hook lands |
| Hub facilities with levels | a facility, its keeper, its level and its projects | the keeper as a region-map NPC whose dialogue offers the services |
| Exchange recipes | inputs (items, Scrap) to outputs (item, upgrade, unlock) | crafting through dialogue choices that `requires` items and Scrap, `take` them and `give` the result |
| Mission board | offers, the next main mission, optional work | Tess's dialogue offering quests behind flags |
| Unit lifecycle | downed / wounded / captured / dead / retired, bunk count | none: downed exists; the rest waits for the hook |
| Campaign meters | Doom Clock, Alert, thresholds and triggers | flags set by dialogue and quests; the travel clock already counts time |

## 5. Gaps

1. Inquisition Alert inputs: what raises it (loud fights, failed stealth, missions) and what lowers it (bribe sizes,
   missions such as ENC-ACT2-03 -30).
2. Facility levels beyond the bunk project: does the Hearth or the Slab have levels, and what each unlocks.
3. Prices: Scrap costs of forging, reinforcement, the bunk project, bribes; drop sizes beyond the Act 1 encounter
   purses.
4. Graft costs and the per-graft Strain floor step.
5. Act 1 Alert at 100% (CANON open question): no siege in Act 1, so what happens.
6. Where the meters show in battle: the map HUD bar only, or a line in the battle screen too.
