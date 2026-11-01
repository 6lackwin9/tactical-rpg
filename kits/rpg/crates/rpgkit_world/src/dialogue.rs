//! RepRules, DialogueRules (runtime reads) and DialogueRunner (scripts/dialogue/): reputation bands, conditions and
//! texts of choices, and one conversation with one world character (services: quest offers, turn-ins, shops).

use crate::data::Data;
use crate::state::{flag_key_ok, flag_value_ok, GameState};
use crate::{board, new_game, progression, quest, roster, shop, story, terms};
use aegis_core::json::{self, int, Dict};
use indexmap::IndexMap;
use serde_json::Value;

pub mod rep {
    use super::*;

    pub const PREFIX: &str = "rep.";
    pub const MIN: i64 = -100;
    pub const MAX: i64 = 100;
    pub const HOSTILE: &str = "hostile";
    pub const NEUTRAL: &str = "neutral";
    /// (band, hi, buy %, sell %), in order.
    const BANDS: [(&str, i64, i64, i64); 5] =
        [("hostile", -50, 0, 0), ("unfriendly", -10, 115, 85), ("neutral", 9, 100, 100), ("friendly", 49, 95, 105), ("honoured", MAX, 90, 110)];

    pub fn key(faction_qid: &str) -> String {
        format!("{PREFIX}{}", faction_qid.replace(':', "."))
    }

    pub fn is_rep_key(k: &str) -> bool {
        k.starts_with(PREFIX)
    }

    pub fn faction_qid_of(k: &str) -> String {
        if !is_rep_key(k) {
            return String::new();
        }
        let parts: Vec<&str> = k[PREFIX.len()..].split('.').collect();
        if parts.len() != 2 || !json::id_ok(parts[0]) || !json::id_ok(parts[1]) {
            return String::new();
        }
        format!("{}:{}", parts[0], parts[1])
    }

    pub fn band(value: i64) -> &'static str {
        BANDS.iter().find(|b| value <= b.1).map(|b| b.0).unwrap_or("honoured")
    }

    pub fn buy_pct(band: &str) -> i64 {
        BANDS.iter().find(|b| b.0 == band).map(|b| b.2).unwrap_or(100)
    }

    pub fn sell_pct(band: &str) -> i64 {
        BANDS.iter().find(|b| b.0 == band).map(|b| b.3).unwrap_or(100)
    }

    /// The int flag of the faction (0 when missing, non-int or unknown).
    pub fn standing(d: &Data, gs: &GameState, faction_ref: &str) -> i64 {
        if faction_ref.is_empty() {
            return 0;
        }
        let hit = d.resolve(&gs.world_id, "factions", faction_ref);
        if hit.is_empty() {
            return 0;
        }
        match gs.flag(&key(&json::s(hit, "_qid"))) {
            Some(v) if json::is_int(v) => json::to_int(v),
            _ => 0,
        }
    }
}

// ---- DialogueRules -------------------------------------------------------------------------------------------

pub fn is_gone(d: &Data, gs: &GameState, character_ref: &str) -> bool {
    if character_ref.is_empty() {
        return false;
    }
    let c = d.resolve(&gs.world_id, "characters", character_ref);
    quest::any_holds(gs, json::get(c, "gone_flags"))
}

pub fn is_member(d: &Data, gs: &GameState, character_ref: &str) -> bool {
    if character_ref.is_empty() {
        return false;
    }
    let c = d.resolve(&gs.world_id, "characters", character_ref);
    if c.is_empty() {
        return false;
    }
    let (id, qid) = (json::s(c, "id"), json::s(c, "_qid"));
    gs.party.members.iter().any(|m| m.id == id || m.lore_ref == qid)
}

fn effects_of(choice: &Dict) -> Vec<Dict> {
    json::arr(&json::normalize(json::get(choice, "effects"))).iter().map(|x| json::dict(x).clone()).collect()
}

pub fn recruits_member(d: &Data, gs: &GameState, choice: &Dict) -> bool {
    effects_of(choice).iter().any(|fx| json::s(fx, "effect") == "recruit" && is_member(d, gs, &json::s(fx, "character")))
}

pub fn character_qid(d: &Data, gs: &GameState, r: &str) -> String {
    if r.is_empty() {
        return String::new();
    }
    json::s(d.resolve(&gs.world_id, "characters", r), "_qid")
}

pub fn faction_qid(d: &Data, gs: &GameState, character_ref: &str) -> String {
    if character_ref.is_empty() {
        return String::new();
    }
    let c = d.resolve(&gs.world_id, "characters", character_ref);
    let fref = json::s(c, "faction");
    if c.is_empty() || fref.is_empty() {
        return String::new();
    }
    json::s(d.resolve(&json::s_or(c, "_world", &gs.world_id), "factions", &fref), "_qid")
}

pub fn giver_qid(d: &Data, gs: &GameState, q: &Dict) -> String {
    character_qid(d, gs, &json::s(q, "giver"))
}

pub fn turn_in_qid(d: &Data, gs: &GameState, q: &Dict) -> String {
    character_qid(d, gs, &json::s(q, "turn_in"))
}

pub fn start_gold(d: &Data, world_id: &str) -> i64 {
    for w in d.lineage(world_id).iter().rev() {
        let v = json::i(d.manifest(w), "start_gold", -1);
        if v >= 0 {
            return v;
        }
    }
    0
}

/// The base line with the manifest `service_lines` merged along the lineage (nearest wins).
pub fn service_line(d: &Data, world_id: &str, id: &str) -> String {
    let mut text = d.service_text(id);
    for w in d.lineage(world_id) {
        if let Some(Value::String(s)) = json::dget(d.manifest(&w), "service_lines").get(id) {
            if !s.trim().is_empty() {
                text = s.clone();
            }
        }
    }
    text
}

/// `{token}` replaced by `values[token]`; unknown tokens stay.
pub fn fill(text: &str, values: &IndexMap<String, String>) -> String {
    let mut out = text.to_string();
    for (k, v) in values {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// The highest effective `stat` among the active members (built with gear and passives). RULESET1 9 D3 (the RULESET1-D
/// review): built as the world being played fights, so under its deck rule only slotted passives count (classic: the
/// deck rule is None, `build` exactly).
pub fn best_stat(d: &Data, gs: &GameState, stat: &str) -> i64 {
    let deck = rpgkit_rules::DeckRule::of(&d.ruleset(&gs.world_id));
    let mut best = 0;
    for m in gs.party.active_members() {
        if let Some(b) = progression::build_in(d, m, &gs.world_id, deck.as_ref()) {
            let ctx = b.unit.weapon_context();
            best = best.max(b.unit.stats.effective(stat, ctx.as_deref(), "") as i64);
        }
    }
    best
}

pub fn condition_holds(d: &Data, gs: &GameState, raw: &Dict) -> bool {
    let cond = json::normalize_dict(raw);
    match json::s_or(&cond, "kind", "flag").as_str() {
        "flag" => quest::condition_met(gs, &cond),
        "quest" => quest::status(d, gs, &json::s(&cond, "quest")) == json::s(&cond, "status"),
        "items" => gs.party.stash.get(&json::s(&cond, "item")).copied().unwrap_or(0) >= json::i(&cond, "count", 1),
        "gold" => gs.party.gold >= json::i(&cond, "min", 0),
        "stat" => best_stat(d, gs, &json::s(&cond, "stat")) >= json::i(&cond, "min", 0),
        _ => false,
    }
}

pub fn choice_open(d: &Data, gs: &GameState, choice: &Dict) -> bool {
    json::aget(choice, "requires").iter().all(|c| condition_holds(d, gs, json::dict(c)))
        && !json::aget(choice, "unless").iter().any(|c| condition_holds(d, gs, json::dict(c)))
}

pub const LOCKED_TEXT: &str = "Locked";
pub const NOT_NOW_TEXT: &str = "Not now";

pub fn lock_reason(d: &Data, gs: &GameState, choice: &Dict) -> String {
    for c in json::arr(&json::normalize(json::get(choice, "requires"))) {
        let cond = json::dict(c);
        if !condition_holds(d, gs, cond) {
            return needs_text(d, gs, cond);
        }
    }
    for c in json::arr(&json::normalize(json::get(choice, "unless"))) {
        if condition_holds(d, gs, json::dict(c)) {
            return NOT_NOW_TEXT.into();
        }
    }
    String::new()
}

fn faction_label(d: &Data, gs: &GameState, k: &str) -> String {
    let faction = d.resolve(&gs.world_id, "factions", &rep::faction_qid_of(k));
    if faction.is_empty() {
        k.to_string()
    } else {
        json::s_or(faction, "name", k)
    }
}

fn needs_text(d: &Data, gs: &GameState, cond: &Dict) -> String {
    match json::s_or(cond, "kind", "flag").as_str() {
        "items" => {
            let item = json::s(cond, "item");
            format!("Needs {} {}", json::i(cond, "count", 1), d.item_name(&item))
        }
        "gold" => terms::needs_currency(d, &gs.world_id, json::i(cond, "min", 0)),
        "stat" => format!("Needs {} {}", json::s(cond, "stat"), json::i(cond, "min", 0)),
        "quest" => {
            let q = json::s(cond, "quest");
            format!("Needs {} ({})", json::s_or(&quest::resolved(d, gs, &q), "name", &q), json::s(cond, "status"))
        }
        "flag" => {
            let k = json::s(cond, "key");
            let op = json::s_or(cond, "op", "==");
            if rep::is_rep_key(&k) && quest::ORDER_OPS.contains(&op.as_str()) {
                let who = faction_label(d, gs, &k);
                let band = rep::band(json::i(cond, "value", 0));
                return match op.as_str() {
                    "<" => format!("Needs {who} below {band}"),
                    "<=" => format!("Needs {who} at most {band}"),
                    _ => format!("Needs {who} {band}"),
                };
            }
            LOCKED_TEXT.into()
        }
        _ => LOCKED_TEXT.into(),
    }
}

pub fn stat_prefix(choice: &Dict) -> String {
    let mut out = String::new();
    for c in json::arr(&json::normalize(json::get(choice, "requires"))) {
        let cond = json::dict(c);
        if json::s_or(cond, "kind", "flag") == "stat" {
            out += &format!("[{} {}] ", json::s(cond, "stat"), json::i(cond, "min", 0));
        }
    }
    out
}

pub fn rep_prefix(d: &Data, gs: &GameState, choice: &Dict) -> String {
    let mut out = String::new();
    for c in json::arr(&json::normalize(json::get(choice, "requires"))) {
        let cond = json::dict(c);
        let k = json::s(cond, "key");
        if json::s_or(cond, "kind", "flag") != "flag" || !rep::is_rep_key(&k) || !quest::ORDER_OPS.contains(&json::s_or(cond, "op", "==").as_str()) {
            continue;
        }
        out += &format!("[{} {}] ", faction_label(d, gs, &k), rep::band(json::i(cond, "value", 0)));
    }
    out
}

/// Completes a ready quest from the journal: only when it has neither a giver nor a turn-in character.
pub fn complete_from_journal(d: &Data, gs: &mut GameState, r: &str, recipient: &str, picks: &[String]) -> Dict {
    let q = quest::resolved(d, gs, r);
    let mut who = if q.is_empty() { String::new() } else { turn_in_qid(d, gs, &q) };
    if who.is_empty() && !q.is_empty() {
        who = giver_qid(d, gs, &q);
    }
    if !who.is_empty() {
        let name = json::s_or(d.resolve(&gs.world_id, "characters", &who), "name", &who);
        let mut out = json::ok_result(false, &[format!("This quest is turned in to {name}.")]);
        out.insert("granted".into(), Value::Object(Dict::new()));
        out.insert("notes".into(), Value::Array(Vec::new()));
        return out;
    }
    quest::complete(d, gs, r, recipient, picks)
}

// ---- DialogueRunner ------------------------------------------------------------------------------------------

pub const ROOT: &str = "$root";
pub const OFFER_PREFIX: &str = "$offer:";

/// One conversation (never saved). The game state is passed to every call.
#[derive(Debug, Clone, Default)]
pub struct DialogueRunner {
    pub character_qid: String,
    pub character_name: String,
    pub errors: Vec<String>,
    pub closed: bool,
    pub start: String,
    pub services: bool,
    pub node_id: String,
    pub shop_qid: String,
    /// STORYUI1 B3: the board whose pane is open over the conversation ("" = none), like `shop_qid`.
    pub board_qid: String,
    tree_speaker: String,
    nodes: IndexMap<String, Dict>,
    /// Turn-in waiting for confirm: (quest qid, trailing effects, to).
    pending: Option<(String, Vec<Dict>, String)>,
    shop_to: String,
    board_to: String,
    pub notes: Vec<String>,
    /// CAMPAIGN1 W1: the campaign events of the effects applied since the last result (a node's effects when the
    /// conversation opens included); the next result carries them as `campaign`.
    pub campaign_events: Vec<Value>,
}

fn res_ok() -> Dict {
    json::ok_result(true, &[])
}

impl DialogueRunner {
    pub fn open(d: &Data, gs: &mut GameState, character_ref: &str) -> Self {
        let mut r = Self { services: true, ..Default::default() };
        let c = d.resolve(&gs.world_id, "characters", character_ref);
        if c.is_empty() {
            return r.refuse(&format!("Unknown character '{character_ref}'."));
        }
        r.character_qid = json::s(c, "_qid");
        r.character_name = json::s_or(c, "name", &json::s(c, "id"));
        if is_gone(d, gs, &r.character_qid) {
            let msg = format!("{} is gone.", r.character_name);
            return r.refuse(&msg);
        }
        let tree_ref = json::s(c, "dialogue");
        if tree_ref.is_empty() {
            r.start = ROOT.into();
            let root = json::obj([("id", ROOT.into()), ("text", service_line(d, &gs.world_id, "greeting").into()), ("choices", Value::Array(Vec::new()))]);
            r.nodes.insert(ROOT.into(), root);
        } else if let Err(why) = r.load_tree(d, gs, &tree_ref) {
            return r.refuse(&why);
        }
        let start = r.start.clone();
        r.show(d, gs, &start);
        r
    }

    /// CAMPAIGN1 T2: a queued `{dialogue, speaker}` scene: the tree `dialogue_ref` with `speaker_ref` as the one talked
    /// to ("" = none: the tree's and the nodes' own speakers), no services (a scene, not a visit). Refused for an
    /// unknown ref or a speaker who is gone, as `open`.
    pub fn open_scene(d: &Data, gs: &mut GameState, dialogue_ref: &str, speaker_ref: &str) -> Self {
        let mut r = Self::default();
        if !speaker_ref.is_empty() {
            let c = d.resolve(&gs.world_id, "characters", speaker_ref);
            if c.is_empty() {
                return r.refuse(&format!("Unknown character '{speaker_ref}'."));
            }
            r.character_qid = json::s(c, "_qid");
            r.character_name = json::s_or(c, "name", &json::s(c, "id"));
            if is_gone(d, gs, &r.character_qid) {
                let msg = format!("{} is gone.", r.character_name);
                return r.refuse(&msg);
            }
        }
        if let Err(why) = r.load_tree(d, gs, dialogue_ref) {
            return r.refuse(&why);
        }
        r.services = false;
        let start = r.start.clone();
        r.show(d, gs, &start);
        r
    }

    /// The nodes, start, services flag and speaker of dialogue tree `tree_ref`; Err(why) for an unknown tree or a
    /// missing start node.
    fn load_tree(&mut self, d: &Data, gs: &GameState, tree_ref: &str) -> Result<(), String> {
        let tree = d.resolve(&gs.world_id, "dialogues", tree_ref);
        if tree.is_empty() {
            return Err(format!("Unknown dialogue '{tree_ref}'."));
        }
        for n in json::arr(&json::normalize(json::get(tree, "nodes"))) {
            let node = json::dict(n).clone();
            self.nodes.insert(json::s(&node, "id"), node);
        }
        self.start = json::s(tree, "start");
        self.services = json::b(tree, "services", true);
        self.tree_speaker = character_qid(d, gs, &json::s(tree, "speaker"));
        if !self.nodes.contains_key(&self.start) {
            return Err(format!("Start node '{}' does not exist.", self.start));
        }
        Ok(())
    }

    fn refuse(mut self, message: &str) -> Self {
        self.errors.push(message.into());
        self.closed = true;
        self
    }

    fn line(&self, d: &Data, gs: &GameState, id: &str) -> String {
        service_line(d, &gs.world_id, id)
    }

    fn node(&self, d: &Data, gs: &GameState, id: &str) -> Dict {
        if let Some(qid) = id.strip_prefix(OFFER_PREFIX) {
            let q = quest::resolved(d, gs, qid);
            let mut text = json::s(&q, "description");
            if text.trim().is_empty() {
                text = json::s_or(&q, "name", qid);
            }
            let accept = json::obj([
                ("text", self.line(d, gs, "accept").into()),
                ("to", self.start.clone().into()),
                ("effects", Value::Array(vec![Value::Object(json::obj([("effect", "start_quest".into()), ("quest", qid.into())]))])),
                ("_quest", qid.into()),
            ]);
            let decline = json::obj([("text", self.line(d, gs, "decline").into()), ("to", self.start.clone().into()), ("_quest", qid.into())]);
            return json::obj([
                ("id", id.into()),
                ("text", text.into()),
                ("_quest", qid.into()),
                ("choices", Value::Array(vec![Value::Object(accept), Value::Object(decline)])),
            ]);
        }
        self.nodes.get(id).cloned().unwrap_or_default()
    }

    fn show(&mut self, d: &Data, gs: &mut GameState, id: &str) {
        self.node_id = id.into();
        let fx = effects_of(&self.node(d, gs, id));
        self.apply(d, gs, &fx);
    }

    fn choices_of(&self, d: &Data, gs: &GameState, id: &str) -> Vec<Dict> {
        let mut out: Vec<Dict> = json::arr(&json::normalize(json::get(&self.node(d, gs, id), "choices"))).iter().map(|c| json::dict(c).clone()).collect();
        if id == self.start && self.services {
            out.extend(self.service_choices(d, gs));
        }
        out
    }

    fn service_choices(&self, d: &Data, gs: &GameState) -> Vec<Dict> {
        let mut out = Vec::new();
        for qid in quest::quests_with(d, gs, "available") {
            let q = quest::resolved(d, gs, &qid);
            // STORYUI1 B3: a board quest is offered only on its board
            if giver_qid(d, gs, &q) == self.character_qid && !board::on_a_board(&q) {
                out.push(json::obj([
                    ("text", self.line(d, gs, "offer").into()),
                    ("to", format!("{OFFER_PREFIX}{qid}").into()),
                    ("_quest", qid.clone().into()),
                    ("_service", "offer".into()),
                ]));
            }
        }
        for qid in quest::quests_with(d, gs, "ready") {
            if turn_in_qid(d, gs, &quest::resolved(d, gs, &qid)) == self.character_qid {
                out.push(json::obj([
                    ("text", self.line(d, gs, "turn_in").into()),
                    ("to", self.start.clone().into()),
                    ("effects", Value::Array(vec![Value::Object(json::obj([("effect", "turn_in".into()), ("quest", qid.clone().into())]))])),
                    ("_quest", qid.clone().into()),
                    ("_service", "turn_in".into()),
                ]));
            }
        }
        // STORYUI1 B3: the keeper's board, after the turn-ins and before the shop, while it is open
        for b in board::boards(d, gs) {
            let bqid = json::s(b, "_qid");
            if board::keeper_qid(d, gs, b) == self.character_qid && board::is_open(d, gs, &bqid) {
                out.push(json::obj([
                    ("text", self.line(d, gs, rpgkit_data::BOARD_SERVICE).into()),
                    ("to", "".into()),
                    ("effects", Value::Array(vec![Value::Object(json::obj([("effect", "open_board".into()), ("board", bqid.clone().into())]))])),
                    ("_board", bqid.into()),
                    ("_service", rpgkit_data::BOARD_SERVICE.into()),
                ]));
            }
        }
        for s in d.merged_entries(&gs.world_id, "shops") {
            let sqid = json::s(s, "_qid");
            if character_qid(d, gs, &json::s(s, "keeper")) == self.character_qid && shop::is_open(d, gs, &sqid) {
                out.push(json::obj([
                    ("text", self.line(d, gs, "shop").into()),
                    ("to", "".into()),
                    ("effects", Value::Array(vec![Value::Object(json::obj([("effect", "open_shop".into()), ("shop", sqid.clone().into())]))])),
                    ("_shop", sqid.into()),
                    ("_service", "shop".into()),
                ]));
            }
        }
        out.push(json::obj([("text", self.line(d, gs, "farewell").into()), ("to", "".into()), ("_service", "farewell".into())]));
        out
    }

    /// (choice, locked) as the player sees them.
    fn visible(&self, d: &Data, gs: &GameState) -> Vec<(Dict, bool)> {
        let mut out = Vec::new();
        for c in self.choices_of(d, gs, &self.node_id) {
            if recruits_member(d, gs, &c) {
                continue;
            }
            let open = choice_open(d, gs, &c);
            if open || json::b(&c, "show_locked", false) {
                out.push((c, !open));
            }
        }
        out
    }

    fn tokens(&self, d: &Data, gs: &GameState, extra: &Dict) -> IndexMap<String, String> {
        let mut values = IndexMap::new();
        values.insert("npc".to_string(), self.character_name.clone());
        values.insert("gold".to_string(), gs.party.gold.to_string());
        let qid = json::s(extra, "_quest");
        if !qid.is_empty() {
            values.insert("quest".into(), json::s_or(&quest::resolved(d, gs, &qid), "name", &qid));
        }
        let sqid = json::s(extra, "_shop");
        if !sqid.is_empty() {
            values.insert("shop".into(), json::s_or(shop::shop_of(d, gs, &sqid), "name", &sqid));
        }
        values
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// CAMPAIGN1 W3: the effects of choice `index` as the player sees the choices (`view`'s order); empty for no such
    /// choice or a locked one. Read-only: the game asks before a choice whose `rest` would end the run.
    pub fn choice_effects(&self, d: &Data, gs: &GameState, index: i64) -> Vec<Dict> {
        if self.closed || index < 0 {
            return Vec::new();
        }
        match self.visible(d, gs).get(index as usize) {
            Some((c, false)) => effects_of(c),
            _ => Vec::new(),
        }
    }

    /// {speaker, speaker_name, node, text, choices: [{text, locked, reason, service}], end, next, shop, pending,
    /// faction, faction_name, band}, plus `board` while a board pane is open (STORYUI1 B3) and `expression` when the
    /// node names one (PORTRAITFX1 7: the speaker's face for the node).
    pub fn view(&self, d: &Data, gs: &GameState) -> Dict {
        let node = self.node(d, gs, &self.node_id);
        let mut speaker = character_qid(d, gs, &json::s(&node, "speaker"));
        if speaker.is_empty() {
            speaker = if self.tree_speaker.is_empty() { self.character_qid.clone() } else { self.tree_speaker.clone() };
        }
        let fq = faction_qid(d, gs, &self.character_qid);
        let mut choices = Vec::new();
        if !self.closed {
            for (c, locked) in self.visible(d, gs) {
                let text = rep_prefix(d, gs, &c) + &stat_prefix(&c) + &fill(&json::s(&c, "text"), &self.tokens(d, gs, &c));
                choices.push(Value::Object(json::obj([
                    ("text", text.into()),
                    ("locked", Value::Bool(locked)),
                    ("reason", if locked { lock_reason(d, gs, &c) } else { String::new() }.into()),
                    ("service", json::s(&c, "_service").into()),
                ])));
            }
        }
        let (faction_name, band) = if fq.is_empty() {
            (String::new(), String::new())
        } else {
            (json::s(d.resolve(&gs.world_id, "factions", &fq), "name"), rep::band(rep::standing(d, gs, &fq)).to_string())
        };
        let mut out = json::obj([
            ("speaker", speaker.clone().into()),
            ("speaker_name", json::s(d.resolve(&gs.world_id, "characters", &speaker), "name").into()),
            ("node", self.node_id.clone().into()),
            ("text", fill(&json::s(&node, "text"), &self.tokens(d, gs, &node)).into()),
            ("choices", Value::Array(choices)),
            ("end", Value::Bool(self.closed || json::b(&node, "end", false))),
            ("next", Value::Bool(!self.closed && !json::s(&node, "next").is_empty())),
            ("shop", self.shop_qid.clone().into()),
            ("pending", Value::Bool(self.pending.is_some())),
            ("faction", fq.into()),
            ("faction_name", faction_name.into()),
            ("band", band.into()),
        ]);
        if !self.board_qid.is_empty() {
            out.insert("board".into(), self.board_qid.clone().into());
        }
        // PORTRAITFX1 7: the speaker's face for this node, only when the node names one (a Godot node never did)
        let expression = json::s(&node, "expression");
        if !expression.is_empty() {
            out.insert("expression".into(), expression.into());
        }
        out
    }

    fn busy(&self) -> Option<&'static str> {
        if self.closed {
            Some("The conversation is over.")
        } else if self.pending.is_some() {
            Some("Confirm or cancel the turn-in first.")
        } else if !self.shop_qid.is_empty() {
            Some("Close the shop first.")
        } else if !self.board_qid.is_empty() {
            Some(board::BUSY_TEXT)
        } else {
            None
        }
    }

    fn with_notes(&mut self, mut res: Dict) -> Dict {
        let mut all = json::sget(&res, "notes");
        all.extend(self.notes.iter().cloned());
        res.insert("notes".into(), json::strs(&all));
        if !self.campaign_events.is_empty() {
            res.insert("campaign".into(), Value::Array(std::mem::take(&mut self.campaign_events)));
        }
        res
    }

    pub fn choose(&mut self, d: &Data, gs: &mut GameState, index: i64) -> Dict {
        self.notes.clear();
        let res = self.choose_inner(d, gs, index);
        self.with_notes(res)
    }

    fn choose_inner(&mut self, d: &Data, gs: &mut GameState, index: i64) -> Dict {
        if let Some(b) = self.busy() {
            return json::fail(b);
        }
        let visible = self.visible(d, gs);
        if index < 0 || index as usize >= visible.len() {
            return json::fail(&format!("There is no choice {index}."));
        }
        let (c, locked) = &visible[index as usize];
        if *locked {
            return json::fail("That choice is locked.");
        }
        let effects = effects_of(c);
        let problems = self.precheck(d, gs, &effects);
        if !problems.is_empty() {
            return json::ok_result(false, &problems);
        }
        if effects.first().is_some_and(|fx| json::s(fx, "effect") == "turn_in") {
            let qid = json::s(&quest::resolved(d, gs, &json::s(&effects[0], "quest")), "_qid");
            self.pending = Some((qid.clone(), effects[1..].to_vec(), json::s(c, "to")));
            let mut out = res_ok();
            out.insert("pending".into(), "turn_in".into());
            out.insert("quest".into(), qid.clone().into());
            out.insert("offers".into(), json::strs(&quest::offers(d, gs, &qid)));
            out.insert("picks_allowed".into(), int(quest::picks_allowed(d, gs, &qid)));
            return out;
        }
        let to = json::s(c, "to");
        self.apply(d, gs, &effects);
        let last = effects.last().cloned().unwrap_or_default();
        match json::s(&last, "effect").as_str() {
            "fight" => {
                self.closed = true;
                let enc = json::s(d.resolve(&gs.world_id, "encounters", &json::s(&last, "encounter")), "_qid");
                let mut out = res_ok();
                out.insert("fight".into(), Value::Object(json::obj([("source", "dialogue".into()), ("encounter", enc.into()), ("character", self.character_qid.clone().into())])));
                out
            }
            "open_shop" => {
                self.shop_qid = json::s(shop::shop_of(d, gs, &json::s(&last, "shop")), "_qid");
                self.shop_to = if to.is_empty() { self.node_id.clone() } else { to };
                let mut out = res_ok();
                out.insert("shop".into(), self.shop_qid.clone().into());
                out
            }
            // STORYUI1 4.7, B3: the board pane opens over the conversation, like the shop
            "open_board" => {
                self.board_qid = json::s(board::board_of(d, gs, &json::s(&last, "board")), "_qid");
                self.board_to = if to.is_empty() { self.node_id.clone() } else { to };
                let mut out = res_ok();
                out.insert("board".into(), self.board_qid.clone().into());
                out
            }
            // STORYUI1 4.7: the sequence started (in `apply`); the conversation closes and the game shows the story
            "sequence" => {
                self.closed = true;
                let mut out = res_ok();
                out.insert("end".into(), Value::Bool(true));
                out.insert("sequence".into(), gs.story.running.as_ref().map(|r| r.sequence.clone()).unwrap_or_default().into());
                out
            }
            _ => self.move_to(d, gs, &to),
        }
    }

    fn move_to(&mut self, d: &Data, gs: &mut GameState, to: &str) -> Dict {
        let mut out = res_ok();
        if to.is_empty() {
            self.closed = true;
            out.insert("end".into(), Value::Bool(true));
            return out;
        }
        self.show(d, gs, to);
        out.insert("node".into(), self.node_id.clone().into());
        out
    }

    fn name_of(&self, d: &Data, gs: &GameState, qid: &str) -> String {
        json::s_or(d.resolve(&gs.world_id, "characters", qid), "name", qid)
    }

    fn precheck(&self, d: &Data, gs: &GameState, effects: &[Dict]) -> Vec<String> {
        let mut out = Vec::new();
        let mut take_items: IndexMap<String, i64> = IndexMap::new();
        let mut take_gold = 0;
        for fx in effects {
            match json::s(fx, "effect").as_str() {
                "set_flag" => {
                    if !flag_key_ok(&json::s(fx, "key")) || !flag_value_ok(fx.get("value").unwrap_or(&Value::Bool(true))) {
                        out.push(format!("Flag '{}' can't be set.", json::s(fx, "key")));
                    } else if rpgkit_data::reserved_key(&json::s(fx, "key")) {
                        out.push(crate::campaign::reserved_text(&json::s(fx, "key")));
                    }
                }
                "add_flag" => {
                    let k = json::s(fx, "key");
                    if !flag_key_ok(&k) || !fx.get("delta").is_some_and(json::is_int) {
                        out.push(format!("Flag '{k}' can't be changed."));
                    } else if rpgkit_data::reserved_key(&k) {
                        out.push(crate::campaign::reserved_text(&k));
                    } else if !gs.flag(&k).map(json::is_int).unwrap_or(true) {
                        out.push(format!("Flag '{k}' is not a number."));
                    }
                }
                "start_quest" => {
                    let r = json::s(fx, "quest");
                    let st = quest::status(d, gs, &r);
                    if st != "available" {
                        out.push(format!("Quest '{r}' can't start (it is {st})."));
                    } else {
                        let giver = giver_qid(d, gs, &quest::resolved(d, gs, &r));
                        if !giver.is_empty() && giver != self.character_qid {
                            out.push(format!("Only {} gives this quest.", self.name_of(d, gs, &giver)));
                        }
                    }
                }
                "turn_in" => {
                    let r = json::s(fx, "quest");
                    let st = quest::status(d, gs, &r);
                    if st != "ready" {
                        out.push(format!("Quest '{r}' can't be turned in (it is {st})."));
                    } else {
                        let taker = turn_in_qid(d, gs, &quest::resolved(d, gs, &r));
                        if !taker.is_empty() && taker != self.character_qid {
                            out.push(format!("This quest is turned in to {}.", self.name_of(d, gs, &taker)));
                        }
                    }
                }
                "give" => {
                    for id in json::dget(fx, "items").keys() {
                        if !d.has("items", id) {
                            out.push(format!("Unknown item '{id}'."));
                        }
                    }
                }
                "take" => {
                    for (id, n) in json::ints(json::get(fx, "items")) {
                        *take_items.entry(id).or_insert(0) += n;
                    }
                    take_gold += json::i(fx, "gold", 0);
                }
                "open_shop" => {
                    if !shop::is_open(d, gs, &json::s(fx, "shop")) {
                        out.push("The shop is closed.".into());
                    }
                }
                "fight" => {
                    let enc = json::s(fx, "encounter");
                    if d.resolve(&gs.world_id, "encounters", &enc).is_empty() {
                        out.push(format!("Unknown encounter '{enc}'."));
                    } else {
                        // ROSTER1 BK6: the encounter's fewest heroes
                        let why = roster::party_min_text(d, gs, &enc);
                        if !why.is_empty() {
                            out.push(why);
                        }
                    }
                }
                // STORYUI1 4.7
                "open_board" => {
                    if !board::is_open(d, gs, &json::s(fx, "board")) {
                        out.push(board::CLOSED_TEXT.into());
                    }
                }
                "sequence" => {
                    let r = json::s(fx, "sequence");
                    if story::is_running(gs) {
                        out.push(story::RUNNING_TEXT.into());
                    } else if d.resolve(&gs.world_id, "sequences", &r).is_empty() {
                        out.push(format!("Unknown sequence '{r}'."));
                    }
                }
                // CAMPAIGN1 R2, R3: the rest rule's refusal (inert outside a campaign world)
                "rest" if gs.campaign.is_some() => {
                    let why = crate::campaign::purse::rest_refusal(d, gs);
                    if !why.is_empty() {
                        out.push(why.into());
                    }
                }
                "recruit" => {
                    let who = json::s(fx, "character");
                    let full = roster::full_text(gs);
                    if new_game::hero_from_character(d, &gs.world_id, &who).is_none() {
                        out.push(format!("'{who}' can't join the party."));
                    } else if is_member(d, gs, &who) {
                        out.push(format!("{} is already with you.", self.name_of(d, gs, &who)));
                    } else if roster::has_departed(d, gs, &who) {
                        // ROSTER1 L1: member ids are never reused
                        out.push(roster::gone_text(&self.name_of(d, gs, &who)));
                    } else if !full.is_empty() {
                        // ROSTER1 BK1, BK3: the bunk cap (the party max when the world sets no bunks)
                        out.push(full);
                    }
                }
                _ => {}
            }
        }
        for (id, n) in &take_items {
            if gs.party.stash.get(id).copied().unwrap_or(0) < *n {
                out.push(format!("Not enough {}.", d.item_name(id)));
            }
        }
        if take_gold > gs.party.gold {
            out.push(terms::not_enough_currency(d, &gs.world_id));
        }
        out
    }

    fn apply(&mut self, d: &Data, gs: &mut GameState, effects: &[Dict]) {
        let act = gs.campaign.as_ref().map(|c| c.act);
        apply_effects(d, gs, effects, &mut self.notes, &mut self.campaign_events);
        if !effects.is_empty() {
            let events = crate::campaign::after_effects(d, gs, act);
            self.campaign_events.extend(events);
        }
    }

    pub fn confirm_turn_in(&mut self, d: &Data, gs: &mut GameState, recipient: &str, picks: &[String]) -> Dict {
        self.notes.clear();
        let Some((qid, rest, to)) = self.pending.clone() else { return json::fail("No turn-in is waiting.") };
        let mut again = vec![json::obj([("effect", "turn_in".into()), ("quest", qid.clone().into())])];
        again.extend(rest.iter().cloned());
        let problems = self.precheck(d, gs, &again);
        if !problems.is_empty() {
            return json::ok_result(false, &problems);
        }
        let res = quest::complete(d, gs, &qid, recipient, picks);
        if !json::b(&res, "ok", false) {
            return json::ok_result(false, &json::sget(&res, "errors"));
        }
        self.apply(d, gs, &rest);
        self.pending = None;
        let mut moved = self.move_to(d, gs, &to);
        moved.insert("granted".into(), res.get("granted").cloned().unwrap_or(Value::Null));
        moved.insert("notes".into(), res.get("notes").cloned().unwrap_or(Value::Array(Vec::new())));
        self.with_notes(moved)
    }

    pub fn cancel(&mut self) -> Dict {
        self.pending = None;
        let mut out = res_ok();
        out.insert("node".into(), self.node_id.clone().into());
        out
    }

    /// STORYUI1 B11: closes the board pane back to the conversation (the node the choice led to).
    pub fn close_board(&mut self, d: &Data, gs: &mut GameState) -> Dict {
        self.notes.clear();
        let res = if self.board_qid.is_empty() {
            json::fail("No board is open.")
        } else {
            self.board_qid.clear();
            if self.board_to == self.node_id {
                let mut out = res_ok();
                out.insert("node".into(), self.node_id.clone().into());
                out
            } else {
                let to = self.board_to.clone();
                self.move_to(d, gs, &to)
            }
        };
        self.with_notes(res)
    }

    /// STORYUI1 B8: Accept on the open board: quest `r` must be on it; the `start_quest` path of a choice (the
    /// precheck: available, given by this character or by no one), then the log says "Quest taken: {name}.".
    /// {ok, errors, notes, quest, campaign?}.
    pub fn accept(&mut self, d: &Data, gs: &mut GameState, r: &str) -> Dict {
        self.notes.clear();
        if self.board_qid.is_empty() {
            return json::fail("No board is open.");
        }
        let q = quest::resolved(d, gs, r);
        if q.is_empty() || board::board_qid_of(d, gs, &q) != self.board_qid {
            return json::fail(&format!("Quest '{r}' is not on this board."));
        }
        let qid = json::s(&q, "_qid");
        let effects = vec![json::obj([("effect", "start_quest".into()), ("quest", qid.clone().into())])];
        let problems = self.precheck(d, gs, &effects);
        if !problems.is_empty() {
            return json::ok_result(false, &problems);
        }
        self.apply(d, gs, &effects);
        self.notes.push(format!("Quest taken: {}.", json::s_or(&q, "name", r)));
        let mut out = res_ok();
        out.insert("quest".into(), qid.into());
        self.with_notes(out)
    }

    pub fn close_shop(&mut self, d: &Data, gs: &mut GameState) -> Dict {
        self.notes.clear();
        let res = if self.shop_qid.is_empty() {
            json::fail("No shop is open.")
        } else {
            self.shop_qid.clear();
            if self.shop_to == self.node_id {
                let mut out = res_ok();
                out.insert("node".into(), self.node_id.clone().into());
                out
            } else {
                let to = self.shop_to.clone();
                self.move_to(d, gs, &to)
            }
        };
        self.with_notes(res)
    }

    pub fn advance(&mut self, d: &Data, gs: &mut GameState) -> Dict {
        self.notes.clear();
        let res = if let Some(b) = self.busy() {
            json::fail(b)
        } else {
            let node = self.node(d, gs, &self.node_id);
            let next = json::s(&node, "next");
            if !next.is_empty() {
                self.move_to(d, gs, &next)
            } else if json::b(&node, "end", false) {
                self.move_to(d, gs, "")
            } else {
                json::fail("Pick a choice.")
            }
        };
        self.with_notes(res)
    }
}

/// DialogueRunner.apply's body for one effect (STORYUI1 Q3: factored out with no change to the conversation), shared by
/// the conversation, story sequences' `effects` steps and campaign rules (CAMPAIGN1 T2): `set_flag`, `add_flag`,
/// `start_quest` (the giver check is the conversation's precheck), `give` (its notes), `take`, `recruit` (its note),
/// the story effects `vignette` (queued, W1), `sequence` (started, Q1; refused while one runs, the refusal a note) and
/// `open_board` (its first open sets the board's flag, B9; the runner opens the pane), `rest` (CAMPAIGN1 R2, its note;
/// the days it passes tick after the list), then the campaign effects (11.3; inert outside a campaign world). No
/// campaign tick: the caller ticks once after its list.
///
/// A rest's days run as their own days(n, rest) tick within the caller's next `campaign::tick` (CAMPAIGN1-T).
pub fn apply_effect(d: &Data, gs: &mut GameState, fx: &Dict, notes: &mut Vec<String>, events: &mut Vec<Value>) {
    match json::s(fx, "effect").as_str() {
        "set_flag" => {
            quest::set_flag(d, gs, &json::s(fx, "key"), fx.get("value").cloned().unwrap_or(Value::Bool(true)));
        }
        "add_flag" => {
            quest::add_flag(d, gs, &json::s(fx, "key"), json::i(fx, "delta", 0), json::get(fx, "min"), json::get(fx, "max"));
        }
        "start_quest" => {
            quest::start(d, gs, &json::s(fx, "quest"));
        }
        "give" => {
            let rewards = json::obj([("items", Value::Object(json::dget(fx, "items").clone())), ("gold", int(json::i(fx, "gold", 0)))]);
            let given = quest::reward_apply(d, gs, &rewards, "", &[]);
            notes.extend(json::sget(&given, "notes"));
        }
        "take" => {
            quest::take(d, gs, json::get(fx, "items"), json::i(fx, "gold", 0));
        }
        "recruit" => recruit(d, gs, &json::s(fx, "character"), fx.get("level").unwrap_or(&Value::Null), notes),
        "vignette" => {
            story::queue(d, gs, &json::s(fx, "vignette"));
        }
        "sequence" => {
            if let Err(why) = story::start_quiet(d, gs, &json::s(fx, "sequence"), notes, events) {
                notes.push(why);
            }
        }
        "open_board" => board::mark_opened(d, gs, &json::s(fx, "board")),
        // CAMPAIGN1 R2, R3: the rest and its note; its days run at the tick after the list (inert outside a campaign)
        "rest" => {
            let note = crate::campaign::purse::rest_effect(d, gs);
            if !note.is_empty() {
                notes.push(note);
            }
        }
        // CAMPAIGN1 11.3 (inert outside a campaign world)
        _ => {
            crate::campaign::apply_effect(d, gs, fx, events);
        }
    }
}

/// An effect list through `apply_effect`, in order, stopping once the run is over (CAMPAIGN1 F1: the first `end`
/// wins). The one list body of the conversation (`DialogueRunner::apply`), story sequences' `effects` steps and the
/// campaign's meter rules, missed `go_on` checkpoints and triggers (`campaign::apply_effects`, CAMPAIGN1 T2). No
/// campaign tick: the caller ticks once after its list.
pub fn apply_effects(d: &Data, gs: &mut GameState, effects: &[Dict], notes: &mut Vec<String>, events: &mut Vec<Value>) {
    for fx in effects {
        if gs.campaign.as_ref().is_some_and(crate::campaign::Campaign::is_over) {
            break;
        }
        apply_effect(d, gs, fx, notes, events);
    }
}

/// The `recruit` effect: the character joins at its `level` (ROSTER1 BK5; default the best active member's XP,
/// RECRUIT1). Outside a conversation (whose precheck refuses these) a character already in the party, a departed one
/// (L1) or a party with no free bunk (BK1, BK3) is a note instead. BK4: it deploys when `active_ids` is empty (the
/// first `max_active` fight) or has room; else it waits on the bench.
fn recruit(d: &Data, gs: &mut GameState, who: &str, level: &Value, notes: &mut Vec<String>) {
    let Some(mut hero) = new_game::hero_from_character(d, &gs.world_id, who) else { return };
    let name = progression::display_name(d, &hero);
    if is_member(d, gs, who) {
        notes.push(format!("{name} is already with you."));
        return;
    }
    if roster::has_departed(d, gs, who) {
        notes.push(roster::gone_text(&name));
        return;
    }
    let full = roster::full_text(gs);
    if !full.is_empty() {
        notes.push(full);
        return;
    }
    hero.total_xp = roster::join_xp(d, &gs.party, hero.total_xp, level);
    if !gs.party.active_ids.is_empty() && (gs.party.active_ids.len() as i64) < gs.party.max_active {
        gs.party.active_ids.push(hero.id.clone());
    }
    gs.party.members.push(hero);
    notes.push(format!("{name} joins the party."));
}

/// After a dialogue fight: BattleBridge.apply (REWARD1: `tally` from the battle, None = settled without one), plus the
/// wake on lost. {ok, errors, xp, advanced, events, rewards, gold?}.
pub fn after_fight(d: &Data, gs: &mut GameState, fight: &Dict, outcome: &str, tally: Option<&crate::battle::Tally>) -> Dict {
    let applied = crate::battle::settle(d, gs, &json::s(fight, "encounter"), outcome, tally);
    if !json::b(&applied, "ok", false) {
        let mut out = json::ok_result(false, &json::sget(&applied, "errors"));
        out.insert("xp".into(), Value::Object(Dict::new()));
        out.insert("advanced".into(), Value::Array(Vec::new()));
        out.insert("events".into(), Value::Array(Vec::new()));
        return out;
    }
    let mut events = Vec::new();
    let mut woke_campaign = Vec::new();
    if outcome == "lost" {
        let woke = crate::explore::wake(d, gs, "");
        events.extend(json::aget(&woke, "events").iter().cloned());
        woke_campaign = json::aget(&woke, "campaign").to_vec();
    }
    let mut out = json::ok_result(true, &json::sget(&applied, "errors"));
    out.insert("xp".into(), applied["xp"].clone());
    out.insert("advanced".into(), applied["advanced"].clone());
    out.insert("events".into(), Value::Array(events));
    for (k, v) in crate::battle::result_keys(&applied) {
        out.insert(k, v);
    }
    // CAMPAIGN1 R4: a retreat's tick events follow the battle's
    crate::campaign::merge_events(&mut out, woke_campaign);
    out
}
