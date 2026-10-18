---
class: StatusBlockCommand
extends: BattleCommand
description: "Tactical RPG - Status Block (a block lost to Stagger / Stun) Occupies the block on the unit's timeline so nothing else can use it."
---

# Properties
* `status`: String = ""

# Methods

### `_init(p_source: Combatant, p_status: String) -> void`
```logic
super(p_source, 0, 1)
status = p_status
command_name = StatusEffects.display_name(p_status)
```

### `execute() -> void`
```logic
execution_log = "[Status] %s is %s and loses this block." % [source.name, StatusEffects.display_name(status).to_lower()]
```

