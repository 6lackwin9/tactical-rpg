# The Heretic Acolyte (engine class `priest`)

Status: canon for the Cold Steel branch and Maren's Act 1 kit; proposal for the Inoculated branch (Act 2 on).
Conventions (timing, armour scale, flat damage and heal values) are in the [classes index](classes_and_skill_trees.md)
section 3; rules in [combat and the timeline](combat_and_timeline.md).

Defrocked scribes, apostate embalmers and cemetery recluses: battlefield surgery with hot iron and sutures, and from
Act 2 the siphoning of Strain between bodies. Profile: moderate HP, robes and a butcher's apron, moderate poise.

## 1. Tree

| Tier | Cold Steel (the Field Suture) | Inoculated (the Marrow Siphon) |
| --- | --- | --- |
| T1 active | Brimstone Cauterize | Strain Siphon |
| T1 passive | Anatomical Precision | Sympathetic Marrow |
| T2 active | Tourniquet and Tonic | Ichor Thread Suture |
| T2 passive | Triage Efficiency | Resonant Catharsis |
| T3 active | Resilience Ward | Aberrant Communion |
| Keystone | Pure Hippocratic Oath | Martyr's Singularity |

## 2. Cold Steel (Act 1 on)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Brimstone Cauterize | 2 AP, Snap, range 2, ally | hot sulphur-dusted steel on the wound: cleanses bleed, poison or burn and heals 35 | `[kit hook]` (exists) support heal and cleanse |
| Anatomical Precision | passive | allies within 2 tiles take 15% less stagger and gain +10% debuff resistance | `[kit hook]` aura |
| Tourniquet and Tonic | 3 AP, Standard, range 2, ally | heals 75; an ally broken this round holds Active Guard for its remaining blocks; +1 tile per move block for 1 round | `[data]`, `[kit hook]` |
| Triage Efficiency | passive | healing an ally below 30% HP refunds 1 AP (once per round) | `[kit hook]` |
| Resilience Ward | 4 AP, Heavy 3 +1 rec, the squad within 4 tiles | liturgy and a swinging bronze censer: the squad (up to 4) gains +30% mitigation and cannot be broken for 1 round | `[data]` grant status |
| Pure Hippocratic Oath | keystone | at Strain 0, her sutures and draughts heal 40% more, and human enemies cannot crit her | `[data]` |

## 3. Inoculated (Act 2 on; proposal)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Strain Siphon | 1 AP, Strain 15, Snap, range 3, ally | black veins across an ally: moves up to 30 Strain from the ally to her and heals the ally 50 | `[kit hook]` meter transfer |
| Sympathetic Marrow | passive | dormant floor +10; when an ally within 3 tiles takes damage she gains 5 Strain and the ally fortify 4 | `[kit hook]` trigger |
| Ichor Thread Suture | 2 AP, Strain 25, Standard, range 3, ally | living ichor threads: heals 120; the ally gains 10 Strain | `[data]` |
| Resonant Catharsis | passive | when she crosses into Flesh-Tension, every ally within 3 tiles vents 15 Strain | `[kit hook]` trigger |
| Aberrant Communion | 2 AP, Strain 35, Heavy 3 +1 rec, the squad | links the squad's blood for 1 round: damage to any ally is pooled and halved; her single-target heals reach every squad member | `[kit hook]` |
| Martyr's Singularity | keystone | dormant floor locked at 30; in the Aberrant Edge she takes an ally's fatal blow in full | `[data]`; lasting cost open (index gaps) |

## 4. Founder: Maren

Defrocked scriptorium scribe of St. Lyra's, 23, who copied the forbidden texts. Proctor Quell denounced her; she was
defrocked under Othemar's seal and sent toward the dissection crypts. She knows Sister Aline only by reputation (the
Hall of Holy Dissection and the discovery of the drugged draughts are Aline's story, not hers). Years of marrow-ink
left black veins in her throat (backstory only in Act 1). Joins: prologue 0c, freed from the iron mortuary cart at the
canal bridge with Hollis. Act 1 mission 2 (Proctor Quell's crossroads cages) is her hook.

| Kit | Profile | Effect | Act |
| --- | --- | --- | --- |
| Copyist of St. Lyra's (trait) | innate | when she cleanses a status (bleed, poison, burn, corrosion) she bottles it as a Bile Phial; using it is a free Snap that coats an ally's weapon, +8 corrosive damage on the next hit | 1 `[kit hook]`; the phial's storage is open |
| Black-Ichor Transfusion (variant of Ichor Thread Suture) | 2 AP, Strain 20, Standard, range 3, ally | heals 130; she may move up to 25 Strain from the ally to herself, or push up to 25 into an Inoculated ally to drive it into the Aberrant Edge | 2 `[kit hook]` meter transfer |
| Martyr's Shroud (signature) | 3 AP, Strain 30, Standard, the squad within 4 tiles | blood and incense from her censer: for 2 rounds no squad member drops below 1 HP; each fatal blow absorbed adds Strain 15 to Maren instead | 2 `[kit hook]` |

In Act 1 Maren has her trait and the Cold Steel branch; her variant and signature are Inoculated and wait for Act 2.

## 5. Notes

- The old trait name "Anatomist of St. Agnes" is retired with the chapel (St. Agnes collided with Old Mother Agnes;
  Maren's origin is the Scriptorium of St. Lyra's).
- Martyr's Shroud: the old "50% of the fatal damage becomes +15% Strain" is restated as Strain 15 per absorbed blow;
  its 3 AP sits above the Inoculated 1-2 guideline (index gaps).
- Tourniquet and Tonic "resets interrupted recovery frames to Active Guard" is restated for poise breaks.
- The Bile Phial needs a place: a pouch slot (3 per operative) or a trait counter (open).
