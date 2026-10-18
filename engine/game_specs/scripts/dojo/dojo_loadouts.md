---
class: DojoLoadouts
extends: RefCounted
description: "Tactical RPG - Dojo Quick-Setup Loadout Presets Declarative gear presets. Pieces are taken from the shared stash when present and spawned from ItemCatalog otherwise, so any preset can always be applied. Class proficiency rules still apply: ineligible pieces are skipped and reported. Presets with a main-hand weapon in the off hand (claymore_backup, dual_wield, archer, duelist) are legacy dual wield: they need Equipment.allow_plain_dual_wield, which the dojo session sets (FORGE1_ITEMS 8.1)."
---

# Methods

### `static find_preset(preset_id: String) -> Dictionary`
```logic
for preset: Dictionary in get_presets():
	if str(preset.get("id", "")) == preset_id:
		return preset
return {}
```

