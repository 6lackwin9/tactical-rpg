# Risk and reward: Cold Steel against the Inoculation

Status: proposal (the Inoculated branch, Strain and Calcification switch on in Act 2; Act 1 is Cold Steel only). The
balance lane confirms every number here once the timeline ruleset runs in the sim; until then these are design
intentions, not measured results.

Numbers follow [CANON](../CANON.md) section 2; mechanics in [combat and the timeline](combat_and_timeline.md) and
[magic and corruption](magic_and_corruption.md); class kits in [classes and skill trees](classes_and_skill_trees.md).

## 1. The question

Why use Inoculated skills when they risk a crisis and permanent Calcification, and how does neither branch make the
other pointless? Corruption systems fail when the dark power is either too punishing (players ignore it) or too cheap
(it becomes the correct choice). Emberwake trades along four axes.

| Axis | Cold Steel | Inoculation |
| --- | --- | --- |
| 1. Action economy | 2-4 AP per action | 1-2 AP per action, plus 15-35 Strain |
| 2. Armour | blunted by the armour curve; answers with expose (sunder) and poise breaks | binary pierce: armour ignored |
| 3. Power over a fight | flat and predictable; no recoil | rises with the Strain brackets; recoil in the Edge |
| 4. Roster | no lasting cost | Calcification through Flesh Breach and Tier III grafts |

## 2. Axis 1: action economy

- AP is 10 max with +4 per round `[kit hook]` (exists), and every action pays when it starts. Cold Steel at 2-4 AP sustains one
  to two actions per round; a banked pool allows a burst round.
- Inoculated skills at 1-2 AP fit two to four actions into one round's AP, but each adds 15-35 Strain while decay is
  only 10-20 per round. Sustained, an Inoculated operative holds about one Inoculated action per round; two to four is a
  one- or two-round burst that ends near Flesh-Tension or the Edge.
- So the Inoculation's density is real but short: a breakthrough tool, not a default rotation.

## 3. Axis 2: armour and terrain

- The engine's armour is hyperbolic: reduction = rating / (rating + 50) `[kit hook]` (exists). The lore's flat soak ("absorbs 4-6
  damage per hit") is replaced by ratings: lore armour points x 2 = engine rating (planner pick). Lore boss armour 3-5
  -> rating 6-10 -> 11-17% reduction; a rating of 25 cuts 33%, 50 cuts 50%.
- Cold Steel answers armour with expose (sunder: Sunder and Pin expose 8) over two or three rounds, and with poise
  breaks that cancel the armoured unit's action outright.
- Inoculated attacks pierce: Calcified Spike and Bile Siphon ignore the rating entirely.
- Terrain: Cold Steel units obey collision and walls; Void-Flicker teleports 3 tiles past walls, grates and enemies,
  and Flensing Filaments pulls a sniper out of cover (forced movement and teleport are new kit hooks).
- Sustain: healing items cost Scrap and take one of 3 pouch slots; Bile Siphon heals 50% of its damage, so an
  Inoculated fighter can sustain without the squad's pouch.

## 4. Axis 3: Strain as an overdrive

| Strain | Bracket | Effect | Best for |
| --- | --- | --- | --- |
| 0-24 | Dormant | none | Cold Steel play |
| 25-49 | Acute Resonance | +15% crit, +1 tile per move block, -25% healing received | strikers (Gutter-Stalker) |
| 50-74 | Flesh-Tension | +25% damage, +4 armour rating, stagger taken x2 | the frontline, if nobody can break it |
| 75-99 | Aberrant Edge | +50% damage, recoil 10% of damage dealt | finishing a fight |
| 100 | Rupture | 1d3 crisis at round end | never on purpose |

Strain surfing means holding 25-74: Acute Resonance for a Gutter-Stalker's reach and crits, Flesh-Tension for a
Vanguard's damage and armour against a guarded elite. The Edge pays +50% damage for recoil and the nearness of
Rupture; the crisis (Spasm on an ally, Collapse, or a Flesh Breach that adds a Calcification stage) is the price of
greed. Ending a battle in the Edge costs nothing lasting (the older "ends in the Edge = Calcification" rule is retired).

## 5. Axis 4: the roster

| Cold Steel anchors | Inoculated specialists |
| --- | --- |
| No lasting cost; carry routine fights | Spent on the fights that need armour ignored or a breakthrough |
| Grow with forged gear at the Tinker's Hearth | Grow with grafts at the Apothecary's Slab; each Tier III graft adds a Calcification stage |

Calcification stages are trade-offs, not only penalties (CANON 2.3): Stage II adds +2 armour rating for -1 tile per
move block; Stage III ignores fear and morale for -15% ranged damage beyond 3 tiles; Stage IV pierces armour in melee
but forbids two-handed weapons. Stage V loses the operative. The dilemma is when to spend a specialist's remaining
stages on a story mission.

## 6. Hybrid builds

- **The grafted Vanguard**: Cold Steel gear and skills, one utility graft (Marrow Chitin: +4 armour rating, floor +10)
  and Calcified Spike as a 1 AP armour-piercer. Most fights stay at Dormant; against a guarded elite one Spike costs 15
  Strain, which decays in one or two rounds, far from Rupture.
- **The Cold Steel infiltrator**: no grafts; smoke flasks and masterwork pikes from the Tinker's Hearth. Reliable for
  the whole campaign and never a Slab patient.

## 7. How the design keeps the balance (intent)

1. **Cold Steel only**: struggles against heavily armoured death-squads and against missions with round limits
   (rescues, escapes), and spends more pouch items and repair Scrap.
2. **Inoculation only**: the roster burns out; crises hit allies, Calcification accumulates, and by mid-campaign the
   bunks empty.
3. **Intended play**: a core of Cold Steel veterans for attrition fights and one or two Inoculated specialists for the
   fights that need the rules broken.

## 8. Balance risks to measure (balance lane)

| Risk | Why | Recommendation |
| --- | --- | --- |
| Interrupt lock | Enemies plan first and are shown as tells; a 2 AP Snap with 25 base stagger deals 50 to a unit in windup, above most 12-36 bars, so the player can cancel every enemy Heavy each round. Poise refills every round, so the lock repeats | Rescale lore stagger to the formula; give heavy enemy windups (elites, bosses) more poise or an unbreakable flag; the repeated-break doubling helps; sim a fight with one Snap user against a Heavy-only enemy |
| Pierce is weak under the curve | At x2 the lore's armour walls are ratings 6-10 (11-17%), so ignoring armour gains little and "Cold Steel plinks off plate" does not hold | Give the armoured Inquisition ratings of 25-50 (33-50%) in data, or raise the lore-to-engine factor; decide before tuning `strain_cost` |
| Inoculated dominance in short fights | Strain resets to the floor after every battle and Calcification moves only on a crisis or a Tier III graft, so a fight short enough to end before Rupture costs nothing lasting | Watch crisis rate per battle in sims; levers: higher floors from grafts, `strain_cost` up, or decay only under Active Guard |
| Strain economy too tight | Decay 10-20 per round against 15-35 per action; a floor of 30 (keystones) starts in Acute Resonance, two actions reach the Edge | Sim Strain curves per class; keep the guideline costs until measured |
| Flesh-Tension is a trap | +4 rating is small next to stagger taken x2 when the enemy can break the unit | Measure breaks per round on Flesh-Tension units; consider +armour poise there instead of rating |
