class_name PlanBlockText
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan block text (COMBAT-UI3)
# Words and colours for one block of the planning strip: the action kind (its colour pair, the same colours the
# old two-row timeline used), the short label on the block, and the block hover card ("how it plays out": a title,
# one or two plain sentences, the blocks and AP). Display only; engine numbers stay out (no formulas, no multipliers
# beyond what the player picks). Pure functions of a scheduled action, so the strip and the tests read the same text.
# ==============================================================================

const KIND_ATTACK: String = "attack"
const KIND_AREA: String = "area"
const KIND_MOVE: String = "move"
const KIND_DEFEND: String = "defend"
const KIND_FOCUS: String = "focus"
const KIND_SUPPORT: String = "support"
const KIND_STATUS: String = "status"
const KIND_REST: String = "rest"
const KIND_TARGET: String = "target"
const KIND_ITEM: String = "item"
const KIND_OTHER: String = "other"
const KIND_EMPTY: String = "empty"

# kind -> [background, border, icon]
const KINDS: Dictionary = {
	KIND_ATTACK: [Color(0.45, 0.12, 0.14, 0.95), Color(0.9, 0.3, 0.3, 1.0), "⚔"],
	KIND_AREA: [Color(0.45, 0.2, 0.08, 0.95), Color(1.0, 0.55, 0.2, 1.0), "✸"],
	KIND_MOVE: [Color(0.08, 0.32, 0.44, 0.95), Color(0.2, 0.8, 0.95, 1.0), "➜"],
	KIND_DEFEND: [Color(0.12, 0.25, 0.48, 0.95), Color(0.3, 0.6, 0.95, 1.0), "🛡"],
	KIND_FOCUS: [Color(0.32, 0.14, 0.48, 0.95), Color(0.75, 0.4, 0.95, 1.0), "✦"],
	KIND_SUPPORT: [Color(0.1, 0.34, 0.2, 0.95), Color(0.35, 0.9, 0.5, 1.0), "+"],
	KIND_STATUS: [Color(0.25, 0.08, 0.08, 0.95), Color(0.7, 0.25, 0.25, 1.0), "✖"],
	KIND_REST: [Color(0.1, 0.3, 0.2, 0.95), Color(0.3, 0.85, 0.5, 1.0), "💤"],
	KIND_TARGET: [Color(0.08, 0.38, 0.24, 0.95), Color(0.25, 0.85, 0.5, 1.0), "🎯"],
	KIND_ITEM: [Color(0.22, 0.26, 0.34, 0.95), Color(0.5, 0.6, 0.7, 1.0), "⚗"],
	KIND_OTHER: [Color(0.22, 0.26, 0.34, 0.95), Color(0.5, 0.6, 0.7, 1.0), "·"],
	KIND_EMPTY: [Color(0.08, 0.10, 0.14, 0.6), Color(0.18, 0.24, 0.32, 0.5), "·"],
}
const COL_EMPTY_TEXT: Color = Color(0.45, 0.55, 0.65)

static func kind_of(cmd: BattleCommand) -> String:
	if cmd is AttackCommand:
		return KIND_ATTACK
	if cmd is AreaAttackCommand:
		return KIND_AREA
	if cmd is WalkCommand:
		return KIND_MOVE
	if cmd is DefendCommand:
		return KIND_DEFEND
	if cmd is FocusCommand:
		return KIND_FOCUS
	if cmd is SupportCommand:
		return KIND_SUPPORT
	if cmd is StatusBlockCommand:
		return KIND_STATUS
	if cmd is RestCommand:
		return KIND_REST
	if cmd is TargetCommand:
		return KIND_TARGET
	if cmd is ItemCommand:
		return KIND_ITEM
	return KIND_OTHER

static func _row(kind: String) -> Array:
	return DictUtil.array_of(KINDS.get(kind, KINDS[KIND_OTHER]))

static func bg(kind: String) -> Color:
	return _row(kind)[0]

static func border(kind: String) -> Color:
	return _row(kind)[1]

static func icon(kind: String) -> String:
	return str(_row(kind)[2])

# The words on a block: "1 Strike" (the colour tells the kind); an empty block "4 Idle".
static func label(act: CombatResolver.ScheduledAction) -> String:
	return "%d %s" % [act.start_block, act.command.command_name]

static func empty_label(block: int) -> String:
	return "%d Idle" % block

# The hover card's title: "⚔ Strike: Goblin Brawler" (the kind icon, the action and whom it is aimed at, when it has one).
static func title(act: CombatResolver.ScheduledAction) -> String:
	var who: Combatant = _aimed_at(act.command)
	var head: String = "%s %s" % [icon(kind_of(act.command)), act.command.command_name]
	return "%s: %s" % [head, who.name] if who != null else head

static func _aimed_at(cmd: BattleCommand) -> Combatant:
	if cmd is AttackCommand:
		return (cmd as AttackCommand).target
	if cmd is SupportCommand:
		return (cmd as SupportCommand).target
	if cmd is ItemCommand:
		return (cmd as ItemCommand).target
	if cmd is WalkCommand:
		return (cmd as WalkCommand).approach_target
	if cmd is TargetCommand:
		var raw: Variant = (cmd as TargetCommand).target_value
		return raw as Combatant if raw is Combatant else null
	return null

# How the action plays out, in plain words ("Step in and hit Goblin Brawler for about 47. Ends adjacent to it.").
static func body(act: CombatResolver.ScheduledAction) -> String:
	var cmd: BattleCommand = act.command
	match kind_of(cmd):
		KIND_ATTACK:
			return _attack_text(cmd as AttackCommand)
		KIND_AREA:
			var area: AreaAttackCommand = cmd as AreaAttackCommand
			var at: String = (" around %s" % str(area.aim_cell)) if area.aim_cell != Vector2i(-1, -1) else ""
			var dmg: String = (" for about %d in total" % area.est_damage) if area.est_damage > 0 else ""
			return "Hit everyone on %d tile%s%s%s." % [area.cells.size(), _s(area.cells.size()), at, dmg]
		KIND_MOVE:
			return _move_text(cmd as WalkCommand)
		KIND_DEFEND:
			var guard: DefendCommand = cmd as DefendCommand
			var grant: String = (" Grants %s." % StatusEffects.display_name(guard.grant_status)) if not guard.grant_status.is_empty() else ""
			return "Guard: take %d%% less damage while it lasts.%s" % [roundi(guard.mitigation_ratio * 100.0), grant]
		KIND_FOCUS:
			var focus: FocusCommand = cmd as FocusCommand
			return "Raise attack to x%.2f for %d round%s." % [focus.boost_multiplier, focus.duration_rounds, _s(focus.duration_rounds)]
		KIND_SUPPORT:
			return _support_text(cmd as SupportCommand)
		KIND_STATUS:
			var lost: StatusBlockCommand = cmd as StatusBlockCommand
			return "Lost to %s: nothing can be done in this block." % StatusEffects.display_name(lost.status)
		KIND_REST:
			return "Rest and recover %d AP." % (cmd as RestCommand).recover_ap
		KIND_TARGET:
			var who: Combatant = _aimed_at(cmd)
			return "Lock on to %s (free; the next strikes aim at it)." % who.name if who != null else "Drop the target lock."
		KIND_ITEM:
			var use: ItemCommand = cmd as ItemCommand
			var item_name: String = use.item.name if use.item != null else cmd.command_name
			return "Use %s%s." % [item_name, (" on %s" % use.target.name) if use.target != null and use.target != cmd.source else ""]
	return cmd.command_name + "."

static func _attack_text(atk: AttackCommand) -> String:
	var who: String = atk.target.name if atk.target != null else "the target"
	var reach: int = atk.max_range if atk.max_range > 0 else (atk.source.attack_range if atk.source != null else 1)
	var melee: bool = reach <= 1
	var lead: String = "Hit %s" % who
	if atk.dash_tiles > 0:
		lead = "Dash up to %d tile%s and hit %s" % [atk.dash_tiles, _s(atk.dash_tiles), who]
	elif atk.follow_tiles > 0 and melee:
		lead = "Step in and hit %s" % who
	var text: String = lead + ((" for about %d" % atk.est_damage) if atk.est_damage > 0 else "") + "."
	if melee:
		text += " Ends adjacent to it."
	else:
		text += " From range %d-%d." % [maxi(1, atk.min_range), reach]
	if not atk.status_effect.is_empty():
		text += " May inflict %s." % StatusEffects.display_name(atk.status_effect)
	return text

static func _move_text(walk: WalkCommand) -> String:
	var tiles: int = walk.path.size()
	if walk.approach_target != null:
		return "Move %d tile%s toward %s." % [tiles, _s(tiles), walk.approach_target.name]
	if walk.grid != null and walk.goal != WalkCommand.NONE and walk.grid.is_exit_tile(walk.goal):
		return "Move %d tile%s to the exit and withdraw." % [tiles, _s(tiles)]
	return "Move %d tile%s (%s)." % [tiles, _s(tiles), PlanEntry.dir_arrow(walk.direction())]

static func _support_text(sup: SupportCommand) -> String:
	var who: String = sup.target.name if sup.target != null else "an ally"
	if sup.target != null and sup.target == sup.source:
		who = "yourself"
	if sup.est_heal > 0:
		return "Heal %s for about %d HP." % [who, sup.est_heal]
	if sup.skill != null and not sup.skill.grant_status.is_empty():
		return "Give %s %s." % [who, StatusEffects.display_name(sup.skill.grant_status)]
	if sup.skill != null and sup.skill.cleanses():
		return "Clear %s's harmful effects." % who
	return "Support %s." % who

# "Blocks 1-2 · 3 AP".
static func cost_line(act: CombatResolver.ScheduledAction) -> String:
	var span: String = ("Block %d" % act.start_block) if act.end_block == act.start_block else ("Blocks %d-%d" % [act.start_block, act.end_block])
	return "%s · %d AP" % [span, act.command.ap_cost + act.command.stance_switch_cost]

# An empty block's card: what happens there (nothing) and what to do.
static func empty_body(unit_name: String, block: int, planning: bool) -> String:
	if planning:
		return "%s waits in block %d. Add an action, or confirm anyway." % [unit_name, block]
	return "No action in block %d." % block

static func _s(n: int) -> String:
	return "" if n == 1 else "s"
