//! Bunk talks (BUNKS1 BK5-BK8, spec BUNKS1 section 2; engine-only, no Godot original;
//! story-free): which talks of the world kind `talks` wait, for whom, and playing one. The pair counter
//! (`PartyState::together`) grows by one for each pair of members who came home from a won or withdrawn battle they
//! fought side by side (`roster::settle_party` calls `count_together`). A talk waits while both its members live in
//! bunks, its flag conditions hold, the pair counter reaches its `min_together` and its seen flag is unset. Written
//! (`pair`) talks name two characters; a `template` talk pairs each generated recruit (a member made from a recruit
//! template) whose temperament (else origin) it names with the first founder (a member who is not generated) of the
//! roster. Rows are tried in table order: the first waiting talk of a member is the one shown (the oldest first).

use crate::data::Data;
use crate::dialogue::DialogueRunner;
use crate::progression;
use crate::state::{GameState, UnitState};
use aegis_core::json;
use rpgkit_data::talk::{pair_key, seen_key, TalkRow, TALKS_KIND, TOKEN_A, TOKEN_B};
use serde_json::Value;

/// A talk waiting to be played: the row, the two members (a written pair in its order; a template's founder `a` and
/// recruit `b`), the seen flag it sets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Waiting {
    pub id: String,
    pub qid: String,
    pub a: String,
    pub b: String,
    pub seen: String,
    pub dialogue: String,
    pub reward: String,
    pub template: bool,
}

impl Waiting {
    /// Whether member `id` is one of the two.
    pub fn has(&self, id: &str) -> bool {
        self.a == id || self.b == id
    }

    /// The other member of the two (`a` when `id` is neither).
    pub fn other(&self, id: &str) -> &str {
        if self.a == id {
            &self.b
        } else {
            &self.a
        }
    }
}

/// BK6: after a battle with `outcome`, each pair of the members `home` (fielded and still in the party) counts one
/// battle together; only a won (`won`), withdrawn (`fled`) or slipped (`escaped`, ENRAGE2) battle counts.
pub fn count_together(gs: &mut GameState, outcome: &str, home: &[String]) {
    if !["won", "fled", crate::battle::ESCAPED].contains(&outcome) {
        return;
    }
    for (i, a) in home.iter().enumerate() {
        for b in &home[i + 1..] {
            if a != b {
                *gs.party.together.entry(pair_key(a, b)).or_insert(0) += 1;
            }
        }
    }
}

/// BK6: the battles members `a` and `b` fought side by side (the pair counter; 0 when none).
pub fn together(gs: &GameState, a: &str, b: &str) -> i64 {
    gs.party.together.get(&pair_key(a, b)).copied().unwrap_or(0)
}

/// The world's talk rows as the played world sees them (lineage, overrides), table order: (qid, row).
pub fn rows(d: &Data, gs: &GameState) -> Vec<(String, TalkRow)> {
    d.merged_entries(&gs.world_id, TALKS_KIND).into_iter().map(|e| (json::s(e, "_qid"), TalkRow::from_obj(e))).collect()
}

/// BK7: a generated recruit (made from a recruit template; a named recruit is a character).
pub fn is_generated(m: &UnitState) -> bool {
    !m.template.is_empty()
}

/// Whether member `m` lives in a bunk (talks are bunk talks: crypt members have none).
fn in_bunk(m: &UnitState) -> bool {
    m.housing() == rpgkit_data::HOUSING[0]
}

/// BK7: the founder template talks pair with: the first member of the roster who is not generated and lives in a bunk.
pub fn founder(gs: &GameState) -> Option<&UnitState> {
    gs.party.members.iter().find(|m| !is_generated(m) && in_bunk(m))
}

/// The bunk member who plays character `character_ref` of the played world (by member id or character qid).
fn member_of<'a>(d: &Data, gs: &'a GameState, character_ref: &str) -> Option<&'a UnitState> {
    let c = d.resolve(&gs.world_id, "characters", character_ref);
    if c.is_empty() {
        return None;
    }
    let (id, qid) = (json::s(c, "id"), json::s(c, "_qid"));
    gs.party.members.iter().find(|m| in_bunk(m) && (m.lore_ref == qid || (m.lore_ref.is_empty() && m.id == id)))
}

/// BK7: whether template row `t` is picked for recruit `m` (its temperament, else its origin).
pub fn template_fits(t: &TalkRow, m: &UnitState) -> bool {
    (!t.temperament.is_empty() && t.temperament == m.temperament) || (!t.origin.is_empty() && t.origin == m.origin)
}

/// BK8: every waiting talk, table order (a template's recruits in roster order).
pub fn waiting(d: &Data, gs: &GameState) -> Vec<Waiting> {
    let mut out = Vec::new();
    for (qid, row) in rows(d, gs) {
        if row.dialogue.is_empty() || !crate::quest::all_hold(gs, &row.requires_flags) {
            continue;
        }
        let mut push = |a: &UnitState, b: &UnitState, recruit: &str| {
            let seen = seen_key(&row.id, recruit);
            if a.id != b.id && together(gs, &a.id, &b.id) >= row.min_together && !gs.flag(&seen).is_some_and(|v| *v != Value::Bool(false)) {
                out.push(Waiting { id: row.id.clone(), qid: qid.clone(), a: a.id.clone(), b: b.id.clone(), seen, dialogue: row.dialogue.clone(), reward: row.reward.clone(), template: row.template });
            }
        };
        if row.template {
            let Some(f) = founder(gs) else { continue };
            for m in gs.party.members.iter().filter(|m| is_generated(m) && in_bunk(m) && template_fits(&row, m)) {
                push(f, m, &m.id);
            }
        } else if let [a, b] = row.pair.as_slice() {
            if let (Some(a), Some(b)) = (member_of(d, gs, a), member_of(d, gs, b)) {
                push(a, b, "");
            }
        }
    }
    out
}

/// BK8: the talk shown for member `id` (its first waiting talk; None when none).
pub fn waiting_for(d: &Data, gs: &GameState, id: &str) -> Option<Waiting> {
    waiting(d, gs).into_iter().find(|w| w.has(id))
}

/// BK8: plays talk `w`: its dialogue opens as a scene (no services; `{a}` and `{b}` the two names), then its seen flag
/// is set and its reward goes to the stash. A refused dialogue (unknown, broken) sets and gives nothing: the runner
/// comes back closed with its errors. Returns the runner and the reward given ("" = none).
pub fn play(d: &Data, gs: &mut GameState, w: &Waiting) -> (DialogueRunner, String) {
    let name = |gs: &GameState, id: &str| gs.party.member(id).map(|m| progression::display_name(d, m)).unwrap_or_default();
    let (a, b) = (name(gs, &w.a), name(gs, &w.b));
    let world = aegis_content::split_qid(&w.qid).map(|(world, _)| world).unwrap_or_default();
    let dialogue = if world.is_empty() || w.dialogue.contains(':') { w.dialogue.clone() } else { format!("{world}:{}", w.dialogue) };
    let mut r = DialogueRunner::open_scene(d, gs, &dialogue, "");
    if r.closed {
        return (r, String::new());
    }
    r.tokens_extra.insert(TOKEN_A.into(), a);
    r.tokens_extra.insert(TOKEN_B.into(), b);
    gs.set_flag(&w.seen, Value::Bool(true));
    let mut reward = String::new();
    if !w.reward.is_empty() && d.cat.items.contains_key(&w.reward) {
        gs.party.add_to_stash(&w.reward, 1);
        reward = w.reward.clone();
    }
    (r, reward)
}
