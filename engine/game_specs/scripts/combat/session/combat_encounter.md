---
class: CombatEncounter
extends: RefCounted
description: "Tactical RPG - Combat Encounter (plug-in definition of a battle) Describes WHAT a battle is: combatants, grid and deployment, per-round plans, mid-round triggers (e.g. reinforcements) and round narration. CombatSession owns HOW it plays out (resolution, snapshots, playback, selection, log). The combatants created in build() are the single source of truth: the session simulates them and every UI panel (tokens, roster, inspector) renders them. Subclass and override the virtual methods below."
---

# Properties
* `GUEST_PRESET`: String = "balanced"   # OBJ1 rule 2: the AI drafts guests with the Balanced tactic (const)
* `encounter_name`: String = "Encounter"
* `max_rounds`: int = 1
* `total_blocks`: int = 6      # blocks in a round (roughly three actions)
* `ap_regen`: int = 4          # AP regained at each round start (-1 = refill to max)
* `unit_ap`: int = 12          # AP pool of units without their own (carries across rounds)
* `rng_seed`: int = 1337      # seeds status rolls so a battle replays identically
* `grid`: CombatGrid = null
* `allies`: Array[Combatant] = []        # the heroes, then the guests (OBJ1 rule 1)
* `guests`: Array[Combatant] = []        # party-side extras (also in allies): planned like heroes, no XP, never saved
* `enemies`: Array[Combatant] = []        # includes reinforcements not yet on the field
* `party_stash`: Inventory = null
* `party_cap`: int = 0                    # party slots the combat HUD shows; 0 = unknown (the HUD falls back to its default)
* `reinforcements_spawned`: bool = false  # generic "extra wave has entered" flag (snapshotted)
* `zone_id`: String = "stone_arena"      # CombatZones terrain preset applied in build()
* `unit_classes`: Dictionary = {}         # Combatant -> class key (UnitArt.CLASS_KEYS)
* `neutral_units`: Array[Combatant] = []  # units on neither side (e.g. training dummies)
* `out`: Array[Combatant] = []
* `out`: Array[Combatant] = []
* `ai_profiles`: Dictionary = {}    # Combatant -> AiProfile
* `ctx`: PlanContext = shared.context if shared != null else plan_context()

# Methods

### `build() -> void`
```logic
pass
```

### `prepare_round(_round_num: int) -> Dictionary`
```logic
return {}
```

### `schedule_round(resolver: CombatResolver, _round_num: int) -> void`
```logic
schedule_enemy_ai(resolver)
```

### `on_block_start(_round_num: int, _block_num: int) -> ActionLogEntry`
```logic
return null
```

### `on_block_end(_round_num: int, _block_num: int) -> void`
```logic
pass
```

### `is_reserved(_c: Combatant) -> bool`
```logic
return false
```

### `verdict(_round_num: int, _block_num: int, _end_of_round: bool) -> String`
```logic
var standing: bool = false
var alive: bool = false
for a: Combatant in allies:
	standing = standing or a.is_active()
	alive = alive or a.is_alive()
var foe_standing: bool = false
for e: Combatant in live_enemies():
	foe_standing = foe_standing or e.is_active()
if standing and foe_standing:
	return ""
if standing:
	return "won"
return "fled" if alive else "lost"
```

### `is_guest(c: Combatant) -> bool`
```logic
return c != null and guests.has(c)
```

### `party_preset_for(c: Combatant, preset_id: String) -> String`
```logic
return GUEST_PRESET if is_guest(c) else preset_id
```

### `objective_state() -> Dictionary`
```logic
return {}
```

### `restore_objective_state(_state: Dictionary) -> void`
```logic
pass
```

### `phase_state() -> Dictionary`
```logic
return {}
```

### `restore_phase_state(_state: Dictionary, _round_num: int) -> void`
```logic
pass
```

### `phase_info(_entry: ActionLogEntry) -> Dictionary`
```logic
return {}
```

### `round_summary(_round_num: int, _combat_over: bool) -> Dictionary`
```logic
return {}
```

### `advance_label(round_num: int) -> String`
```logic
return "⚔ ADVANCE TO ROUND %d" % (round_num + 1)
```

### `class_of(c: Combatant) -> String`
```logic
if c == null:
	return "unit"
return str(unit_classes.get(c, c.name.to_lower()))
```

### `get_token_texture(c: Combatant) -> Texture2D`
```logic
return UnitArt.token(class_of(c), team_of(c)) if c != null else null
```

### `get_sprite_texture(c: Combatant) -> Texture2D`
```logic
return UnitArt.portrait(class_of(c), team_of(c)) if c != null else null
```

### `plan_context() -> PlanContext`
```logic
return PlanContext.create(grid, allies, enemies, ai_profiles, total_blocks)
```

### `profile_of(c: Combatant) -> AiProfile`
```logic
return plan_context().profile_of(c)
```

### `hero_key(c: Combatant) -> String`
```logic
if c == null or guests.has(c):
	return ""
return c.lore_ref if not c.lore_ref.is_empty() else c.name
```

### `schedule_enemy_ai(resolver: CombatResolver) -> void`
```logic
var ctx: PlanContext = plan_context()
var enemy_plan := CombatPlan.new(ctx)
var drafted: Array[Combatant] = []
for e: Combatant in enemies:
	if e.is_active() and grid.has_combatant(e):
		drafted.append(e)
		TacticalAI.build_steps(e, ctx, ctx.profile_of(e), enemy_plan)   # drafts into the shared plan: packmates avoid each other
# A packmate drafted later can claim a tile an earlier draft needed and break it (the commit would then drop that
# unit's actions). Re-draft the broken ones in enemy order (a re-draft moves the unit to the end of the plan's
# sequences, which only affects commit order), a bounded number of passes.
for _pass: int in range(drafted.size()):
	var redrafted: bool = false
	for e: Combatant in drafted:
		if _draft_broken(enemy_plan, e):
			TacticalAI.build_steps(e, ctx, ctx.profile_of(e), enemy_plan)
			redrafted = true
	if not redrafted:
		break
# Still broken: keep each unit's valid prefix (none: hold). Plans only shrink, so this ends.
var trimmed: bool = true
while trimmed:
	trimmed = false
	for e: Combatant in drafted:
		if _draft_broken(enemy_plan, e):
			var st: CombatPlan.SimState = enemy_plan.compile(e)
			var steps: Array[PlanStep] = enemy_plan.steps_of(e)
			var keep: int = st.error_index if st.error_index >= 0 else 0
			enemy_plan.set_steps(e, steps.slice(0, keep))
			trimmed = true
enemy_plan.commit(resolver)
```

### `static _draft_broken(enemy_plan: CombatPlan, e: Combatant) -> bool`
```logic
if enemy_plan.steps_of(e).is_empty():
	return false
var st: CombatPlan.SimState = enemy_plan.compile(e)
return st.error_index >= 0 or st.entries.is_empty()
```

### `static make_event_entry(round_num: int, block_num: int, actor: String, actor_is_ally: bool, skill: String, subtype: String, outcome: String, detail: String, status: String) -> ActionLogEntry`
```logic
var entry := ActionLogEntry.new(round_num, block_num, "EVENT", actor, "", detail, 0, outcome)
entry.actor_is_ally = actor_is_ally
entry.skill_name = skill
entry.skill_subtype = subtype
entry.status_text = status
return entry
```

### `party_slots() -> int`
```logic
return party_cap
```

