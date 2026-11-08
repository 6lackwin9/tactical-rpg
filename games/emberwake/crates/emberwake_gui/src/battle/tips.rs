//! Hover cards and the text and colours of a unit's plans: intent chips and badges, the strip's action rows, unit tips, the AP gauge and its lines.

use aegis_ui::theme::th;
use super::board::with_alpha;
use super::marks::{held_guard, meter_text, HELD, METER};
use super::overlays::reach_text;
use super::{ALLY, AP_ANIM_MS, AP_PIPS, POISE};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::intent_text;
use emberwake_battle::intent::Intent;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Tip, SMALL};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::plan::expected;
use emberwake_battle::round_intent::{self as ri, RoundIntent};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use std::time::Instant;

/// RULESET1_INTENT UI5: a round intent's colour, its commands' colour (Assault an attack's, Guard a Defend's, Support's,
/// Maneuver a Disengage's, Channel a Focus's). A4.2-K1b: Field takes the theme's earth tone (`field_color`).
fn intent_color(i: RoundIntent) -> Rgba {
    kind_color(match i {
        RoundIntent::Assault => "ATTACK",
        RoundIntent::Guard => "DEFEND",
        RoundIntent::Support => "SUPPORT",
        RoundIntent::Maneuver => "DISENGAGE",
        RoundIntent::Channel => "FOCUS",
        RoundIntent::Field => return field_color(),
    })
}

/// A4.2-K1b (INTENT2-U "Field ring colour"): the Field intent's colour, from the theme palette (the checked box's earth
/// tone, an amber brown in the game's theme), so a theme retunes it with no new art.
pub(super) fn field_color() -> Rgba {
    th().check_on
}

/// VISCLAR1-D (mock `intent_ring`): the ring under a unit whose intent shows, a brighter cut of the intent's colour, so it
/// reads on the ground.
pub(super) fn intent_ring_color(i: RoundIntent) -> Rgba {
    match i {
        RoundIntent::Assault => rgb(200, 70, 70),
        RoundIntent::Guard => rgb(50, 160, 160),
        RoundIntent::Support => rgb(70, 165, 90),
        RoundIntent::Maneuver => rgb(70, 145, 205),
        RoundIntent::Channel => rgb(175, 135, 50),
        // A4.2-K1b: the Field colour brightened by half (the earth tone kept, apart from Assault's red), as the ground needs
        RoundIntent::Field => {
            let c = field_color();
            let up = |v: u8| (v as u16 * 3 / 2).min(255) as u8;
            rgb(up(c[0]), up(c[1]), up(c[2]))
        }
    }
}

/// VISCLAR1-D: the intent unit `i`'s ring shows; None keeps the side ring: no record (classic), the intent glyphs off, or
/// an enemy while the tells are hidden (a ring would tell what the player chose not to see).
pub(super) fn ring_intent(rounds: &[Intent], i: usize, ally: bool, glyphs: bool, show_tells: bool) -> Option<RoundIntent> {
    if !glyphs || (!ally && !show_tells) {
        return None;
    }
    rounds.iter().find(|r| r.unit == i).and_then(|r| r.round_intent)
}

/// UI5: an intent's colour for text on the dark cards (its colour lifted toward white, so it reads at SMALL).
fn intent_ink(i: RoundIntent) -> Rgba {
    let c = intent_color(i);
    let lift = |v: u8| (v as u16 + (255 - v as u16) * 45 / 100) as u8;
    rgb(lift(c[0]), lift(c[1]), lift(c[2]))
}

/// UI5: the intent glyph the strip, the cards and the board share: a plate in the intent's colour with its initial
/// (TERMS1 TU5: of the intent's name through `cat`'s terms).
pub(super) fn intent_glyph(p: &mut Painter, cat: &emberwake_data::Catalog, i: RoundIntent, r: Rect) {
    p.panel(r, intent_color(i), rgba(255, 255, 255, 170), 2.0);
    let letter: String = i.label(cat).chars().take(1).collect();
    p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, &letter, SMALL, th().text);
}

/// A tell line's kind as the command colour key (`kind_color`).
fn tell_color(kind: &str) -> Rgba {
    kind_color(match kind {
        "attack" | "area" => "ATTACK",
        "support" => "SUPPORT",
        "guard" => "DEFEND",
        "focus" => "FOCUS",
        "rest" => "REST",
        "move" | "flee" | "retreat" => "MOVE",
        "switch" => "TARGET",
        _ => "",
    })
}

/// RULESET1 V1, V2 on a hostile card: a row of the round's blocks with an impact badge at each impact block (a hit red,
/// anything else gold); a revealed plan (V2) also fills each action from its start to its impact in its kind's colour,
/// its recovery blocks dim. Returns the row's tip (a title, then a line per badge).
pub(super) fn draw_badge_row(p: &mut Painter, t: &Intent, row: Rect, blocks: i32) -> Tip {
    use emberwake_battle::intent::HIT;
    let cw = row.w as f32 / blocks as f32;
    let cell = |b: i32| {
        let x0 = row.x + ((b - 1) as f32 * cw).round() as i32;
        Rect::new(x0, row.y, (row.x + (b as f32 * cw).round() as i32 - x0 - 1).max(1), row.h)
    };
    for b in 1..=blocks {
        p.fill(cell(b), th().game("strip_cell", rgba(14, 20, 34, 255)));
    }
    if t.revealed {
        for l in t.plan.iter().filter(|l| l.blocks.0 >= 1) {
            for b in l.blocks.0..=l.blocks.1.min(blocks) {
                p.fill(cell(b), with_alpha(tell_color(l.kind), 210));
            }
            for b in (l.blocks.1 + 1)..=(l.blocks.1 + l.recovery).min(blocks) {
                p.fill(cell(b), rgba(120, 120, 140, 110));
            }
        }
    }
    let mut tip: Tip = vec![("Impacts this round".into(), th().gold)];
    for &(b, k) in t.impacts.iter().filter(|(b, _)| (1..=blocks).contains(b)) {
        let c = cell(b);
        let col = if k == HIT { th().foe } else { th().gold };
        p.fill(Rect::new(c.x + c.w / 2 - 2, c.y, 4, c.h), col);
        p.fill(Rect::new(c.x + c.w / 2 - 1, c.y - 1, 2, c.h + 2), col);
        tip.push((format!("B{b}: {}", if k == HIT { "a hit lands" } else { "an action lands" }), col));
    }
    if tip.len() == 1 {
        tip.push(("Nothing lands this round".into(), th().dim));
    }
    tip
}

/// INPUT1-AP (AP10): the AP gauge's levels in pips (full, solid outline) moving from `from` to `to` over `AP_ANIM_MS`
/// from `at`, for member `unit` in `round`, in the planning look or not.
#[derive(Debug, Clone, Copy)]
pub(super) struct ApAnim {
    pub(super) unit: usize,
    pub(super) round: i32,
    pub(super) planning: bool,
    pub(super) from: (f32, f32),
    pub(super) to: (f32, f32),
    pub(super) at: Instant,
}

impl ApAnim {
    /// The levels at `now`.
    pub(super) fn level(&self, now: Instant) -> (f32, f32) {
        let k = (now.saturating_duration_since(self.at).as_secs_f32() * 1000.0 / AP_ANIM_MS).clamp(0.0, 1.0);
        (self.from.0 + (self.to.0 - self.from.0) * k, self.from.1 + (self.to.1 - self.from.1) * k)
    }
}

// ---- VISCLAR1-D: the party card's AP pips (mock `party_ap`) -------------------------------------------------------

/// The pips' one colour and a spent pip's socket; no plan colours here, the strip's gauge has those.
pub(super) const AP_PIP: Rgba = rgb(110, 160, 235);
pub(super) const AP_PIP_EMPTY: Rgba = rgb(36, 44, 60);
/// How long a card's pips take to fall when a move pays AP and to rise when the round starts (ms of the FX clock: the
/// animation speed and 0.5x apply; at once with skip animations or reduce motion).
pub(super) const AP_DRAIN_MS: u64 = 240;
pub(super) const AP_REFILL_MS: u64 = 480;
const AP_PIP_H: i32 = 5;

/// What one party card's pips show over time: a pure function of the FX clock, so no window is needed to test it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ApPips {
    pub(super) from: f32,
    pub(super) to: f32,
    pub(super) at: u64,
}

impl ApPips {
    /// A card first seen: at its AP, no animation.
    pub(super) fn still(ap: f32, now: u64) -> ApPips {
        ApPips { from: ap, to: ap, at: now }
    }

    /// The AP the pips show at clock `now`.
    pub(super) fn level(&self, now: u64) -> f32 {
        let ms = if self.to >= self.from { AP_REFILL_MS } else { AP_DRAIN_MS };
        let k = now.saturating_sub(self.at) as f32 / ms as f32;
        // landed means exactly the target: `pips_moving` compares them
        if k >= 1.0 {
            self.to
        } else {
            self.from + (self.to - self.from) * k
        }
    }

    /// The unit's AP at clock `now`: a change starts from what shows now (`instant`: lands at once).
    pub(super) fn follow(self, ap: f32, now: u64, instant: bool) -> ApPips {
        if instant {
            ApPips::still(ap, now)
        } else if ap == self.to {
            self
        } else {
            ApPips { from: self.level(now), to: ap, at: now }
        }
    }
}

/// How full pip `k` is at `level` AP, with `unit` AP a pip (0 empty, 1 full; the one between is falling or rising).
pub(super) fn pip_fill(level: f32, unit: i32, k: i32) -> f32 {
    (level / unit.max(1) as f32 - k as f32).clamp(0.0, 1.0)
}

/// The row of AP pips with its left end at (`x`, `y`), one size whatever the maximum (the engine gauge's slots, lower);
/// each pip a socket and the part of it still held. Returns the row's rect.
pub(super) fn draw_ap_pips(p: &mut Painter, x: i32, y: i32, level: f32, max: i32) -> Rect {
    use aegis_ui::pips::{count, unit, PIP_GAP, PIP_W};
    let per = unit(max, AP_PIPS);
    let slots = count(max, per);
    for k in 0..slots {
        let r = Rect::new(x + k * (PIP_W + PIP_GAP), y, PIP_W, AP_PIP_H);
        p.fill(r, AP_PIP_EMPTY);
        let held = (pip_fill(level, per, k) * PIP_W as f32).round() as i32;
        if held > 0 {
            p.fill(Rect::new(r.x, r.y, held, r.h), AP_PIP);
        }
    }
    Rect::new(x, y, aegis_ui::pips::pips_w(slots), AP_PIP_H)
}

// ---- Drawing ------------------------------------------------------------------------------------------------------

pub(crate) fn kind_color(kind: &str) -> Rgba {
    match kind {
        "ATTACK" => rgb(160, 56, 56),
        "MOVE" => rgb(52, 92, 160),
        "DEFEND" => rgb(40, 122, 122),
        "SUPPORT" => rgb(52, 128, 68),
        "FOCUS" => rgb(128, 98, 34),
        "REST" => rgb(74, 80, 96),
        "STATUS" => rgb(104, 56, 128),
        "TARGET" => rgb(92, 76, 120),
        "DISENGAGE" => rgb(56, 116, 160),
        _ => rgb(60, 66, 84),
    }
}

/// VISCLAR1-B: a strip block's icon square with its border, in old units (the 16-high block keeps 1 above and below).
pub(super) const BLOCK_ICON: i32 = 14;
const BLOCK_ICON_EDGE: i32 = 1;
const BLOCK_ICON_GAP: i32 = 2;
const BLOCK_PAD: i32 = 2;
/// A ghost block's icon and outline alpha.
const BLOCK_GHOST_ALPHA: u8 = 150;
/// Mock C on every kind colour, as `kind_color` is fixed too: the icon border, the name and its outline.
const BLOCK_ICON_LIGHT: Rgba = rgb(238, 230, 210);
const BLOCK_TEXT: Rgba = rgb(246, 242, 232);
const BLOCK_TEXT_EDGE: Rgba = rgb(12, 12, 16);
/// VISCLAR1-D (mock `rest_colour`, the user's pick "light slate"): a Rest block's one colour and its dark ink.
pub(super) const REST_BLOCK: Rgba = rgb(170, 184, 204);
const REST_INK: Rgba = rgb(28, 34, 46);

/// How a planned block reads (VISCLAR1-B): chosen, the planner's ghost, or a Rest (VISCLAR1-D: bright, dark ink).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockLook {
    Chosen,
    Ghost,
    Rest,
}

/// What a planned block shows and where (VISCLAR1-B, mock C): the icon and the name centred together, the icon alone
/// centred when the name does not fit, the name alone centred when there is no icon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct BlockLabel {
    /// The icon square, its border included.
    pub(super) icon: Option<Rect>,
    /// The name's left edge.
    pub(super) name_x: Option<f32>,
}

pub(super) fn block_label(r: Rect, has_icon: bool, name_w: f32) -> BlockLabel {
    let room = (r.w - 2 * BLOCK_PAD) as f32;
    let icon_at = |x: i32| Some(Rect::new(x, r.y + (r.h - BLOCK_ICON) / 2, BLOCK_ICON, BLOCK_ICON));
    let centred = |w: f32| r.x as f32 + (r.w as f32 - w) / 2.0;
    if !has_icon {
        return BlockLabel { icon: None, name_x: Some(centred(name_w.min(room))) };
    }
    let both = (BLOCK_ICON + BLOCK_ICON_GAP) as f32 + name_w;
    if both > room {
        return BlockLabel { icon: icon_at(r.x + (r.w - BLOCK_ICON) / 2), name_x: None };
    }
    let x = centred(both).round() as i32;
    BlockLabel { icon: icon_at(x), name_x: Some((x + BLOCK_ICON + BLOCK_ICON_GAP) as f32) }
}

/// `s` with a dark outline one screen pixel wide.
pub(super) fn text_outlined(p: &mut Painter, x: f32, y: f32, s: &str, size: f32, col: Rgba, edge: Rgba) {
    let d = 1.0 / p.v.s;
    for (dx, dy) in [(-1.0, -1.0), (0.0, -1.0), (1.0, -1.0), (-1.0, 0.0), (1.0, 0.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0)] {
        p.text(x + dx * d, y + dy * d, s, size, edge);
    }
    p.text(x, y, s, size, col);
}

/// VISCLAR1-B (mock C): a planned block's icon (`chain`, the first file the layers hold) with its thin light border and
/// its name in near-white with a dark outline. A ghost shows them faded and its name dim; a dim Rest has no border and no
/// outline. Returns what it drew for the agent link.
pub(super) fn draw_block_label(ui: &mut aegis_ui::ui::Ui<super::Action>, art: &mut aegis_ui::art::Art, r: Rect, name: &str, chain: &[String], look: BlockLook) -> serde_json::Value {
    let icon_rel = art.first_rel(chain);
    let px = (((BLOCK_ICON - 2 * BLOCK_ICON_EDGE) as f32) * ui.p.v.s).round().max(1.0) as i32;
    let img = icon_rel.as_deref().and_then(|rel| art.image(rel, px, px));
    let name_w = ui.p.text_w(name, SMALL);
    let label = block_label(r, img.is_some(), name_w);
    if let (Some(sq), Some(img)) = (label.icon, img) {
        let inner = Rect::new(sq.x + BLOCK_ICON_EDGE, sq.y + BLOCK_ICON_EDGE, sq.w - 2 * BLOCK_ICON_EDGE, sq.h - 2 * BLOCK_ICON_EDGE);
        ui.p.image_alpha(img, inner, if look == BlockLook::Ghost { BLOCK_GHOST_ALPHA } else { 255 });
        let edge = match look {
            BlockLook::Chosen => BLOCK_ICON_LIGHT,
            BlockLook::Ghost => with_alpha(BLOCK_ICON_LIGHT, BLOCK_GHOST_ALPHA),
            BlockLook::Rest => REST_INK,
        };
        for e in [Rect::new(sq.x, sq.y, sq.w, 1), Rect::new(sq.x, sq.y + sq.h - 1, sq.w, 1), Rect::new(sq.x, sq.y + 1, 1, sq.h - 2), Rect::new(sq.x + sq.w - 1, sq.y + 1, 1, sq.h - 2)] {
            ui.p.fill(e, edge);
        }
    }
    let mut cut = false;
    if let Some(x) = label.name_x {
        let y = r.y as f32 + (r.h as f32 - ui.p.line_h(SMALL)) / 2.0;
        let shown = ui.p.fit(name, SMALL, (r.w - 2 * BLOCK_PAD) as f32);
        cut = shown != name;
        // Transient for the text audit: a span's name crosses the hover widgets of its later blocks by design, and
        // `block_label` and `fit` keep it inside the span (a cut name reports in `strip_labels`; the tip has it whole)
        ui.p.transient(|p| match look {
            BlockLook::Chosen => text_outlined(p, x, y, &shown, SMALL, BLOCK_TEXT, BLOCK_TEXT_EDGE),
            BlockLook::Ghost => text_outlined(p, x, y, &shown, SMALL, th().dim, with_alpha(BLOCK_TEXT_EDGE, BLOCK_GHOST_ALPHA)),
            BlockLook::Rest => {
                p.text(x, y, &shown, SMALL, REST_INK);
            }
        });
    }
    serde_json::json!({"name": name, "icon": if label.icon.is_some() { icon_rel } else { None }, "shows_name": label.name_x.is_some(), "cut": cut, "look": format!("{look:?}").to_lowercase()})
}

/// The icon chain of each of unit `u`'s committed blocks, in `committed`'s order: the skill the command runs (by id; names
/// are not unique), else its kind's icon, if any (an item has none yet: its name shows alone).
pub(super) fn committed_icon_chains(s: &Session, u: usize) -> Vec<Vec<String>> {
    committed(s, u)
        .iter()
        .zip(s.resolver.scheduled.iter().filter(|a| a.unit == u))
        .map(|(row, a)| match s.w.cat.skill(&a.command.skill) {
            Some(sk) => emberwake_content::art::skill_icon_chain(sk.family_id(), &row.3),
            None => vec![kind_icon(&row.3)],
        })
        .collect()
}

/// A kind's own icon.
pub(super) fn kind_icon(kind: &str) -> String {
    format!("skills/kinds/{}.png", kind.to_lowercase())
}

pub(super) fn short_name(name: &str) -> String {
    name.strip_prefix("Goblin ").unwrap_or(name).to_string()
}

pub(super) fn stance_name(u: &emberwake_rules::Unit) -> &'static str {
    ["Neutral", "Offensive", "Defensive", "Tactical"][u.stance as usize]
}

/// The actions the resolver holds for `u` this round (after confirm): (start, end, name, kind, target name).
pub(crate) fn committed(s: &Session, u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    action_rows(&s.w, &s.resolver.scheduled, u)
}

/// Unit `u`'s actions among `actions` (this round's, or a past round's from the archive): (start, end, name, kind,
/// target name).
pub(crate) fn action_rows(w: &World, actions: &[emberwake_battle::resolver::ScheduledAction], u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    actions
        .iter()
        .filter(|a| a.unit == u)
        .map(|a| {
            let (kind, target) = match &a.command.kind {
                CmdKind::Attack { target, .. } => ("ATTACK", *target),
                CmdKind::Area { .. } => ("ATTACK", None),
                CmdKind::Defend { .. } => ("DEFEND", None),
                CmdKind::Focus { .. } => ("FOCUS", None),
                CmdKind::Rest { .. } => ("REST", None),
                CmdKind::StatusBlock { .. } => ("STATUS", None),
                CmdKind::Support { target, .. } => ("SUPPORT", Some(*target)),
                CmdKind::Item { target, .. } => ("ITEM", Some(*target)),
                CmdKind::Target { target } => ("TARGET", *target),
                CmdKind::Walk { .. } => ("MOVE", None),
                CmdKind::Disengage { .. } => ("DISENGAGE", None),
            };
            let name = if a.cancelled { format!("{} (cancelled)", a.command.name) } else { a.command.name.clone() };
            (a.start, a.end, name, kind.to_string(), target.filter(|t| *t != u).map(|t| w.units[t].name.clone()))
        })
        .collect()
}

/// A unit's next (or current) action this round, for cards and hover cards.
pub(super) fn next_action(s: &Session, e: usize) -> String {
    if s.phase == Phase::Replay {
        return "-".into();
    }
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    match s.resolver.scheduled.iter().find(|a| a.unit == e && !a.cancelled && (planning || a.end >= s.current_block)) {
        Some(a) => match a.command.kind {
            CmdKind::Attack { target: Some(t), .. } => format!("{} on {} (B{}-{})", a.command.name, short_name(&s.w.units[t].name), a.start, a.end),
            _ => format!("{} (B{}-{})", a.command.name, a.start, a.end),
        },
        None => "-".into(),
    }
}

/// DERIVE1 4.4: unit `i`'s catalog derived stats shown in battle under the battle's ruleset (none in classic).
pub(super) fn derived_rows(s: &Session, i: usize) -> Vec<crate::party::DRow> {
    crate::party::data_rows(&s.w.cat, &s.w.ruleset.id, &s.w.units[i], "battle")
}

/// Derived rows in one line (`Composure 26   Grit 3`), None when there are none.
pub(super) fn derived_line(rows: &[crate::party::DRow]) -> Option<String> {
    (!rows.is_empty()).then(|| rows.iter().map(|r| format!("{} {}", r.label, r.shown)).collect::<Vec<_>>().join("   "))
}

/// HOOKS1 CN2 (HOOKS1-UI slice 2): each unit's class name in world `world` (its catalog row's class: the world's
/// `classes` overlay name, else the class row's; "" for a unit without a class).
pub(super) fn class_names(d: &emberwake_world::Data, world: &str, w: &World) -> Vec<String> {
    w.units.iter().map(|u| d.cat.units.get(&u.catalog_id).filter(|def| !def.class_id.is_empty() && d.cat.classes.contains_key(&def.class_id)).map(|def| d.class_name(world, &def.class_id)).unwrap_or_default()).collect()
}

/// Hover card of unit `i` (`tells`: the enemy intents while planning, INTENT1).
/// RULESET1-U slice 4 (UI5): `rounds` is every unit's round intent record (`Session::round_intents`; empty in classic):
/// the card leads with the unit's intent and target (either side) and, during the playback, an enemy's record stands in
/// for its tell (so a reveal and its readers stay listed, a reader who fell included).
/// HOOKS1-UI slice 2: `classes[i]` is unit `i`'s class name in the battle's world ("" = none shown; `class_names`).
pub(super) fn unit_tip(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent], rounds: &[Intent], classes: &[String]) -> Tip {
    unit_tip_as(s, i, sel, tells, rounds, classes, None)
}

/// INPUT1 IS1-IS3: the intel panel's lines for the inspected unit `i`, headed "Inspecting <name> (Esc clears)"
/// (`show_tells`: the Show enemy tells setting, I). An enemy shows only what the party knows (IS2's "Never" list: no
/// unrevealed plan's skills, blocks or walks, deck, order, AI row or profile): the hover card's lines with its facing,
/// without the tell's first action and its forecast while the tells are hidden and without the classic "Next:" line;
/// a party member adds its picks, its plan and its AP numbers (IS3, AP9's lines) while planning under the round intent.
pub(super) fn inspect_tip(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent], rounds: &[Intent], classes: &[String], show_tells: bool) -> Tip {
    let mut t = unit_tip_as(s, i, sel, tells, rounds, classes, Some(show_tells));
    if let Some(head) = t.first_mut() {
        head.0 = format!("Inspecting {} (Esc clears)", s.w.units[i].name);
    }
    t
}

/// AP9 (INPUT1 7): a party member's AP numbers, one line each: "AP <now> of <max>", "Plan: <spent> (picked <picked>,
/// auto-filled <filled>)", "Left: <left>", "Next round: +<gain> -> <next> (Rest +<rest_gain>)", and with more than one
/// AP a pip (AP6) "1 pip = <u> AP".
pub(crate) fn ap_lines(f: &rip::ApForecast) -> Vec<String> {
    let mut lines = vec![
        format!("AP {} of {}", f.now, f.max),
        format!("Plan: {} (picked {}, auto-filled {})", f.spent, f.picked, f.filled),
        format!("Left: {}", f.left),
        format!("Next round: +{} -> {} (Rest +{})", f.gain, f.next, f.rest_gain),
    ];
    lines.extend(pip_line(f.max));
    lines
}

/// AP6, AP9: "1 pip = <u> AP" when a pip of the AP gauge stands for more than one AP (`max_ap` above `AP_PIPS`).
fn pip_line(max: i32) -> Option<String> {
    let u = aegis_ui::pips::unit(max, AP_PIPS);
    (u > 1).then(|| format!("1 pip = {u} AP"))
}

/// INPUT2: the colour of the AP a party member has paid this round on the playback's gauge (one block at its left).
pub(super) const AP_PAID: Rgba = rgb(96, 106, 128);

/// INPUT2 (style D): the gauge's planned blocks from a member's plan entries `(AP, skill kind, ghost)`: one block an
/// entry that costs AP, in its kind's colour (`kind_color`, the strip's), a ghost entry as a ghost block.
pub(super) fn ap_blocks(entries: &[(i32, String, bool)]) -> Vec<aegis_ui::pips::PipBlock> {
    entries.iter().filter(|e| e.0 > 0).map(|(ap, kind, ghost)| aegis_ui::pips::PipBlock { ap: *ap, colour: kind_color(kind), ghost: *ghost }).collect()
}

/// INPUT1-AP (AP6-AP10), INPUT2 (style D): the AP gauge of party member `u`. Planning (`f` its forecast, of the hovered
/// row's preview when one shows; `blocks` its plan's AP blocks, `ap_blocks`): one capsule a planned block in its kind's
/// colour, free sockets up to its AP, crossed slots up to its max; red with "-<n>" after the slots for a hovered row it
/// cannot pay (`short`); playback: what it paid since `start` as one block, its live AP as sockets. A debt shows "-<n>"
/// before crossed slots. Its hover card: AP9's lines (playback: the live AP), "1 pip = <u> AP" above 12 AP.
pub(super) fn ap_gauge_of(w: &World, u: usize, f: Option<&rip::ApForecast>, blocks: &[aegis_ui::pips::PipBlock], short: Option<i32>, start: i32) -> aegis_ui::pips::PipGauge {
    use aegis_ui::pips::PipGauge;
    let unit = &w.units[u];
    match f {
        Some(f) => {
            let g = PipGauge::planned(f.now, blocks, f.max, AP_PIPS).with_tip(ap_lines(f).into_iter().map(|l| (l, ALLY)).collect());
            match short {
                Some(n) => g.short(n),
                None => g,
            }
        }
        None => {
            let mut tip: Tip = vec![(format!("AP {} of {}", unit.ap, unit.max_ap), ALLY)];
            if start > unit.ap {
                tip.push((format!("Paid this round: {}", start - unit.ap), th().text));
            }
            tip.extend(pip_line(unit.max_ap).map(|l| (l, th().text)));
            PipGauge::live(start, unit.ap, AP_PAID, unit.max_ap, AP_PIPS).with_tip(tip)
        }
    }
}

/// IS3: an inspected party member's picks and plan: "Picks: <pick>, ..." (or "Picks: none, the planner's round"), then
/// one line a planned entry: "B1-2 Heavy Cleave on Wolf" (", auto-filled" for a ghost entry, GH1); once the round is
/// confirmed its committed actions.
fn plan_lines(s: &Session, u: usize) -> Tip {
    let w = &s.w;
    let target = |t: Option<usize>| t.filter(|t| *t != u).map(|t| format!(" on {}", w.units[t].name)).unwrap_or_default();
    if s.plan_confirmed || s.phase != Phase::Planning {
        return committed(s, u).into_iter().map(|c| (format!("{} {}{}", intent_text::block_text((c.0, c.1)), c.2, c.4.map(|n| format!(" on {n}")).unwrap_or_default()), th().text)).collect();
    }
    let picks = rip::picks_of(w, u);
    let mut t: Tip = vec![(if picks.is_empty() { "Picks: none, the planner's round".to_string() } else { format!("Picks: {}", picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", ")) }, th().gold)];
    let ghosts = rip::ghost_entries(w, &s.plan, u);
    for (k, e) in s.plan.entries_for(w, u).iter().enumerate() {
        let ghost = ghosts.get(k).copied().unwrap_or(false);
        let line = format!("{} {}{}{}", intent_text::block_text((e.start_block, e.end_block(w))), w.cat.skills[e.skill].name, target(e.target), if ghost { ", auto-filled" } else { "" });
        t.push((line, if ghost { th().dim } else { th().text }));
    }
    t
}

/// IS2: a facing in words (the board's top is north); None for no facing.
fn facing_name(f: V2) -> Option<&'static str> {
    match (f.x.signum(), f.y.signum()) {
        (0, -1) => Some("north"),
        (0, 1) => Some("south"),
        (1, 0) => Some("east"),
        (-1, 0) => Some("west"),
        (1, -1) => Some("north-east"),
        (-1, -1) => Some("north-west"),
        (1, 1) => Some("south-east"),
        (-1, 1) => Some("south-west"),
        _ => None,
    }
}

/// The hover card (`inspect` None) or the inspect lines (`inspect` Some(the Show enemy tells setting), IS2, IS3).
fn unit_tip_as(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent], rounds: &[Intent], classes: &[String], inspect: Option<bool>) -> Tip {
    let w = &s.w;
    let u = &w.units[i];
    let ally = w.allies.contains(&i);
    let tells_shown = inspect.unwrap_or(true);
    let mut t: Tip = vec![(u.name.clone(), if ally { ALLY } else { th().foe })];
    let round = rounds.iter().find(|r| r.unit == i);
    let tell = tells.iter().find(|r| r.unit == i).or(round.filter(|_| !ally));
    if let Some((head, intent)) = round.filter(|_| u.is_active()).and_then(|r| Some((intent_text::round_head(w, r)?, r.round_intent?))) {
        t.push((head, intent_ink(intent)));
    }
    t.push((format!("HP {}/{}   AP {}/{}   {}", u.hp, u.max_hp, u.ap, u.max_ap, stance_name(u)), th().text));
    let level = match classes.get(i).filter(|c| !c.is_empty()) {
        Some(c) if u.tier.is_empty() => format!("Level {} {c}", u.stats.level),
        Some(c) => format!("Level {} {c} ({})", u.stats.level, u.tier),
        None => format!("Level {} {}", u.stats.level, u.tier),
    };
    t.push((format!("{level}   Initiative {}   Reach {}", u.initiative, reach_text(w, i)), th().dim));
    if let Some(line) = derived_line(&derived_rows(s, i)) {
        t.push((line, th().dim));
    }
    // IS2: the facing (the round intent's, rule round_intent)
    if let Some(f) = inspect.filter(|_| u.is_active() && ri::on(w)).and_then(|_| facing_name(ri::facing(w, i))) {
        t.push((format!("Facing {f}"), th().dim));
    }
    // RULESET1-U slice 5: the poise bar in words (rule poise)
    if let Some(st) = emberwake_battle::poise::state(w, i).filter(|st| st.max > 0 && u.is_active()) {
        // S2.2-S3: in pips under poise_mode pips, with the threshold a hit must reach to take one
        let n = &w.ruleset.numbers;
        let what = if emberwake_battle::poise::pips_mode(n) { format!("Poise pips {}/{} (T {})", if st.broken { 0 } else { st.pips }, n.poise_pips.max(1), emberwake_battle::poise::threshold(n, st.max)) } else { format!("Poise {}/{}", st.value, st.max) };
        t.push((format!("{what}{}", if st.broken { "  BROKEN this round" } else if st.broken_last { "  (broke last round: stagger x repeat)" } else { "" }), if st.broken { th().foe } else { POISE }));
    }
    // HOOKS1-UI slice 3: its meters (UM7; none without the rule, a hidden meter left out) and a held guard (8.3)
    if u.is_active() {
        for m in emberwake_battle::meter::shown(w, i) {
            t.push((meter_text(&m), METER));
        }
    }
    if let Some(g) = held_guard(s, i) {
        t.push((format!("Held guard {:.0}% through B{}", g.mitigation * 100.0, g.last_block), HELD));
    }
    // RULESET1-U slice 3 (GD5): a stealthed unit that moved this round is lost to attacks locked on it
    if ri::hidden(w, i) {
        // good news on a hero, a warning on an enemy
        t.push(("Hidden: an attack locked on it finds nothing this round".into(), if ally { th().good } else { th().gold }));
    }
    if !u.st.active.is_empty() {
        t.push((format!("Now: {}", u.st.active.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), th().foe));
    }
    if !u.st.pending.is_empty() {
        t.push((format!("Next round: {}", u.st.pending.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), th().gold));
    }
    // IS3: an inspected party member's AP numbers, picks and plan (round intent)
    if ally && u.is_active() && inspect.is_some() && ri::on(w) {
        if s.phase == Phase::Planning && !s.plan_confirmed {
            t.extend(ap_lines(&rip::ap_forecast(w, &s.plan, i)).into_iter().map(|l| (l, ALLY)));
        }
        t.extend(plan_lines(s, i));
    }
    if !ally && u.is_active() {
        match tell {
            Some(r) => {
                // IS2: the tell's first action only while the tells show (I)
                if tells_shown {
                    t.push((format!("Intent: {}", intent_text::first_line(w, &r.line)), th().gold));
                }
                // V1, V2: who reads the plan (a reader who fell stays listed), the enrage or the battle's plan reveal
                if let Some(head) = intent_text::reveal_head(w, r, |p| s.w.cat.passives.get(p).map(|x| x.name.clone()).unwrap_or_else(|| p.to_string())) {
                    t.push((head, ALLY));
                    for l in &r.plan {
                        t.push((intent_text::plan_line(w, l), th().text));
                    }
                }
                if let Some(f) = sel.and_then(|h| intent_text::forecast(r, h)).filter(|_| tells_shown) {
                    t.push((f, th().foe));
                }
            }
            // IS2: an inspect never names an unrevealed next action
            None if inspect.is_some() => {}
            None => t.push((format!("Next: {}", next_action(s, i)), th().gold)),
        }
        if let Some(h) = sel.filter(|h| w.units[*h].is_active()) {
            let strike = w.book().lowest(Kind::Attack);
            t.push((format!("{} deals about {} with a Strike, takes about {}", w.units[h].name, expected(w, h, i, strike, None, 0.0, 1.0), expected(w, i, h, strike, None, 0.0, 1.0)), ALLY));
        }
    }
    if !u.is_alive() {
        t.push(("Down".into(), th().dim));
    } else if u.is_withdrawn {
        t.push(("Left the battle".into(), th().dim));
    }
    t
}
