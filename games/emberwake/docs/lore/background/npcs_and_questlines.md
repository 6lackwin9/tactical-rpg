# NPCs and questlines: the society of Cinderreach

Status: **`[LOCKED]` (`2026-11-10` Lore & Visual Hooks Session)** — canonical hub facility anchors (`Chirurgeon Sabina`, `Berek "Cinder-Lung" Dorn`, `Talia Rusk`, `Ivo`, `Widow Varka`), regime bosses, and street neutrals, matching [CANON.md](../CANON.md) Sections 2-3 & 6 and [cast_registry.md](cast_registry.md). Keep 100% data/VFS-driven (`characters.json`, `dialogue.json`, `shops.json`). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Overview

NPCs fall into three groups, all authored as data (`emberwake_world::dialogue` conditions `flag`, `quest`, `items`, `gold` = Scrap, `stat`; effects `set_flag`, `add_flag`, `give`, `take`, `recruit`, `open_shop`, `fight`, `start_quest`, `turn_in`) `[kit hook]` + `[data]`:

```text
                                  [ CINDERREACH CAST ]
                                          |
          +-------------------------------+-------------------------------+
          v                               v                               v
   [ HUB CAST (SLUICE) ]           [ REGIME AND FACTIONS ]          [ STREET NEUTRALS ]
   Chirurgeon Sabina (Slab)        Arch-Prelate Othemar             Bailiff Corvo (Third Ward gate)
   Berek "Cinder-Lung" Dorn        Inquisitor-General Ruthven       Agnes of Block 12
   Talia Rusk & Ivo (Basin)        Lazar, Nemeth, Ulric, Quell      Brother Rook, Skerrit Bone-Wright
   Widow Varka (Basin Sluice)      Orlov, Grendel, Jessop, Vane     Brother Malachi, Ferrand, Clemence
```

## 2. Hub cast (the Sluice Undercroft — Spent mentors, non-combat)

### 2.1 Chirurgeon Sabina (`Sabina Kress`, the Apothecary)

- **Age:** `31` (Spent who kept her mind via her own alum counter-draught). **Origin:** former outer-ward prosector under `Chief Anatomist Lazar` at `21` (unaware of the inner Embalming Cloisters until exile, closing `G-31`).
- **Role:** surgeon at the Apothecary's Slab; the **Flesh-Broker** voice. Non-combat.
- **Visual hook (`AI-05`, `[LOCKED]`):** gaunt `31`-yo; **silver-templed cropped black hair**; dark oiled-green prosector collar with **unhooked leather surgical half-mask at neck** (no spectacles or habit—avoids `Maren` silhouette collision); chalk-pale sulfur knuckles; **glass retort pipettes & silver scalpels**.
- **Past & belief:** when Sabina exposed calcified bone-sediment in commoners' communion draughts (`G-18`: told in her first Act 1 Slab talk), `Lazar` sent `Proctor Quell` to silence her. After `M5` proves Cold Steel bounces off `Nemeth`, she pushes retort-refined ossuary bone-pins:
  > "Steel cannot pierce the mantle of a continent. Distill what the sleeper bleeds, pin it clean into living bone, and turn their sacrament back on them."
- **Milestone quest, `2.4` Embalming Cloisters ("Anatomy of a Living Saint"):** confronts and slays `Lazar`, uncovers the `Lullaby House` ledgers and Great Tithing date/scale (`max(days, 18)`), witnesses `Nemeth` deliver the `M5` captive (`no rescue`), and unlocks Tier III Apex implants, **bone-scraping** (`-1` Calcification stage, `-10%` permanent max HP), and **`Sabina's Prosector Crook`**. Voice of **Ending B (`The Living Conduit`)**.

### 2.2 Berek "Cinder-Lung" Dorn (the Tinker)

- **Age:** `33` (Spent who avoided communion; lungs scarred by kiln ash, mind sharp). **Origin:** `15`-year master valve-caster under `Foundry-Master Jessop` at the Fourth Kiln.
- **Role:** armourer, weaponsmith and demolitions anchor at the Tinker's Hearth; the **Preservationist** voice. Non-combat.
- **Visual hook (`bram.png` reused `1:1`, `[LOCKED]`):** barrel chest, shaggy grey-streaked mane & soot beard, **strapped right-eye iron loupe**, soot-scarred forearms, riveted cowhide apron.
- **Past & belief:** cast high-pressure brass conduit valves for `15` years believing they were steam vents—until `Jessop` handed `14`-yo apprentice `Lucan` to the Tithing to cover up a cracked batch and Berek traced the valve blueprints to the Cloisters:
  > "I spent fifteen years casting the brass throats they drink our children through. Put your faith in honest tempered steel—iron doesn't dream, and iron doesn't turn you into them."
- **Arc & unlocks:** anchors `M3` (`Grendel`), `2.2` (`Lucan` rescue -> `-20%` upgrade Scrap), `Expanded Timber Frames` (`6 -> 8` bunks), shared Act 2 Fourth Kiln raid with `Dunstan` ("Cold Slag, Black Iron" — kills `Jessop`, floods mortar moulds, secures `3.2` conduit plans), weapon repair, masterwork reforging (`4th` technique), and **Ending A (`Black-Iron Severance`)**.

### 2.3 Talia Rusk (`29`), Ivo (`12`) & Widow Varka (`42`) — Water-Lock Basin

- **Talia Rusk (`29`, Basin spymaster):** built the Ash-Crawl bone-cipher network (mentoring `Vesper` and sheltering `12`-yo `Ivo`) until `Inquisitor-General Ruthven`'s sweep crippled her leg and deafened her right ear with a magnesium flash-charge.
  - **Visual hook (`AI-05`, `[LOCKED]`):** `29`-yo lines around **flash-burned right ear**, side-braid, **weathered wool dispatch cloak** (no rogue leather shingles—avoids `Vesper` collision), **ash-wood underarm crutch top & bone-cipher rods**.
  - **3-Act Arc (`G-19` closed):** Act 1 runs the bone-cipher board, timeline **Reconnaissance**, and **Sewer flank grates**, plus board job *"The Night of the Red Pen"* rescuing `Ivo` from the Seventh Ward Toll-Pen (`CL-04`: if soft `0d` fires first, a lesser tithing claims unrescued `Ivo`). Act 2 shelters `1` class-matched `18-24` runaway (`CL-07`) and cracks `Ruthven`'s `2.5` Belltower & Breach hunts. Act 3 leads the surface Unsealed evacuation.
- **Widow Varka (`42`, Basin sluice fence — replaces `"Madame Sallow"`):** salt-widow proprietor of the Leaking Vat sluice line (**waxed black mourning veil & brass salt-scales**, `AI-05`). Once `Ivo` is rescued in Act 1, she opens the Basin's non-child black-market `Trade` shop (`Rock Salt`, `Warded Tallow`, `Lead-Sealed Communion Wafers`, arbalest parts) across Acts 1-3 (`G-22`).
- **`25+` Basin Guests (`CL-07`):** rescued Spent specialists **`Ferrand` (`40`, Clock-Smith: `+10%` Hearth re-tempering)** and **`Herbalist Clemence` (`36`: `-20%` Slab `Rock Salt` flush cost)** sleep bunk-free in the Basin and are never sworn into `18-24` combat bunks.

## 3. Key regime figures (`[LOCKED]`)

- **Arch-Prelate Othemar (`64`, `3.5` High Altar boss):** theological marrow addict kept `20` yr past Spent lifespan (**half-petrified black-stone left skull & hand**, cloth-of-gold pearl-skull copes, **gilded reliquary throne** with brass marrow tubes, `AI-06`). Calls the **second Great Tithing** because the cell's raids starve the dream; his death at `3.5` halts the Tithing and triggers `3.6`'s genuine caldera-waking crisis.
- **Inquisitor-General Ruthven (`44`, "the Blind Eye"):** blinded in the left eye by `Hollis`'s crag bolt (**`4`-screw windowless black-iron mask with deep quarrel-dent**, spiked gorget over illegal **brass collarbone tremor-grafts**, `AI-06`). Sends `Render-Beast` packs at Lockdown (`71+` Alert), holds `2.1` cordon, and fights as Act 2 Hub Breach boss or `2.6` Reliquary approach boss (`CL-06`).
- **Bailiff Corvo (`29`, Third Ward gatekeeper):** bribable Spent Watchman with a Stage II calcified leg who kept his mind on clean water; `Rock Salt` bribes open his gate and his patrol schedules cut Alert by `20%` while he tries to save sister `Ida` (`16`, lost if soft `0d` hits before rescue, `CL-04`). Burning Third Ward tenements turns him hostile.

## 4. Street neutrals & heretical fringe (`[LOCKED]`)

- **Brother Rook (`22`, Inquisition penitent):** chemical-scarred youth in an iron penitent collar mutilated for helping his sister flee a culling wagon; trades liturgical drainage ciphers on Act 1 street nodes.
- **Skerrit, Bone-Wright (`37`):** hunchbacked tenement bone-wright and seal-forger; turns `Calcified Bone Shards` into armour-shredding quarrels and barbed cestuses (`[kit hook]` exchange recipes).
- **Agnes of Block 12 (`44`):** ancient Ash-Crawl matriarch with black-stone forearms/chin who keeps her grandmother's pre-Extinguishing star-tales and hides Unsealed orphans in *"The Rafters of Tenement 12"* (intervene = save orphans, `+35` Alert; ignore = culled).
- **Lucan (`14`, `Berek`'s apprentice):** given to the Tithing by `Jessop` to cover up a cracked batch of conduit valves; rescued from a `2.2` culling wagon (`-20%` Hearth Scrap upgrade cost).
- **Brother Malachi (`38`, Fissure hermit — never a boss):** apostate in flooded basalt catacombs post-`Interlude` (purple marrow slime, spine spikes, sewer-reed robes). Forecasts tremors and bell tolls, decodes ossuary growths, mediates `3.3` with `Cantor Ysolde` on Flesh-Broker runs, and delivers the **fixed full harvest reveal before `3.5`** regardless of Doctrine (`G-08`).

## 5. Dispositions (`[PROVISIONAL]` thresholds, `[LOCKED]` cast)

| NPC | Starts | Turns friendly | Turns closed / hostile |
| --- | --- | --- | --- |
| `Bailiff Corvo` | Wary (bribe gate) | `Rock Salt` bribe; low Third Ward Watch casualties | Tenement fires or reckless Watch slaughter |
| `Widow Varka` | Mercenary (`Ivo` rescue opens sluice shop) | Sold stolen Clerisy ledgers; `500+` Scrap traded | Sluice tallow barges destroyed |
| `Agnes of Block 12` | Friendly | Block 12 attic orphans defended | Block 12 sweep ignored |
| `Brother Rook` | Pitiable | Given clean water / cloth | Attacked or driven off |
| `Brother Malachi` | Recluse | Flesh-Broker Doctrine (`3.3` mediation) | Strict Preservationist cuts optional favours (never blocks `3.5` truth reveal) |
