# Open questions: world, campaign, roster, hub

Status: open (2026-10-25, the lore consistency pass, final check). Part of [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md),
which explains the columns and lists the questions to decide first. Ids are stable: a decided question keeps its id
and gets "Decided (date, by whom): ..." in its Rec. column; the planner copies the decision to
`games/emberwake/docs/decisions/lore.md`.

Columns: Id; Question; Options; Rec. (the recommendation, never applied until decided); Where (doc and section,
paths from `games/emberwake/docs/lore/`); Decides (user, balance lane, design lane, data authoring).

## Story and world

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-01 | What does the Red Tithing really do, and how do the Act 3 win and the Doom loss relate? | (a) the lullaby is a harvest: a dreaming titan bleeds the marrow the Clerisy drinks; (b) the lullaby is real and the Clerisy is also corrupt | Decided (2026-10-25, user): a; the Clerisy is a selfish elite feeding the horror for eternal youth, while the horror is blamed for the folk's woes | background/central_truth.md | user |
| Q-02 | When does the player learn the truth? | (a) a hint at the Embalming Cloisters (2.4), the full reveal from Malachi before node 21; (b) only at the Dreaming Core | a | central_truth.md "What the user must decide" 2 | user |
| Q-03 | Do ordinary Spent adults ever drink marrow? | (a) no, only the Clerisy; (b) yes, diluted in the draughts | a | central_truth.md 3 | user |
| Q-04 | Size and frequency of the Red Tithing | (a) a yearly rite of hundreds plus a Great Tithing of thousands that the Doom Clock counts; (b) qualitative only ("every Unsealed youth in the pens") | Follow-up (2026-10-25, user): a accepted in principle; needs story context (what commoners know, why the Great Tithing is needed now, precedent): options in background/tithing_options.md | cosmology_and_setting 5, 6; factions 2.1; onboarding 2; mantle 1; choices 6 | user |
| Q-05 | Do operatives age, and what happens to one who turns 25 during the campaign? | (a) no: the campaign spans weeks, the 25 line is a world fact; (b) ages advance at act transitions | a | cosmology_and_setting 3.2, 6 | user |
| Q-06 | How do the regime's adult officers (Vane, Quell, Nemeth, Ruthven, Grendel, Orlov) act with purpose if the Spent are docile? (found in the final check) | (a) the draughts suppress dissent, not competence: the Spent serve the order well; (b) officers avoided communion or take Clerisy marrow (ties to Q-03) | a; their ages stay unset | cosmology_and_setting 3.1; background/cast_registry.md | user |
| Q-07 | What do the forbidden texts Maren copied reveal? | (a) the true history of the Tithing (follows Q-01); (b) a plain heresy, no reveal | a | narrative/prologue_and_starter_squad_lore.md 2.3, 5 | user |
| Q-08 | When and where was Kaelen's involuntary graft trial (Act 2 hook)? | written by the design lane after Q-01 | - | prologue_and_starter_squad_lore 2.1, 5 | design lane |
| Q-09 | Bell resonance: CANON 3.3 switches it on at the Act 2-3 transition, but Act 2 M2.5 "silences the bells' resonance", so it already acts in Act 2 | (a) resonance starts in Act 2 in Upper Cinderreach, M2.5 ends it there, Act 3's continental resonance is a new wave; (b) none in Act 2: M2.5 gets another payoff, resonance starts in Act 3 | no pick (left to the user by the pass) | CANON 3.3, 3.4; upper_bastion 4.5; mantle 2.2; six_founders 5.1 | user |
| Q-10 | What bell resonance does to calcified operatives (the old out-of-combat Strain clashes with the after-battle reset) | (a) Stage I+ operatives start the next battle at +X Strain (mantle 2.2 writes +10); (b) only an in-battle pulse | a | six_founders 5.1; mantle 2.2 | user, then balance lane |
| Q-11 | Whose beasts are the Render-Beasts? | (a) Guild-bred; the Inquisition borrows packs at Lockdown; (b) Inquisition-bred | a | systems/enemy_bestiary_and_ai.md gap 7 | data authoring (user confirms) |
| Q-12 | "The tomb of the first Extinguishers" adds a third word on the Extinguishing stem | (a) keep it and say who they were; (b) drop the word (the tomb is the Reliquary Crypts of the First Apostles) | b | upper_bastion 4.6 | user |

## Campaign and meters

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-13 | What does Inquisition Alert 100% do in Act 1 (no siege there)? | (a) a forced Render-Beast hunt mission until Alert drops below 100 (canon writer); (b) cap at 99 in Act 1, Lockdown effects hold (narrative writer) | Decided (2026-10-25, user): an Act 1 Breach ends the run, reachable only by blatantly ignoring the meter; from Act 2 Alert is active management | CANON 2.6; background/sunk_ward_campaign_map.md 2.2; safehouse gap 5; campaign 8 | user |
| Q-14 | Alert across act lines | (a) carries over with a floor of 30; (b) resets to 0 | a | CANON 3.3 | user |
| Q-15 | Alert inputs and Lockdown effects: what raises it, what lowers it, what Lockdown changes beyond roaming packs | (a) loud and quiet tags on missions plus bribes (Scrap, Rock Salt) and the listed story amounts; numbers by the balance lane; (b) per-action Alert | a | systems/safehouse_economy_and_triage.md gap 1; narrative_agency 2.2, 3.3; sunk_ward 2 | balance lane, data authoring |
| Q-16 | What happens when the soft Doom Clock reaches 0 in Acts 1-2? | (a) recurring lesser Tithings with story costs, then the Great Tithing date revealed at M2.4; (b) the clock holds at 7 days until the story moves | Follow-up (2026-10-25, user): a in principle, with Q-04's story context | CANON 2.6; narrative_agency 4 | user |
| Q-17 | The true Tithing date (M2.4) and the Act 3 day budget | (a) the found date corrects the visible clock once and becomes the Act 3 budget (days by the balance lane); (b) Act 3 restarts the clock at a fixed budget | a | upper_bastion 4.4; mantle 2.1 | user, then balance lane |
| Q-18 | Zero Warded Tallow in the Low Sleep: CANON has -2 starting AP, while sunk_ward 2.1 and mantle 2.3 also keep the travel runtime's x1.5 road-fight chance | (a) both effects; (b) only -2 AP in the Low Sleep; x1.5 stays a "classic" (Ashlands) option of the purse hook | b (one penalty, DEC-LORE-24) | CANON 2.5; sunk_ward 2.1; mantle 2.3; specs/rpg_kit_and_system_architecture.md 3.3 | user |
| Q-19 | Doctrine inputs, thresholds and visibility | (a) grafts and high-Strain battles push Flesh-Broker, forge projects and salt flushes push Preservationist, about +-5 per input, a visible meter; (b) scripted hub choices only | a | factions 4; choices 3; roadmap P2-04 | user, balance lane numbers |
| Q-20 | A lost Act 1 battle (all four downed) | (a) retreat to the hub, all wounded, the mission stays on the board, a day passes; (b) reload | a | campaign_narrative 8; narrative_agency 4 | user |
| Q-21 | Defeat in the final battle (ENC-ACT3-06) | (a) Extinguished; (b) the normal downed and capture rules, retry while any operative remains | b | mantle 4.6 | user |
| Q-22 | Siege content (Acts 2-3): map, facility damage, relocation, Bram and Aline | (a) reuse the ENC-EVT-05 chamber; a lost siege takes 1 level from each facility and moves the cell to the act's forward base; Bram and Aline are protect targets; (b) a new siege map per act, named facilities lost | a | choices 5; narrative_agency 3.3; roadmap P2-06 | user |
| Q-23 | Forward bases (the Scribe's Crypt, the Weeping Redoubt) | (a) story only: the Sluice menu renamed, nothing copied; (b) real second hubs with copied facilities | a | world_geography 2.2-2.3 | user |
| Q-24 | Captain Vane after M1 | (a) dies at M1; (b) retreats; a later bounty beat | b | sunk_ward 4 | user |
| Q-25 | Nemeth at the Crucible: defeat or escape? (also decides whether Nemeth's Warded Cassock drops) | (a) the cell escapes, Nemeth survives into Act 2; (b) the cell defeats him, then the sacrifice | Decided (2026-10-25, user): a (escape) | CANON 6; sunk_ward 3.7; systems/bestiary_iron_inquisition.md; items 7, gap 5 | user |
| Q-26 | Is Inquisitor-General Ruthven ever fought? | (a) as the Act 2 siege commander; (b) he stays off stage | a | upper_bastion 4.1; npcs 3.3 | user |
| Q-27 | The act of Bram's quest "Cold Slag, Black Iron" | (a) Act 2, after Lucan's rescue; (b) Act 1 board work | a | npcs 2.2; skill_acquisition 1.1 | user |
| Q-28 | Act 1 founder hooks: rewards of Kaelen's bellows and Hollis's vantage points, hooks for Dunstan and Elyse, Vesper's vendetta mechanic | (a) each hook raises a facility level or changes M5 (vantage points: a deployment zone at the Crucible); the vendetta is a bonus objective with a reward item; the design lane writes Dunstan's and Elyse's hooks; (b) flavour only | a | campaign_narrative 4.2; sunk_ward 3.5 | user, design lane |
| Q-29 | What the youths freed in M2 (and under-18 youths freed from Act 2 wagons) give the cell | (a) a "saved" counter read by scenes and endings; under-18s go to Agnes's attics or Pip's network; (b) nothing beyond the scene | a | sunk_ward 3.3; upper_bastion 2.3 | user |
| Q-30 | Where the optional board jobs sit on the graph, including Corvo's Third Ward gate | (a) markers on nodes plus a gated route between 03 and 04 (Corvo holds the barricade between the Ash-Crawl and the Fourth Kiln); (b) board-only jobs with no node | a | sunk_ward 3.8; npcs 3.2 | data authoring |

## Roster and lifecycle

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-31 | What "wounded" does in Act 1 and how it heals | (a) sits out the next mission or 2 rest days; (b) deploys at -20% max HP until a rest | Decided (2026-10-25, user): b, -20% max HP until a rest as a baseline; the balance lane revisits | CANON 2.4; roster gap 1; narrative_agency 4; world_geography 2.1 | user |
| Q-32 | Downed operatives from Act 2 | (a) dead at 0 HP; (b) downed units bleed out after N rounds unless revived; on a lost battle the downed are captured | b | narrative_agency 4; campaign_narrative 8 | user |
| Q-33 | Can a captured founder be rescued in Act 2? | (a) no: a captive returns only as the optional Act 3 boss; (b) a rescue mission | a | CANON 6; roster 6 | user |
| Q-34 | Keepsake numbers | (a) one ACC passive at about 50% of the founder's trait; (b) a flat stat item | a | CANON 3.3; choices 2 | balance lane |
| Q-35 | Crucible staying beats (Maren, Hollis, Elyse) and keepsake names (Vesper, Maren, Dunstan, Elyse) | written by the design lane | - | narrative_choices 2, 8 | design lane |
| Q-36 | Isolation Crypts and bunks | (a) a crypt place keeps the operative's bunk; Living Martyrs take a crypt place and no bunk; the move is automatic at Stage IV; (b) crypt places are a separate facility level | a | roster 7; recruits 4.2; six_founders 5.3 | user |
| Q-37 | Living Martyrs: the old "+1 Calcification stage per battle, Stage V after 1-2 missions" clashes with CANON 2.3 (only Flesh Breach and Tier III advance stages); also their numbers, the mercy reward, the stasis cost | (a) a lifecycle expiry after N missions instead of stage advances; stasis costs only the crypt place; no mercy reward until a trait system; (b) CANON 2.3 unchanged for them | a (narrative writer); the rule clash itself is the user's call | recruits 4.2; choices 4; roadmap P2-03 | user, balance lane numbers |
| Q-38 | The 8-bunk project: when and what it costs | (a) Act 2 on, a Scrap cost from the balance lane; (b) available in Act 1 | a | CANON 2.4; safehouse 2 | balance lane |
| Q-39 | Guests: ages of Ferrand and Clemence, and who can be sworn in | (a) they are adults (25+), perk-only guests; only named rescues aged 18-24 can be sworn in; (b) both are 18-24 and can be sworn in | a | recruits 3; cast_registry | user |
| Q-40 | Small roster rules: retirement bonus, dismissal, recruit class, origin tags and temperaments, grief traits, founder-dead variants of later beats | retirement: none; dismissal: gear stays in the stash; recruit class: set by the encounter; origin: a small stat or skill, temperament flavour only; grief traits: none until a trait system; variants: design lane | as listed | roster 9; narrative_agency 1.2, 3.1 | user, design lane |
| Q-41 | Founders' levels and gear in the slice, starting Scrap and Tallow | proposed by the balance lane after M1 | - | roadmap gap 5 | balance lane |

## Hub and economy

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-42 | Facility levels beyond the bunk project | (a) only the bunk project in the slice; Act 1 founder hooks may add Hearth and Slab levels later (Q-28); (b) a full level tree per facility | a | safehouse gap 2 | user |
| Q-43 | What a Rock Salt flush does | (a) cancels the next Calcification advance (one charge; canon writer); (b) -10 dormant floor for the next battle (combat writer) | b (one number, reads on the card); see Q-69 | CANON 6; magic gaps; safehouse 2.1 | user |
| Q-44 | Prices: forge and reinforcement, bribes, graft costs, the per-graft floor step, drop sizes beyond the Act 1 purses | per-graft step per organ in data (default +10); the rest by the balance lane | as listed | safehouse gaps 3-4; skill_acquisition 1.2 | balance lane |
| Q-45 | Act 3 Tallow scarcity (the old "daily use doubles") | (a) Tallow costs more Scrap in Act 3; daily use stays 1; (b) daily use doubles | a | six_founders 5.2; roadmap P3-02 | user |
| Q-46 | Hub screens and meters | facility screens opened by the keeper NPC (rec) or a facility list; the Doom Clock and Alert on the map HUD bar only (rec) or also in battle | as listed | screen_architecture 7; safehouse gap 6 | user |
