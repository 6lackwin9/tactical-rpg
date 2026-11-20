//! The full timeline view (INPUT1-T, docs/specs/INPUT1.md 8): one builder for the battle screen's overlay (key T, over
//! the grid panel) and the UI Gallery's "Battle timeline" entry, so both show the same tracks for the same session. It
//! turns a session into the engine widget's value (`aegis_ui::timeline::Timeline`): the ruler of the round's blocks
//! (the playhead during playback), "Party" (allies in roster order, guests too) above "Enemies" (the hostile list's
//! order), one track per unit present or acting in the round, a downed unit's lane hatched.
//!
//! A party track holds the member's plan while planning (picks solid, the planner's ghost in the ghost style, GH8),
//! its committed actions during playback (a broken one with its BREAK mark); its
//! label leads with its intent's letter ("A Vanguard") and its card carries AP9's lines. An enemy track shows only what
//! the party knows (TV4): while planning an unrevealed enemy is masked with "`<Intent>`: `<target>`" (else "Plans in
//! secret") and its tell while the tells show (its target in the row's aside square); during playback it holds only its actions
//! that have started; a revealed one (foresight, enrage, `plan_reveal`) shows its whole plan and the card "Plan revealed
//! (`<reader>`)". VISCLAR1 VC4's Party foresight off masks foresight reveals (the records come through `seen_intents`).
//! Display only: nothing here changes the battle.
//!
//! STATUSVIS1-B (PR7-57, SV13-SV15, takes 34-36, contract 9.3): the calmer view: every span a tile (the engine's
//! `tiles`: a dark tint of its kind colour, a kind band on top, a kind rim) with its skill icon (`Segment.icon`, the
//! icon chain joined by `|`), no wind-up parts and no per-blow ticks; a unit's locked blocks one grey tile per run with
//! the status icon (stun one tile over blocks 1-2, stagger one on 1 and one on 4); each row's one target for the round
//! in its aside square (`row_aims`: crimson an enemy, teal an ally or itself, an amber ring for an area skill's main
//! target, a 3 x 3 floor patch for a ground tile, a dim dash for none).

use crate::battle::{action_rows, ap_lines, channel_action, channel_block, channel_fill, channeled_word, kind_color, seen_intents, skill_text, Layers, CHANNEL_TINT};
use crate::intent_text;
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, Rgba};
use aegis_ui::timeline::{Mark, Section, Segment, Timeline, Track};
use aegis_ui::ui::{Tip};
use emberwake_battle::intent::{self, Intent, ARRIVING, IDLE};
use emberwake_battle::plan::Plan;
use emberwake_battle::resolver::ScheduledAction;
use emberwake_battle::round_intent as ri;
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_battle::status_view::{self as sv, RowAim};
use emberwake_battle::world::World;
use std::collections::BTreeMap;

/// The party's colour (labels, the section title).
use crate::tokens::ally;
/// The enemies' colour.
pub const FOE: Rgba = rgb(240, 100, 92);
/// A Rest span under the round intent (dim, as on the strip).
const REST_FILL: Rgba = rgb(34, 40, 54);
/// A downed unit's label.
const DOWN: Rgba = rgb(132, 146, 172);

/// What the view shows of a session besides the session itself.
#[derive(Clone, Copy)]
pub struct Opts<'a> {
    /// The round on view (the live one; the Gallery pages the round archive).
    pub viewed: i32,
    /// Every enemy unmasked (the Gallery's Reveal button).
    pub reveal_all: bool,
    /// VC4: the debug Party foresight off (a foresight reveal reads unrevealed).
    pub foresight_off: bool,
    /// The Show enemy tells setting (key I): a masked track's note adds the tell.
    pub show_tells: bool,
    /// The overlay groups (`battle_overlays`); the view reads `breaks`.
    pub layers: Layers,
    /// The selected party member (its track is the selected one) and the selected block.
    pub selected: Option<usize>,
    pub block: Option<i32>,
    /// The selected member's round as the strip shows it (a hovered menu row's preview or a ghost waiting for
    /// Confirm, on copies); None: the battle's own.
    pub shown: Option<(usize, &'a World, &'a Plan)>,
    /// INPUT1 PB2: the block the battle's playback resolves (the strip's cursor); None: the session's current block.
    pub playhead: Option<i32>,
}

/// The track key of unit `u` (`u<index>`; the widget ids `<id>.track.u<index>`).
pub fn track_key(u: usize) -> String {
    format!("u{u}")
}

/// The unit of a track key (None: not a unit's track).
pub fn unit_of(key: &str) -> Option<usize> {
    key.strip_prefix('u').and_then(|n| n.parse().ok())
}

/// A segment: its label ("Cut > Wolf"), fill by kind (a ghost in the ghost look, a Rest dim under the round intent),
/// its card (an "Auto-filled" head on a ghost, the blocks, the target, extra lines).
#[allow(clippy::too_many_arguments)]
fn segment(key: String, start: i32, end: i32, name: &str, kind: &str, target: Option<String>, ghost: bool, dim_rest: bool, card: Vec<(String, Rgba)>) -> Segment {
    // the card leads with the label in full (TEXTFIT1: a label cut on its bar reads whole on hover)
    let label = target.map(|t| format!("{name} > {t}")).unwrap_or_else(|| name.to_string());
    let mut tip: Tip = vec![(label.clone(), th().gold), (format!("Blocks {start}-{end}  {kind}"), th().dim)];
    tip.extend(card);
    let rest = dim_rest && kind == "REST";
    let mut fill = if rest { REST_FILL } else { kind_color(kind) };
    fill[3] = 230;
    Segment { key, start, end, label, fill, dashed: kind == "MOVE", tip, ghost: ghost && !rest, ..Segment::default() }
}

/// The committed (or archived) actions of `u` among `actions` as segments (a tile with its skill icon, a lock run one
/// grey tile): a broken one marked BREAK, a cancelled one hatched; `keep` filters the actions (an unrevealed enemy's
/// started ones).
fn action_segments(w: &World, actions: &[ScheduledAction], u: usize, layers: &Layers, keep: impl Fn(&ScheduledAction) -> bool) -> (Vec<Segment>, Vec<Mark>) {
    let mine: Vec<&ScheduledAction> = actions.iter().filter(|a| a.unit == u).collect();
    let rows = action_rows(w, actions, u);
    let timeline = ri::on(w);
    let (mut segs, mut marks) = (Vec::new(), Vec::new());
    for (i, (a, r)) in mine.iter().zip(rows).enumerate().filter(|(_, (a, _))| keep(a)) {
        // SV15: a locked block (its status's lost block) joins its run's one grey tile
        if let emberwake_battle::command::CmdKind::StatusBlock { status } = &a.command.kind {
            push_lock(&mut segs, format!("{i}"), a.start, a.end.max(a.start), status);
            continue;
        }
        let mut s = segment(format!("{i}"), r.0, r.1, &r.2, &r.3, r.4, false, timeline, Vec::new());
        s.icon = w.cat.skill(&a.command.skill).map(|sk| icon_key(&emberwake_content::art::skill_icon_chain(sk.family_id(), &sk.kind)));
        // PK1-S2c (placeholder): an action in a Channel round's mode is tinted
        if channel_action(w, a) {
            channel_look(w, &mut s);
        }
        s.hatched = a.cancelled;
        if a.broken && layers.breaks {
            s.tip.push(("BREAK: cancelled by a poise break".into(), FOE));
            marks.push(Mark { key: format!("break{i}"), block: a.start, glyph: "BREAK".into(), colour: FOE, tip: vec![("BREAK".into(), FOE), (format!("{}: cancelled by a poise break (AP paid stays spent)", r.2), th().text)] });
        }
        segs.push(s);
    }
    (segs, marks)
}

/// Member `u`'s plan while planning (on `pw`, `plan`): picks solid, the planner's ghost after them, each with its reason
/// tag and AP, each a tile with its skill icon (SV14: no wind-up parts).
fn plan_segments(pw: &World, plan: &Plan, u: usize) -> Vec<Segment> {
    let timeline = ri::on(pw);
    let entries = plan.compile(pw, u).entries;
    let (ghosts, reasons) = if timeline { (rip::ghost_entries(pw, plan, u), rip::reasons(pw, plan, u)) } else { (Vec::new(), Vec::new()) };
    let mut out: Vec<Segment> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        // SV15: a locked block joins its run's one grey tile
        if !e.status_name.is_empty() {
            push_lock(&mut out, format!("{i}"), e.start_block, e.end_block(pw), &e.status_name);
            continue;
        }
        out.push({
            let sk = &pw.cat.skills[e.skill];
            let ghost = ghosts.get(i).copied().unwrap_or(false);
            let why = reasons.get(i).filter(|r| !r.is_empty());
            let mut card = Vec::new();
            if e.total_ap() > 0 {
                card.push((format!("{} AP", e.total_ap()), ally()));
            }
            if let Some(why) = why.filter(|_| !ghost) {
                card.push((format!("Why: {why}"), th().dim));
            }
            // PK1-S2c (PK1 4.3 item 6): a marrow version's name with the marrow mark, its span tinted (placeholder)
            let name = if e.item.is_some() { sk.name.clone() } else { skill_text(pw, u, e.skill) };
            let mut s = segment(format!("{i}"), e.start_block, e.end_block(pw), &name, &sk.kind, e.target.filter(|t| *t != u).map(|t| pw.units[t].name.clone()), ghost, timeline, card);
            if e.item.is_none() && channel_block(pw, u, sk) {
                channel_look(pw, &mut s);
            }
            // GH8: a ghost's card starts "Auto-filled" with its reason tag
            if s.ghost {
                s.tip.insert(0, (why.map_or_else(|| "Auto-filled".to_string(), |r| format!("Auto-filled: {r}")), th().dim));
            }
            let chain = if e.item.is_some() { vec![crate::battle::kind_icon("ITEM")] } else { emberwake_content::art::skill_icon_chain(sk.family_id(), &sk.kind) };
            s.icon = Some(icon_key(&chain));
            s
        });
    }
    out
}

/// SV13: a segment's icon key: its icon chain (the first file there is drawn) joined by `|` (`icon_chain`).
pub fn icon_key(chain: &[String]) -> String {
    chain.join("|")
}

/// The icon chain of an icon key (`icon_key`).
pub fn icon_chain(key: &str) -> Vec<String> {
    key.split('|').filter(|p| !p.is_empty()).map(str::to_string).collect()
}

/// SV15: lock block `start`-`end` of `status` on a track: it extends the last segment when that is the same status's
/// tile ending the block before, else it starts a grey tile with the status icon (its card: the status, its blocks,
/// no plan there).
fn push_lock(segs: &mut Vec<Segment>, key: String, start: i32, end: i32, status: &str) {
    let icon = Some(icon_key(&[emberwake_content::art::status_icon(status)]));
    if let Some(last) = segs.last_mut().filter(|l| l.icon == icon && l.key.starts_with("lock") && l.end + 1 == start) {
        last.end = end;
        last.tip = lock_card(status, last.start, end);
        return;
    }
    let name = emberwake_rules::status::display_name(status);
    segs.push(Segment { key: format!("lock{key}"), start, end, label: name, fill: crate::tokens::lock_block(), tip: lock_card(status, start, end), icon, ..Segment::default() });
}

/// SV15: a lock tile's card.
fn lock_card(status: &str, start: i32, end: i32) -> Tip {
    let blocks = if start == end { format!("Block {start} locked") } else { format!("Blocks {start}-{end} locked") };
    vec![(emberwake_rules::status::display_name(status), FOE), (blocks, th().text), ("No move can be planned here.".into(), th().dim)]
}

/// SV14, SV15: each unit track's one target for the round (`status_view::row_aim` over the round as it will run: the
/// shown member's plan while planning). A masked enemy (TV4: planning, unrevealed) reads its round state's target only,
/// which the masked note already names: never its scheduled actions, so the row shows no area or ground shape the
/// party does not know.
pub fn row_aims(s: &Session, o: &Opts) -> BTreeMap<String, RowAim> {
    let w = &s.w;
    let live = o.viewed == s.current_round;
    let planning = live && s.phase == Phase::Planning && !s.plan_confirmed;
    let actions: Vec<ScheduledAction> = if live { s.resolver.scheduled.clone() } else { s.round_record(o.viewed).map(|r| r.actions.clone()).unwrap_or_default() };
    let planned = if planning { intent::planned_schedule(w, &actions, &s.plan) } else { actions.clone() };
    let tells = if planning { seen_intents(w, s.enemy_intents(), o.foresight_off) } else { Vec::new() };
    let mut out = BTreeMap::new();
    for &u in w.allies.iter().chain(w.enemies.iter()) {
        let revealed = o.reveal_all || tells.iter().find(|t| t.unit == u).is_some_and(|t| t.revealed);
        let masked = planning && w.enemies.contains(&u) && w.units[u].is_active() && !revealed;
        let aim = if masked {
            let target = match w.round_states.get(&u).and_then(|r| r.target) {
                Some(ri::RoundTarget::Unit(t)) if w.enemies.contains(&t) => sv::RowTarget::Ally(t),
                Some(ri::RoundTarget::Unit(t)) => sv::RowTarget::Foe(t),
                Some(ri::RoundTarget::Cell(c)) => sv::RowTarget::Ground(c),
                Some(ri::RoundTarget::Myself) => sv::RowTarget::Myself,
                None => sv::RowTarget::None,
            };
            RowAim { target, area: false }
        } else {
            match o.shown {
                Some((m, pw, pp)) if m == u && planning => sv::row_aim(pw, &intent::planned_schedule(pw, &actions, pp), u),
                _ => sv::row_aim(w, &planned, u),
            }
        };
        out.insert(track_key(u), aim);
    }
    out
}

/// PK1-S2c (PK1 4.3 item 6, placeholder, user 2026-11-10): a span in a Channel round's mode: its fill blended toward
/// the Channel tint and a card line saying so (PR5-32: the Channel label and the channeled mark's word are the game's
/// terms: "Channel round: marrow").
fn channel_look(w: &World, s: &mut Segment) {
    let a = s.fill[3];
    s.fill = channel_fill(s.fill);
    s.fill[3] = a;
    s.tip.push((format!("{} round: {}", emberwake_battle::round_intent::RoundIntent::Channel.label(&w.cat), channeled_word(w)), CHANNEL_TINT));
}

/// TV4: a masked enemy's note: its round intent and target ("Assault: Vanguard"; without one "Plans in secret"), then
/// its tell while the tells show ("; Next: Cut B1"; "; Holding" for one with no action).
pub fn masked_note(w: &World, rec: Option<&Intent>, show_tells: bool) -> String {
    let Some(r) = rec else { return "Plans in secret".into() };
    let head = intent_text::round_head(w, r).unwrap_or_else(|| "Plans in secret".into());
    if !show_tells {
        return head;
    }
    if r.line.kind == ARRIVING || r.line.kind == IDLE {
        format!("{head}; {}", intent_text::core(w, &r.line))
    } else {
        format!("{head}; Next: {} {}", r.line.skill, intent_text::block_text(r.line.blocks))
    }
}

/// "Plan revealed (Read by Sella (Hawk Eye))", "Plan revealed (enraged)", "Plan revealed (plan reveal)".
fn reveal_card(w: &World, r: &Intent) -> String {
    let why = if !r.readers.is_empty() {
        intent_text::read_by(w, r, |id| w.cat.passives.get(id).map(|p| p.name.clone()).unwrap_or_else(|| id.to_string()))
    } else if r.enraged {
        "enraged".into()
    } else {
        "plan reveal".into()
    };
    format!("Plan revealed ({why})")
}

/// The view of `s` (TV2-TV4): the same value for the battle's overlay and the Gallery's Battle timeline.
pub fn build(s: &Session, o: &Opts) -> Timeline {
    let w = &s.w;
    let live = o.viewed == s.current_round;
    let planning = live && s.phase == Phase::Planning && !s.plan_confirmed;
    let actions: Vec<ScheduledAction> = if live { s.resolver.scheduled.clone() } else { s.round_record(o.viewed).map(|r| r.actions.clone()).unwrap_or_default() };
    let blocks = s.def.total_blocks.max(1);
    // PB2: the playhead marks the strip's resolving block (the session's current block without one)
    let at = o.playhead.unwrap_or(s.current_block);
    let playhead = if live && !planning && s.phase != Phase::Replay { at } else { 0 };
    // the records the screen shows (VC4): the tells while planning, the round records otherwise
    let tells = if planning { seen_intents(w, s.enemy_intents(), o.foresight_off) } else { Vec::new() };
    let rounds = seen_intents(w, s.round_intents(), o.foresight_off);
    let rec_of = |u: usize| tells.iter().find(|t| t.unit == u).or_else(|| rounds.iter().find(|t| t.unit == u));
    let acts = |u: usize| actions.iter().any(|a| a.unit == u);
    let timeline = ri::on(w);
    let party: Vec<Track> = w
        .allies
        .iter()
        .copied()
        .filter(|&u| w.has(u) || acts(u))
        .map(|u| {
            let unit = &w.units[u];
            let (pw, plan) = match o.shown {
                Some((m, pw, pp)) if m == u => (pw, pp),
                _ => (w, &s.plan),
            };
            let (segments, marks) = if planning && unit.is_active() { (plan_segments(pw, plan, u), Vec::new()) } else { action_segments(w, &actions, u, &o.layers, |_| true) };
            let letter = ri::intent_of(pw, u).filter(|_| timeline && live && unit.is_active()).map(|i| format!("{} ", i.label(&pw.cat).chars().next().unwrap_or(' '))).unwrap_or_default();
            let label = format!("{letter}{}", unit.name);
            let mut tip: Tip = vec![(label.clone(), th().gold)];
            if let Some(i) = ri::intent_of(pw, u).filter(|_| timeline && live) {
                tip.push((format!("Intent: {}", i.label(&pw.cat)), th().text));
            }
            // TV3: AP9's lines while planning (the gauge's forecast), the live AP otherwise
            if timeline && planning && unit.is_active() {
                tip.extend(ap_lines(&rip::ap_forecast(pw, plan, u)).into_iter().map(|l| (l, ally())));
            } else {
                tip.push((crate::intent_text::ap_text(unit.ap, unit.max_ap, " of "), ally()));
            }
            if !unit.is_alive() {
                tip.push(("Down".into(), FOE));
            }
            Track { key: track_key(u), label, colour: if unit.is_active() { ally() } else { DOWN }, segments, marks, tip, hatched: !unit.is_alive(), aside: true, ..Track::default() }
        })
        .collect();
    let enemies: Vec<Track> = w
        .enemies
        .iter()
        .copied()
        .filter(|&e| w.has(e) || acts(e))
        .map(|e| {
            let unit = &w.units[e];
            let rec = rec_of(e);
            let revealed = !live || o.reveal_all || rec.is_some_and(|r| r.revealed);
            let mut tip: Tip = vec![(unit.name.clone(), th().gold)];
            let mut track = Track { key: track_key(e), label: unit.name.clone(), colour: if unit.is_active() { FOE } else { DOWN }, hatched: !unit.is_alive(), aside: true, ..Track::default() };
            if planning && unit.is_active() && !revealed {
                // TV4: masked: the round intent and target, the tell; none of the plan's skills
                let note = masked_note(w, rec, o.show_tells);
                tip.push((note.clone(), th().text));
                track.masked = Some(note);
                // SV14 (take 35): no per-blow ticks in the calmer view; the masked note names the intent and target
            } else {
                // playback: an unrevealed enemy's actions that have started (block <= the playhead), a revealed one's all
                let upto = if live && !revealed && s.phase != Phase::Replay { at } else { i32::MAX };
                let (segs, marks) = action_segments(w, &actions, e, &o.layers, |a| a.start <= upto);
                track.segments = segs;
                track.marks = marks;
                match rec.filter(|r| r.revealed && live && !o.reveal_all) {
                    Some(r) => tip.push((reveal_card(w, r), th().gold)),
                    None if live && !revealed => tip.push(("Plans in secret: its actions show as they start.".into(), th().dim)),
                    None => {}
                }
            }
            if !unit.is_alive() {
                tip.push(("Down".into(), FOE));
            }
            track.tip = tip;
            track
        })
        .collect();
    Timeline {
        blocks,
        playhead,
        selected_block: o.block,
        selected_track: o.selected.filter(|u| w.allies.contains(u)).map(track_key),
        sections: vec![Section { title: "Party".into(), colour: ally(), tracks: party }, Section { title: "Enemies".into(), colour: FOE, tracks: enemies }],
        ruler: None,
        tiles: true,
    }
}

/// The agent link's `timeline_view`: the playhead, the selected track and each section's tracks (key, label, masked
/// note, hatched, spans with their blocks and ghost flag, marks).
pub fn json(t: &Timeline) -> serde_json::Value {
    use serde_json::json;
    let track = |x: &Track| {
        json!({"key": x.key, "label": x.label, "masked": x.masked, "hatched": x.hatched,
            "segments": x.segments.iter().map(|s| json!({"key": s.key, "start": s.start, "end": s.end, "label": s.label, "ghost": s.ghost, "hatched": s.hatched})).collect::<Vec<_>>(),
            "marks": x.marks.iter().map(|m| json!({"block": m.block, "glyph": m.glyph})).collect::<Vec<_>>()})
    };
    json!({"blocks": t.blocks, "playhead": t.playhead, "selected_track": t.selected_track,
        "sections": t.sections.iter().map(|s| json!({"title": s.title, "tracks": s.tracks.iter().map(track).collect::<Vec<_>>()})).collect::<Vec<_>>()})
}
