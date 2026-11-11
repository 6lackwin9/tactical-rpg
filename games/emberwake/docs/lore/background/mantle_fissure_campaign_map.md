# The Mantle Fissure: campaign graph and Act 3 missions

Status: canon (`2026-11-11`, synced with [CANON.md](../CANON.md) & [STORY_GUIDE.md](../STORY_GUIDE.md); Act 3 *"The Awakening Abyss"*, `CL-04..CL-11`). Related:
[world geography](world_geography_and_hub.md), [High Bastion map](upper_bastion_campaign_map.md),
[NPCs and questlines](npcs_and_questlines.md), [ACTS23 spec](../../../../docs/specs/ACTS23.md),
[campaign structure](../narrative/campaign_narrative_structure.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. Overview

Act 3 takes place in **the Mantle Fissure and the titan catacombs** (the Sub-Basalt Abyss). With the **Key of the Mantle
Chasm** won from `Inquisitor-General Ruthven` at `2.6` ([High Bastion map](upper_bastion_campaign_map.md) section 4.6),
the cell unseals `15 Grand Spire Gate` and descends into the volcanic caldera beneath Cinderreach.

Far below the city lies the dreaming geological leviathan (**the Slumbering Root**). Twin Fourth-Kiln brass conduits run up
from the sacrificial steeping-basins beneath `21 High Altar` to `Lazar`'s Cloisters (`11`) alongside a raw abyssal condensate
bleed-line (`CL-10`), while `Arch-Prelate Othemar` prepares a **second Great Tithing** of thousands of Unsealed above the abyss.

The district graph has 8 nodes `[data]`. Main path `16 -> 17 -> 18 -> 21 -> 22`; nodes `19` (*Chorus Spire*) and `20` (*Silent Ossuary*)
must be resolved **before** crossing `21 High Altar`'s warned **point-of-no-return** into `22 The Dreaming Core` (`CL-11`).

```text
                              [ THE MANTLE FISSURE NODE GRAPH ]
                                             |
                                 [ 22. THE DREAMING CORE ]
                       (3.6, ENC-ACT3-06: Dreaming Core 2x2; Endings A/B/C)
                                             ^
                                             | [cracked Altar lip: 1x camp prep, no return]
                                 [ 21. HIGH ALTAR ]
                         (3.5, ENC-ACT3-05: Arch-Prelate Othemar)
                                             ^
                    +------------------------+------------------------+
                    | [route: Silent Ossuary]                         | [route: Pulsing Vein Chasm]
         [ 20. SILENT OSSUARY ] (optional)                 [ 18. MARROW CONDUITS ]
     (3.4, ENC-ACT3-04: Nemeth + M5 Captive)         (3.2, ENC-ACT3-02: Warden-Inquisitor Ulric)
                    ^                                                 ^
                    | [route: Chasm Rift]                             | [route: Fissure Bridges]
         [ 19. CHORUS SPIRE ] (optional)                   [ 17. WEEPING BASALT CHASM ]
        (3.3, ENC-ACT3-03: Cantor Ysolde)             (3.1, ENC-ACT3-01: Conduit Survey Outpost)
                    ^                                                 ^
                    | [route: Cavern Siphon]                          | [route: Basalt Stair]
                    +------------------------+------------------------+
                                             |
                                 [ 16. CHASM INGRESS ]
                                  (entry from Act 2)
                                             |
                                             v [siphon lift back to Sluice]
                              [ 00. SLUICE UNDERCROFT ]
                     (single campaign hub; Isolation Crypts & Quench-Anvil)
```

## 2. Campaign Pressure (`CL-04`, `CL-05`)

### 2.1 Hard Doom Clock (`min 12d`, `CL-04`)

- Entering Act 3 locks the Doom Clock (*the Great Tithing*) to **hard mode** with a guaranteed floor of **`max(remaining_days, 12)`** (`CL-04`).
- Each route leg (`1d`; stealth `+1d`), rest (`1d`), or `3.6` dream-retry ticks days; hitting `0d` before `21 High Altar` falls ends the run on the **`Extinguished`** failure screen (`CANON` 4.2).

### 2.2 Mantle Resonance (`CL-05`)

- Ambient abyssal exhalations inflict **`+15` starting Strain** on all operatives in Act 3 battles plus a periodic continental shockwave hazard (`ACTS23` `PK8`). Resolving `3.3 Chorus Spire` (`19`) halves Mantle Resonance for the rest of Act 3 (`CL-05`).

### 2.3 Single Hub (`Sluice Undercroft`), Isolation Crypts & Living Martyrs

- **Warded Tallow** upkeep stays `1/d` of travel or rest (`0` = `-2` starting AP, `CANON` 4.2).
- **Berek's Hearth & Crypts (`Sluice Undercroft`)**: `Berek` forges **Black-Iron** heavy arms; `2` **Isolation Crypt** stasis slots hold Stage IV (*The Shakes*) operatives between missions and allow `1` final **Living Martyr** deployment (`ACTS23` `PK2`, crystallizing into `Dreamer's Marrow Geode`).

## 3. Act 3 Missions (`CL-10`, `CL-11`)

| # | Mission | Node | Encounter | Payoff |
| --- | --- | --- | --- | --- |
| `3.1` | Survey Outpost | `17` Weeping Basalt Chasm | `enc_act3_01`, overrun Clerisy Conduit Survey Outpost | Rescues/introduces **`Brother Malachi`**; reveals `3.2` Conduit & `3.3` Chorus bearings (`CL-11`) |
| `3.2` | Conduits | `18` Marrow Conduits | `enc_act3_02`, **`Warden-Inquisitor Ulric`** | Severs steeped-ichor & raw-condensate lines (`CL-10`); **`+2d`** Doom Clock |
| `3.3` | The Choir *(optional, pre-`3.5`)* | `19` Chorus Spire | `enc_act3_03`, **`Cantor Ysolde`** | Halves Act 3 Mantle Resonance (`CL-05`); `Malachi` mediates peacefully on Flesh-Broker runs (`CL-06`) |
| `3.4` | The Martyr *(optional, pre-`3.5`)* | `20` Silent Ossuary | `enc_act3_04`, **`Nemeth` + `M5` Captive** (`2x2` Phase 2) | Captive goes `hostile_all` (`PHASEFX1`) & tears `Nemeth` apart -> `Nemeth's Warded Cassock` + `Transcendent Relic` |
| `3.5` | High Altar *(point of no return)* | `21` High Altar | `enc_act3_05`, **`Arch-Prelate Othemar`** | `Othemar` slain; Great Tithing halted -> **waking caldera collapse** + 1x Altar-lip camp prep (`CL-11`) |
| `3.6` | The Sleep | `22` Dreaming Core | `enc_act3_06`, **`Dreaming Core` (`2x2`)** | **Endings A / B / C** (`Black-Iron Severance`, `The Living Conduit`, `Stone Martyrdom`) |

## 4. Encounters & Finale Flow

### 4.1 Mission `3.1`: Weeping Basalt Chasm — Overrun Conduit Survey Outpost (`enc_act3_01`, `CL-11`)

- **Story (`CL-11` — replaces generic monster gatekeepers)**: Rather than an unmotivated beast roadblock, `17 Weeping Basalt Chasm` is a shattered **Clerisy Conduit Survey Outpost** where `Lazar` and `Jessop`'s pressure-gauges cracked under rising abyssal pressure, drawing `Calcified Husks` and `Ash-Crawlers` over the garrison.
- **Payoff:** The squad pulls **`Brother Malachi`** (an ex-Clerisy fissure surveyor turned hermit, never a boss) out of the collapsed gauge-house; `Malachi` maps the pressure-surge ahead at `18 Marrow Conduits` and offers to guide the cell to `Cantor Ysolde` at `19 Chorus Spire`.

### 4.2 Mission `3.2`: The Marrow Conduits (`enc_act3_02`, `CL-10`)

- **Unified Conduit Lore (`CL-10`)**: Using the Fourth Kiln blueprints seized by `Berek`/`Dunstan` in Act 2, the cell reaches the twin brass arteries bolted into the chasm wall:
  1. **Primary Steeped-Ichor Conduit**: Murdered Lullaby tithes steep in dream-exhalations in the sacrificial basins beneath `21 High Altar`; Fourth-Kiln brass pumps drive that **steeped sacrificial ichor** *up* to `Lazar`'s Cloisters (`11`) to refine `Othemar`'s pure sacrament.
  2. **Secondary Raw-Condensate Bleed-Line**: Bleeds unsteeped **raw abyssal marrow condensate** skimmed by the Inquisition (`Quell`, `Nemeth`) for crude combat infusions—the same back-pressure that ruptured into the Sluice in Act 1's `Interlude`.
- **Boss:** **`Warden-Inquisitor Ulric`**—a Stage IV black-stone Living Saint bolted directly into Fourth-Kiln brass-iron conduit hoses (`Berek` recognizes his own stolen valve stamps in a mid-battle `talk` cue)—flanked by Suture-Priests and Purifiers.
- **Payoff:** `Ulric` falls (`CL-01`); severing both lines strips Clerisy/Inquisition marrow regeneration downstream and adds **`+2` days** to the Doom Clock, while visibly intensifying the Slumbering Root's waking tremors.

### 4.3 Mission `3.3`: The Chorus Spire (`enc_act3_03`, Optional Pre-`3.5`)

- **Story (`CL-06`)**: **`Cantor Ysolde`** (former Lullaby House choir-mistress) and the **Awakening Chorus** sing through bone tuning-forks over a resonant geyser to wake the Slumbering Root even if Cinderreach collapses.
- **Resolution:** If the cell leans **Flesh-Broker** and spoke with `Brother Malachi` (`3.1`), `Malachi` mediates so `Ysolde` redirects the Choir's resonance against `Othemar`'s High Altar without bloodshed; otherwise `Ysolde` is fought and slain. Either resolution halves Act 3 Mantle Resonance (`CL-05`).

### 4.4 Mission `3.4`: The Silent Ossuary (`enc_act3_04`, Optional Pre-`3.5`)

- **Story:** In `Lazar`'s deepest ossuary vault waits **`Inquisitor-Prelate Nemeth`** beside the **Founder captured on the `M5` Crucible winch** (`sunk_ward.crucible_stayed`), surgically remade into a gilded Saint-Weapon (`Q-94`: empowered if that Founder also took `Sabina`'s `Interlude` marrow dose).
- **Two-Phase Tragedy (`PHASEFX1` `PR5-26b`)**:
  - **Phase 1:** `Nemeth` + guard + the corrupted `M5` Founder using twisted echoes of their old class kit while `talk` cues answer surviving Founders by name.
  - **The Break (`side: "hostile_all"` + `swap`):** Hearing their old cellmates fractures `Lazar`'s cranial pins—the captive snaps `hostile_all`, tears `Nemeth` and his guard apart (`kill`), and mutates (`swap`) into a `2x2` (`96x96`) fractured aberration.
- **Payoff:** `Nemeth's Warded Cassock` + mercy release of the fallen Founder, upgrading their `keepsake_<id>` into the **Founder's Transcendent Relic** (`CANON` 5.1; permanently missed if `3.5` is launched first).

### 4.5 Mission `3.5`: High Altar — Point of No Return (`enc_act3_05`, `CL-11`)

- **Warning Gate (`CL-11`)**: Launching `21 High Altar` displays an explicit **point-of-no-return prompt** warning that optional nodes (`3.3 Chorus Spire`, `3.4 Silent Ossuary`) and Sluice hub errands close once the assault begins.
- **Encounter:** **`Arch-Prelate Othemar`** (`64`, shrunken half-petrified corpse-king wired into a gilded reliquary throne with brass marrow tubes) + Castellan & Purifier Executioners above the sacrificial basins (no violence to children depicted on-screen, `CANON` 1.1).
- **Catastrophe Twist & Altar-Lip Camp Prep (`CL-11`)**: Slaying `Othemar` permanently stops the Great Tithing—**and triggers the catastrophe**: deprived of the sedative blood-rite with its `3.2` Conduits severed, the Slumbering Root convulses awake and splits the High Altar floor! Before descending into `22 The Dreaming Core`, the squad gets a **one-time camp prep/rest screen on the cracked Altar lip** (healing, pouch restock, gear/deck swap, final `Sabina` & `Berek` counsel) with no world-map backtrack.

### 4.6 Mission `3.6`: The Dreaming Core & Endings `A / B / C` (`enc_act3_06`)

- **Encounter (`ACTS23` `PK8`)**: Multi-phase battle against the `2x2` **`Dreaming Core`**. A wipe in `3.6` does not instantly Game Over unless the Act 3 clock is at `0`: the squad snaps back to the cracked Altar lip as a **dream-retry** costing Act 3 days (`CL-11`), while operatives lost earlier in the campaign surface as brief memory/dream-echoes.
- **Catastrophe-Stopping Endings (`CANON` 4.1 `[LOCKED]` — the Perpetual Ash-Shroud stays dark across all three)**:

| Ending | Canonical Name & Action | Mentor Voice | Doctrine Gate |
| --- | --- | --- | --- |
| **A** | **Black-Iron Severance**: Drive `Berek`'s quenched black-iron spikes into the Core's brainstem, killing the Slumbering Root cold—saving Cinderreach and ending the Tithings forever, while ending marrow miracles, cooling geothermal vents into bitter soot-winter, and petrifying heavy implants | `Berek` | Open unless Flesh-Broker |
| **B** | **The Living Conduit**: Using `Sabina`'s refined pins, grafted wardens bind willingly into the Core's crown as living regulators—keeping geothermal warmth and marrow medicine alive without child blood, at the haunting cost of becoming Cinderreach's new immortal wardens | `Chirurgeon Sabina` | Flesh-Broker only |
| **C** | **Stone Martyrdom**: Regardless of Doctrine, the deployed squad channels a final ossification surge from inside the wounded Core, petrifying both the Core and themselves into silent black-stone pillars holding up Cinderreach | The Cell | Always open |
| *Fail* | **Extinguished**: Campaign failure screen (`0` living operatives or Act 3 clock hits `0d`), never a narrative ending | - | - |

## 5. Data sketch `[data]`

```json
{
  "locations": [
    { "id": "chasm_ingress", "name": "The Chasm Ingress", "kind": "gate", "region": "mantle_fissure" },
    { "id": "weeping_basalt_chasm", "name": "Conduit Survey Outpost", "kind": "chasm", "encounters": ["enc_act3_01"] },
    { "id": "marrow_conduits", "name": "The Marrow Conduits", "kind": "conduit", "encounters": ["enc_act3_02"] },
    { "id": "chorus_spire", "name": "The Chorus Spire", "kind": "sanctum", "encounters": ["enc_act3_03"] },
    { "id": "silent_ossuary", "name": "The Silent Ossuary", "kind": "crypt", "encounters": ["enc_act3_04"] },
    { "id": "high_altar", "name": "The High Altar", "kind": "altar", "encounters": ["enc_act3_05"] },
    { "id": "dreaming_core", "name": "The Dreaming Core", "kind": "core", "encounters": ["enc_act3_06"] }
  ]
}
```

## 6. Resolved design notes

- **Unified `3.2` Marrow Conduits (`CL-10`)**: Primary brass conduit pumps steeped sacrificial ichor *up* from `21 High Altar`'s basins to `Lazar`'s Cloisters (`11`); secondary bleed-line carries raw abyssal condensate skimmed by the Inquisition.
- **`3.1` Survey Outpost & `3.5 -> 3.6` Point-of-No-Return (`CL-11`)**: `3.1` introduces `Brother Malachi` inside an overrun Clerisy Conduit Survey Outpost; `3.5` warns before locking optional nodes and chains directly into `3.6` via a one-time cracked-Altar camp prep screen (`3.6` dream-retries tick Act 3 days).
- **`3.4` Captive Identity**: Always uses `sunk_ward.crucible_stayed` (the `M5` winch Founder handed to `Lazar` at `2.4`); Act 2 captures are separate `roster.departed` losses (`ACTS23` `PK3`).
