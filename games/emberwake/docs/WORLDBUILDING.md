# Worldbuilding: the worlds of Emberwake as they stand

Status: review copy for the user (2026-10-21), written from the game data by `tools/worldbook.py`; the story overview
and the open threads below are hand-written. Proposed changes go to the design lane, then into the data through
`aegis content`; re-run the tool afterwards. Art that no longer fits the story is revisited once the story is set.

## The story so far

Emberwake opens in **the Ashlands**, a burnt frontier on the edge of an old empire. Every dry season, the Long Dry,
the wells of the cinder wastes fail and the **Ashen Horde**, goblin warbands from the wastes, march on the passes. The
**Ember Wardens**, a poor but fiercely loyal border order ("Keep one fire lit."), hold **Warden Keep** and the
**Cinder Pass** against them.

The player starts at Warden Keep as **Bryn Hale**, a recruit whose father's post at Emberfall burned, in a party of up
to three. The main questline, given by Captain **Mara Voss**:

1. **Hold the Courtyard** (starts by itself): the Long Dry has come early; drive Skarn Ironjaw and Vex the Cutpurse
   out of the **Ruined Courtyard** while the keep's gates are shored up. Muster Sergeant **Brask Toller**'s duty roll
   opens: the first recruits join.
2. **Watch at the Cinder Gate**: the warband left scouts in the pass; Skarn and Vex ambush the party at the **Cinder
   Gate**, with reinforcements.
3. **The Warden's Oath** (from the scout **Tev Quill**): once the pass is clear, the party swears the Wardens' oath at
   the keep; more of Toller's roll opens (the recruits: Tev, Sella, Dagny, Alda, Kit, Oren).
4. **Steel for the Gate** (starts by itself after the oath): the Cinder Gate will not hold another Long Dry on timber.
   The party takes the south road into the **Iron Reaches** to **Slag Gate**, where the Forge Guild's **Orsa Tallow**
   sells the Wardens steel at cost, and brings it home on her nephew **Wenn Tallow**'s wagon (the Horde raids it on
   the road). **The choice:** the steel goes to the gate (the gorge is barred) or to arms (Hest's counter sells
   broadswords and claymores).
5. **Keep One Fire Lit**: **Grakka Ashmaw**, chieftain of the **Ashen Horde**, has united the warbands (the rumour of
   the Ashmaw banner: one skull per warband). The party finds the **Ashmaw Camp** and breaks him. If the steel barred
   the gate, his flankers never arrive and he fights alone, throwing off his crown of rusted nails at half strength.

Side work: **Sweep the Ruins** for Hest the Quartermaster once the Wardens trust the party (reputation 30). Road
fights on the old road and the gorge; the Horde's named fighters (Skarn, Vex, Nok, Gruk, Bolg and the patrol
Warchief) are the faces of its warbands. **Grakka at the fire** is a parley before the last fight.

Beyond the Ashlands:

- **The Iron Reaches** (connected, not inherited): the mining highland of the **Forge Guild** ("Strike true."), allies
  of the Wardens. One place (Slag Gate), Orsa Tallow, and her own quest (Iron for the Embers), hidden for now.
- **Tidewrack Coast** (inherits the Ashlands and re-skins it, "the same war, wetter"): its founders were Wardens who
  followed the smoke to the sea; the order became the **Harbour Guard** ("Light the lamp."), the Horde's descendants
  sail as **Reef Raiders**. Two places, Lampwarden **Ilse Marrow** (Mara's counterpart, plays exactly like her),
  the scout **Pell Driftwood**, five raiders led by **Captain Dredge**, and one fight (Clash at Wreck Strand).

## Open threads and things to settle

- Man or woman is not in the data for Tev, Sella, Alda, Kit, Hest, the Horde Warchief, Nok and Vex; the portraits use
  best guesses (Tev, Kit and the Warchief men; the others women). Settle them in the characters' text.
- Orsa's own quest stays hidden until the Iron Reaches campaign (decision "Orsa quest stays hidden").
- The deserter as a second story choice (DESERT1) and the Warden Keep upgrades (KEEP1) are approved ideas without a
  spec yet; both change this story.
- Rewards are moving to per-enemy (REWARD1): the named lieutenants and Grakka will carry key loot.
- The generic portraits (Warden soldier, archer, brawler, acolyte, cutpurse, hedge-mage, five Horde goblins) are
  faceless on purpose; any of them can be promoted to a named character as the world fills out.
- The Iron Reaches and Tidewrack characters (Orsa, Ilse, Pell, the Reef Raiders) have no portraits yet: the one world
  first rule kept the art session to the Ashlands.
- The "Arena party" column of the story fights is the encounter's authored party (Mara and Tev) for the Scene
  Runner's arena; in the campaign the player's own party fights.

## Contents

- [The Ashlands](#the-ashlands-ashlands)
- [The Iron Reaches](#the-iron-reaches-ironreaches)
- [Tidewrack Coast](#tidewrack-coast-tidewrack)

## The Ashlands (`ashlands`)

*Where the embers never quite die.*

A burnt frontier on the edge of the old empire. The Ember Wardens hold the passes against the Ashen Horde, goblin warbands that crawl out of the cinder wastes every dry season.

- the game starts at `warden_keep` with Bryn Hale, 100 gold, 10 supplies, a party of up to 3

### Regions

| Region | Climate | Held by | Description |
| --- | --- | --- | --- |
| The Ashen Marches (`ashen_marches`) | burnt scrub and ash drifts | ember_wardens | Rolling black hills where every road is lined with the ribs of old watchtowers. |
| Cinder Pass (`cinder_pass`) | narrow volcanic gorge | ember_wardens | The only way through the Marches. Hold it and the Horde stays out. |

### Factions

| Faction | Motto | Relations | Description |
| --- | --- | --- | --- |
| Ember Wardens (`ember_wardens`) | Keep one fire lit. | ashen_horde: enemy | A border order of soldiers and scouts sworn to hold the Cinder Pass. Poorly funded, fiercely loyal. |
| The Ashen Horde (`ashen_horde`) | Burn what burns. | ember_wardens: enemy | Loose goblin warbands united by hunger and a few loud chieftains. |

### Places

**Warden Keep** (`warden_keep`, fortress, region `ashen_marches`). A squat basalt keep, its hearth never allowed to go cold.

- People here: Mara Voss, Tev Quill, Hest the Quartermaster, Brask Toller; markers: chest, exit, lore; exits: ruined_courtyard
- Fights here: -; rest place: yes; map `ashlands_warden_keep`; background art: yes

**The Ruined Courtyard** (`ruined_courtyard`, ruin, region `cinder_pass`). What is left of a watch post: broken arches and a sooty cobbled square, ideal for an ambush.

- People here: -; markers: chest, encounter, exit, lore, roamer; exits: warden_keep, cinder_gate
- Fights here: courtyard_skirmish, ash_scouts, ash_raiders, ash_patrol, ash_straggler; rest place: no; map `ashlands_ruined_courtyard`; background art: yes

**The Cinder Gate** (`cinder_gate`, gate, region `cinder_pass`). A collapsed toll arch at the top of the gorge, where the pass opens onto the wastes.

- People here: -; markers: chest, encounter, exit, lore, point; exits: ruined_courtyard
- Fights here: pass_ambush, ash_scouts, ash_raiders, ash_patrol; rest place: no; map `ashlands_cinder_gate`; background art: yes

**The Ashmaw Camp** (`ashmaw_camp`, camp, region `cinder_pass`). A ring of hide tents in the wastes, smoke rising from a dozen fires.

- People here: Grakka Ashmaw; markers: chest, exit, roamer; exits: cinder_gate
- Fights here: ashmaw_stand, ashmaw_stand_barred, ash_raiders, ash_scouts, ash_patrol; rest place: no; map `ashlands_ashmaw_camp`; background art: yes

### Roads

| Road | From | To | Time | Road fights | Scripted |
| --- | --- | --- | --- | --- | --- |
| The old road | warden_keep | ruined_courtyard | 2 | ash_scouts (1) (15%) | ash_raiders unless `ashlands.courtyard_held` |
| The high road | warden_keep | cinder_gate | 4 | ash_scouts (2), ash_raiders (2) (25%) | - |
| The wastes track | cinder_gate | ashmaw_camp | 3 | ash_raiders (2), ash_scouts (1) (30%) | - |

### Characters

**Mara Voss**, Captain of the Ember Wardens (`mara_voss`)

- playable hero; warrior class on unit `warrior` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Raised in the keep's kitchens, Mara earned her captaincy by holding the Courtyard alone for two nights. She trusts drills over speeches.
- Personality: dry, stubborn, protective
- disciplines ember_blade; named items ember_brigandine; dialogue `mara_talk`; leaves the story on `ashlands.mara_fallen`
- Art: body `warrior` (the base unit's), portrait unique (`portraits/heroes/mara_voss.png`)

**Tev Quill**, Scout of the Pass (`tev_quill`)

- playable hero; ranger class on unit `ranger` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: A quiet orphan of the Marches who reads smoke like other people read books.
- Personality: watchful, wry
- disciplines cinder_archery; dialogue `tev_talk`; leaves the story on `ashlands.tev_fallen`
- Art: body `ranger` (the base unit's), portrait unique (`portraits/heroes/tev_quill.png`)

**Hest the Quartermaster**, Keeper of the Stores (`hest_the_quartermaster`)

- important NPC; warrior class on unit `warrior` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Counts every arrow twice and every soldier three times.
- dialogue `hest_talk`
- Art: body `warrior/hest_the_quartermaster` (its own), portrait unique (`portraits/npcs/hest_the_quartermaster.png`)

**Skarn Ironjaw**, Brawler of the Horde (`skarn_ironjaw`)

- enemy; brawler class on unit `goblin_brawler` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Cracked his jaw on a Warden shield and never stopped grinning about it.
- Art: body `brawler/goblin` (the base unit's), portrait unique (`portraits/elites/skarn_ironjaw.png`)

**Vex the Cutpurse**, Skirmisher (`vex_cutpurse`)

- enemy; rogue class on unit `goblin_skirmisher` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Goes for whoever bleeds first.
- Art: body `rogue/goblin` (the base unit's), portrait unique (`portraits/elites/vex_cutpurse.png`)

**Nok Flamebow**, Horde Archer (`nok_flamebow`)

- enemy; ranger class on unit `goblin_archer` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Fires pitch-tipped arrows from the ridgeline and runs when anyone gets close.
- Art: body `ranger/goblin` (the base unit's), portrait unique (`portraits/elites/nok_flamebow.png`)

**Gruk Smokecaller**, Warband Shaman (`gruk_smokecaller`)

- enemy; priest class on unit `goblin_shaman` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Claims the ash talks to him. It mostly says 'attack'.
- Art: body `priest/goblin` (the base unit's), portrait unique (`portraits/elites/gruk_smokecaller.png`)

**Bolg Redhand**, Berserker (`bolg_redhand`)

- enemy; brawler class on unit `goblin_berserker` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Has never retreated, mostly because he forgets there is a direction for it.
- Art: body `brawler/goblin_berserker` (the base unit's), portrait unique (`portraits/elites/bolg_redhand.png`)

**Horde Brawler**, Warband Muscle (`ash_brawler`)

- enemy; brawler class on unit `goblin_brawler` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: One of many who fight for whoever feeds them.
- Art: body `brawler/goblin` (the base unit's), portrait shared (the body's common portrait)

**Horde Skirmisher**, Warband Runner (`ash_skirmisher`)

- enemy; rogue class on unit `goblin_skirmisher` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Fast, light and never alone.
- Art: body `rogue/goblin` (the base unit's), portrait shared (the body's common portrait)

**Horde Archer**, Warband Bow (`ash_archer`)

- enemy; ranger class on unit `goblin_archer` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Shoots from the ridges and runs.
- Art: body `ranger/goblin` (the base unit's), portrait shared (the body's common portrait)

**Horde Smokecaller**, Warband Shaman (`ash_shaman`)

- enemy; priest class on unit `goblin_shaman` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Mutters at the ash and patches the wounded.
- Art: body `priest/goblin` (the base unit's), portrait shared (the body's common portrait)

**Horde Berserker**, Warband Berserker (`ash_berserker`)

- enemy; brawler class on unit `goblin_berserker` (tier minion, level 3); faction `ashen_horde`; home `-`
- Background: Screams first, thinks never.
- Art: body `brawler/goblin_berserker` (the base unit's), portrait shared (the body's common portrait)

**Horde Warchief**, Patrol Leader (`ash_warchief`)

- enemy; brawler class on unit `ambush_berserker` (tier standard, level 4); faction `ashen_horde`; home `-`
- Background: Leads the patrols that circle the ruins.
- Art: body `brawler/ash_warchief` (its own), portrait unique (`portraits/elites/ash_warchief.png`)

**Grakka Ashmaw**, Chieftain of the Horde (`grakka_ashmaw`)

- enemy; warrior class on unit `goblin_warlord` (tier boss, level 6); faction `ashen_horde`; home `-`
- Background: Wears a crown of rusted nails, one for every Warden post he burned.
- dialogue `grakka_parley`; leaves the story on `quest.ashlands.keep_one_fire_lit.ready`, `ashlands.horde_broken`
- Art: body `warrior/grakka_ashmaw` (its own), portrait unique (`portraits/bosses/grakka_ashmaw.png`)

**Bryn Hale**, Warden Recruit (`bryn_hale`)

- playable hero; warrior class on unit `warrior` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: A Warden's son from the burned post at Emberfall; carries his father's shield and his temper.
- Art: body `warrior` (the base unit's), portrait unique (`portraits/heroes/bryn_hale.png`)

**Sella Marsh**, Archer of the North Wall (`sella_marsh`)

- playable hero; ranger class on unit `ranger` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Hunts ash-hares in the wastes and counts every arrow twice.
- Art: body `ranger` (the base unit's), portrait unique (`portraits/heroes/sella_marsh.png`)

**Kit Vane**, Oathbound Cutpurse (`kit_vane`)

- playable hero; rogue class on unit `rogue` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Stole from the Wardens once, got caught, and stayed.
- Art: body `rogue` (the base unit's), portrait unique (`portraits/heroes/kit_vane.png`)

**Oren Dusk**, Hedge-Mage of the Forge (`oren_dusk`)

- playable hero; mage class on unit `mage` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Reads embers the way Tev reads smoke; nobody asks where he learned fire.
- Art: body `mage` (the base unit's), portrait unique (`portraits/heroes/oren_dusk.png`)

**Alda Wren**, Hearth-Keeper (`alda_wren`)

- playable hero; priest class on unit `priest` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Tends the keep's one fire and every Warden who comes back from the pass.
- Art: body `priest` (the base unit's), portrait unique (`portraits/heroes/alda_wren.png`)

**Dagny Coal**, Pit Fighter of Slag Gate (`dagny_coal`)

- playable hero; brawler class on unit `brawler` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Fought for coin in the slag pits until the Horde burned them; fists are what she kept.
- Art: body `brawler` (the base unit's), portrait unique (`portraits/heroes/dagny_coal.png`)

**Brask Toller**, Muster Sergeant of the Wardens (`brask_toller`)

- important NPC; warrior class on unit `warrior` (tier hero, level 5); faction `ember_wardens`; home `warden_keep`
- Background: Keeps the duty roll in his head and the names of the dead on his belt.
- dialogue `muster_talk`
- Art: body `warrior/brask_toller` (its own), portrait unique (`portraits/npcs/brask_toller.png`)

**Wenn Tallow**, Orsa's Carter (`wenn_tallow`)

- important NPC; ranger class on unit `ranger` (tier hero, level 5); faction `-`; home `-`
- Background: Orsa's nephew; drives the steel wagon and swears he can shoot.
- Art: body `ranger/wenn_tallow` (its own), portrait unique (`portraits/npcs/wenn_tallow.png`)

### Questlines

**Hold the Courtyard** (`hold_the_courtyard`, given by Mara Voss, region `cinder_pass`)

The Long Dry has come early, and the Horde with it. Mara needs the Courtyard held while the keep's gates are shored up.

- Steps: 1. Rally at Warden Keep; 2. Drive the warband out of the Ruined Courtyard (fight `courtyard_skirmish`)
- Needs: quests -; flags -; starts by itself
- Reward: The Wardens' thanks, and a share of the keep's winter stores.; sets `ashlands.courtyard_held`, `ashlands.muster_1`

**Watch at the Cinder Gate** (`cinder_gate_watch`, given by Mara Voss, region `cinder_pass`)

Mara suspects the warband left scouts in the pass. Find them before they find the keep. Take someone from Toller's roll first; the pass is no place to walk alone.

- Steps: 1. Break the ambush at the Cinder Gate (fight `pass_ambush`)
- Needs: quests hold_the_courtyard; flags -; starts from the giver
- Reward: The Wardens' thanks, and a pick from the keep.; sets `ashlands.pass_cleared`

**The Warden's Oath** (`warden_oath`, given by Tev Quill, region `ashen_marches`)

Once the pass is safe, Tev will speak for the party before the Wardens.

- Steps: 1. Wait for word that the Cinder Gate is clear; 2. Return to Warden Keep and swear the oath
- Needs: quests hold_the_courtyard; flags -; starts from the giver
- Reward: A Warden's gift, and the oath.; sets `ashlands.oath_sworn`, `ashlands.muster_2`

**Sweep the Ruins** (`ember_patrol`, given by Hest the Quartermaster, region `cinder_pass`)

Hest wants the ruins swept of raiders before the next supply run.

- Steps: 1. Break a raiding party in the ruins (fight `ash_raiders`)
- Needs: quests -; flags `rep.ashlands.ember_wardens`=30; starts from the giver
- Reward: Hest's gratitude and a pick from the stores.; sets `ashlands.ruins_swept`

**Steel for the Gate** (`steel_for_the_gate`, given by Mara Voss, region `ashen_marches`)

The Cinder Gate will not hold another Long Dry on timber. Mara sends you up the south road to Slag Gate for Orsa Tallow's steel.

- Steps: 1. Load Orsa's steel at Slag Gate; 2. Bring the steel to Warden Keep
- Needs: quests warden_oath; flags -; starts by itself
- Reward: Hest's thanks and a pick from the stores.; sets `ashlands.steel_delivered`

**Keep One Fire Lit** (`keep_one_fire_lit`, given by Mara Voss, region `cinder_pass`)

Grakka Ashmaw has gathered the warbands past the Cinder Gate. Break him and the Horde scatters.

- Steps: 1. Find the Ashmaw Camp; 2. Break Grakka Ashmaw (fight `ashmaw_stand`)
- Needs: quests steel_for_the_gate; flags `ashlands.steel_spent`; starts from the giver
- Reward: The Wardens' gratitude and the chieftain's torc.; sets `ashlands.horde_broken`

### Story fights

| Fight | Where | With the party | Against | Objective | Notes |
| --- | --- | --- | --- | --- | --- |
| Holding the Courtyard (`courtyard_skirmish`) | ruined_courtyard | Mara Voss, Tev Quill | Skarn Ironjaw, Vex the Cutpurse | hold: Hold the Courtyard: keep the ring for 3 rounds, or drive them off. | - |
| Ambush at the Cinder Gate (`pass_ambush`) | cinder_gate | Mara Voss, Tev Quill | Skarn Ironjaw, Vex the Cutpurse | rout | reinforcements |
| Horde Scouts (`ash_scouts`) | ruined_courtyard | Mara Voss, Tev Quill | Horde Skirmisher, Horde Archer | rout | morale |
| Horde Raiders (`ash_raiders`) | ruined_courtyard | Mara Voss, Tev Quill | Horde Brawler, Horde Skirmisher, Horde Smokecaller | rout | morale |
| Horde Patrol (`ash_patrol`) | ruined_courtyard | Mara Voss, Tev Quill | Horde Warchief, Horde Berserker | rout | - |
| Stand at the Ashmaw Camp (`ashmaw_stand`) | ashmaw_camp | Mara Voss, Tev Quill | Grakka Ashmaw, Horde Smokecaller, Horde Archer | slay: Break Grakka Ashmaw; the Horde scatters when he falls. | 1 phases; reinforcements |
| Stand at the Ashmaw Camp (barred gate) (`ashmaw_stand_barred`) | ashmaw_camp | Mara Voss, Tev Quill | Grakka Ashmaw, Horde Smokecaller, Horde Archer | slay: Break Grakka Ashmaw; the Horde scatters when he falls. | 2 phases |
| A Horde Straggler (`ash_straggler`) | ruined_courtyard | Mara Voss, Tev Quill | Horde Brawler | rout | - |
| Raid on the Steel Wagon (`wagon_raid`) | ruined_courtyard | Mara Voss, Tev Quill | Horde Brawler, Horde Skirmisher, Horde Smokecaller | rout | guests Wenn Tallow |

- *Holding the Courtyard*: The Horde has come early this year. Two raiders drop from the Courtyard wall, blades out.
- *Ambush at the Cinder Gate*: Too quiet. Bryn raises a hand: smoke, upwind, much too close.
- *Horde Scouts*: Movement in the rubble: scouts.
- *Horde Raiders*: Raiders spill out of the ruins.
- *Horde Patrol*: The patrol has seen you.
- *Stand at the Ashmaw Camp*: Grakka rises from the fire, nail-crown glinting.
- *Stand at the Ashmaw Camp (barred gate)*: Grakka looks past you to the gorge. Nobody comes.
- *A Horde Straggler*: A straggler lurches out of the rubble.
- *Raid on the Steel Wagon*: The wagon lurches. Goblins come out of the ditch on both sides.

### Dialogues

**Mara at the map table** (`mara_talk`, Mara Voss, 9 nodes): Mara looks up from a map pinned flat with knives. "Make it quick."

- `keep`: "Warden Keep. One fire, four walls and too few of us to hold either."
- `drifts`: "Sharp eyes. Take this; you will need it before I do."
- `proud`: "It is. For now. Drink to it later."
- `steel_choice`: Mara weighs a bar of slag-steel. "Enough for one thing done well. The gate, or your hands?"
- `gate_barred`: "Then the gorge is shut behind you. Whatever waits in the wastes waits alone."
- `party_armed`: "Then go out sharp. Hest has the rest on his counter."
- `end_gate`: "One fire lit, and a door that holds. The Long Dry is over. This year."
- `end_arms`: "The wastes will remember that steel. Rest; you have earned the hearth."

**Tev by the arrow bench** (`tev_talk`, Tev Quill, 4 nodes): Tev is whittling arrow shafts. "Smoke's wrong today."

- `smoke`: "Too much of it, too close, and upwind. Someone wants us looking the wrong way."
- `thanks`: "Then I owe you a drink and an arrow. Have the arrow money."
- `steel`: "Slag Gate, up the south road. Orsa sells to us at cost. Take the signpost to the world map."

**Hest at the stores** (`hest_talk`, Hest the Quartermaster, 4 nodes): Hest counts arrows without looking up. "Buying or bothering?"

- `traded`: "Fewer bombs in the hall is fine by me."
- `needs`: "Everything. Bring the potions back unopened and we will get along."
- `spare`: "For you? Two, and don't tell the others."

**Grakka at the fire** (`grakka_parley`, Grakka Ashmaw, 2 nodes): A goblin in a crown of rusted nails grins over the fire. "Come to beg for your little flame?"

- `scorn`: "The Long Dry is mine. Your fire goes out tonight."

**Toller at the duty roll** (`muster_talk`, Brask Toller, 3 nodes): Toller runs a thumb down the duty roll. "The keep can spare a blade or two, Warden. No more."

- `roll`: Toller nods at the yard. "Pick one. The rest stay on the wall."
- `joined`: Toller strikes a line through a name. "Bring them back."

### Lore

**The Long Dry** (history): Every dry season the wells of the cinder wastes run out and the Horde marches. The Wardens call it the Long Dry, and keep a tally of how many they have survived.

**Keep One Fire Lit** (myth): The first Warden swore never to let the keep's hearth go cold. The Wardens say the Horde fears the glow more than the steel.

**The Ashmaw Banner** (rumor): A hide banner nailed to the arch, one skull per warband. Somebody is uniting the Horde.

### Disciplines

| Discipline | Faction | Passives | Description |
| --- | --- | --- | --- |
| Ember Blade | ember_wardens | steady_nerves | The Wardens' close-order sword drill: short strokes, never overextend. |
| Cinder Archery | ember_wardens | keen_senses | Shooting through smoke and heat shimmer; scouts learn to aim at the shape, not the glare. |

### Shops

- **Keep Stores Counter** (Hest the Quartermaster): con_heal_potion, con_focus_elixir, con_fire_bomb, shd_buckler, arm_leather, wpn_broadsword, wpn_claymore, shd_kite, arm_plate

### Named rewards and items

- **Wardens' Drill** (passive `steady_nerves`, rare): Mara's drill against fear, taught to those who held the line.
- **Rallying Cry** (skill `rally`, rare): The Wardens' call that steadies a wavering line.
- **Keep Ring** (item `acc_bear_ring`, rare): A heavy ring from the keep's strongbox.
- **Keep Stores** (item `con_heal_potion`, common): A satchel of the keep's healing draughts.
- **Chieftain's Torc** (item `acc_swift_amulet`, unique): 
- **Warchief's Axe** (item `wpn_battle_axe`, rare): A notched Horde axe, still warm from the warchief's grip.
- **Ember-Scale Brigandine** (item `ember_brigandine`): Leather studded with cooled slag scales.

## The Iron Reaches (`iron_reaches`)

*Mines, forges and a road south.*

A mining highland that does not inherit the Ashlands but is linked to them: its road leads down into Warden Keep.

- connects to ashlands

### Regions

| Region | Climate | Held by | Description |
| --- | --- | --- | --- |
| The Ore Highlands (`ore_highlands`) | terraced mine slopes | forge_guild | Cold terraces cut into the mountain, lit by furnace glow. |

### Factions

| Faction | Motto | Relations | Description |
| --- | --- | --- | --- |
| The Forge Guild (`forge_guild`) | Strike true. | ashlands:ember_wardens: ally | Smiths and miners who trade slag-steel to anyone who pays. |

### Places

**Slag Gate** (`slag_gate`, gate, region `ore_highlands`). The guild's southern toll gate.

- People here: Orsa Tallow; markers: chest, exit; exits: ashlands:warden_keep
- Fights here: -; rest place: yes; map `walled_town`; background art: no

### Roads

| Road | From | To | Time | Road fights | Scripted |
| --- | --- | --- | --- | --- | --- |
| The south road | slag_gate | ashlands:warden_keep | 6 | ashlands:ash_scouts (1) (20%) | ashlands:wagon_raid unless `ashlands.steel_delivered` |

### Characters

**Orsa Tallow**, Guildmistress (`orsa_tallow`)

- important NPC; warrior class on unit `warrior` (tier hero, level 5); faction `forge_guild`; home `slag_gate`
- Background: Sells the Wardens steel at cost and never says why.
- dialogue `orsa_talk`
- Art: body `warrior/orsa_tallow` (its own), portrait shared (the body's common portrait)

### Questlines

**Iron for the Embers** (`iron_for_embers`, given by Orsa Tallow, region `ore_highlands`)

Escort a slag-steel shipment down the south road to Warden Keep.

- Steps: 1. Meet Orsa at Slag Gate; 2. Deliver the shipment to Warden Keep
- Needs: quests -; flags -; starts from the giver
- Reward: A discount at the Guild forge.; sets `iron_reaches.shipment_delivered`

### Dialogues

**Orsa at the forge gate** (`orsa_talk`, Orsa Tallow, 3 nodes): Orsa wipes soot from her hands. "Wardens. Thought you'd come."

- `why`: "A Warden captain pulled me out of a slag fire. I pay the hearth, not the order."
- `loaded`: "Forty bars on one wagon. The Horde smells steel on a road, so ride hard."

## Tidewrack Coast (`tidewrack`)

*The same war, wetter.*

A salt-bitten coast where the old Warden order is a harbour guard and the Horde's descendants sail as Reef Raiders. It inherits the Ashlands and re-skins them: the same battles, the same behaviour, new faces.

- extends ashlands

### Regions

| Region | Climate | Held by | Description |
| --- | --- | --- | --- |
| The Tidewrack Coast (`tidewrack_coast`) | storm-cut cliffs and tidal flats | harbour_guard | A coast of lighthouses, wrecks and long grey beaches. |

### Factions

| Faction | Motto | Relations | Description |
| --- | --- | --- | --- |
| Harbour Guard (`harbour_guard`) | Light the lamp. | - | Lamp-keepers and pikes of the Tidewrack ports. |
| Reef Raiders (`reef_raiders`) | The tide takes, we collect. | harbour_guard: enemy | Goblin crews who strip wrecks and ports alike. |

### Places

**Gull Harbour** (`gull_harbour`, town, region `tidewrack_coast`). A crooked port town built from ship timber.

- People here: -; markers: -; exits: wreck_strand
- Fights here: -; rest place: no; map ``; background art: no

**Wreck Strand** (`wreck_strand`, wild, region `tidewrack_coast`). A beach strewn with hulls, ideal cover for raiders.

- People here: -; markers: -; exits: gull_harbour
- Fights here: -; rest place: no; map ``; background art: no

### Characters

**Ilse Marrow**, Lampwarden of Gull Harbour (`ilse_marrow`)

- playable hero; warrior class on unit `warrior` (tier hero, level 5); faction `harbour_guard`; home `gull_harbour`
- Background: A former ship's bosun who now keeps Gull Harbour's lamp lit and its raiders out. Mara Voss's counterpart in this world: she plays exactly like her.
- Personality: loud, loyal
- disciplines ember_blade; named items ember_brigandine
- Art: body `warrior` (the base unit's), portrait shared (the body's common portrait)

**Pell Driftwood**, Beachcomber Scout (`pell_driftwood`)

- playable hero; ranger class on unit `ranger` (tier hero, level 5); faction `harbour_guard`; home `gull_harbour`
- Background: Reads tides the way Tev reads smoke.
- disciplines cinder_archery
- Art: body `ranger` (the base unit's), portrait shared (the body's common portrait)

**Brannoch Saltfist**, Reef Brawler (`brannoch_saltfist`)

- enemy; brawler class on unit `goblin_brawler` (tier minion, level 3); faction `reef_raiders`; home `-`
- Background: Hauls his catch with one hand and punches with the other.
- Art: body `brawler/goblin` (the base unit's), portrait shared (the body's common portrait)

**Captain Dredge**, Reef Raider Captain (`captain_dredge`)

- enemy; brawler class on unit `goblin_berserker` (tier minion, level 3); faction `reef_raiders`; home `-`
- Background: A berserker with a ship. Tougher than the Horde's Bolg on purpose: the Forge shows exactly how.
- Art: body `brawler/goblin_berserker` (the base unit's), portrait shared (the body's common portrait)

**Salt-Eyed Sniper**, Mast Archer (`salt_archer`)

- enemy; ranger class on unit `goblin_archer` (tier minion, level 3); faction `reef_raiders`; home `-`
- Background: Perches in rigging and shoots anyone who looks up.
- Art: body `ranger/goblin` (the base unit's), portrait shared (the body's common portrait)

**Tide-Whisper**, Reef Shaman (`tide_shaman`)

- enemy; priest class on unit `goblin_shaman` (tier minion, level 3); faction `reef_raiders`; home `-`
- Background: Listens to the surf for omens.
- Art: body `priest/goblin` (the base unit's), portrait shared (the body's common portrait)

**Gull-Hand**, Raider (`gull_skirmisher`)

- enemy; rogue class on unit `goblin_skirmisher` (tier minion, level 3); faction `reef_raiders`; home `-`
- Background: Quick, light and never where you aimed.
- Art: body `rogue/goblin` (the base unit's), portrait shared (the body's common portrait)

### Story fights

| Fight | Where | With the party | Against | Objective | Notes |
| --- | --- | --- | --- | --- | --- |
| Clash at Wreck Strand (`strand_clash`) | wreck_strand | Ilse Marrow, Pell Driftwood | Brannoch Saltfist, Gull-Hand, Salt-Eyed Sniper, Captain Dredge | rout | - |

- *Clash at Wreck Strand*: The tide is going out and the raiders are coming in.

### Lore

**The Lamp and the Tide** (history): Tidewrack's founders were Ashlands Wardens who followed the smoke to the sea. The order's creed, keep one fire lit, became the harbour lamp.

## Quest templates (shared by every world)

| Template | Objectives | Default reward |
| --- | --- | --- |
| Reach and clear (`reach_and_clear`) | reach_location, win_encounter | {"xp": 60, "items": {"con_heal_potion": 2}, "gold": 30} |
| Two-stop delivery (`two_stop_delivery`) | reach_location, reach_location | {"xp": 40, "items": {"con_focus_elixir": 1}, "gold": 25} |
| Clear a site (`clear_site`) | win_encounter | {"xp": 80, "items": {"con_heal_potion": 1}, "gold": 40} |
| Signal and return (`signal_and_return`) | flag, reach_location | {"xp": 50, "items": {"con_focus_elixir": 1}, "gold": 20} |
