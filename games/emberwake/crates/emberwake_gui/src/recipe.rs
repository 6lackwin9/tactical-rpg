//! The recipe pane (PK2-S3, `docs/sessions/PK2_GRAFTS.md` section 4; engine-only, no Godot original; story-free): the
//! NPC service `recipe` over the dimmed backdrop in the shop's frame: the character's name, the party member (and the
//! stash item) the recipes work on, then each offered recipe with its cost, why it can't run now, and Do; Close at the
//! bottom. The rules are the kit's (`recipe::view`, `recipe::run`, the runner's `close_recipes`); the pane only draws
//! them. `talk::Services` draws it while its conversation has the recipe pane open. PK2-FOLLOWUP (PR5-36): a weapon
//! picker (`recipe.weapon`: the party's weapon instances, for `repair` and `reforge`) and a family picker
//! (`recipe.family`: the families a row lets the pick name, such as a minor family for `train_proficiency`; each row
//! reads it through `recipe::pick_for`) show while an offered recipe needs them.

use crate::screens::Cx;
use aegis_core::json::{self, s};
use aegis_ui::theme::th;
use aegis_ui::canvas::Rect;
use aegis_ui::ui::{tip, Ui, BODY, SMALL};
use emberwake_data::recipe::{TARGET_ITEM, TARGET_MEMBER, TARGET_WEAPON};
use emberwake_world::recipe::{self, Pick};
use emberwake_world::{weapons, Data, GameState};
use serde_json::{json, Value};

/// The frame's inner margin (the shop's) and the name's size.
const PAD: i32 = 7;
const HEAD: f32 = 10.0;

/// A click on the recipe pane.
#[derive(Debug, Clone)]
pub enum RecipeAct {
    /// Picks the party member (roster order) the member recipes work on.
    Member(usize),
    /// Picks the stash item (the listed order) the item recipes break down.
    Item(usize),
    /// PK2-FOLLOWUP: picks the weapon instance (the listed order) the weapon recipes work on.
    Weapon(usize),
    /// PK2-FOLLOWUP: picks the family (the listed order) a recipe that offers several trains or drills.
    Family(usize),
    /// Runs a recipe.
    Run(String),
    /// Back to the conversation.
    Close,
}

/// The pane's own state: the picked member, item, weapon and family, the last run's notes and refusal.
#[derive(Debug, Default)]
pub struct RecipePane {
    pub member: usize,
    pub item: usize,
    pub weapon: usize,
    pub family: usize,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

/// The `index`th of `list`, clamped to the last ("" for an empty list).
fn clamped(list: &[String], index: usize) -> String {
    list.get(index.min(list.len().saturating_sub(1))).cloned().unwrap_or_default()
}

/// PK2-FOLLOWUP: a weapon instance's pick label: its item, its durability of its full (when it wears), a masterwork
/// mark and who wields it (else the stash), read on a synced copy as `recipe::targets` lists them.
fn weapon_labels(d: &Data, gs: &GameState, uids: &[String]) -> Vec<String> {
    let mut work = gs.clone();
    weapons::sync(d, &mut work);
    uids.iter()
        .map(|uid| {
            let Some(w) = weapons::find(&work.party, uid) else { return uid.clone() };
            let full = weapons::full(d, &w.item);
            let wear = if full > 0 { format!(" {}/{full}", w.durability) } else { String::new() };
            let holder = work.party.members.iter().find(|m| m.weapons.values().any(|x| x.uid == *uid)).map_or_else(|| "stash".to_string(), |m| emberwake_world::progression::display_name(d, m));
            format!("{}{wear}{} ({holder})", d.item_name(&w.item), if w.masterwork { " MW" } else { "" })
        })
        .collect()
}

impl RecipePane {
    /// The agent link's reading.
    pub fn state(&self) -> Value {
        json!({"member": self.member, "item": self.item, "weapon": self.weapon, "family": self.family, "notes": self.notes, "errors": self.errors})
    }

    /// The targets of the first recipe of `view` working on `target` (the stash items an item recipe breaks down, the
    /// weapon instances a weapon recipe works on).
    fn targets(view: &[Value], target: &str) -> Vec<String> {
        view.iter().map(json::dict).find(|r| s(r, "target") == target).map(|r| json::sget(r, "targets")).unwrap_or_default()
    }

    /// The stash items an item recipe of `view` may break down (the first such recipe's targets).
    fn items(view: &[Value]) -> Vec<String> {
        Self::targets(view, TARGET_ITEM)
    }

    /// PK2-FOLLOWUP: the families the rows of `view` let the pick name, in order and each once.
    fn families(view: &[Value]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for f in view.iter().flat_map(|r| json::sget(json::dict(r), "families")) {
            if !out.contains(&f) {
                out.push(f);
            }
        }
        out
    }

    /// The pick the recipes run on now (each row reads it through `recipe::pick_for`).
    pub fn pick(&self, cx: &Cx, npc: &str) -> Pick {
        let gs = cx.gs();
        let first = Pick { member: gs.party.members.get(self.member).map(|m| m.id.clone()).unwrap_or_default(), ..Pick::default() };
        let view = recipe::view(&cx.data, gs, npc, &first);
        Pick { item: clamped(&Self::items(&view), self.item), weapon: clamped(&Self::targets(&view, TARGET_WEAPON), self.weapon), family: clamped(&Self::families(&view), self.family), ..first }
    }

    /// After a run: the item pick follows its item to its new place in the list, or goes back to the first item once
    /// the run used the last of it (never a silent jump to a neighbour).
    fn follow_item(&mut self, cx: &Cx, npc: &str, before: &str) {
        let pick = Pick { item: String::new(), ..self.pick(cx, npc) };
        let items = Self::items(&recipe::view(&cx.data, cx.gs(), npc, &pick));
        self.item = items.iter().position(|id| id == before).unwrap_or(0);
    }

    /// One click: a pick, or a run (its notes logged and kept; a refusal kept). True for Close.
    pub fn act(&mut self, cx: &mut Cx, npc: &str, a: RecipeAct) -> bool {
        match a {
            RecipeAct::Member(i) => self.member = i,
            RecipeAct::Item(i) => self.item = i,
            RecipeAct::Weapon(i) => self.weapon = i,
            RecipeAct::Family(i) => self.family = i,
            RecipeAct::Run(id) => {
                let pick = self.pick(cx, npc);
                let d = cx.data.clone();
                let row_pick = d.cat.recipes.get(&id).map_or_else(|| pick.clone(), |rc| recipe::pick_for(&d, cx.gs(), rc, &pick));
                let res = recipe::run(&d, cx.gs_mut(), npc, &id, &row_pick);
                self.follow_item(cx, npc, &pick.item);
                self.errors = json::sget(&res, "errors");
                self.notes = json::sget(&res, "notes");
                for n in &self.notes {
                    cx.log(n.clone());
                }
            }
            RecipeAct::Close => return true,
        }
        false
    }

    /// Draws character `npc`'s recipes in the shop's frame.
    pub fn draw<A: Clone>(&mut self, ui: &mut Ui<A>, cx: &Cx, npc: &str, wrap: impl Fn(RecipeAct) -> A) {
        let gs = cx.gs();
        let d = &cx.data;
        let fr = crate::shop::frame_rect(&ui.p);
        let name = json::s_or(d.resolve(&gs.world_id, "characters", npc), "name", npc);
        ui.p.panel(fr, th().panel, th().panel_edge, 4.0);
        ui.push("recipe", "panel", name.clone(), fr, true, false, None, Vec::new());
        let (lb, ls) = (ui.p.line_h(BODY).ceil() as i32, ui.p.line_h(SMALL).ceil() as i32);
        let (x0, right, inner) = (fr.x + PAD, fr.x + fr.w - PAD, fr.w - 2 * PAD);
        ui.p.text_fit(x0 as f32, fr.y as f32 + 5.0, &name, HEAD, inner as f32, th().gold);
        let mut y = fr.y + 5 + ui.p.line_h(HEAD).ceil() as i32 + 4;
        let pick = self.pick(cx, npc);
        let view = recipe::view(d, gs, npc, &pick);
        // -- the picks: the member (member recipes), the family (a row offering several), the stash item (item recipes)
        // and the weapon instance (weapon recipes); a pick that does not fit the line starts the next
        let bh = 17.max(lb + 6);
        let names: Vec<String> = gs.party.members.iter().map(|m| crate::hub::member_name(cx, m)).collect();
        let items: Vec<String> = Self::items(&view).iter().map(|id| d.item_name(id)).collect();
        let families = Self::families(&view);
        let arms = weapon_labels(d, gs, &Self::targets(&view, TARGET_WEAPON));
        let mut picks: Vec<(&str, &str, &[String], usize, i32, RecipeAct)> = Vec::new();
        if view.iter().any(|r| s(json::dict(r), "target") == TARGET_MEMBER) {
            picks.push(("recipe.member", "For", &names, self.member, 130, RecipeAct::Member(0)));
        }
        if !families.is_empty() {
            picks.push(("recipe.family", "Family", &families, self.family, 90, RecipeAct::Family(0)));
        }
        if !items.is_empty() {
            picks.push(("recipe.item", "Item", &items, self.item, 150, RecipeAct::Item(0)));
        }
        if !arms.is_empty() {
            picks.push(("recipe.weapon", "Weapon", &arms, self.weapon, 190, RecipeAct::Weapon(0)));
        }
        let mut x = x0;
        for (id, label, list, at, w, kind) in picks {
            let need = ui.p.text_w(label, BODY).ceil() as i32 + 6 + w;
            if x > x0 && x + need > right {
                (x, y) = (x0, y + bh + 2);
            }
            ui.p.text(x as f32, (y + 3) as f32, label, BODY, th().dim);
            x += ui.p.text_w(label, BODY).ceil() as i32 + 6;
            let act = move |i: usize| match kind {
                RecipeAct::Member(_) => RecipeAct::Member(i),
                RecipeAct::Family(_) => RecipeAct::Family(i),
                RecipeAct::Item(_) => RecipeAct::Item(i),
                _ => RecipeAct::Weapon(i),
            };
            ui.select(id, Rect::new(x, y, w, bh - 2), list, at.min(list.len().saturating_sub(1)), |i| wrap(act(i)), list.len() > 1, Vec::new());
            x += w + 10;
        }
        y += bh + 4;
        // -- the recipes, each its name, its cost and why it can't run now, Do at the right
        let bar_y = fr.y + fr.h - PAD - bh;
        let do_w = (ui.p.text_w("Do", BODY).ceil() as i32 + 16).max(44);
        for row in view.iter().map(json::dict) {
            let reason = s(row, "reason");
            let h = lb + ls + if reason.is_empty() { 0 } else { ls } + 4;
            if y + h > bar_y - lb - 4 {
                break;
            }
            let ready = json::b(row, "ready", false);
            let id = s(row, "id");
            ui.p.text_fit(x0 as f32, y as f32, &s(row, "name"), BODY, (inner - do_w - 8) as f32, if ready { th().text } else { th().dim });
            let cost = json::sget(row, "cost").join(", ");
            ui.p.text_fit(x0 as f32 + 8.0, (y + lb) as f32, if cost.is_empty() { "Free" } else { &cost }, SMALL, (inner - do_w - 16) as f32, th().dim);
            if !reason.is_empty() {
                ui.p.text_fit(x0 as f32 + 8.0, (y + lb + ls) as f32, &reason, SMALL, (inner - do_w - 16) as f32, th().foe);
            }
            let body = if ready { s(row, "description") } else { reason.clone() };
            ui.button_full(&format!("recipe.do.{id}"), Rect::new(right - do_w, y, do_w, bh), "Do", wrap(RecipeAct::Run(id.clone())), ready, false, BODY, tip(&s(row, "name"), &body));
            y += h;
        }
        if view.is_empty() {
            ui.p.text(x0 as f32, y as f32, "Nothing is offered now.", BODY, th().dim);
        }
        // -- the last run's refusal or notes, and Close
        let line = if self.errors.is_empty() { self.notes.join(" ") } else { self.errors.join(" ") };
        if !line.is_empty() {
            let r = Rect::new(x0, bar_y - lb - 3, inner, lb);
            ui.p.text_fit(r.x as f32, r.y as f32, &line, BODY, r.w as f32, if self.errors.is_empty() { th().gold } else { th().foe });
            ui.push("recipe.line", "label", line.clone(), r, true, false, None, Vec::new());
        }
        let close_w = (ui.p.text_w("Close", BODY).ceil() as i32 + 16).max(44);
        ui.button_full("recipe.close", Rect::new(x0, bar_y, close_w, bh), "Close", wrap(RecipeAct::Close), true, false, BODY, tip("Close", "Back to the conversation (Esc)."));
    }
}

/// PK2-S3 on a scratch world `ward` (no shipped data): Smith offers a proficiency training; the conversation's recipe
/// reply opens the pane in the dialogue box's place, Do runs the recipe (the cost paid, the member trained, the line
/// says so) and turns off once it can't run again (its reason shows), Back returns to the conversation.
#[cfg(test)]
mod tests {
    use crate::hud::hud_tests::{frame, gfx};
    use crate::screens::{enter_game, GameCx, Screen};
    use crate::story::tests::story_cx;
    use crate::recipe::RecipeAct;
    use crate::talk::{ServiceScreen, SvcAct};
    use aegis_core::json::{self, s};
    use emberwake_world::{explore, new_game, weapons, Data};
    use serde_json::{json, Value};

    const FUNDS: i64 = 50;
    const PRICE: i64 = 5;
    const SALVAGE: i64 = 2;

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

    fn data() -> Data {
        let catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter", "proficiency": {"sword": {"level": 3, "role": "main"}, "bow": {"level": 2, "role": "secondary"}}}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
            "recipes": [{"id": "train_bow", "name": "Bow Training", "npc": "smith", "requires": [], "cost": {"gold": PRICE}, "target": "member",
                "effects": [{"effect": "train_proficiency", "role": "secondary", "to": 3}]},
                {"id": "break_down", "name": "Break Down", "npc": "smith", "requires": [], "cost": {}, "target": "item", "effects": [{"effect": "salvage", "gold": SALVAGE}]}],
            "items": [{"id": "rusty", "name": "Rusty", "item_type": "MATERIAL"}, {"id": "spare", "name": "Spare", "item_type": "MATERIAL"}]
        });
        let worlds = json!({"ward": {"manifest": {"id": "ward", "name": "Ward", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids("ward", json!([{"id": "hub", "name": "Hub"}])),
            "characters": with_ids("ward", json!([{"id": "hero", "name": "Hero", "role": "party", "base_unit": "hero"}, {"id": "smith", "name": "Smith", "role": "npc"}]))
        }}});
        Data::from_values(&catalog, &worlds, &json!({}))
    }

    fn widget(s: &dyn Screen<GameCx>, id: &str) -> Option<Value> {
        s.widgets().into_iter().find(|w| w["id"] == id)
    }

    #[test]
    fn the_recipe_reply_opens_the_pane_and_do_runs_a_recipe() {
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut gs = new_game::create_for_world(&cx.data.clone(), "ward", 1).expect("a new game");
        gs.party.gold = FUNDS;
        gs.party.add_to_stash("rusty", 1);
        gs.party.add_to_stash("spare", 1);
        enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        assert!(explore::npc_services(&cx.data, cx.gs(), "ward:smith").contains(&"recipe"), "the service badge");
        let mut host = ServiceScreen::default();
        host.talk(&mut cx, "smith");
        frame(&mut host, &mut cx, &mut g, 1.0);
        let choices = json::aget(json::dict(&host.state(&cx)["dialogue"]), "choices").to_vec();
        let k = choices.iter().position(|c| s(json::dict(c), "service") == "recipe").expect("the recipe reply");
        host.click_id(&mut cx, &format!("choice.{k}")).expect("the recipe reply");
        frame(&mut host, &mut cx, &mut g, 1.0);
        assert_eq!(host.state(&cx)["recipe"], json!("ward:smith"));
        assert!(widget(&host, "recipe").is_some() && widget(&host, "dialogue").is_none(), "the pane in the dialogue box's place");
        assert!(widget(&host, "recipe.do.train_bow").is_some_and(|w| w["enabled"] == json!(true)), "Do on a recipe that can run");
        host.click_id(&mut cx, "recipe.do.train_bow").expect("Do");
        frame(&mut host, &mut cx, &mut g, 1.0);
        let m = &cx.gs().party.members[0];
        assert_eq!((m.trained.get("bow").copied(), cx.gs().party.gold), (Some(1), FUNDS - PRICE), "trained and paid");
        assert!(widget(&host, "recipe.line").is_some_and(|w| s(json::dict(&w), "text").contains("Bow Training")), "the line names the run");
        assert!(widget(&host, "recipe.do.train_bow").is_some_and(|w| w["enabled"] == json!(false)), "already at the level: off");
        // the item pick: past the list it reads the last item; a run that uses the last of the picked item goes back
        // to the first item (never salvaging a neighbour unseen)
        host.svc.act(&mut cx, SvcAct::Recipe(RecipeAct::Item(5)));
        frame(&mut host, &mut cx, &mut g, 1.0);
        host.click_id(&mut cx, "recipe.do.break_down").expect("Do");
        frame(&mut host, &mut cx, &mut g, 1.0);
        let stash = &cx.gs().party.stash;
        assert_eq!((stash.get("spare"), stash.get("rusty").copied(), cx.gs().party.gold), (None, Some(1), FUNDS - PRICE + SALVAGE), "the clamped pick (the last item) broke down");
        assert_eq!(host.state(&cx)["recipe_pane"]["item"], json!(0), "the pick went back to the first item");
        host.back(&mut cx);
        frame(&mut host, &mut cx, &mut g, 1.0);
        assert!(host.state(&cx)["recipe"].is_null() && widget(&host, "dialogue").is_some(), "back in the conversation");
    }

    /// The act weapon instances start in, the blade's full durability and the repair's full price.
    const FROM_ACT: i64 = 2;
    const FULL: i64 = 20;
    const REPAIR: i64 = 10;

    /// PK2-FOLLOWUP: a class with two minor families, a wearing blade, the smith's minor training and repair, under a
    /// ruleset with the rule `durability` from act `FROM_ACT`.
    fn steel_data() -> Data {
        let catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter", "proficiency": {"sword": {"level": 3, "role": "main"}, "mace": {"level": 1, "role": "minor"}, "dagger": {"level": 1, "role": "minor"}}}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
            "recipes": [{"id": "train_minor", "name": "Minor Training", "npc": "smith", "requires": [], "cost": {}, "target": "member",
                "effects": [{"effect": "train_proficiency", "role": "minor", "to": 2}]},
                {"id": "repair", "name": "Repair", "npc": "smith", "requires": [], "cost": {}, "target": "weapon", "effects": [{"effect": "repair", "gold": REPAIR}]}],
            "items": [{"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "durability": FULL}],
            "rulesets": [{"id": "steel", "name": "Steel", "rules": ["durability"]}]
        });
        let worlds = json!({"ward": {"manifest": {"id": "ward", "name": "Ward", "start_location": "hub", "start_party": ["hero"], "ruleset": "steel", "campaign": {"start_act": 1}}, "own": {
            "locations": with_ids("ward", json!([{"id": "hub", "name": "Hub"}])),
            "characters": with_ids("ward", json!([{"id": "hero", "name": "Hero", "role": "party", "base_unit": "hero"}, {"id": "smith", "name": "Smith", "role": "npc"}]))
        }}});
        let mut d = Data::from_values(&catalog, &worlds, &json!({}));
        d.from_act = FROM_ACT;
        d
    }

    /// PK2-FOLLOWUP (FB-212 (d), FB-211): the pane shows a family picker for the minor training (the class's two minors)
    /// and a weapon picker for the repair; a picked family trains that family, and Do on the repair restores the blade
    /// for its share of the price.
    #[test]
    fn the_family_and_weapon_pickers_drive_their_recipes() {
        let (mut cx, mut g) = (story_cx(steel_data()), gfx());
        let d = cx.data.clone();
        let mut gs = new_game::create_for_world(&d, "ward", 1).expect("a new game");
        gs.campaign.as_mut().expect("a campaign world").act = FROM_ACT;
        gs.party.gold = FUNDS;
        gs.party.members[0].equipment.insert("main_hand".into(), "blade".into());
        weapons::sync(&d, &mut gs);
        let uid = weapons::all(&gs.party)[0].uid.clone();
        weapons::find_mut(&mut gs.party, &uid).expect("the blade").durability = FULL / 2;
        enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        let mut host = ServiceScreen::default();
        host.talk(&mut cx, "smith");
        frame(&mut host, &mut cx, &mut g, 1.0);
        let choices = json::aget(json::dict(&host.state(&cx)["dialogue"]), "choices").to_vec();
        let k = choices.iter().position(|c| s(json::dict(c), "service") == "recipe").expect("the recipe reply");
        host.click_id(&mut cx, &format!("choice.{k}")).expect("the recipe reply");
        frame(&mut host, &mut cx, &mut g, 1.0);
        assert!(widget(&host, "recipe.family").is_some() && widget(&host, "recipe.weapon").is_some(), "both pickers show");
        assert!(widget(&host, "recipe.weapon").is_some_and(|w| s(json::dict(&w), "text").contains(&format!("{}/{FULL}", FULL / 2))), "the weapon's durability shows");
        host.svc.act(&mut cx, SvcAct::Recipe(RecipeAct::Family(1)));
        frame(&mut host, &mut cx, &mut g, 1.0);
        host.click_id(&mut cx, "recipe.do.train_minor").expect("Do");
        frame(&mut host, &mut cx, &mut g, 1.0);
        let class = d.cat.classes["fighter"].clone();
        let m = &cx.gs().party.members[0];
        assert_eq!((m.proficiency(&d, &class).level_of("dagger"), m.proficiency(&d, &class).level_of("mace")), (2, 1), "the second minor picked and trained");
        host.click_id(&mut cx, "recipe.do.repair").expect("Do");
        frame(&mut host, &mut cx, &mut g, 1.0);
        let paid = emberwake_data::recipe::repair_share(REPAIR, FULL / 2, FULL);
        assert_eq!((weapons::find(&cx.gs().party, &uid).map(|w| w.durability), cx.gs().party.gold), (Some(FULL), FUNDS - paid), "repaired for its share");
    }
}
