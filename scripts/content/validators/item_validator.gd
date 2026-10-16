class_name ItemValidator
extends RefCounted

# Item validation rules (moved from ContentCatalog, CLN-9); ContentCatalog.validate_entry dispatches here.

static func item_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var type: String = str(e["item_type"])
	var slot: String = str(e["slot"])
	if type != "WEAPON" and Equipment.FOCUS_TYPES.has(str(e["weapon_type"])) and ContentSchema.EQUIP_TYPES.has(type):
		errors.append("Orb and tome are weapon types (item type WEAPON).")
	if Equipment.FOCUS_TYPES.has(str(e["off_hand_category"])) and str(e["weapon_type"]) != str(e["off_hand_category"]):
		errors.append("Off-hand category '%s' needs the matching gear type '%s'." % [str(e["off_hand_category"]), str(e["off_hand_category"])])
	if ContentSchema.EQUIP_TYPES.has(type):
		errors.append_array(Passives.validate_effects(e["effects"], func(skill_id: String) -> bool: return ContentCatalog.has("skills", skill_id), true, true))
	match type:
		"WEAPON":
			if slot != "MAIN_HAND" and slot != "OFF_HAND":
				errors.append("Weapons go in MAIN_HAND or OFF_HAND.")
			if DictUtil.to_bool(e["two_handed"]) and slot != "MAIN_HAND":
				errors.append("Two-handed weapons use the MAIN_HAND slot.")
			if DictUtil.to_bool(e["two_handed"]) and not str(e["off_hand_category"]).is_empty():
				errors.append("Hand conflict: a two-handed weapon (main_2h) can't also be an off-hand category weapon (off).")
			if Equipment.FOCUS_TYPES.has(str(e["weapon_type"])):
				if slot != "OFF_HAND":
					errors.append("Orb and tome weapons are off-hand only (use the OFF_HAND slot).")
				if str(e["off_hand_category"]) != str(e["weapon_type"]):
					errors.append("An orb or tome needs off-hand category '%s' (matching its gear type)." % str(e["weapon_type"]))
				if DictUtil.to_bool(e["two_handed"]):
					errors.append("Orb and tome weapons are never two-handed.")
			var tiers: Variant = e["threshold_tiers"]
			if not (tiers is Array):
				errors.append("Threshold tiers must be a list.")
			else:
				var tier_list: Array = tiers
				for t: Variant in tier_list:
					var tier: Dictionary = t if t is Dictionary else {}
					if not tier.has("stat") or not tier.has("threshold"):
						errors.append("Each threshold tier needs \"stat\" and \"threshold\".")
						break
		"SHIELD":
			if slot != "OFF_HAND":
				errors.append("Shields go in the OFF_HAND slot.")
		"ARMOR":
			if slot != "HEAD" and slot != "BODY":
				errors.append("Armor goes in HEAD or BODY.")
		"ACCESSORY":
			if slot != "ACCESSORY_1" and slot != "ACCESSORY_2":
				errors.append("Accessories go in ACCESSORY_1 or ACCESSORY_2.")
		"CONSUMABLE":
			if DictUtil.to_int(e["heal"]) == 0 and DictUtil.to_int(e["damage"]) == 0 and DictUtil.to_float(e["buff_pct"]) == 0.0:
				errors.append("A consumable should heal, damage or buff.")
	return errors
