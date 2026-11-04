//! The battle's flow on the screen: starting, the action dispatcher (`run`), autoplay planning, the round's end, settling a world battle, and the sounds of log lines.

use super::input::RowAct;
use super::marks::{hold_line, round_summary_text};
use super::overlays::Layers;
use super::{after_fight, log_settled, Action, BattleScreen, Mode, AUTO_CONFIRM, CONFIRM_GUARD_MS, PLAYBACK_SPEED, PLAYBACK_SPEEDS, TACTICS, TACTIC_LABELS};
use super::tokens::Tokens;
use crate::screens::{show_errors, Cx, Nav};
use aegis_ui::audio::Bus;
use emberwake_battle::round_intent::{self as ri, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use aegis_core::json::{self, s};
use emberwake_world::dialogue;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

impl BattleScreen {
    /// ENCOUNTER2-C4: the Arena's encounter has a script and the ruleset the `new` request picks has the round intent
    /// (the host's choice: the request's ruleset and rules over the catalog's classic or the world's ruleset).
    fn arena_scripted(&self) -> bool {
        let (entry, default) = match (&self.arena_world, self.host.data.as_ref()) {
            (Some(world), Some(d)) => (d.resolve(world, "encounters", &self.encounter).clone(), d.ruleset(world)),
            (Some(_), None) => return false,
            (None, _) => match self.host.cat.encounters.get(&self.encounter) {
                Some(e) => (e.clone(), emberwake_rules::ruleset::Ruleset::classic()),
                None => return false,
            },
        };
        if json::arr(entry.get("script").unwrap_or(&Value::Null)).is_empty() {
            return false;
        }
        let mut r = match self.arena_ruleset.as_deref().filter(|id| !id.is_empty()) {
            Some(id) => match emberwake_rules::ruleset::Ruleset::of(&self.host.cat, id) {
                Ok(r) => r,
                Err(_) => return false,
            },
            None => default,
        };
        if !self.arena_rules.is_empty() && r.adjust(&self.arena_rules).is_err() {
            return false;
        }
        r.has(emberwake_rules::ruleset::ROUND_INTENT)
    }

    /// Starts the arena's encounter through the command host (a world one as "<world>:<id>"), so the session replays.
    pub(super) fn start_battle(&mut self) {
        let encounter = match &self.arena_world {
            Some(world) => format!("{world}:{}", self.encounter),
            None => self.encounter.clone(),
        };
        let mut req = json!({"cmd": "new", "encounter": encounter});
        if let Some(s) = self.seed {
            req["seed"] = json!(s);
        }
        if let Some(r) = &self.arena_ruleset {
            req["ruleset"] = json!(r);
        }
        // RULESET1-G: the Arena's extension switches and plan reveal (absent when unset: a classic request is unchanged)
        if !self.arena_rules.is_empty() {
            req["rules"] = json!(self.arena_rules);
        }
        if self.arena_reveal {
            req["plan_reveal"] = json!(true);
        }
        // INPUT1 MR11: the Arena's diagonal melee pick (absent when the row's value plays)
        if !self.arena_diagonal.is_empty() {
            req["melee_diagonal"] = json!(self.arena_diagonal);
        }
        // ENCOUNTER2-C4: the party tactic drafts a scripted round's auto heroes (absent when balanced, the kit's default,
        // and in a battle that plays no script, so its recording is unchanged; the hero tactics start over each battle)
        if self.party_tactic != 0 && self.arena_scripted() {
            req["tactics"] = json!({"party": TACTICS[self.party_tactic]});
        }
        let reply = self.host.handle(&req);
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("could not start").to_string();
            return;
        }
        self.new_session();
        self.after_start(reply["intro"].as_str().unwrap_or(""));
    }

    pub(super) fn after_start(&mut self, intro: &str) {
        self.tokens = Tokens::default();
        self.rev += 1;
        // A new battle: the last one's pending feedback has nothing left to show on (rule F's flush happens in `send`).
        self.impacts.clear();
        self.flashes.clear();
        self.gap_until = None;
        self.fx.clear();
        self.camera.stop();
        self.sidesteps.clear();
        self.phases_seen = self.session().phases.fired.len();
        self.reset_talks();
        self.last_hp = self.session().w.units.iter().map(|u| u.hp).collect();
        // PORTRAITFX2 5.3: the start events wait for the first frame; the round, outcome and popped action as they are
        (self.portrait_start, self.portrait_round, self.portrait_outcome, self.portrait_popped) = (true, self.session().current_round, self.session().outcome.clone(), self.session().last_popped);
        self.log_seen = self.session().resolver.log.len();
        self.selected = self.session().w.allies.first().copied();
        self.popup = None;
        self.playing = false;
        self.auto = false;
        self.pending_next_round = false;
        self.cutscene = 0;
        self.floats.clear();
        self.log_scroll = 0;
        self.player_target.clear();
        self.member_tactic.clear();
        self.deferred.clear();
        self.ghosted = 0;
        self.inspect = None;
        self.tracers.clear();
        self.traced = self.session().last_popped;
        self.play_block = None;
        self.block_ended = None;
        self.waiting = None;
        self.card_ap.clear();
        self.banner = None;
        self.round_note = None;
        self.advanced_at = None;
        let (objective, n) = {
            let s = self.session();
            (emberwake_battle::encounter::Objectives::text_of(&s.def.raw, |r| s.units_of_ref(r).first().map(|&c| s.w.units[c].name.clone()).unwrap_or_else(|| r.to_string())), s.events.len())
        };
        self.events_seen = n;
        // ENCOUNTER2 H1, H7: a hold escape asks for its holder first; once picked the line names them
        let s = self.session();
        let objective = match &s.w.hold {
            Some(gate) if gate.holder.is_some() => format!("{objective} {}", hold_line(s)),
            Some(gate) => format!("{objective} Pick who holds {} first.", gate.label),
            None => objective,
        };
        self.message = if intro.is_empty() { format!("{objective} Plan your round, then confirm.") } else { format!("{intro} {objective}") };
    }

    pub(super) fn planning(&self) -> bool {
        let s = self.session();
        s.phase == Phase::Planning && !s.plan_confirmed
    }

    /// LOOK1 5.4: the stage set of the battle's world (a world fight's, the Arena's when it plays a world's encounter);
    /// "" (`common_stage`) for a catalog encounter.
    pub(super) fn stage_set(&self, cx: &Cx) -> String {
        let world = if self.world_mode() { self.class_world.as_str() } else { self.arena_world.as_deref().unwrap_or("") };
        if world.is_empty() {
            String::new()
        } else {
            dialogue::stage_set(&cx.data, world)
        }
    }

    /// ENCOUNTER2-C SR5, C4: the current round is scripted and locked (its plan is the script's, confirmed at its start).
    pub(super) fn locked(&self) -> bool {
        self.host.session.as_ref().is_some_and(|s| s.round_locked())
    }

    /// ENCOUNTER2-C4: a locked round still to play shows Skip in Confirm's place.
    pub(super) fn can_skip(&self) -> bool {
        self.locked() && self.session().phase != Phase::Replay && !self.session().combat_over
    }

    /// ENCOUNTER2-C4 (decision "ENCOUNTER2-C3 answers" (1)): the scripted rounds still to plan draft their auto heroes
    /// with the players' tactics (host `tactics`: the party tactic and each hero tactic by hero key); sent only in a
    /// battle with a script under the round intent, so other battles record no extra request.
    fn send_tactics(&mut self, cx: &Cx) {
        let Some(s) = self.host.session.as_ref().filter(|s| !s.script.is_empty() && ri::on(&s.w)) else { return };
        // sorted by hero key, so the recorded request is the same bytes whatever the map's order
        let mut keyed: Vec<(String, usize)> = self.member_tactic.iter().map(|(&m, &t)| (s.hero_key(m), t)).filter(|(k, _)| !k.is_empty()).collect();
        keyed.sort();
        let members: serde_json::Map<String, Value> = keyed.into_iter().map(|(k, t)| (k, json!(TACTICS[t]))).collect();
        let req = json!({"cmd": "tactics", "party": TACTICS[self.party_tactic], "members": members});
        self.send(cx, req);
    }

    /// BattleBridge.outcome: the session's verdict, else its `timeout_outcome` once the last round is over (ENCOUNTER2
    /// S2, `Session::final_outcome`; fled by default); "" while it can go on.
    pub(super) fn outcome(&self) -> String {
        self.session().final_outcome()
    }

    /// World battle over: the runner settles the fight from the battle's tally (REWARD1's foes and ENCOUNTER2 W1's
    /// party side: the kit's `battle::apply` settles the party first by `roster::settle_party`: the stay-behind `then`,
    /// the world's rule row, HP; then the rewards), the results popup opens.
    fn settle(&mut self, cx: &mut Cx) {
        let outcome = self.outcome();
        let Mode::World { fight, runner, members } = &self.mode else { return };
        if outcome.is_empty() {
            return;
        }
        let d = cx.data.clone();
        let tally = emberwake_world::battle::Tally::of_world_fight(self.session(), members, &self.class_world);
        let gs = cx.gs_mut();
        let res = after_fight(&d, gs, runner, fight, &outcome, Some(&tally));
        cx.set_busy(false);
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
        }
        log_settled(cx, runner, &res);
        cx.autosave();
        let fight = fight.clone();
        cx.nav.push(Nav::Pop);
        cx.nav.push(Nav::Push(Box::new(crate::hub::Results::new(&res, &outcome, &fight))));
    }

    // ---- Actions (all mutations are command-API requests) -----------------------------------------------------

    /// A player's action (a click or a key): the playback controls click (SoundEvents ui_click), then `run`.
    pub(super) fn act(&mut self, cx: &mut Cx, a: Action) {
        // ENCOUNTER2-C DT5: while a talk is open only its box acts
        if self.talk.is_some() != matches!(a, Action::Talk(_)) {
            return;
        }
        // VISCLAR1-D: the player's Confirm only (autoplay's goes through `run`): see `CONFIRM_GUARD_MS`
        if a == Action::Confirm && self.advanced_at.is_some_and(|t| t.elapsed() < Duration::from_millis(CONFIRM_GUARD_MS)) {
            return;
        }
        if matches!(a, Action::Confirm | Action::Step | Action::Autoplay | Action::NextRound | Action::PlaybackSpeed(_) | Action::NextBlock | Action::Skip) {
            cx.audio.play(Bus::Ui, "sfx/ui_click.wav");
        }
        self.run(cx, a);
    }

    /// An action, from the player or chained by another action or autoplay.
    pub(super) fn run(&mut self, cx: &mut Cx, a: Action) {
        match a {
            Action::Select(u) => {
                // INPUT1 SL4, SL6 (round intent): a party card always selects; the selected hero's card selects nothing
                let again = self.selected == Some(u) && self.host.session.as_ref().is_some_and(|s| ri::on(&s.w));
                self.selected = if again { None } else { Some(u) };
                self.popup = None;
            }
            Action::UnitClick(u) => self.unit_click(cx, u),
            Action::TileClick(c) => self.tile_click(cx, c),
            Action::FillGhost => self.accept_ghost(cx, None),
            Action::GhostSlot(k) => self.accept_ghost(cx, Some(k)),
            Action::Choose(i) => {
                let Some(p) = self.popup.take() else { return };
                let Some(row) = p.rows.get(i).filter(|r| r.valid).cloned() else {
                    // a disabled row keeps the menu open (its reason is on the row)
                    self.popup = Some(p);
                    return;
                };
                match row.act {
                    RowAct::Pick(picks) => {
                        if let Some((h, at)) = p.menu {
                            self.popup = Some(p);
                            self.choose_pick(cx, h, at, &picks, &row.label);
                        }
                    }
                    RowAct::SelectAlly(u) => {
                        self.selected = Some(u);
                    }
                    RowAct::Fill => self.accept_ghost(cx, None),
                    RowAct::Step(step) => {
                        if let Some(h) = self.selected {
                            let r = self.send(cx, json!({"cmd": "add_step", "unit": h, "step": step}));
                            if r["ok"] == json!(true) {
                                self.message = format!("{}: {}", self.session().w.units[h].name, row.label);
                            }
                        }
                    }
                    RowAct::PartyTactic(t) => {
                        self.party_tactic = t;
                        // IF9: the party tactic drafts every member with no pick (the kit skips the others); its
                        // drafts are the player's own call, so no ghost waits for Confirm
                        self.member_tactic.clear();
                        let picked: Vec<usize> = self.session().w.allies.iter().copied().filter(|&a| !rip::picks_of(&self.session().w, a).is_empty()).collect();
                        self.player_target.retain(|a| picked.contains(a));
                        self.deferred.clear();
                        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[t]}));
                        self.send_tactics(cx);
                        self.message = format!("Party drafted: {}", TACTIC_LABELS[t]);
                    }
                    RowAct::HeroTactic(t) => {
                        self.hero_tactic = t;
                        if let Some(h) = self.selected {
                            // IF9: the hero tactic clears the hero's picks and its player-set target, then drafts it
                            self.member_tactic.insert(h, t);
                            if ri::on(&self.session().w) {
                                self.player_target.retain(|a| *a != h);
                                self.deferred.retain(|d| d.0 != h);
                                if !rip::picks_of(&self.session().w, h).is_empty() {
                                    self.send(cx, json!({"cmd": "clear", "unit": h}));
                                }
                            }
                            self.send(cx, json!({"cmd": "preset", "unit": h, "preset": TACTICS[t]}));
                            self.send_tactics(cx);
                            self.message = format!("{} drafted: {}", self.session().w.units[h].name, TACTIC_LABELS[t]);
                            if ri::on(&self.session().w) {
                                self.reghost_after(cx, h);
                            }
                        }
                    }
                }
            }
            Action::SelfTarget => {
                if let Some(h) = self.selected.filter(|_| self.planning() && ri::on(&self.session().w)) {
                    self.target_click(cx, h, RoundTarget::Myself);
                }
            }
            Action::ClosePopup => self.popup = None,
            Action::TimelineView => {
                if self.tv_open {
                    self.close_view(cx);
                } else if self.timeline_view_on() {
                    self.tv_open = true;
                }
            }
            Action::PlaybackSpeed(k) => {
                // INPUT1 PB3: the remembered setting (the Settings Battle tab shows it too); leaving Block mode lets a
                // held playback go on (unless the timeline view holds it)
                if let Some((v, label, _)) = PLAYBACK_SPEEDS.get(k).filter(|_| self.playback_aids()) {
                    cx.settings.v[PLAYBACK_SPEED] = json!(v);
                    self.message = match cx.settings.save() {
                        Err(e) => format!("Playback speed {label}, but the setting was not saved: {e}"),
                        Ok(()) => format!("Playback speed: {label} (P cycles)."),
                    };
                }
            }
            Action::NextBlock => {
                // PB3, PB5: a held playback plays the next block (it holds again at that block's end while it should)
                if self.waiting.is_some() {
                    self.release_block();
                }
            }
            Action::TimelinePick(Some(u)) => {
                // TV5: a party track selects its hero (the view stays open; the selected hero's track selects nothing)
                if self.session().w.allies.contains(&u) && self.selected != Some(u) {
                    self.selected = Some(u);
                    self.popup = None;
                }
            }
            Action::TimelinePick(None) => {}
            Action::StayPick(c) => {
                let r = self.send(cx, json!({"cmd": "stay_pick", "unit": c}));
                if r["ok"] == json!(true) {
                    let line = hold_line(self.session());
                    self.message = format!("{line} Plan your round, then confirm.");
                }
            }
            // ENCOUNTER2-C4: a locked round has no Confirm (the script confirmed it; it plays by itself)
            Action::Confirm if self.locked() => {}
            Action::Confirm => {
                // GH2 a, IN6: the ghosts that wait for Confirm are drafted now (their intent change drops its buffs)
                for (m, d) in std::mem::take(&mut self.deferred) {
                    if self.planning() && rip::picks_of(&self.session().w, m).is_empty() {
                        self.send(cx, d.req(m));
                    }
                }
                let r = self.send(cx, json!({"cmd": "confirm"}));
                if r["ok"] == json!(true) {
                    self.playing = true;
                    self.next_at = Instant::now();
                    self.message = "Resolving the round...".into();
                }
            }
            Action::Clear => {
                if let Some(h) = self.selected {
                    if ri::on(&self.session().w) {
                        self.unpick(cx, h, true);
                    } else {
                        self.send(cx, json!({"cmd": "clear", "unit": h}));
                    }
                }
            }
            Action::Undo => {
                if let Some(h) = self.selected {
                    if ri::on(&self.session().w) {
                        self.unpick(cx, h, false);
                    } else {
                        self.send(cx, json!({"cmd": "undo", "unit": h}));
                    }
                }
            }
            Action::Autoplay => {
                self.auto = !self.auto;
                self.auto_after_talk = false;
                if self.auto && self.planning() {
                    self.auto_plan(cx);
                }
            }
            Action::PartyMenu => self.tactic_menu(true),
            Action::HeroMenu => self.tactic_menu(false),
            Action::NextRound => {
                self.pending_next_round = false;
                let r = self.send(cx, json!({"cmd": "advance"}));
                if r["ok"] == json!(true) {
                    self.message = format!("Round {}: plan your party.", self.session().current_round);
                    // VISCLAR1-D: the old round's playback is over: left on, the next tick's `step` would confirm this plan
                    self.playing = false;
                    self.astir_at = Instant::now();
                    if ri::on(&self.session().w) {
                        self.advanced_at = Some(Instant::now());
                    }
                    if self.auto {
                        self.auto_plan(cx);
                    } else if !self.viewing && !self.talk_due() && ri::on(&self.session().w) && self.planning() {
                        // VISCLAR1-D: the ghosts draft with the advance, not a tick later: a click that lands between
                        // them would lose its target to the planning start (`plan_start` resets them). A round-start
                        // talk still goes first: the tick drafts when it closes
                        self.plan_start(cx);
                    }
                }
            }
            Action::Step => {
                // VISCLAR1-D: while the party plans a step would confirm the plan (the host's `step`): with the next round
                // coming by itself, a Space meant for the round's end must not take the plan unseen; the Step button is
                // off there too. Classic stays as it was
                if self.session().phase == Phase::Replay {
                    self.run(cx, Action::NextRound);
                } else if !(self.planning() && ri::on(&self.session().w)) {
                    self.send(cx, json!({"cmd": "step"}));
                }
            }
            Action::Restart => {
                if !self.world_mode() {
                    self.save_session(cx);
                    self.start_battle();
                }
            }
            Action::ToggleLog => {
                self.show_log = !self.show_log;
                self.log_scroll = 0;
            }
            Action::Continue => self.settle(cx),
            Action::Leave => cx.nav.push(Nav::Pop),
            Action::Talk(a) => self.talk_reply(cx, a),
            Action::Skip => {
                if !self.can_skip() {
                    return;
                }
                // ENCOUNTER2-C4: the rest of the locked round at once (a held block's wait ends with it)
                self.waiting = None;
                let r = self.send(cx, json!({"cmd": "resolve"}));
                if r["ok"] == json!(true) {
                    self.playing = false;
                    self.round_over(cx, Instant::now());
                }
            }
        }
    }

    /// Autoplay's plan for the round (the party tactic, then Confirm); ENCOUNTER2-C DT5: held back while a talk is open
    /// or waiting to open (a round-start talk), and sent when the last one closes.
    pub(super) fn auto_plan(&mut self, cx: &mut Cx) {
        if self.talk_due() {
            self.auto_after_talk = true;
            return;
        }
        // ENCOUNTER2-C4: a locked round plays by itself (`tick`); nothing to draft
        if self.locked() {
            return;
        }
        // INPUT1 IF9: autoplay drafts whole rounds (members with picks keep them); no ghost waits for Confirm
        self.deferred.clear();
        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
        self.run(cx, Action::Confirm);
    }

    /// The round's playback is over: its message; `next_round_by_itself` starts the next round after the round-end pause
    /// (only the next round's planning: nothing confirms a plan but autoplay).
    pub(super) fn round_over(&mut self, cx: &Cx, now: Instant) {
        let t = cx.tuning.value.clone();
        let pace = self.pace(cx);
        let s = self.session();
        let can_advance = s.can_advance_round();
        // classic keeps its old flow: only autoplay goes on by itself
        let setting = cx.settings.v[AUTO_CONFIRM].as_bool().unwrap_or(true) && ri::on(&s.w);
        let by_itself = next_round_by_itself(can_advance, self.auto, s.round_locked(), self.viewing, setting);
        let hint = if by_itself { "" } else { " Space (or Next) for the next round." };
        let message = if s.combat_over {
            match s.outcome.as_str() {
                "won" => "Victory!".to_string(),
                "lost" => "Defeat.".to_string(),
                o => format!("Battle over: {o}"),
            }
        } else if !can_advance {
            "Round limit reached.".into()
        } else {
            // RULESET1-U slice 5 (RULESET1 12): the round summary under the round intent
            match round_summary_text(s, &Layers::of(&cx.settings.v["battle_overlays"])) {
                Some(sum) => format!("Round over: {sum}.{hint}"),
                None => format!("Round over.{hint}"),
            }
        };
        self.message = message;
        if by_itself {
            self.next_at = now + Duration::from_millis(t.round_pause_ms * pace);
            self.pending_next_round = true;
        }
    }
}

/// VISCLAR1-D (decision "Battle view answers, part 2"): the next round starts by itself once a round has played: under
/// autoplay; after a locked round (ENCOUNTER2-C4); and, with the Auto-confirm setting, after any other, except while
/// watching a recording (`viewing`), whose rounds the player steps through.
pub(super) fn next_round_by_itself(can_advance: bool, auto: bool, locked: bool, viewing: bool, setting: bool) -> bool {
    can_advance && (auto || (!viewing && (locked || setting)))
}

/// ENCOUNTER2-C DT8: the qualified id of world character `r` in `world` ("" when there is none).
pub(super) fn qid_in(d: &emberwake_world::Data, world: &str, r: &str) -> String {
    s(d.resolve(world, "characters", r), "_qid")
}

/// Whether any enemy unit has the boss tier (MusicDirector.combat_context). Wave and phase units are built when the
/// battle starts (held in reserve), so they count from the first round like Godot's encounter refs.
pub(super) fn boss_fight(s: &Session) -> bool {
    (0..s.w.units.len()).any(|c| !s.w.allies.contains(&c) && s.tier_of(c) == "boss")
}

/// The sound a resolver log line plays (scripts/audio/sound_events.gd FOR_PRESET); None: silent. The engine's log
/// has no mitigation tag, so a hit for 0 plays mitigated; a status that is a buff plays nothing.
pub(super) fn sound_of(line: &str) -> Option<&'static str> {
    let tag = line.split_once(']').map(|(t, _)| t.trim_start_matches('['))?;
    match tag {
        "Attack Miss" | "Area Miss" => Some("miss"),
        "Attack" if line.contains(" CRIT") => Some("crit"),
        "Attack" if line.contains(" for 0 ") => Some("mitigated"),
        "Attack" => Some("hit"),
        "Status" => {
            let id = line.split_once("(inflicts ")?.1.split(')').next()?;
            (!emberwake_rules::status::row_of(id).is_some_and(|r| r.buff)).then_some("debuff")
        }
        "Defend" if !line.contains("guard was broken") => Some("defend"),
        "Focus" => Some("buff"),
        "Support" => Some("heal"),
        "Move" | "Move Cut Short" | "Withdrawal" | "Dash" | "Chase" | "Disengage" => Some("move_step"),
        _ => None,
    }
}
