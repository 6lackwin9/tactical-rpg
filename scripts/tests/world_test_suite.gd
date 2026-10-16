class_name WorldTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - World Engine & World Forge Test Suite
# The shipped worlds (ashlands, tidewrack extends ashlands, iron_reaches connects
# to ashlands) load and validate; inheritance, namespacing and overrides resolve;
# two worlds can wear the same base units with different lore and play identically
# (except explicit tuning deltas); references follow renames; worlds connect
# without inheriting; authored worlds save and reload; world encounters play to
# the end with their lore names; the World Forge UI and the Scene Runner can open
# a world's battle. Writes go to a sandbox copy (a TestHelpers sandbox).
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("world_test")

const TAG: String = "World"

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING WORLD ENGINE & WORLD FORGE TEST        ===")
	print("=======================================================")
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_shipped_worlds() and ok
	ok = _test_inheritance() and ok
	ok = _test_same_base_different_lore() and ok
	ok = _test_tuning_items_disciplines() and ok
	ok = _test_encounters_play() and ok
	var forked: bool = TestHelpers.fork_worlds(SANDBOX)
	ok = TestHelpers.report_ok(TAG, "Sandbox fork of the shipped worlds", forked and WorldCatalog.world_ids().size() == 3) and ok
	if forked:
		ok = _test_rules_and_connections() and ok
		ok = _test_authoring_and_renames() and ok
		ok = _test_forge_ui_and_runner(tree_parent) and ok
	TestHelpers.cleanup_worlds(SANDBOX)
	return ok

# ------------------------------------------------------------------------------

static func _test_shipped_worlds() -> bool:
	var problems: Dictionary = WorldCatalog.validate_all()
	var ids: Array[String] = WorldCatalog.world_ids()
	var checks: Dictionary = {}
	checks["three worlds"] = ids == ["ashlands", "iron_reaches", "tidewrack"]
	checks["valid"] = problems.is_empty() and WorldCatalog.load_errors().is_empty()
	checks["lineage"] = WorldCatalog.lineage("tidewrack") == ["ashlands", "tidewrack"] and WorldCatalog.lineage("ashlands") == ["ashlands"]
	checks["descendants"] = WorldCatalog.descendants("ashlands") == ["tidewrack"]
	var kinds_filled: bool = true
	for kind: String in ["regions", "locations", "factions", "characters", "items", "disciplines", "quests", "lore", "encounters"]:
		kinds_filled = kinds_filled and not WorldCatalog.own_entries("ashlands", kind).is_empty()
	checks["every kind used by ashlands"] = kinds_filled
	return TestHelpers.report_ok(TAG, "Shipped worlds load and validate (every element kind, lineage, descendants)", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks) if not problems.is_empty() else str(ids))

static func _test_inheritance() -> bool:
	var checks: Dictionary = {}
	# The child sees the parent's entries, namespaced.
	var names: Array[String] = []
	for e: Dictionary in WorldCatalog.merged_entries("tidewrack", "regions"):
		names.append(str(e["_qid"]))
	checks["inherits regions"] = names.has("ashlands:ashen_marches") and names.has("tidewrack:tidewrack_coast")
	checks["parent unchanged by child"] = WorldCatalog.merged_entries("ashlands", "regions").size() == 2 and not WorldCatalog.has_entry("ashlands", "regions", "tidewrack_coast")
	# Overrides replace the ancestor's entry for the child (and refs follow), not for the parent.
	var merged_chars: Array[String] = []
	for e: Dictionary in WorldCatalog.merged_entries("tidewrack", "characters"):
		merged_chars.append(str(e["_qid"]))
	checks["override replaces"] = not merged_chars.has("ashlands:skarn_ironjaw") and merged_chars.has("tidewrack:brannoch_saltfist")
	checks["parent keeps original"] = WorldCatalog.has_entry("ashlands", "characters", "skarn_ironjaw") and str(WorldCatalog.resolve("ashlands", "characters", "skarn_ironjaw").get("name", "")) == "Skarn Ironjaw"
	checks["old id redirects"] = str(WorldCatalog.resolve("tidewrack", "characters", "skarn_ironjaw").get("name", "")) == "Brannoch Saltfist" and str(WorldCatalog.resolve("tidewrack", "characters", "ashlands:skarn_ironjaw").get("_qid", "")) == "tidewrack:brannoch_saltfist"
	checks["faction override"] = str(WorldCatalog.resolve("tidewrack", "factions", "ashen_horde").get("name", "")) == "Reef Raiders"
	# Unqualified refs resolve nearest-first; qualified refs are explicit.
	checks["nearest wins"] = str(WorldCatalog.resolve("tidewrack", "locations", "warden_keep").get("_qid", "")) == "ashlands:warden_keep"
	checks["ref value"] = WorldCatalog.ref_value("tidewrack", WorldCatalog.resolve("tidewrack", "locations", "warden_keep")) == "ashlands:warden_keep" and WorldCatalog.ref_value("ashlands", WorldCatalog.resolve("ashlands", "locations", "warden_keep")) == "warden_keep"
	# Cycles and unknown parents are rejected.
	var errs_cycle: Array[String] = WorldCatalog.update_manifest("ashlands", {"extends": ["tidewrack"]})
	checks["cycle rejected"] = not errs_cycle.is_empty() and WorldCatalog.lineage("ashlands") == ["ashlands"]
	checks["unknown parent rejected"] = not WorldCatalog.update_manifest("ashlands", {"extends": ["nowhere"]}).is_empty()
	return TestHelpers.report_ok(TAG, "Inheritance: namespaced entries, overrides replace for the child only, old ids redirect, cycles rejected", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks))

# Same base units in two worlds, different lore: they play the same (tuning deltas aside).
static func _test_same_base_different_lore() -> bool:
	var ash := WorldEncounter.new("ashlands", "courtyard_skirmish")
	var tide := WorldEncounter.new("tidewrack", "courtyard_skirmish")
	ash.build()
	tide.build()
	var checks: Dictionary = {}
	checks["same roster size"] = ash.allies.size() == tide.allies.size() and ash.enemies.size() == tide.enemies.size() and ash.enemies.size() == 5
	var names_differ: bool = true
	var stats_same: bool = true
	var detail: Array[String] = []
	var all_ash: Array[Combatant] = ash.allies + ash.enemies
	var all_tide: Array[Combatant] = tide.allies + tide.enemies
	for i: int in range(mini(all_ash.size(), all_tide.size())):
		var a: Combatant = all_ash[i]
		var b: Combatant = all_tide[i]
		names_differ = names_differ and a.name != b.name
		var same: bool = (a.max_hp == b.max_hp and a.speed == b.speed and a.attack_range == b.attack_range and a.max_ap == b.max_ap
			and DamageCalculator.power_of(a, a.attack_range > 1) == DamageCalculator.power_of(b, b.attack_range > 1)
			and DamageCalculator.armor_rating(a) == DamageCalculator.armor_rating(b) and ash.class_of(a) == tide.class_of(b)
			and a.known_skills == b.known_skills)
		if not same:
			detail.append("%s/%s" % [a.name, b.name])
		stats_same = stats_same and (same or b.lore_ref == "tidewrack:captain_dredge")   # the tuned boss differs on purpose
	checks["lore names differ"] = names_differ
	checks["gameplay identical except the tuned boss"] = stats_same
	# Behaviour is the base unit's: identical damage and the same AI plan.
	var skarn: Combatant = ash.unit("skarn_ironjaw")
	var brannoch: Combatant = tide.unit("skarn_ironjaw")   # tidewrack's override answers to the old id
	var dummy := Combatant.new("Dummy", 500, 10, 1)
	dummy.armor_override = 0
	var hit_a: int = DictUtil.to_int(DamageCalculator.compute(skarn, dummy, false, false)["final_dmg"])
	var hit_b: int = DictUtil.to_int(DamageCalculator.compute(brannoch, dummy, false, false)["final_dmg"])
	checks["same damage"] = hit_a == hit_b and hit_a > 0 and skarn != null and brannoch != null and brannoch.name == "Brannoch Saltfist"
	checks["both link back to their lore"] = skarn.lore_ref == "ashlands:skarn_ironjaw" and brannoch.lore_ref == "tidewrack:brannoch_saltfist"
	var plans_same: bool = true
	for pair: Array in [[ash, skarn], [tide, brannoch]]:
		var enc: WorldEncounter = pair[0]
		var unit: Combatant = pair[1]
		var plan := CombatPlan.new(enc.plan_context())
		var steps: Array[PlanStep] = enc.draft_steps(unit, "balanced", plan)
		pair.append(steps.size())
	plans_same = (ash.draft_steps(skarn).size() == tide.draft_steps(brannoch).size())
	checks["same AI plan length"] = plans_same
	return TestHelpers.report_ok(TAG, "Same base, different lore: names differ, every stat/class/skill/AI matches (only the tuned boss differs)", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks) if not TestHelpers.failed(checks).is_empty() else "")

static func _test_tuning_items_disciplines() -> bool:
	var checks: Dictionary = {}
	var tide := WorldEncounter.new("tidewrack", "courtyard_skirmish")
	tide.build()
	var ash := WorldEncounter.new("ashlands", "courtyard_skirmish")
	ash.build()
	# Tuning deltas vs the base goblin berserker (Dredge is tidewrack's override of Bolg).
	var dredge: Combatant = tide.unit("bolg_redhand")
	var base_unit: Combatant = ContentFactory.unit(ContentCatalog.get_entry("units", "goblin_berserker"), "", 12)
	checks["dredge hp +20%"] = dredge != null and dredge.max_hp == roundi(float(base_unit.max_hp) * 1.2) and dredge.hp == dredge.max_hp
	checks["dredge power +2"] = DamageCalculator.power_of(dredge) == DamageCalculator.power_of(base_unit) + 2
	checks["dredge armor +1"] = DamageCalculator.armor_rating(dredge) == DamageCalculator.armor_rating(base_unit) + 1
	var bolg: Combatant = ash.unit("bolg_redhand")
	checks["untuned twin identical"] = bolg.max_hp == base_unit.max_hp and DamageCalculator.power_of(bolg) == DamageCalculator.power_of(base_unit)
	var info: String = WorldFactory.divergence(WorldCatalog.resolve("tidewrack", "characters", "captain_dredge"), "tidewrack")
	checks["divergence text"] = info.contains("hp +20%") and info.contains("power +2") and WorldFactory.divergence(WorldCatalog.resolve("ashlands", "characters", "skarn_ironjaw"), "ashlands").is_empty()
	# Named item: the base brigandine + 2 mitigation, worn by Mara; vitals untouched (fixed unit).
	var mara: Combatant = ash.unit("mara_voss")
	var plain: Combatant = ContentFactory.unit(ContentCatalog.get_entry("units", "warrior"), "", 12)
	var worn: Equipment = mara.inventory.get_equipped(Equipment.EquipSlot.BODY)
	checks["named item worn"] = worn != null and worn.name == "Ember-Scale Brigandine" and worn.mitigation == ItemCatalog.create_equipment("arm_leather").mitigation + 2
	checks["named item armor"] = DamageCalculator.armor_rating(mara) == mara.stats.get_mitigation_rating(mara.inventory.get_total_flat_mitigation(), mara.get_weapon_context()) and mara.max_hp == plain.max_hp
	# Disciplines and factions grant base passives (traits from the world, never part of the class).
	var tev: Combatant = ash.unit("tev_quill")
	checks["discipline rank"] = tev.passives.has("keen_senses") and mara.passives.has("steady_nerves") and DictUtil.int_of(mara.status_resist, "stagger") >= 25
	# Illegal tuning is rejected.
	var bad: Array[String] = WorldCatalog.blocking_errors("ashlands", "characters", {"id": "x", "name": "X", "tuning": {"mana": 5}})
	checks["unknown tuning rejected"] = not bad.is_empty() and bad[0].contains("Unknown")
	return TestHelpers.report_ok(TAG, "Tuning deltas, named items and disciplines apply exactly as described (and unknown tuning is refused)", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks))

static func _resolve(session: CombatSession) -> void:
	for i: int in range(600):
		if session.phase == CombatSession.Phase.REPLAY:
			return
		session.step_next()

# World encounters (inherited ones too) play a full battle in the real engine with lore names.
static func _test_encounters_play() -> bool:
	var checks: Dictionary = {}
	for id: String in ["ashlands:pass_ambush", "tidewrack:strand_clash", "tidewrack:courtyard_skirmish"]:
		var enc: CombatEncounter = EncounterSource.create(id)
		var session := CombatSession.new(enc)
		session.start()
		var rounds: int = 0
		for r: int in range(enc.max_rounds):
			session.plan_apply_preset_all("balanced")
			if not session.confirm_plan():
				break
			_resolve(session)
			rounds += 1
			if session.combat_over or not session.can_advance_round():
				break
			session.advance_round()
		var lore_names: bool = false
		var wave: bool = id != "ashlands:pass_ambush"
		var party_name: String = session.allies[0].name
		for e: ActionLogEntry in session.log_entries:
			if e.actor_name == party_name and e.category == "ATTACK":
				lore_names = true
			if e.outcome_tag == "ARRIVED":
				wave = true
		var any_damage: bool = false
		for c: Combatant in session.allies + session.enemies:
			any_damage = any_damage or c.hp < c.max_hp or not c.is_active()
		checks[id] = rounds > 0 and lore_names and wave and any_damage
		session.dispose()
	var tide_party: String = (EncounterSource.create("tidewrack:courtyard_skirmish") as WorldEncounter).definition["party"][0]
	checks["inherited battle uses the child's characters"] = tide_party == "mara_voss"
	var opts: Array[String] = []
	for opt: Dictionary in EncounterSource.options():
		opts.append(str(opt["id"]))
	checks["encounter source lists both layers"] = opts.has("goblin_skirmish") and opts.has("ashlands:pass_ambush") and opts.has("tidewrack:courtyard_skirmish") and opts.has("tidewrack:strand_clash")
	return TestHelpers.report_ok(TAG, "World encounters play to the end in the real engine with lore names (wave included, inherited battles re-skinned)", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks))

# ------------------------------------------------------------------------------
# Sandbox: rules, connections, authoring
# ------------------------------------------------------------------------------

static func _test_rules_and_connections() -> bool:
	var checks: Dictionary = {}
	# Connected worlds: iron_reaches' exit into ashlands resolves because it connects, not inherits.
	var gate: Dictionary = WorldCatalog.resolve("iron_reaches", "locations", "slag_gate")
	var exit_ref: String = str(DictUtil.dict_of(DictUtil.array_of(gate["exits"])[0])["to"])
	checks["cross-world exit resolves"] = exit_ref == "ashlands:warden_keep" and str(WorldCatalog.resolve("iron_reaches", "locations", exit_ref).get("name", "")) == "Warden Keep"
	checks["connect is not inherit"] = not WorldCatalog.lineage("iron_reaches").has("ashlands") and WorldCatalog.can_reach_world("iron_reaches", "ashlands") and not WorldCatalog.has_entry("iron_reaches", "locations", "warden_keep")
	# A qualified ref to a world that is neither an ancestor nor connected is a problem.
	var bad_world: Array[String] = WorldCatalog.validate_entry("tidewrack", "locations", {"id": "stray", "name": "Stray", "exits": [{"to": "iron_reaches:slag_gate", "label": "x"}]})
	checks["unconnected world refused"] = _has(bad_world, "neither an ancestor nor connected")
	WorldCatalog.update_manifest("tidewrack", {"connects": ["iron_reaches"]})
	checks["connecting makes it legal"] = not _has(WorldCatalog.validate_entry("tidewrack", "locations", {"id": "stray", "name": "Stray", "exits": [{"to": "iron_reaches:slag_gate", "label": "x"}]}), "neither an ancestor")
	WorldCatalog.update_manifest("tidewrack", {"connects": []})
	# Reference checks: base assets, roles, slots, overrides.
	checks["unknown base unit"] = _has(WorldCatalog.validate_entry("ashlands", "characters", {"id": "x", "name": "X", "base_unit": "nope"}), "unknown base unit")
	checks["role filter"] = _has(WorldCatalog.validate_entry("ashlands", "encounters", {"id": "e", "name": "E", "party": ["skarn_ironjaw"], "enemies": ["mara_voss"]}), "can't be used here")
	checks["party limit"] = not WorldCatalog.blocking_errors("ashlands", "encounters", {"id": "e", "name": "E", "party": ["a", "b", "c", "d", "e"]}).is_empty()
	checks["override must target an ancestor"] = _has(WorldCatalog.blocking_errors("ashlands", "characters", {"id": "x", "name": "X", "override": "ashlands:mara_voss"}), "ancestor") and _has(WorldCatalog.blocking_errors("tidewrack", "characters", {"id": "x", "name": "X", "override": "ashlands:ghost"}), "no character")
	var slot_clash: Array[String] = WorldCatalog.validate_entry("ashlands", "characters", {"id": "x", "name": "X", "base_unit": "warrior", "named_items": ["ember_brigandine", "ember_brigandine"]})
	checks["duplicate ref and slot"] = _has(slot_clash, "twice")
	# In-use entries can't be deleted.
	checks["in-use delete refused"] = WorldCatalog.remove("ashlands", "characters", "mara_voss").contains("still used") and WorldCatalog.remove_world("ashlands").contains("still extended")
	return TestHelpers.report_ok(TAG, "Connections and rules: connect without inheriting, unconnected worlds refused, role/base/override/slot checks, safe deletes", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks))

static func _has(errors: Array[String], fragment: String) -> bool:
	for e: String in errors:
		if e.contains(fragment):
			return true
	return false

static func _test_authoring_and_renames() -> bool:
	var checks: Dictionary = {}
	var errs: Array[String] = WorldCatalog.create_world("sandbox_isle", "Sandbox Isle", ["tidewrack"])
	checks["create world"] = errs.is_empty() and WorldCatalog.lineage("sandbox_isle") == ["ashlands", "tidewrack", "sandbox_isle"]
	# Blocking vs unresolved: a location may exit to one that doesn't exist yet.
	var loc_errs: Array[String] = WorldCatalog.upsert("sandbox_isle", "locations", {"id": "isle_dock", "name": "Isle Dock", "kind": "town", "exits": [{"to": "isle_cave", "label": "path"}]})
	checks["dangling ref allowed while authoring"] = loc_errs.is_empty() and _has(WorldCatalog.validate_entry("sandbox_isle", "locations", WorldCatalog.get_own("sandbox_isle", "locations", "isle_dock"), "isle_dock"), "unknown location")
	WorldCatalog.upsert("sandbox_isle", "locations", {"id": "isle_cave", "name": "Isle Cave", "kind": "dungeon", "exits": [{"to": "isle_dock", "label": "back"}]})
	checks["resolved once both exist"] = WorldCatalog.validate_world("sandbox_isle").is_empty()
	# Override an inherited character: replaces it here, the original stays in the parent.
	var new_id: String = WorldCatalog.override_entry("sandbox_isle", "characters", "tidewrack:ilse_marrow")
	checks["override copy"] = not new_id.is_empty() and str(WorldCatalog.get_own("sandbox_isle", "characters", new_id).get("override", "")) == "tidewrack:ilse_marrow"
	var ilse: Dictionary = WorldCatalog.get_own("sandbox_isle", "characters", new_id)
	ilse["name"] = "Ilse of the Isle"
	ilse["background"] = "Exiled lampwarden."
	checks["edit override lore"] = WorldCatalog.upsert("sandbox_isle", "characters", ilse, new_id).is_empty()
	var iso := WorldEncounter.new("sandbox_isle", "courtyard_skirmish")
	iso.build()
	checks["child of a child plays re-skinned"] = iso.allies[0].name == "Ilse of the Isle" and iso.allies[1].name == "Pell Driftwood" and iso.enemies[0].name == "Brannoch Saltfist"
	checks["tidewrack unchanged"] = str(WorldCatalog.resolve("tidewrack", "characters", "ilse_marrow").get("name", "")) == "Ilse Marrow"
	# Renaming ids rewrites references (plain refs, json refs, descendants' refs, override pointers).
	var rename: Dictionary = WorldCatalog.get_own("ashlands", "locations", "warden_keep")
	rename["id"] = "warden_fort"
	checks["rename ok"] = WorldCatalog.upsert("ashlands", "locations", rename, "warden_keep").is_empty()
	var mara: Dictionary = WorldCatalog.get_own("ashlands", "characters", "mara_voss")
	var quest: Dictionary = WorldCatalog.get_own("ashlands", "quests", "hold_the_courtyard")
	var exit_back: Dictionary = DictUtil.dict_of(DictUtil.array_of(WorldCatalog.get_own("ashlands", "locations", "ruined_courtyard")["exits"])[0])
	var tide_ilse: Dictionary = WorldCatalog.get_own("tidewrack", "characters", "ilse_marrow")
	var gate_exit: Dictionary = DictUtil.dict_of(DictUtil.array_of(WorldCatalog.get_own("iron_reaches", "locations", "slag_gate")["exits"])[0])
	checks["plain ref rewritten"] = str(mara["home"]) == "warden_fort"
	checks["json refs rewritten"] = str(exit_back["to"]) == "warden_fort" and str(DictUtil.dict_of(DictUtil.array_of(quest["steps"])[0])["location"]) == "warden_fort"
	checks["descendant refs follow"] = str(tide_ilse["home"]) == "gull_harbour" and WorldCatalog.validate_world("tidewrack").is_empty()
	checks["cross-world ref rewritten"] = str(gate_exit["to"]) == "ashlands:warden_fort" and WorldCatalog.validate_all().is_empty()
	var rename_char: Dictionary = WorldCatalog.get_own("ashlands", "characters", "skarn_ironjaw")
	rename_char["id"] = "skarn_the_jaw"
	checks["override pointer follows rename"] = WorldCatalog.upsert("ashlands", "characters", rename_char, "skarn_ironjaw").is_empty() and str(WorldCatalog.get_own("tidewrack", "characters", "brannoch_saltfist")["override"]) == "ashlands:skarn_the_jaw" and WorldCatalog.validate_all().is_empty()
	# Duplicate as a template, save, reload from disk.
	var copy: String = WorldCatalog.duplicate_entry("ashlands", "characters", "vex_cutpurse")
	checks["duplicate template"] = copy == "vex_cutpurse_copy" and str(WorldCatalog.get_own("ashlands", "characters", copy)["base_unit"]) == "goblin_skirmisher"
	var saved: bool = WorldCatalog.save_all() == OK and FileAccess.file_exists(WorldCatalog.path_for("sandbox_isle", "characters"))
	WorldCatalog.reload_from_disk()
	checks["save and reload"] = saved and WorldCatalog.has_world("sandbox_isle") and WorldCatalog.lineage("sandbox_isle").size() == 3 and WorldCatalog.validate_all().is_empty() and str(WorldCatalog.resolve("sandbox_isle", "characters", "ilse_marrow").get("name", "")) == "Ilse of the Isle"
	return TestHelpers.report_ok(TAG, "Authoring: new world, dangling refs while drafting, overrides, renames rewrite every reference, templates, save & reload", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks))

# ------------------------------------------------------------------------------
# World Forge UI & Scene Runner
# ------------------------------------------------------------------------------

static func _test_forge_ui_and_runner(tree_parent: Node) -> bool:
	if tree_parent == null:
		return TestHelpers.report_ok(TAG, "World Forge UI & runner (need a scene tree)", true, "skipped")
	var host: Node = (load("res://scenes/world_forge.tscn") as PackedScene).instantiate()
	tree_parent.add_child(host)
	var forge: WorldForge = host.get("forge")
	var checks: Dictionary = {}
	checks["opens on a world"] = not forge.world_id.is_empty() and forge.kind == WorldForge.OVERVIEW
	# New world extending another, from the UI.
	checks["create world"] = forge.create_world("wide_world", "Wide World", ["tidewrack"]) and forge.world_id == "wide_world"
	# Inherited entries are read-only; Override copies one in; the copy is editable.
	forge.show_kind("characters")
	forge.select("ashlands:tev_quill")
	checks["inherited is read-only"] = not forge.selected_inherited.is_empty() and not forge.form.visible and forge.selected_id.is_empty()
	forge.select("pell_driftwood")
	forge.override_selected()
	checks["override from the forge"] = forge.selected_id.begins_with("pell_driftwood") and str(forge.form.draft.get("override", "")) == "tidewrack:pell_driftwood"
	forge.form.set_value("name", "Wide-World Pell")
	checks["edit and apply"] = forge.apply() and str(WorldCatalog.get_own("wide_world", "characters", forge.selected_id).get("name", "")) == "Wide-World Pell"
	# A new character with a base unit, tuning and an unresolved reference: stored, problems listed.
	forge.new_entry()
	forge.form.set_value("id", "wide_hero")
	forge.form.set_value("name", "Wide Hero")
	forge.form.set_value("role", "party")
	forge.form.set_value("base_unit", "warrior")
	forge.form.set_value("home", "no_such_place")
	checks["unresolved ref doesn't block apply"] = forge.apply() and not forge.draft_problems().is_empty()
	forge.form.set_value("home", "")
	forge.form.set_value("tuning", {"hp_pct": 10})
	checks["fixed up"] = forge.apply() and forge.draft_problems().is_empty()
	forge.form.set_value("tuning", {"mana": 3})
	checks["blocking error refused"] = not forge.apply()
	forge.revert()
	# An encounter for the new world, played in the arena.
	forge.show_kind("encounters")
	forge.new_entry()
	forge.form.set_value("id", "wide_fight")
	forge.form.set_value("name", "Wide Fight")
	forge.form.set_value("party", ["wide_hero", "pell_driftwood"])
	forge.form.set_value("enemies", ["brannoch_saltfist"])
	checks["encounter applied"] = forge.apply() and forge.draft_problems().is_empty()
	forge.playtest()
	var arena: Node = host.get("arena")
	var arena_session: CombatSession = arena.get("session") if arena != null else null
	checks["playtest runs the arena"] = host.call("is_playtesting") and arena_session != null and arena_session.allies[0].name == "Wide Hero" and arena_session.allies[1].name == "Wide-World Pell"
	host.call("close_playtest")
	# Unsaved guard and save.
	checks["unsaved flagged"] = DictUtil.to_bool(host.call("has_unsaved_changes"))
	checks["saved"] = forge.save_all() and not WorldCatalog.is_dirty()
	forge.delete_world()
	forge.delete_world()
	checks["world deleted"] = not WorldCatalog.has_world("wide_world")
	host.queue_free()
	# Scene Runner: the arena picker offers world battles, and launching one plays that world's characters.
	var runner: SceneRunner = (load("res://scenes/scene_runner.tscn") as PackedScene).instantiate() as SceneRunner
	tree_parent.add_child(runner)
	var launched: bool = runner.launch("res://scenes/combat_arena.tscn", "tidewrack:strand_clash")
	var run_session: CombatSession = runner.active_scene.get("session") if runner.active_scene != null else null
	checks["runner launches a world battle"] = launched and run_session != null and run_session.encounter.encounter_name == "Clash at Wreck Strand" and run_session.allies[0].name == "Ilse Marrow"
	runner.back_to_menu()
	runner.queue_free()
	return TestHelpers.report_ok(TAG, "World Forge UI: create world, inherited read-only, override, drafts with dangling refs, tuning, play in the arena, save, delete; runner launches world battles", TestHelpers.failed(checks).is_empty(), TestHelpers.failed(checks))
