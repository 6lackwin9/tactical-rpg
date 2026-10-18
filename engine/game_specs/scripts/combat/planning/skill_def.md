---
class: SkillDef
extends: RefCounted
description: "Tactical RPG - Skill Definition (one tier of a command family) Leveled skills (level 1-5) cost AP equal to their level: a level 2 Attack or Defend always costs 2 AP unless an effect shifts it (Combatant.ap_cost_modifier). Move is the exception: 1 AP per tile covered (1-2 blocks, once per round). Rest is free and recovers AP. Switch Target is a free 1-block step inserted automatically when an attack changes the current target lock. Stances (rock-paper-scissors: Offensive > Tactical > Defensive > Offensive): Attack -> Offensive, Defend -> Defensive, Focus -> Tactical. Move keeps whatever stance the unit is in; Rest puts it in Tactical. Using a skill from another stance switches stance automatically for CombatTypes.STANCE_SWITCH_COST AP (no block cost). Rest enters Tactical for free. SUPPORT: heals and/or grants a buff status to itself or a friendly unit (SupportRules, SupportCommand). It keeps the unit's stance like Move does (no switch cost). DISENGAGE (M2c-2a2): a jump of up to `jump_tiles` tiles AWAY from the nearest (or a chosen) enemy. It is the unit's move for the round (first in the sequence, refused when rooted), costs AP = the skill's level (not per tile), leaps over units but not walls, and keeps the stance like Move. Which hand's weapon an ATTACK uses: the main weapon, the off-hand weapon (or bare hands), or both (dual strike, engine pending)."
---

# Properties
* `BOTH_ENABLED`: bool = false   # `both` is rejected by validation while the engine cannot play it (const)
* `UNARMED_TOKEN`: String = "unarmed" (const)
* `UNARMED_PROFICIENCY`: String = "fist" (const)
* `out`: Array[String] = Proficiency.gear_types()
* `CLEANSE_ALL`: String = "all"   # keyword in cleanse_statuses: every debuff (const)
* `SUPPORT_REACH`: int = 3   # default reach of a SUPPORT skill (or the caster's attack range if longer) (const)
* `MIN_LEVEL`: int = 1 (const)
* `MAX_LEVEL`: int = 5 (const)
* `id`: String = ""
* `display_name`: String = ""
* `kind`: Kind = Kind.ATTACK
* `level`: int = 1
* `blocks`: int = 1            # timeline length
* `power_mult`: float = 1.0    # ATTACK: multiplier on the unit's strike power
* `mitigation`: float = 0.5    # DEFEND: damage reduction
* `boost`: float = 1.5         # FOCUS: attack multiplier
* `boost_rounds`: int = 2      # FOCUS: duration
* `recover_ap`: int = 2        # REST: AP regained
* `area`: Area = Area.NONE     # ATTACK: area shape (tile-targeted)
* `weapon_hand`: WeaponHand = WeaponHand.MAIN   # ATTACK: which hand's weapon the strike uses (FORGE1 4.1; see is_off_hand)
* `stance_override`: int = -1  # ATTACK in another stance pool (e.g. Shield Bash is Defensive)
* `status_effect`: String = "" # status rolled on hit (StatusEffects), active next round
* `status_potency`: int = 0
* `status_magnitude`: int = 0   # carried by the inflicted / granted instance (not rolled; range and unit from the status registry row)
* `reach_min`: int = 0          # skill-specific reach band (0 = the unit's default band)
* `reach_max`: int = 0
* `weapon_types`: Array[String] = []   # gear types that may use it (empty = any); class-derived units only
* `min_proficiency`: int = 0           # training level in the wielded weapon type needed (0 = none)
* `status_rounds`: int = 1      # rounds an inflicted / granted status lasts once active
* `dash_tiles`: int = 0         # ATTACK gap-close: move up to this many tiles to get adjacent, then strike (counts as the round's move)
* `target_mode`: TargetMode = TargetMode.ALLY_OR_SELF   # SUPPORT: who it may target
* `heal_stat`: String = ""      # SUPPORT: stat the heal scales with ("INT" / "KNOW"; "" = fixed heal only)
* `heal_base`: int = 0          # SUPPORT: flat heal
* `heal_mult`: float = 0.0      # SUPPORT: heal per point of heal_stat (see SupportRules.heal_amount)
* `grant_status`: String = ""   # SUPPORT: buff status given to the target (StatusEffects.buffs())
* `cleanse_statuses`: Array[String] = []   # SUPPORT: debuffs removed from the target, active and pending (CLEANSE_ALL = every debuff; empty = none)
* `jump_tiles`: int = 0         # DISENGAGE: how many tiles the jump covers at most
* `ends_in_stance`: int = -1    # flow move: CombatTypes.StanceType the unit ends in after the last block (-1 = none)
* `disrupt`: int = Disrupt.NONE # ATTACK: STANCE_BREAK knocks a surviving Offensive / Defensive target into Tactical
* `exploit`: Array[Dictionary] = []   # ATTACK: [{"status": debuff id, "power_mult": float}] bonus vs an ACTIVE status
* `exploit_consume`: bool = false     # ATTACK: remove the matched status with the highest multiplier after the hit
* `learned_only`: bool = false  # unrestricted (hand-built) units don't know it unless they learn it
* `family`: String = ""         # skill family (tiers of one skill); empty = its own id (see family_id())
* `tier`: int = 1               # position in the family (1..5); each tier is its own skill with its own AP, blocks and power
* `basic`: bool = false         # available to every class (counted outside the loadout slots)
* `grade`: Grade = Grade.NOVICE   # family-level: class-path steps of advanced+ need StatRules.ADVANCED_MIN_LEVEL, KNOW can open them earlier
* `requires`: Array[String] = []  # family ids that must ALL be owned to own / learn this skill (each of a lower grade)
* `unlock_offset`: int = -1     # levels after the family's unlock level this row opens (-1 = derive from the tier, see tier_offset)
* `description`: String = ""
* `template`: String = ""        # the description template this rank's text was rendered from (SkillText); "" = a literal description
* `origin`: String = "system"    # "system" (shipped) or "user" (ContentSchema.ORIGIN_*); informational for the engine
* `system`: bool = false       # engine skill (Move, Rest, Switch Target, status blocks): always known, not deletable
* `SPELL_SOURCE_TYPES`: Array[String] = ["staff", "wand"] (const)
* `out`: Array[String] = []
* `cells`: Array[Vector2i] = []
* `dir`: Vector2i = cell - from
* `quadrant`: Variant

# Methods

### `static create(p_id: String, p_name: String, p_kind: Kind, p_level: int, p_blocks: int, p_description: String) -> SkillDef`
```logic
var skill := SkillDef.new()
skill.id = p_id
skill.display_name = p_name
skill.kind = p_kind
skill.level = clampi(p_level, MIN_LEVEL, MAX_LEVEL)
skill.blocks = p_blocks
skill.description = p_description
return skill
```

### `tier_offset() -> int`
```logic
if unlock_offset >= 0:
	return unlock_offset
return StatRules.TIER_UNLOCK_STEP * maxi(0, tier - 1)
```

### `family_id() -> String`
```logic
return id if family.is_empty() else family
```

### `power_for(user: Combatant) -> float`
```logic
return power_mult if user == null else power_mult * user.potency_for_skill(self) * user.weaken_factor()
```

### `deals_damage() -> bool`
```logic
return kind != Kind.ATTACK or power_mult > 0.0
```

### `ap_cost(actor: Combatant = null) -> int`
```logic
var ap_rule: String = str(SkillKinds.info(kind_name()).get("ap", "LEVEL"))
if ap_rule == "FREE":
	return 0
var shift: int = actor.ap_cost_modifier if actor != null else 0
var base: int = 1 if ap_rule == "PER_TILE" else level
return clampi(base + shift, MIN_LEVEL, MAX_LEVEL)
```

### `kind_name() -> String`
```logic
return str(Kind.keys()[kind])
```

### `keeps_stance() -> bool`
```logic
var rule: String = str(SkillKinds.info(kind_name()).get("stance", ""))
return rule == "KEEP" or (rule == "KEEP_UNLESS_SET" and stance_override < 0)
```

### `flows() -> bool`
```logic
return ends_in_stance >= 0
```

### `static exploit_match(entries: Array[Dictionary], target: Combatant, skip: Array[String] = []) -> Dictionary`
```logic
var best: Dictionary = {"status": "", "mult": 1.0}
if target == null:
	return best
for row: Dictionary in entries:
	var status: String = str(row.get("status", ""))
	var mult: float = DictUtil.to_float(row.get("power_mult", 1.0), 1.0)
	if target.has_status(status) and not skip.has(status) and mult > DictUtil.to_float(best["mult"]):
		best = {"status": status, "mult": mult}
return best
```

### `switch_cost(current: CombatTypes.StanceType) -> int`
```logic
if kind == Kind.REST or keeps_stance():
	return 0
return 0 if current == stance() else CombatTypes.STANCE_SWITCH_COST
```

### `heals() -> bool`
```logic
return kind == Kind.SUPPORT and (heal_base > 0 or heal_mult > 0.0)
```

### `in_group(group: String) -> bool`
```logic
match group:
	"spell":
		if kind != Kind.ATTACK:
			return false
		for source: String in SPELL_SOURCE_TYPES:
			if weapon_types.has(source):
				return true
		return false
	"heal":
		return heals()
return false
```

### `cleanses() -> bool`
```logic
return kind == Kind.SUPPORT and not cleanse_statuses.is_empty()
```

### `targets_ally() -> bool`
```logic
return kind == Kind.SUPPORT and target_mode != TargetMode.SELF
```

### `support_reach(c: Combatant) -> Vector2i`
```logic
if reach_max > 0:
	return Vector2i(reach_min, reach_max)
return Vector2i(0, maxi(SUPPORT_REACH, c.attack_range))
```

### `icon() -> String`
```logic
return SkillKinds.icon_of(kind_name())
```

### `reach_for(c: Combatant) -> Vector2i`
```logic
var extra: int = c.spell_reach_bonus() if in_group("spell") and area != Area.LINE and area != Area.SWEEP else 0   # spell_reach effects (not LINE / SWEEP)
if reach_max > 0:
	return Vector2i(reach_min, reach_max + extra)
if is_off_hand() or area == Area.LINE or area == Area.SWEEP:
	return Vector2i(1, 1)
if c.attack_range > 1:
	return Vector2i(2, maxi(3, c.attack_range) + extra)
return Vector2i(1, c.attack_range + extra)
```

### `is_area() -> bool`
```logic
return area != Area.NONE
```

### `is_off_hand() -> bool`
```logic
return weapon_hand == WeaponHand.OFF
```

### `label() -> String`
```logic
return "%s %s" % [icon(), display_name]
```

