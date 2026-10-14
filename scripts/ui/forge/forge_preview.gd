class_name ForgePreview
extends RefCounted

# ==============================================================================
# Tactical RPG - Forge Preview (what a catalog entry turns into in the engine)
# Builds the engine object from a draft entry and describes it as BBCode, plus
# an icon / portrait texture. Pure functions: the Forge and tests both use them.
# ==============================================================================

static func texture_for(kind: String, entry: Dictionary) -> Texture2D:
	match kind:
		"items":
			return ItemArt.icon(ContentFactory.item(entry))
		"units":
			var team: UnitArt.Team = UnitArt.Team.FRIENDLY
			match str(entry.get("side", "")):
				"enemy":
					team = UnitArt.Team.HOSTILE
				"neutral":
					team = UnitArt.Team.NEUTRAL
			return UnitArt.portrait(ContentFactory.art_of(entry), team)
	return null

static func describe(kind: String, entry: Dictionary) -> String:
	var e: Dictionary = ContentSchema.normalize(kind, entry)
	var text: String = ""
	match kind:
		"skills":
			text = _skill(e)
		"items":
			text = _item(e)
		"passives":
			text = "[b]%s[/b]
%s

%s
" % [str(e["name"]), Passives.describe(e), str(e["description"])]
		"classes":
			text = _class(e)
		"units":
			text = _unit(e)
		"encounters":
			text = _encounter(e)
	var users: Array[String] = ContentCatalog.references_to(kind, str(e.get("id", "")))
	text += "\n[color=#8ba3b8]Used by:[/color] %s" % (", ".join(users) if not users.is_empty() else "nothing yet")
	return text

static func _line(label: String, value: String) -> String:
	return "[color=#8ba3b8]%s:[/color] %s\n" % [label, value]

static func _skill(e: Dictionary) -> String:
	var s: SkillDef = ContentFactory.skill(e)
	var t: String = "[b]%s[/b]  %s\n" % [s.label(), "[color=#fbbf24](system)[/color]" if s.system else ""]
	t += _line("Cost", "%d AP%s, %d block%s" % [s.ap_cost(), " per tile" if s.kind == SkillDef.Kind.MOVE else "", s.blocks, "" if s.blocks == 1 else "s"])
	t += _line("Stance", CombatTypes.stance_to_string(s.stance()) if s.kind != SkillDef.Kind.MOVE else "keeps the current stance")
	match s.kind:
		SkillDef.Kind.ATTACK:
			var melee := Combatant.new("melee", 50, 12, 1)
			var ranged := Combatant.new("ranged", 50, 12, 3)
			var mb: Vector2i = s.reach_for(melee)
			var rb: Vector2i = s.reach_for(ranged)
			t += _line("Reach", "melee unit %d-%d, ranged unit %d-%d" % [mb.x, mb.y, rb.x, rb.y])
			t += _line("Damage", "attack power x %.2f  (~%d at power 20, ~%d at power 40; then stance, crit, guard, armor)" % [s.power_mult, roundi(20.0 * s.power_mult), roundi(40.0 * s.power_mult)])
			if s.is_area():
				t += _line("Area", "%s: %d tiles, hits allies too" % [SkillDef.Area.keys()[s.area], s.area_cells(Vector2i(4, 6), Vector2i(4, 5)).size()])
			if not s.status_effect.is_empty():
				t += _line("Status", "%s on hit (potency %d%%), next round" % [StatusEffects.display_name(s.status_effect), s.status_potency])
			if s.offhand:
				t += _line("Off-hand", "melee jab for ranged units (only offered when the unit's range is 2+)")
		SkillDef.Kind.DEFEND:
			t += _line("Guard", "-%d%% damage from its first block" % roundi(s.mitigation * 100.0))
		SkillDef.Kind.FOCUS:
			t += _line("Boost", "x%.2f attack for %d rounds" % [s.boost, s.boost_rounds])
		SkillDef.Kind.REST:
			t += _line("Recover", "+%d AP, back to Neutral" % s.recover_ap)
	t += "\n%s\n" % s.description
	return t

static func _item(e: Dictionary) -> String:
	var it: Item = ContentFactory.item(e)
	var col: String = UiTheme.rarity_color(it.rarity).to_html(false)
	var t: String = "[b][color=#%s]%s[/color][/b]\n" % [col, it.name]
	t += _line("Type", "%s, %s" % [Item.type_to_string(it.item_type), str(Item.ItemRarity.keys()[it.rarity]).capitalize()])
	if it is Equipment:
		var eq: Equipment = it
		t += _line("Slot", Equipment.slot_display_name(eq.slot) + (" (two-handed)" if eq.is_two_handed else ""))
		var summary: String = UiTheme.equipment_summary(eq, CharacterStats.new())
		t += _line("Stats", summary if not summary.is_empty() else "-")
		if not eq.required_stats.is_empty():
			t += _line("Requires", str(eq.required_stats))
		for tier: Dictionary in eq.threshold_tiers:
			t += _line("Tier", "%s %s+: %s" % [str(tier.get("stat", "?")), str(tier.get("threshold", "?")), str(tier.get("desc", ""))])
		var users: Array[String] = []
		for c: Dictionary in ContentCatalog.entries("classes"):
			if ContentFactory.class_archetype(c).can_equip_gear(eq):
				users.append(str(c.get("name", "")))
		t += _line("Classes", ", ".join(users) if not users.is_empty() else "none")
	elif it is Consumable:
		var con: Consumable = it
		t += _line("Effect", "%s %s, %d block" % [UiTheme.consumable_icon(con), UiTheme.consumable_effect_text(con), con.block_cost])
	t += _line("Weight / value", "%.1f / %d g" % [it.weight, it.value_gold])
	t += _line("Starter stash", "yes" if DictUtil.to_bool(e.get("starter_stash", false)) else "no")
	t += "\n%s\n" % it.description
	return t

static func _class(e: Dictionary) -> String:
	var arch: ClassArchetype = ContentFactory.class_archetype(e)
	var t: String = "[b]%s[/b]\n" % arch.display_name
	for lvl: int in [1, 5, 10]:
		var st := CharacterStats.new(arch, lvl)
		var parts: Array[String] = []
		for key: String in ContentSchema.STAT_KEYS:
			parts.append("%s %d" % [key, st.get_base_stat(key)])
		t += _line("Level %d" % lvl, "%s  •  HP %d, speed %d" % [" ".join(parts), st.get_max_hp({}), st.get_initiative_speed(0, {})])
	var base_total: int = StatRules.total_of(arch.base_stats)
	t += _line("Budget", "%d / %d (growth %d / %d per level)%s" % [base_total, StatRules.CLASS_BUDGET, StatRules.total_of(arch.growth_per_level), StatRules.GROWTH_TOTAL, "" if base_total == StatRules.CLASS_BUDGET else " [color=#f87171](off budget)[/color]"])
	var prof: Proficiency = arch.make_proficiency()
	var trained: Array[String] = []
	for key: Variant in prof.entries:
		trained.append("%s %d (x%.2f)" % [str(key), prof.level_of(str(key)), prof.scaling_of(str(key))])
	t += _line("Proficiency", ", ".join(trained) if not trained.is_empty() else "none")
	var path_parts: Array[String] = []
	for step: Dictionary in arch.skill_path:
		path_parts.append("Lv%d %s" % [DictUtil.to_int(step.get("level", 1)), str(ContentCatalog.get_entry("skills", str(step.get("skill", ""))).get("name", step.get("skill", "")))])
	t += _line("Skill path", ", ".join(path_parts) if not path_parts.is_empty() else "none")
	var kit: Array[String] = []
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		if arch.loadout.has(slot):
			kit.append(str(ContentCatalog.get_entry("items", str(arch.loadout[slot])).get("name", arch.loadout[slot])))
	t += _line("Default gear", ", ".join(kit) if not kit.is_empty() else "none")
	t += "\n%s\n" % str(e.get("description", ""))
	return t

static func _unit(e: Dictionary) -> String:
	var c: Combatant = ContentFactory.unit(e, "", 12)
	var profile: AiProfile = ContentFactory.ai_profile(e, c)
	var t: String = "[b]%s[/b]  (%s)\n" % [c.name, str(e["side"])]
	if str(e["stat_mode"]) == "derived" and not str(e["class_id"]).is_empty():
		var spend: Dictionary = ContentFactory._ints(e["spend"])
		t += _line("Build", "%s level %d, %s tier (HP x%.1f, power x%.1f), %s" % [str(e["class_id"]).capitalize(), DictUtil.to_int(e["level"], 1), str(e["tier"]), c.hp_scale, c.power_scale, ("spend " + str(spend)) if not spend.is_empty() else "spend follows the class"])
	t += _line("Vitals", "HP %d, speed %d (%d tiles / block), range %d, AP %d" % [c.max_hp, c.speed, c.move_per_block(), c.attack_range, c.max_ap])
	var power_res: Dictionary = DamageCalculator.attack_power(c, c.attack_range > 1)
	var power_parts: String = "authored" if DictUtil.to_bool(power_res["authored"]) else "weapon %.0f + unit %.0f (%s)" % [DictUtil.to_float(power_res["weapon_power"]), DictUtil.to_float(power_res["unit_power"]), str(power_res["stat_name"])]
	t += _line("Attack power", "%d (%s)" % [DictUtil.to_int(power_res["attack_power"]), power_parts])
	var rating: int = DamageCalculator.armor_rating(c)
	t += _line("Defence", "armor %d (-%d%% damage), crit %.0f%% x%.1f" % [rating, roundi(DamageCalculator.armor_reduction(rating) * 100.0), DamageCalculator.crit_chance(c), DamageCalculator.CRIT_MULT])
	t += _line("AI", "%s, targets %s, flees at %d%% HP" % [AiProfile.role_name(profile.role), str(AiProfile.TargetPref.keys()[profile.target_pref]).to_lower(), roundi(profile.flee_ratio * 100.0)])
	var known: Array[String] = []
	for s: SkillDef in SkillCatalog.all():
		if not s.system and s.level <= profile.skill_cap and c.knows_skill(s.id) and not (s.offhand and c.attack_range <= 1):
			known.append(s.display_name)
	t += _line("Skills (cap %d)" % profile.skill_cap, ", ".join(known) if not known.is_empty() else "none")
	var gear: Array[String] = []
	for eq: Equipment in c.inventory._iter_equipped_gear():
		gear.append(eq.name)
	t += _line("Gear", ", ".join(gear) if not gear.is_empty() else "none")
	var pouch: Array[String] = []
	for con: Consumable in c.inventory.battle_pouch:
		pouch.append(con.name)
	if not pouch.is_empty():
		t += _line("Pouch", ", ".join(pouch))
	if not c.status_resist.is_empty():
		t += _line("Resists", str(c.status_resist))
	t += "\n%s\n" % str(e.get("description", ""))
	return t

static func _encounter(e: Dictionary) -> String:
	var t: String = "[b]%s[/b]\n" % str(e["name"])
	t += _line("Zone", "%s (%s arena)" % [CombatZones.zone_name(str(e["zone"])), str(e["arena"]).to_lower()])
	t += _line("Rules", "%d rounds max, %d blocks / round, %d AP (+%d / round)" % [DictUtil.to_int(e["max_rounds"]), DictUtil.to_int(e["total_blocks"]), DictUtil.to_int(e["unit_ap"]), DictUtil.to_int(e["ap_regen"])])
	t += deployment_report(str(e["id"]), e)
	t += "\n%s\n" % str(e.get("description", ""))
	return t

# Builds the encounter (from `entry`, not yet stored) and reports where each unit deploys.
static func deployment_report(encounter_id: String, entry: Dictionary) -> String:
	var probe: Dictionary = entry.duplicate(true)
	probe["id"] = encounter_id
	var enc: CatalogEncounter = CatalogEncounter.from_entry(probe)
	enc.build()
	var t: String = ""
	for side: String in ["party", "enemies"]:
		var units: Array[Combatant] = enc.allies if side == "party" else enc.enemies
		var parts: Array[String] = []
		for c: Combatant in units:
			var placed: bool = enc.grid.has_combatant(c)
			parts.append("%s %s" % [c.name, str(enc.grid.get_position(c)) if placed else "[color=#f87171](no room!)[/color]"])
		t += _line(side.capitalize(), ", ".join(parts) if not parts.is_empty() else "[color=#f87171]none[/color]")
	return t
