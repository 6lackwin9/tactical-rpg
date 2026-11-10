# Emberwake: A Scarlet Lullaby - Canon & Design Blueprint

Status: **canon blueprint** (2026-11-10; wins on conflict). Title: **Emberwake: A Scarlet Lullaby** (`[LOCKED]`, user 2026-10-25; replaces DEC-LORE-13 "The Low Sleep"; optional in game text). Codename, crates, and paths stay `emberwake`; world id is `sunk_ward` (act-gated regions in Acts 2-3).

## 0. Blueprint Governance, Maturity Tags & Layer Rules

Blueprint for **Design**, **Build**, **Audit** (`/audit-build`, `/consistency-audit`), and **Art/UI**. Context docs: [cast_registry.md](background/cast_registry.md), [central_truth.md](background/central_truth.md), [STORY_GUIDE.md](STORY_GUIDE.md), [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) (`[PLACEHOLDER]` flavour unless `[LOCKED]` here).

- **`[LOCKED]`**: User-finalized **(a) overall campaign narrative spine** (central truth, 3-act ascent, Prologue & Act 1 structure, core Act 2-3 turning points, 3 endings) and **(b) system-linked mechanics** (classes/roles, 3-family weapon lineups, `timeline` combat, Inoculation vs. Cold Steel, roster, economy, meters, stage/sprite geometry). Changes need user approval.
- **`[PROVISIONAL]`**: Design ruling or working mission outline (`ACTS23.md`, `CL-01..CL-07` in 5.1). **Build and Art/UI may build against `[PROVISIONAL]` specs now** via generic `kits/rpg` hooks and `games/emberwake/data/**` rows; **not `[LOCKED]` until finalized by the user**.
- **`[PLACEHOLDER]`**: Unfinalized flavour — **all Founder, Hub NPC, Boss, and Faction backstories, taints/vendettas, ages, exact names/epithets, voices, portraits, costumes, and palettes**. Keep 100% data-driven (`characters.json`, `terms.json`, `VFS` paths) for drop-in replacement.
- **`[MISSING]`**: Unauthored slots (5.2). Use `PLACEHOLDER:` strings or `[PROVISIONAL]` defaults; never invent silent canon.
- **Layers**: `[engine]` (`aegis_*`, genre-free) | `[kit hook]` (`rpgkit_*` + `schema.json`, story-free RPG: `classic` vs. `timeline`) | `[data]` (`games/emberwake/data/**` via `aegis content`) | `[game code]` (`emberwake_*` thin shell; no unique rules in game crates).

## 1. Narrative Spine & Campaign Structure

### 1.1 Premise & Central Truth (`[LOCKED]`, Option A)

- **Strata**: Cinderreach spans a caldera over a sleeping titan (**the Low Sleep**): **Sunk Ward** (Act 1), **High Bastion** (Act 2), **Mantle Fissure** (Act 3). Two centuries ago **the Extinguishing** hid the sky behind the Shroud (`b2`: cause stays a mystery).
- **Lie vs. Harvest**: **Sovereign Clerisy** rules via **Iron Inquisition**, **Castellan Watch**, and **Tallow Guild** (**Fourth Kiln Syndicate** foundry arm). Adults `25+` (**the Spent**) are dulled into compliance—not incompetence (`Q-06`)—by bone-sediment draughts and tallow fumes, and rendered into tallow when dead; youths `under 25` (**the Unsealed**) resist. Publicly, the yearly **Red Tithing** pacifies the titan. **Truth**: Clerisy founders (**the first Extinguishers**, publicly First Apostles) held the first **Great Tithing** during the Extinguishing; sacrifices steeped over dreaming veins and rendered in Embalming Cloisters yield **rendered marrow**, granting ageless youth to the Clerisy alone (`Q-01`, `Q-03`). **Lullaby Houses** raise tithe-children for potent marrow; swept Unsealed youths fill the count. **Raw black marrow** mutates; **rendered marrow** preserves youth (`G-33`). Waking the titan ends the dream and harvest.
- **Escalation & Reveal (`[LOCKED]`)**: Act 1 raids starve the dream and stir the titan, forcing Arch-Prelate Othemar (`64`) to call a second **Great Tithing** (`Q-04`). Act 1 hints (`Q-07`) -> `2.4` Cloisters (rendering, Great Tithing date/cause, clock relabel, Nemeth delivering captive) -> `2.6` Reliquary (founders' record + Mantle Key) -> pre-`node 21` (truth confirmed regardless of doctrine, `Q-02`, `G-08`).
- **Tone Rule (`[LOCKED]`)**: Operatives/recruits `18-24`. Children (`under 18`) are stakes only—never fighters, graft subjects, or on-screen violence victims.

### 1.2 Prologue & Act 1 "The Young Vanguard" — Sunk Ward (`[LOCKED]` Spine, `[PLACEHOLDER]` Copy)

Scripted Prologue (`ENC-PRO-0A..0C`) enters **Sluice Undercroft** via `node 01` (`6` bunks / `4` filled, `42d` soft Doom Clock; Cold Steel only — zero Strain, Calcification, or weapon durability in Act 1).

| # | Mission & Place | Encounter & `[LOCKED]` Outcome | Roster / System Beat (`[LOCKED]`) | Flavour |
| --- | --- | --- | --- | --- |
| `0a` | The Alley (7th Ward) | `ENC-PRO-0A`: 2 Castellan Footmen; "Order 44" & withdraw | Solo `warrior`; windup, Active Guard, poise break | `[PLACEHOLDER]` |
| `0b` | The Rooftop (Meat Mkt) | `ENC-PRO-0B`: Mortuary Flagellants (`Q-73` `[PROVISIONAL]`) | `rogue` joins; smoke cover, root, disengage | `[PLACEHOLDER]` |
| `0c` | The Bridge (Canal Cart) | `ENC-PRO-0C`: Footmen + 1 Watch Crossbowman (`Q-73` `[PROVISIONAL]`) | `priest` + `ranger` freed; ranged pins, suture | `[PLACEHOLDER]` |
| `M1` | The Sweep (`02` Jct 7) | `ENC-EVT-01`: Capt. Vane (`CL-01`: `25%` HP withdraw `[PROVISIONAL]`, relic) | Opens Sunk Ward dead-drop routes | `[PLACEHOLDER]` |
| `M2` | Crossroads Cages (`03`) | `ENC-EVT-02`: Proctor Quell (valve cheat-death, slain); adult captives | Frees youths (`saved_youths`); Slab retorts | `[PLACEHOLDER]` |
| `M3` | Bone-Mill Raid (`04`) | `ENC-EVT-03`: Overseer Grendel (slain); kiln vents | `brawler` joins (bunk 5) | `[PLACEHOLDER]` |
| `M4` | Tallow Vats (`05`) | `ENC-EVT-04`: Orlov; **withdraws (`orlov_withdraws`)** (`[LOCKED]`) | `mage` joins (bunk 6; post-slice) | `[PLACEHOLDER]` |
| `Int.` | Awakening Fissure (`00`) | `ENC-EVT-05`: Calcified Amalgam (non-siege); vein splits hub | Chosen founder takes dose -> **Act 2 Tier I graft** (`[LOCKED]`) | `[PLACEHOLDER]` |
| `M5` | The Crucible (`06->07`) | `ENC-EVT-06`: Nemeth (unkillable escape); `4` deployed | **1 founder holds winch (captured)**; Nemeth escapes; keepsake (`[LOCKED]`) | `[MISSING]` (`CL-03`) |

- **Optional Board Jobs (`[PROVISIONAL]` lineup, `[PLACEHOLDER]` copy)**: Watchpost bellows (Hearth), Crucible vantages (`M5` deploy zone), drainage locks (shortcut), Toll-Pen Pip rescue, Tenement 12 attics, Third Ward gate (`Corvo` 2-way slice dilemma), cipher trades (`Q-28`, `Q-30`, `Q-85`).

### 1.3 Act Transitions & Acts 2-3 Spine (`[LOCKED]` Milestones, `[PROVISIONAL]` Node Lineup)

- **Prologue -> Act 1 (`[LOCKED]`)**: Hub (`6` bunks), Alert `0`, Doom Clock `42d` soft (`"The Red Tithing"`), board/travel/upkeep, checkpoints (`M2` d14, `M5` d38 `[PROVISIONAL]`).
- **Act 1 -> Act 2 (`[LOCKED]`)**: Same hub (High Bastion `08-15`; Scribe's Crypt story-only); Alert floor `30` (`floors.from_act 2 = 30`); enables Inoculation (`Channel`, `Strain`, `Calcification`, `4` graft slots), Cold Steel techniques & durability, recruits (1st fills M5 bunk), bleed-out/revive/capture/permadeath, Doctrine, facility trees, `8` bunks, hub sieges, Act 2 tutorials (`ACTS23` 5.1a).
- **Act 2 -> Act 3 (`[LOCKED]`)**: Same hub (Mantle Fissure `16-22`; Weeping Redoubt story-only); Alert `min 30`; Doom Clock **hard** (`0` = `Extinguished`); Tallow price rises; Mantle Resonance (`CL-05`) & Core retries.

| # | Node & Encounter | Narrative / System Payoff | Status |
| --- | --- | --- | --- |
| `2.1` | `09` Gilded Blvd (`ENC-ACT2-01`) | Breaches Upper Cinderreach past Ruthven's cordon (`CL-06`) | `[PROVISIONAL]` |
| `2.2` | `12` Stockades (`ENC-ACT2-02`, repeat) | Recruits (`18-24`); rescues Lucan -> opens Bram's forge-rite (`Q-27`) | `[PROVISIONAL]` |
| `2.3` | `10` Tallow Reservoir (`ENC-ACT2-03`) | `+20` Warded Tallow, `-30` Alert | `[PROVISIONAL]` |
| `2.4` | `11` Cloisters (`ENC-ACT2-04`, Lazar) | **Tier III Apex grafts; Great Tithing date/cause & clock relabel; Nemeth hands M5 captive to Lazar (`no rescue`, `Q-33`); locks pre-reveal side jobs (`Q-17`)** | **`[LOCKED]` beat** |
| `2.5` | `13` Grand Belltower (`ENC-ACT2-05`) | Silences Upper Cinderreach curfew bells (`Q-09`, `CL-05`) | `[PROVISIONAL]` |
| `2.6` | `14-15` Reliquary -> Spire (`ENC-ACT2-06`) | **First Extinguishers' record (`Q-12`); Mantle Chasm Key opens Act 3** | **`[LOCKED]` beat** |
| `Loyalty` | `1` per surviving founder (`6` total) | **Dual ritual**: unlocks Cranial slot + Cold Steel keystone (`PK2_GRAFTS` 3, 5) | `[LOCKED]` / `[MISSING]` (`CL-02`) |
| `3.1` | `17` Basalt Chasm (`ENC-ACT3-01`) | Fissure foothold (main path `16->17->18->21->22`) | `[PROVISIONAL]` |
| `3.2` | `18` Marrow Conduits (`ENC-ACT3-02`, Ulric) | **Cuts marrow pipeline; `+2` Doom Clock days; tremors escalate** | **`[LOCKED]` beat** |
| `3.3` | `19` Chorus Spire (`ENC-ACT3-03`, Ysolde, opt.) | Flesh-Brokers may turn Chorus via Malachi mediation (`CL-06`) | `[PROVISIONAL]` opt. |
| `3.4` | `20` Silent Ossuary (`ENC-ACT3-04`, opt. 2-phase) | **P1: M5 captive (`Q-94`: only M5's, stronger if dosed) beside Nemeth. P2 (`hp_below`/`or_round`): slays Nemeth & guards (`hostile_all`, `96x96` `2x2` swap); win -> Cassock, mercy, Transcendent Relic. Skipped: saint-aberration, Nemeth lives (`G-15`)** | **`[LOCKED]` beat** |
| `3.5` | `21` High Altar (`ENC-ACT3-05`, Othemar) | **Truth confirmed pre-fight (`Q-02`); Othemar falls; hard clock stops** | **`[LOCKED]` beat** |
| `3.6` | `22` Dreaming Core (`ENC-ACT3-06`, titan `2x2`) | **Final boss (loss wakes at Core edge costing days; fallen return as dream-echoes, `Q-21a`); Ending A / B / C** | **`[LOCKED]` climax** |

### 1.4 Endings & Fail Screen (`[LOCKED]` Gates, `[MISSING]` Epilogue Prose)

- **A Preservation** (open unless Flesh-Broker): black-iron spikes sever synapse into dreamless coma; harvest ends; humanity stays mortal.
- **B Ascendancy** (Flesh-Broker only): cell claims chronicle and merges with marrow as ageless rulers.
- **C Red Martyrdom** (always open): cell gives marrow-laden bodies as one final lullaby, sating the dream forever.
- Epilogues state fates of Bram, Aline (`G-17`), M5 captive (`G-15`), `saved_youths` (`Q-29`), and fallen/retired lean (`Q-19a`).
- **Extinguished (`[LOCKED]` fail screen, not an ending)**: `0` living operatives, Act 3 clock `0`, or Act 1 Alert `100` Breach (`extinguished_breach`). Single deaths, lost sieges, and Act 1-2 clock `0` never trigger it.

## 2. System-Linked Gameplay Blueprint (`[LOCKED]`)

### 2.1 Combat (`timeline` Ruleset; Ashlands Keeps `classic`)

| Rule | `[LOCKED]` Canon Value | Layer |
| --- | --- | --- |
| Round & AP | `6` simultaneous blocks, top timeline strip (`aegis_ui`); `12` max AP (`start_ap: 10`), `+4` regen/round; Rest banks `+0.5` AP/block (cap `+2`, next round) + `+5` Poise/block | `[kit hook]` + `[data]` |
| Speed & Cost | Windups (`blocks`: Snap `1`, Standard `2`, Heavy `3-4`), impact on final block; initiative breaks ties (no mutual trades). Decoupled `ap_cost` (Cold Steel `2-4`); player `0` recovery, enemies `0` AP with recovery | `[kit hook]` + `[data]` |
| Intents & Channel | **5 Intents** (`Assault`, `Guard`/Stand, `Support`, `Maneuver`, `Field`; `<= 2`/skill; `Field` tile-aims with shape bonuses, `Assault` locks foes at `0.5x` splash & `+1` windup). Act 2+ Strain carriers get `0 AP` **`Channel` toggle**: marrow-selected single-intent actives (`1`/graft, max `3`) + class basic swap (`1.25x` power, binary `pierce`, Strain `15/25/35` by AP `1/2/3+`), graft skills unlock, items/techniques grey out, Rest -> `Feed` (`4%` heal), `Vent` at `50+` Strain (`15%` HP). Dual-intent heavies stay Cold Steel | `[kit hook]` + `[data]` |
| Poise & Breaks | `Max Poise = CONS * 2 + gear poise + passives`, **`5` pips** (`T = max(1, roundi(max_poise / 5))`, refills/round, no random break in `pips`; `sure_break` strips all). Mults: `x1.0` windup, `x2.0` Flesh-Tension, `x0.5` Guard, `x1.5` flank, `x0.5` repeat. Break cancels windup + blocks, drops guard, queues `stagger` (locks blocks `1` & `4` next round) | `[kit hook]` |
| Guard & Armour | `Active Guard`: `35%` DR (`Brace`: `20%` / `2` rounds), `0.5x` stagger, `-10` extra Strain decay, `0.90` cap with cover. Armour `rating / (rating + 50)` (Act 2+ armoured foes `25-50`, `Q-49`) | `[kit hook]` + `[data]` |
| Loadout & Stats | Deck: `3` actives + `2` passives + `1` keystone (`5` basics `move`, `rest`, `strike`, `offhand_jab`, `active_guard` + `1` class basic free). Pouch `3` (`pouch_stacks`); `6` gear + `4` Act 2 `body` slots. `Composure = KNOW * 2 + round(CONS / 2)` (`DERIVE1`). Prof `0` soft-wield (`0.75x` power, `+1` burden); reach-`2` `(1, 2)` on `polearm`/`scourge` | `[engine]` + `[kit hook]` + `[data]` |
| Space & Cuts | `12x9` arenas (room chains carry HP/downed); `hold` & `stand` stay-behind; tells + impact badges (`plan_reveal` unlocks full plan). **Cut**: hit/evade rolls, cones, d100 crisis, classic stances | `[kit hook]` |

### 2.2 Act 2+ Dual Progression: Inoculation vs. Cold Steel (`[LOCKED]`)

- **Strain (`unit_meters.strain`, Act 2+ grafted units & aberrants; humans never)**: `0-24` **Dormant** | `25-49` **Acute Resonance** (`+15%` crit, `+1` move/block, `-25%` heal recv) | `50-74` **Flesh-Tension** (`+25%` dmg, `+4` armour, `x2` stagger) | `75-99` **Aberrant Edge** (`+50%` dmg, `10%` recoil) | `100` **Rupture**. Decay `-10`/round without marrow (`-10` more on Guard) to **graft-count floor** (`0/5/15/30/50` for `0..4` grafts); resets post-battle. Seeded `roll_table` **Crisis** at `100` (sets Strain `50`): `(1)` **Unchecked Spasm** (`strike_nearest` + stun); `(2)` **Biological Collapse** (`0` AP next round, Bleed `3`, `rating_set` armour `0`); `(3)` **Flesh Breach** (`mark: "calcify"`).
- **Calcification (`unit_tracks.calcification`, cumulative `I..V`, ONLY via Flesh Breach or Tier III graft)**: **I Vein-Blackening** (`-5` Composure, `+5%` debuff resist) | **II Calloused Chitin** (`+2` armour, `-1` move/block) | **III Sensory Rot** (fear/morale immune, `-15%` ranged dmg `> 3` tiles) | **IV The Shakes** (`forbid_hands: 2` unequips 2H/bows/arbalests; `pierce_melee`; crypt housing keeping bunk) | **V Terminal Drift** (`hostile_all` in battle; `lost` at settle). **Undo**: in battle via `Quenched Cinder Salt` or `priest` keystone (`unmark`, calms Stage V next round); at Slab post-Aline quest via **bone-scraping** (`-1` stage, `-10%` permanent HP).
- **Grafts (`4` Slab `body` slots; `PK2_GRAFTS` 3)**: `Forearms`, `Thoracic`, `Visceral` take monster core + slot part, teaching **that monster's skill** (`Tier I` ordinary, `Tier II` elite, `Tier III` Apex post-`2.4` needing `3+` grafts and `+1` Calcification). **`Cranial`** opens via class ritual for class skills or **Inoculated keystone** (`Tier III`, Cranial + keystone slot). Overwrite destroys old organ (`0` HP cost); **excision** costs `-15%` permanent HP. Held-field fallen grafted allies yield `1` organ (Apex for founders, class organ for recruits); captured/Stage V leave keepsakes only.
- **Cold Steel Techniques & Durability (Hearth, Act 2+; `PK2_GRAFTS` 5-7)**: **3 baseline main-hand families/class** (`main 3 / secondary 2 / third 2`; mastery XP `1x` at prof `2`, `1.5x` at prof `3`). Mastery `1/2/3` unlocks **Techniques `1/2/3`** (`1h`/`2h` grip tags, heavy budget, Cold Steel only); **masterwork** adds `4th` unique technique; **Mastery `3` + forge-rite** grants **Cold Steel keystone**. Act 2 (`durability.from_act = 2`) turns weapons into `{uid, item, durability, masterwork}` instances: Cold Steel attacks cost `1` durability, techniques `2`, basics/supports/items/marrow `0`; `0` = **dulled** (`-50%` power). Repaired/reforged with `Scrap` + `Fine`/`Themed` salvage from broken-down loot.

### 2.3 Roster, Economy & Meters (`[LOCKED]`)

- **Roster**: `PARTY_MAX = 4`; `6` bunks Act 1 (founders only), `8` Act 2. `0` HP = downed (Act 1: wounded `-20%` HP until rest; Act 2+: bleeds out in `N` rounds unless `revive`d, captured on loss/stay-behind). Departed units leave **keepsakes** (`~50%` trait, `Q-34`). **Living Martyrs** use `2` Crypt slots (no bunk) and tick per deployment **and** per `N` stasis days (`Q-37a`) until retirement or one last Stage V fight.
- **Economy**: **Scrap** (currency + common salvage); **Warded Tallow** (`1/day` travel or `1d` rest; `0` = `-2` start AP only); **Rock Salt** (`floor_offset -10` flush; bribes); **Aberrant Organs**; **Fine/Themed Salvage** & **Skill Books**. Battles `0d`; stealth `+1d`; Act 1 loss retreats (`+1d`, wounded).
- **Meters**: **Inquisition Alert** (`0-30` Guarded, `31-70` Suspicious, `71-99` Lockdown `150%` road fights, `100` Breach = Act 1 fail / Act 2-3 hub room-chain siege costing facility levels); **Doom Clock** (`42d`, soft Acts 1-2, hard Act 3); **Doctrine** (`-100..+100` cell + member lean); **Checkpoints** (Act 1 `bad_end` reload / Act 2+ `go_on` worse).

## 3. Cast, Factions & Visual Blueprint (`[LOCKED]` Role vs. `[PLACEHOLDER]` Flavour & Look)

> **Blueprint Rule**: Gameplay roles, stable IDs, weapon lineups, and frame geometry are `[LOCKED]`. **All character names, epithets, ages, backstories, taints, voices, silhouettes, costumes, portraits, and palettes are `[PLACEHOLDER]`** until user sign-off; keep 100% data/asset-driven.

| ID | `[LOCKED]` Kit -> World Class | `[LOCKED]` Weapons (`3/2/2`) | `[LOCKED]` Join & Role | `[PLACEHOLDER]` Identity & Keepsake |
| --- | --- | --- | --- | --- |
| `kaelen` | `warrior` -> **Vanguard** | `polearm`, `sword`, `axe` | `0a` solo; poise-break & `Guard` intercept | Kaelen (`21`), Watch deserter, spurs; `Kaelen's Gouged Sigil` |
| `vesper` | `rogue` -> **Gutter-Stalker** | `dagger`, `bow`, `sword` | `0b`; flank finisher (`kill` `+2 AP`) & `M4` | Vesper (`18`), curfew runner; `[MISSING]` (`CL-03`) |
| `maren` | `priest` -> **Heretic Acolyte** | `censer`, `bone_saw`, `scourge` | `0c`; suture, Bile Phial, keystone `unmark` | Maren (`23`), defrocked scribe; `[MISSING]` (`CL-03`) |
| `hollis` | `ranger` -> **Arbalestier** | `crossbow`, `bow`, `sword` | `0c`; windup-breaker & Hawkeye reveal | Hollis (`20`), crag poacher; `Hollis's Focusing Monocle` |
| `dunstan` | `brawler` -> **Slag-Brute** | `fist`, `hammer`, `mace` | `M3` (bunk 5); wall-slam stagger bruiser | Dunstan "Slag-Jaw" (`22`), stoker; `[MISSING]` (`CL-03`) |
| `elyse` | `mage` -> **Ichor Alchemist** | `sprayer`, `staff`, `wand` | `M4` (bunk 6); `Field` caustic/fire control | Elyse (`19`), vat chemist; `[MISSING]` (`CL-03`) |

- **Hub & Cast (`[LOCKED]` function, `[PLACEHOLDER]` persona/art)**: **Apothecary's Slab** (`"Sister Aline"`, always titled: surgery/flush/scraping, Flesh-Broker); **Tinker's Hearth** (`Bram`, Preservationist: salvage/repair/reforge/drills/bunks); **Water-Lock Basin** (`Tess Morn`: board/bribes/recon; `Pip` `12` stake). Bosses/NPCs: `Vane` (`0a`/`M1`), `Quell` (`M2`), `Grendel` (`M3`), `Orlov` (`M4` withdraw), `Amalgam` (`Int.` `2x2`), `Nemeth` (`M5` escape -> `2.4` -> `node 20` P1), `Ruthven` (`2.1`/siege), `Lazar` (`2.4`), `Ulric` (`3.2`), `Ysolde` (`3.3`), `Othemar` (`3.5`), `Dreaming Core` (`3.6` `2x2`), `Malachi` (catacomb truth/mediation), `Corvo`, `Sallow`, `Skerrit`, `Rook`, `Agnes`. Mobs use `Type + Letter` (`Castellan Footman A`, `[LOCKED]`).
- **Visual Spec**: **`[LOCKED]` Geometry**: `640x360` canvas, `STAGE1` `32x24` cells (`12x9`), `40 px` figures in `48x48` frames (`96x96` on `2x2` footprints for Amalgam, Node 20 monstrosity, Core), `Alt` outline, shake + `reduce_motion`. **`[PLACEHOLDER]` Aesthetics**: `16` master hues (`<= 1` dark/light step), world palettes + skin ramps, faction rim-lighting, costumes/faces.

## 4. Canonical Glossary (`[LOCKED]` Terms & Retired Variants)

Flag any retired variant in `data/**`, `terms.json`, UI, or docs. `(P)` marks `[PLACEHOLDER]` proper nouns replaceable via data (single spelling while active).

| Canonical Term | Status | Retired Variants (Never Use) |
| --- | --- | --- |
| `Act 1 / 2 / 3` (*The Young Vanguard*, *The First Rupture*, *The Awakening Abyss*) | `[LOCKED]` | Phase 1-3, The Iron Vanguard, The Transhuman Rupture, The Blight of the Low Sleep, The Closed Four |
| `The Low Sleep` / `The Extinguishing` / `Extinguished` | `[LOCKED]` | Emberwake: The Low Sleep (title), the Great Extinguishing (fail screen) |
| `The Red Tithing` / `The Great Tithing` / `lesser tithing` / `Lullaby Houses` `(P)` | `[LOCKED]` | - |
| `The Unsealed` (`under 25`) / `the Spent` (`25+`) | `[LOCKED]` | The Caged Young, Docile Masses (`The Spent Generation` prose only) |
| `Sovereign Clerisy`, `Iron Inquisition`, `Castellan Watch`, `Tallow Guild`, `Fourth Kiln Syndicate`, `Severed Root`, `Awakening Chorus` `(P)` | Factions | Hegemony, High Hegemony, Sub-Basalt Cults, Seventh Ward Watch, Castellan Guard |
| `Preservationist` / `Flesh-Broker` | `[LOCKED]` | The Purists, The Inoculated (as faction names) |
| `Cold Steel` / `Inoculated` (`Channel`) | `[LOCKED]` | Martial path, Resonant path |
| `Dormant`, `Acute Resonance`, `Flesh-Tension`, `Aberrant Edge`, `Rupture` | `[LOCKED]` | Clear, Quiescence, Twitch, Tremor, Resonance, Rupture Verge, Rupt, CRISIS |
| `Vein-Blackening`, `Calloused Chitin`, `Sensory Rot`, `The Shakes`, `Terminal Drift` | `[LOCKED]` | Veins, Brittle, Gargoyle, Martyr, Death/Aberr, calcification "points" |
| `Composure` | `[LOCKED]` | Composure as a 7th base stat |
| `Scrap` / `Warded Tallow` / `Rock Salt` / `Quenched Cinder Salt` / `Aberrant Organs` | `[LOCKED]` | Gold, Scrap Iron, Salvaged Slag, vials, candles, High-Grade Warded Tallow, Salt Rations, Sanctified Rock Salt, Harvested/Marrow Organs (`Sanctified Salt Sachet` = pouch item) |
| `Sluice Undercroft` `(P)`: `Apothecary's Slab`, `Tinker's Hearth`, `Dry Hollows`, `Water-Lock Basin`, `Isolation Crypts`, `Mission board` | `[LOCKED]` | Bram's Hearth/Forge, surgical rig, Water-Lock Gate/Siphon, Dead-Drop/Central Basin, Quarantine Kiln/Vaults/Crypts |
| `Doom Clock`, `Inquisition Alert` | `[LOCKED]` | Great Extinguishing (as meter) |
| `Castellan Footman`, `Watch Crossbowman` `(P)` | Mobs | Castellan Warden, Watch Arbalestier (`Arbalestier` = player class) |

## 5. Open Lore Gaps (`[PROVISIONAL]` Defaults) & `[MISSING]` Checklist

### 5.1 Ambiguities — Buildable `[PROVISIONAL]` Defaults (Awaiting User Lock)

Build and Art/UI may build against `CL-01..CL-07` now using replaceable data/assets; **none are `[LOCKED]` until user sign-off**.

| ID | Ambiguity | Buildable `[PROVISIONAL]` Default | Guardrail |
| --- | --- | --- | --- |
| `CL-01` | **Act 1 bosses (`Q-24`, `G-03`, `G-04`)** | **Vane withdraws at `25%` HP in `M1`** (`Q-24 b`, drops `Vane's Pavise`, returns in `kaelen` Act 2 ritual); **Quell (`M2`) & Grendel (`M3`) die**. | `hp_below` withdraw phase. |
| `CL-02` | **Act 2 dual-ritual quests (`G-16`, `PK2`)** | 1-node quests: `kaelen` (Vane + trial ledger `Q-08`), `vesper` (Orlov), `maren` (Scriptorium chronicle), `hollis` (Ridge convoy / Wren), `dunstan` (Fourth Kiln floor / Jessop), `elyse` (sedative cisterns). | Quest + recipe flag/class. |
| `CL-03` | **`M5` beats & keepsakes (`Q-35`, `G-14`)** | `Kaelen's Gouged Sigil`, `Vesper's Runner Coin`, `Maren's Burned Vellum`, `Hollis's Focusing Monocle`, `Dunstan's Jaw-Pin`, `Elyse's Amber Phial` (`~50%` trait) + 2-line winch barks. | `items.json` (`keepsake_<id>`) & story. |
| `CL-04` | **Doom Clock `0` & Act 3 floor (`Q-16`, `Q-17`)** | `0d` restarts `14d`, `+10` Alert, claims `1` open stake job (else `-2` `saved_youths`). `2.4` sets `max(days, 18)`; Act 3 floors clock at `min 12d`. | `meters.json` & `clock_set`. |
| `CL-05` | **Bell Resonance (`Q-09`, `Q-10`)** | **Act 2**: St. Lyra's curfew bells give Stage I+ `+10` start Strain in Upper Cinderreach until `2.5`. **Act 3**: chasm **Mantle Resonance** gives `+15` start Strain + pulse (`3.3` halves). | `arena_hazards` + track Strain. |
| `CL-06` | **Bosses & sub-orders (`Q-11`, `Q-26`, `Q-95`, `Q-97`, `G-06`, `G-07`)** | **Ruthven** = Act 2 siege boss (`2.6` fallback); **Chorus** knows of harvest & sings to wake titan (`Ysolde` spared via Malachi, else slain; `Lazar`/`Ulric` slain); **First Apostles** = public title of first Extinguishers; Clerisy runs Houses, Lazar renders; Render-Beasts = Guild-bred. | Encounter & dialogue data. |
| `CL-07` | **Aging, guests & aesthetics (`Q-05`, `Q-39`)** | Ages static (`Q-05 a`); `25+` guests (`Ferrand`, `Clemence`) grant Basin perks only, `18-24` rescues take bunks (`Q-39 a`); cast bios/names/art stay `[PLACEHOLDER]`. | `roster.json` / `characters.json` / `VFS`. |

### 5.2 `[MISSING]` Content & Placeholder Checklist

- **Act 1 (`M2b`)**: `M1`, `M2`, `M4` (`elyse` + `orlov_withdraws`), `Interlude` (`ENC-EVT-05` + dose), `7` board jobs, `checkpoints.json`, `0b`/`0c` mobs (`Q-73`).
- **Crucible & Recruits (`PK3`/`M2b`)**: `4` keepsakes (`CL-03`), `6` winch vignettes, recruit/quirk/keepsake pools.
- **Acts 2-3 (`PK1..PK8`)**: `PK2` tracks/grafts/recipes/techniques/salvage; High Bastion (`08-15`) & Mantle Fissure (`16-22`) graphs; `12` main fights, `6` dual-ritual quests (`CL-02`), bestiary (`Q-76`), siege rooms (`PK5`), Endings `A/B/C` & epilogues (`PK7`).
- **Art & Audio (`Art/UI`)**: Placeholders allowed everywhere (`Q-81`); final cast look & feel, portraits, `96x96` (`2x2`) boss sheets, and backdrops stay `[PLACEHOLDER]` / `[MISSING]` until user review.
