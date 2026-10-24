# Narrative agency, roster continuity and fail states

Status: canon for the lifecycle and fail-state rules (CANON 2.4 and 2.6); proposal for the recruit pool and the Acts 2-3
detail (consistency pass 2026-10-25). This doc follows [CANON.md](../CANON.md); where they differ, CANON wins. Names
follow the [cast registry](../background/cast_registry.md). Related: [roster lifecycle](../systems/roster_expansion_and_lifecycle.md),
[NPCs and questlines](../background/npcs_and_questlines.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Authored founders, hybrid recruits

The cast is a resilient ensemble: the founders are authored characters with stakes; the recruits who replace the fallen
come from a pool with authored parts. If one founder falls, the cause goes on.

### 1.1 The founders

| Founder | World tie | Personal hook |
| --- | --- | --- |
| Kaelen (Vanguard) | Deserter from the Seventh Ward Castellan Watch; guilt over enforcing curfew sweeps | Hunts his former commanding officer, Captain Vane (M1) |
| Vesper (Gutter-Stalker) | Ash-Crawl curfew runner and thief; knows the drainage locks and runner routes | Vendetta against Guild-Master Orlov (M4) |
| Maren (Heretic Acolyte) | Defrocked scribe of the Scriptorium of St. Lyra's who copied the forbidden texts | Proctor Quell denounced her (M2) |
| Hollis (Arbalestier) | Crag poacher who shot the Castellan commander dragging his sister Wren to the cages | Open |
| Dunstan "Slag-Jaw" (Slag-Brute) | Bone-mill stoker at the Fourth Kiln under Overseer Grendel | Grendel (M3) |
| Elyse (Ichor Alchemist) | Hedge chemist with a still beneath the Tallow Vats | Open |

- Voices: each founder has a personal voice in hub dialogue with Sister Aline, Bram and Tess `[kit hook]` (dialogue,
  today `emberwake_world::dialogue`) `[data]` (lines).
- Battle barks: short reactive lines on battle events `[kit hook]` (new: bark lines keyed to battle events) `[data]`;
  whether barks are in scope: Open.

### 1.2 Replacement recruits (Act 2 on)

- Founders cannot die in Act 1. From Act 2 a founder lost to death, capture or Calcification Stage V (Terminal Drift)
  is replaced from the Unsealed pool, never by nameless units.
- Recruits are Unsealed youths aged 18-24 `[data]` (recruit pool), each with:
  - a personal name and a visual silhouette;
  - an origin tag (for example Escaped Kiln Stoker, Defrocked Acolyte, Gutter Orphan, Smuggler's Apprentice);
  - a temperament (for example Grim Fatalist, Cold Pragmatist, Vengeful Zealot) that picks their barks and hub lines.
- Whether origin tags and temperaments carry mechanics: Open.

## 2. Reserves

Recruits come from two sources `[kit hook]` (RECRUIT1; dialogue effect `recruit`):

| Source | What | Notes |
| --- | --- | --- |
| Story recruits | Named characters freed in milestone quests | Higher starting skill or a unique perk; wait as guests first ([recruits and prologue](recruits_and_prologue_architecture.md) 3) |
| Culling raids | Repeatable culling-wagon or stockade raids (ENC-ACT2-02) | Always on Tess's board from Act 2; free 1-2 recruits each |

### 2.1 No soft lock

- From Act 2, Tess's board always offers a repeatable culling raid `[kit hook]` (repeatable board mission) `[data]`,
  so the bench can always be refilled while anyone is alive to run it.
- Act 1 needs no such valve: founders cannot die there.

### 2.2 Limits on hoarding

1. Bunks: 6 in the Dry Hollows, 8 after Bram's "Expanded Timber Frames" project (Act 2) `[kit hook]` (hub facility
   levels). A recruit needs a free bunk; guests wait in the Water-Lock Basin and take none.
2. Inquisition Alert `[kit hook]` (campaign meter): loud raids raise it. At Lockdown (71-99%) Inquisitor-General
   Ruthven's Render-Beast packs hunt and routes harden (effects: Open); at Breach (100%) a siege starts (Acts 2-3).
3. Upkeep does not scale with the bench: it is 1 Warded Tallow per day of travel or rest for the whole cell. At 0
   Tallow every battle starts with -2 AP until restocked. The old per-operative "Tallow and Salt" tax and its "Strain
   and panic" penalty are dropped.

## 3. Fail states

### 3.1 No single death ends the campaign

- Losing any one operative, a founder included, never ends the campaign. A campaign that ended on one death would make
  players treat founders as fragile VIPs, reload every loss and avoid Inoculation and grafts.
- Act 1: founders at 0 HP are downed and end the battle wounded; the Crucible capture is the one scripted loss.
- Acts 2-3: when an operative dies or reaches Terminal Drift, their gear goes to the stash `[kit hook]` (stash) and
  their name onto the Memorial Stone in the Dry Hollows `[kit hook]` (lifecycle event) `[data]`. How the survivors react
  (the old "Resolve or grief traits"): Open.
- A captured operative may return as the optional corrupted-founder boss in Act 3; whether a capture can be undone in
  Act 2: Open.

### 3.2 The Extinguished screen

Extinguished is a failure screen, not an ending `[engine]` (a screen on `aegis_ui::screen::Screen`) raised by
`[kit hook]` fail conditions:

| Condition | When |
| --- | --- |
| No living operative | No operative is left alive in the field or in the bunks (cannot happen in Act 1) |
| The Act 3 Doom Clock | It reaches 0 before node 21 (the High Altar) falls: the Great Tithing completes |

Not a fail: a single death; a lost siege; an Act 1-2 clock expiry (soft). If the squad is wiped while at least one
operative is resting in the bunks, the campaign goes on: the survivors run a low-tier rescue or scouting mission to
rebuild the cell.

### 3.3 Siege of the Sluice (Acts 2-3)

- Loud play raises Alert (which actions raise it, and by how much: Open). At Breach (100%) an Inquisition strike force
  finds the Drowned Runoff Grate and assaults the Sluice Undercroft `[kit hook]` (meter threshold trigger) `[data]`
  (siege encounter with reinforcement waves and a hold objective).
- Bram and Aline are non-combat (registry); whether they appear in the battle, for example as protect targets: Open.
- A lost siege costs facilities and forces a relocation; it is never a Game Over. Where the cell relocates: Open. The
  defend-or-finish dilemma when Breach fires during a mission: [choices](narrative_choices_and_dilemmas.md) Choice 4.

### 3.4 The Red Tithing Doom Clock

- The Red Tithing calendar in days, visible from Act 1 day 1 at 42 ("six weeks") `[kit hook]` (campaign meter over the
  travel clock in `travel.rs`).
- Soft in Acts 1-2 (story pressure, no fail); hard in Act 3.
- Time: a route leg costs 1 day (stealth +1), a rest 1 day, a battle none (CANON 2.6).
- The Clerisy preaches that the Tithing keeps the titan asleep. What it truly does, and its size, are in the
  [central truth](../background/central_truth.md) (proposal). The old loss text "the bells toll, the Low Sleep wakes and
  drags the continent into the tar-sea" is retired: the Act 3 loss is the Great Tithing completing.

## 4. Gaps (Open)

- The Act 2-3 rule that decides whether a downed operative dies, is captured or ends the battle wounded.
- Wounded effects and recovery; the outcome of a lost Act 1 battle.
- Whether barks are in scope; whether origin tags and temperaments carry mechanics.
- Survivors' reaction to a death; capture rescue in Act 2.
- Alert sources and Lockdown effects; Bram and Aline in a siege; the relocation target.
- Expiry of the soft clock in Acts 1-2 (what the story does when it reaches 0).
