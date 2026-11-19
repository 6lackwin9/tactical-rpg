//! TRANSITIONS1 (`PR7-14`) on the battle screen: T1, the battle beat a story fight opens with (the ash cover and wipe,
//! the combat sting, the fight's banner, the party's walk-in; control once it ends), and T2, a won world fight's
//! victory beat (the last hit in slow motion, the Victory banner, then the results). Any press during a beat skips it
//! (`skip_beat`); the timing is `crate::transitions`'.

use super::{Action, BattleScreen, Mode};
use crate::screens::Cx;
use crate::transitions::{self as tr, Beat, IntroCard, Kind};
use aegis_core::clock::Instant;
use aegis_core::json::{s, s_or};
use aegis_ui::audio::Bus;
use aegis_ui::canvas::Rect;
use aegis_ui::ui::{Ui, H, W};

impl BattleScreen {
    /// T1: the battle beat starts (Reduce motion: the plain fade), its banner read from the battle: the encounter's
    /// name, its objective line and the enemies on the field; the combat sting plays.
    pub(super) fn start_intro(&mut self, cx: &mut Cx) {
        let Some(sess) = self.host.session.as_ref() else { return };
        let Mode::World { fight, .. } = &self.mode else { return };
        let world = cx.gs().world_id.clone();
        let name = s_or(cx.data.resolve(&world, "encounters", &s(fight, "encounter")), "name", "Enemies");
        let objective = emberwake_battle::encounter::Objectives::text_of(&sess.def.raw, |r| sess.units_of_ref(r).first().map(|&c| sess.w.units[c].name.clone()).unwrap_or_else(|| r.to_string()));
        let foes = sess.w.units.iter().enumerate().filter(|(i, u)| !sess.w.allies.contains(i) && u.is_alive()).count();
        self.intro_card = IntroCard { name, objective, foes };
        self.beat = Some(Beat::new(Kind::Intro, cx.settings.reduce_motion()));
        self.walked = false;
        cx.audio.play(Bus::Sfx, tr::COMBAT_STING);
    }

    /// T1 each tick: on the first, the tokens catch up with the battle and the party's walk-in is set (each ally waits
    /// off its cell until `WALK_AT_MS` on the FX clock, then walks for `WALK_MS`; none with Reduce motion); the beat ends
    /// as control starts. True while it runs (the battle waits).
    pub(super) fn intro_tick(&mut self, cx: &mut Cx, now: Instant) -> bool {
        let Some(b) = self.beat.filter(|b| b.kind == Kind::Intro) else { return false };
        if !self.walked {
            self.walked = true;
            if b.walks() {
                self.sync_anims(cx);
                let at = self.clock.now() + tr::WALK_AT_MS.saturating_sub(b.ms(now));
                if let Some(sess) = self.host.session.as_ref() {
                    self.tokens.walk_in(sess, at, tr::WALK_MS, tr::WALK_CELLS);
                }
            }
        }
        if b.done(now) {
            self.beat = None;
            return false;
        }
        true
    }

    /// T2 each tick: a won world fight (not a recording watched) starts the victory beat once; when it ends the battle
    /// settles and the results open. True while it runs.
    pub(super) fn victory_tick(&mut self, cx: &mut Cx, now: Instant) -> bool {
        if self.beat.is_none() && !self.victory_seen && self.world_mode() && !self.viewing && self.outcome() == "won" {
            self.victory_seen = true;
            self.beat = Some(Beat::new(Kind::Victory, cx.settings.reduce_motion()));
            self.playing = false;
            self.auto = false;
        }
        let Some(b) = self.beat.filter(|b| b.kind == Kind::Victory) else { return false };
        if b.done(now) {
            self.beat = None;
            self.run(cx, Action::Continue);
        }
        true
    }

    /// A press during a beat: T1 ends at once (the walk-in lands), T2 opens the results now.
    pub(super) fn skip_beat(&mut self, cx: &mut Cx) {
        match self.beat.take().map(|b| b.kind) {
            Some(Kind::Intro) => {
                self.walked = true;
                self.tokens.end_walk_in();
            }
            Some(Kind::Victory) => self.run(cx, Action::Continue),
            None => {}
        }
    }

    /// The beat over the battle, last on the screen: T1's cover or plain fade and its banner, T2's Victory banner; a
    /// full-screen widget takes the click that skips it.
    pub(super) fn draw_beat(&mut self, ui: &mut Ui<Action>, cx: &mut Cx, now: Instant) {
        let Some(b) = self.beat else { return };
        let f = b.frame(now);
        self.ash.draw(&mut ui.p, &mut cx.art, f.cover);
        tr::draw_fade(&mut ui.p, f.fade);
        if let Some(at) = f.banner {
            match b.kind {
                Kind::Intro => tr::draw_intro_banner(&mut ui.p, &mut cx.art, &self.intro_card, at),
                Kind::Victory => tr::draw_victory_banner(&mut ui.p, &mut cx.art, at),
            }
        }
        let tip = if b.kind == Kind::Intro { "Click to start the fight now." } else { "Click for the results." };
        ui.push("battle.beat", "backdrop", tip, Rect::new(0, 0, W, H), true, false, Some(Action::SkipBeat), Vec::new());
    }
}
