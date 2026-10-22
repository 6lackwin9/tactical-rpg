//! PLAYUI1-M: `explore::npc_services` (games/emberwake/docs/specs/PLAYUI1.md 2.2 A), checked against what the dialogue
//! itself offers: the quest offer, turn-in and shop service lines of a conversation, and the recruit choices of a
//! character's dialogue tree. Read-only over the shipped data (no content is written); the expectations come from the
//! dialogue rules, not from content numbers.

use aegis_core::json::{self, Dict};
use emberwake_world::dialogue::{self, DialogueRunner};
use emberwake_world::explore::{npc_services, SERVICES};
use emberwake_world::{new_game, quest, Data, GameState};
use serde_json::Value;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// A new game of every playable world.
fn games(d: &Data) -> Vec<GameState> {
    new_game::playable_worlds(d).iter().map(|w| new_game::create_for_world(d, w, 1).expect("a new game")).collect()
}

/// Every available quest started and marked ready to turn in.
fn all_ready(d: &Data, gs: &mut GameState) {
    for q in quest::quests_with(d, gs, "available") {
        assert!(quest::start(d, gs, &q).is_empty(), "{q} starts");
        if let Some(Value::Object(st)) = gs.quests.get_mut(&q) {
            st.insert("state".into(), "ready".into());
        }
    }
}

fn characters(d: &Data, gs: &GameState) -> Vec<Dict> {
    d.merged_entries(&gs.world_id, "characters").into_iter().cloned().collect()
}

/// Whether the character's conversation opens with the dialogue's service lines (no tree, or a tree with services on).
fn has_service_lines(d: &Data, gs: &GameState, c: &Dict) -> bool {
    let tree = json::s(c, "dialogue");
    tree.is_empty() || json::b(d.resolve(&gs.world_id, "dialogues", &tree), "services", true)
}

/// The service lines of the conversation's first node (`_service` of each choice the player sees).
fn service_lines(d: &Data, gs: &GameState, qid: &str) -> Vec<String> {
    let mut scratch = gs.clone();
    let r = DialogueRunner::open(d, &mut scratch, qid);
    if r.closed {
        return Vec::new();
    }
    json::aget(&r.view(d, &scratch), "choices").iter().map(|c| json::s(json::dict(c), "service")).filter(|s| !s.is_empty()).collect()
}

#[test]
fn offers_turn_ins_and_shops_match_the_conversation() {
    let d = data();
    let mut seen = 0;
    for start in games(&d) {
        let mut ready = start.clone();
        all_ready(&d, &mut ready);
        for gs in [start, ready] {
            for c in characters(&d, &gs) {
                let qid = json::s(&c, "_qid");
                let services = npc_services(&d, &gs, &qid);
                let order: Vec<&str> = SERVICES.iter().copied().filter(|s| services.contains(s)).collect();
                assert_eq!(services, order, "{qid}: one of each, in badge order");
                if dialogue::is_gone(&d, &gs, &qid) || !has_service_lines(&d, &gs, &c) {
                    continue;
                }
                let lines = service_lines(&d, &gs, &qid);
                for s in ["turn_in", "offer", "shop"] {
                    assert_eq!(services.contains(&s), lines.iter().any(|l| l == s), "{qid} in {}: {s} (services {services:?}, lines {lines:?})", gs.world_id);
                }
                seen += services.iter().filter(|s| **s != "recruit").count();
            }
        }
    }
    assert!(seen > 0, "no character offered a quest, a turn-in or a shop in any playable world: the check saw nothing");
}

/// The characters a recruit choice of `c`'s dialogue tree names, each with whether that choice's conditions hold.
fn recruit_targets(d: &Data, gs: &GameState, c: &Dict) -> Vec<(String, bool)> {
    let tree = d.resolve(&gs.world_id, "dialogues", &json::s(c, "dialogue"));
    let mut out = Vec::new();
    for n in json::arr(&json::normalize(json::get(tree, "nodes"))) {
        for ch in json::arr(&json::normalize(json::get(json::dict(n), "choices"))) {
            for f in json::arr(&json::normalize(json::get(json::dict(ch), "effects"))) {
                let f = json::dict(f);
                if json::s(f, "effect") == "recruit" {
                    out.push((json::s(f, "character"), dialogue::choice_open(d, gs, json::dict(ch))));
                }
            }
        }
    }
    out
}

#[test]
fn a_recruiter_offers_recruits_until_they_all_joined() {
    let d = data();
    let mut recruiters = 0;
    for gs in games(&d) {
        for c in characters(&d, &gs) {
            let qid = json::s(&c, "_qid");
            let targets = recruit_targets(&d, &gs, &c);
            let outside = |gs: &GameState| targets.iter().any(|(t, open)| *open && !dialogue::is_member(&d, gs, t));
            if targets.is_empty() {
                assert!(!npc_services(&d, &gs, &qid).contains(&"recruit"), "{qid} has no recruit choice");
                continue;
            }
            recruiters += 1;
            assert_eq!(npc_services(&d, &gs, &qid).contains(&"recruit"), outside(&gs), "{qid}: recruits {targets:?}");
            let mut joined = gs.clone();
            for (t, _) in &targets {
                if let Some(hero) = new_game::hero_from_character(&d, &joined.world_id, t) {
                    joined.party.members.push(hero);
                }
            }
            assert_eq!(npc_services(&d, &joined, &qid).contains(&"recruit"), outside(&joined), "{qid}: everyone who can join has joined");
        }
    }
    assert!(recruiters > 0, "no dialogue tree recruits anyone: the check saw nothing");
}
