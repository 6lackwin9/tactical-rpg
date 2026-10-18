class_name CombatEncounter
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Encounter (plug-in definition of a battle)
# Describes WHAT a battle is: combatants, grid and deployment, per-round plans,
# mid-round triggers (e.g. reinforcements) and round narration. CombatSession
# owns HOW it plays out (resolution, snapshots, playback, selection, log).
#
# The combatants created in build() are the single source of truth: the session
# simulates them and every UI panel (tokens, roster, inspector) renders them.
# Subclass and override the virtual methods below.
# ==============================================================================

const GUEST_PRESET: String = "balanced"   # OBJ1 rule 2: the AI drafts guests with the Balanced tactic

var encounter_name: String = "Encounter"
var max_rounds: int = 1
var total_blocks: int = 6      # blocks in a round (roughly three actions)
var ap_regen: int = 4          # AP regained at each round start (-1 = refill to max)
var unit_ap: int = 12          # AP pool of units without their own (carries across rounds)
var rng_seed: int = 1337      # seeds status rolls so a battle replays identically
var grid: CombatGrid = null
var allies: Array[Combatant] = []        # the heroes, then the guests (OBJ1 rule 1)
var guests: Array[Combatant] = []        # party-side extras (also in allies): planned like heroes, no XP, never saved
var enemies: Array[Combatant] = []        # includes reinforcements not yet on the field
var party_stash: Inventory = null
var party_cap: int = 0                    # party slots the combat HUD shows; 0 = unknown (the HUD falls back to its default)
var reinforcements_spawned: bool = false  # generic "extra wave has entered" flag (snapshotted)

var zone_id: String = "stone_arena"      # CombatZones terrain preset applied in build()
var unit_classes: Dictionary = {}         # Combatant -> class key (UnitArt.CLASS_KEYS)
var neutral_units: Array[Combatant] = []  # units on neither side (e.g. training dummies)

# Creates combatants, grid and initial deployment. Called on start and full reset.
func build() -> void:
	pass

# Applies the starting state of `round_num` (round 1 right after build()).
# Returns intro info: {"entry": ActionLogEntry or null, "entries": Array[ActionLogEntry] (logged after entry; BOSS1
# phases fired at the round start), "phase": String, "status": String, "log": String}
func prepare_round(_round_num: int) -> Dictionary:
	return {}

# Schedules the enemy side for `round_num` (the party is planned by the player and
# committed on Confirm). Default: every enemy on the field plans with TacticalAI.
func schedule_round(resolver: CombatResolver, _round_num: int) -> void:
	schedule_enemy_ai(resolver)

# Mid-round trigger evaluated before a block resolves. Return an EVENT entry to
# announce something (the block then resolves on the next step), or null.
func on_block_start(_round_num: int, _block_num: int) -> ActionLogEntry:
	return null

# After a block resolves (after round-end effects on the last block, before the end-of-battle check): a chance to
# notice state changes (BOSS1: hp_below phases become due). Default: nothing.
func on_block_end(_round_num: int, _block_num: int) -> void:
	pass

# Right after an action resolves (before its log entry): a chance to read what it did (MORALE1: crits). Default: nothing.
func on_action_resolved(_action: CombatResolver.ScheduledAction) -> void:
	pass

# MORALE1 rule 13: the shown morale state of `c` ("steady", "shaken", "wavering", "fearless"; "" = morale off or none).
func morale_state_of(_c: Combatant) -> String:
	return ""

# MORALE1 rule 13: the morale group index of `c` (-1 = morale off or none).
func morale_group_of(_c: Combatant) -> int:
	return -1

# MORALE1 rule 8: the flee ratio `c` drafts with, given its live profile's `base`. Default: `base`.
func flee_ratio_of(_c: Combatant, base: float) -> float:
	return base

# A unit spawned but held back (BOSS1: a phase wave before its phase fires): off the field and skipped by the
# end-of-battle check. Default: none.
func is_reserved(_c: Combatant) -> bool:
	return false

# The enemies that count for the end-of-battle check (every enemy that is not reserved).
func live_enemies() -> Array[Combatant]:
	var out: Array[Combatant] = []
	for e: Combatant in enemies:
		if not is_reserved(e):
			out.append(e)
	return out

# OBJ1 rule 3: after each block (after round-end effects and on_block_end) "" while the battle goes on, else won,
# lost or fled; `end_of_round` is true after the last block. Default (today's rule): over when no ally or no enemy
# that counts is active; won when an ally stands, else fled when one is alive (withdrew), else lost.
func verdict(_round_num: int, _block_num: int, _end_of_round: bool) -> String:
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

# The party units that are not guests (OBJ1 rule 1).
func heroes() -> Array[Combatant]:
	var out: Array[Combatant] = []
	for a: Combatant in allies:
		if not guests.has(a):
			out.append(a)
	return out

func is_guest(c: Combatant) -> bool:
	return c != null and guests.has(c)

# The preset an AI draft of the whole party uses for `c` (OBJ1 rule 2: guests always Balanced).
func party_preset_for(c: Combatant, preset_id: String) -> String:
	return GUEST_PRESET if is_guest(c) else preset_id

# Objective state kept in every BlockSnapshot (OBJ1: hold rounds counted); {} = none.
func objective_state() -> Dictionary:
	return {}

func restore_objective_state(_state: Dictionary) -> void:
	pass

# Encounter-owned battle state kept in every BlockSnapshot (BOSS1: phase fired / due / dropped); {} = none.
func phase_state() -> Dictionary:
	return {}

# Restores phase_state() from a snapshot of `round_num` (and rebuilds anything derived from it).
func restore_phase_state(_state: Dictionary, _round_num: int) -> void:
	pass

# {id, name, actor, line, pause_autoplay, enrage} when `entry` is a phase EVENT entry of this battle, else {}.
func phase_info(_entry: ActionLogEntry) -> Dictionary:
	return {}

# PACE1: true when `c` is enraged in the round being planned (its whole plan is shown; CatalogEncounter).
func is_enraged(_c: Combatant) -> bool:
	return false

# Narration when a round concludes: {"phase": String, "status": String, "log": String}
func round_summary(_round_num: int, _combat_over: bool) -> Dictionary:
	return {}

func advance_label(round_num: int) -> String:
	return "⚔ ADVANCE TO ROUND %d" % (round_num + 1)

# ------------------------------------------------------------------------------
# Art: team-colored tokens & portraits generated by UnitArt
# ------------------------------------------------------------------------------

func team_of(c: Combatant) -> UnitArt.Team:
	if allies.has(c):
		return UnitArt.Team.FRIENDLY
	if enemies.has(c):
		return UnitArt.Team.HOSTILE
	return UnitArt.Team.NEUTRAL

func class_of(c: Combatant) -> String:
	if c == null:
		return "unit"
	return str(unit_classes.get(c, c.name.to_lower()))

func get_token_texture(c: Combatant) -> Texture2D:
	return UnitArt.token(class_of(c), team_of(c)) if c != null else null

func get_sprite_texture(c: Combatant) -> Texture2D:
	return UnitArt.portrait(class_of(c), team_of(c)) if c != null else null


# ------------------------------------------------------------------------------
# Planning & AI: units plan their rounds as PlanEntry lists (see CombatPlan)
# ------------------------------------------------------------------------------

var ai_profiles: Dictionary = {}    # Combatant -> AiProfile

func plan_context() -> PlanContext:
	return PlanContext.create(grid, allies, enemies, ai_profiles, total_blocks)

func profile_of(c: Combatant) -> AiProfile:
	return plan_context().profile_of(c)

# Stable key of a party unit across battles (AUTO1: per-hero autoplay tactics are saved under it). Default: the world
# character it plays, else its name; GameEncounter uses the party member id. Guests have none ("": never saved).
func hero_key(c: Combatant) -> String:
	if c == null or guests.has(c):
		return ""
	return c.lore_ref if not c.lore_ref.is_empty() else c.name

# AI-drafted sequence for c from the current state (preset_id: see PlanPresets).
func draft_steps(c: Combatant, preset_id: String = "balanced", shared: CombatPlan = null) -> Array[PlanStep]:
	var ctx: PlanContext = shared.context if shared != null else plan_context()
	return TacticalAI.build_steps(c, ctx, PlanPresets.profile_for(preset_id, ctx.profile_of(c)), shared)

# Fills the blocks c's draft left empty (TacticalAI.fill_steps); player-facing drafts only.
func fill_steps(c: Combatant, preset_id: String, shared: CombatPlan) -> void:
	var ctx: PlanContext = shared.context
	TacticalAI.fill_steps(c, ctx, PlanPresets.profile_for(preset_id, ctx.profile_of(c)), shared)

# Plans every enemy on the field with the AI and schedules it on the resolver.
func schedule_enemy_ai(resolver: CombatResolver) -> void:
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

# True when the unit's draft does not compile or compiles to nothing.
static func _draft_broken(enemy_plan: CombatPlan, e: Combatant) -> bool:
	if enemy_plan.steps_of(e).is_empty():
		return false
	var st: CombatPlan.SimState = enemy_plan.compile(e)
	return st.error_index >= 0 or st.entries.is_empty()

# Convenience for subclasses: an EVENT log entry for narration / triggers.
static func make_event_entry(round_num: int, block_num: int, actor: String, actor_is_ally: bool, skill: String, subtype: String, outcome: String, detail: String, status: String) -> ActionLogEntry:
	var entry := ActionLogEntry.new(round_num, block_num, "EVENT", actor, "", detail, 0, outcome)
	entry.actor_is_ally = actor_is_ally
	entry.skill_name = skill
	entry.skill_subtype = subtype
	entry.status_text = status
	return entry

# Party slots the combat HUD shows: the explicit cap, else 0 (unknown). World battles override it.
func party_slots() -> int:
	return party_cap
