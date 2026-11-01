//! The full timeline view (INPUT1-T, docs/specs/INPUT1.md 8): one builder for the battle screen's overlay (key T, over
//! the grid panel) and the UI Gallery's "Battle timeline" entry, so both show the same tracks for the same session. It
//! turns a session into the engine widget's value (`aegis_ui::timeline::Timeline`): the ruler of the round's blocks
//! (the playhead during playback), "Party" (allies in roster order, guests too) above "Enemies" (the hostile list's
//! order), one track per unit present or acting in the round, a downed unit's lane hatched.
//!
//! A party track holds the member's plan while planning (picks solid, the planner's ghost in the ghost style, GH8; the
//! wind-up, impact and recovery parts and BREAK marks as on the strip), its committed actions during playback; its
//! label leads with its intent's letter ("A Vanguard") and its card carries AP9's lines. An enemy track shows only what
//! the party knows (TV4): while planning an unrevealed enemy is masked with "`<Intent>`: `<target>`" (else "Plans in
//! secret") and its tell while the tells show, its impact badges as marks; during playback it holds only its actions
//! that have started; a revealed one (foresight, enrage, `plan_reveal`) shows its whole plan and the card "Plan revealed
//! (`<reader>`)". VISCLAR1 VC4's Party foresight off masks foresight reveals (the records come through `seen_intents`).
//! Display only: nothing here changes the battle.

use crate::battle::{action_rows, ap_lines, kind_color, seen_intents, Layers};
use crate::intent_text;
use aegis_ui::canvas::{rgb, Rgba};
use aegis_ui::timeline::{Mark, Part, PartStyle, Section, Segment, Timeline, Track};
use aegis_ui::ui::{Tip, DIM, GOLD, TEXT};
use emberwake_battle::intent::{Intent, ARRIVING, HIT, IDLE};
use emberwake_battle::plan::Plan;
use emberwake_battle::resolver::ScheduledAction;
use emberwake_battle::round_intent as ri;
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_battle::world::World;

/// The party's colour (labels, the section title).
pub const ALLY: Rgba = rgb(110, 170, 255);
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
    /// The overlay groups `parts`, `breaks` and `badges` (`battle_overlays`).
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

/// A span's styled runs: the wind-up before its impact shaded, the impact bright, the blocks after it to `end`
/// hatched (RULESET1-U slice 5; the strip's spans use it too).
pub fn parts_of(start: i32, impact: i32, end: i32, winds: bool) -> Vec<Part> {
    let mut out = Vec::new();
    if winds && impact > start {
        out.push(Part { start, end: impact - 1, style: PartStyle::Windup });
    }
    if winds && (impact > start || end > impact) {
        out.push(Part { start: impact, end: impact, style: PartStyle::Impact });
    }
    if end > impact {
        out.push(Part { start: impact + 1, end, style: PartStyle::Recovery });
    }
    out
}

/// A segment: its label ("Cut > Wolf"), fill by kind (a ghost in the ghost look, a Rest dim under the round intent),
/// its card (an "Auto-filled" head on a ghost, the blocks, the target, extra lines).
#[allow(clippy::too_many_arguments)]
fn segment(key: String, start: i32, end: i32, name: &str, kind: &str, target: Option<String>, ghost: bool, dim_rest: bool, card: Vec<(String, Rgba)>) -> Segment {
    // the card leads with the label in full (TEXTFIT1: a label cut on its bar reads whole on hover)
    let label = target.map(|t| format!("{name} > {t}")).unwrap_or_else(|| name.to_string());
    let mut tip: Tip = vec![(label.clone(), GOLD), (format!("Blocks {start}-{end}  {kind}"), DIM)];
    tip.extend(card);
    let rest = dim_rest && kind == "REST";
    let mut fill = if rest { REST_FILL } else { kind_color(kind) };
    fill[3] = 230;
    Segment { key, start, end, label, fill, dashed: kind == "MOVE", tip, ghost: ghost && !rest, ..Segment::default() }
}

/// The committed (or archived) actions of `u` among `actions` as segments: parts from each action's impact, a broken
/// one marked BREAK, a cancelled one hatched; `keep` filters the actions (an unrevealed enemy's started ones).
fn action_segments(w: &World, actions: &[ScheduledAction], u: usize, layers: &Layers, keep: impl Fn(&ScheduledAction) -> bool) -> (Vec<Segment>, Vec<Mark>) {
    let mine: Vec<&ScheduledAction> = actions.iter().filter(|a| a.unit == u).collect();
    let rows = action_rows(w, actions, u);
    let timeline = ri::on(w);
    let (mut segs, mut marks) = (Vec::new(), Vec::new());
    for (i, (a, r)) in mine.iter().zip(rows).enumerate().filter(|(_, (a, _))| keep(a)) {
        let mut s = segment(format!("{i}"), r.0, r.1, &r.2, &r.3, r.4, false, timeline, Vec::new());
        if layers.parts && timeline {
            s.parts = parts_of(a.start, a.impact, a.end, emberwake_battle::poise::winds_up(&a.command));
        }
        s.hatched = a.cancelled;
        if a.broken && layers.breaks {
            s.tip.push(("BREAK: cancelled by a poise break".into(), FOE));
            marks.push(Mark { key: format!("break{i}"), block: a.start, glyph: "BREAK".into(), colour: FOE, tip: vec![("BREAK".into(), FOE), (format!("{}: cancelled by a poise break (AP paid stays spent)", r.2), TEXT)] });
        }
        segs.push(s);
    }
    (segs, marks)
}

/// Member `u`'s plan while planning (on `pw`, `plan`): picks solid, the planner's ghost after them, each with its reason
/// tag and AP; parts as the strip draws them (impact at the span's end).
fn plan_segments(pw: &World, plan: &Plan, u: usize, layers: &Layers) -> Vec<Segment> {
    let timeline = ri::on(pw);
    let entries = plan.compile(pw, u).entries;
    let (ghosts, reasons) = if timeline { (rip::ghost_entries(pw, plan, u), rip::reasons(pw, plan, u)) } else { (Vec::new(), Vec::new()) };
    entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let sk = &pw.cat.skills[e.skill];
            let ghost = ghosts.get(i).copied().unwrap_or(false);
            let why = reasons.get(i).filter(|r| !r.is_empty());
            let mut card = Vec::new();
            if e.total_ap() > 0 {
                card.push((format!("{} AP", e.total_ap()), ALLY));
            }
            if let Some(why) = why.filter(|_| !ghost) {
                card.push((format!("Why: {why}"), DIM));
            }
            let mut s = segment(format!("{i}"), e.start_block, e.end_block(pw), &sk.name, &sk.kind, e.target.filter(|t| *t != u).map(|t| pw.units[t].name.clone()), ghost, timeline, card);
            // GH8: a ghost's card starts "Auto-filled" with its reason tag
            if s.ghost {
                s.tip.insert(0, (why.map_or_else(|| "Auto-filled".to_string(), |r| format!("Auto-filled: {r}")), DIM));
            }
            if layers.parts && timeline {
                s.parts = parts_of(e.start_block, e.end_block(pw), e.end_block(pw), emberwake_battle::poise::winds_up(&e.build_command(pw, u)));
            }
            s
        })
        .collect()
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

/// V1 as marks: a red "!" at each hit's impact block, a gold "*" at any other action's.
fn impact_marks(r: &Intent, blocks: i32) -> Vec<Mark> {
    r.impacts
        .iter()
        .filter(|(b, _)| (1..=blocks).contains(b))
        .enumerate()
        .map(|(k, &(b, kind))| {
            let hit = kind == HIT;
            let col = if hit { FOE } else { GOLD };
            Mark { key: format!("i{k}"), block: b, glyph: if hit { "!" } else { "*" }.into(), colour: col, tip: vec![(format!("B{b}"), col), (if hit { "A hit lands." } else { "An action lands." }.into(), TEXT)] }
        })
        .collect()
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
            let (segments, marks) = if planning && unit.is_active() { (plan_segments(pw, plan, u, &o.layers), Vec::new()) } else { action_segments(w, &actions, u, &o.layers, |_| true) };
            let letter = ri::intent_of(pw, u).filter(|_| timeline && live && unit.is_active()).map(|i| format!("{} ", i.name().chars().next().unwrap_or(' '))).unwrap_or_default();
            let label = format!("{letter}{}", unit.name);
            let mut tip: Tip = vec![(label.clone(), GOLD)];
            if let Some(i) = ri::intent_of(pw, u).filter(|_| timeline && live) {
                tip.push((format!("Intent: {}", i.name()), TEXT));
            }
            // TV3: AP9's lines while planning (the gauge's forecast), the live AP otherwise
            if timeline && planning && unit.is_active() {
                tip.extend(ap_lines(&rip::ap_forecast(pw, plan, u)).into_iter().map(|l| (l, ALLY)));
            } else {
                tip.push((format!("AP {} of {}", unit.ap, unit.max_ap), ALLY));
            }
            if !unit.is_alive() {
                tip.push(("Down".into(), FOE));
            }
            Track { key: track_key(u), label, colour: if unit.is_active() { ALLY } else { DOWN }, segments, marks, tip, hatched: !unit.is_alive(), ..Track::default() }
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
            let mut tip: Tip = vec![(unit.name.clone(), GOLD)];
            let mut track = Track { key: track_key(e), label: unit.name.clone(), colour: if unit.is_active() { FOE } else { DOWN }, hatched: !unit.is_alive(), ..Track::default() };
            if planning && unit.is_active() && !revealed {
                // TV4: masked: the round intent and target, the tell, the impact badges; none of the plan's skills
                let note = masked_note(w, rec, o.show_tells);
                tip.push((note.clone(), TEXT));
                track.masked = Some(note);
                if let Some(r) = rec.filter(|_| emberwake_battle::intent::badges_on(w) && o.layers.badges) {
                    track.marks = impact_marks(r, blocks);
                }
            } else {
                // playback: an unrevealed enemy's actions that have started (block <= the playhead), a revealed one's all
                let upto = if live && !revealed && s.phase != Phase::Replay { at } else { i32::MAX };
                let (segs, marks) = action_segments(w, &actions, e, &o.layers, |a| a.start <= upto);
                track.segments = segs;
                track.marks = marks;
                match rec.filter(|r| r.revealed && live && !o.reveal_all) {
                    Some(r) => tip.push((reveal_card(w, r), GOLD)),
                    None if live && !revealed => tip.push(("Plans in secret: its actions show as they start.".into(), DIM)),
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
        sections: vec![Section { title: "Party".into(), colour: ALLY, tracks: party }, Section { title: "Enemies".into(), colour: FOE, tracks: enemies }],
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
