# Balance report (written by the balance agent; read by the brainstorm agent)

> NOTE 2026-10-15: the roster was reworked (M2c-1: warrior tank, ranger, priest, brawler, rogue, mage; M1 combat extensions; AI fixes). The tables below describe the OLD roster (shaman/berserker/scout) and are STALE; rerun `tools/fc.sh sim ...` once the balance-relevant systems are frozen.

Target: every class between **40% and 60%** average duel win rate at each tested level (10 seeds near the bracket). Only systems marked `balance-ready` in `docs/ROADMAP.md` are tested. Append new runs on top; never delete old runs (history).

## Run 2026-10-15 (baseline, level 5, 3 seeds, systems S1-S4 before any tuning)

| Class | Duel avg | In 40-60%? | Notes |
| --- | --- | --- | --- |
| warrior | 100% | no | armor 25 (plate + kite shield), HP 314 |
| berserker | 80% | no | Reckless Swing 1.7x |
| brawler | 53% | yes | |
| scout | 47% | yes | fastest, Quick Stab 0.6x |
| ranger | 20% | no | HP 158, armor 7, AI kites and wastes turns |
| shaman | 0% | no | armor 2, no body armor item, Hex 0.6x / Arcane Bolt 1.1x; cannot beat a minion of itself |

Tier 1v1 (hero vs own class): minion 100%, standard 100%, elite 50%, boss 0%. 1v3 minions: hero loses (except shaman stalemates).

Issues for the brainstorm agent: (1) warrior dominance from gear budget; (2) shaman damage + armor + AI turn waste; (3) ranger durability/AI; (4) are 1v3 and boss outcomes intended?

> NOTE 2026-10-16: since the baseline run below, the game gained new class kits (M2C2B), the AI1 skill use (riposte finisher, taunt, disengage, areas), statuses and gear effects. BAL1 must A/B `AiProfile.smart_skills` and rerun duel, tier and level tables; XP curve values are starting values.
