//! Battle talk on the screen (ENCOUNTER2-C2): the shared talk box opened by each new talk cue, its replies and the speaker's token.

use super::flow::qid_in;
use super::BattleScreen;
use crate::talk::TalkAct;
use crate::screens::Cx;
use aegis_core::json::{self, Dict};
use emberwake_world::dialogue;
use std::time::{Duration, Instant};

/// ENCOUNTER2-C DT6: an open battle talk: the tree it plays and its runner (a scene: no services).
pub(super) struct BattleTalk {
    pub(super) dialogue: String,
    pub(super) runner: dialogue::DialogueRunner,
}

impl BattleScreen {
    // ---- ENCOUNTER2-C2: the battle talk (docs/specs/ENCOUNTER2_C.md 3.3) ----------------------------------------

    /// A new battle: no talk open, every cue of the session still to open, the Arena's scratch state dropped.
    pub(super) fn reset_talks(&mut self) {
        self.talk = None;
        self.talks_seen = 0;
        self.talk_gs = None;
        self.auto_after_talk = false;
    }

    /// A talk is open, or a cue of the session waits to open.
    pub(super) fn talk_due(&self) -> bool {
        self.talk.is_some() || self.host.session.as_ref().is_some_and(|s| s.talks().len() > self.talks_seen)
    }

    /// DT5: once the impacts of the step that raised it have played (rule P's wait), the next cue opens; true while a
    /// talk is open or a cue waits (the tick sends nothing). With none left, autoplay's held plan goes out.
    pub(super) fn talk_tick(&mut self, cx: &mut Cx, now: Instant) -> bool {
        if self.session().talks().len() < self.talks_seen {
            // another battle began under the screen
            self.reset_talks();
        }
        while self.talk.is_none() && self.talks_seen < self.session().talks().len() {
            if !self.impacts_done(now) {
                return true;
            }
            let cue = self.session().talks()[self.talks_seen].clone();
            self.talks_seen += 1;
            self.open_talk(cx, &cue);
        }
        if self.talk.is_some() {
            return true;
        }
        if std::mem::take(&mut self.auto_after_talk) && self.auto && self.planning() {
            self.auto_plan(cx);
        }
        false
    }

    /// DT6: opens `cue`'s scene (`DialogueRunner::open_scene`) in the game in play (a world fight) or the Arena's
    /// scratch game state of its world (`world_id` set, nothing else); one that cannot open is skipped, its reason in
    /// the message bar.
    fn open_talk(&mut self, cx: &mut Cx, cue: &emberwake_battle::encounter::TalkCue) {
        let d = cx.data.clone();
        let runner = if self.world_mode() {
            if !cx.has_game() {
                return;
            }
            dialogue::DialogueRunner::open_scene(&d, cx.gs_mut(), &cue.dialogue, &cue.speaker)
        } else {
            let world = self.arena_world.clone().unwrap_or_default();
            let gs = self.talk_gs.get_or_insert_with(|| emberwake_world::GameState { world_id: world, ..Default::default() });
            dialogue::DialogueRunner::open_scene(&d, gs, &cue.dialogue, &cue.speaker)
        };
        if runner.closed {
            self.message = if runner.errors.is_empty() { format!("The talk '{}' did not open.", cue.dialogue) } else { runner.errors.join(" ") };
            return;
        }
        self.popup = None;
        self.talk = Some(BattleTalk { dialogue: cue.dialogue.clone(), runner });
    }

    /// The UI Gallery's Battle talk entry: opens tree `dialogue` of `world` (speaker `speaker`) over this battle as a
    /// cue would, in a scratch game state of that world; false (the reason in the message bar) when it cannot open.
    pub(crate) fn preview_talk(&mut self, cx: &mut Cx, world: &str, dialogue: &str, speaker: &str) -> bool {
        self.talk_gs = Some(emberwake_world::GameState { world_id: world.to_string(), ..Default::default() });
        let cue = emberwake_battle::encounter::TalkCue { phase: "preview".into(), dialogue: dialogue.into(), speaker: speaker.into(), round: 1, block: 0 };
        self.open_talk(cx, &cue);
        self.talk.is_some()
    }

    /// The game state the open talk runs in (a clone; None without a talk).
    pub(super) fn talk_state_of(&self, cx: &Cx) -> Option<emberwake_world::GameState> {
        self.talk.as_ref()?;
        Some(if self.world_mode() { cx.gs().clone() } else { self.talk_gs.clone().unwrap_or_default() })
    }

    /// The open talk's view in `gs` (None without one).
    pub(super) fn talk_view(&self, cx: &Cx, gs: &emberwake_world::GameState) -> Option<Dict> {
        self.talk.as_ref().map(|t| t.runner.view(&cx.data, gs))
    }

    /// DT9: a reply or Continue in the talk (its effects, flags only, on the talk's game state); DT10: once the scene
    /// closes the battle goes on where the cue found it, the playback after `impact_gap_ms`.
    pub(super) fn talk_reply(&mut self, cx: &mut Cx, a: TalkAct) {
        let d = cx.data.clone();
        let world = self.world_mode();
        let Some(t) = self.talk.as_mut() else { return };
        let gs = if world { cx.gs_mut() } else { self.talk_gs.get_or_insert_with(Default::default) };
        let res = match a {
            TalkAct::Choose(i) => t.runner.choose(&d, gs, i),
            TalkAct::Advance => t.runner.advance(&d, gs),
        };
        let closed = t.runner.closed;
        if !json::b(&res, "ok", false) {
            self.message = json::sget(&res, "errors").join(" ");
        }
        if closed {
            self.talk = None;
            self.gap_until = Some(Instant::now() + Duration::from_millis(cx.tuning.value.impact_gap_ms * self.pace(cx)));
        }
    }

    /// DT8: the token of the open node's speaker (the character's qualified id `speaker`): the unit on the field whose
    /// hero key is it, names it, or is the party member who plays it.
    pub(super) fn speaker_unit(&self, cx: &Cx, gs: &emberwake_world::GameState, speaker: &str) -> Option<usize> {
        let s = self.host.session.as_ref()?;
        if speaker.is_empty() {
            return None;
        }
        let qid_of = |r: &str| qid_in(&cx.data, &gs.world_id, r);
        (0..s.w.units.len()).filter(|&c| s.w.grid.pos(c).is_some()).find(|&c| {
            let key = s.hero_key(c);
            !key.is_empty() && (key == speaker || qid_of(&key) == speaker || gs.party.members.iter().any(|m| m.id == key && qid_of(&m.lore_ref) == speaker))
        })
    }
}
