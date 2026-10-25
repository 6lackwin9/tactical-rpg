//! Skill checks (SkillValidator, SkillKinds, SkillText.validate / warnings, SkillRanks.fold): family rules across the
//! rank rows, row rules of one flat row, combo and extra-grant rules, the description template, soft warnings.

use super::View;
use crate::source::{content, fields, render, token_value, RANK_KEYS};
use aegis_content::schema::is_id;
use aegis_core::json::{self, Dict};
use emberwake_rules::status;
use serde_json::Value;

// ---- SkillKinds ----------------------------------------------------------------------------------------------------

/// The kinds in table order (SkillKinds.KINDS).
pub const KINDS: [&str; 9] = ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE", "REST", "MOVE", "TARGET", "STATUS"];
const AUTHORABLE: [&str; 5] = ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"];
const COMMON_FIELDS: [&str; 16] = ["id", "name", "kind", "level", "blocks", "description", "template", "system", "basic", "learned_only", "grade", "requires", "origin", "unlock_offset", "family", "tier"];
const SECTION_FIELDS: [(&str, &[&str]); 13] = [
    ("damage", &["power_mult", "weapon_hand", "dash_tiles"]),
    ("inflict", &["status_effect", "status_potency", "status_magnitude", "status_rounds"]),
    ("reach", &["reach_min", "reach_max"]),
    ("area", &["area"]),
    ("support", &["target_mode", "heal_stat", "heal_base", "heal_mult", "cleanse_statuses"]),
    ("grant", &["grant_status", "status_magnitude", "status_rounds", "extra_grants"]),
    ("guard", &["mitigation"]),
    ("focus", &["boost", "boost_rounds"]),
    ("jump", &["jump_tiles"]),
    ("flow", &["ends_in_stance"]),
    ("combo", &["disrupt", "exploit", "exploit_consume"]),
    ("rest", &["recover_ap"]),
    ("gate", &["weapon_types", "min_proficiency"]),
];

fn sections_of(kind: &str) -> &'static [&'static str] {
    match kind {
        "ATTACK" => &["damage", "inflict", "combo", "flow", "reach", "area", "gate"],
        "DEFEND" => &["grant", "guard", "flow", "gate"],
        "FOCUS" => &["focus", "flow"],
        "SUPPORT" => &["reach", "support", "grant", "flow", "gate"],
        "DISENGAGE" => &["jump"],
        "REST" => &["rest"],
        _ => &[],
    }
}

fn section_fields(section: &str) -> &'static [&'static str] {
    SECTION_FIELDS.iter().find(|(s, _)| *s == section).map_or(&[], |(_, f)| f)
}

/// SkillKinds.has_field: a common field, a field of one of the kind's sections, or the ATTACK-only stance.
pub fn has_field(kind: &str, field: &str) -> bool {
    if COMMON_FIELDS.contains(&field) {
        return true;
    }
    if field == "stance" {
        return kind == "ATTACK";
    }
    sections_of(kind).iter().any(|s| section_fields(s).contains(&field))
}

/// Every section-bound field (SkillKinds.sectioned_fields), in table order.
fn sectioned_fields() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (_, list) in SECTION_FIELDS {
        for f in list.iter() {
            if !out.contains(f) {
                out.push(f);
            }
        }
    }
    out
}

fn kinds_with(field: &str) -> Vec<&'static str> {
    KINDS.iter().copied().filter(|k| has_field(k, field)).collect()
}

/// The stance a skill fights in (SkillDef.stance): the row's override, else its kind's.
fn own_stance(e: &Dict) -> &'static str {
    let set = json::s(e, "stance");
    if let Some(s) = ["NEUTRAL", "OFFENSIVE", "DEFENSIVE", "TACTICAL"].iter().find(|s| **s == set) {
        return s;
    }
    match json::s(e, "kind").as_str() {
        "ATTACK" => "OFFENSIVE",
        "DEFEND" => "DEFENSIVE",
        "FOCUS" | "REST" => "TACTICAL",
        _ => "NEUTRAL",
    }
}

// ---- SkillRanks ----------------------------------------------------------------------------------------------------

const DERIVED_KEYS: [&str; 6] = ["id", "family", "tier", "name", "description", "template"];
const CORE_KEYS: [&str; 6] = ["id", "name", "kind", "level", "blocks", "description"];

pub fn head_of(row: &Dict) -> String {
    let fam = json::s(row, "family");
    if fam.is_empty() {
        json::s(row, "id")
    } else {
        fam
    }
}

pub fn rank_id(head: &str, rank: i64) -> String {
    if rank <= 1 {
        head.to_string()
    } else {
        format!("{head}_{rank}")
    }
}

fn rank_name(base: &str, rank: usize) -> String {
    if rank <= 1 {
        base.to_string()
    } else {
        format!("{base} {}", ["", "I", "II", "III", "IV", "V"][rank.min(5)])
    }
}

fn tier(e: &Dict) -> i64 {
    json::i(e, "tier", 1)
}

fn modifiers_of(e: &Dict) -> Vec<Dict> {
    json::aget(e, "ranks").iter().map(|r| json::dict(r).clone()).collect()
}

/// SkillRanks.effective: rank `rank` of a ranked entry (base overlaid by its modifier), derived keys set.
fn effective(e: &Dict, rank: usize) -> Dict {
    let mut raw = e.clone();
    for k in ["ranks", "family", "tier", "template"] {
        raw.shift_remove(k);
    }
    let base = content("skills", &raw);
    let head = json::s(&base, "id");
    let mut row = base.clone();
    if rank >= 2 {
        if let Some(m) = modifiers_of(e).get(rank - 2) {
            for (k, v) in m {
                if RANK_KEYS.contains(&k.as_str()) {
                    row.insert(k.clone(), v.clone());
                }
            }
        }
    }
    let mut row = content("skills", &row);
    row.insert("id".into(), Value::from(rank_id(&head, rank as i64)));
    row.insert("family".into(), Value::from(if rank <= 1 { String::new() } else { head.clone() }));
    row.insert("tier".into(), Value::from(rank.max(1) as i64));
    row.insert("name".into(), Value::from(rank_name(&json::s(&base, "name"), rank)));
    row.insert("template".into(), Value::from(json::s(&base, "description")));
    row
}

fn escape(text: &str) -> String {
    text.replace('{', "{{").replace('}', "}}")
}

/// SkillRanks.collapse: the flat skill rows as ranked entries (one per family, file order of the family's first row)
/// and the reasons a family can't be folded.
pub fn collapse(rows: &[Dict]) -> (Vec<Dict>, Vec<String>) {
    let mut groups: Vec<(String, Vec<Dict>)> = Vec::new();
    for raw in rows {
        let row = content("skills", raw);
        let head = head_of(&row);
        match groups.iter_mut().find(|(h, _)| *h == head) {
            Some((_, list)) => list.push(row),
            None => groups.push((head, vec![row])),
        }
    }
    let (mut entries, mut errors) = (Vec::new(), Vec::new());
    for (head, list) in &groups {
        match fold(head, list) {
            Ok(entry) => entries.push(entry),
            Err(e) => errors.extend(e),
        }
    }
    (entries, errors)
}

/// SkillKinds.REQUIRED_IDS: the skills the engine looks up by id.
pub const REQUIRED_IDS: [&str; 6] = ["strike", "guard", "move", "rest", "switch_target", "offhand_jab"];

/// ContentCatalog._required_skill_errors: the required skills exist, and they, the basics and the engine skills are
/// system assets.
pub fn required_errors(v: &View) -> Vec<String> {
    let system = |row: &Dict| json::s_or(row, "origin", super::ORIGIN_SYSTEM) == super::ORIGIN_SYSTEM;
    let mut errors = Vec::new();
    for id in REQUIRED_IDS {
        let row = v.get("skills", id);
        if row.is_empty() {
            errors.push(format!("Required engine skill '{id}' is missing."));
        } else if !system(row) {
            errors.push(format!("Required engine skill '{id}' must be origin system."));
        }
    }
    for row in v.entries("skills") {
        if (json::b(row, "basic", false) || json::b(row, "system", false)) && !system(row) {
            errors.push(format!("'{}' is a basic or engine skill and must be origin system.", json::s(row, "id")));
        }
    }
    errors
}

/// SkillRanks.fold: one family's flat rows as a ranked entry, or the reasons it can't be folded.
pub fn fold(head: &str, family_rows: &[Dict]) -> Result<Dict, Vec<String>> {
    let mut errors = Vec::new();
    let mut rows: Vec<Dict> = family_rows.to_vec();
    rows.sort_by_key(tier);
    let first = rows[0].clone();
    let kind = json::s(&first, "kind");
    if json::s(&first, "id") != head || tier(&first) != 1 {
        errors.push(format!("Family '{head}' has no tier 1 skill with that id (found {}).", json::s(&first, "id")));
    }
    for (i, row) in rows.iter().enumerate() {
        let t = tier(row);
        if t != i as i64 + 1 {
            errors.push(format!("Family '{head}': tiers must run 1..n without gaps (found tier {t} at position {}).", i + 1));
        } else if json::s(row, "id") != rank_id(head, t) {
            errors.push(format!("Family '{head}': tier {t} must have the id '{}' (found '{}').", rank_id(head, t), json::s(row, "id")));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let schema = fields("content", "skills");
    let field_of = |key: &str| schema.iter().map(json::dict).find(|f| json::s(f, "key") == key);
    for (i, row) in rows.iter().enumerate() {
        let rank = i + 1;
        for f in schema.iter().map(json::dict) {
            let key = json::s(f, "key");
            if has_field(&kind, &key) || !row.contains_key(&key) || json::deep_equal(&row[&key], json::get(f, "default")) {
                continue;
            }
            errors.push(format!("Family '{head}': rank {rank} sets '{key}', which {kind} skills do not use (it would be lost on save)."));
        }
        if rank < 2 {
            continue;
        }
        for (key, v) in row {
            if field_of(key).is_some() {
                continue;
            }
            if !first.get(key).is_some_and(|w| json::deep_equal(w, v)) {
                errors.push(format!("Family '{head}': rank {rank} has the unknown key '{key}' that rank 1 does not share (only rank 1 keeps unknown keys)."));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let base_template = if json::s(&first, "template").is_empty() { escape(&json::s(&first, "description")) } else { json::s(&first, "template") };
    let rank_fields: Vec<String> = schema.iter().map(|f| json::s(json::dict(f), "key")).filter(|k| RANK_KEYS.contains(&k.as_str()) && has_field(&kind, k)).collect();
    let mut modifiers = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        let rank = i + 1;
        for f in schema.iter().map(json::dict) {
            let key = json::s(f, "key");
            if DERIVED_KEYS.contains(&key.as_str()) || RANK_KEYS.contains(&key.as_str()) || !has_field(&kind, &key) {
                continue;
            }
            if !json::deep_equal(json::get(row, &key), json::get(&first, &key)) {
                errors.push(format!("Family '{head}': '{key}' must match across ranks (rank {rank} differs)."));
            }
        }
        let derived = rank_name(&json::s(&first, "name"), rank);
        if json::s(row, "name") != derived {
            errors.push(format!("Family '{head}': rank {rank} name '{}' is not the derived name '{derived}'.", json::s(row, "name")));
        }
        if !json::s(row, "template").is_empty() && json::s(row, "template") != base_template {
            errors.push(format!("Family '{head}': the description template must match across ranks (rank {rank} differs)."));
        }
        let mut m = Dict::new();
        for k in &rank_fields {
            if !json::deep_equal(json::get(row, k), json::get(&first, k)) {
                m.insert(k.clone(), json::get(row, k).clone());
            }
        }
        if json::s(row, "description") != render(&base_template, &kind, row, rank) {
            m.insert("description".into(), Value::from(json::s(row, "description")));
        }
        modifiers.push(Value::Object(m));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut out = Dict::new();
    for f in schema.iter().map(json::dict) {
        let key = json::s(f, "key");
        if matches!(key.as_str(), "family" | "tier" | "template") || !has_field(&kind, &key) {
            continue;
        }
        let value = match key.as_str() {
            "id" => Value::from(head),
            "description" => Value::from(base_template.clone()),
            _ => json::get(&first, &key).clone(),
        };
        if !CORE_KEYS.contains(&key.as_str()) && json::deep_equal(&value, json::get(f, "default")) {
            continue;
        }
        out.insert(key, value);
    }
    for (k, v) in &first {
        if !out.contains_key(k) && field_of(k).is_none() {
            out.insert(k.clone(), v.clone());
        }
    }
    if !modifiers.is_empty() {
        out.insert("ranks".into(), Value::Array(modifiers));
    }
    Ok(out)
}

// ---- SkillText -----------------------------------------------------------------------------------------------------

/// SkillText.TOKEN_KINDS in table order.
const TOKENS: [(&str, &[&str]); 23] = [
    ("name", &["ALL"]),
    ("ap", &["ALL"]),
    ("blocks", &["ALL"]),
    ("power", &["ATTACK"]),
    ("dash", &["ATTACK"]),
    ("reach", &["ATTACK", "SUPPORT"]),
    ("weapons", &["ATTACK", "DEFEND", "SUPPORT"]),
    ("proficiency", &["ATTACK", "DEFEND", "SUPPORT"]),
    ("status", &["ATTACK", "SUPPORT", "DEFEND"]),
    ("potency", &["ATTACK"]),
    ("magnitude", &["ATTACK", "SUPPORT", "DEFEND"]),
    ("rounds", &["ATTACK", "SUPPORT", "DEFEND", "FOCUS"]),
    ("heal", &["SUPPORT"]),
    ("cleanse", &["SUPPORT"]),
    ("mitigation", &["DEFEND"]),
    ("boost", &["FOCUS"]),
    ("jump", &["DISENGAGE"]),
    ("recover", &["REST"]),
    ("unlock", &["ALL"]),
    ("flow", &["ATTACK", "DEFEND", "SUPPORT", "FOCUS"]),
    ("exploit", &["ATTACK"]),
    ("disrupt", &["ATTACK"]),
    ("extra", &["DEFEND", "SUPPORT"]),
];
const COUNTED: [&str; 5] = ["blocks", "dash", "reach", "rounds", "jump"];
const MAX_LENGTH: usize = 240;

fn token_valid_for(token: &str, kind: &str) -> bool {
    TOKENS.iter().find(|(t, _)| *t == token).is_some_and(|(_, k)| k.contains(&"ALL") || k.contains(&kind))
}

fn tokens_for(kind: &str) -> Vec<&'static str> {
    TOKENS.iter().map(|(t, _)| *t).filter(|t| token_valid_for(t, kind)).collect()
}

pub(crate) enum Part {
    Lit(String),
    Token { token: String, suffix: String, raw: String },
}

fn chars_len(s: &str) -> usize {
    s.chars().count()
}

/// SkillText.parse: literal and token parts, and the brace problems (positions count characters).
pub(crate) fn parse(template: &str) -> (Vec<Part>, Vec<String>) {
    let t: Vec<char> = template.chars().collect();
    let (mut parts, mut errors, mut lit) = (Vec::new(), Vec::new(), String::new());
    let n = t.len();
    let find = |from: usize, c: char| t[from..].iter().position(|x| *x == c).map(|p| p + from);
    let mut i = 0;
    while i < n {
        let ch = t[i];
        if ch == '{' {
            if i + 1 < n && t[i + 1] == '{' {
                lit.push('{');
                i += 2;
                continue;
            }
            let close = find(i + 1, '}');
            let nested = find(i + 1, '{');
            let Some(close) = close.filter(|c| nested.is_none_or(|nn| nn > *c)) else {
                errors.push(format!("Unbalanced '{{' at position {i}: close the token with '}}' (write '{{{{' for a literal brace)."));
                lit.push(ch);
                i += 1;
                continue;
            };
            let body: String = t[i + 1..close].iter().collect();
            if body.trim().is_empty() {
                errors.push(format!("Empty token '{{}}' at position {i}."));
            } else {
                if !lit.is_empty() {
                    parts.push(Part::Lit(std::mem::take(&mut lit)));
                }
                let (token, suffix) = match body.split_once(':') {
                    Some((a, b)) => (a.to_string(), b.split(':').next().unwrap_or("").to_string()),
                    None => (body.clone(), String::new()),
                };
                parts.push(Part::Token { token, suffix, raw: body });
            }
            i = close + 1;
        } else if ch == '}' {
            if i + 1 < n && t[i + 1] == '}' {
                lit.push('}');
                i += 2;
            } else {
                errors.push(format!("Stray '}}' at position {i} (write '}}}}' for a literal brace)."));
                i += 1;
            }
        } else {
            lit.push(ch);
            i += 1;
        }
    }
    if !lit.is_empty() {
        parts.push(Part::Lit(lit));
    }
    (parts, errors)
}

/// SkillText.validate: the template problems of a ranked entry (every rank rendered).
fn validate_template(template: &str, entry: &Dict) -> Vec<String> {
    let kind = json::s_or(entry, "kind", "ATTACK");
    let (parts, mut errors) = parse(template);
    if chars_len(template) > MAX_LENGTH {
        errors.push(format!("The description template is {} characters (max {MAX_LENGTH}).", chars_len(template)));
    }
    let valid = tokens_for(&kind).join(", ");
    let mut usable: Vec<String> = Vec::new();
    for p in &parts {
        let Part::Token { token, suffix, raw } = p else { continue };
        if !TOKENS.iter().any(|(t, _)| t == token) {
            errors.push(format!("Unknown token {{{raw}}}. Valid for {kind}: {valid}."));
            continue;
        }
        if !token_valid_for(token, &kind) {
            errors.push(format!("Token {{{token}}} does not apply to {kind} skills. Valid: {valid}."));
            continue;
        }
        if token == "unlock" && (json::b(entry, "basic", false) || json::b(entry, "system", false)) {
            errors.push("Token {unlock} does not apply to basic or engine skills.".into());
            continue;
        }
        if !suffix.is_empty() && (suffix != "n" || !COUNTED.contains(&token.as_str())) {
            errors.push(format!("Token {{{token}}}: unknown suffix ':{suffix}' (only ':n' on {}).", COUNTED.join(", ")));
            continue;
        }
        if !usable.contains(raw) {
            usable.push(raw.clone());
        }
    }
    let count = modifiers_of(entry).len() + 1;
    for rank in 1..=count {
        let row = effective(entry, rank);
        for raw in &usable {
            let (token, suffix) = raw.split_once(':').map_or((raw.as_str(), ""), |(a, b)| (a, b.split(':').next().unwrap_or("")));
            if token_value(token, suffix, &row, rank).is_empty() {
                errors.push(format!("Rank {rank} ({}): token {{{raw}}} resolves to nothing (the field is empty or 0 for that rank).", json::s(&row, "name")));
            }
        }
        if errors.is_empty() {
            let text = render(template, &kind, &row, rank);
            if chars_len(&text) > MAX_LENGTH {
                errors.push(format!("Rank {rank} ({}): the rendered description is {} characters (max {MAX_LENGTH}).", json::s(&row, "name"), chars_len(&text)));
            }
        }
    }
    errors
}

/// SkillText.warnings: literal numbers outside tokens (with the token that holds them), literal rank descriptions.
fn template_warnings(template: &str, entry: &Dict) -> Vec<String> {
    let (parts, _) = parse(template);
    let kind = json::s_or(entry, "kind", "ATTACK");
    let first = effective(entry, 1);
    let mut out = Vec::new();
    for p in &parts {
        let Part::Lit(lit) = p else { continue };
        let mut digits = String::new();
        let mut runs = Vec::new();
        for c in lit.chars().chain(std::iter::once(' ')) {
            if c.is_ascii_digit() {
                digits.push(c);
            } else if !digits.is_empty() {
                runs.push(std::mem::take(&mut digits));
            }
        }
        for d in runs {
            let hint = tokens_for(&kind)
                .into_iter()
                .filter(|t| *t != "name")
                .find(|t| {
                    let plain = token_value(t, "", &first, 1);
                    let plain = plain.strip_suffix('%').unwrap_or(&plain);
                    token_value(t, "n", &first, 1) == d || plain.strip_suffix(" AP").unwrap_or(plain) == d
                })
                .map(|t| format!(" Use {{{t}}}."))
                .unwrap_or_default();
            out.push(format!("Description has the literal number {d}; it can drift from the fields.{hint}"));
        }
    }
    for (i, r) in json::aget(entry, "ranks").iter().enumerate() {
        if json::dict(r).contains_key("description") {
            out.push(format!("Rank {} has a literal description instead of the template.", i + 2));
        }
    }
    out
}

// ---- SkillValidator ------------------------------------------------------------------------------------------------

const MAX_CLASS_TIER: i64 = 3;
const MAX_UNLOCK_OFFSET: i64 = 8;
const UNARMED_TOKEN: &str = "unarmed";
const CLEANSE_ALL: &str = "all";
const HEAL_STATS: [&str; 2] = ["INT", "KNOW"];
pub const GEAR_TYPES: [&str; 16] = ["sword", "axe", "mace", "hammer", "fist", "dagger", "bow", "crossbow", "staff", "wand", "orb", "tome", "shield", "plate", "leather", "cloth"];

fn grade_rank(grade: &str) -> usize {
    ["novice", "advanced", "expert"].iter().position(|g| *g == grade).unwrap_or(0)
}

fn support_buffs() -> Vec<&'static str> {
    status::ROWS.iter().filter(|r| r.buff && r.grant == "support").map(|r| r.id).collect()
}

fn has_magnitude(st: &str) -> bool {
    status::row_of(st).is_some_and(|r| r.magnitude_max > 0)
}

fn magnitude_range(st: &str) -> (i64, i64) {
    status::row_of(st).map_or((0, 0), |r| (r.magnitude_min as i64, r.magnitude_max as i64))
}

fn magnitude_ids() -> Vec<&'static str> {
    status::ROWS.iter().filter(|r| r.magnitude_max > 0).map(|r| r.id).collect()
}

/// Skill family rules: a valid family id, ids <head> / <head>_<n>, ranks 1..n without gaps, one kind per family, and
/// every base-only field identical across the ranks.
fn family_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let id = json::s(e, "id");
    let family = json::s(e, "family");
    let head = if family.is_empty() { id.clone() } else { family.clone() };
    if !family.is_empty() && !is_id(&family) {
        errors.push(format!("Family '{family}' must be lowercase snake_case (a-z, 0-9, _)."));
    }
    let t = tier(e);
    let basic = json::b(e, "basic", false);
    let system = json::b(e, "system", false);
    let kind = json::s(e, "kind");
    if basic && t > 1 {
        errors.push(format!("Basic skills have exactly one rank (tier {t} is above the 1-tier limit for basic skills)."));
    } else if !basic && !system && t > MAX_CLASS_TIER {
        errors.push(format!("Tier {t} is above the {MAX_CLASS_TIER}-tier limit for class skills."));
    }
    if t <= 1 {
        if !family.is_empty() && family != id {
            errors.push(format!("A tier 1 skill is the family head: its id '{id}' must equal the family '{family}'."));
        }
    } else if id != rank_id(&head, t) {
        errors.push(format!("Tier {t} of family '{head}' must have the id '{}' (found '{id}').", rank_id(&head, t)));
    }
    let mut tiers = vec![t];
    for other in v.entries("skills") {
        let other_id = json::s(other, "id");
        if other_id == id || head_of(other) != head {
            continue;
        }
        let ot = tier(other);
        if ot == t {
            errors.push(format!("Family '{head}' already has a tier {t} skill ({other_id})."));
        }
        tiers.push(ot);
        if json::s(other, "kind") != kind {
            errors.push(format!("Family '{head}' mixes kinds ({other_id} is {}).", json::s(other, "kind")));
        }
        if json::b(other, "basic", false) != basic {
            errors.push(format!("Family '{head}': basic must match across all tiers ({other_id} differs)."));
        }
        for f in fields("content", "skills").iter().map(json::dict) {
            let key = json::s(f, "key");
            if DERIVED_KEYS.contains(&key.as_str()) || RANK_KEYS.contains(&key.as_str()) || key == "basic" || key == "kind" || !has_field(&kind, &key) {
                continue;
            }
            if !json::deep_equal(json::get(other, &key), json::get(e, &key)) {
                errors.push(format!("Family '{head}': {} must match across all tiers ({other_id} differs).", json::s_or(f, "label", &key).to_lowercase()));
            }
        }
    }
    tiers.sort();
    if let Some(i) = tiers.iter().enumerate().position(|(i, t)| *t != i as i64 + 1) {
        errors.push(format!("Family '{head}': tiers must run 1..n without gaps (it needs a tier {} skill).", i + 1));
    }
    errors
}

/// SkillValidator.entry_errors: family rules, row rules, and the family's description template (from its first rank).
pub fn entry_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = family_errors(v, e);
    errors.extend(row_problems(v, e));
    let template = json::s(e, "template");
    if !template.is_empty() && tier(e) <= 1 {
        let id = json::s(e, "id");
        let mut rows = vec![e.clone()];
        rows.extend(v.entries("skills").iter().filter(|o| head_of(o) == id && json::s(o, "id") != id).cloned());
        if let Ok(folded) = fold(&id, &rows) {
            errors.extend(validate_template(&template, &folded));
        }
    }
    errors
}

/// SkillValidator.row_problems: the rules of one flat row.
fn row_problems(v: &View, e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let kind = json::s(e, "kind");
    let t = tier(e);
    let basic = json::b(e, "basic", false);
    let system = json::b(e, "system", false);
    let grade = json::s(e, "grade");
    let unlock_offset = json::i(e, "unlock_offset", -1);
    if !(-1..=MAX_UNLOCK_OFFSET).contains(&unlock_offset) {
        out.push(format!("Unlock offset must be -1 (derived) or 0-{MAX_UNLOCK_OFFSET}."));
    } else if unlock_offset > 0 && t == 1 {
        out.push("A tier 1 skill opens at its family's unlock level: leave the unlock offset at -1 or 0.".into());
    }
    let requires = json::sget(e, "requires");
    if (basic || system) && grade != "novice" {
        out.push("Only class skills can be advanced (not basic or engine skills).".into());
    }
    if (basic || system) && !requires.is_empty() {
        out.push("Basic and engine skills cannot require other skills.".into());
    }
    let head = head_of(e);
    for required in &requires {
        let target = v.get("skills", required);
        if target.is_empty() {
            continue;
        }
        if *required == head {
            out.push("A skill cannot require itself.".into());
        } else if !json::s(target, "family").is_empty() || tier(target) != 1 {
            out.push(format!("Requires: '{required}' is a higher rank; name the family head."));
        } else if grade_rank(&json::s_or(target, "grade", "novice")) >= grade_rank(&grade) {
            out.push(format!("Requires: '{required}' ({}) must be a lower grade than this skill ({grade}).", json::s_or(target, "grade", "novice")));
        }
    }
    if t == 1 {
        for other in v.entries("skills") {
            if head_of(other) == head || !json::sget(other, "requires").contains(&head) {
                continue;
            }
            if tier(other) == 1 && grade_rank(&json::s_or(other, "grade", "novice")) <= grade_rank(&grade) {
                out.push(format!("Grade: '{}' requires this skill and must stay a higher grade (a grade change would break it).", json::s(other, "id")));
            }
        }
    }
    if !system && !AUTHORABLE.contains(&kind.as_str()) {
        out.push(format!("{kind} skills are engine-only; author an {} skill.", AUTHORABLE.join(" / ")));
    }
    let mut guarded = sectioned_fields();
    guarded.push("stance");
    for key in guarded {
        if has_field(&kind, key) || key == "extra_grants" {
            continue;
        }
        let Some(f) = fields("content", "skills").iter().map(json::dict).find(|f| json::s(f, "key") == key) else { continue };
        if !json::deep_equal(json::get(e, key), json::get(f, "default")) {
            out.push(format!("{} belongs to {} skills, not {kind}.", json::s(f, "label"), kinds_with(key).join("/")));
        }
    }
    let status_id = json::s(e, "status_effect");
    let magnitude = json::i(e, "status_magnitude", 0);
    let magnitude_status = if status_id.is_empty() { json::s(e, "grant_status") } else { status_id.clone() };
    if has_magnitude(&magnitude_status) {
        let (lo, hi) = magnitude_range(&magnitude_status);
        if magnitude < lo || magnitude > hi {
            out.push(format!("{magnitude_status} needs a magnitude {lo}-{hi}"));
        }
    } else if magnitude != 0 {
        out.push(format!("magnitude only applies to {}", magnitude_ids().join(", ")));
    }
    let grant = json::s(e, "grant_status");
    let reach_min = json::i(e, "reach_min", 0);
    let reach_max = json::i(e, "reach_max", 0);
    let debuffs = status::debuffs();
    if kind == "SUPPORT" {
        let heal_stat = json::s(e, "heal_stat");
        let heal_mult = json::f(e, "heal_mult", 0.0);
        let heal_base = json::i(e, "heal_base", 0);
        let cleanse = json::sget(e, "cleanse_statuses");
        if heal_base <= 0 && heal_mult <= 0.0 && grant.is_empty() && cleanse.is_empty() {
            out.push("A support skill needs a heal (base or per-stat), a cleanse or a granted status.".into());
        }
        if cleanse.iter().any(|c| c == CLEANSE_ALL) && cleanse.len() > 1 {
            out.push(format!("Cleanses: '{CLEANSE_ALL}' stands alone (it already clears every debuff)."));
        }
        for st in &cleanse {
            if st != CLEANSE_ALL && !debuffs.contains(&st.as_str()) {
                out.push(format!("Cleanses: '{st}' is not a debuff status ({}, or '{CLEANSE_ALL}').", debuffs.join(", ")));
            }
        }
        if heal_mult > 0.0 && !HEAL_STATS.contains(&heal_stat.as_str()) {
            out.push("Heal per stat point needs a heal stat (INT or KNOW).".into());
        }
        if heal_mult <= 0.0 && !heal_stat.is_empty() {
            out.push("Heal stat is set but heal per stat point is 0.".into());
        }
        if !grant.is_empty() && !support_buffs().contains(&grant.as_str()) {
            out.push(format!("Grants: '{grant}' is not a support buff status ({}).", support_buffs().join(", ")));
        }
        if reach_max > 0 && reach_min > reach_max {
            out.push("Reach min is above reach max.".into());
        }
        if grant == "taunt" && json::s(e, "target_mode") != "SELF" {
            out.push("Taunt is cast on the taunter itself: set Targets to SELF.".into());
        }
    } else if kind == "DEFEND" {
        let mut options = status::defend_buffs();
        options.extend(support_buffs());
        if !grant.is_empty() && !options.contains(&grant.as_str()) {
            out.push(format!("Grants: '{grant}' is not a defend or support buff status ({}).", options.join(", ")));
        }
    }
    if kind == "DISENGAGE" && json::i(e, "jump_tiles", 0) < 1 {
        out.push("A disengage skill needs jump tiles of at least 1.".into());
    }
    if kind == "ATTACK" {
        let area = json::s(e, "area");
        let dash = json::i(e, "dash_tiles", 0);
        let hand = json::s(e, "weapon_hand");
        if json::f(e, "power_mult", 1.0) <= 0.0 {
            if status_id.is_empty() {
                out.push("A skill with no damage needs a status".into());
            }
            if area != "NONE" {
                out.push("A skill with no damage can't be an area attack.".into());
            }
        }
        if area == "SWEEP" && (dash > 0 || reach_max > 1) {
            out.push("SWEEP hits the L of tiles around you; leave reach at 0 or 1 and no gap-close.".into());
        }
        if dash > 0 {
            if area != "NONE" {
                out.push("An area attack can't be a gap-close.".into());
            }
            if hand != "main" {
                out.push("An off-hand or dual-wield strike can't be a gap-close.".into());
            }
        }
        if hand == "both" {
            out.push("Weapon hand 'both' is not playable yet (engine pending).".into());
        }
        if !status_id.is_empty() && !debuffs.contains(&status_id.as_str()) {
            out.push(format!("Inflicts: '{status_id}' is not a debuff status ({}).", debuffs.join(", ")));
        }
        if reach_max > 0 && reach_min > reach_max {
            out.push("Reach min is above reach max.".into());
        }
        if reach_max == 0 && reach_min > 0 {
            out.push("Set reach max too (or leave both at 0 for the unit's default band).".into());
        }
        if area == "LINE" && reach_max > 1 {
            out.push("LINE areas start from an adjacent tile; leave the reach at 0 or 1.".into());
        }
        if !status_id.is_empty() && json::i(e, "status_potency", 0) <= 0 {
            out.push("Give the status a potency above 0.".into());
        }
    }
    out.extend(extra_grant_problems(e, &kind));
    out.extend(combo_problems(e, &kind));
    let types = json::sget(e, "weapon_types");
    for gear in &types {
        if !GEAR_TYPES.contains(&gear.as_str()) && gear != UNARMED_TOKEN {
            out.push(format!("Needs weapon type: unknown gear type '{gear}' (gear types or '{UNARMED_TOKEN}')."));
        }
    }
    if !types.is_empty() && json::i(e, "min_proficiency", 0) < 1 {
        out.push("Needs proficiency: a weapon type is listed, so give a proficiency of at least 1 (otherwise the types do nothing).".into());
    }
    out
}

/// extra_grants (BAL5-L): DEFEND or SUPPORT only, at most one, another buff the kind can grant, rounds 1-3, a
/// magnitude inside the status's range (0 without one); a SUPPORT taunt targets SELF.
fn extra_grant_problems(e: &Dict, kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    let extras = json::aget(e, "extra_grants");
    if extras.is_empty() {
        return out;
    }
    if kind != "DEFEND" && kind != "SUPPORT" {
        out.push("extra_grants needs a DEFEND or SUPPORT skill.".into());
        return out;
    }
    if extras.len() > 1 {
        out.push("A skill holds at most one extra grant.".into());
    }
    let mut options = support_buffs();
    if kind == "DEFEND" {
        options.extend(status::defend_buffs());
    }
    let grant = json::s(e, "grant_status");
    for raw in extras {
        let row = json::dict(raw);
        let st = json::s(row, "status");
        if !options.contains(&st.as_str()) || st == grant {
            out.push(format!("Extra grant '{st}' must be another buff."));
            continue;
        }
        let rounds = json::i(row, "rounds", 0);
        if !(1..=3).contains(&rounds) {
            out.push(format!("Extra grant '{st}' needs rounds 1-3."));
        }
        let magnitude = json::i(row, "magnitude", 0);
        if has_magnitude(&st) {
            let (lo, hi) = magnitude_range(&st);
            if magnitude < lo || magnitude > hi {
                out.push(format!("Extra grant '{st}' needs a magnitude {lo}-{hi}."));
            }
        } else if magnitude != 0 {
            out.push(format!("Extra grant '{st}' takes no magnitude (leave it 0)."));
        }
        if st == "taunt" && kind == "SUPPORT" && json::s(e, "target_mode") != "SELF" {
            out.push("Taunt is cast on the taunter itself: set Targets to SELF.".into());
        }
    }
    out
}

/// The combo fields (S12C-L): flow end stance, stance-break disrupt, exploit list and consume.
fn combo_problems(e: &Dict, kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    let ends = json::s(e, "ends_in_stance");
    if !ends.is_empty() && has_field(kind, "ends_in_stance") {
        if ends == own_stance(e) {
            out.push("Ends in stance equals the skill's stance; leave it empty.".into());
        }
        if kind == "DEFEND" && json::s(e, "grant_status") == "riposte" {
            out.push("A riposte skill cannot flow: a stance change ends riposte.".into());
        }
    }
    if kind != "ATTACK" {
        return out;
    }
    let zero_power = json::f(e, "power_mult", 1.0) <= 0.0;
    if json::s_or(e, "disrupt", "NONE") == "STANCE_BREAK" && zero_power {
        out.push("A zero-power attack cannot break a stance.".into());
    }
    let exploit = json::aget(e, "exploit");
    if exploit.len() > 2 {
        out.push("Exploit takes at most 2 statuses.".into());
    }
    let mut seen: Vec<String> = Vec::new();
    let debuffs = status::debuffs();
    for raw in exploit {
        let row = json::dict(raw);
        let st = json::s(row, "status");
        if !debuffs.contains(&st.as_str()) {
            out.push(format!("Exploit status '{st}' is not a debuff."));
        } else if seen.contains(&st) {
            out.push(format!("Exploit lists '{st}' twice."));
        }
        seen.push(st);
        let mult = json::f(row, "power_mult", 0.0);
        if !(1.05..=2.0).contains(&mult) {
            out.push("Exploit power must be 1.05-2.0.".into());
        }
    }
    if !exploit.is_empty() && zero_power {
        out.push("A zero-power attack cannot exploit.".into());
    }
    if json::b(e, "exploit_consume", false) && exploit.is_empty() {
        out.push("Consume needs an exploit status.".into());
    }
    out
}

/// SkillValidator.warnings: a ranked entry's template and rank warnings, else a flat row's template warnings; an
/// off-hand weapon gate other than unarmed. `raw` is the entry as given, `e` its normalized form.
pub fn warnings(raw: &Dict, e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let ranked = !raw.contains_key("tier") && !raw.contains_key("family");
    if ranked {
        out.extend(template_warnings(&json::s(e, "description"), raw));
        let base = effective(raw, 1);
        for (i, m) in modifiers_of(raw).iter().enumerate() {
            for (k, val) in m {
                if RANK_KEYS.contains(&k.as_str()) && base.get(k).is_some_and(|b| json::deep_equal(b, val)) {
                    out.push(format!("Rank {}: {k} equals the base value; the modifier is dropped on save.", i + 2));
                }
            }
        }
    } else if !json::s(e, "template").is_empty() {
        out.extend(template_warnings(&json::s(e, "template"), e));
    }
    let hand = json::s(e, "weapon_hand");
    if hand != "main" {
        for gear in json::sget(e, "weapon_types") {
            if gear != UNARMED_TOKEN {
                out.push(format!("Weapon hand '{hand}' with weapon type '{gear}': off-hand slots hold shields, orbs and tomes, so only 'unarmed' can satisfy this gate."));
            }
        }
    }
    out
}
