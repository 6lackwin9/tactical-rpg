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
use emberwake_world::{battle, board, campaign, dialogue, explore, mastery, new_game, quest, recipe, roster, shop, story, travel, weapons, Data, GameState};
use serde_json::Value;
use std::path::Path;

const WORLD: &str = "sunk_ward";
const HUB: &str = "sluice_undercroft";
const BOARD: &str = "talia_board";

fn data_dir() -> std::path::PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn data() -> Data {
    Data::open(&data_dir()).expect("data").0
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
    /// Per vignette the indices of its cards that played (`story::card_now`).
    cards: Vec<(String, Vec<usize>)>,
}

/// The speakers (qids) of vignette `qid`'s `line` cards that play now.
fn line_speakers(d: &Data, gs: &GameState, qid: &str) -> Vec<String> {
    let v = story::vignette(d, gs, qid);
    let home = json::s_or(v, "_world", WORLD);
    json::aget(v, "cards").iter().filter_map(|c| story::card_now(d, gs, &home, json::dict(c))).filter(|c| json::s(c, "style") == "line").map(|c| json::s(&c, "speaker")).collect()
}

/// The indices of vignette `qid`'s cards that play now.
fn cards_now(d: &Data, gs: &GameState, qid: &str) -> Vec<usize> {
    let v = story::vignette(d, gs, qid);
    let home = json::s_or(v, "_world", WORLD);
    json::aget(v, "cards").iter().enumerate().filter(|(_, c)| story::card_now(d, gs, &home, json::dict(c)).is_some()).map(|(i, _)| i).collect()
}

/// Plays the running sequence to its end as the story stage does: a vignette ends (`step_done`), a fight is won with
/// no battle (`battle::finish_world` by the fight's `story` source), the `enter` / `end` step finishes it.
fn play_sequence(d: &Data, gs: &mut GameState) -> Played {
    let mut out = Played::default();
    for _ in 0..200 {
        match story::current(d, gs) {
            Some(Step::Vignette { vignette }) => {
                out.speakers.push((vignette.clone(), line_speakers(d, gs, &vignette)));
                out.cards.push((vignette.clone(), cards_now(d, gs, &vignette)));
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

/// `run_quest` with the default tally but for the encounter's party-side guests: the first `alive` of them (spawn
/// order) live, the rest fall (the settle's `survivors` count, ACT1-HOOKS1).
fn run_quest_with_guests(d: &Data, gs: &mut GameState, quest: &str, alive: usize) {
    run_quest_tally(d, gs, quest, None, Some(alive));
}

/// Takes board quest `quest`, travels to its step's location and wins its encounter at the location's marker (the
/// explore runner, as walking into the marker does); the quest's objective is met. `stayer`: the member whose end in
/// the fight is `Stayed` (a stay-behind fight's winch holder), the others' ends as the default tally gives them; None =
/// the default tally (no battle).
fn run_quest(d: &Data, gs: &mut GameState, quest: &str, stayer: Option<&str>) {
    run_quest_tally(d, gs, quest, stayer, None);
}

fn run_quest_tally(d: &Data, gs: &mut GameState, quest: &str, stayer: Option<&str>, alive: Option<usize>) {
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
    let tally = (stayer.is_some() || alive.is_some()).then(|| {
        let world = battle::fight_world(d, gs, &enc);
        let mut t = battle::Tally::default_for(d, &gs.world_id.clone(), &enc, "won");
        t.party = roster::default_ends(d, gs, &world, "won");
        if let Some(id) = stayer {
            let me = t.party.iter_mut().find(|e| e.member == id).unwrap_or_else(|| panic!("{id} is fielded in {enc}"));
            me.fate = Some(PartyFate::Stayed);
        }
        if let Some(n) = alive {
            let guests = t.guests.as_mut().unwrap_or_else(|| panic!("the no-battle tally of {enc} knows its guests"));
            for (i, g) in guests.iter_mut().enumerate() {
                g.alive = i < n;
            }
        }
        t.world = world;
        t
    });
    let r = battle::finish_world(d, gs, "explore", &fight, "won", tally.as_ref());
    assert!(ok(&r), "{quest} {enc}: {r:?}");
    assert!(["ready", "completed"].contains(&quest::status(d, gs, &q(quest)).as_str()), "{quest} after its fight: {}", quest::status(d, gs, &q(quest)));
    assert!(!encounter_markers(d, gs).iter().any(|(_, e)| *e == enc), "{enc} leaves {at} once won");
}

/// The characters of the present NPC markers at the party's location.
fn present_npcs(d: &Data, gs: &GameState) -> Vec<String> {
    let loc = explore::location(d, gs).clone();
    let lq = json::s(&loc, "_qid");
    explore::markers_of(&loc).into_iter().filter(|m| json::s(m, "kind") == "npc" && explore::presence(d, gs, &lq, m) == "present").map(|m| json::s(&m, "character")).collect()
}

/// Turns in ready quest `quest` at the hub as a player does: back to the hub, then the conversation of the hub
/// character that offers its turn-in (a `turn_in` service reply), the reply, the turn-in picker confirmed with the
/// first offered loot up to the picks allowed. Returns the picks.
fn turn_in_at_hub(d: &Data, gs: &mut GameState, quest: &str) -> Vec<String> {
    assert_eq!(quest::status(d, gs, &q(quest)), "ready", "{quest} waits to be turned in");
    if gs.location_id != q(HUB) {
        go(d, gs, HUB);
    }
    let npcs = present_npcs(d, gs);
    for npc in &npcs {
        let mut r = DialogueRunner::open(d, gs, npc);
        if !r.errors.is_empty() {
            continue;
        }
        let services: Vec<String> = json::aget(&r.view(d, gs), "choices").iter().map(|c| json::s(json::dict(c), "service")).collect();
        for (i, svc) in services.iter().enumerate() {
            if svc != "turn_in" {
                continue;
            }
            let res = r.choose(d, gs, i as i64);
            if json::s(&res, "quest") != q(quest) {
                r.cancel();
                continue;
            }
            let picks: Vec<String> = json::sget(&res, "offers").into_iter().take(json::i(&res, "picks_allowed", 0).max(0) as usize).collect();
            let done = r.confirm_turn_in(d, gs, "", &picks);
            assert!(ok(&done), "{quest} turned in to {npc}: {done:?}");
            assert_eq!(quest::status(d, gs, &q(quest)), "completed", "{quest}");
            return picks;
        }
    }
    let rq = quest::resolved(d, gs, &q(quest));
    let line = json::aget(&board::view(d, gs, &q(BOARD)), "sections")
        .iter()
        .flat_map(|s| json::aget(json::dict(s), "rows").to_vec())
        .map(|r| json::dict(&r).clone())
        .find(|r| json::s(r, "quest") == q(quest))
        .map(|r| json::s(&r, "line"))
        .unwrap_or_default();
    panic!("no one in the hub takes {quest}'s turn-in (hub characters {npcs:?}; its giver '{}', turn_in '{}'; the board line \"{line}\")", json::s(&rq, "giver"), json::s(&rq, "turn_in"));
}

/// Leaves the hub to a node one leg away and walks back in (the hub's `enter` tick runs again).
fn reenter_hub(d: &Data, gs: &mut GameState) {
    let next = travel::known(d, gs)
        .into_iter()
        .filter_map(|n| n.strip_prefix(&format!("{WORLD}:")).map(str::to_string))
        .find(|n| *n != HUB && open_path(d, gs, n).is_some_and(|p| p.len() == 1))
        .expect("a node one leg from the hub");
    go(d, gs, &next);
    go(d, gs, HUB);
}

/// A New game in `sunk_ward` played through the `new_game` sequence into the hub.
fn after_prologue(d: &Data) -> (GameState, Played) {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("a new sunk_ward game");
    assert_eq!(story::start_sequence(d, WORLD), "new_game", "the manifest's start sequence");
    assert!(story::is_running(&gs), "a new game opens on the Prologue");
    let played = play_sequence(d, &mut gs);
    (gs, played)
}

/// Tier 1 (M1, M2) and Tier 2 (M3, M4) won from the hub, the scenes the triggers queue played (the recruits); M3 and
/// M4 turned in at the hub and the hub entered again, so the Interlude's scene stays queued for the caller.
fn after_tier2(d: &Data) -> GameState {
    let (mut gs, _) = after_prologue(d);
    for m in ["q_a1_m1", "q_a1_m2", "q_a1_m3", "q_a1_m4"] {
        run_quest(d, &mut gs, m, None);
        play_scenes(d, &mut gs, PICK);
    }
    for m in ["q_a1_m3", "q_a1_m4"] {
        turn_in_at_hub(d, &mut gs, m);
    }
    if !gs.flag_true(&flag("interlude_started")) {
        reenter_hub(d, &mut gs);
    }
    gs
}

/// The Interlude played: Sabina's dose scene is answered (its first choice that starts a sequence starts
/// `seq_interlude`), which runs to the hub.
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

/// vg_debate_3's founder cards: (founder, whether that card played in `played`) for every card gated on
/// `crucible_stayed` != that founder, as the data writes it (a line card, or a text card for a founder who does not
/// speak).
fn debate3_founders(d: &Data, played: &Played) -> Vec<(String, bool)> {
    let shown = played.cards.iter().find(|(v, _)| *v == q("vg_debate_3")).map(|(_, c)| c.clone()).expect("seq_after_m5 plays vg_debate_3");
    json::aget(entry(d, "vignettes", "vg_debate_3"), "cards")
        .iter()
        .map(json::dict)
        .enumerate()
        .filter_map(|(i, c)| {
            let w = json::aget(c, "when").iter().map(json::dict).find(|w| json::s(w, "key") == flag("crucible_stayed") && json::s(w, "op") == "!=")?;
            Some((json::s(w, "value"), shown.contains(&i)))
        })
        .collect()
}

/// PR5-41: one vg_debate_3 card per founder in the party at M5 (6 with Dunstan and Elyse), each gated on that
/// founder not having held the winch.
fn assert_a_card_per_founder(gated: &[(String, bool)], party: &[String]) {
    let mut founders: Vec<String> = gated.iter().map(|(f, _)| f.clone()).collect();
    founders.sort();
    let mut want = party.to_vec();
    want.sort();
    assert_eq!(founders, want, "vg_debate_3 has one stayed-gated card per founder in the M5 party");
}

/// PR5-41: `seq_after_m5` moves the campaign to the act its `set_act` effect names, so the game's act-gated rules
/// (`durability.from_act`) switch on after M5.
fn assert_act_after_m5(d: &Data, gs: &GameState) {
    let act = json::aget(entry(d, "sequences", "seq_after_m5"), "steps")
        .iter()
        .flat_map(|s| json::aget(json::dict(s), "effects").to_vec())
        .map(|fx| json::dict(&fx).clone())
        .find(|fx| json::s(fx, "effect") == "set_act")
        .map(|fx| json::i(&fx, "act", 0))
        .expect("seq_after_m5 sets the act");
    let now = gs.campaign.as_ref().map(|c| c.act).expect("sunk_ward is a campaign world");
    assert_eq!(now, act, "the campaign is in act {act} after M5");
    assert!(weapons::active(d, gs), "durability (from act {}) is on after M5 (act {now})", d.from_act);
    assert!(mastery::open(d, gs), "weapon mastery is gained after M5 (act {now})");
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
/// `m1_done` / `m2_done`); after M3 and M4 are won the party stays in the field (PR5-41: Orlov's stores and the
/// counter-still stay open) until both are turned in at the hub; then the Interlude trigger (on entering the hub)
/// sets `interlude_started` and queues Sabina's dose scene, offered to party Founders only; the scene starts
/// `seq_interlude`, which sets `interlude_done` and opens the convoy route to the Crucible.
#[test]
fn tier_one_then_tier_two_then_the_interlude() {
    let d = data();
    let (mut gs, _) = after_prologue(&d);
    let prologue_party: Vec<String> = gs.party.members.iter().map(|m| m.id.clone()).collect();
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
    let scenes = play_scenes(&d, &mut gs, PICK);
    assert!(scenes.iter().any(|(s, fx)| s == "dlg_post_m4" && fx.iter().any(|e| e == "recruit")), "Elyse's recruit choice: {scenes:?}");
    assert!(dialogue::is_member(&d, &gs, "elyse") && gs.flag_true(&flag("elyse_joined")), "Elyse joins through dlg_post_m4");
    assert_eq!(gs.party.members.len(), before + 2, "Elyse takes the next bunk");

    // PR5-41: after M3 and M4 the party stays in the field until it turns both in at the hub; Orlov's stores and
    // Elyse's counter-still at the Vats stay open before the Interlude
    assert!(!saw(&scenes, "dlg_int_dose") && !gs.flag_true(&flag("interlude_started")), "winning M3 and M4 does not start the Interlude: {scenes:?}");
    let vats = entry(&d, "locations", "tallow_vats");
    for id in ["orlov_stores", "counter_still"] {
        let m = explore::markers_of(vats).into_iter().find(|m| json::s(m, "id") == id).unwrap_or_else(|| panic!("the Vats have {id}"));
        assert_eq!(explore::presence(&d, &gs, &q("tallow_vats"), &m), "present", "{id} is open after M4, before the Interlude");
    }
    go(&d, &mut gs, HUB);
    assert!(!gs.flag_true(&flag("interlude_started")) && pending(&gs).is_empty(), "back at the hub with M3 and M4 not turned in: no Interlude yet");
    let trig = entry(&d, "triggers", "t_interlude");
    let when: Vec<String> = json::aget(trig, "when").iter().map(|w| json::s(json::dict(w), "key")).collect();
    for m in ["q_a1_m3", "q_a1_m4"] {
        turn_in_at_hub(&d, &mut gs, m);
        for c in json::aget(entry(&d, "quests", m), "complete_flags") {
            let key = json::s(json::dict(c), "key");
            assert!(gs.flag_true(&key), "{m}'s turn-in sets {key}");
            assert!(when.contains(&key), "t_interlude waits for {key}");
        }
    }
    for key in &when {
        assert!(gs.flag_true(key), "t_interlude's condition {key} holds after the turn-ins");
    }
    // t_interlude is an `enter` trigger at the hub: it fires the next time the party walks into the hub
    if !gs.flag_true(&flag("interlude_started")) {
        reenter_hub(&d, &mut gs);
    }
    assert!(gs.flag_true(&flag("interlude_started")), "t_interlude after M3 and M4 are turned in");
    let queued = pending(&gs);
    assert!(queued.contains(&("dialogue".into(), "dlg_int_dose".into())), "Sabina's dose scene is queued: {queued:?}");
    // PR5-41: only a Founder in the party can take the dose (no "Nobody"); the Tier 2 recruits' replies wait for
    // their joined flags
    let party: Vec<String> = gs.party.members.iter().map(|m| m.id.clone()).collect();
    let dose: Vec<(String, Vec<String>)> = json::aget(entry(&d, "dialogues", "dlg_int_dose"), "nodes")
        .iter()
        .flat_map(|n| json::aget(json::dict(n), "choices").to_vec())
        .map(|c| json::dict(&c).clone())
        .filter(|c| json::aget(c, "effects").iter().any(|fx| json::s(json::dict(fx), "effect") == "sequence"))
        .map(|c| {
            let to = json::aget(&c, "effects").iter().map(json::dict).find(|fx| json::s(fx, "key") == flag("sabina_dose_to")).map(|fx| json::s(fx, "value")).unwrap_or_default();
            (to, json::aget(&c, "requires").iter().map(|r| json::s(json::dict(r), "key")).collect())
        })
        .collect();
    assert!(!dose.is_empty(), "dlg_int_dose has dose replies");
    for (to, requires) in &dose {
        assert!(party.contains(to), "the dose reply for '{to}' names a Founder in the party ({party:?})");
        if !prologue_party.contains(to) {
            assert!(requires.contains(&flag(&format!("{to}_joined"))), "{to}'s dose reply waits for {to}_joined: {requires:?}");
        }
    }

    let scenes = play_scenes(&d, &mut gs, PICK);
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
    let party: Vec<String> = gs.party.members.iter().map(|m| m.id.clone()).collect();
    let stayer = party[0].clone();
    run_quest(&d, &mut gs, "q_a1_m5", Some(&stayer));
    assert!(gs.flag_true(&flag("m5_done")), "t_m5_overlook_won");
    assert_eq!(gs.flag(&flag("crucible_stayed")), Some(&Value::String(stayer.clone())), "the stay-behind flag names the winch holder");
    assert_eq!(gs.story.running.as_ref().map(|r| r.sequence.clone()), Some(q("seq_after_m5")), "the Breach sequence starts");
    let played = play_sequence(&d, &mut gs);
    assert!(played.fights.is_empty(), "{played:?}");
    assert_eq!(gs.location_id, q(HUB));
    let gated = debate3_founders(&d, &played);
    assert_a_card_per_founder(&gated, &party);
    for (f, shown) in &gated {
        assert_eq!(*shown, *f != stayer, "vg_debate_3: {f}'s card plays unless {f} held the winch ({stayer})");
    }
    assert_act_after_m5(&d, &gs);
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
    let party: Vec<String> = gs.party.members.iter().map(|m| m.id.clone()).collect();
    run_quest(&d, &mut gs, "q_a1_m5", None);
    assert!(gs.flag_true(&flag("m5_done")), "t_m5_won");
    assert!(!gs.flag_true(&flag("hollis_overlook")));
    assert_eq!(gs.story.running.as_ref().map(|r| r.sequence.clone()), Some(q("seq_after_m5")));
    let played = play_sequence(&d, &mut gs);
    let gated = debate3_founders(&d, &played);
    assert_a_card_per_founder(&gated, &party);
    for (f, shown) in &gated {
        assert!(*shown, "vg_debate_3: with no winch holder {f}'s card plays");
    }
    assert_act_after_m5(&d, &gs);
    for npc in ["sabina_kress", "berek_dorn"] {
        assert!(!offered(&d, &gs, npc).is_empty(), "{npc} offers the PK2 recipes after M5");
    }
}

// ---- PR5-41 ACT1-DATA1 acceptance -------------------------------------------------------------------------------

/// The items of loot entry `id` (its `ref`), as the stash keys them.
fn loot_item(d: &Data, id: &str) -> String {
    json::s(entry(d, "loot", id), "ref")
}

/// M1: winning the Sweep pays its objective reward (the gate pass, no longer kill loot); turning it in at the hub
/// delivers its loot pool (the Gilded Pavise); the road to Corvo's Gate opens with `m1_done`.
#[test]
fn m1_pays_the_gate_pass_and_the_pavise_and_opens_the_road_to_corvo() {
    let d = data();
    let enc = entry(&d, "encounters", "enc_evt_01");
    let pass = json::s(json::dget(enc, "objective_reward"), "loot");
    assert!(!pass.is_empty(), "enc_evt_01 has an objective reward");
    assert!(json::aget(enc, "key_loot").iter().all(|k| json::s(json::dict(k), "loot") != pass), "{pass} is not kill loot any more");
    let (mut gs, _) = after_prologue(&d);
    assert!(!reachable(&d, &gs, "corvos_gate"), "Corvo's Gate waits for M1");
    run_quest(&d, &mut gs, "q_a1_m1", None);
    assert!(gs.loot_claimed.contains(&q(&pass)), "winning M1 pays {pass}: {:?}", gs.loot_claimed);
    assert!(gs.party.stash.get(&loot_item(&d, &pass)).copied().unwrap_or(0) > 0, "the gate pass is in the stash");
    assert!(reachable(&d, &gs, "corvos_gate"), "the Third Ward road opens with m1_done");
    let pool = json::sget(entry(&d, "quests", "q_a1_m1"), "loot_pool");
    assert!(!pool.is_empty(), "q_a1_m1 has a loot pool");
    let picks = turn_in_at_hub(&d, &mut gs, "q_a1_m1");
    assert!(!picks.is_empty(), "the turn-in offers the pool");
    for p in &picks {
        assert!(pool.iter().any(|l| q(l) == *p), "{p} is from M1's pool {pool:?}");
        assert!(gs.loot_claimed.contains(p), "{p} is claimed at the turn-in");
        let item = loot_item(&d, p.trim_start_matches(&format!("{WORLD}:")));
        assert!(gs.party.stash.get(&item).copied().unwrap_or(0) > 0, "{item} is in the stash after the turn-in");
    }
}

/// M2: the settle counts the captives that lived into the encounter's `survivors` flag, and Maren's scene offers the
/// one reply that count opens (no self-report): its Rock Salt is what the party gets, more saved never pays less.
#[test]
fn m2_rewards_the_captives_that_lived() {
    let d = data();
    let enc = entry(&d, "encounters", "enc_evt_02");
    let survivors = json::dget(enc, "survivors");
    let key = json::s(survivors, "flag");
    assert_eq!(key, flag("saved_youths"), "enc_evt_02 counts its captives");
    let unit = json::s(survivors, "unit");
    let u = d.entry("units", &unit);
    assert!(!u.is_empty(), "the captive unit {unit}");
    assert_eq!(u.get("flee_ratio").and_then(Value::as_f64), Some(0.0), "{unit} no longer flees");
    let guests = json::sget(enc, "guests").len();
    assert!(guests > 0, "enc_evt_02 has captives");
    let mut paid: Vec<i64> = Vec::new();
    for alive in 0..=guests {
        let (mut gs, _) = after_prologue(&d);
        let salt_before = gs.party.stash.get("rock_salt").copied().unwrap_or(0);
        run_quest_with_guests(&d, &mut gs, "q_a1_m2", alive);
        assert_eq!(gs.flag(&key).and_then(Value::as_i64), Some(alive as i64), "{alive} captives lived");
        let item = campaign::pending_head(&gs).cloned().expect("Maren's scene is queued");
        campaign::pop_pending(&mut gs);
        let dlg = json::s(&item, "dialogue");
        let mut r = DialogueRunner::open_scene(&d, &mut gs, &dlg, &json::s(&item, "speaker"));
        let view = r.view(&d, &gs);
        let open: Vec<usize> = json::aget(&view, "choices").iter().enumerate().filter(|(_, c)| !json::b(json::dict(c), "locked", false)).map(|(i, _)| i).collect();
        assert_eq!(open.len(), 1, "{alive} lived: one reply opens: {view:?}");
        let node = r.node_id.clone();
        let chosen = json::dict(&json::aget(&view, "choices")[open[0]]).clone();
        let tree = entry(&d, "dialogues", &dlg);
        let data_choice = json::aget(tree, "nodes").iter().map(json::dict).find(|n| json::s(n, "id") == node).map(|n| json::aget(n, "choices").to_vec()).unwrap_or_default();
        for c in &data_choice {
            assert!(json::aget(json::dict(c), "requires").iter().any(|r| json::s(json::dict(r), "key") == key), "every reply is gated on the count: {c:?}");
        }
        let give: i64 = data_choice
            .iter()
            .map(json::dict)
            .find(|c| json::s(c, "text") == json::s(&chosen, "text"))
            .map(|c| json::aget(c, "effects").iter().map(json::dict).filter(|fx| json::s(fx, "effect") == "give").map(|fx| json::i(json::dget(fx, "items"), "rock_salt", 0)).sum())
            .unwrap_or(0);
        assert!(ok(&r.choose(&d, &mut gs, open[0] as i64)), "the reply is taken");
        assert_eq!(gs.party.stash.get("rock_salt").copied().unwrap_or(0) - salt_before, give, "{alive} lived: the reply's Rock Salt");
        paid.push(give);
    }
    assert_eq!(paid[0], 0, "nobody saved pays nothing");
    assert!(paid.windows(2).all(|w| w[0] <= w[1]) && paid[guests] > 0, "more saved never pays less: {paid:?}");
}

/// Opens `character`'s conversation and leaves it by its own reply that leads to an end node (the tree has
/// `services: false`, so no farewell is added): the conversation closes; a conversation is never a must-choose scene.
fn leave_by_reply(d: &Data, gs: &mut GameState, character: &str, dlg: &str) {
    let tree = entry(d, "dialogues", dlg).clone();
    assert!(!json::b(&tree, "services", true), "{dlg} has services: false");
    let ends: Vec<String> = json::aget(&tree, "nodes").iter().map(json::dict).filter(|n| json::b(n, "end", false)).map(|n| json::s(n, "id")).collect();
    let mut r = DialogueRunner::open(d, gs, character);
    assert!(r.errors.is_empty(), "{character} talks: {:?}", r.errors);
    assert!(!r.must_choose(d, gs), "{dlg}: a conversation may be left");
    let view = r.view(d, gs);
    let choices = json::aget(&view, "choices").to_vec();
    assert!(choices.iter().all(|c| json::s(json::dict(c), "service") != "farewell"), "{dlg}: no added farewell: {view:?}");
    let start = json::s(&tree, "start");
    let start_choices = json::aget(&tree, "nodes").iter().map(json::dict).find(|n| json::s(n, "id") == start).map(|n| json::aget(n, "choices").to_vec()).unwrap_or_default();
    let leave = start_choices
        .iter()
        .map(json::dict)
        .find(|c| ends.contains(&json::s(c, "to")) && json::aget(c, "effects").is_empty())
        .map(|c| json::s(c, "text"))
        .unwrap_or_else(|| panic!("{dlg}: a reply that leaves (to an end node, no effects)"));
    let k = choices.iter().position(|c| json::s(json::dict(c), "text") == leave).unwrap_or_else(|| panic!("{dlg}: '{leave}' is shown"));
    assert!(!json::b(json::dict(&choices[k]), "locked", false), "{dlg}: '{leave}' is open");
    assert!(ok(&r.choose(d, gs, k as i64)), "{dlg}: '{leave}'");
    if !r.closed {
        assert!(ok(&r.advance(d, gs)), "{dlg}: the end line");
    }
    assert!(r.closed, "{dlg}: '{leave}' ends the conversation");
}

/// Corvo's Gate: Ivo's and Corvo's conversations offer no services and are left by their own reply; forcing the gate
/// starts Corvo's fight and, once won, his marker is gone for good (`corvo_forced`).
#[test]
fn ivo_and_corvo_are_left_by_a_reply_and_a_forced_corvo_is_gone() {
    let d = data();
    let (mut gs, _) = after_prologue(&d);
    leave_by_reply(&d, &mut gs, "ivo", "dlg_ivo");
    run_quest(&d, &mut gs, "q_a1_m1", None);
    play_scenes(&d, &mut gs, PICK);
    go(&d, &mut gs, "corvos_gate");
    let corvo = present_npcs(&d, &gs).into_iter().find(|c| json::s(entry(&d, "characters", c), "dialogue") == "dlg_corvo_gate").expect("Corvo stands at his gate");
    leave_by_reply(&d, &mut gs, &corvo, "dlg_corvo_gate");
    let tree = entry(&d, "dialogues", "dlg_corvo_gate").clone();
    let force = json::aget(&tree, "nodes")
        .iter()
        .flat_map(|n| json::aget(json::dict(n), "choices").to_vec())
        .map(|c| json::dict(&c).clone())
        .find(|c| json::aget(c, "effects").iter().map(json::dict).any(|fx| json::s(fx, "key") == flag("corvo_forced")))
        .expect("a reply forces the gate");
    let mut r = DialogueRunner::open(&d, &mut gs, &corvo);
    let view = r.view(&d, &gs);
    let k = json::aget(&view, "choices").iter().position(|c| json::s(json::dict(c), "text") == json::s(&force, "text")).expect("the force reply is shown");
    let res = r.choose(&d, &mut gs, k as i64);
    let fight = json::dict(res.get("fight").unwrap_or_else(|| panic!("forcing the gate starts a fight: {res:?}"))).clone();
    let w = battle::finish_world(&d, &mut gs, &json::s(&fight, "source"), &fight, "won", None);
    assert!(ok(&w), "Corvo's fight: {w:?}");
    assert!(gs.flag_true(&flag("corvo_forced")));
    assert!(!present_npcs(&d, &gs).contains(&corvo), "a forced Corvo is gone from his gate");
    reenter_or_return(&d, &mut gs, "corvos_gate");
    assert!(!present_npcs(&d, &gs).contains(&corvo), "and stays gone");
}

/// Leaves `at` for the hub and walks back to `at`.
fn reenter_or_return(d: &Data, gs: &mut GameState, at: &str) {
    go(d, gs, HUB);
    go(d, gs, at);
}

/// The value of meter `id` now.
fn meter(gs: &GameState, id: &str) -> i64 {
    gs.campaign.as_ref().map(|c| c.value(id, gs.time)).unwrap_or(0)
}

/// Whether countdown `id` is running (`meter.<id>.running`).
fn running(gs: &GameState, id: &str) -> bool {
    gs.campaign.as_ref().and_then(|c| c.meters.get(id)).is_some_and(|m| m.running)
}

/// Passes days one at a time (a rest each) until meter rule `rule` fires or `max` days went by; the tick events.
fn days_until_rule(d: &Data, gs: &mut GameState, rule: &str, max: i64) -> Vec<Value> {
    let mut events = Vec::new();
    for _ in 0..max {
        gs.time += 1;
        let now = campaign::tick(d, gs, &campaign::Cause::Days { n: 1, why: "rest".into() });
        let fired = now.iter().any(|e| json::s(json::dict(e), "rule") == rule);
        events.extend(now);
        if fired {
            break;
        }
    }
    events
}

/// PR5-41: the Red Tithing (`doom_clock`) counts down in Act 1 once the Prologue is over, so its `d_zero` rule can
/// fall at all.
#[test]
fn the_red_tithing_runs_after_the_prologue() {
    let d = data();
    let (gs, _) = after_prologue(&d);
    assert!(running(&gs, "doom_clock"), "the Red Tithing counts down after the Prologue (no clock_start in the shipped story)");
}

/// The Red Tithing at zero (`d_zero`) raises the Inquisition's Alert by the rule's own amount (the row's +10), in
/// Act 1, and the run goes on. The clock is started here with the campaign's own `clock_start` effect.
#[test]
fn the_red_tithing_at_zero_raises_the_alert() {
    let d = data();
    let clock = entry(&d, "meters", "doom_clock").clone();
    let rule = json::aget(&clock, "rules").iter().map(json::dict).find(|r| json::s(r, "id") == "d_zero").cloned().expect("doom_clock has d_zero");
    let add = json::aget(&rule, "effects").iter().map(json::dict).find(|fx| json::s(fx, "effect") == "meter_add" && json::s(fx, "meter") == "alert").cloned().expect("d_zero adds to the Alert");
    let delta = json::i(&add, "delta", 0);
    assert_eq!(delta, 10, "the row's +10 Alert");
    assert!(json::aget(&rule, "acts").iter().any(|a| a.as_i64() == Some(1)), "d_zero runs in Act 1");
    assert!(!json::s(&rule, "warn").is_empty(), "d_zero warns");
    let (mut gs, _) = after_prologue(&d);
    if !running(&gs, "doom_clock") {
        let mut events = Vec::new();
        campaign::apply_effects(&d, &mut gs, &[json::obj([("effect", "clock_start".into()), ("meter", "doom_clock".into())])], &mut events);
    }
    assert!(running(&gs, "doom_clock"));
    let alert_before = meter(&gs, "alert");
    let events = days_until_rule(&d, &mut gs, "d_zero", json::i(&clock, "max", 0) + 1);
    assert!(gs.campaign.as_ref().is_some_and(|c| !c.is_over()), "the run goes on: {events:?}");
    let fired = events.iter().map(json::dict).any(|e| json::s(e, "rule") == "d_zero");
    assert!(fired, "d_zero ran at zero: {events:?}");
    assert_eq!(meter(&gs, "alert"), alert_before + delta, "the Alert rose by d_zero's amount");
}

/// Widow Varka's trade holds the PK2 salt back until the Slab opens (`pk2.slab_open`, set by `seq_after_m5`).
#[test]
fn varka_holds_the_quenched_salt_back_until_the_slab_opens() {
    let d = data();
    let sh = entry(&d, "shops", "varka_trade").clone();
    let gated: Vec<String> = json::aget(&sh, "stock")
        .iter()
        .map(json::dict)
        .filter(|st| json::aget(st, "requires_flags").iter().any(|f| json::s(json::dict(f), "key") == "pk2.slab_open"))
        .map(|st| json::s(st, "item"))
        .collect();
    assert!(gated.contains(&"con_quenched_salt".to_string()), "con_quenched_salt waits for the Slab: {gated:?}");
    let sets_slab = json::aget(entry(&d, "sequences", "seq_after_m5"), "steps")
        .iter()
        .flat_map(|s| json::aget(json::dict(s), "effects").to_vec())
        .any(|fx| json::s(json::dict(&fx), "key") == "pk2.slab_open");
    assert!(sets_slab, "seq_after_m5 opens the Slab");
    let (mut gs, _) = after_prologue(&d);
    // the shop's own gate (Ivo rescued), as the data writes it
    for f in json::aget(&sh, "requires_flags") {
        let f = json::dict(f);
        gs.set_flag(&json::s(f, "key"), f.get("value").cloned().unwrap_or(Value::Bool(true)));
    }
    let items = |gs: &GameState| shop::offers(&d, gs, &q("varka_trade")).iter().map(|o| json::s(o, "item")).collect::<Vec<_>>();
    let act1 = items(&gs);
    assert!(!act1.is_empty(), "Varka trades in Act 1");
    for g in &gated {
        assert!(!act1.contains(g), "Act 1: {g} is not on sale");
    }
    gs.set_flag("pk2.slab_open", Value::Bool(true));
    let later = items(&gs);
    for g in &gated {
        assert!(later.contains(g), "after the Slab opens: {g} is on sale");
    }
}

/// Every `description` string under `v` (any depth) that starts with "Test ", by path.
fn test_descriptions(v: &Value, path: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                let at = format!("{path}.{k}");
                if k == "description" && x.as_str().is_some_and(|s| s.starts_with("Test ")) {
                    out.push(at.clone());
                }
                test_descriptions(x, &at, out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                test_descriptions(x, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

/// The shipped data's JSON file `rel` (under data/).
fn data_file(rel: &str) -> Value {
    let text = std::fs::read_to_string(data_dir().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// PR5-41 read-only data clauses: no item and no `sunk_ward` entry shows a "Test ..." hover description; the
/// CANON 5.3 names (`marrow_valve` "Valve Overpressure" with no promise it will not let him fall, `render_beast_tendon`
/// "Render-Beast Tendon"); the Hearth repair asks 2 fine salvage of a higher tier; every `sunk_ward` road is free of
/// random fights (`encounter_pct` 0).
#[test]
fn act1_data_reads_in_world() {
    let d = data();
    let mut found = Vec::new();
    test_descriptions(&data_file("catalog/items.json"), "catalog/items", &mut found);
    let dir = data_dir().join("worlds").join(WORLD);
    let mut files: Vec<String> = std::fs::read_dir(&dir).expect("the world folder").filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".json")).collect();
    files.sort();
    for f in &files {
        test_descriptions(&data_file(&format!("worlds/{WORLD}/{f}")), &format!("worlds/{WORLD}/{f}"), &mut found);
    }
    assert!(found.is_empty(), "\"Test ...\" hover descriptions: {found:?}");

    let valve = d.entry("passives", "marrow_valve");
    assert_eq!(json::s(valve, "name"), "Valve Overpressure");
    assert!(!json::s(valve, "description").contains("will not let him fall"), "the retired promise is gone: {valve:?}");
    assert_eq!(d.item_name("render_beast_tendon"), "Render-Beast Tendon");

    let repair = d.entry("recipes", "hearth_repair");
    let fx = json::aget(repair, "effects").iter().map(json::dict).find(|fx| json::s(fx, "effect") == "repair").cloned().expect("hearth_repair repairs");
    assert!(!json::dget(&fx, "tiers").is_empty(), "the higher tiers ask salvage");
    assert_eq!(json::i(&fx, "count", 0), 2, "2 salvage per higher-tier repair");

    let routes = d.merged_entries(WORLD, "routes");
    assert!(!routes.is_empty());
    for r in routes {
        assert_eq!(json::i(r, "encounter_pct", -1), 0, "route {} has no random fights", json::s(r, "id"));
    }
}
