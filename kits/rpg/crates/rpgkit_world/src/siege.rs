//! Sieges (PR6-9, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` section 5, B6; engine-only, no Godot original; story-free):
//! the siege runner. The effect `start_siege` opens a siege row (`rpgkit_data::siege`) as the saved `SiegeState`
//! (`GameState.siege`, the key `siege` only while one runs); the game fights its rooms in order (`fight`, runner source
//! `FIGHT_SOURCE`, settled by `after_fight`). Between rooms the state carries the party: exact HP (`carry_hp`: no rally
//! after a won room), the heroes left downed (`carry_downed`: not fielded until `triage` revives them), the pouches
//! (`carry_pouch`: no stash top-up after a room, `roster::settle_party` and `battle::begin_fight` ask `holds_pouch`;
//! `restock` is the row's explicit re-stock), and no rest (`no_camp`, `camp_refusal`). A lost or fled room ends the
//! siege: its facility drops one level (never below the first) and the `loss_effects` run; the last room won runs the
//! `win_effects`. With no siege running every hook is inert.

use crate::data::Data;
use crate::state::GameState;
use aegis_core::json::{self, int, Dict};
use rpgkit_data::facility::{FACILITIES_KIND, LEVEL_MIN};
use rpgkit_data::siege::{SiegeDef, SIEGES_KIND};
use serde_json::Value;

/// The runner source of a siege room's fight (`battle::finish_world`).
pub const FIGHT_SOURCE: &str = "siege";
/// The refusals.
pub const NO_CAMP_TEXT: &str = "No rest during a siege.";
pub const RUNNING_TEXT: &str = "A siege is already under way.";
pub const NONE_TEXT: &str = "No siege is under way.";
pub const NO_RESTOCK_TEXT: &str = "No re-stock left.";
pub const FIRST_ROOM_TEXT: &str = "Nothing to re-stock before the first room.";
pub const NOT_DOWNED_TEXT: &str = "That fighter is not down.";
pub const NO_REVIVE_TEXT: &str = "That item cannot revive.";
pub const ALL_DOWN_TEXT: &str = "Every fighter is down.";

/// The siege under way (saved as the key `siege`): the row's qid, the room to fight next (0-based), the re-stocks left
/// and the members left downed (fielded again only once triaged).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SiegeState {
    pub siege: String,
    pub room: i64,
    pub restocks: i64,
    pub downed: Vec<String>,
}

impl SiegeState {
    pub fn to_value(&self) -> Value {
        Value::Object(json::obj([
            ("siege", self.siege.clone().into()),
            ("room", int(self.room)),
            ("restocks", int(self.restocks)),
            ("downed", json::strs(&self.downed)),
        ]))
    }

    /// A saved block (problems go to `problems`; a bad block reads None).
    pub fn from_value(raw: &Value, problems: &mut Vec<String>) -> Option<Self> {
        let Some(o) = json::normalize(raw).as_object().cloned() else {
            problems.push("siege must be an object {siege, room, restocks, downed}.".into());
            return None;
        };
        let siege = json::s(&o, "siege");
        if siege.is_empty() {
            problems.push("siege.siege must name the siege.".into());
            return None;
        }
        Some(Self { siege, room: json::i(&o, "room", 0).max(0), restocks: json::i(&o, "restocks", 0).max(0), downed: json::sget(&o, "downed") })
    }
}

/// Siege row `r` as the played world sees it (None when unknown).
pub fn def_of(d: &Data, gs: &GameState, r: &str) -> Option<SiegeDef> {
    if r.is_empty() {
        return None;
    }
    let row = d.resolve(&gs.world_id, SIEGES_KIND, r);
    (!row.is_empty()).then(|| SiegeDef::from_obj(row))
}

/// The siege under way with its row (None when none runs or its row is gone).
pub fn current(d: &Data, gs: &GameState) -> Option<(SiegeState, SiegeDef)> {
    let st = gs.siege.clone()?;
    let def = def_of(d, gs, &st.siege)?;
    Some((st, def))
}

/// Whether a siege is under way.
pub fn running(gs: &GameState) -> bool {
    gs.siege.is_some()
}

/// Member `id` was left downed in an earlier room (not fielded until triaged).
pub fn is_downed(gs: &GameState, id: &str) -> bool {
    gs.siege.as_ref().is_some_and(|s| s.downed.iter().any(|m| m == id))
}

/// `no_camp`: the rest refusal while a siege runs ("" = none).
pub fn camp_refusal(d: &Data, gs: &GameState) -> &'static str {
    match current(d, gs) {
        Some((_, def)) if def.no_camp => NO_CAMP_TEXT,
        _ => "",
    }
}

/// `carry_pouch`: the pouches are not topped up from the stash now. `after` true: at a room's settle (every room but
/// the last); false: at a room's start (every room but the first).
pub fn holds_pouch(d: &Data, gs: &GameState, after: bool) -> bool {
    match current(d, gs) {
        Some((st, def)) if def.carry_pouch => {
            if after {
                st.room + 1 < def.rooms.len() as i64
            } else {
                st.room > 0
            }
        }
        _ => false,
    }
}

/// `start_siege`: opens siege `r` at its first room ("" = started, else why not).
pub fn start(d: &Data, gs: &mut GameState, r: &str) -> String {
    if running(gs) {
        return RUNNING_TEXT.into();
    }
    let Some(def) = def_of(d, gs, r) else { return format!("Unknown siege '{r}'.") };
    if def.rooms.is_empty() {
        return format!("Siege '{r}' has no rooms.");
    }
    let qid = json::s(d.resolve(&gs.world_id, SIEGES_KIND, r), "_qid");
    gs.siege = Some(SiegeState { siege: qid, room: 0, restocks: def.restocks, downed: Vec::new() });
    String::new()
}

/// The world fight of the room to fight next: `{source: "siege", encounter (qid), siege, room}`; None when no siege
/// runs.
pub fn fight(d: &Data, gs: &GameState) -> Option<Dict> {
    let (st, def) = current(d, gs)?;
    let room = def.rooms.get(st.room as usize)?;
    let qid = json::s_or(d.resolve(&gs.world_id, "encounters", &room.encounter), "_qid", &room.encounter);
    Some(json::obj([("source", FIGHT_SOURCE.into()), ("encounter", qid.into()), ("siege", st.siege.clone().into()), ("room", int(st.room))]))
}

/// Why the next room can't start now ("" = it can): no siege, or every active member left downed.
pub fn fight_refusal(gs: &GameState) -> &'static str {
    match gs.siege.as_ref() {
        None => NONE_TEXT,
        Some(_) if gs.party.active_members().iter().all(|m| is_downed(gs, &m.id)) => ALL_DOWN_TEXT,
        Some(_) => "",
    }
}

/// Facility `r` (as the played world resolves it) drops one level, never below the first. Returns its name when it
/// dropped.
fn damage_facility(d: &Data, gs: &mut GameState, r: &str) -> Option<String> {
    let fac = d.resolve(&gs.world_id, FACILITIES_KIND, r);
    let qid = json::s(fac, "_qid");
    let now = crate::facility::level(gs, &qid);
    if qid.is_empty() || now <= LEVEL_MIN {
        return None;
    }
    if now - 1 <= LEVEL_MIN {
        gs.facilities.shift_remove(&qid);
    } else {
        gs.facilities.insert(qid.clone(), now - 1);
    }
    Some(json::s_or(fac, "name", r))
}

/// After a siege room's fight: settled like a dialogue fight (`dialogue::after_fight`: `battle::settle`, the wake on a
/// loss), then the siege moves on. Won, rooms left: `carry_hp` writes each fielded member's exact end HP back (no
/// rally) and `carry_downed` keeps each one left at 0 HP downed; the next room is up. Won, last room: the siege ends and
/// its `win_effects` run. Lost or fled: the siege ends, the room's facility drops a level and the `loss_effects` run.
/// The result carries `siege` {id, name, room, rooms, state: next | won | lost, lines}.
pub fn after_fight(d: &Data, gs: &mut GameState, fight: &Dict, outcome: &str, tally: Option<&crate::battle::Tally>) -> Dict {
    let mut out = crate::dialogue::after_fight(d, gs, fight, outcome, tally);
    let Some((mut st, def)) = current(d, gs) else { return out };
    let room = def.rooms.get(st.room as usize).cloned().unwrap_or_default();
    let mut lines = Vec::new();
    let last = st.room + 1 >= def.rooms.len() as i64;
    let state = if outcome == "won" && !last {
        for e in tally.map_or(&[][..], |t| &t.party[..]) {
            let Some(m) = gs.party.member_mut(&e.member) else { continue };
            if e.alive && def.carry_hp {
                m.hp_lost = (e.max_hp - e.hp.min(e.max_hp)).max(0);
            } else if !e.alive && def.carry_downed {
                m.hp_lost = e.max_hp.max(0);
                if !st.downed.contains(&e.member) {
                    st.downed.push(e.member.clone());
                }
            }
        }
        st.room += 1;
        lines.push(format!("{} holds.", if room.name.is_empty() { "The room" } else { &room.name }));
        "next"
    } else {
        let won = outcome == "won";
        let act_before = gs.campaign.as_ref().map(|c| c.act);
        gs.siege = None;
        let mut events = Vec::new();
        if won {
            lines.push(format!("{} is broken.", if def.name.is_empty() { "The siege" } else { &def.name }));
        } else {
            lines.push(format!("{} is lost.", if room.name.is_empty() { "The room" } else { &room.name }));
            if let Some(name) = damage_facility(d, gs, &room.facility) {
                lines.push(format!("{name} is damaged."));
            }
        }
        let notes = crate::campaign::apply_effects(d, gs, if won { &def.win_effects } else { &def.loss_effects }, &mut events);
        lines.extend(notes);
        events.extend(crate::campaign::after_effects(d, gs, act_before));
        crate::campaign::merge_events(&mut out, events);
        if won {
            "won"
        } else {
            "lost"
        }
    };
    let room_no = st.room;
    if state == "next" {
        gs.siege = Some(st);
    }
    out.insert(
        "siege".into(),
        Value::Object(json::obj([
            ("id", json::s(fight, "siege").into()),
            ("name", def.name.clone().into()),
            ("room", int(room_no)),
            ("rooms", int(def.rooms.len() as i64)),
            ("state", state.into()),
            ("lines", json::strs(&lines)),
        ])),
    );
    out
}

/// Yields the room to fight next without a battle (every fighter down): settled as a lost room (`after_fight` with no
/// tally).
pub fn forfeit(d: &Data, gs: &mut GameState) -> Dict {
    let Some(fight) = fight(d, gs) else { return json::fail(NONE_TEXT) };
    after_fight(d, gs, &fight, "lost", None)
}

/// `restocks`: tops the active members' pouches up from the stash once between rooms (`roster::auto_fill`): {ok,
/// errors, short (the "Stash short" lines), restocks (left)}.
pub fn restock(d: &Data, gs: &mut GameState) -> Dict {
    let Some(st) = gs.siege.clone() else { return json::fail(NONE_TEXT) };
    if st.room == 0 {
        return json::fail(FIRST_ROOM_TEXT);
    }
    if st.restocks <= 0 {
        return json::fail(NO_RESTOCK_TEXT);
    }
    let ids: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).filter(|id| !is_downed(gs, id)).collect();
    let world = gs.world_id.clone();
    let short = crate::roster::auto_fill(d, gs, &world, &ids);
    if let Some(s) = gs.siege.as_mut() {
        s.restocks -= 1;
    }
    let mut out = json::ok_result(true, &[]);
    out.insert("short".into(), json::strs(&short));
    out.insert("restocks".into(), int(st.restocks - 1));
    out
}

/// Triage between rooms: a stash item that revives (`ItemDef::revive`) brings downed member `member` back at the item's
/// heal (at least 1, at most its max HP); the item is spent. {ok, errors, hp}.
pub fn triage(d: &Data, gs: &mut GameState, member: &str, item: &str) -> Dict {
    if !running(gs) {
        return json::fail(NONE_TEXT);
    }
    if !is_downed(gs, member) {
        return json::fail(NOT_DOWNED_TEXT);
    }
    let Some(def) = d.cat.items.get(item).filter(|i| i.revive) else { return json::fail(NO_REVIVE_TEXT) };
    if gs.party.stash.get(item).copied().unwrap_or(0) <= 0 {
        return json::fail(&format!("Not enough {}.", d.item_name(item)));
    }
    let heal = i64::from(def.heal);
    let world = gs.world_id.clone();
    let Some(max) = gs.party.member(member).map(|m| crate::progression::max_hp_of(d, m, &world)) else { return json::fail(NOT_DOWNED_TEXT) };
    let hp = heal.clamp(1, max.max(1));
    gs.party.remove_from_stash(item, 1);
    if let Some(m) = gs.party.member_mut(member) {
        m.hp_lost = (max - hp).max(0);
    }
    if let Some(s) = gs.siege.as_mut() {
        s.downed.retain(|m| m != member);
    }
    let mut out = json::ok_result(true, &[]);
    out.insert("hp".into(), int(hp));
    out
}

/// The stash items that can triage, (id, name, count) in stash order.
pub fn triage_items(d: &Data, gs: &GameState) -> Vec<(String, String, i64)> {
    gs.party.stash.iter().filter(|(id, n)| **n > 0 && d.cat.items.get(id.as_str()).is_some_and(|i| i.revive)).map(|(id, n)| (id.clone(), d.item_name(id), *n)).collect()
}

/// The siege pane's reading: {id, name, room, rooms: [{name, state: cleared | next | ahead}], restocks, can_restock,
/// downed: [{member, name}], items: [{id, name, count}], no_camp, refusal}; None when no siege runs.
pub fn view(d: &Data, gs: &GameState) -> Option<Value> {
    let (st, def) = current(d, gs)?;
    let rooms: Vec<Value> = def
        .rooms
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let state = match (i as i64).cmp(&st.room) {
                std::cmp::Ordering::Less => "cleared",
                std::cmp::Ordering::Equal => "next",
                std::cmp::Ordering::Greater => "ahead",
            };
            let name = if r.name.is_empty() { format!("Room {}", i + 1) } else { r.name.clone() };
            Value::Object(json::obj([("name", name.into()), ("state", state.into())]))
        })
        .collect();
    let downed: Vec<Value> = st
        .downed
        .iter()
        .filter_map(|id| gs.party.member(id).map(|m| Value::Object(json::obj([("member", id.clone().into()), ("name", crate::progression::display_name(d, m).into())]))))
        .collect();
    let items: Vec<Value> = triage_items(d, gs).into_iter().map(|(id, name, n)| Value::Object(json::obj([("id", id.into()), ("name", name.into()), ("count", int(n))]))).collect();
    Some(Value::Object(json::obj([
        ("id", st.siege.clone().into()),
        ("name", (if def.name.is_empty() { def.id.clone() } else { def.name.clone() }).into()),
        ("room", int(st.room)),
        ("rooms", Value::Array(rooms)),
        ("restocks", int(st.restocks)),
        ("can_restock", Value::Bool(st.room > 0 && st.restocks > 0)),
        ("downed", Value::Array(downed)),
        ("items", Value::Array(items)),
        ("no_camp", Value::Bool(def.no_camp)),
        ("refusal", fight_refusal(gs).into()),
    ])))
}
