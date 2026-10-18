---
class: ActionLogEntry
extends: RefCounted
description: "Tactical RPG - Action Log Entry Model Standardized action event structure for timeline cells and expanded stream log Test mode: log cells explain damage on hover. On in debug builds (editor / test runs), off in release builds."
---

# Properties
* `round_num`: int = 1
* `block_num`: int = 1
* `ref_tag`: String = "R1-B01"   # R<X>-B<Y>
* `actor_name`: String = ""
* `actor_is_ally`: bool = true
* `target_name`: String = ""
* `target_is_ally`: bool = false
* `category`: String = "ATTACK"  # ATTACK, DEFEND, BUFF, MOVE, TARGET, EVENT
* `skill_name`: String = ""
* `skill_subtype`: String = ""
* `value_num`: int = 0
* `outcome_tag`: String = "HIT"
* `detail_text`: String = ""
* `status_text`: String = ""
* `damage`: int = 0              # ATTACK: damage actually dealt
* `breakdown`: String = ""       # ATTACK: how that number came about (formula hover in test mode)
* `coord_from`: Vector2i = Vector2i(-1, -1)
* `coord_to`: Vector2i = Vector2i(-1, -1)

