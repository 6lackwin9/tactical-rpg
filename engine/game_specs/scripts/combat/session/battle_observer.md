---
class: BattleObserver
extends: RefCounted
description: "Tactical RPG - Battle observer hooks (BREPORT1-L 1.1, moved from tools/sim/sim_observer.gd) The base does nothing. The balance sim runner (tools/sim/sim_runner.gd) calls the four hooks; the per-unit collector BattleMetrics extends it (and can also attach to a live session, BREPORT1 1.2). After session.start() and the start-distance placement, before the first plan."
---

# Methods

### `on_start(_session: CombatSession) -> void`
```logic
pass
```

### `on_step(_session: CombatSession) -> void`
```logic
pass
```

### `on_round(_session: CombatSession) -> void`
```logic
pass
```

### `on_end(_session: CombatSession, _result: Dictionary) -> void`
```logic
pass
```

