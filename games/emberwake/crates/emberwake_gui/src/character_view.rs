//! UISPLIT1-H (ENGINE-21) Stage A: the Character window's view-models (`party::Character`). What the window and its
//! Stats, Level up and Skills tabs show and offer, as `screen_layout::View`s, for layout files to place later (Stage B,
//! its own row once the mock is signed off); today's hand drawing stays. The Gear tab is the loadout's view
//! (`loadout_view`, ENGINE-23). Fixtures `ui/views/{character,stats,level_up,skills,deck}.view.json`
//! (`tests::the_fixtures_have_the_view_shapes`). The agent link carries the window's view under `shell` and the open
//! tab's under `view`.
//!
//! - `character` (the window): `title`; `member` ("<Name>  Lv N", "  +P" with unspent points); `tabs` (Stats, "Level
//!   up (P)", Skills, Gear; the open one). Actions `member_prev`, `member_next` (Q and E, on with two members or
//!   more), `tab` (the tab id as arg) and `close`.
//! - `stats`: `summary` (level, XP and to next, the party's money); `wounded` ("" or "Wounded field HP a / b", `foe`);
//!   `in_party` (flag) and `party` ("In party (n/max)"); `bunks` ("" without a roster block); `core` (a row per stat,
//!   id the stat: the total as the detail, "Base b · Growth g · Spent s" as the sub, the stat's line as the note);
//!   `derived` (a row per derived stat, id its label: the value as shown, the formula text as the note); `mastery` (""
//!   or "Mastery: ..."); `innate` (the innate passives line). Action `toggle_active`.
//! - `level_up`: `points` (points left after the draft); `stats` (a row per stat, id the stat: the new total as the
//!   detail, "+N" as `detail2` toned `gold` while drafted, "Now X" as the sub, `-` and `+` buttons); `derived` (the
//!   derived rows with the draft, "old -> new" toned `gold` where it changes). Actions `remove`, `add` (a stat as arg),
//!   `auto`, `revert`, `confirm`.
//! - `skills` (classic slots): `loadout` ("Default loadout" or "Custom loadout"); `class` and `extra` (a row per slot,
//!   id "<group>.<i>": the family's name, "N AP" as the detail, its rank as `detail2` toned `gold` below the top
//!   unlocked rank, the description as the note, `<` and `>` buttons stepping the rank; "Empty", off, for a free
//!   slot); `owned` (a row per owned family, id the family, then the worn items' skills, id "worn.<skill>", off: the
//!   name with grade and rank, the description as the sub, "Slot", "Slotted" or "Full" as the detail, on while a click
//!   slots it). Actions `slot` (a family), `unslot` (a slot id), `rank_down`, `rank_up` (a slot id), `reset`,
//!   `use_default`, `clear_all`, `confirm`.
//! - `deck` (the deck rule): per kind (`active`, `passive`, `keystone`) a `<kind>_head` ("Actives n / cap"), a
//!   `<kind>_hint` ("Automatic deck" or "Custom deck" for actives, where the kind comes from for the others) and the
//!   `<kind>` rows (id "<kind>.<i>": the name, for an active "N AP" as the detail and "<Intent> <rank>" as `detail2`,
//!   the description as the note; "Empty", off, with the slot's help); `basics` (a row per basic, id the family, off);
//!   `right` (tabs Owned and "Order by intent"); `owned` (id "<kind>.<id>", then "worn.<skill>", as on the classic
//!   tab); `order` (a row per place, id "<intent>.<k>": "k. <name>", the intent's name as the sub, "N AP" as the
//!   detail, toned `info` for a basic; on from the second place, a click moves it up). Actions `deck_slot` (an owned
//!   row id), `deck_unslot` (a slot id), `right` ("owned" or "order"), `order_up` (a place id), `order_auto`, `reset`,
//!   `use_default`, `confirm`. The follow-up editor and every drag stay the drawing's own until Stage B.

use crate::party::{deck_source_hint, deck_skill, derived, party_ruleset, passive_text, pk2_rows, roman, stat_desc, tagged, Character, PAct, TABS};
use crate::screens::Cx;
use aegis_core::json::s_or;
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_data::STATS;
use emberwake_world::party_ops::{self, draft_rank, title_case, DeckTerms, DECK_KINDS};
use emberwake_world::progression;
use indexmap::IndexMap;

pub const CHARACTER: &str = "character";
pub const STATS_SCREEN: &str = "stats";
pub const LEVEL_UP: &str = "level_up";
pub const SKILLS: &str = "skills";
pub const DECK: &str = "deck";

pub const MEMBER_PREV: &str = "member_prev";
pub const MEMBER_NEXT: &str = "member_next";
pub const TAB: &str = "tab";
pub const CLOSE: &str = "close";
pub const TOGGLE_ACTIVE: &str = "toggle_active";
pub const ADD: &str = "add";
pub const REMOVE: &str = "remove";
pub const AUTO: &str = "auto";
pub const REVERT: &str = "revert";
pub const CONFIRM: &str = "confirm";
pub const SLOT: &str = "slot";
pub const UNSLOT: &str = "unslot";
pub const RANK_DOWN: &str = "rank_down";
pub const RANK_UP: &str = "rank_up";
pub const RESET: &str = "reset";
pub const USE_DEFAULT: &str = "use_default";
pub const CLEAR_ALL: &str = "clear_all";
pub const DECK_SLOT: &str = "deck_slot";
pub const DECK_UNSLOT: &str = "deck_unslot";
pub const RIGHT: &str = "right";
pub const ORDER_UP: &str = "order_up";
pub const ORDER_AUTO: &str = "order_auto";

/// The open tab's view: Stats, Level up, Skills (the deck under the deck rule) or the loadout on the Gear tab.
pub fn tab_view(cx: &Cx, c: &Character) -> View {
    match c.tab() {
        "stats" => stats(cx, c),
        "level_up" => level_up(cx, c),
        "skills" if c.skills_now(cx).0.is_some() => deck(cx, c),
        "skills" => skills(cx, c),
        _ => crate::loadout_view::view(cx, c),
    }
}

/// The window: the member picked, the tabs and the window's own actions.
pub fn character(cx: &Cx, c: &Character) -> View {
    let d = &*cx.data;
    let m = cx.gs().party.member(c.member()).expect("member");
    let unspent = m.unspent(d);
    let member = format!("{}  Lv {}{}", crate::hub::member_name(cx, m), m.level(d), if unspent > 0 { format!("  +{unspent}") } else { String::new() });
    let labels: Vec<String> = TABS.iter().map(|(id, label)| if *id == "level_up" { format!("{label} ({unspent})") } else { label.to_string() }).collect();
    let open = TABS.iter().position(|t| t.0 == c.tab()).unwrap_or(0);
    let many = cx.gs().party.members.len() > 1;
    View::new(CHARACTER)
        .text("title", "Character")
        .text("member", member)
        .field("tabs", Field::Tabs { labels, open })
        .action(MEMBER_PREV, "Previous member", many)
        .action(MEMBER_NEXT, "Next member", many)
        .action(TAB, "Tab", true)
        .action(CLOSE, "Close", true)
}

/// The Stats tab.
pub fn stats(cx: &Cx, c: &Character) -> View {
    let d = &*cx.data;
    let gs = cx.gs();
    let m = gs.party.member(c.member()).expect("member");
    let lvl = m.level(d);
    let summary = format!("Level {lvl}  ·  XP {} ({} to next)  ·  {} {}", m.total_xp, progression::xp_to_next(d, m.total_xp), crate::hud::term_short(cx, emberwake_world::terms::CURRENCY), gs.party.gold);
    let (hp, max, wounded) = crate::hub::field_hp(cx, m);
    let active = gs.party.active_members().iter().any(|a| a.id == m.id);
    let core: Vec<RowView> = match d.cat.classes.get(&m.class_id(d)) {
        Some(arch) => {
            let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
            if spend.is_empty() {
                spend.insert(STATS[0].into(), 0);
            }
            let cs = emberwake_rules::stats::CharacterStats::new(arch, lvl as i32, &spend);
            STATS
                .iter()
                .map(|st| {
                    let base = arch.base_stats.get(*st).copied().unwrap_or(0);
                    let sp = m.spend.get(*st).copied().unwrap_or(0) as i32;
                    let total = cs.base(st);
                    RowView { id: st.to_string(), label: st.to_string(), detail: total.to_string(), sub: format!("Base {base} · Growth {} · Spent {sp}", total - base - sp), note: stat_desc(st), enabled: true, ..Default::default() }
                })
                .collect()
        }
        None => Vec::new(),
    };
    let derived_rows: Vec<RowView> = derived(d, &gs.world_id, &party_ruleset(cx), m).into_iter().map(|r| RowView { id: r.label.clone(), label: r.label, detail: r.shown, note: r.desc, enabled: true, ..Default::default() }).collect();
    let pk2 = pk2_rows(d, &gs.world_id, m);
    let mastery = if pk2.is_empty() { String::new() } else { format!("Mastery: {}", pk2.iter().map(|(_, short)| short.as_str()).collect::<Vec<_>>().join(" · ")) };
    let owned: Vec<String> = progression::member_passives(d, m).into_iter().filter(|p| d.cat.passives.get(p).is_none_or(|x| x.slot.is_empty())).collect();
    let names: Vec<String> = owned.iter().map(|p| s_or(d.entry("passives", p), "name", p)).collect();
    View::new(STATS_SCREEN)
        .text("summary", summary)
        .text("wounded", if wounded { format!("Wounded field HP {hp} / {max}") } else { String::new() })
        .tone("wounded", &["foe"])
        .field("in_party", Field::Flag(active))
        .text("party", format!("In party ({}/{})", gs.party.active_members().len(), gs.party.max_active))
        .text("bunks", crate::hub::bunks_text(gs).unwrap_or_default())
        .field("core", Field::Rows(core))
        .field("derived", Field::Rows(derived_rows))
        .text("mastery", mastery)
        .text("innate", format!("Innate passives (no slot): {}", if names.is_empty() { "none".into() } else { names.join(", ") }))
        .action(TOGGLE_ACTIVE, "In party", true)
}

/// The Level up tab with its draft.
pub fn level_up(cx: &Cx, c: &Character) -> View {
    let d = &*cx.data;
    let m = cx.gs().party.member(c.member()).expect("member");
    let draft = c.level_draft();
    let drafted: i64 = draft.values().sum();
    let left = m.unspent(d) - drafted;
    let rows: Vec<RowView> = match d.cat.classes.get(&m.class_id(d)) {
        Some(arch) => {
            let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
            spend.entry(STATS[0].into()).or_insert(0);
            let cs = emberwake_rules::stats::CharacterStats::new(arch, m.level(d) as i32, &spend);
            STATS
                .iter()
                .map(|st| {
                    let now = cs.base(st) as i64;
                    let add = draft.get(*st).copied().unwrap_or(0);
                    RowView {
                        id: st.to_string(),
                        label: st.to_string(),
                        detail: (now + add).to_string(),
                        detail2: format!("+{add}"),
                        tone2: if add > 0 { "gold".into() } else { "dim".into() },
                        sub: format!("Now {now}"),
                        note: stat_desc(st),
                        enabled: true,
                        buttons: vec![RowButton { action: REMOVE.into(), label: "-".into(), enabled: add > 0 }, RowButton { action: ADD.into(), label: "+".into(), enabled: left > 0 }],
                        ..Default::default()
                    }
                })
                .collect()
        }
        None => Vec::new(),
    };
    let mut work = m.clone();
    for (k, v) in draft {
        *work.spend.entry(k.clone()).or_insert(0) += v;
    }
    let (ruleset, world) = (party_ruleset(cx), &cx.gs().world_id);
    let (before, after) = (derived(d, world, &ruleset, m), derived(d, world, &ruleset, &work));
    let derived_rows: Vec<RowView> = after
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let old = before.get(i).filter(|b| b.value != r.value);
            RowView { id: r.label.clone(), label: r.label.clone(), detail: old.map_or_else(|| r.shown.clone(), |b| format!("{} -> {}", b.shown, r.shown)), tone: if old.is_some() { "gold".into() } else { String::new() }, note: r.desc.clone(), enabled: true, ..Default::default() }
        })
        .collect();
    View::new(LEVEL_UP)
        .text("points", format!("Points left: {left}   (confirmed points are permanent)"))
        .field("stats", Field::Rows(rows))
        .field("derived", Field::Rows(derived_rows))
        .action(REMOVE, "-", true)
        .action(ADD, "+", true)
        .action(AUTO, "Auto", left > 0)
        .action(REVERT, "Revert", drafted > 0)
        .action(CONFIRM, "Confirm", drafted > 0)
}

/// The classic Skills tab with its draft.
pub fn skills(cx: &Cx, c: &Character) -> View {
    let d = &*cx.data;
    let cat = &d.cat;
    let m = cx.gs().party.member(c.member()).expect("member");
    let (_, _, draft, using_default) = c.skills_now(cx);
    let built = progression::build_with(d, m, &cx.gs().world_id, false);
    let u = built.as_ref().map(|b| &b.unit);
    let stances = d.ruleset(&cx.gs().world_id).stances();
    let fam_name = |f: &str| cat.tier_one(f).map(|s| s.name.clone()).unwrap_or_else(|| f.to_string());
    let row_of = |f: &str, rank: i32| cat.family_rows(f).into_iter().find(|s| s.tier == rank);
    let desc_of = |row: Option<&emberwake_data::SkillDef>| row.map(|s| emberwake_content::source::skill_description(&s.raw, stances)).unwrap_or_default();
    let slot_rows = |class: bool| -> Vec<RowView> {
        let g = if class { "class" } else { "extra" };
        let list = if class { &draft.class } else { &draft.extra };
        (0..party_ops::slot_cap(class))
            .map(|i| {
                let id = format!("{g}.{i}");
                let (Some(f), Some(u)) = (list.get(i), u) else {
                    return RowView { id, label: "Empty".into(), tone: "dim".into(), note: "Drag a skill here, or click a skill in the list.".into(), ..Default::default() };
                };
                let ranks = u.selectable_ranks(cat, f);
                let top = ranks.last().copied().unwrap_or(0);
                let rank = draft_rank(&ranks, draft.ranks.get(f).map(|&r| r as i32));
                let at = ranks.iter().position(|&x| x == rank).unwrap_or(0);
                let row = row_of(f, rank);
                let ap = row.map(|s| emberwake_battle::skill::ap_cost(s, Some(u))).unwrap_or(0);
                RowView {
                    id,
                    label: fam_name(f),
                    detail: format!("{ap} AP"),
                    detail2: roman(rank).into(),
                    tone2: if rank < top { "gold".into() } else { String::new() },
                    note: desc_of(row),
                    enabled: true,
                    buttons: vec![RowButton { action: RANK_DOWN.into(), label: "<".into(), enabled: at > 0 }, RowButton { action: RANK_UP.into(), label: ">".into(), enabled: at + 1 < ranks.len() }],
                    ..Default::default()
                }
            })
            .collect()
    };
    let (class_rows, extra_rows) = (slot_rows(true), slot_rows(false));
    let path = c.path_families(cx);
    let mut owned: Vec<RowView> = Vec::new();
    if let Some(u) = u {
        for f in u.owned_families(cat).into_iter().filter(|f| !cat.is_basic_family(f)) {
            let top = u.selectable_ranks(cat, &f).last().copied().unwrap_or(0);
            let slotted = draft.has(&f);
            let full = !slotted && draft.click_group(&path, &f).is_err();
            let grade = cat.family_rows(&f).first().map(|s| title_case(&s.grade)).unwrap_or_default();
            owned.push(RowView { label: format!("{}{}  Rank {}", fam_name(&f), tagged(&grade), roman(top)), sub: desc_of(row_of(&f, top)), detail: if slotted { "Slotted" } else if full { "Full" } else { "Slot" }.into(), enabled: !slotted && !full, id: f, ..Default::default() });
        }
        for id in &u.gear_skills {
            let f = cat.family_of(id).to_string();
            let top = u.selectable_ranks(cat, &f).last().copied().unwrap_or(1);
            let row = row_of(&f, top).or_else(|| cat.tier_one(&f));
            let ap = row.map(|s| emberwake_battle::skill::ap_cost(s, Some(u))).unwrap_or(0);
            owned.push(RowView { id: format!("worn.{id}"), label: format!("{}  {ap} AP  from worn gear", fam_name(&f)), sub: desc_of(row), detail: "Worn".into(), tone: "gold".into(), ..Default::default() });
        }
    }
    let any = !draft.class.is_empty() || !draft.extra.is_empty();
    View::new(SKILLS)
        .text("loadout", if using_default { "Default loadout" } else { "Custom loadout" })
        .field("class", Field::Rows(class_rows))
        .field("extra", Field::Rows(extra_rows))
        .field("owned", Field::Rows(owned))
        .action(SLOT, "Slot", true)
        .action(UNSLOT, "Take out", true)
        .action(RANK_DOWN, "<", true)
        .action(RANK_UP, ">", true)
        .action(RESET, "Reset", true)
        .action(USE_DEFAULT, "Use default", !using_default)
        .action(CLEAR_ALL, "Clear all", any)
        .action(CONFIRM, "Confirm", c.has_draft(cx))
}

/// The Skills tab under the deck rule with its draft (the classic tab's view when the member has no deck terms).
pub fn deck(cx: &Cx, c: &Character) -> View {
    let d = &*cx.data;
    let cat = &d.cat;
    let m = cx.gs().party.member(c.member()).expect("member");
    let (terms, deck_draft, _, _) = c.skills_now(cx);
    let Some(t) = terms else { return skills(cx, c) };
    let rs = d.ruleset(&cx.gs().world_id);
    let built = progression::build_with(d, m, &cx.gs().world_id, false).map(|b| b.unit);
    let shown = t.shown(&deck_draft);
    let skill_of = |f: &str| deck_skill(d, built.as_ref(), &t, &rs, f);
    let (level, key_level) = (m.level(d) as i32, progression::keystone_level(d, m));
    let mut v = View::new(DECK);
    for (kind, label, takes) in DECK_KINDS {
        let list = DeckTerms::list(&shown, kind);
        let cap = t.cap(kind);
        let hint = if kind == "active" {
            if deck_draft.is_empty() { "Automatic deck" } else { "Custom deck" }.to_string()
        } else {
            let source = if kind == "keystone" { list.first().and_then(|k| progression::keystone_source(d, m, k)) } else { None };
            deck_source_hint(kind, level, key_level, source)
        };
        let rows: Vec<RowView> = (0..cap)
            .map(|i| {
                let id = format!("{kind}.{i}");
                match list.get(i) {
                    None => {
                        let note = if kind == "keystone" && level < key_level {
                            format!("Holds one of the hero's {takes}. The class keystone opens at level {key_level} (the hero is level {level}); a Tier III graft or a mastery 3 masterwork can take the slot instead.")
                        } else if t.owned(kind).is_empty() && kind != "active" {
                            format!("Holds one of the hero's {takes}; none owned yet: they come from the hero's own deck, background, faction, disciplines and named items.")
                        } else {
                            format!("Holds one of the hero's {takes}: drag one here, or click it in the owned list.")
                        };
                        RowView { id, label: "Empty".into(), tone: "dim".into(), note, ..Default::default() }
                    }
                    Some(f) if kind == "active" => {
                        let (name, intent, ap, rank, desc) = skill_of(f);
                        RowView { id, label: name, detail: format!("{ap} AP"), detail2: format!("{intent} {}", roman(rank)), note: desc, enabled: true, ..Default::default() }
                    }
                    Some(f) => {
                        let (name, desc) = passive_text(d, f);
                        RowView { id, label: name, note: desc, enabled: true, ..Default::default() }
                    }
                }
            })
            .collect();
        v = v.text(&format!("{kind}_head"), format!("{label} {} / {cap}", list.len())).text(&format!("{kind}_hint"), hint).field(kind, Field::Rows(rows));
    }
    let basics: Vec<RowView> = t
        .basics
        .iter()
        .map(|f| {
            let (name, intent, ap, _, desc) = skill_of(f);
            let line = if t.intent_of.get(f).is_some_and(|i| t.intents.contains(i)) { format!("{intent} basic, {ap} AP") } else { format!("Basic for any intent, {ap} AP") };
            RowView { id: f.clone(), label: name, detail: format!("{ap} AP"), sub: line, note: desc, ..Default::default() }
        })
        .collect();
    let mut owned: Vec<RowView> = Vec::new();
    for (kind, _, _) in DECK_KINDS {
        for id in t.owned(kind) {
            let slotted = DeckTerms::list(&shown, kind).contains(id);
            let full = !slotted && DeckTerms::list(&shown, kind).len() >= t.cap(kind);
            let (line, desc) = if kind == "active" {
                let (name, intent, ap, rank, desc) = skill_of(id);
                (format!("{name}{}  Rank {}  {ap} AP", tagged(&intent), roman(rank)), desc)
            } else {
                let (name, desc) = passive_text(d, id);
                (format!("{name}  [{}]", title_case(kind)), desc)
            };
            owned.push(RowView { id: format!("{kind}.{id}"), label: line, sub: desc, detail: if slotted { "Slotted" } else if full { "Full" } else { "Slot" }.into(), enabled: !slotted && !full, ..Default::default() });
        }
    }
    for id in built.as_ref().map(|u| u.gear_skills.clone()).unwrap_or_default() {
        let (name, intent, ap, _, desc) = skill_of(cat.family_of(&id));
        owned.push(RowView { id: format!("worn.{id}"), label: format!("{name}{}  {ap} AP  from worn gear", tagged(&intent)), sub: desc, detail: "Worn".into(), tone: "gold".into(), ..Default::default() });
    }
    let mut order: Vec<RowView> = Vec::new();
    for intent in &t.intents {
        let iname = crate::party::intent_label(d, intent);
        for (k, f) in t.order(&shown, intent).iter().enumerate() {
            let (fname, _, ap, _, desc) = skill_of(f);
            let basic = t.basics.contains(f);
            order.push(RowView { id: format!("{intent}.{k}"), label: format!("{}. {fname}", k + 1), sub: iname.clone(), detail: format!("{ap} AP"), tone: if basic { "info".into() } else { String::new() }, note: desc, enabled: k > 0, ..Default::default() });
        }
    }
    let right = if c.deck_view() == "order" { 1 } else { 0 };
    v.field("basics", Field::Rows(basics))
        .field(RIGHT, Field::Tabs { labels: vec!["Owned".into(), "Order by intent".into()], open: right })
        .field("owned", Field::Rows(owned))
        .field("order", Field::Rows(order))
        .action(DECK_SLOT, "Slot", true)
        .action(DECK_UNSLOT, "Take out", true)
        .action(RIGHT, "Show", true)
        .action(ORDER_UP, "Move up", true)
        .action(ORDER_AUTO, "Auto order", !shown.order.is_empty())
        .action(RESET, "Reset", true)
        .action(USE_DEFAULT, "Use default", !deck_draft.is_empty())
        .action(CONFIRM, "Confirm", c.has_draft(cx))
}

/// A view click as the window's own (`PAct`), read against the open tab and its draft; None for an unknown action, a
/// row that is not there or one a click does nothing on. Stage B's layout draw hands its clicks here; until then only
/// the tests call it. The Gear tab's clicks are `loadout_view::loadout_act`'s.
#[cfg_attr(not(test), allow(dead_code))]
pub fn character_act(cx: &Cx, c: &Character, a: &Act) -> Option<PAct> {
    let arg = a.arg.clone().filter(|x| !x.is_empty());
    let stat = || arg.as_deref().and_then(|s| STATS.iter().copied().find(|x| *x == s));
    // "<group>.<i>" -> (group, i)
    let split = || arg.as_deref().and_then(|s| s.rsplit_once('.')).and_then(|(g, i)| Some((g.to_string(), i.parse::<usize>().ok()?)));
    let many = cx.gs().party.members.len() > 1;
    match (c.tab(), a.action.as_str()) {
        (_, MEMBER_PREV) => many.then_some(PAct::Member(-1)),
        (_, MEMBER_NEXT) => many.then_some(PAct::Member(1)),
        (_, TAB) => arg.and_then(|t| TABS.iter().find(|x| x.0 == t)).map(|x| PAct::Tab(x.0)),
        (_, CLOSE) => Some(PAct::Close),
        ("stats", TOGGLE_ACTIVE) => Some(PAct::ToggleActive),
        ("level_up", _) => level_act(cx, c, a.action.as_str(), stat()),
        ("skills", _) => match c.skills_now(cx) {
            (Some(t), deck, _, _) => {
                let shown = t.shown(&deck);
                match a.action.as_str() {
                    DECK_SLOT => {
                        let (kind, id) = arg.as_deref()?.split_once('.')?;
                        let kind = DECK_KINDS.iter().find(|k| k.0 == kind)?.0;
                        let list = DeckTerms::list(&shown, kind);
                        (t.owned(kind).iter().any(|x| x == id) && !list.iter().any(|x| x == id) && list.len() < t.cap(kind)).then(|| PAct::DeckSlot(kind, id.to_string()))
                    }
                    DECK_UNSLOT => {
                        let (kind, i) = split()?;
                        let kind = DECK_KINDS.iter().find(|k| k.0 == kind)?.0;
                        (i < DeckTerms::list(&shown, kind).len()).then_some(PAct::DeckUnslot(kind, i))
                    }
                    RIGHT => match arg.as_deref() {
                        Some("owned") => Some(PAct::DeckView("owned")),
                        Some("order") => Some(PAct::DeckView("order")),
                        _ => None,
                    },
                    ORDER_UP => {
                        let (intent, k) = split()?;
                        (t.intents.contains(&intent) && k > 0 && k < t.order(&shown, &intent).len()).then_some(PAct::OrderUp(intent, k))
                    }
                    ORDER_AUTO => (!shown.order.is_empty()).then_some(PAct::OrderAuto),
                    RESET => Some(PAct::ResetSkills),
                    USE_DEFAULT => (!deck.is_empty()).then_some(PAct::UseDefault),
                    CONFIRM => c.has_draft(cx).then_some(PAct::ConfirmSkills),
                    _ => None,
                }
            }
            (None, _, draft, using_default) => {
                let list = |g: &str| if g == "class" { &draft.class } else { &draft.extra };
                let slot = || split().filter(|(g, i)| (g == "class" || g == "extra") && *i < list(g).len());
                match a.action.as_str() {
                    SLOT => {
                        let f = arg?;
                        (!draft.has(&f) && draft.click_group(&c.path_families(cx), &f).is_ok()).then_some(PAct::Slot(f))
                    }
                    UNSLOT => slot().map(|(g, i)| PAct::Unslot(g == "class", i)),
                    RANK_DOWN | RANK_UP => {
                        let row = skills(cx, c);
                        let (g, i) = slot()?;
                        let Some(Field::Rows(rows)) = row.get(&g) else { return None };
                        let at = if a.action == RANK_DOWN { 0 } else { 1 };
                        rows.get(i)?.buttons.get(at).filter(|b| b.enabled).map(|_| PAct::RankStep(g == "class", i, if at == 0 { -1 } else { 1 }))
                    }
                    RESET => Some(PAct::ResetSkills),
                    USE_DEFAULT => (!using_default).then_some(PAct::UseDefault),
                    CLEAR_ALL => (!draft.class.is_empty() || !draft.extra.is_empty()).then_some(PAct::ClearAll),
                    CONFIRM => c.has_draft(cx).then_some(PAct::ConfirmSkills),
                    _ => None,
                }
            }
        },
        _ => None,
    }
}

/// The Level up tab's clicks: `-` and `+` on a stat while the draft or the points allow it, Auto, Revert, Confirm.
fn level_act(cx: &Cx, c: &Character, action: &str, stat: Option<&'static str>) -> Option<PAct> {
    let m = cx.gs().party.member(c.member())?;
    let drafted: i64 = c.level_draft().values().sum();
    let left = m.unspent(&cx.data) - drafted;
    match action {
        ADD => stat.filter(|_| left > 0).map(PAct::Add),
        REMOVE => stat.filter(|s| c.level_draft().get(*s).copied().unwrap_or(0) > 0).map(PAct::Remove),
        AUTO => (left > 0).then_some(PAct::Auto),
        REVERT => (drafted > 0).then_some(PAct::Revert),
        CONFIRM => (drafted > 0).then_some(PAct::ConfirmLevel),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rows_of<'a>(v: &'a View, name: &str) -> &'a [RowView] {
        match v.get(name) {
            Some(Field::Rows(r)) => r,
            _ => panic!("no rows {name}"),
        }
    }

    fn text<'a>(v: &'a View, name: &str) -> &'a str {
        match v.get(name) {
            Some(Field::Text(t)) => t,
            _ => panic!("no text {name}"),
        }
    }

    fn names(v: &View) -> Vec<&str> {
        v.fields.iter().map(|(n, _)| n.as_str()).collect()
    }

    fn click(a: &str, arg: &str) -> Act {
        Act { action: a.into(), arg: (!arg.is_empty()).then(|| arg.to_string()) }
    }

    /// The test bed world (`party::tests::deck_cx`): classic, or under the deck rule (its own copy of the manifest takes
    /// the timeline row); the first member's window open on `tab`.
    fn cx_on(deck: bool, tab: &str) -> (Cx, Character) {
        const WORLD: &str = "ashlands";
        let mut cx = crate::hud::hud_tests::game_cx();
        if deck {
            let manifest = &mut std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data").catalogs.worlds.get_mut(WORLD).expect("the world").manifest;
            manifest.insert("ruleset".into(), json!(emberwake_rules::ruleset::TIMELINE));
        }
        cx.gs_mut().world_id = WORLD.into();
        let member = cx.gs().party.members[0].id.clone();
        let c = Character::new(&cx, &member, tab);
        (cx, c)
    }

    /// The window: its title, the member line, the four tabs with the open one; Q and E, a tab by id and Close map to the
    /// window's own clicks; an unknown tab maps to nothing.
    #[test]
    fn the_window_view_is_the_member_and_the_tabs() {
        let (cx, c) = cx_on(false, "level_up");
        let v = character(&cx, &c);
        assert_eq!(names(&v), ["title", "member", "tabs"]);
        let m = cx.gs().party.member(c.member()).unwrap();
        assert!(text(&v, "member").starts_with(&format!("{}  Lv {}", crate::hub::member_name(&cx, m), m.level(&cx.data))));
        match v.get("tabs") {
            Some(Field::Tabs { labels, open }) => {
                assert_eq!(labels.len(), TABS.len());
                assert_eq!(*open, 1, "Level up is open");
                assert!(labels[1].starts_with("Level up ("), "{labels:?}");
            }
            _ => panic!("no tabs"),
        }
        let many = cx.gs().party.members.len() > 1;
        assert_eq!(matches!(character_act(&cx, &c, &click(MEMBER_NEXT, "")), Some(PAct::Member(1))), many);
        assert_eq!(matches!(character_act(&cx, &c, &click(MEMBER_PREV, "")), Some(PAct::Member(-1))), many);
        assert!(matches!(character_act(&cx, &c, &click(TAB, "gear")), Some(PAct::Tab("gear"))));
        assert!(character_act(&cx, &c, &click(TAB, "nope")).is_none());
        assert!(matches!(character_act(&cx, &c, &click(CLOSE, "")), Some(PAct::Close)));
    }

    /// The Stats tab: a row per stat with its total and parts, the derived rows the tab draws, the party line; the In
    /// party toggle maps on this tab only.
    #[test]
    fn the_stats_view_is_the_sheet() {
        let (cx, c) = cx_on(false, "stats");
        let v = tab_view(&cx, &c);
        assert_eq!(v.screen, STATS_SCREEN);
        assert_eq!(names(&v), ["summary", "wounded", "in_party", "party", "bunks", "core", "derived", "mastery", "innate"]);
        let core = rows_of(&v, "core");
        assert_eq!(core.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), STATS.to_vec());
        assert!(core.iter().all(|r| r.detail.parse::<i32>().is_ok() && r.sub.starts_with("Base ") && !r.note.is_empty()), "{core:?}");
        let m = cx.gs().party.member(c.member()).unwrap();
        let want: Vec<(String, String)> = derived(&cx.data, &cx.gs().world_id, &party_ruleset(&cx), m).into_iter().map(|r| (r.label, r.shown)).collect();
        assert_eq!(rows_of(&v, "derived").iter().map(|r| (r.label.clone(), r.detail.clone())).collect::<Vec<_>>(), want);
        assert!(text(&v, "summary").starts_with(&format!("Level {}", m.level(&cx.data))));
        assert!(matches!(character_act(&cx, &c, &click(TOGGLE_ACTIVE, "")), Some(PAct::ToggleActive)));
        let (cx2, c2) = cx_on(false, "level_up");
        assert!(character_act(&cx2, &c2, &click(TOGGLE_ACTIVE, "")).is_none(), "not on another tab");
    }

    /// The Level up tab: with points to spend `+` maps and the draft shows on the row (+1 in gold, the new total) and in
    /// the points line; `-` maps only on a drafted stat; Revert and Confirm turn on with a draft.
    #[test]
    fn the_level_up_view_follows_the_draft() {
        let (mut cx, mut c) = cx_on(false, "level_up");
        let id = c.member().to_string();
        cx.gs_mut().party.member_mut(&id).unwrap().total_xp += 100_000;
        let unspent = cx.gs().party.member(&id).unwrap().unspent(&cx.data);
        assert!(unspent > 0, "points to spend");
        let st = STATS[0];
        let v = tab_view(&cx, &c);
        assert_eq!(names(&v), ["points", "stats", "derived"]);
        assert!(!v.act(CONFIRM).unwrap().enabled && !v.act(REVERT).unwrap().enabled && v.act(AUTO).unwrap().enabled);
        assert!(character_act(&cx, &c, &click(REMOVE, st)).is_none(), "nothing drafted");
        let Some(a) = character_act(&cx, &c, &click(ADD, st)) else { panic!("+ maps") };
        assert!(matches!(a, PAct::Add(s) if s == st));
        c.act(&mut cx, a);
        let v = tab_view(&cx, &c);
        let row = &rows_of(&v, "stats")[0];
        let now: i64 = row.sub.trim_start_matches("Now ").parse().unwrap();
        assert_eq!((row.detail.clone(), row.detail2.as_str(), row.tone2.as_str()), ((now + 1).to_string(), "+1", "gold"));
        assert_eq!(text(&v, "points"), format!("Points left: {}   (confirmed points are permanent)", unspent - 1));
        assert!(v.act(CONFIRM).unwrap().enabled && v.act(REVERT).unwrap().enabled);
        assert!(matches!(character_act(&cx, &c, &click(REMOVE, st)), Some(PAct::Remove(s)) if s == st));
        assert!(matches!(character_act(&cx, &c, &click(CONFIRM, "")), Some(PAct::ConfirmLevel)));
        assert!(character_act(&cx, &c, &click(ADD, "nope")).is_none());
    }

    /// The classic Skills tab: a row per class and extra slot, the owned families with their button word; slot maps for a
    /// family a click slots, unslot for a filled slot (the view then shows the slot empty), the rank buttons as the
    /// drawing has them on.
    #[test]
    fn the_classic_skills_view_is_the_slots_and_the_owned_list() {
        let (mut cx, mut c) = cx_on(false, "skills");
        let v = tab_view(&cx, &c);
        assert_eq!(v.screen, SKILLS);
        assert_eq!(names(&v), ["loadout", "class", "extra", "owned"]);
        assert_eq!(rows_of(&v, "class").len(), party_ops::slot_cap(true));
        assert_eq!(rows_of(&v, "extra").len(), party_ops::slot_cap(false));
        let filled: Vec<RowView> = rows_of(&v, "class").iter().filter(|r| r.enabled).cloned().collect();
        assert!(!filled.is_empty(), "the default loadout fills class slots");
        assert!(filled.iter().all(|r| r.detail.ends_with(" AP") && r.buttons.len() == 2), "{filled:?}");
        let owned = rows_of(&v, "owned");
        assert!(owned.iter().any(|r| r.detail == "Slotted" && !r.enabled), "{owned:?}");
        for r in owned.iter().filter(|r| !r.id.starts_with("worn.")) {
            assert_eq!(character_act(&cx, &c, &click(SLOT, &r.id)).is_some(), r.enabled, "{r:?}");
        }
        let first = filled[0].id.clone();
        let Some(a) = character_act(&cx, &c, &click(UNSLOT, &first)) else { panic!("unslot maps") };
        assert!(matches!(a, PAct::Unslot(true, 0)), "{a:?}");
        for (action, at) in [(RANK_DOWN, 0), (RANK_UP, 1)] {
            assert_eq!(character_act(&cx, &c, &click(action, &first)).is_some(), filled[0].buttons[at].enabled);
        }
        c.act(&mut cx, a);
        let v = tab_view(&cx, &c);
        assert!(v.act(CONFIRM).unwrap().enabled, "a draft is open");
        assert!(character_act(&cx, &c, &click(UNSLOT, "class.99")).is_none() && character_act(&cx, &c, &click(UNSLOT, "nope.0")).is_none());
        assert!(matches!(character_act(&cx, &c, &click(RESET, "")), Some(PAct::ResetSkills)));
    }

    /// The Skills tab under the deck rule: the three buckets with their heads and hints, the basics, the owned list and
    /// the order; slotting an owned active, taking a slot out, the right column and a move up map to the tab's clicks.
    #[test]
    fn the_deck_view_is_the_buckets_the_owned_list_and_the_order() {
        let (cx, c) = cx_on(true, "skills");
        let v = tab_view(&cx, &c);
        assert_eq!(v.screen, DECK);
        assert_eq!(names(&v), ["active_head", "active_hint", "active", "passive_head", "passive_hint", "passive", "keystone_head", "keystone_hint", "keystone", "basics", RIGHT, "owned", "order"]);
        let (t, deck, _, _) = c.skills_now(&cx);
        let t = t.expect("deck terms");
        let shown = t.shown(&deck);
        assert_eq!(rows_of(&v, "active").len(), t.cap("active"));
        assert_eq!(text(&v, "active_head"), format!("Actives {} / {}", DeckTerms::list(&shown, "active").len(), t.cap("active")));
        assert_eq!(rows_of(&v, "basics").len(), t.basics.len());
        let order = rows_of(&v, "order");
        assert_eq!(order.len(), t.intents.iter().map(|i| t.order(&shown, i).len()).sum::<usize>());
        let up = order.iter().find(|r| r.enabled).expect("a second place");
        let (intent, k) = up.id.rsplit_once('.').unwrap();
        assert!(matches!(character_act(&cx, &c, &click(ORDER_UP, &up.id)), Some(PAct::OrderUp(i, n)) if i == intent && n.to_string() == k));
        assert!(character_act(&cx, &c, &click(ORDER_UP, &format!("{intent}.0"))).is_none(), "the first place does not move up");
        for r in rows_of(&v, "owned").iter().filter(|r| !r.id.starts_with("worn.")) {
            assert_eq!(character_act(&cx, &c, &click(DECK_SLOT, &r.id)).is_some(), r.enabled, "{r:?}");
        }
        let filled = rows_of(&v, "active").iter().find(|r| r.enabled).expect("an active slotted").id.clone();
        assert!(matches!(character_act(&cx, &c, &click(DECK_UNSLOT, &filled)), Some(PAct::DeckUnslot("active", 0))));
        assert!(matches!(character_act(&cx, &c, &click(RIGHT, "order")), Some(PAct::DeckView("order"))));
        assert!(character_act(&cx, &c, &click(RIGHT, "nope")).is_none() && character_act(&cx, &c, &click(SLOT, "x")).is_none());
    }

    /// The agent link carries the window's view under `shell` and the open tab's under `view`.
    #[test]
    fn the_state_carries_the_window_and_the_tab_views() {
        use crate::screens::Screen;
        for (tab, screen) in [("stats", STATS_SCREEN), ("level_up", LEVEL_UP), ("skills", SKILLS), ("gear", crate::loadout_view::SCREEN)] {
            let (cx, c) = cx_on(false, tab);
            let st = c.state(&cx);
            assert_eq!(st["shell"]["screen"], json!(CHARACTER), "{tab}");
            assert_eq!(st["view"]["screen"], json!(screen), "{tab}");
        }
    }

    fn views(deck: bool) -> Vec<View> {
        let mut out = Vec::new();
        for tab in ["stats", "level_up", "skills"] {
            let (cx, c) = cx_on(deck, tab);
            if tab == "stats" {
                out.push(character(&cx, &c));
            }
            if !deck || tab == "skills" {
                out.push(tab_view(&cx, &c));
            }
        }
        out
    }

    /// US6: each fixture has its view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixtures_have_the_view_shapes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        let all: Vec<View> = views(false).into_iter().chain(views(true).into_iter().filter(|v| v.screen == DECK)).collect();
        assert_eq!(all.iter().map(|v| v.screen.as_str()).collect::<Vec<_>>(), [CHARACTER, STATS_SCREEN, LEVEL_UP, SKILLS, DECK]);
        for v in &all {
            let fixture = aegis_ui::screen_layout::load_fixture(&layers, &v.screen).expect("the fixture file").expect("it parses");
            assert_eq!(shape(&fixture), shape(v), "{}", v.screen);
        }
    }

    /// Writes the fixtures from the test bed's first member (run by hand: `cargo test -p emberwake_gui
    /// write_character_fixtures -- --ignored`), so they hold the sample's values.
    #[test]
    #[ignore]
    fn write_character_fixtures() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views");
        for v in views(false).into_iter().chain(views(true).into_iter().filter(|v| v.screen == DECK)) {
            std::fs::write(dir.join(format!("{}.view.json", v.screen)), serde_json::to_string_pretty(&v.to_json()).unwrap() + "\n").unwrap();
        }
    }
}
