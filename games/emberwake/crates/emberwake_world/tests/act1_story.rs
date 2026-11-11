//! PR5-38 ACT1-STORY acceptance on the shipped data, read only: the `sunk_ward` world plays from New game through the
//! Prologue into the hub, and its campaign graph routes Tier 1 -> Tier 2 -> Interlude -> M5 through the kit's own
//! paths: the world's `start_sequence` (story steps, story fights settled `won` with no battle, as the debug fight
//! popup settles them), the hub's services (board, shops, recipes), the world map (`travel`), the quests taken from
//! the board, the location encounter markers (settled through the explore runner), the campaign triggers and their
//! queued scenes (`campaign::pending`, a scene dialogue played through `DialogueRunner::open_scene`). Ids are the
//! story's (the row names them); counts and orders are read from the shipped data.

use aegis_core::json::{self, Dict};
use emberwake_data::Step;
use emberwake_world::dialogue::DialogueRunner;
use emberwake_battle::encounter::PartyFate;
use emberwake_world::{battle, board, campaign, dialogue, explore, new_game, quest, recipe, roster, shop, story, travel, Data, GameState};
use serde_json::Value;
use std::path::Path;

const WORLD: &str = "sunk_ward";
const HUB: &str = "sluice_undercroft";
const BOARD: &str = "talia_board";

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// `sunk_ward:<id>`.
fn q(id: &str) -> String {
    format!("{WORLD}:{id}")
}

/// `sunk_ward.<name>`, the story's flag keys.
fn flag(name: &str) -> String {
    format!("{WORLD}.{name}")
}

fn ok(r: &Dict) -> bool {
    json::b(r, "ok", false)
}

/// The world's own entry `id` of `kind`.
fn entry<'a>(d: &'a Data, kind: &str, id: &str) -> &'a Dict {
    let e = d.resolve(WORLD, kind, id);
    assert!(!e.is_empty(), "the shipped {WORLD} has {kind} '{id}'");
    e
}

/// What one run of `play_sequence` saw: the vignettes and the encounters, in order, where it ended, and per vignette
/// the speakers of the `line` cards that played (their `when` holding, `story::card_now`).
#[derive(Debug, Default)]
struct Played {
    vignettes: Vec<String>,
    fights: Vec<String>,
    to: String,
    speakers: Vec<(String, Vec<String>)>,
}

/// The speakers (qids) of vignette `qid`'s `line` cards that play now.
fn line_speakers(d: &Data, gs: &GameState, qid: &str) -> Vec<String> {
    let v = story::vignette(d, gs, qid);
    let home = json::s_or(v, "_world", WORLD);
    json::aget(v, "cards").iter().filter_map(|c| story::card_now(d, gs, &home, json::dict(c))).filter(|c| json::s(c, "style") == "line").map(|c| json::s(&c, "speaker")).collect()
}

/// Plays the running sequence to its end as the story stage does: a vignette ends (`step_done`), a fight is won with
/// no battle (`battle::finish_world` by the fight's `story` source), the `enter` / `end` step finishes it.
fn play_sequence(d: &Data, gs: &mut GameState) -> Played {
    let mut out = Played::default();
    for _ in 0..200 {
        match story::current(d, gs) {
            Some(Step::Vignette { vignette }) => {
                out.speakers.push((vignette.clone(), line_speakers(d, gs, &vignette)));
                let r = story::step_done(d, gs);
                assert!(ok(&r), "{vignette}: {r:?}");
                out.vignettes.push(vignette);
            }
            Some(Step::Fight { encounter, .. }) => {
                let fight = story::fight(d, gs).expect("a story fight at a fight step");
                let r = battle::finish_world(d, gs, &json::s(&fight, "source"), &fight, "won", None);
                assert!(ok(&r), "{encounter}: {r:?}");
                out.fights.push(encounter);
            }
            Some(Step::Enter { .. }) | Some(Step::End { .. }) => {
                let r = story::finish(d, gs);
                assert!(ok(&r), "the sequence's last step: {r:?}");
                out.to = json::s(&r, "to");
                assert!(!story::is_running(gs));
                return out;
            }
            other => panic!("an unplayable step {other:?}"),
        }
    }
    panic!("the sequence did not end in 200 steps: {out:?}");
}

/// The queued campaign items (`then` of the fired triggers), head first, as (kind, ref) pairs.
fn pending(gs: &GameState) -> Vec<(String, String)> {
    let Some(c) = gs.campaign.as_ref() else { return Vec::new() };
    c.pending
        .iter()
        .map(|item| {
            let kind = ["dialogue", "vignette", "battle"].into_iter().find(|k| item.contains_key(*k)).unwrap_or("");
            (kind.to_string(), json::s(item, kind))
        })
        .collect()
}

/// The effect names of the data choice behind visible choice `view` of node `node` in dialogue `dlg` (matched by
/// text: the Act 1 scene choices carry no tokens).
fn choice_effects(d: &Data, dlg: &str, node: &str, view: &Dict) -> Vec<String> {
    let tree = entry(d, "dialogues", dlg);
    let n = json::aget(tree, "nodes").iter().map(json::dict).find(|n| json::s(n, "id") == node).cloned().unwrap_or_default();
    json::aget(&n, "choices")
        .iter()
        .map(json::dict)
        .find(|c| json::s(c, "text") == json::s(view, "text"))
        .map(|c| json::aget(c, "effects").iter().map(|fx| json::s(json::dict(fx), "effect")).collect())
        .unwrap_or_default()
}

/// Plays every queued scene dialogue as the run-over layer does (pop, `open_scene`; each opens with no refusal) and
/// answers it as a player must (a scene has no farewell reply): at each node the first open choice with an effect
/// named in `want` (a `recruit`, the `sequence` that starts the Interlude), else the first open choice, until the
/// conversation closes or reaches an end node. Returns (dialogue ref, the effect names chosen) per scene.
fn play_scenes(d: &Data, gs: &mut GameState, want: &[&str]) -> Vec<(String, Vec<String>)> {
    let mut played = Vec::new();
    while let Some(item) = campaign::pending_head(gs).cloned() {
        let dlg = json::s(&item, "dialogue");
        assert!(!dlg.is_empty(), "only scene dialogues are queued by the Act 1 triggers: {item:?}");
        campaign::pop_pending(gs);
        let mut r = DialogueRunner::open_scene(d, gs, &dlg, &json::s(&item, "speaker"));
        assert!(r.errors.is_empty(), "{dlg} opens: {:?}", r.errors);
        let mut chosen = Vec::new();
        for _ in 0..10 {
            let view = r.view(d, gs);
            let choices: Vec<Dict> = json::aget(&view, "choices").iter().map(|c| json::dict(c).clone()).collect();
            if r.closed || choices.is_empty() {
                break;
            }
            let open: Vec<usize> = (0..choices.len()).filter(|i| !json::b(&choices[*i], "locked", false)).collect();
            let node = r.node_id.clone();
            let k = open.iter().copied().find(|i| choice_effects(d, &dlg, &node, &choices[*i]).iter().any(|e| want.contains(&e.as_str()))).or(open.first().copied());
            let k = k.unwrap_or_else(|| panic!("{dlg} {node}: no open choice"));
            chosen.extend(choice_effects(d, &dlg, &node, &choices[k]));
            let res = r.choose(d, gs, k as i64);
            assert!(ok(&res), "{dlg} choice {k}: {res:?}");
        }
        played.push((dlg, chosen));
    }
    played
}

/// Whether scene `dlg` was played.
fn saw(scenes: &[(String, Vec<String>)], dlg: &str) -> bool {
    scenes.iter().any(|(s, _)| s == dlg)
}

/// The node path from the party's node to `to` over the open routes (their `requires_flags` holding), every node
/// counted (the world map shows a node once a visited node has an open leg to it; the party walks node by node).
fn open_path(d: &Data, gs: &GameState, to: &str) -> Option<Vec<String>> {
    let from = travel::current_node(d, gs);
    let legs = travel::legs(d, gs, WORLD);
    let (_, routes) = travel::best_path(&legs, &travel::nodes(d, WORLD), &from, &q(to))?;
    let mut at = from;
    let mut hops = Vec::new();
    for rq in routes {
        let r = travel::route_in(d, WORLD, &rq);
        at = if json::s(&r, "_from") == at { json::s(&r, "_to") } else { json::s(&r, "_from") };
        hops.push(at.clone());
    }
    Some(hops)
}

/// Whether `to` can be reached from the party's node over open routes.
fn reachable(d: &Data, gs: &GameState, to: &str) -> bool {
    open_path(d, gs, to).is_some_and(|p| !p.is_empty())
}

/// Travels to `to` over the world map one node at a time (each next node is known once the one before is visited); a
/// road fight on the way is won (the travel runner) and the trip goes on.
fn go(d: &Data, gs: &mut GameState, to: &str) {
    let hops = open_path(d, gs, to).unwrap_or_else(|| panic!("no open way to {to}"));
    for hop in hops {
        assert!(travel::known(d, gs).contains(&hop), "the world map shows {hop} on the way to {to}");
        let mut arrived = false;
        for _ in 0..10 {
            let r = travel::travel(d, gs, &hop);
            assert!(ok(&r), "travel to {hop}: {r:?}");
            match json::s(&r, "status").as_str() {
                "arrived" => {}
                "fight" => {
                    let fight = json::dict(&r["fight"]).clone();
                    let w = battle::finish_world(d, gs, "travel", &fight, "won", None);
                    assert!(ok(&w), "a road fight on the way to {hop}: {w:?}");
                }
                other => panic!("travel to {hop} stopped: {other} {r:?}"),
            }
            if gs.location_id == hop {
                arrived = true;
                break;
            }
        }
        assert!(arrived, "travel to {hop} did not arrive");
    }
    assert_eq!(gs.location_id, q(to));
}

/// The present encounter markers of the party's location: (marker id, encounter qid).
fn encounter_markers(d: &Data, gs: &GameState) -> Vec<(String, String)> {
    let loc = explore::location(d, gs).clone();
    let lq = json::s(&loc, "_qid");
    explore::markers_of(&loc)
        .into_iter()
        .filter(|m| json::s(m, "kind") == "encounter" && explore::presence(d, gs, &lq, m) == "present")
        .map(|m| (json::s(&m, "id"), json::s(d.resolve(&gs.world_id, "encounters", &json::s(&m, "encounter")), "_qid")))
        .collect()
}

/// The encounter of quest `quest`'s first step as the quest resolves now (its variants applied), as a qid.
fn quest_encounter(d: &Data, gs: &GameState, quest: &str) -> String {
    let rq = quest::resolved(d, gs, &q(quest));
    let step = json::aget(&rq, "steps").first().map(json::dict).cloned().unwrap_or_default();
    json::s(d.resolve(WORLD, "encounters", &json::s(&step, "encounter")), "_qid")
}

/// Takes board quest `quest`, travels to its step's location and wins its encounter at the location's marker (the
/// explore runner, as walking into the marker does); the quest's objective is met. `stayer`: the member whose end in
/// the fight is `Stayed` (a stay-behind fight's winch holder), the others' ends as the default tally gives them; None =
/// the default tally (no battle).
fn run_quest(d: &Data, gs: &mut GameState, quest: &str, stayer: Option<&str>) {
    assert_eq!(board::board_qid_of(d, gs, &quest::resolved(d, gs, &q(quest))), q(BOARD), "{quest} is on Talia's board");
    assert_eq!(quest::status(d, gs, &q(quest)), "available", "{quest}");
    assert_eq!(quest::start(d, gs, &q(quest)), Vec::<String>::new(), "{quest} is taken");
    let rq = quest::resolved(d, gs, &q(quest));
    let step = json::aget(&rq, "steps").first().map(json::dict).cloned().unwrap_or_default();
    let at = json::s(d.resolve(WORLD, "locations", &json::s(&step, "location")), "id");
    go(d, gs, &at);
    let enc = quest_encounter(d, gs, quest);
    let (marker, _) = encounter_markers(d, gs).into_iter().find(|(_, e)| *e == enc).unwrap_or_else(|| panic!("{at} shows {quest}'s encounter {enc}"));
    let fight = json::obj([("source", "marker".into()), ("encounter", enc.clone().into()), ("marker", marker.into())]);
    let tally = stayer.map(|id| {
        let world = battle::fight_world(d, gs, &enc);
        let mut t = battle::Tally::default_for(d, &gs.world_id.clone(), &enc, "won");
        t.party = roster::default_ends(d, gs, &world, "won");
        let me = t.party.iter_mut().find(|e| e.member == id).unwrap_or_else(|| panic!("{id} is fielded in {enc}"));
        me.fate = Some(PartyFate::Stayed);
        t.world = world;
        t
    });
    let r = battle::finish_world(d, gs, "explore", &fight, "won", tally.as_ref());
    assert!(ok(&r), "{quest} {enc}: {r:?}");
    assert!(["ready", "completed"].contains(&quest::status(d, gs, &q(quest)).as_str()), "{quest} after its fight: {}", quest::status(d, gs, &q(quest)));
    assert!(!encounter_markers(d, gs).iter().any(|(_, e)| *e == enc), "{enc} leaves {at} once won");
}

/// A New game in `sunk_ward` played through the `new_game` sequence into the hub.
fn after_prologue(d: &Data) -> (GameState, Played) {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("a new sunk_ward game");
    assert_eq!(story::start_sequence(d, WORLD), "new_game", "the manifest's start sequence");
    assert!(story::is_running(&gs), "a new game opens on the Prologue");
    let played = play_sequence(d, &mut gs);
    (gs, played)
}

/// Tier 1 (M1, M2) and Tier 2 (M3, M4) won from the hub; the scenes the triggers queue are played, except the ones
/// M4's fight queues (the Interlude's scene stays queued for the caller).
fn after_tier2(d: &Data) -> GameState {
    let (mut gs, _) = after_prologue(d);
    for m in ["q_a1_m1", "q_a1_m2", "q_a1_m3"] {
        run_quest(d, &mut gs, m, None);
        play_scenes(d, &mut gs, PICK);
    }
    run_quest(d, &mut gs, "q_a1_m4", None);
    gs
}

/// The Interlude played: M4's scenes are answered (Elyse's recruit, Sabina's dose: its first choice that starts a
/// sequence starts `seq_interlude`), which runs to the hub.
fn after_interlude(d: &Data) -> GameState {
    let mut gs = after_tier2(d);
    let scenes = play_scenes(d, &mut gs, PICK);
    assert!(saw(&scenes, "dlg_int_dose"), "{scenes:?}");
    assert_eq!(gs.story.running.as_ref().map(|r| r.sequence.clone()), Some(q("seq_interlude")), "the dose starts the Interlude sequence");
    let played = play_sequence(d, &mut gs);
    assert_eq!(gs.location_id, q(HUB), "the Interlude ends in the hub ({played:?})");
    gs
}

/// The scene effects a player picks first: a recruit, and the sequence that starts the Interlude.
const PICK: &[&str] = &["recruit", "sequence"];

/// vg_debate_3's founder lines: (the founders whose line card is gated on `crucible_stayed` != that founder, as the
/// data writes it; the speakers whose lines played in `played`).
fn debate3_founders(d: &Data, played: &Played) -> (Vec<String>, Vec<String>) {
    let gated: Vec<String> = json::aget(entry(d, "vignettes", "vg_debate_3"), "cards")
        .iter()
        .map(json::dict)
        .filter_map(|c| {
            let w = json::aget(c, "when").iter().map(json::dict).find(|w| json::s(w, "key") == flag("crucible_stayed") && json::s(w, "op") == "!=")?;
            (json::s(w, "value") == json::s(c, "speaker")).then(|| json::s(c, "speaker"))
        })
        .collect();
    let shown = played.speakers.iter().find(|(v, _)| *v == q("vg_debate_3")).map(|(_, s)| s.clone()).expect("seq_after_m5 plays vg_debate_3");
    (gated, shown)
}

/// The recipes a character offers now, as ids.
fn offered(d: &Data, gs: &GameState, npc: &str) -> Vec<String> {
    recipe::offered(d, gs, &q(npc)).into_iter().map(|r| r.id.clone()).collect()
}

/// New game -> Prologue -> hub: the `new_game` sequence's vignettes and fights in order, its recruits, the Debate 1
/// flag, the hub; before Debate 1 the hub's board and the two gated shops are closed, after it they are open and the
/// world map knows the hub; the PK2 recipes are not offered yet.
#[test]
fn new_game_plays_the_prologue_into_the_hub_and_opens_its_services() {
    let d = data();
    let seq = entry(&d, "sequences", "new_game");
    let steps: Vec<Dict> = json::aget(seq, "steps").iter().map(|s| json::dict(s).clone()).collect();
    let want_fights: Vec<String> = steps.iter().filter(|s| json::s(s, "kind") == "fight").map(|s| q(&json::s(s, "encounter"))).collect();
    let want_vignettes: Vec<String> = steps.iter().filter(|s| json::s(s, "kind") == "vignette").map(|s| q(&json::s(s, "vignette"))).collect();
    let recruits: Vec<String> = steps
        .iter()
        .filter(|s| json::s(s, "kind") == "effects")
        .flat_map(|s| json::aget(s, "effects").iter().map(|fx| json::dict(fx).clone()).collect::<Vec<_>>())
        .filter(|fx| json::s(fx, "effect") == "recruit")
        .map(|fx| json::s(&fx, "character"))
        .collect();
    let enter = steps.last().expect("steps");
    assert_eq!((json::s(enter, "kind"), json::s(enter, "location")), ("enter".into(), HUB.into()), "the Prologue enters the hub");

    // before the Prologue ends: no Debate 1, the board and the gated shops closed
    let fresh = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
    assert!(!fresh.flag_true(&flag("debate1_seen")));
    assert!(!board::is_open(&d, &fresh, &q(BOARD)), "the board waits for Debate 1");
    for sh in ["sabina_slab", "berek_hearth"] {
        assert!(!shop::is_open(&d, &fresh, &q(sh)), "{sh} waits for Debate 1");
    }

    let (gs, played) = after_prologue(&d);
    assert_eq!(played.fights, want_fights, "the Prologue's fights in order");
    assert_eq!(played.vignettes, want_vignettes, "the Prologue's vignettes in order");
    assert_eq!(played.to, "map");
    let start_party = json::sget(d.manifest(WORLD), "start_party");
    let ids: Vec<String> = gs.party.members.iter().map(|m| m.id.clone()).collect();
    let want: Vec<String> = start_party.iter().chain(&recruits).cloned().collect();
    assert_eq!(ids, want, "Kaelen and the Prologue's recruits");
    assert!(gs.flag_true(&flag("debate1_seen")), "Debate 1 seen");
    assert_eq!(gs.location_id, q(HUB));
    assert!(!story::is_running(&gs));

    // the hub: its NPCs and their services
    let hub = entry(&d, "locations", HUB);
    let npcs: Vec<String> = explore::markers_of(hub)
        .iter()
        .filter(|m| json::s(m, "kind") == "npc" && explore::presence(&d, &gs, &q(HUB), m) == "present")
        .map(|m| json::s(m, "character"))
        .collect();
    let keeper = json::s(entry(&d, "boards", BOARD), "keeper");
    assert!(npcs.contains(&keeper), "Talia stands in the hub");
    assert!(board::is_open(&d, &gs, &q(BOARD)), "Talia's board opens after Debate 1");
    assert!(explore::npc_services(&d, &gs, &q(&keeper)).contains(&"board"), "Talia offers the board");
    let on_board = board::quests(&d, &gs, &q(BOARD));
    for m in ["q_a1_m1", "q_a1_m2"] {
        assert!(on_board.contains(&q(m)), "{m} on the board");
        assert_eq!(quest::status(&d, &gs, &q(m)), "available", "Tier 1: {m}");
    }
    for m in ["q_a1_m3", "q_a1_m4", "q_a1_m5"] {
        assert_eq!(quest::status(&d, &gs, &q(m)), "locked", "{m} waits");
    }
    for sh in ["sabina_slab", "berek_hearth"] {
        assert!(shop::is_open(&d, &gs, &q(sh)), "{sh} opens after Debate 1");
        let k = json::s(entry(&d, "shops", sh), "keeper");
        assert!(npcs.contains(&k), "{sh}'s keeper {k} stands in the hub");
        assert!(explore::npc_services(&d, &gs, &q(&k)).contains(&"shop"), "{k} offers the shop");
        assert!(!shop::offers(&d, &gs, &q(sh)).is_empty(), "{sh} has stock");
    }
    for npc in ["sabina_kress", "berek_dorn"] {
        assert_eq!(offered(&d, &gs, npc), Vec::<String>::new(), "{npc}: no PK2 recipe before M5");
        assert!(!explore::npc_services(&d, &gs, &q(npc)).contains(&"recipe"));
    }

    // the world map: the hub is a node; Tier 1 is reachable, Tier 2 is not
    assert_eq!(travel::current_node(&d, &gs), q(HUB));
    let view = travel::view(&d, &gs);
    assert!(!json::aget(&view, "nodes").is_empty(), "the world map lists nodes: {view:?}");
    assert!(travel::known(&d, &gs).contains(&q("drowned_runoff_grate")), "the map shows the way out of the hub");
    for t1 in ["alleyway_junction_7", "ash_crawl_crossroads"] {
        assert!(reachable(&d, &gs, t1), "Tier 1 {t1} is reachable");
    }
    for t2 in ["fourth_kiln", "tallow_vats", "crucible_watchpost", "slag_gate"] {
        assert!(!reachable(&d, &gs, t2), "{t2} is not reachable from the hub yet");
    }
}

/// Tier 1 -> Tier 2 -> Interlude: Tier 2 stays out of reach until both M1 and M2 are won (their triggers set
/// `m1_done` / `m2_done`); after M3 and M4 the Interlude trigger sets `interlude_started` and queues Sabina's dose
/// scene; the scene starts `seq_interlude`, which sets `interlude_done` and opens the convoy route to the Crucible.
#[test]
fn tier_one_then_tier_two_then_the_interlude() {
    let d = data();
    let (mut gs, _) = after_prologue(&d);
    run_quest(&d, &mut gs, "q_a1_m1", None);
    assert!(gs.flag_true(&flag("m1_done")), "t_m1_won");
    play_scenes(&d, &mut gs, PICK);
    for t2 in ["fourth_kiln", "tallow_vats"] {
        assert!(!reachable(&d, &gs, t2), "{t2}: M1 alone does not open Tier 2");
        let r = travel::travel(&d, &mut gs, &q(t2));
        assert!(!ok(&r), "{t2}: {r:?}");
    }
    assert_eq!(quest::status(&d, &gs, &q("q_a1_m3")), "locked");
    run_quest(&d, &mut gs, "q_a1_m2", None);
    assert!(gs.flag_true(&flag("m2_done")), "t_m2_won");
    let scenes = play_scenes(&d, &mut gs, PICK);
    assert!(saw(&scenes, "dlg_post_m2"), "Maren's follow-up scene: {scenes:?}");
    assert!(gs.flag_true(&flag("m2_counted")), "Maren's scene is answered");
    for t2 in ["fourth_kiln", "tallow_vats"] {
        assert!(reachable(&d, &gs, t2), "{t2}: Tier 2 opens after M1 and M2");
    }
    for t3 in ["crucible_watchpost", "slag_gate"] {
        assert!(!reachable(&d, &gs, t3), "{t3} waits for the Interlude");
    }
    for m in ["q_a1_m3", "q_a1_m4"] {
        assert_eq!(quest::status(&d, &gs, &q(m)), "available", "Tier 2: {m}");
    }
    assert_eq!(quest::status(&d, &gs, &q("q_a1_m5")), "locked");

    let before = gs.party.members.len();
    run_quest(&d, &mut gs, "q_a1_m3", None);
    assert!(gs.flag_true(&flag("m3_done")));
    let scenes = play_scenes(&d, &mut gs, PICK);
    assert!(scenes.iter().any(|(s, fx)| s == "dlg_post_m3" && fx.iter().any(|e| e == "recruit")), "Dunstan's recruit choice: {scenes:?}");
    assert!(dialogue::is_member(&d, &gs, "dunstan") && gs.flag_true(&flag("dunstan_joined")), "Dunstan joins through dlg_post_m3");
    assert_eq!(gs.party.members.len(), before + 1, "Dunstan takes the next bunk");
    assert!(!gs.flag_true(&flag("interlude_started")), "M3 alone does not start the Interlude");
    run_quest(&d, &mut gs, "q_a1_m4", None);
    assert!(gs.flag_true(&flag("m4_done")));
    assert!(gs.flag_true(&flag("interlude_started")), "t_interlude after M3 and M4");
    let queued = pending(&gs);
    assert!(queued.contains(&("dialogue".into(), "dlg_int_dose".into())), "Sabina's dose scene is queued: {queued:?}");

    let scenes = play_scenes(&d, &mut gs, PICK);
    assert!(scenes.iter().any(|(s, fx)| s == "dlg_post_m4" && fx.iter().any(|e| e == "recruit")), "Elyse's recruit choice: {scenes:?}");
    assert!(dialogue::is_member(&d, &gs, "elyse") && gs.flag_true(&flag("elyse_joined")), "Elyse joins through dlg_post_m4");
    assert_eq!(gs.party.members.len(), before + 2, "Elyse takes the next bunk");
    assert!(scenes.iter().any(|(s, fx)| s == "dlg_int_dose" && fx.iter().any(|e| e == "sequence")), "the dose choice: {scenes:?}");
    assert!(gs.flag_true(&flag("sabina_dose")), "the dose is given");
    assert_eq!(gs.story.running.as_ref().map(|r| r.sequence.clone()), Some(q("seq_interlude")));
    let want_fights: Vec<String> =
        json::aget(entry(&d, "sequences", "seq_interlude"), "steps").iter().map(json::dict).filter(|s| json::s(s, "kind") == "fight").map(|s| q(&json::s(s, "encounter"))).collect();
    let played = play_sequence(&d, &mut gs);
    assert_eq!(played.fights, want_fights, "the Interlude's fights");
    assert!(gs.flag_true(&flag("interlude_done")), "seq_interlude sets interlude_done");
    assert_eq!(gs.location_id, q(HUB));
    let convoy = travel::route_in(&d, WORLD, &q("convoy_track"));
    assert!(travel::is_open(&gs, &convoy), "the convoy route opens");
    assert!(reachable(&d, &gs, "crucible_watchpost"), "the Crucible is reachable after the Interlude");
    assert!(!reachable(&d, &gs, "slag_gate"), "the Slag Gate waits for M5");
    assert_eq!(quest::status(&d, &gs, &q("q_a1_m5")), "available", "M5 opens");
}

/// M5 with Hollis's overlook: the overlook point at the Crucible sets `hollis_overlook`, the quest swaps to the overlook
/// encounter and the location shows only its marker; winning it sets `m5_done`, runs `seq_after_m5` (which opens the
/// PK2 recipes at the Slab and the Hearth) and opens the high gorge to the Slag Gate.
#[test]
fn m5_with_the_overlook_opens_the_gate_and_the_pk2_recipes() {
    let d = data();
    let mut gs = after_interlude(&d);
    assert_eq!(quest_encounter(&d, &gs, "q_a1_m5"), q("enc_evt_06"), "M5 without the overlook");
    go(&d, &mut gs, "crucible_watchpost");
    let loc = explore::location(&d, &gs).clone();
    let overlook = explore::markers_of(&loc)
        .into_iter()
        .find(|m| json::s(m, "kind") == "point" && json::aget(m, "sets_flags").iter().any(|f| json::s(json::dict(f), "key") == flag("hollis_overlook")))
        .expect("the Crucible has the overlook point");
    assert_eq!(explore::presence(&d, &gs, &q("crucible_watchpost"), &overlook), "present", "the overlook opens after the Interlude");
    // walk onto the point (`explore::step`) from an open tile beside it
    let t = explore::template_of(&d, &loc).clone();
    let at = explore::marker_pos(&t, &overlook, gs.explore_int("loc_steps"));
    let dir = explore::DIRS.into_iter().find(|dd| explore::is_open(&t, at.0 - dd.0, at.1 - dd.1)).expect("an open tile beside the overlook");
    gs.position = (at.0 - dir.0, at.1 - dir.1);
    let r = explore::step(&d, &mut gs, dir);
    assert!(ok(&r) && gs.position == at, "the party steps onto the overlook: {r:?}");
    let set = json::aget(&r, "events").iter().map(json::dict).any(|e| json::s(e, "kind") == "flags_set" && json::s(e, "marker") == json::s(&overlook, "id"));
    assert!(set, "the overlook's flags are set by the step: {r:?}");
    assert!(gs.flag_true(&flag("hollis_overlook")));
    assert_eq!(explore::presence(&d, &gs, &q("crucible_watchpost"), &overlook), "absent", "the overlook is used once");
    assert_eq!(quest_encounter(&d, &gs, "q_a1_m5"), q("enc_evt_06_overlook"), "the hollis_overlook variant picks the overlook fight");
    let shown: Vec<String> = encounter_markers(&d, &gs).into_iter().map(|(_, e)| e).collect();
    assert_eq!(shown, [q("enc_evt_06_overlook")], "the Crucible shows the overlook fight only");
    for npc in ["sabina_kress", "berek_dorn"] {
        assert!(offered(&d, &gs, npc).is_empty(), "{npc}: no PK2 recipe before M5");
    }

    go(&d, &mut gs, "sluice_undercroft");
    let stayer = gs.party.members[0].id.clone();
    run_quest(&d, &mut gs, "q_a1_m5", Some(&stayer));
    assert!(gs.flag_true(&flag("m5_done")), "t_m5_overlook_won");
    assert_eq!(gs.flag(&flag("crucible_stayed")), Some(&Value::String(stayer.clone())), "the stay-behind flag names the winch holder");
    assert_eq!(gs.story.running.as_ref().map(|r| r.sequence.clone()), Some(q("seq_after_m5")), "the Breach sequence starts");
    let played = play_sequence(&d, &mut gs);
    assert!(played.fights.is_empty(), "{played:?}");
    assert_eq!(gs.location_id, q(HUB));
    let (gated, shown) = debate3_founders(&d, &played);
    assert!(!gated.is_empty(), "vg_debate_3 has founder lines gated on crucible_stayed");
    for f in &gated {
        assert_eq!(shown.contains(&q(f)), *f != stayer, "vg_debate_3: {f}'s line plays unless {f} held the winch ({stayer})");
    }
    for npc in ["sabina_kress", "berek_dorn"] {
        assert!(!offered(&d, &gs, npc).is_empty(), "{npc} offers the PK2 recipes after M5");
        assert!(explore::npc_services(&d, &gs, &q(npc)).contains(&"recipe"), "{npc} wears the recipe badge");
    }
    assert!(reachable(&d, &gs, "slag_gate"), "the high gorge opens to the Slag Gate");
}

/// M5 without the overlook: the plain gate fight; its trigger also sets `m5_done` and runs `seq_after_m5`.
#[test]
fn m5_without_the_overlook_also_opens_the_gate() {
    let d = data();
    let mut gs = after_interlude(&d);
    run_quest(&d, &mut gs, "q_a1_m5", None);
    assert!(gs.flag_true(&flag("m5_done")), "t_m5_won");
    assert!(!gs.flag_true(&flag("hollis_overlook")));
    assert_eq!(gs.story.running.as_ref().map(|r| r.sequence.clone()), Some(q("seq_after_m5")));
    let played = play_sequence(&d, &mut gs);
    let (gated, shown) = debate3_founders(&d, &played);
    for f in &gated {
        assert!(shown.contains(&q(f)), "vg_debate_3: with no winch holder {f}'s line plays");
    }
    for npc in ["sabina_kress", "berek_dorn"] {
        assert!(!offered(&d, &gs, npc).is_empty(), "{npc} offers the PK2 recipes after M5");
    }
}
