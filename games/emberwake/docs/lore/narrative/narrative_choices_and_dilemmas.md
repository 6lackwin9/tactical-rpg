# Narrative choices and dilemmas

Status: **`[LOCKED]` (`2026-11-11` Lore, Narrative & Encounter-Flow Tightening)** — all structural campaign dilemmas (`M5` Crucible winch beats & keepsakes `CL-03`, `Berek` vs. `Sabina` Doctrine & 4 Hub Table Debates `CL-11`, Act 2 Dual-Rituals + **Survivor Inheritance** `CL-02`/`CL-09`, Living Martyrs, Hub Breach sieges w/ `Ruthven`'s `25%` retreat -> `2.6` climax `CL-10`, and `3.5->3.6` point-of-no-return Catastrophe-Stopping Endings `A/B/C`), matching [CANON.md](../CANON.md) Sections 3-6. Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Core Design Principle (`[LOCKED]`)

Every major choice leaves a permanent mechanical trace (roster, implants/techniques, hub economy, or ending gate) and pits **mortal discipline (`Preservationist` — `Berek "Cinder-Lung" Dorn`)** against **retort-refined marrow adaptation (`Flesh-Broker` — `Chirurgeon Sabina`)**, framed across **4 Hub Table Debates** (`Post-0c`, `Post-Interlude`, `Post-M5` wake, `Post-2.4` Lullaby ledger) with no bloodless golden path:

| Doctrine | Preservationist (`Berek`) | Flesh-Broker (`Sabina`) |
| --- | --- | --- |
| Creed | *"Iron doesn't dream, and iron doesn't turn you into them."* | *"Distill what the sleeper bleeds, and turn their sacrament back on them."* |
| Build focus | Cold Steel Mastery `1-3` techniques, masterwork `4th` techniques, `0` Strain floor | `M2` `Slab Retorts` ossuary bone-pins (`4` slots), `Channel` (`x1.25` armour-pierce) |
| Roster & cost | Protects veterans' human bodies; spends `Scrap`, durability & `Rock Salt` | High Strain floors (`0/5/15/30/50`), Calcification churn, Living Martyrs & bone-scraping |
| Exclusive payoffs | Gates **Ending A (`Black-Iron Severance`)** | `Malachi` `3.3` mediation; gates **Ending B (`The Living Conduit`)** |

## 2. Choice 1: The Crucible Winch Sacrifice (`M5`, `[LOCKED]`, `CL-03`)

- **Context (`ENC-EVT-06`):** Storming the Crucible Watchpost (`node 06 -> 07`), `Inquisitor-Prelate Nemeth` shrugs off Cold Steel (`Hollis`'s *"Crag-Sight Overlook"* delays `gate_reinforcements` from Round `3` to Round `4`, `CL-08`). One of the **`4` deployed founders** must hold the jammed portcullis winch so the other three break through into Act 2 (`deployment = the choice`).
- **Arc Payoff:** `Nemeth` takes the winch founder alive (`keepsake_<id>` stays in stash, `ACC`, `value 0`, `~50%` founder trait), followed by **Hub Debate 3 (*The Empty Bunk*)**. At Act 2 start `Talia Rusk` fills their open bunk with `1` class-matched `18-24` visored/cowled runaway (`CL-07`); at `2.4` the squad sees `Nemeth` hand the captive to `Lazar` (`no rescue`); at optional `3.4` Silent Ossuary (`node 20`), the remade captive fights beside `Nemeth`, snaps into `hostile_all` and tears `Nemeth` apart, then faces the squad as a `2x2` aberration (`Q-94`: stronger if given `Sabina`'s `Interlude` marrow dose) -> mercy upgrades `keepsake_<id>` into its **Transcendent Relic**.

| Founder | Locked `M5` Winch Stay-Behind Beat (`CL-03`) | Squad Loses | Locked Keepsake (`keepsake_<id>`) |
| --- | --- | --- | --- |
| `Kaelen` | Jams his weighted billhook haft into the winch cogs and holds the gate line alone | Front-line anchor & Poise guard | **Kaelen's Gouged Sigil** (`keepsake_kaelen`) |
| `Vesper` | Burns her last phosphorus/smoke pot at the winch to blind `Nemeth`'s vanguard | Snap single-target striker | **Vesper's Runner Whistle** (`keepsake_vesper`) |
| `Maren` | Loops her bronze censer chain through the ratchet teeth and braces the bar under fire | Field sutures & Poise triage | **Maren's Copyist Quill** (`keepsake_maren`) |
| `Hollis` | Wedges his steel arbalest prod into the pinion gear and holds the corridor with sidearm bolts | Long-range pins & sightline lock | **Hollis's Focusing Monocle** (`keepsake_hollis`) |
| `Dunstan` | Wedges his iron-wired jaw and penal chain into the winch teeth until pried loose alive | Grappler & `line_2` sunder bruiser | **Dunstan's Shackle Link** (`keepsake_dunstan`) |
| `Elyse` | Fuses the brake housing with a point-blank phosphorus-vitriol charge through the acid cloud | `3x3` hazard & status control | **Elyse's Cracked Retort** (`keepsake_elyse`) |

## 3. Choice 2: Act 2 Dual-Rituals & Survivor Inheritance (`-100..+100`, `[LOCKED]`, `CL-02`, `CL-09`)

Across Act 2, hub choices, Slab surgeries vs. Hearth masterworks, and each Founder's **dual-branch personal ritual** shift Doctrine (`Cold Steel keystone + Hearth relic` vs. `Cranial implant + Inoculated keystone`). **Survivor Inheritance (`CL-09`)**: if `roster.departed.<id>` is true (`M5` capture or Act 2 loss), the mission **never disappears**—their closest Hub mentor and surviving cellmates inherit it, paying cell-wide Hearth/Slab relics + Doctrine shifts instead of that Founder's personal keystone:

1. **`Kaelen` *(inherited by `Berek` + cell)*:** Hunt `Capt. Vane` & burn/leverage 7th Ward debt ledgers (`Deserter's Kettle Helm` vs. forearm bone-spur keystone).
2. **`Vesper` *(inherited by `Talia` + cell)*:** Raid Guild Counting-House, slay `Orlov` & seize tallow ledgers (Cold Steel edge vs. ash-smoke keystone).
3. **`Maren` *(inherited by `Sabina` + cell)*:** Unseal St. Lyra's hidden vault using `M2` cipher rubbings (sanctified censer rite vs. marrow-ink throat keystone).
4. **`Hollis` *(inherited by `Talia` + cell)*:** Ambush Bastion decoy convoy to free `Wren` (`14`) into Basin shelter (crag winch mastery vs. amber tapetum pin).
5. **`Dunstan` + `Berek` *(inherited by `Berek` + cell)*:** Raid Fourth Kiln, slay `Jessop`, flood mortar moulds & seize `3.2` Conduit blueprints (black-iron fist rite vs. basalt-jaw keystone).
6. **`Elyse` *(inherited by `Sabina` + cell)*:** Sabotage High Bastion sedative cisterns alongside `2.3`/`2.4` (pure alum alchemy vs. geode-burn keystone).

## 4. Choice 3: Living Martyrs (`Acts 2-3`, `[LOCKED]` roles)

Prisoners rescued at Calcification Stage III-IV (`18-24` only) occupy up to `2` stasis slots in the **Isolation Crypts** (freeing Dry Hollows bunks) and pose a three-way choice (`[kit hook]` + `[data]`):

- **Deploy as shock troop:** Enters with a Tier III Apex implant; if fallen on a held field or claimed by Drift, `Sabina` recovers `1` **`Dreamer's Marrow Geode`** (`titan_marrow_core`).
- **Merciful salt cremation:** `Sabina` administers a painless draught and `Rock Salt` cremation so they never suffer Stage V Terminal Drift (`Doctrine` shifts Preservationist).
- **Crypt stasis:** Held in the `2` Isolation Crypt stasis slots ticking per deployment/day until needed.

## 5. Choice 4: Hub Breach Sieges & `Ruthven`'s Climax (`Acts 2-3`, `[LOCKED]`, `CL-06`, `CL-10`)

Reaching `100` Alert in Act 1 ends the run (`Extinguished`); in Acts 2-3 it triggers a **Sluice Undercroft Breach siege** (`Chirurgeon Sabina` & `Berek` stand on `protect` tiles; losing costs `10` Warded Tallow, `50` Scrap, `+2d` facility lock, Alert `-> 60`). In Act 2, **`Inquisitor-General Ruthven` ("the Blind Eye")** leads the Breach—withdrawing at **`25%` HP (`ruthven_withdraws`)** so he *always* commands the **`2.6` Reliquary Gate** climax to the death alongside an awakened **First Apostle Saint-Warden** (`CL-10`).

## 6. Choice 5: Point-of-No-Return & Catastrophe-Stopping Ending (`3.5 -> 3.6`, `[LOCKED]`, `CL-11`)

Launching `3.5` High Altar (`node 21`) warns that optional Fissure nodes (`3.3`, `3.4`) and hub errands close once crossed (`CL-11`). Slaying `Arch-Prelate Othemar` stops the Great Tithing—which starves the waking titan and **begins cracking Cinderreach's volcanic caldera** under an **unbroken Shroud**. After a **one-time camp prep/rest screen on the cracked Altar lip**, the squad descends into the `2x2` **Dreaming Core (`node 22`)** (`dream-retries` tick Act 3 days):

| Ending (`[LOCKED]`) | Hub Voice | Gate | Outcome & Permanent Cost |
| --- | --- | --- | --- |
| **A — `Black-Iron Severance`** | `Berek` | Open unless Flesh-Broker | Black-iron spikes kill the Core's brainstem—**saves Cinderreach & ends Tithings/marrow forever**, but **kills geothermal warmth** (freezing dark caldera) & **petrifies active implants**. |
| **B — `The Living Conduit`** | `Sabina` | Flesh-Broker only | Grafted wardens bind into the Core's crown to sedate it **without child tithes**, **keeping warmth & marrow**—at the dark risk of becoming an **immortal new Clerisy**. |
| **C — `Stone Martyrdom`** | The squad | Always open | Total rapid ossification inside the wounded Core **petrifies both Core and deployed squad into black-stone pillars** holding up Cinderreach forever. |
| **`Extinguished`** | *(Failure)* | `0` living operatives or Act 3 `0d` | Not an ending: the second Great Tithing completes, feeding the Clerisy for a generation. |
