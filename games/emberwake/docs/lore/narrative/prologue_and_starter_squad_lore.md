# Prologue and starter squad dossiers

Status: **`[LOCKED]` (`2026-11-10` Lore & Visual Hooks Session)** — 3-mission prologue tutorial spine (`0a`->`0b`->`0c`->Hearth), clean Day 1 human Founder rule, starter 4 backstories & visual briefs, and Hearth induction scene, matching [CANON.md](../CANON.md) Section 3 and [FOUNDER_PORTRAITS.md](../../specs/FOUNDER_PORTRAITS.md). Keep 100% data/VFS-driven (`characters.json`, `sequences.json`, `vignettes.json`). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Structure: three missions and the Hearth (`[LOCKED]`)

The prologue is an interactive tutorial and character convergence: `0a` (`warrior` solo), `0b` (`rogue` joins), and `0c` (`priest` + `ranger` freed) wash into the **Sluice Undercroft** (`0%` corruption and **100% clean mortal Cold-Steel human visuals** throughout Act 1—zero bone spurs, glowing eyes, black marrow veins, or smoke fingertips on Day 1).

| Step | Mission & Encounter | Founders / Mentors Introduced |
| --- | --- | --- |
| `0a` | **The Alley**: the deserter's fight (`ENC-PRO-0A`) | `Kaelen` (`Vanguard`) |
| `0b` | **The Rooftop**: the Meat Market ambush (`ENC-PRO-0B`) | `Vesper` (`Gutter-Stalker`) |
| `0c` | **The Bridge**: the mortuary cart (`ENC-PRO-0C`) | `Maren` (`Heretic Acolyte`) & `Hollis` (`Arbalestier`) |
| **Hearth** | **The Sluice Undercroft**: the alliance forms | `Berek "Cinder-Lung" Dorn` & `Chirurgeon Sabina` (Hub mentors) |

## 2. The four starting dossiers (`[LOCKED]` backstories, clean Day 1 looks & Act 2 taint hooks)

### 2.1 Kaelen (`Vanguard`, `21`)

- **Clean Day 1 Visual (`FOUNDER_PORTRAITS.md`):** tall, broad-shouldered `21`-yo (`base_man`); wavy **straw-blond** hair off brow; square jaw, light stubble, grey eyes; dented Watch plate with **chiselled-out cathedral cross**, **faded Lullaby House wrist-number under bloodied linen**, notched **billhook** (`notched_billhook`).
- **Backstory & Rupture:** raised from infancy inside a Clerisy **Lullaby House** and drafted at `14` into `Captain Vane`'s Tithe-Escort Watch for his size. For seven years he guarded Tithing sweeps believing his childhood housemates had died in sacred peace to save Cinderreach—until he caught `Vane` selling swept captives off-ledger to `Proctor Quell`'s vivisectors during `Order 44`, shattered `Corporal Fitch`'s jaw, kicked over the lantern so the youths could scatter, and chiselled the cross off his chestplate.
- **Act 2 Arc & Taint Hook:** `2.4` Cloister ledgers reveal what became of his Lullaby housemates; his Act 2 dual-ritual hunts `Vane` & burns the 7th Ward debt ledgers (`Deserter's Kettle Helm`); Inoculated path grows **chalk-white forearm bone spurs**.
- **In `0a`:** solo unit; teaches `Active Guard`, armour mitigation, and `Pommel Strike` into an enemy windup to break Poise.

### 2.2 Vesper (`Gutter-Stalker`, `18`)

- **Clean Day 1 Visual:** small, wiry `18`-yo (`base_lean`); sleek **ink-black hair with ash-grey streaks** & side fringe; fox face, soot cheeks, razor smirk; bunched charcoal cowl, **slate leather-shingle jerkin**, **tin runner whistle at throat**, chimney soot & wax on fingers gripping a reversed **bodkin** (`curfew_bodkin`).
- **Backstory & Rupture:** orphaned to marrow-debt at `9` when her parents were rendered at the Tallow Vats; became `Talia Rusk`'s fastest Ash-Crawl rooftop cipher-runner. Desperate to buy out her dead parents' marrow-debt before turning `18`, she ran one blind drop for `Guild-Master Orlov`'s factor—unknowingly carrying the sweep map that hung her runner crew from the chimney flues and maimed `Talia`'s leg.
- **Act 2 Arc & Taint Hook:** corners `Orlov` in `M4` (`orlov_withdraws` at `20%` HP, dropping `Flensing Talon`) -> finishes `Orlov` & burns Guild debt ledgers in Act 2; Inoculated path curls **cold ash-smoke off her fingertips**.
- **In `0b`:** Snap strikes, `3x3` ash-smoke zone, `Hamstring Sever` root, and disengaging.

### 2.3 Maren (`Heretic Acolyte`, `23`)

- **Clean Day 1 Visual:** slender `23`-yo (`base_woman`); thick wavy **rust-red** hair; earnest brown eyes behind **round wire spectacles (one cracked lens)**; **slate mortuary habit with ripped sigil patch** & stained embalmer cuff, **gall-ink stained fingers** (clean human throat!) on a **bronze censer crook** (`bronze_censer`).
- **Backstory & Rupture:** St. Lyra's liturgical scribe **and** mortuary novice trained in censer rites, bone-saw anatomy, and field sutures. Once a devout believer in the Tithing, she uncovered the forbidden chronicle of the **first Great Tithing** (`Q-07`) alongside her aged scriptorium mentor—whom `Proctor Quell` vivisected under `Arch-Prelate Othemar`'s seal before chaining Maren into `0c`'s mortuary cart.
- **Act 2-3 Arc & Taint Hook:** slays `Quell` in `M2` (`Quell's Marrow Scalpel`) -> Act 2 St. Lyra's vault & `2.6` Reliquary tomb cipher -> confronts `Othemar` at `3.5`; Inoculated path traces **spidery black marrow-ink veins up her throat**.
- **In `0c`:** field sutures, clearing `Stagger`, and restoring ally Poise.

### 2.4 Hollis (`Arbalestier`, `20`)

- **Clean Day 1 Visual:** rangy `20`-yo (`base_lean`); knife-cropped **dusty dark-brown** hair; crooked nose, sparse beard; **two natural hazel eyes with a shrapnel-scarred left squint** + **corded brass monocle at collar**, **pale horn recoil pauldron**, steel **lever-arbalest** (`winch_arbalest`) & bone bolts.
- **Backstory & Rupture:** outer-crag poacher raiding Clerisy supply wagons with his `14`-yo sister `Wren` on the limestone passes until `Inquisitor-General Ruthven` swept their ridge. Hollis punched a heavy iron quarrel straight through `Ruthven`'s visor—**blinding his left eye and making him "the Blind Eye"**—before being beaten down out of bolts and chained in `0c`'s cart while `Wren` was swept toward High Bastion.
- **Act 2 Arc & Taint Hook:** `2.1` cordon -> Act 2 convoy rescue of `Wren` & `Ruthven` Breach/`2.6` showdown; Inoculated path installs a **glowing amber tapetum pin in his left eye**.
- **In `0c`:** line of sight, long-range shots, and `Pinning Shot` to root heavy foes before they close.

## 3. `0c` The Bridge & Hearth Induction (`[LOCKED]`)

- **Bridge Escape (`ENC-PRO-0C`):** `Kaelen` and `Vesper` hit the stranded mortuary cart from the rear, blind the Watch crossbowman with ash-smoke, and strike the cage padlock so `Maren` (snatching a bone-saw & censer) and `Hollis` (snatching a lever-arbalest) join for the first `4`-unit fight before `Vesper` pries open the drowned runoff grate into the **Sluice Undercroft**.
- **Clean Day 1 Hearth Induction Scene:** Water drips into `Talia Rusk`'s Water-Lock Basin; orange light spills from `Berek "Cinder-Lung" Dorn`'s charcoal forge; at the Slab, `Chirurgeon Sabina` cleans silver scalpels as the four soaked, clean-human fugitives wash up:
  - `Kaelen` unwraps the bloodied linen on his forearm over his faded **Lullaby House** tally mark while `Maren` stitches the Watch gash (`0` bone spurs or supernatural grafts).
  - `Vesper` wrings soot from her hair (`"The Red Tithing is in six weeks. If we sit down here, they'll flush the drains with quicklime. We need steel, salt, and targets."`).
  - `Hollis` checks his arbalest cable under his scarred natural squint (`"None of us four will see five-and-twenty under their bells. Let's crack as many iron masks as we can while our blood's still ours."`).
  - `Sabina` warns that steel alone will not survive what sleeps under the stone, while `Berek` strikes sparks off his anvil (`"Iron doesn't dream, and iron doesn't turn you into them. Six dry bunks in the Hollows behind me—when the next cathedral bell tolls, the war begins."`).
- Fades to **Act 1: The Young Vanguard** (`4/6` bunks filled, `42`-day Doom Clock, Cold Steel only).
