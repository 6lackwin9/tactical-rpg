---
class: SkillShowcase
extends: VBoxContainer
description: "Tactical RPG - Skill showcase (FORGE-VIEW): the Content Forge skill preview, i.e. how one rank of a skill looks in the game's own UI elements. A rank bar (◀ I II III ▶) picks the rank; every element below follows it: - Action menu: the real PlanActionPopup (embedded) listing the PlanOption the real CombatPlan.probe builds for the skill in a small sandbox fight (Hero, an Ally, a Foe; fixed power, no armor), titled like the arena popup (CombatGridPanel.popup_title). - Row tooltip: the hover text of that row (PlanOption.cost_text) in the theme's tooltip panel. - Resolution log: the real ActionLogDockUI cell for the CombatLogBuilder entry of the sandbox round (the skill is planned, committed and resolved by CombatResolver), and the resolver's feed lines (format_resolver_message). - Skills screen: the LoadoutView family line (or the basics line for basic and engine skills). - Engine summary: ForgePreview of the resolved rank row. simulate() is the headless half (tests call it); the control only renders it."
---

# Properties
* `SANDBOX_POWER`: int = 20 (const)
* `ROMAN`: Array[String] = ["I", "II", "III", "IV", "V"] (const)
* `entry`: Dictionary = {}
* `rank`: int = 1
* `_rank_label`: Label = null
* `_body`: VBoxContainer = null
* `out`: Array[String] = []

# Methods

### `_init() -> void`
```logic
name = "SkillShowcase"
add_theme_constant_override("separation", 6)
size_flags_horizontal = Control.SIZE_EXPAND_FILL
```

### `show_entry(p_entry: Dictionary, p_rank: int = 1) -> void`
```logic
entry = p_entry
rank = clampi(p_rank, 1, rank_count())
_rebuild()
```

### `rank_count() -> int`
```logic
return maxi(1, SkillRanks.rank_count(entry)) if not entry.is_empty() else 1
```

### `set_rank(r: int) -> void`
```logic
rank = clampi(r, 1, rank_count())
_rebuild()
```

### `cycle(step: int) -> void`
```logic
var n: int = rank_count()
set_rank(((rank - 1 + step) % n + n) % n + 1)
```

### `static roman(r: int) -> String`
```logic
return ROMAN[r - 1] if r >= 1 and r <= ROMAN.size() else str(r)
```

### `static simulate(row: Dictionary) -> Dictionary`
```logic
var skill: SkillDef = ContentFactory.skill(row)
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
var ranged: bool = skill.weapon_types.any(func(t: String) -> bool: return Equipment.ALWAYS_RANGED_TYPES.has(t))
var hero: Combatant = _unit("Hero", 3 if ranged else 1)
var ally: Combatant = _unit("Ally", 1)
ally.hp = ally.max_hp / 2   # a heal has something to restore
var foe: Combatant = _unit("Foe", 1)
hero.learn_skill(skill.id, false)
var reach: Vector2i = skill.support_reach(hero) if skill.kind == SkillDef.Kind.SUPPORT else CombatPlan.reach(hero, skill)
var dist: int = clampi(reach.y, 1, 5)
if skill.kind == SkillDef.Kind.DISENGAGE or skill.area == SkillDef.Area.SWEEP:
	dist = 1   # a jump starts next to the foe; a sweep is aimed at a diagonal neighbour
elif skill.dash_tiles > 0:
	dist = clampi(reach.y + skill.dash_tiles, 2, 7)   # a gap-close needs a gap
grid.place_combatant(hero, Vector2i(3, 4))
grid.place_combatant(ally, Vector2i(3, 3))
grid.place_combatant(foe, Vector2i(3 + dist, 4))
var allies: Array[Combatant] = [hero, ally]
var enemies: Array[Combatant] = [foe]
if skill.is_area():
	var foe_2: Combatant = _unit("Foe 2", 1)
	grid.place_combatant(foe_2, Vector2i(3 + dist, 5))
	enemies.append(foe_2)
var profiles: Dictionary = {}
for c: Combatant in allies + enemies:
	profiles[c] = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.0, maxi(3, skill.level))
var ctx: PlanContext = PlanContext.create(grid, allies, enemies, profiles)
var target: Combatant = hero
var step: PlanStep = PlanStep.self_skill(skill)
var title: String = ""
match skill.kind:
	SkillDef.Kind.ATTACK:
		target = foe
		var aim: Vector2i = Vector2i(3 + dist, 5) if skill.area == SkillDef.Area.SWEEP else grid.get_position(foe)
		step = PlanStep.area_at(skill, aim) if skill.is_area() else PlanStep.engage(skill, foe)
	SkillDef.Kind.DISENGAGE:
		target = foe
		step = PlanStep.engage(skill, foe)
	SkillDef.Kind.SUPPORT:
		if skill.targets_ally():
			target = ally
			step = PlanStep.ally(skill, ally)
	SkillDef.Kind.MOVE:
		var cell: Vector2i = Vector2i(4, 5)
		step = PlanStep.go_to(skill, cell)
		title = "Tile %s" % str(cell)
if title.is_empty():
	title = CombatGridPanel.popup_title(target, target == hero)
var plan := CombatPlan.new(ctx)
var option: PlanOption = plan.probe(hero, step)
var out: Dictionary = {"skill": skill, "title": title, "option": option, "log": null, "feed": [] as Array[String], "caster": hero}
if not option.valid:
	return out
plan.add_step(hero, step)
var resolver := CombatResolver.new()
resolver.grid = grid
resolver.total_blocks = ctx.total_blocks
for c: Combatant in allies + enemies:
	resolver.register_combatant(c)
var feed: Array[String] = out["feed"]
resolver.log_emitted.connect(func(msg: String) -> void: feed.append(msg))
plan.commit(resolver)
var actions: Array[CombatResolver.ScheduledAction] = resolver.scheduled_actions.duplicate()
resolver.resolve_round()
var shown: CombatResolver.ScheduledAction = null
for act: CombatResolver.ScheduledAction in actions:
	if act.combatant == hero and (skill.kind == SkillDef.Kind.MOVE or not (act.command is WalkCommand)):
		shown = act
if shown != null:
	out["log"] = CombatLogBuilder.from_action(shown, 1, shown.start_block, allies, grid)
return out
```

### `static _unit(unit_name: String, attack_range: int) -> Combatant`
```logic
var c := Combatant.new(unit_name, 160, 10, attack_range)
c.power_override = SANDBOX_POWER
c.armor_override = 0
return c
```

### `static loadout_line(p_entry: Dictionary, shown_rank: int) -> String`
```logic
var base: Dictionary = SkillRanks.effective(p_entry, 1)
if SkillFilter.view_grade(p_entry) == SkillFilter.BASIC_GRADE:
	var basic_names: Array[String] = [str(base["name"])]
	return LoadoutView.basics_line(basic_names)
var start: int = StatRules.ADVANCED_MIN_LEVEL if str(base.get("grade", "novice")) != "novice" else 1
var tiers: Array = []
for r: int in range(1, SkillRanks.rank_count(p_entry) + 1):
	var tier: SkillDef = ContentFactory.skill(SkillRanks.resolve(p_entry, r))
	tiers.append({"tier": r, "castable": r <= shown_rank, "unlock_level": mini(StatRules.MAX_LEVEL, start + tier.tier_offset())})
return LoadoutView.family_line({"name": str(base["name"]), "grade": str(base.get("grade", "novice")).capitalize(), "tiers": tiers})
```

### `_rebuild() -> void`
```logic
UiTheme.clear_children(self)
if entry.is_empty():
	return
add_child(_rank_bar())
var row: Dictionary = SkillRanks.resolve(entry, rank)
var sim: Dictionary = simulate(row)
var option: PlanOption = sim["option"]
_body = VBoxContainer.new()
_body.name = "Body"
_body.add_theme_constant_override("separation", 6)
add_child(_body)

_section("Combat: action menu", "The popup that opens when the hero clicks the target (sandbox: power %d, no armor)." % SANDBOX_POWER)
var popup := PlanActionPopup.new()
popup.name = "ActionMenu"
popup.embed()
var options: Array[PlanOption] = [option]
popup.open_for(str(sim["title"]), options, Vector2.ZERO)
popup.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
_body.add_child(popup)

_section("Combat: row tooltip", "Shown when hovering the row.")
_body.add_child(_tooltip(option.cost_text() if option.valid else option.reason))

_section("Combat: resolution log", "The log cell and feed lines of the resolved round.")
var dock := ActionLogDockUI.new()
dock.name = "LogDock"
dock.size_flags_horizontal = Control.SIZE_EXPAND_FILL
dock.custom_minimum_size.x = 0
var entries: Array[ActionLogEntry] = []
var log_entry: Variant = sim["log"]
if log_entry is ActionLogEntry:
	entries.append(log_entry as ActionLogEntry)
dock.update_active_cell(entries, 0 if not entries.is_empty() else -1)
_body.add_child(dock)
var feed_lines: Array[String] = []
for msg: String in DictUtil.array_of(sim["feed"]):
	if not msg.contains("] Status:"):   # the per-unit status dump of the round end
		feed_lines.append(CombatLogBuilder.format_resolver_message(msg))
if not feed_lines.is_empty():
	var feed: RichTextLabel = _rich("\n".join(feed_lines), UiTheme.FONT_SM)
	feed.name = "LogFeed"
	_body.add_child(feed)

_section("Skills screen", "The family row of the loadout screen at this rank.")
var line: Label = MainMenuView.text(loadout_line(entry, rank), UiTheme.COL_TEXT)
line.name = "LoadoutLine"
line.add_theme_font_size_override("font_size", UiTheme.FONT_MD)
_body.add_child(line)

_section("Engine summary", "")
var text: String = ForgePreview.describe("skills", row)
var cut: int = text.rfind("\n[color=#8ba3b8]Used by:")
if cut >= 0:   # references point at the family head, not at the rank row
	var users: Array[String] = ContentCatalog.references_to("skills", str(entry.get("id", "")))
	text = text.substr(0, cut) + "\n[color=#8ba3b8]Used by:[/color] %s" % (", ".join(users) if not users.is_empty() else "nothing yet")
var summary: RichTextLabel = _rich(text, UiTheme.FONT_SM)
summary.name = "Summary"
_body.add_child(summary)
```

### `_rank_bar() -> Control`
```logic
var bar := HBoxContainer.new()
bar.name = "RankBar"
bar.add_theme_constant_override("separation", 4)
var n: int = rank_count()
var prev: Button = UiTheme.button("◀", "Previous rank", func() -> void: cycle(-1), UiTheme.FONT_MD)
prev.name = "PrevRank"
prev.disabled = n < 2
bar.add_child(prev)
for r: int in range(1, n + 1):
	var pick: int = r
	var btn: Button = UiTheme.button("Rank %s" % roman(r), "Show rank %s" % roman(r), func() -> void: set_rank(pick), UiTheme.FONT_MD)
	btn.name = "Rank_%d" % r
	btn.toggle_mode = true
	btn.set_pressed_no_signal(r == rank)
	bar.add_child(btn)
var next: Button = UiTheme.button("▶", "Next rank", func() -> void: cycle(1), UiTheme.FONT_MD)
next.name = "NextRank"
next.disabled = n < 2
bar.add_child(next)
_rank_label = UiTheme.label(str(SkillRanks.resolve(entry, rank)["name"]), UiTheme.FONT_LG, UiTheme.COL_TITLE)
_rank_label.name = "RankName"
bar.add_child(_rank_label)
return bar
```

### `_section(title: String, hint: String) -> void`
```logic
var head: Label = UiTheme.header(title.to_upper())
if not hint.is_empty():
	head.tooltip_text = hint
	head.mouse_filter = Control.MOUSE_FILTER_STOP
_body.add_child(head)
```

### `_tooltip(text: String) -> Control`
```logic
var frame := PanelContainer.new()
frame.name = "RowTooltip"
frame.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
var theme_src: Theme = ThemeDB.get_project_theme() if ThemeDB.get_project_theme() != null else ThemeDB.get_default_theme()
if theme_src.has_stylebox("panel", "TooltipPanel"):
	frame.add_theme_stylebox_override("panel", theme_src.get_stylebox("panel", "TooltipPanel"))
var lbl := Label.new()
lbl.name = "Text"
lbl.text = text
if theme_src.has_font_size("font_size", "TooltipLabel"):
	lbl.add_theme_font_size_override("font_size", theme_src.get_font_size("font_size", "TooltipLabel"))
frame.add_child(lbl)
return frame
```

### `_rich(text: String, font: int) -> RichTextLabel`
```logic
var rich := RichTextLabel.new()
rich.bbcode_enabled = true
rich.fit_content = true
rich.scroll_active = false
rich.size_flags_horizontal = Control.SIZE_EXPAND_FILL
rich.add_theme_font_size_override("normal_font_size", font)
rich.add_theme_font_size_override("bold_font_size", font + 1)
rich.text = text
return rich
```

