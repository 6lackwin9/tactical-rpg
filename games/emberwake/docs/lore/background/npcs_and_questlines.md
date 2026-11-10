# NPCs and questlines: the society of Cinderreach

Status: `[LOCKED]` hub facility roles (`Slab`, `Hearth`, `Basin`), boss slots & Central Truth reveals (`2.4`, `3.5`); `[PROVISIONAL]` optional board jobs, personal quests & disposition thresholds (`CL-01..07`); **`[PLACEHOLDER]` NPC names, ages, backstories, physical looks & quotes** ([CANON.md](../CANON.md) wins on conflict). Active placeholder spellings follow [cast_registry.md](cast_registry.md); keep 100% data/VFS-driven (`characters.json`, `dialogue.json`, `shops.json`). Related: [factions](factions_and_sociology.md), [Sunk Ward map](sunk_ward_campaign_map.md), [who's who](who_is_who.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Overview

NPCs fall into three groups.

1. **Hub cast (the Sluice Undercroft):** non-combat anchors who run the facilities and unlock their top tiers.
2. **Regime and faction figures:** leaders of the Clerisy, the Inquisition and the Tallow Guild.
3. **Street neutrals:** commoners, fences, conflicted guards and heretic scholars who give intel, trade, dilemmas and
   side quests.

All are data: characters, dialogue (`emberwake_world::dialogue` conditions flag (reputation keys included), quest,
items, gold, stat; effects set_flag, add_flag, give, take, recruit, open_shop, fight, start_quest, turn_in), quests, shops and faction bands (RepRules)
`[kit hook]` (exists), `[data]` entries. The dialogue condition named `gold` reads the currency, which is Scrap here.

```text
                                  [ CINDERREACH CAST ]
                                          |
          +-------------------------------+-------------------------------+
          v                               v                               v
   [ HUB CAST ]                    [ REGIME AND FACTIONS ]          [ STREET NEUTRALS ]
   Sister Aline (Slab)             Arch-Prelate Othemar             Madame Sallow (the Leaking Vat)
   Bram Karr (Hearth)              Inquisitor-General Ruthven       Bailiff Corvo (Third Ward gate)
   Tess Morn (board, Basin)        Guild-Master Orlov               Old Mother Agnes (Tenement 12)
   Pip (runner network)            Proctor Quell                    Brother Rook, "Nails" Skerrit
                                                                    Brother Malachi (the hermit)
```

## 2. Hub cast

### 2.1 Sister Aline Vance (the Apothecary)

- **Age:** 26. **Origin:** former Anointed Prosector of the Hall of Holy Dissection, Sovereign Clerisy. Called
  "Sister Aline" in all text (surname in her dossier only).
- **Role:** surgeon at the Apothecary's Slab; the Flesh-Broker voice. Non-combat.
- **Look:** tall and gaunt, cropped soot-black hair; a defaced clerical habit under a stiff leather embalmer's apron
  stained with vitriol; sulfur-yellow fingers; a bandolier of silver scalpels and bone cannulas. Early black veins.
- **Past:** fled the Clerisy after finding that the communion draughts given to commoners are laced with calcified bone
  sediment to dull them into compliance by 25. She keeps her own mind with a counter-draught she brews herself.
- **Belief:** a cold pragmatist; survival means taking the titan's marrow into the body.
  > "Steel cannot pierce the mantle of a continent. Our only salvation is to adapt: to graft the titan's flesh into our
  > sinew until we can breathe in the dark."
- **Personal quest, "Anatomy of a Living Saint" (Act 2, proposal):** this is Act 2 mission 2.4 at the Embalming
  Cloisters ([High Bastion map](upper_bastion_campaign_map.md) section 4.4): retrieve her notes and excise the gland
  cluster of an embalmed prelate before it decays `[kit hook]` objectives with a round limit, `[data]`. The same
  mission reveals the Great Tithing's date, scale and cause in Lazar's records, and the cell sees Nemeth deliver the
  captured founder to Lazar's anatomists (CANON 3.6).
- **Unlocks:** Tier III (Apex) grafts; bone-scraping, the only Calcification undo (-1 stage, -10% max HP for good);
  Tincture of Still Blood (Strain does not change for 1 round) `[kit hook]` exchange recipes, `[data]` items.

### 2.2 Bram "Iron-Lung" Karr (the Tinker)

- **Age:** 33; one of the few Spent who avoided communion: mind intact, lungs ruined, cannot fight. **Origin:** master
  foundryman of the Fourth Kiln Syndicate's upper forge.
- **Role:** armourer, weaponsmith and demolitions man at the Tinker's Hearth; the Preservationist voice. Non-combat.
- **Look:** broad and barrel-chested, a scarred throat and a rasping metallic cough; an iron jeweller's loupe over the
  right eye; a scorched cowhide apron studded with rivet pouches.
- **Past:** joined the rebellion after his apprentice Lucan (14) was seized as flesh-stock for a prelate's lung graft.
- **Belief:** Cold Steel without compromise; despises the Inoculation.
  > "Every time you stick that black filth into your arm, you give them a piece of your soul. Put your faith in three
  > inches of honest tempered steel. Iron doesn't mutate. Iron doesn't betray you when the bells toll."
- **Hub project:** "Expanded Timber Frames" raises the bunks from 6 to 8, chosen by the player (costs open)
  `[kit hook]` facility levels, `[data]`.
- **Personal quest, "Cold Slag, Black Iron" (act open, CANON.md section 6):** break into the Fourth Kiln casting floor,
  flood it with molten iron and kill Foundry-Master Jessop before reinforcements arrive `[kit hook]` objectives (slay),
  reinforcement waves (exists), `[data]`.
- **Unlocks:** masterwork steel (sunder-pikes, spiked pavises); demolition satchels that collapse walls into
  chokepoints; Star-Slag Re-Tempering (+6 armour rating, the lore's +3 x 2; immune to acid)
  `[kit hook]` exchange recipes, `[data]` items and statuses.

### 2.3 Tess Morn and Pip (the courier network)

- **Ages:** Tess 19, Pip 12 (both non-combat). **Origin:** Ash-Crawl runners.
- **Role:** Tess is the cell's dead-drop courier and runs the mission board at the Water-Lock Basin `[kit hook]` mission
  board, `[data]` quests. She was Vesper's runner contact before the cell.
- **Look:** wiry and compact; patched leather shingle-armour, greasy wool wraps, soft calfskin boots. Right ear scarred
  and deaf from an Inquisition flash-bomb.
- **Past:** Pip was caught in a curfew sweep and is held in the Seventh Ward Toll-Pen for the Red Tithing.
- **Personal quest, "The Night of the Red Pen" (Act 1 board job):** the squad slips into the Seventh Ward Toll-Pen,
  avoids the rotating searchlights, breaks Cage Wagon #3 with Bram's crowbar and gets Pip out through the drainage canal
  within 4 rounds `[kit hook]` objectives (protect, escape), `[data]`. Tess does not fight.
- **Unlocks:**
  - **Reconnaissance:** shows every enemy's planned action, speed tier, impact block and movement on the timeline strip
    during planning `[engine]` `aegis_ui::timeline`, `[kit hook]` intent visibility (what is shown without it is open,
    see gaps);
  - **Sewer insertion:** up to 2 squad units may deploy on the map flanks through grates `[kit hook]` deployment zones,
    `[data]`;
  - **Pip's runner network:** black-market trade lines for Rock Salt, Warded Tallow and rare ammunition `[data]` shop
    (`shop.rs` `Trade`).

## 3. The regime

### 3.1 Arch-Prelate Othemar (the Voice of the Sleep)

- **Faction:** the Sovereign Clerisy, its head. **Role:** the arch-antagonist; overseer of the Red Tithing.
- **Age:** 64, where the Spent rarely pass 45; kept alive past his years by the Clerisy's marrow (spoiler: see the
  [central truth](central_truth.md)).
- **Look:** shrunken and skeletal in cloth-of-gold copes stiff with pearl-crusted ossuary lace; a wheeled iron sedan
  chair pushed by deaf-mute acolytes; half his skull calcified into smooth black stone.
- **Character:** not a cartoon tyrant but a frightening theological bureaucrat who preaches that the continent rests on
  the sleeping titan's brow and that the Tithing is an agonising necessity to keep it from waking. The folk know a
  yearly rite of hundreds; his last sacrament is this year's rite turned into a Great Tithing of thousands, because the
  cell's raids hasten the waking ([CANON](../CANON.md) 3.6). The truth (spoiler): the marrow that keeps him alive is
  rendered from the remains of the tithed, the younger the more potent.
  > "You call me a butcher because you see the blood on my altar. But you do not see the beast beneath the stone,
  > child. If I do not feed it the sweet warmth of your youth, it will wake, and its waking will snap this continent
  > like a dry bone."
- **Arc:** named in the prologue; on stage in Act 3 at the High Altar (node 21, ENC-ACT3-05), where he always falls
  (CANON.md section 3.5). The old assassination and church-schism variants are dropped.

### 3.2 Bailiff Corvo (the weary sentinel)

- **Faction:** the Castellan Watch, Third Ward gatekeeper. **Role:** bribable neutral; non-combat.
- **Age:** 29; one of the few Spent who kept his mind, by paying for clean water. His left leg is rigid black bone.
- **Look:** weather-beaten, dented plate over a frayed watch tunic; chews raw brimstone root for the pain.
- **Past:** holds the barricade between the Ash-Crawl Tenements and the Fourth Kiln. His sister Ida (16) shows marrow
  sensitivity and will be taken at the next Tithing unless he smuggles her out.
- **Interaction (Act 1)** `[data]` dialogue conditions and effects:
  - bribe with Rock Salt or clean medicine to pass his gate without a fight (`take`);
  - patrol schedules that lower the Inquisition Alert by 20% `[kit hook]` campaign meter;
  - breaking point: a loud, bloody raid in the Third Ward that sets tenements alight turns him hostile for good, and his
    gate is fortified with ballistas `[data]` flags.

### 3.3 Inquisitor-General Ruthven (the Blind Eye)

- **Faction:** the Iron Inquisition, its commander.
- **Look:** tall, in black-lacquered plate; an eyeless iron mask held to his skull by four brass tension screws. He
  senses black marrow through a tremor-sensor grafted into his collarbone.
- **Hypocrisy:** burns anyone with marrow mutations in public while carrying illegal high-grade grafts himself.
- **In play:** named in Act 1, on stage in Act 2 mission 2.1. At Lockdown (Alert 71% or more) his Render-Beast packs
  roam the district map `[kit hook]` campaign meter triggers, `[data]` roaming encounters. Whether he is ever fought is
  open (CANON.md section 6).

## 4. Street neutrals

### 4.1 Brother Rook (the broken penitent)

- **Faction:** the Iron Inquisition's penitent caste. **Role:** wandering neutral, informant. Under 25.
- **Look:** a barefoot youth in shredded burlap; eyes clouded by chemical burns; an iron penitent's collar with inward
  barbs riveted around his neck.
- **Past:** an Inquisition acolyte who helped his sister flee a culling transport; punished with mutilation, he walks
  the fog singing hymns to warn the Unsealed before the purge squads come.
- **In play (Act 1):** a wandering event on Sunk Ward street nodes `[kit hook]` route events (exists as scripted route
  encounters), `[data]`. Give him clean water or cloth and he recites liturgical ciphers that reveal drainage shortcuts
  and routes around the searchlights `[data]` dialogue (`take`, `set_flag`).

### 4.2 Madame Sallow (the Grease Queen)

- **Faction:** independent broker on the Tallow Guild's black margin. **Role:** owner of the Leaking Vat, an
  underground drinking den and black market. Middle-aged.
- **Look:** dyed-leather bodice over grease-proof oiled silk; rings of yellow tallow-amber; a long clay pipe of dried
  sulfur herbs.
- **Character:** a survivor who trades with anyone; hates the church's tallow tax, distrusts the cell's martyrdom.
  > "Rebellion is an expensive hobby, young blood. The church wants your bones; the rebels want your blood. Me? I only
  > want your Scrap. Scrap doesn't care if you turn to stone tomorrow."
- **In play** `[data]` shop (`shop.rs` `Trade`) and dialogue:
  - black-market stock Bram cannot forge: foreign arbalest parts, stolen communion sedatives, Warded Tallow;
  - buys stolen Clerisy ledgers and officer insignia for Scrap.

### 4.3 "Nails" Skerrit (the bone-carver)

- **Faction:** independent tenement outlaw. **Role:** neutral artisan, black-market bone carver and forger of church
  seals.
- **Look:** short and hunchbacked, blackened teeth, a thick leather apron of gouges and bone files, a lens strapped to
  his forehead.
- **In play:** given Calcified Bone Shards from slain husks, he carves armour-shredding quarrels for Hollis and barbed
  cestuses for Dunstan for less than refined iron costs `[kit hook]` exchange recipes, `[data]`.

### 4.4 Old Mother Agnes (the tenement matriarch)

- **Faction:** the Spent of the Ash-Crawl. **Age:** 44, one of the oldest in the Ash-Crawl.
- **Look:** stooped; forearms and lower jaw petrified into brittle black stone; a faded wool shawl; a rickety stool in
  the courtyard of Tenement Block 12.
- **Lore:** keeps her grandmother's tales of the stars from before the Extinguishing, two centuries ago; they hold a
  first hint that the Extinguishing hid a Great Tithing, the Clerisy's founding act (CANON 3.6). Runs a hidden
  orphanage in the tenement attics for Unsealed children whose parents have gone docile.
- **Side quest, "The Rafters of Tenement 12" (Act 1 board job):** a purge squad sweeps the chimneys of Block 12.
  - Intervene: wipe out the inquisitors and save Agnes and the children; the Inquisition Alert rises by 35% `[data]`.
  - Stay hidden: the safehouse stays secret, but the children are culled and Agnes is dragged to the bone-mills
    `[data]` flags.

### 4.5 Lucan (the rescued apprentice; Act 2, proposal)

- **Age:** 14 (Unsealed, non-combat). **Origin:** a Tallow Vats vat-boy whom Bram took as apprentice.
- **Look:** skinny and soot-stained; a burn scar on his cheek shaped like a gear tooth; oversized work trousers held up
  with frayed rope.
- **Rescue:** freed from a culling wagon in Act 2 (ENC-ACT2-02, one wagon); a turning point for Bram.
- **Benefit:** joins Bram at the Tinker's Hearth: faster crafting, -20% Scrap cost on upgrades, Bram's masterwork
  blueprints `[kit hook]` facility modifiers, `[data]`.

## 5. The heretical fringe

### 5.1 Brother Malachi (the hermit of the Fissure)

- **Faction:** none; an apostate. **Role:** titan lorekeeper; friendly, never a boss (the Act 3 cult leader is Cantor
  Ysolde).
- **Look:** a wild-eyed recluse in the flooded basalt catacombs beneath the storm-drains, reached after the ENC-EVT-05
  interlude opens them; hair matted with purple marrow slime; black bone spikes along his spine; robes of dried sewer
  reed and tallow wax.
- **Mind:** half-tuned to the titan's dream; speaks in prophecy that swings between nonsense and exact tactical truth.
- **In play** `[data]` dialogue and flags:
  - tremor forecasting: shows the coming tremor hazard tiles on the battle board before the battle `[kit hook]` hazard
    preview;
  - organ lore: decodes harvested Aberrant Organs, revealing their hidden passive traits before surgery (Act 2 on);
  - bell warnings: tells the cell on which day the cathedral bells will toll, so expeditions can fall between
    resonance waves;
  - Act 3: mediates with the Awakening Chorus for a Flesh-Broker cell (ENC-ACT3-03), and confirms what the dream is
    fed for (CANON 3.6).

## 6. Dispositions (proposal)

Choices, collateral damage and doctrine change how neutrals treat the cell `[kit hook]` faction bands (RepRules,
exists) and per-NPC flags, `[data]` thresholds.

| NPC | Starts | Turns friendly | Turns hostile or closed |
| --- | --- | --- | --- |
| Bailiff Corvo | Wary (opens the gate for a bribe) | Bribed with Rock Salt; no watchmen killed in the Third Ward | Tenement fires, or his men killed recklessly |
| Madame Sallow | Mercenary (trades for Scrap) | Sold stolen ledgers; more than 500 Scrap spent in her shop | Her tallow shipments are lost in the cell's raids |
| Old Mother Agnes | Friendly (shelters couriers) | The cell protects her attic orphans | The cell ignores the sweep of Block 12 |
| Brother Rook | Pitiable (recites drainage ciphers) | Given water; the cell listens to his hymns | Attacked or driven off |
| Brother Malachi | Recluse (decodes organs) | The cell leans Flesh-Broker (Act 2 on) | A strict Preservationist cell: he stops talking (never hostile in combat) |

## 7. Gaps in this file

- What the timeline shows of enemy actions without Tess's reconnaissance (section 2.3): Open, see gaps.
- The act of Bram's quest; a fight for Ruthven; doctrine thresholds: open in CANON.md section 6.
- Red Tithing size (section 3.1): decided (a yearly rite of hundreds; the Great Tithing of thousands; CANON 3.6).
- What Sister Aline, a former prosector of the Clerisy, knew of the rendering of the tithed (section 2.1): not written
  (story guide G-31).
- Where Corvo's Third Ward gate sits on the Sunk Ward graph (section 3.2): Open, see gaps.
