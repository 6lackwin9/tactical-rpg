# Recruits, prologue and Living Martyrs

Status: canon for the prologue (section 2); proposal for guests and Living Martyrs (sections 3-5, Acts 2-3)
(consistency pass 2026-10-25). This doc follows [CANON.md](../CANON.md); where they differ, CANON wins. Names follow the
[cast registry](../background/cast_registry.md). Related: [roster lifecycle](../systems/roster_expansion_and_lifecycle.md),
[campaign structure](campaign_narrative_structure.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Summary

- An interactive prologue replaces a menu start: three short missions bring in the first four founders and teach the
  basics one at a time.
- Named recruits rescued in Acts 2-3 wait in the hub as guests before taking a bunk, which gives the player personnel
  decisions beyond the procedural pool.
- Living Martyrs, dying prisoners at Calcification III-IV, can be spent as short-lived weapons or given a peaceful end;
  corruption becomes a tactical asset with a moral price.

## 2. The prologue: "The Convergence in the Gutters" (canon)

Three scripted missions before Act 1 M1, outside the district graph `[data]` (maps, encounters ENC-PRO-0A..0C),
chained by a `[kit hook]` scripted mission sequence; each join is a `[kit hook]` recruit event (dialogue effect
`recruit`).

| Mission | You control | Teaches |
| --- | --- | --- |
| 0a The Alley | Kaelen alone | windup, Active Guard, Pommel Strike interrupt (poise break) |
| 0b The Rooftop | Kaelen and Vesper | smoke zone, root (Hamstring Sever), disengage |
| 0c The Bridge | Kaelen and Vesper, then Maren and Hollis | ranged pins (Pinning Shot), field suture |

Not taught, because they are not in the game: stances (DEC-LORE-11), facing and backstabs, cover percentages, evasion
and accuracy rolls (CANON 2.1).

### 2.1 0a Kaelen's flight

- Setting: the Alleyway of Cinders, an ash-slick alley behind the Seventh Ward garrison, in a downpour.
- Narrative: Kaelen has refused to cripple a girl in a Tithing sweep and broken Corporal Fitch's jaw. Captain Vane
  reads him "Order 44" and withdraws; two Castellan Footmen corner him.
- Teaches: the 6-block round on the timeline strip at the top of the screen `[engine]` (`aegis_ui::timeline`), windups,
  armour, and Pommel Strike in a foe's windup to break its poise `[data]` (skill).

### 2.2 0b Vesper's ambush

- Setting: rotting shingle rooftops and narrow overhangs above the Meat Market.
- Narrative: as reinforcements corner Kaelen, Vesper drops from a broken flue and hurls an ash flask into the
  inquisitors' eyes.
- Teaches: Snap strikes, the ash flask's 3x3 smoke zone, Hamstring Sever to root a foe, disengaging through smoke.

### 2.3 0c The mortuary cart

- Setting: a barred canal bridge where an iron mortuary cart is stranded under the watchtower's sightlines.
- Narrative: Maren and Hollis are chained in the cart, bound for the dissection crypts. Kaelen and Vesper attack the
  bridge guards (three Inquisitors and a crossbowman) to free them.
- Teaches: Hollis's line of sight and Pinning Shot; Maren's field suture and clearing a Stagger.
- Resolution: alarms ring across Cinderreach; Vesper wrenches open the drowned drainage grate (the Drowned Runoff Grate,
  node 01). The four ride the black runoff into the Sluice Undercroft, where Bram and Sister Aline wait. The hearth
  scene: [prologue and starter squad](prologue_and_starter_squad_lore.md) 4.

## 3. Named recruits and hub guests (proposal, Act 2 on)

```text
rescue mission -> guest in the Water-Lock Basin (no bunk; grants a hub perk) -> bunk opens -> sworn in (combat roster)
```

- Guests sleep in the Water-Lock Basin (the hub's guest quarters) and take none of the 6-8 Dry Hollows bunks
  `[kit hook]` (guest list beside the roster).
- While a guest, each grants a hub perk `[data]`:
  - Ferrand the Clock-Smith: +10% re-tempering speed at the Tinker's Hearth.
  - Sister Clemence (heretic herbalist): -20% Rock Salt cost of the Slab's flushes.
- Swearing in: when a bunk opens (a death, a retirement, the 8-bunk project) the player can talk to a guest to induct
  them `[kit hook]` (dialogue effect `recruit`). Only guests aged 18-24 can be sworn in; Ferrand's and Clemence's ages
  are Open.
- In Act 1 the only authored recruits are the two late founders: Dunstan (M3) and Elyse (M4).

## 4. Living Martyrs (proposal, Acts 2-3)

Prisoners of Inquisition torture cells, church flesh-pens and bone-mills, put through forced grafts or long exposure to
blight mist, who arrive at Calcification Stage III (Sensory Rot) or IV (The Shakes). Also called the Marrow-Broken.
"Living Martyr" names these people; it is no longer a Calcification stage name.

### 4.1 The moral weight

- Their bodies are cracked with chitin plates; black blood seeps from their tear ducts. Aline's bone-scraping (the only
  undo, -1 stage, CANON 2.3) can buy one stage at most; it cannot save them.
- They know they have days, or hours, before their minds dissolve, and they offer what remains against the regime that
  made them.
- The dilemma (weapon or a peaceful bed): [choices](narrative_choices_and_dilemmas.md) Choice 3.

### 4.2 In battle (old numbers, not confirmed)

- They arrive with a Tier III Apex graft installed `[data]`.
- Their strikes pierce armour; they carry a fear aura on adjacent foes and +50% damage `[data]` (balance lane to check).
- Rapid collapse: the old rule "+1 Calcification stage per battle, Stage V after 1-2 missions" conflicts with CANON
  2.3 (Calcification advances only through Flesh Breach and Tier III surgery): Open.
- Legacy harvest: when a Martyr falls or drifts, Aline harvests an Apex organ (a Tier III Aberrant Organ) at the Slab
  `[kit hook]` (lifecycle event to stash) `[data]`.
- A Martyr who fights must be 18-24 (CANON 2.4).
- Housing: the Isolation Crypts (Stage IV sleeps there, CANON 2.3); whether a Crypt place counts against the bunks:
  Open.

## 5. Summary

| Category | Source | Bunk | Lifespan | Role |
| --- | --- | --- | --- | --- |
| Founders | prologue (4), Act 1 M3 and M4 (2) | the first six bunks | Act 1: wounded, never lost, except the Crucible capture; Act 2 on: standard | emotional anchors; personal hooks |
| Named recruits | story milestones, Act 2 on | guest first; a bunk on induction | standard | unique dialogue, hub perks, faction ties |
| Unsealed pool | culling raids, Act 2 on | needs a free bunk | standard | origin tags, temperaments |
| Living Martyrs | Inquisition death-pens, Act 2 on | Isolation Crypts (bunk count Open) | 1-2 missions (rule Open) | burst shock troops; leave an Apex organ |

The first playable slice has five founders (Elyse joins later).

## 6. Gaps (Open)

- Guest ages (Ferrand, Clemence) and which guests can be sworn in.
- The Living Martyr collapse rule against CANON 2.3; their numbers; whether Crypt places count as bunks.
