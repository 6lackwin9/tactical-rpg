---
class: DojoPanel
extends: UiPanel
description: "Tactical RPG - Dojo Panel Base UiPanel with a strongly-typed DojoSession reference. Dojo components extend this and read/mutate state only through `session`."
---

# Properties
* `session`: DojoSession = null

# Methods

### `_bind_session(p_session: RefCounted) -> void`
```logic
session = p_session as DojoSession
```

