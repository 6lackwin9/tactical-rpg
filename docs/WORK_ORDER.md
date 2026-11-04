# Work order

Decision "Order of work: engine and kit first" (user 2026-11-05). `docs/TASKS.md` stays the queue: this file says which phase each open row belongs to. The planner sorted the rows; the user moves any that sit wrong. Each system builds its own placeholder widgets and mockups and takes just enough Sunk Ward data to test on.

| Phase | Goal | Rows |
| --- | --- | --- |
| 0. Base rules | Stats, damage, items, classes and the rule library: which options work together, conflict or ship together | BASERULES1-M and BASERULES1-F, the wider flat model (the minimum before battle; decision "Combat tunes on its own values, not on base rules"); in the base silo, not gating battle: BASERULES1-DOM, BASERULES1-V, BASERULES1-D, KITIDS1, STATUSROWS1, BASERULES1-G, BASERULES1-U, BASERULES1-P, BASERULES1-PP, BASERULES1-B, BASERULES1-A, BASERULES1-BN, STATUSPASS1, LEVERS1-K, OFFHAND1, DRILLSAVE1, TERMS1 |
| 1. Battle | The combat built on those rules | ATTACKLOCK1-O, COMBATLIMIT1, WORLDFIGHT1, INTENT2-K1, INTENT2-K2, INTENT2-K3, BASERULES1-H, INTENT2-AI, INTENT2-A, INTENT2-U, INTENT2-B, SKILLPASS1, POISE1, TACTIC2, BATTLEMAP1, OBJ1-U, ENCOUNTER2-C, ENCOUNTER2-L, ENCOUNTER2-X, ENCOUNTER2-F, ENCOUNTER2-R, ENCOUNTER2-RA, HOOKS1-Q, BIGUNIT1, ROSTER1-B, ROSTER1-M, PLAYBACK2, SIMKPI1, CROWDFIGHT1, VISCLAR1 |
| 2. Exploration and maps | The region map and how it plays | INTEG1-G, SERVICES1, GALLERYWORLD1, TURNFEEL1, EXPLOREVIEW1-R, FPSCAP2, REGIONGROUND1-W, REGIONGROUND1-S, REGIONGROUND1-R1, REGIONGROUND1-C, REGIONGROUND1-X, REVEAL1, GROUNDTUNE1, LOOK1-L2, MARKERS1, ARTSCALE1, REWARDXW1, PLAYTEST2 |
| 3. Other gameplay | Party, progression, economy, quests, story systems | SHOPTEST1, ROSTER1-T, ROSTER1-P, ROSTER1-K, DEEDS1, KEEP1, RESPEC1, DIFF1, CAMPAIGN1-P, CAMPAIGN1-DOC, QUESTTRACK1-U, DRAG1, STORYUI1-F |
| 4. UI screens | Final screens, layouts as data, final look | KIT2-UI, LAYOUT1, LAYOUT720-Q, LAYOUT720-G0, LAYOUTDATA1-E2, LAYOUTDATA1-D1, LAYOUTDATA1-D2, LAYOUTDATA1-M, LAYOUTDATA1-G1, LAYOUTDATA1-G2, LAYOUTDATA1-R, LOOK1-M2, LOOK1-M2-L, LOOK1-M2-B1, LOOK1-M2-B2, LOOK1-M2-C, LOOK1-M1F, UIR-4, UIR-7, RCLOSE1-U, ICONUI1, ROSTER1-BU, ROSTER1-RU, ENCOUNTER2-RU, PUPPET1, SKILLTEXT2, SKILLICONS1 |
| 5. Data | Bulk content and art | CAST1-A5, CAST1-A6, CAST1-A6b, CAST1-B, CAMPAIGN1-D, ROSTER1-D, ROSTER1-TA, ROSTER1-KD, STORYUI1-A, ENCOUNTER2-CA, ENCOUNTER2-D2, FX1-A, LOOK1-A, LOOK1-R, REGIONGROUND1-A2, REGIONGROUND1-D2, REGIONGROUND1-A3, REGIONGROUND1-D3, PROPS1-A3a, PROPS1-A3b, PIXEL4, MOB1, ALINE1-F, PORTRAITFX1-XA, PORTRAITFX2-A, ART2, DIR4-R4, HOLDART1 |
| 6. Integration | The whole game, performance, the slice playtest | INTEG1-E2, INTEG1-E3, INTEG1-S2, INTEG1-X, CAMP1-P |

Any time, as they serve every phase: INTEG1-R, INTEG1-S1 and SILOSCN1 (a small set of headless scenarios at each flush); SILOCARD1, SKILLS1, SCOPE1, KITDONE1, LINKGATE1, TOOLTESTS2, WEBPROBE1, ARTTOOL1, CLN-23, CLN-27, CLN-30, CLN-33, CLN-34, CLN-49, CLN-53, CLN-71.

Parts of phase 5 move up when a system needs them to test: a Sunk Ward fight for battle, one Sunk Ward map for exploration. PLAYTEST2 comes after WORLDFIGHT1. The map of systems and their joins: `docs/specs/SILOS1.md`. M1 waits on the user's look at the battle view; M2, the vertical slice, comes after phase 6 by design.
