//! The campaign's screens (CAMPAIGN1-U, spec CAMPAIGN1 F3-F5, W1, W3; engine-only, no Godot original): the campaign
//! events of every action's result (`take`: a log line each, a notice for each warning, the act-start save), the
//! notices the map screens show one by one when nothing else is open (`map_tick`), the run-over screen that replaces
//! every other screen once the run is over (the card, then Load the act's start, Load a save, Main menu), and the asks
//! before a step that would end the run (`fight_ask`, `days_ask`; No is the default); slice 2: the campaign queue's
//! battle and dialogue heads (`play_pending_fight`, `pending_talk`; T2). The rules are the kit's
//! (`emberwake_world::campaign`); this module only shows them. Story-free: every text it shows beyond its own button
//! labels comes from the data.

use crate::hud::tone_colour;
use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use aegis_core::json::{self, s, Dict};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, HEAD, TITLE, W};
use aegis_ui::widget_plumbing;
use emberwake_world::campaign::trigger;
use emberwake_world::dialogue::DialogueRunner;
use emberwake_world::{campaign, save};
use serde_json::{json, Value};

/// The run-over screen's backdrop (dark, a red cast).
const OVER_FILL: Rgba = rgb(0x12, 0x0b, 0x0e);
/// A notice's or the run-over card's least inner width and its margins (logical px at text size Normal; both grow with
/// the text).
const NOTICE_W: i32 = 250;
const CARD_W: i32 = 300;
const PAD: i32 = 10;
/// The answer tags of the asks the world map takes back (`Cx::ask_tag_no`).
pub const ASK_TRIP: &str = "campaign.trip";
pub const ASK_REST: &str = "campaign.rest";
/// The answer tag of the ask before a dialogue choice whose `rest` would end the run (the region map takes it back).
pub const ASK_TALK_REST: &str = "campaign.talk_rest";

/// Whether the game in play's run is over (F1).
pub fn is_over(cx: &Cx) -> bool {
    cx.game.as_ref().is_some_and(|g| !campaign::save_refusal(&g.gs).is_empty())
}

/// The meter `id`'s HUD name now (its label, else its name; the id when unknown).
fn meter_name(cx: &Cx, id: &str) -> String {
    let Some(c) = cx.game.as_ref().and_then(|g| g.gs.campaign.as_ref()) else { return id.to_string() };
    match (c.defs.get(id), c.meters.get(id)) {
        (_, Some(st)) if !st.label.is_empty() => st.label.clone(),
        (Some(def), _) => def.name.clone(),
        _ => id.to_string(),
    }
}

/// A checkpoint event's name: its own, else its quest's, else its id.
fn checkpoint_name(cx: &Cx, e: &Dict) -> String {
    let name = s(e, "name");
    if !name.is_empty() {
        return name;
    }
    let quest = s(e, "quest");
    if !quest.is_empty() && cx.has_game() {
        let q = emberwake_world::quest::resolved(&cx.data, cx.gs(), &quest);
        return json::s_or(&q, "name", &quest);
    }
    s(e, "checkpoint")
}

/// "1 day" or "n days".
pub fn days(n: i64) -> String {
    if n == 1 {
        "1 day".into()
    } else {
        format!("{n} days")
    }
}

/// W1: a notice for campaign event `e`: (title, tone, text); None for an event that only logs.
pub fn notice_of(cx: &Cx, e: &Dict) -> Option<(String, String, String)> {
    match s(e, "event").as_str() {
        "meter_warning" => {
            let mut title = meter_name(cx, &s(e, "meter"));
            let band = s(e, "band");
            let def = cx.game.as_ref().and_then(|g| g.gs.campaign.as_ref()).and_then(|c| c.defs.get(&s(e, "meter")).cloned());
            if let Some(b) = def.as_ref().and_then(|d| d.bands.iter().find(|b| b.id == band)) {
                title = format!("{title}: {}", b.name);
            }
            Some((title, s(e, "tone"), s(e, "text")))
        }
        "checkpoint_warning" => {
            let left = json::i(e, "left", 0);
            let tone = if left <= 1 { "danger" } else { "warn" };
            Some((checkpoint_name(cx, e), tone.into(), format!("{} left: the deadline is day {}.", days(left), json::i(e, "day", 0))))
        }
        "checkpoint_missed" => Some((checkpoint_name(cx, e), "danger".into(), "The deadline has passed.".into())),
        _ => None,
    }
}

/// W1: the log line of campaign event `e` ("" for none).
fn log_line(cx: &Cx, e: &Dict) -> String {
    match s(e, "event").as_str() {
        "meter_warning" => format!("{}: {}", meter_name(cx, &s(e, "meter")), s(e, "text")),
        "checkpoint_warning" => format!("{}: {} left (day {}).", checkpoint_name(cx, e), days(json::i(e, "left", 0)), json::i(e, "day", 0)),
        "checkpoint_missed" => format!("{}: the deadline has passed.", checkpoint_name(cx, e)),
        "act_start" => format!("Act {} begins.", json::i(e, "act", 0)),
        "run_over" => campaign::RUN_OVER_TEXT.to_string(),
        _ => String::new(),
    }
}

/// W1, F5: the campaign events of an action's result (its `campaign` list): a log line each (a trigger's notes too), a
/// queued notice for each warning, and the act-start save for an `act_start` that names one (not in a debug or sample
/// game, never while the run is over).
pub fn take(cx: &mut Cx, res: &Dict) {
    events(cx, json::aget(res, "campaign"));
}

/// `take` over a list of campaign events.
pub fn events(cx: &mut Cx, events: &[Value]) {
    for ev in events {
        let e = json::dict(ev);
        let line = log_line(cx, e);
        if !line.is_empty() {
            cx.log(line);
        }
        if s(e, "event") == "trigger_fired" {
            for n in json::sget(e, "notes") {
                cx.log(n);
            }
        }
        if s(e, "event") == "act_start" {
            act_save(cx, &s(e, "save"));
        }
        if notice_of(cx, e).is_some() {
            if let Some(g) = cx.game.as_mut() {
                g.notices.push(ev.clone());
            }
        }
    }
}

/// F5: writes the act-start save `key` (nothing for "", in a debug or sample game, or while the run is over); a
/// problem goes to the log.
fn act_save(cx: &mut Cx, key: &str) {
    if key.is_empty() || cx.debug || cx.game.as_ref().is_none_or(|g| g.sample) || is_over(cx) {
        return;
    }
    let errors = save::write_keyed(&cx.data, &cx.save_dir, cx.gs(), key, 0);
    if !errors.is_empty() {
        cx.log(format!("The act-start save failed: {}", errors.join("; ")));
    }
}

/// A map screen's tick with nothing of its own open (as `story::map_tick`): once the run is over the run-over screen
/// replaces every screen (F3; warnings still waiting are dropped, their lines are in the log); else the oldest waiting
/// notice shows (W1). True when it showed something.
pub fn map_tick(cx: &mut Cx) -> bool {
    if !cx.has_game() || cx.busy() || !cx.nav.is_empty() {
        return false;
    }
    if is_over(cx) {
        if let Some(g) = cx.game.as_mut() {
            g.notices.clear();
        }
        let screen = RunOverScreen::new(cx);
        cx.nav.push(Nav::Reset(Box::new(screen)));
        return true;
    }
    let next = cx.game.as_mut().and_then(|g| (!g.notices.is_empty()).then(|| g.notices.remove(0)));
    if let Some(ev) = next {
        if let Some((title, tone, text)) = notice_of(cx, json::dict(&ev)) {
            cx.nav.push(Nav::Push(Box::new(CampaignNotice::new(&title, &tone, &text))));
            return true;
        }
    }
    false
}

/// T2: the campaign queue's head when it is a `{battle}` item and nothing is open: the item leaves the queue and the
/// battle starts as source `trigger` ("Fight: <name>!"; a queued battle cannot be refused, so one that could end the
/// run gets the notice, W3). True when it started (or its start failed and said why).
pub fn play_pending_fight(cx: &mut Cx) -> bool {
    if !cx.has_game() || cx.busy() || !cx.nav.is_empty() {
        return false;
    }
    let d = cx.data.clone();
    let Some(fight) = trigger::fight(&d, cx.gs()) else { return false };
    campaign::pop_pending(cx.gs_mut());
    crate::region::fight_now(cx, &fight, trigger::FIGHT_SOURCE);
    warn_fight(cx, &s(&fight, "encounter"));
    true
}

/// T2: the campaign queue's head when it is a `{dialogue, speaker}` item: the item leaves the queue and its scene opens
/// (`DialogueRunner::open_scene`); None for another head, or when the scene cannot open (its reason goes to the log).
/// The region map shows the talk; the world map leaves the item for it.
pub fn pending_talk(cx: &mut Cx) -> Option<DialogueRunner> {
    let item = campaign::pending_head(cx.gs()).filter(|item| item.contains_key("dialogue"))?.clone();
    campaign::pop_pending(cx.gs_mut());
    let d = cx.data.clone();
    let r = DialogueRunner::open_scene(&d, cx.gs_mut(), &s(&item, "dialogue"), &s(&item, "speaker"));
    if r.closed {
        for e in &r.errors {
            cx.log(e.clone());
        }
        return None;
    }
    if !r.character_name.is_empty() {
        cx.log(format!("You speak with {}.", r.character_name));
    }
    Some(r)
}

/// "won" -> "Winning", "lost" -> "Losing", "fled" -> "Fleeing".
fn case_word(case: &str) -> &'static str {
    match case {
        "won" => "Winning",
        "lost" => "Losing",
        _ => "Fleeing",
    }
}

/// ": text" when an end names one, else ".".
fn reason(ends: &[Value]) -> String {
    let text = ends.iter().map(|e| s(json::dict(e), "text")).find(|t| !t.is_empty()).unwrap_or_default();
    if text.is_empty() {
        ".".into()
    } else {
        format!(": {text}")
    }
}

/// A reason closed as a sentence: "." stays, ": text" gets one full stop ("text!" keeps its own mark).
fn sentence_end(r: &str) -> String {
    if r.ends_with(['.', '!', '?']) {
        r.to_string()
    } else {
        format!("{r}.")
    }
}

/// W3: the question before fighting `encounter` when an outcome would end the run (`campaign::preview_battle`), None
/// when none would: "Winning or losing this fight would end the run: <text>. Fight anyway?".
pub fn fight_ask(cx: &Cx, encounter: &str) -> Option<String> {
    let ends = campaign::preview_battle(&cx.data, cx.gs(), encounter);
    if ends.is_empty() {
        return None;
    }
    let cases: Vec<String> = ends.iter().map(|e| s(json::dict(e), "case")).collect();
    let mut words: Vec<String> = cases.iter().enumerate().map(|(i, c)| if i == 0 { case_word(c).to_string() } else { case_word(c).to_lowercase() }).collect();
    words.dedup();
    let r = reason(&ends);
    Some(format!("{} this fight would end the run{} Fight anyway?", words.join(" or "), sentence_end(&r)))
}

/// W3: the line for a fight that cannot be refused (a road or a roaming fight) when an outcome would end the run.
pub fn fight_warning(cx: &Cx, encounter: &str) -> Option<String> {
    fight_ask(cx, encounter).map(|q| q.trim_end_matches(" Fight anyway?").to_string())
}

/// The title of the notice over a fight that cannot be refused but could end the run.
pub const FIGHT_WARNING: &str = "This fight can end the run";

/// W3: a fight that cannot be refused (a road or a roaming fight) and could end the run: the notice over the battle
/// it just started (nothing when no outcome would end the run).
pub fn warn_fight(cx: &mut Cx, encounter: &str) {
    if let Some(text) = fight_warning(cx, encounter) {
        cx.nav.push(Nav::Push(Box::new(CampaignNotice::new(FIGHT_WARNING, "fatal", &text))));
    }
}

/// W3: the question before `n` days pass (a trip or a rest; `verb` "Travel" or "Rest") when they would end the run
/// (`campaign::preview_days`), None when they would not.
pub fn days_ask(cx: &Cx, n: i64, what: &str, verb: &str) -> Option<String> {
    let ends = campaign::preview_days(&cx.data, cx.gs(), n);
    if ends.is_empty() {
        return None;
    }
    Some(format!("{what} would end the run{} {verb} anyway?", sentence_end(&reason(&ends))))
}

/// W3, CAMPAIGN1-TM: the question before the trip to `to` when its legs would end the run (`campaign::preview_trip`:
/// the days and the leg inputs of the travel mode now), None when they would not.
pub fn trip_ask(cx: &Cx, to: &str) -> Option<String> {
    let ends = campaign::preview_trip(&cx.data, cx.gs(), to);
    if ends.is_empty() {
        return None;
    }
    Some(format!("This trip would end the run{} Travel anyway?", sentence_end(&reason(&ends))))
}

/// The wrapped lines of `text` in `w` logical px at `size` (none for "").
fn lines(ui: &mut Ui<impl Clone>, text: &str, size: f32, w: f32) -> Vec<String> {
    if text.is_empty() {
        Vec::new()
    } else {
        ui.p.wrap(text, size, w)
    }
}

/// A button's width for `label` at the text size now (at least 50 px).
fn button_w(ui: &mut Ui<impl Clone>, label: &str) -> i32 {
    ((ui.p.text_w(label, BODY) + 16.0).ceil() as i32).max(50)
}

// ---- the notice (W1) ----

/// One campaign notice: a title in the tone's colour, the data's text, OK. Enter, Space, Esc and right click close it.
pub struct CampaignNotice {
    title: String,
    tone: String,
    text: String,
    w: Vec<Widget<()>>,
}

impl CampaignNotice {
    pub fn new(title: &str, tone: &str, text: &str) -> Self {
        Self { title: title.to_string(), tone: tone.to_string(), text: text.to_string(), w: Vec::new() }
    }

    fn act(&mut self, cx: &mut Cx, _a: ()) {
        cx.nav.push(Nav::Pop);
    }
}

impl Screen<GameCx> for CampaignNotice {
    fn id(&self) -> &'static str {
        "campaign_notice"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<()> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().scrim);
        let scale = ui.p.fonts.scale.max(1.0);
        let inner = ((NOTICE_W as f32 * scale).ceil() as i32).max(ui.p.text_w(&self.title, TITLE).ceil() as i32).min(W - 40 - 2 * PAD);
        let (tl, bl) = (ui.p.line_h(TITLE), ui.p.line_h(BODY));
        let title = lines(&mut ui, &self.title, TITLE, inner as f32);
        let body = lines(&mut ui, &self.text, BODY, inner as f32);
        let bh = (bl + 6.0).ceil() as i32;
        let h = PAD + (title.len() as f32 * tl + 4.0 + body.len() as f32 * bl).ceil() as i32 + 8 + bh + PAD;
        let r = Rect::new((W - inner - 2 * PAD) / 2, (H - h) / 2, inner + 2 * PAD, h);
        ui.p.panel(r, th().panel, tone_colour(&self.tone), 4.0);
        let mut y = (r.y + PAD) as f32;
        for l in &title {
            ui.p.text((r.x + PAD) as f32, y, l, TITLE, tone_colour(&self.tone));
            y += tl;
        }
        y += 4.0;
        for l in &body {
            ui.p.text((r.x + PAD) as f32, y, l, BODY, th().text);
            y += bl;
        }
        let ok = button_w(&mut ui, "OK");
        ui.button("campaign_notice.ok", Rect::new(r.x + r.w - PAD - ok, r.y + r.h - PAD - bh, ok, bh), "OK", (), true);
        self.w = ui.widgets;
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if k == Key::Enter || k == Key::Space {
            self.act(cx, ());
        }
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({"title": self.title, "tone": self.tone, "text": self.text})
    }

    widget_plumbing!();
}

// ---- the run-over screen (F3, F4, F5) ----

#[derive(Debug, Clone)]
pub enum OverAct {
    /// Load the act-start save of the act the run ended in.
    LoadAct,
    /// The load menu (slots and the autosave written before the fatal step).
    LoadSave,
    MainMenu,
}

/// F4: the only screen while the run is over: the end's card (its vignette, played once as the screen opens), then a
/// plain card (the title, the end's text) with Load the act's start, Load a save and Main menu. Esc and right click do
/// nothing; no game menu, no saving (F3).
pub struct RunOverScreen {
    kind: String,
    card: String,
    text: String,
    act: i64,
    /// The act-start save's key ("" when the world keeps none).
    key: String,
    /// The card's vignette played (or there is none).
    played: bool,
    w: Vec<Widget<OverAct>>,
}

impl RunOverScreen {
    pub fn new(cx: &Cx) -> Self {
        let c = cx.game.as_ref().and_then(|g| g.gs.campaign.as_ref());
        let over = c.and_then(|c| c.over.clone());
        let act = c.map_or(0, |c| c.act);
        let world = cx.game.as_ref().map(|g| g.gs.world_id.clone()).unwrap_or_default();
        let keeps = !world.is_empty() && campaign::manifest(&cx.data, &world).act_saves;
        let key = match c {
            Some(c) if keeps && act > 0 => campaign::act_save_key(act, &c.run),
            _ => String::new(),
        };
        Self {
            kind: over.as_ref().map(|o| o.kind.clone()).unwrap_or_default(),
            card: over.as_ref().map(|o| o.card.clone()).unwrap_or_default(),
            text: over.map(|o| o.text).unwrap_or_default(),
            act,
            key,
            played: false,
            w: Vec::new(),
        }
    }

    /// Why Load the act's start is off ("" when the save is there).
    fn act_save_missing(&self, cx: &Cx) -> String {
        if !self.key.is_empty() && save::exists_keyed(&cx.save_dir, &self.key) {
            String::new()
        } else if self.act < 1 {
            "No act has started yet.".into()
        } else {
            format!("There is no save from the start of act {}.", self.act)
        }
    }

    fn act(&mut self, cx: &mut Cx, a: OverAct) {
        match a {
            OverAct::LoadAct => match save::read_keyed(&cx.data, &cx.save_dir, &self.key) {
                Ok(l) => crate::screens::enter_game(cx, l.state, -1, l.warnings),
                Err(errors) => crate::screens::show_errors(cx, errors),
            },
            OverAct::LoadSave => cx.nav.push(Nav::Push(Box::new(crate::menus::Slots::new(false)))),
            OverAct::MainMenu => {
                cx.end_game();
                cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
            }
        }
    }

    /// Plays the end card's vignette over the screen once (nothing when it has no card to show).
    fn play_card(&mut self, cx: &mut Cx) {
        self.played = true;
        if self.card.is_empty() || !cx.has_game() || !cx.nav.is_empty() {
            return;
        }
        let d = cx.data.clone();
        let scene = emberwake_world::story::scene(&d, cx.gs(), &self.card);
        if json::aget(&scene, "cards").is_empty() {
            return;
        }
        let reel = crate::story::reel_of(&d, &cx.gs().world_id, &scene);
        let screen = crate::story::vignette_screen(cx, reel, |cx: &mut Cx, _| cx.nav.push(Nav::Pop));
        cx.nav.push(Nav::Push(Box::new(screen)));
    }
}

/// The run-over card's title.
pub const OVER_TITLE: &str = "The run is over";
/// The button labels (F4).
pub const LOAD_ACT: &str = "Load the act's start";
pub const LOAD_SAVE: &str = "Load a save";
pub const MAIN_MENU: &str = "Main menu";

impl Screen<GameCx> for RunOverScreen {
    fn id(&self) -> &'static str {
        "run_over"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let missing = self.act_save_missing(cx);
        let mut ui: Ui<OverAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("over_fill", OVER_FILL));
        let labels = [LOAD_ACT, LOAD_SAVE, MAIN_MENU];
        let widths: Vec<i32> = labels.iter().map(|l| button_w(&mut ui, l)).collect();
        let row = widths.iter().sum::<i32>() + 6 * (widths.len() as i32 - 1);
        let scale = ui.p.fonts.scale.max(1.0);
        let inner = ((CARD_W as f32 * scale).ceil() as i32).max(row).max(ui.p.text_w(OVER_TITLE, HEAD).ceil() as i32).min(W - 40 - 2 * PAD);
        let (hl, bl) = (ui.p.line_h(HEAD), ui.p.line_h(BODY));
        let body = lines(&mut ui, &self.text, BODY, inner as f32);
        let bh = (bl + 6.0).ceil() as i32;
        // the buttons in one row when they fit, else one under the other
        let stacked = row > inner;
        let buttons_h = if stacked { 3 * bh + 2 * 4 } else { bh };
        let h = PAD + hl.ceil() as i32 + 8 + (body.len() as f32 * bl).ceil() as i32 + 12 + buttons_h + PAD;
        let r = Rect::new((W - inner - 2 * PAD) / 2, (H - h) / 2, inner + 2 * PAD, h);
        ui.p.panel(r, th().game("defeat_card", rgba(24, 16, 22, 255)), th().foe, 4.0);
        ui.p.text_center((r.x + r.w / 2) as f32, (r.y + PAD) as f32, OVER_TITLE, HEAD, th().foe);
        let mut y = (r.y + PAD) as f32 + hl.ceil() + 8.0;
        for l in &body {
            ui.p.text_center((r.x + r.w / 2) as f32, y, l, BODY, th().text);
            y += bl;
        }
        let acts = [OverAct::LoadAct, OverAct::LoadSave, OverAct::MainMenu];
        let ids = ["run_over.load_act", "run_over.load_save", "run_over.main_menu"];
        let mut x = r.x + (r.w - row) / 2;
        let mut by = r.y + r.h - PAD - buttons_h;
        for (i, label) in labels.iter().enumerate() {
            let enabled = i != 0 || missing.is_empty();
            let card = match i {
                0 if !enabled => tip(label, &missing),
                0 => tip(label, &format!("Start act {} again from its first moment.", self.act)),
                1 => tip(label, "The save slots and the autosave written before the run ended."),
                _ => tip(label, ""),
            };
            let br = if stacked { Rect::new(r.x + (r.w - inner) / 2, by, inner, bh) } else { Rect::new(x, by, widths[i], bh) };
            ui.button_tip(ids[i], br, label, acts[i].clone(), enabled, card);
            if stacked {
                by += bh + 4;
            } else {
                x += widths[i] + 6;
            }
        }
        self.w = ui.widgets;
    }

    /// F3: no way back into the run (Esc and right click do nothing).
    fn back(&mut self, _cx: &mut Cx) {}

    fn auto_focus(&self) -> bool {
        true
    }

    fn tick(&mut self, cx: &mut Cx) {
        if !self.played {
            self.play_card(cx);
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        json!({"kind": self.kind, "card": self.card, "text": self.text, "act": self.act, "act_save": self.key, "act_save_ready": self.act_save_missing(cx).is_empty()})
    }

    fn music(&self, _cx: &Cx) -> Option<String> {
        Some(String::new())
    }

    widget_plumbing!();
}

/// CAMPAIGN1-U tests (CAMPAIGN1_TASKS 1, U rows; W1, W3, W4, F3-F5) on a scratch campaign world `camp` (Scrap and
/// Warded Tallow; Alert, a gauge that rises 31 a day and ends the run at 100; the Red Tithing, a countdown with a
/// warning at 7 days that ends the run at 0; the hub and one node a day away; a loud fight at node A that raises Alert
/// 10 on a win; the end's card as a vignette; Camp and the Keeper's "Rest a day." cost a day and 1 Tallow; zero Tallow
/// costs 2 AP; the quest `job` (that fight) with a checkpoint on day 60 that goes on when missed). Expectations follow
/// from the rules and the fixture's own numbers. Slice 2: the campaign queue, the rest ask, the purse words, the
/// Travel tip's day, the map's deadlines and the start-AP change.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, gfx, rect};
    use crate::screens::Game;
    use crate::story::tests::story_cx;
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_world::{explore, new_game, Data, GameState};

    const ALERT_PER_DAY: i64 = 31;
    const LOUD_WON: i64 = 10;
    const BREACH: i64 = 100;
    const WEEK: i64 = 7;
    const END_TEXT: &str = "The tithing came for the cell.";
    const START_AP: i64 = -2;
    const DEADLINE: i64 = 60;
    /// CAMPAIGN1-TM: the stealth mode's days a leg; the forced march's Alert a leg.
    const STEALTH_DAYS: i64 = 1;
    const FORCED_ALERT: i64 = 5;

    fn with_ids(w: &str, rows: Value) -> Value {
        let rows = json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect();
        Value::Array(rows)
    }

    pub(crate) fn data() -> Data {
        data_with(Value::Null)
    }

    /// `data()` with the manifest's `travel_modes` block (none when null; CAMPAIGN1-TM).
    fn data_with(travel_modes: Value) -> Data {
        data_edit(travel_modes, |_, _| {})
    }

    /// `data_with` after `edit(worlds, catalog)` (a test's extra rows).
    fn data_edit(travel_modes: Value, edit: impl FnOnce(&mut Value, &mut Value)) -> Data {
        let w = "camp";
        let meters = json!([
            {"id": "alert", "name": "Inquisition Alert", "kind": "gauge", "min": 0, "max": BREACH, "unit": "%", "visible": true,
             "hud": {"short": "Alert", "order": 2},
             "bands": [{"id": "guarded", "name": "Guarded", "from": 0, "tone": "calm"}, {"id": "suspicious", "name": "Suspicious", "from": ALERT_PER_DAY, "tone": "warn"},
                       {"id": "breach", "name": "Breach", "from": BREACH, "tone": "fatal"}],
             "rules": [{"id": "a_sus", "at": "suspicious", "warn": "Gates are watched."},
                       {"id": "a_breach", "at": "breach", "effects": [{"effect": "end", "kind": "fail", "card": "the_end", "text": "They found the grate."}]}],
             "inputs": [{"on": "day", "delta": ALERT_PER_DAY}, {"on": "battle", "tags": ["loud"], "outcomes": ["won"], "delta": LOUD_WON},
                        {"on": "leg", "modes": ["forced"], "delta": FORCED_ALERT}]},
            {"id": "clock", "name": "The Red Tithing", "kind": "countdown", "min": 0, "max": 99, "unit": "days", "visible": true,
             "hud": {"short": "Tithing", "order": 1},
             "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}, {"id": "far", "name": "Far", "from": WEEK + 1, "tone": "calm"}],
             "rules": [{"id": "week", "at": WEEK, "warn": "Seven days to the Red Tithing."},
                       {"id": "zero", "at": 0, "effects": [{"effect": "end", "kind": "fail", "card": "the_end", "text": END_TEXT}]}]}
        ]);
        let purse = json!({"currency": {"name": "Scrap", "tip": "Salvaged iron."}, "upkeep": {"name": "Warded Tallow", "short": "Tallow", "tip": "1 a day.", "at_zero": {"road_pct": 100, "start_ap": START_AP}}});
        let worlds = json!({
            w: {"manifest": {"id": w, "name": "Camp", "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}, "purse": purse,
                             "rest": {"days": 1, "cost": 1, "where": "nodes", "on_enter": false, "needs_upkeep": false}}, "own": {
                "locations": with_ids(w, json!([{"id": "hub", "name": "Hub", "rest": true, "map": "yard",
                                                  "markers": [{"id": "m_job", "kind": "encounter", "encounter": "loud_job", "anchor": "job"}]},
                                                 {"id": "a", "name": "Node A"}])),
                "characters": with_ids(w, json!([{"id": "clerk", "name": "Clerk", "role": "npc", "dialogue": "talk"}, {"id": "keeper", "name": "Keeper", "role": "npc", "dialogue": "bunk"}])),
                "dialogues": with_ids(w, json!([{"id": "talk", "start": "n", "services": false, "nodes": [{"id": "n", "text": "A loud job?", "choices": [
                    {"text": "Take it.", "effects": [{"effect": "set_flag", "key": "took_job", "value": true}, {"effect": "fight", "encounter": "loud_job"}]}]}]},
                    {"id": "bunk", "start": "n", "services": false, "nodes": [{"id": "n", "text": "Sleep?", "choices": [{"text": "Rest a day.", "effects": [{"effect": "rest"}]}]}]}])),
                "routes": with_ids(w, json!([{"id": "r_a", "from": "hub", "to": "a", "time": 1}])),
                "encounters": with_ids(w, json!([{"id": "loud_job", "name": "Loud Job", "tags": ["loud"], "location": "a", "enemies": []}])),
                "vignettes": with_ids(w, json!([{"id": "the_end", "name": "The end", "cards": [{"style": "title", "title": "Extinguished", "subtitle": "The end"}]},
                                                {"id": "news", "name": "News", "cards": [{"style": "title", "title": "News"}]}])),
                "quests": with_ids(w, json!([{"id": "job", "name": "The Job", "template": "clear_site", "steps": [{"text": "Clear A", "encounter": "loud_job"}]}])),
                "checkpoints": with_ids(w, json!([{"id": "cp", "name": "The Job", "act": 1, "quest": "job", "deadline": DEADLINE, "warn_days": [], "mode": "go_on"}])),
                "meters": with_ids(w, meters)
            }}
        });
        let mut worlds = worlds;
        if !travel_modes.is_null() {
            worlds[w]["manifest"]["travel_modes"] = travel_modes;
        }
        let mut catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter"}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
        });
        edit(&mut worlds, &mut catalog);
        // the quest template of `job`; the hub's map: the spawn, and the job's marker one step to the right
        let game = json!({"quests": [{"id": "clear_site", "objectives": [{"kind": "win_encounter"}], "rewards": {}}],
                          "maps": [{"id": "yard", "rows": ["#####", "#...#", "#####"], "anchors": [{"id": "spawn", "x": 1, "y": 1}, {"id": "job", "x": 2, "y": 1}]}]});
        Data::from_values(&catalog, &worlds, &game)
    }

    /// A debug context playing a new game in `camp` at the hub (both nodes known), Alert at `alert`, the Red Tithing
    /// running with `days` left.
    pub(crate) fn camp_cx(alert: i64, days: i64) -> Cx {
        camp_cx_in(data(), alert, days)
    }

    /// `camp_cx` on `d`.
    fn camp_cx_in(d: Data, alert: i64, days: i64) -> Cx {
        let mut cx = story_cx(d);
        let d = cx.data.clone();
        let mut gs: GameState = new_game::create_for_world(&d, "camp", 1).expect("a new game");
        for n in ["hub", "a"] {
            gs.set_flag(&explore::visited_flag(&format!("camp:{n}")), json!(true));
        }
        explore::enter(&d, &mut gs, "hub", explore::SPAWN);
        let time = gs.time;
        let c = gs.campaign.as_mut().expect("a campaign");
        c.meters.get_mut("alert").unwrap().value = alert;
        let clock = c.meters.get_mut("clock").unwrap();
        clock.running = true;
        clock.due = time + days;
        c.sync(time);
        cx.game = Some(Game::new(gs, -1, false));
        cx
    }

    /// The screen the last navigation opened (Push or Reset), and how.
    fn take_nav(cx: &mut Cx) -> (&'static str, Box<dyn Screen<GameCx>>) {
        match cx.nav.pop() {
            Some(Nav::Push(s)) => ("push", s),
            Some(Nav::Reset(s)) => ("reset", s),
            Some(Nav::Pop) => panic!("a pop, not a screen"),
            None => panic!("nothing was opened"),
        }
    }

    /// The text audit of `screen` at every text size: nothing cut, past the canvas, overlapping or out of its widget.
    fn audit_clean(screen: &mut dyn Screen<GameCx>, cx: &mut Cx) {
        let mut g = gfx();
        for (size, scale) in TEXT_SIZES {
            let recs: Vec<TextRec> = frame(screen, cx, &mut g, scale as f32);
            let boxes: Vec<(String, f32, f32, f32, f32)> = screen
                .widgets()
                .iter()
                .map(|w| {
                    let (x, y, ww, hh) = rect(w);
                    (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{} at {size}: {v}", screen.id());
        }
    }

    /// W4, P1 (task U: "a campaign world HUD"): the world map's bar shows Scrap and Tallow (their names on the hover
    /// cards), then the Red Tithing and Alert by `hud.order`, each as its short name and value with the unit; it reads
    /// clean at every text size.
    #[test]
    fn the_campaign_hud_shows_the_purse_names_and_the_meter_chips() {
        let mut cx = camp_cx(12, 37);
        let mut g = gfx();
        let mut wm = crate::worldmap::WorldMap::new();
        let recs = frame(&mut wm, &mut cx, &mut g, 1.0);
        let mut bar: Vec<Value> = wm.widgets().into_iter().filter(|w| rect(w).1 + rect(w).3 <= crate::hud::HUD_H as i64).collect();
        bar.sort_by_key(|w| rect(w).0);
        let ids: Vec<String> = bar.iter().map(|w| s(json::dict(w), "id")).collect();
        assert_eq!(ids[1..5], ["hud.gold", "hud.supplies", "hud.meter.clock", "hud.meter.alert"], "{ids:?}");
        let tip_of = |id: &str| bar.iter().find(|w| w["id"] == id).map(|w| w["tip"].to_string()).unwrap_or_default();
        assert!(tip_of("hud.gold").contains("Scrap") && tip_of("hud.gold").contains("Salvaged iron."), "{}", tip_of("hud.gold"));
        assert!(tip_of("hud.supplies").contains("Warded Tallow"), "{}", tip_of("hud.supplies"));
        assert!(tip_of("hud.meter.alert").contains("Inquisition Alert") && tip_of("hud.meter.alert").contains("Band: Guarded"), "{}", tip_of("hud.meter.alert"));
        let texts: Vec<&str> = recs.iter().map(|r| r.text.as_str()).collect();
        for t in ["Tithing ", "37 days", "Alert ", "12%"] {
            assert!(texts.contains(&t), "'{t}' on the bar: {texts:?}");
        }
        for (size, scale) in TEXT_SIZES {
            let recs: Vec<TextRec> = frame(&mut wm, &mut cx, &mut g, scale as f32).into_iter().filter(|r| r.y + r.h / 2.0 < crate::hud::HUD_H as f32).collect();
            let boxes: Vec<(String, f32, f32, f32, f32)> = wm
                .widgets()
                .iter()
                .filter(|w| rect(w).1 < crate::hud::HUD_H as i64)
                .map(|w| {
                    let (x, y, ww, hh) = rect(w);
                    (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the campaign bar at {size}: {v}");
        }
    }

    /// W1: one trip's day moves Alert into Suspicious and the Red Tithing to 7 days; each warning logs a line and waits
    /// as a notice, shown one at a time in event order (gauges before countdowns, X4) when the map has nothing open.
    #[test]
    fn warnings_log_and_show_one_notice_each_in_order() {
        let mut cx = camp_cx(0, WEEK + 1);
        let mut g = gfx();
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        wm.click_id(&mut cx, "node.camp.a").expect("node a");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        wm.click_id(&mut cx, "world_map.travel").expect("travel");
        assert!(cx.nav.is_empty(), "no run-ending step: no ask");
        assert_eq!(cx.gs().location_id, "camp:a");
        assert!(cx.log.iter().any(|l| l == "Inquisition Alert: Gates are watched."), "{:?}", cx.log);
        assert!(cx.log.iter().any(|l| l == "The Red Tithing: Seven days to the Red Tithing."), "{:?}", cx.log);
        let mut seen = Vec::new();
        for _ in 0..2 {
            wm.tick(&mut cx);
            let (how, n) = take_nav(&mut cx);
            assert_eq!((how, n.id()), ("push", "campaign_notice"));
            seen.push((n.state(&cx)["title"].clone(), n.state(&cx)["tone"].clone()));
        }
        assert_eq!(seen, [(json!("Inquisition Alert: Suspicious"), json!("warn")), (json!("The Red Tithing: Due"), json!("danger"))]);
        wm.tick(&mut cx);
        assert!(cx.nav.is_empty(), "every notice shown; the run goes on");
    }

    /// W3, F3: a trip whose day would end the run asks first with No as the default; No changes nothing; Yes travels,
    /// the run ends, nothing is autosaved, and the next tick shows only the run-over screen.
    #[test]
    fn a_trip_that_would_end_the_run_asks_first() {
        let mut cx = camp_cx(0, 1);
        cx.debug = false;
        let mut g = gfx();
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        wm.click_id(&mut cx, "node.camp.a").expect("node a");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let time = cx.gs().time;
        wm.click_id(&mut cx, "world_map.travel").expect("travel");
        let (how, ask) = take_nav(&mut cx);
        let st = ask.state(&cx);
        assert_eq!((how, ask.id(), st["default"].clone()), ("push", "ask", json!("no")));
        assert_eq!(st["question"], json!(format!("This trip would end the run: {END_TEXT} Travel anyway?")));
        assert_eq!(cx.gs().time, time, "nothing moved before the answer (No changes nothing)");
        cx.answers.push(ASK_TRIP.to_string());
        wm.tick(&mut cx);
        assert_eq!(cx.gs().time, time + 1, "Yes: the trip");
        assert!(is_over(&cx));
        cx.autosave();
        assert!(!save::exists(&cx.save_dir, save::AUTOSAVE_SLOT) && !cx.log.iter().any(|l| l.starts_with("Autosave failed")), "F3: no autosave, no complaint");
        wm.tick(&mut cx);
        let (how, over) = take_nav(&mut cx);
        assert_eq!((how, over.id()), ("reset", "run_over"));
        assert_eq!(over.state(&cx)["text"], json!(END_TEXT));
    }

    /// W3: stepping onto a marker fight a win of which would end the run asks first (No as the default): the party is
    /// back on the tile it stepped from with the encounter counter started over (as after fleeing), nothing fought;
    /// Yes fights from there.
    #[test]
    fn a_marker_fight_that_could_end_the_run_asks_and_no_steps_back() {
        let mut cx = camp_cx(BREACH - LOUD_WON / 2, 30);
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        let from = cx.gs().position;
        cx.gs_mut().explore.insert("steps".into(), json!(3));
        map.key(&mut cx, Key::Right);
        let (how, mut ask) = take_nav(&mut cx);
        let st = ask.state(&cx);
        assert_eq!((how, ask.id(), st["default"].clone()), ("push", "ask", json!("no")));
        assert_eq!(st["question"], json!("Winning this fight would end the run: They found the grate. Fight anyway?"));
        assert_eq!(cx.gs().position, from, "the party stands where it stepped from");
        assert_eq!(json::i(&cx.gs().explore, "steps", -1), 0, "the encounter counter started over");
        assert!(cx.nav.is_empty(), "nothing fought before the answer (No is the end of it)");
        ask.key(&mut cx, Key::Char('y'));
        let (how, fight) = take_nav(&mut cx);
        assert_eq!((how, fight.id()), ("push", "debug_fight"), "Yes: the fight");
        assert_eq!((fight.state(&cx)["encounter"].clone(), fight.state(&cx)["marker"].clone()), (json!("camp:loud_job"), json!("m_job")));
        assert!(cx.log.iter().any(|l| l == "Fight: Loud Job!"));
        // at a safe Alert the step fights at once
        let mut cx = camp_cx(0, 30);
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        map.key(&mut cx, Key::Right);
        assert_eq!(take_nav(&mut cx).1.id(), "debug_fight");
    }

    /// W3: a dialogue choice that ends in a fight a win of which would end the run closes the talk and asks; the
    /// choice's other effects stay applied; No fights nothing, Yes fights.
    #[test]
    fn a_dialogue_fight_that_could_end_the_run_asks_first() {
        let mut cx = camp_cx(BREACH - LOUD_WON / 2, 30);
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        map.talk(&mut cx, "clerk");
        map.key(&mut cx, Key::Char('1'));
        let (how, mut ask) = take_nav(&mut cx);
        assert_eq!((how, ask.id(), ask.state(&cx)["default"].clone()), ("push", "ask", json!("no")));
        assert!(cx.nav.is_empty() && !cx.busy(), "the talk is over; nothing fought yet");
        assert!(map.state(&cx)["dialogue"].is_null());
        assert!(cx.gs().flag_true("took_job"), "the choice's other effects stay applied");
        ask.key(&mut cx, Key::Char('y'));
        let (_, fight) = take_nav(&mut cx);
        assert_eq!((fight.id(), fight.state(&cx)["source"].clone()), ("debug_fight", json!("dialogue")));
    }

    /// W3: a fight one of whose outcomes would end the run gets the ask (a win at Alert 95 reaches Breach); at Alert 50
    /// no outcome does.
    #[test]
    fn a_fight_that_could_end_the_run_is_asked_about() {
        let cx = camp_cx(BREACH - LOUD_WON / 2, 30);
        assert_eq!(fight_ask(&cx, "loud_job"), Some("Winning this fight would end the run: They found the grate. Fight anyway?".to_string()));
        assert_eq!(fight_warning(&cx, "loud_job"), Some("Winning this fight would end the run: They found the grate.".to_string()));
        let cx = camp_cx(BREACH / 2, 30);
        assert_eq!(fight_ask(&cx, "loud_job"), None);
    }

    /// F4, F5: an act start writes the keyed act-start save (not in a debug game); once the run is over the screen plays
    /// the card, then offers Load the act's start (that save), Load a save (the load menu) and Main menu; Esc does
    /// nothing; the save menu refuses every slot; without the act-start save its button is off and says why.
    #[test]
    fn the_run_over_screen_loads_the_acts_start_a_save_or_the_main_menu() {
        let mut cx = camp_cx(0, 30);
        let d = cx.data.clone();
        let set_act = |cx: &mut Cx, act: i64| {
            let mut evs = Vec::new();
            campaign::apply_effects(&d, cx.gs_mut(), &[json::dict(&json!({"effect": "set_act", "act": act})).clone()], &mut evs);
            evs
        };
        let evs = set_act(&mut cx, 2);
        let key = s(json::dict(&evs[0]), "save");
        events(&mut cx, &evs);
        assert!(!key.is_empty() && !save::exists_keyed(&cx.save_dir, &key), "a debug game writes no act-start save");
        assert!(cx.log.iter().any(|l| l == "Act 2 begins."));
        cx.debug = false;
        let evs = set_act(&mut cx, 3);
        let key = s(json::dict(&evs[0]), "save");
        events(&mut cx, &evs);
        assert!(save::exists_keyed(&cx.save_dir, &key), "F5: the act-start save");
        campaign::apply_effects(&d, cx.gs_mut(), &[json::dict(&json!({"effect": "end", "kind": "fail", "card": "the_end", "text": END_TEXT})).clone()], &mut Vec::new());
        let mut over = RunOverScreen::new(&cx);
        over.tick(&mut cx);
        let (how, card) = take_nav(&mut cx);
        assert_eq!((how, card.id(), card.state(&cx)["reel"].clone()), ("push", "vignette", json!("camp:the_end")), "the card plays first");
        over.tick(&mut cx);
        assert!(cx.nav.is_empty(), "once");
        let mut g = gfx();
        frame(&mut over, &mut cx, &mut g, 1.0);
        let on: Vec<(String, bool)> = over.widgets().iter().map(|w| (s(json::dict(w), "id"), json::b(json::dict(w), "enabled", false))).collect();
        assert_eq!(on, [("run_over.load_act".to_string(), true), ("run_over.load_save".to_string(), true), ("run_over.main_menu".to_string(), true)]);
        over.back(&mut cx);
        assert!(cx.nav.is_empty(), "F3: no way back into the run");
        let mut slots = crate::menus::Slots::new(true);
        frame(&mut slots, &mut cx, &mut g, 1.0);
        assert!(slots.widgets().iter().filter(|w| s(json::dict(w), "id").starts_with("slot.") && s(json::dict(w), "kind") == "card").all(|w| !json::b(json::dict(w), "enabled", true)), "F3: no slot takes a save");
        over.click_id(&mut cx, "run_over.load_save").expect("load a save");
        assert_eq!(take_nav(&mut cx).1.id(), "load_slots");
        over.click_id(&mut cx, "run_over.load_act").expect("load the act's start");
        let (how, _) = take_nav(&mut cx);
        assert_eq!(how, "reset");
        let c = cx.gs().campaign.as_ref().expect("a campaign");
        assert_eq!((c.act, c.is_over()), (3, false), "the act's start: act 3, the run goes on");
        // without the act-start save the button is off and says why; Main menu ends the game
        let mut cx = camp_cx(0, 30);
        campaign::apply_effects(&d, cx.gs_mut(), &[json::dict(&json!({"effect": "end", "kind": "fail"})).clone()], &mut Vec::new());
        let mut over = RunOverScreen::new(&cx);
        frame(&mut over, &mut cx, &mut g, 1.0);
        assert!(over.click_id(&mut cx, "run_over.load_act").is_err());
        assert!(over.widget_rect("run_over.load_act").is_some_and(|(_, t)| t.iter().any(|l| l == "There is no save from the start of act 1.")));
        over.click_id(&mut cx, "run_over.main_menu").expect("main menu");
        assert_eq!(take_nav(&mut cx).1.id(), "main_menu");
        assert!(!cx.has_game());
    }

    /// TEXTFIT1: the notice (a long text) and the run-over screen read clean at every text size.
    #[test]
    fn the_notice_and_the_run_over_screen_read_clean_at_every_text_size() {
        let mut cx = camp_cx(0, 30);
        let long = "One more loud job and they find the grate. The run would end: quiet jobs and bribes at Tess lower Alert, a rest does not.";
        audit_clean(&mut CampaignNotice::new("Inquisition Alert: Lockdown", "danger", long), &mut cx);
        let d = cx.data.clone();
        campaign::apply_effects(&d, cx.gs_mut(), &[json::dict(&json!({"effect": "end", "kind": "fail", "card": "", "text": long})).clone()], &mut Vec::new());
        let mut over = RunOverScreen::new(&cx);
        audit_clean(&mut over, &mut cx);
        // the three buttons stand in one row under the text at Normal size
        frame(&mut over, &mut cx, &mut gfx(), 1.0);
        let rows: Vec<i64> = over.widgets().iter().map(|w| rect(w).1).collect();
        assert!(rows.len() == 3 && rows.iter().all(|y| *y == rows[0]), "{rows:?}");
    }

    // ---- slice 2 ----

    fn items(rows: Value) -> Vec<Dict> {
        json::arr(&rows).iter().map(|v| json::dict(v).clone()).collect()
    }

    /// T2: the region map plays the campaign queue head first when nothing is open: a `{vignette}` through the story
    /// queue, then a `{battle}` as source `trigger` (it settles like a dialogue fight: the loud win's Alert input), then
    /// a `{dialogue}` scene as the conversation with its speaker; each item leaves the queue as it starts.
    #[test]
    fn the_region_map_plays_the_campaign_queue_head_first() {
        let mut cx = camp_cx(0, 30);
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        cx.gs_mut().campaign.as_mut().unwrap().pending = items(json!([{"vignette": "news"}, {"battle": "loud_job"}, {"dialogue": "bunk", "speaker": "keeper"}]));
        map.tick(&mut cx);
        let (how, v) = take_nav(&mut cx);
        assert_eq!((how, v.id(), v.state(&cx)["reel"].clone()), ("push", "vignette", json!("camp:news")));
        assert_eq!(campaign::pending_head(cx.gs()).map(|h| s(h, "battle")), Some("loud_job".to_string()), "the vignette left the queue");
        // the reel ended: off the story queue
        emberwake_world::story::vignette_done(cx.gs_mut(), "camp:news");
        map.tick(&mut cx);
        let (how, f) = take_nav(&mut cx);
        let st = f.state(&cx);
        assert_eq!((how, f.id(), st["source"].clone(), st["encounter"].clone()), ("push", "debug_fight", json!(trigger::FIGHT_SOURCE), json!("camp:loud_job")));
        assert!(cx.nav.is_empty(), "no outcome would end the run: no notice");
        assert!(cx.log.iter().any(|l| l == "Fight: Loud Job!"));
        assert_eq!(campaign::pending_head(cx.gs()).map(|h| s(h, "dialogue")), Some("bunk".to_string()), "the battle left the queue");
        let d = cx.data.clone();
        let fight = json::dict(&st).clone();
        let res = emberwake_world::battle::finish_world(&d, cx.gs_mut(), trigger::FIGHT_SOURCE, &fight, "won", None);
        assert!(json::b(&res, "ok", false), "{res:?}");
        assert_eq!(cx.gs().campaign.as_ref().unwrap().value("alert", cx.gs().time), LOUD_WON, "settled as a battle: the loud win's input");
        map.tick(&mut cx);
        assert!(cx.nav.is_empty() && cx.busy(), "the scene opens as the conversation");
        let talk = map.state(&cx)["dialogue"].clone();
        assert_eq!((talk["text"].clone(), talk["speaker_name"].clone()), (json!("Sleep?"), json!("Keeper")));
        assert!(cx.log.iter().any(|l| l == "You speak with Keeper."));
        assert!(campaign::pending_head(cx.gs()).is_none());
    }

    /// T2, W3: a queued battle cannot be refused; when an outcome would end the run it starts with the notice over it.
    /// The world map plays a queued battle too but leaves a queued dialogue for the region map.
    #[test]
    fn a_queued_battle_that_could_end_the_run_gets_the_notice_and_the_world_map_leaves_talks() {
        let mut cx = camp_cx(BREACH - LOUD_WON / 2, 30);
        let mut wm = crate::worldmap::WorldMap::new();
        cx.gs_mut().campaign.as_mut().unwrap().pending = items(json!([{"battle": "loud_job"}, {"dialogue": "bunk", "speaker": "keeper"}]));
        wm.tick(&mut cx);
        let (_, notice) = take_nav(&mut cx);
        assert_eq!((notice.id(), notice.state(&cx)["title"].clone()), ("campaign_notice", json!(FIGHT_WARNING)));
        assert_eq!(notice.state(&cx)["text"], json!(fight_warning(&cx, "loud_job").unwrap()));
        assert_eq!(take_nav(&mut cx).1.id(), "debug_fight", "the battle under the notice");
        wm.tick(&mut cx);
        assert!(cx.nav.is_empty(), "the world map has no talk box: the scene waits");
        assert_eq!(campaign::pending_head(cx.gs()).map(|h| s(h, "dialogue")), Some("bunk".to_string()));
    }

    /// W3, R3: a reply that rests (the `rest` effect) asks first, No as the default, when the rest's day would end the
    /// run: nothing rests before the answer and the talk stays open; Yes rests (the run ends). With days to spare the
    /// reply rests at once.
    #[test]
    fn a_reply_whose_rest_would_end_the_run_asks_first() {
        let mut cx = camp_cx(0, 1);
        cx.gs_mut().party.members[0].hp_lost = 3;
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        map.talk(&mut cx, "keeper");
        let time = cx.gs().time;
        map.key(&mut cx, Key::Char('1'));
        let (how, ask) = take_nav(&mut cx);
        let st = ask.state(&cx);
        assert_eq!((how, ask.id(), st["default"].clone()), ("push", "ask", json!("no")));
        assert_eq!(st["question"], json!(format!("Resting would end the run: {END_TEXT} Rest anyway?")));
        assert_eq!((cx.gs().time, cx.gs().party.members[0].hp_lost), (time, 3), "nothing rested before the answer");
        assert!(!map.state(&cx)["dialogue"].is_null(), "the talk stays open");
        cx.answers.push(ASK_TALK_REST.to_string());
        map.tick(&mut cx);
        let days = campaign::purse::rest_rule(&cx.data, "camp").days as i64;
        assert_eq!(cx.gs().time, time + days, "Yes: the rest");
        assert!(is_over(&cx));
        let mut cx = camp_cx(0, 30);
        cx.gs_mut().party.members[0].hp_lost = 3;
        let mut map = crate::region::RegionMap::new(&mut cx);
        cx.nav.clear();
        map.talk(&mut cx, "keeper");
        let time = cx.gs().time;
        map.key(&mut cx, Key::Char('1'));
        assert!(cx.nav.is_empty(), "no ask");
        assert_eq!((cx.gs().time, cx.gs().party.members[0].hp_lost), (time + days, 0), "rested at once");
    }

    /// TERMS1 TU2-TU4 (rule TM6: labels and Results lines `short`, sentences `word`): in the camp world the Results
    /// read "Scrap +N" and "Objective bonus: Scrap +N", the journal's reward line "N Scrap", the party sheet "Scrap N",
    /// and the shop names the upkeep "Tallow" and the currency "Scrap" (the total, the purse after the trade, the
    /// refusal); a world without a purse keeps today's "Gold +N" and "N gold".
    #[test]
    fn results_journal_and_shop_read_the_worlds_purse_names() {
        use emberwake_world::terms::{self, CURRENCY, UPKEEP};
        let d = data_edit(Value::Null, |worlds, _| {
            let own = &mut worlds["camp"]["own"];
            let mut chars = json::arr(&own["characters"]).to_vec();
            chars.extend(json::arr(&with_ids("camp", json!([{"id": "trader", "name": "Trader", "role": "npc", "dialogue": "trade"}]))).iter().cloned());
            own["characters"] = Value::Array(chars);
            let mut trees = json::arr(&own["dialogues"]).to_vec();
            trees.extend(json::arr(&with_ids("camp", json!([{"id": "trade", "start": "n", "nodes": [{"id": "n", "text": "Buying?", "choices": [{"text": "Bye."}]}]}]))).iter().cloned());
            own["dialogues"] = Value::Array(trees);
            own["shops"] = with_ids("camp", json!([{"id": "store", "name": "Store", "keeper": "trader", "supply_price": 2}]));
        });
        let (cur, word) = (terms::short(&d, "camp", CURRENCY), terms::word(&d, "camp", CURRENCY));
        assert_eq!((cur.as_str(), word.as_str(), terms::short(&d, "camp", UPKEEP).as_str()), ("Scrap", "Scrap", "Tallow"), "the fixture's purse block");
        assert_eq!((terms::short(&d, "", CURRENCY), terms::word(&d, "", CURRENCY)), ("Gold".to_string(), "gold".to_string()), "no purse: the defaults");
        let mut cx = camp_cx_in(d, 0, 30);
        let mut g = gfx();
        // TU2: the Results popup in the battle's world
        let rewards = json!({"foes": 1, "defeated": {"slain": 1}, "gold": 12, "objective": {"gold": 10}});
        let res = json::obj([("ok", json!(true)), ("rewards", rewards.clone())]);
        let mut r = crate::hub::Results::new(&res, "won", &json::Dict::new());
        let t: Vec<String> = frame(&mut r, &mut cx, &mut g, 1.0).into_iter().map(|x| x.text).collect();
        assert!(t.iter().any(|x| x == "Scrap +12") && t.iter().any(|x| x == "Objective bonus: Scrap +10"), "{t:?}");
        // TU3: the journal's reward line in the game's world
        assert_eq!(crate::hub::reward_lines(&cx, json::dict(&json!({"gold": 12}))), ["12 Scrap"]);
        // TU4: the party sheet's stats line
        let first = cx.gs().party.members[0].id.clone();
        let mut sheet = crate::party::Character::new(&mut cx, &first, "stats");
        let t: Vec<String> = frame(&mut sheet, &mut cx, &mut g, 1.0).into_iter().map(|x| x.text).collect();
        assert!(t.iter().any(|x| x.ends_with(&format!("·  Scrap {}", cx.gs().party.gold))), "{t:?}");
        // TU4: the shop: the supply row, the total, the purse after and the refusal
        cx.gs_mut().party.gold = 0;
        let mut map = crate::region::RegionMap::new(&mut cx);
        map.talk(&mut cx, "trader");
        frame(&mut map, &mut cx, &mut g, 1.0);
        let choices = json::aget(json::dict(&map.state(&cx)["dialogue"]), "choices").to_vec();
        let k = choices.iter().position(|c| s(json::dict(c), "service") == "shop").expect("the trader's shop reply");
        map.click_id(&mut cx, &format!("choice.{k}")).expect("the shop reply");
        let t: Vec<String> = frame(&mut map, &mut cx, &mut g, 1.0).into_iter().map(|x| x.text).collect();
        assert!(t.iter().any(|x| x == "Tallow") && t.iter().any(|x| x == "Total: 0 Scrap") && t.iter().any(|x| x == "Scrap: 0 -> 0"), "{t:?}");
        let row = map.widgets().into_iter().map(|w| s(json::dict(&w), "id")).find(|id| id.starts_with("shop.row.")).expect("the supply row");
        map.click_id(&mut cx, &row).expect("adds the supplies");
        frame(&mut map, &mut cx, &mut g, 1.0);
        let card = map.widgets().into_iter().find(|w| w["id"] == "trade.confirm").expect("confirm")["tip"].to_string();
        assert!(card.contains("Not enough Scrap: 2 more needed."), "{card}");
    }

    /// P1: the purse words of log lines and tips: the camp world's short names; a world without a purse keeps today's
    /// "gold" and "supplies", and its Camp costs today's 3.
    #[test]
    fn purse_words_take_the_worlds_names_and_keep_todays() {
        let d = data();
        use crate::hud::{purse_word_in, PurseWord};
        assert_eq!((purse_word_in(&d, "camp", PurseWord::Currency), purse_word_in(&d, "camp", PurseWord::Upkeep)), ("Scrap".to_string(), "Tallow".to_string()));
        assert_eq!((purse_word_in(&d, "", PurseWord::Currency), purse_word_in(&d, "", PurseWord::Upkeep)), ("gold".to_string(), "supplies".to_string()));
        assert_eq!(campaign::purse::rest_rule(&d, "").cost as i64, emberwake_world::progression::CAMP_COST, "the Ashlands Camp reads 'Make camp (3 supplies)'");
        // a camp's log line names the upkeep and the day
        let mut cx = camp_cx(0, 30);
        crate::region::log_events(&mut cx, &[json!({"kind": "camped", "spent": 1, "restored": 4, "days": 1})]);
        assert_eq!(cx.log.last().map(String::as_str), Some("The party makes camp (-1 Tallow, +4 HP, 1 day)."));
        crate::region::log_events(&mut cx, &[json!({"kind": "defeat", "mode": "retreat", "spent": 2, "days": 2, "location": "camp:hub"})]);
        assert_eq!(cx.log.last().map(String::as_str), Some("Defeated. The party falls back to Hub (2 days, -2 Tallow)."));
    }

    /// P1, R1, W5, B10: the world map's Camp names the rest's cost in the upkeep's name and its day; the Travel tip
    /// gives the trip in days and upkeep, the arrival day and the Red Tithing's days left then; node A's tip carries the
    /// checkpoint line of the quest whose node it is.
    #[test]
    fn the_world_map_names_the_upkeep_the_arrival_day_and_the_deadlines() {
        let mut cx = camp_cx(0, 30);
        let mut g = gfx();
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let rule = campaign::purse::rest_rule(&cx.data, "camp");
        let camp = wm.widgets().into_iter().find(|w| w["id"] == "world_map.camp").expect("Camp");
        assert_eq!(camp["text"], json!(format!("Make camp ({} Tallow)", rule.cost)));
        let (_, node_tip) = wm.widget_rect("node.camp.a").expect("node A");
        assert!(node_tip.contains(&emberwake_world::board::checkpoint_text(DEADLINE)), "{node_tip:?}");
        let (_, hub_tip) = wm.widget_rect("node.camp.hub").expect("the hub");
        assert!(!hub_tip.iter().any(|l| l.starts_with("Checkpoint")), "{hub_tip:?}");
        wm.click_id(&mut cx, "node.camp.a").expect("node a");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let gs = cx.gs().clone();
        let trip = emberwake_world::travel::trip(&cx.data, &gs, &gs.location_id, "camp:a");
        let time = json::i(&trip, "time", 0);
        let (_, travel_tip) = wm.widget_rect("world_map.travel").expect("Travel");
        let want = format!("{}, {} Tallow. Roads can be ambushed. Arrive on day {}, {} left.", days(time), json::i(&trip, "supplies", 0), gs.time + time + 1, days(30 - time));
        assert!(travel_tip.iter().any(|l| l.contains(&want)) || travel_tip.join(" ").contains(&want), "{travel_tip:?} wants {want}");
        let mut cx = camp_cx(0, 30);
        cx.gs_mut().party.members[0].hp_lost = 3;
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let (_, camp_tip) = wm.widget_rect("world_map.camp").expect("Camp");
        assert!(camp_tip.join(" ").contains(&format!("Takes {}.", days(rule.days as i64))), "{camp_tip:?}");
    }

    /// CAMPAIGN1-TM: the world map's mode button shows only in a world with travel modes; it names the party's mode
    /// (the default first) and cycles it with a log line; the Travel tip and the road label count the mode's days; a
    /// forced march whose leg input would reach Breach asks before the trip, though the trip's days alone would not.
    #[test]
    fn the_world_map_toggles_the_travel_mode() {
        let mut g = gfx();
        let mut cx = camp_cx(0, 30);
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert!(wm.widget_rect("world_map.mode").is_none(), "no modes: no button");
        let modes = json!({"default": "stealth", "modes": [{"id": "forced", "name": "Forced march"}, {"id": "stealth", "name": "Stealth", "days": STEALTH_DAYS, "road_pct": 50}]});
        let mut cx = camp_cx_in(data_with(modes), BREACH - ALERT_PER_DAY - FORCED_ALERT, 30);
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let label = |wm: &crate::worldmap::WorldMap| wm.widgets().into_iter().find(|w| w["id"] == "world_map.mode").map(|w| w["text"].clone());
        assert_eq!(label(&wm), Some(json!("Travel: Stealth")), "the default");
        wm.click_id(&mut cx, "node.camp.a").expect("node a");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let leg = 1 + STEALTH_DAYS;
        let (_, travel_tip) = wm.widget_rect("world_map.travel").expect("Travel");
        assert!(travel_tip.join(" ").contains(&format!("Stealth: {}, {leg} Tallow.", days(leg))), "{travel_tip:?}");
        assert_eq!(wm.state(&cx)["view"]["routes"][0]["days"], json!(leg), "the road label's days");
        wm.click_id(&mut cx, "world_map.mode").expect("mode");
        assert_eq!(cx.log.last().map(String::as_str), Some("Travel mode: Forced march."));
        assert_eq!(cx.gs().campaign.as_ref().unwrap().mode, "forced");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(label(&wm), Some(json!("Travel: Forced march")));
        // TEXTFIT1: the longer label fits its button at every text size
        for (size, scale) in TEXT_SIZES {
            let recs = frame(&mut wm, &mut cx, &mut g, scale as f32);
            let w = wm.widgets().into_iter().find(|w| w["id"] == "world_map.mode").expect("the mode button");
            let (x, y, ww, hh) = rect(&w);
            let (x, y, ww, hh) = (x as f32, y as f32, ww as f32, hh as f32);
            let inside: Vec<TextRec> = recs.into_iter().filter(|r| r.x + r.w / 2.0 > x && r.x + r.w / 2.0 < x + ww && r.y + r.h / 2.0 > y && r.y + r.h / 2.0 < y + hh).collect();
            assert!(!inside.is_empty(), "{size}: the label is drawn");
            let v = aegis_ui::text_audit::issues(&inside, W as f32, H as f32, &[("world_map.mode".to_string(), x, y, ww, hh)]);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the mode button at {size}: {v}");
        }
        frame(&mut wm, &mut cx, &mut g, 1.0);
        // the forced leg is 1 day: the day input alone stops short of Breach, the leg input reaches it
        assert!(campaign::preview_days(&cx.data, cx.gs(), 1).is_empty());
        let time = cx.gs().time;
        wm.click_id(&mut cx, "world_map.travel").expect("travel");
        let (how, ask) = take_nav(&mut cx);
        assert_eq!((how, ask.id()), ("push", "ask"));
        assert_eq!(ask.state(&cx)["question"], json!("This trip would end the run: They found the grate. Travel anyway?"));
        assert_eq!(cx.gs().time, time, "nothing moved before the answer");
        wm.click_id(&mut cx, "world_map.mode").expect("mode");
        assert_eq!(cx.gs().campaign.as_ref().unwrap().mode, "", "back to the default: kept as none");
    }

    /// P3, W6: the battle bridge's start-AP change lowers each party unit's round-1 AP by the mods' `start_ap` (never
    /// below 0) and leaves guests and enemies alone; the round's start snapshot shows it; no mods change nothing.
    #[test]
    fn the_start_ap_change_lowers_the_party_ap_not_below_zero() {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.app.encounter = "goblin_ambush".into();
        cx.app.seed = Some(1);
        let mut b = crate::battle::BattleScreen::sandbox(&cx);
        let s = b.host.session.as_mut().expect("the ambush starts");
        let heroes = s.w.allies.len() - s.guests.len();
        let before: Vec<i32> = s.w.units.iter().map(|u| u.ap).collect();
        crate::battle::lower_start_ap(s, heroes, &campaign::purse::BattleMods::default());
        assert_eq!(s.w.units.iter().map(|u| u.ap).collect::<Vec<_>>(), before, "no mods: no change");
        let mods = campaign::purse::BattleMods { start_ap: START_AP, note: String::new() };
        s.w.units[0].ap = 1;
        let before: Vec<i32> = s.w.units.iter().map(|u| u.ap).collect();
        crate::battle::lower_start_ap(s, heroes, &mods);
        let want: Vec<i32> = before.iter().enumerate().map(|(i, &ap)| if i < heroes { campaign::purse::start_ap(ap as i64, &mods) as i32 } else { ap }).collect();
        assert_eq!(s.w.units.iter().map(|u| u.ap).collect::<Vec<_>>(), want);
        assert_eq!(s.w.units[0].ap, 0, "not below 0");
        assert_eq!(s.history.last().unwrap().snapshots[0].iter().map(|u| u.ap).collect::<Vec<_>>(), want, "the start snapshot shows it");
        // at zero Tallow the camp world's mods are the purse's (logged once at the battle's start)
        let mut cx = camp_cx(0, 30);
        cx.gs_mut().party.supplies = 0;
        let mods = campaign::purse::battle_mods(&cx.data, cx.gs());
        assert_eq!((mods.start_ap, mods.note.clone()), (START_AP, format!("No Warded Tallow: {START_AP} AP")));
        cx.gs_mut().party.supplies = 1;
        assert_eq!(campaign::purse::battle_mods(&cx.data, cx.gs()), campaign::purse::BattleMods::default());
        // the bridge: a world battle at zero Tallow logs the note once and starts the heroes lower than at 1 Tallow
        let fight = json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": "camp:loud_job"})).clone();
        let fed = crate::battle::BattleScreen::world(&mut cx, &fight, trigger::FIGHT_SOURCE).expect("the battle starts");
        let fed_ap: Vec<i32> = fed.host.session.as_ref().unwrap().w.units.iter().map(|u| u.ap).collect();
        assert!(!cx.log.iter().any(|l| l.starts_with("No Warded Tallow")), "with upkeep: no note");
        cx.gs_mut().party.supplies = 0;
        let starved = crate::battle::BattleScreen::world(&mut cx, &fight, trigger::FIGHT_SOURCE).expect("the battle starts");
        let s = starved.host.session.as_ref().unwrap();
        let heroes = s.w.allies.len() - s.guests.len();
        assert!(heroes > 0);
        let want: Vec<i32> = fed_ap.iter().enumerate().map(|(i, &ap)| if i < heroes { campaign::purse::start_ap(ap as i64, &mods) as i32 } else { ap }).collect();
        assert_ne!(want, fed_ap, "the hero has AP to lose");
        assert_eq!(s.w.units.iter().map(|u| u.ap).collect::<Vec<_>>(), want);
        assert_eq!(cx.log.iter().filter(|l| **l == mods.note).count(), 1, "logged once");
        assert!(starved.message.ends_with(&format!("{}.", mods.note)), "{}", starved.message);
    }
}
