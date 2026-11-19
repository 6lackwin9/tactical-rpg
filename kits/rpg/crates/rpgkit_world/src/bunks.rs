//! The hub's bunks and idle spots (BUNKS1 BK1-BK4, BK9-BK12, spec BUNKS1 sections 1 and 3;
//! engine-only, no Godot original; story-free). A location whose map has bunk props (ids `bunk_<n>`) in a world with a
//! `roster` block is a hub: each bunk prop, in number order, is bound to the next member housed in a bunk or a crypt
//! (roster order; `crypt_only` members take none), and shows the member's state. Members idle in the hub: active
//! members (not the lead) at the social tables (the map's `idle_*` anchors, then the open cells around its `table*`
//! props); benched members sleep in their bunks (HUB-BUNKS2: drawn on the bed, not idling); crypt members are not
//! shown. Passing the lead trades places: the new lead stands up from their spot and the old lead takes it; no one else
//! moves.

use crate::data::Data;
use crate::explore::{self, Pos};
use crate::state::{GameState, UnitState};
use aegis_core::json::{self, Dict};

/// The id prefix of a bunk prop (`bunk_1`, `bunk_2`, ...).
pub const BUNK_PROP_PREFIX: &str = "bunk_";
/// The id prefix of a social table prop.
pub const TABLE_PROP_PREFIX: &str = "table";
/// The id prefix of a map anchor that is an idle spot at the tables (before the cells around the tables).
pub const IDLE_ANCHOR_PREFIX: &str = "idle_";

/// BK2: what a bunk shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BunkState {
    /// Bare, the mattress rolled.
    Empty,
    /// A benched member resting there (made bed, keepsake on the post, name plate).
    Benched,
    /// An active member (made bed, kit packed at the foot, name plate).
    Active,
    /// A member in the crypts (made bed under a cloth, name plate).
    Crypt,
}

impl BunkState {
    /// The state's name (the art suffix `<art>_<name>`, the agent link).
    pub fn name(self) -> &'static str {
        match self {
            BunkState::Empty => "empty",
            BunkState::Benched => "benched",
            BunkState::Active => "active",
            BunkState::Crypt => "crypt",
        }
    }
}

/// The four states, in the order of `BunkState`.
pub const BUNK_STATES: [BunkState; 4] = [BunkState::Empty, BunkState::Benched, BunkState::Active, BunkState::Crypt];

/// One bunk prop of the hub and who sleeps there.
#[derive(Debug, Clone, PartialEq)]
pub struct Bunk {
    /// The prop's id and art.
    pub prop: String,
    pub art: String,
    /// The prop's footprint (x, y, w, h) in cells.
    pub foot: (i64, i64, i64, i64),
    /// The member bound to it ("" = empty).
    pub member: String,
    pub state: BunkState,
}

impl Bunk {
    /// BK9: the cell in front of the bunk (south of its footprint's left cell; HUB-BUNKS2: its benched member sleeps in
    /// the bunk instead of idling there).
    pub fn spot(&self) -> Pos {
        (self.foot.0, self.foot.1 + self.foot.3)
    }
}

/// The bunk props of map `t` in number order (`bunk_<n>`, then map order).
pub fn bunk_props(t: &Dict) -> Vec<Dict> {
    let mut props: Vec<(i64, usize, Dict)> = json::aget(t, "props")
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let p = p.as_object()?;
            let n = json::s(p, "id").strip_prefix(BUNK_PROP_PREFIX)?.parse::<i64>().ok()?;
            Some((n, i, p.clone()))
        })
        .collect();
    props.sort_by_key(|(n, i, _)| (*n, *i));
    props.into_iter().map(|(_, _, p)| p).collect()
}

/// BK1: whether map `t` is a hub for game `gs`: the world has a roster block and the map bunk props.
pub fn is_hub(gs: &GameState, t: &Dict) -> bool {
    gs.party.roster.has_block() && !bunk_props(t).is_empty()
}

/// BK1, BK2: the hub's bunks: each bunk prop bound, in order, to the next member housed `bunk` or `crypt` (roster
/// order); the rest are empty. Members past the props have no bunk drawn.
pub fn bunks(gs: &GameState, t: &Dict) -> Vec<Bunk> {
    let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
    let mut sleepers = gs.party.members.iter().filter(|m| m.housing() != crate::roster::CRYPT_ONLY);
    bunk_props(t)
        .into_iter()
        .map(|p| {
            let foot = (json::i(&p, "x", 0), json::i(&p, "y", 0), json::i(&p, "w", 1).max(1), json::i(&p, "h", 1).max(1));
            let (member, state) = match sleepers.next() {
                None => (String::new(), BunkState::Empty),
                Some(m) if m.housing() != rpgkit_data::HOUSING[0] => (m.id.clone(), BunkState::Crypt),
                Some(m) if active.contains(&m.id) => (m.id.clone(), BunkState::Active),
                Some(m) => (m.id.clone(), BunkState::Benched),
            };
            Bunk { prop: json::s(&p, "id"), art: json::s(&p, "art"), foot, member, state }
        })
        .collect()
}

/// BK3: the condition words of the hover and the card.
pub const RESTING: &str = "resting";
pub const WOUNDED_WORD: &str = "wounded";
pub const HIGH_STRAIN: &str = "high Strain";
pub const READY_WORD: &str = "ready";

/// BK3: member `m`'s condition word: wounded (the roster condition), high Strain (its next battle starts its channel
/// meter at half its max or more: the floor its grafts give), else resting (benched) or ready (active).
pub fn condition(d: &Data, gs: &GameState, m: &UnitState) -> &'static str {
    if m.has_condition(crate::roster::WOUNDED) {
        WOUNDED_WORD
    } else if strain_high(d, gs, m) {
        HIGH_STRAIN
    } else if gs.party.active_members().iter().any(|a| a.id == m.id) {
        READY_WORD
    } else {
        RESTING
    }
}

/// BK3: whether member `m` starts its next battle with high Strain: the world's channel meter (`ruleset.channel_meter`)
/// floor its build gives is at least half the meter's max (false without the rule or the meter).
pub fn strain_high(d: &Data, gs: &GameState, m: &UnitState) -> bool {
    let world = gs.world_id.clone();
    let meter = d.ruleset(&world).numbers.channel_meter;
    let Some(def) = (!meter.is_empty()).then(|| d.cat.unit_meters.get(&meter)).flatten() else { return false };
    let Some(f) = crate::roster::field_unit(d, gs, m, &world) else { return false };
    f.unit.meter_floor(&meter).is_some_and(|floor| floor > 0 && floor * 2 >= def.max)
}

/// The members shown idling, and where (BK9), for a lead `leader`.
pub type Spots = Vec<(String, Pos)>;

/// BK9: the social table spots of map `t` in their fixed order: the `idle_*` anchors (map order), then around each
/// `table*` prop (map order) the open cells west, east, south and north of it; never a cell another anchor or a marker
/// of location `loc` takes.
pub fn table_spots(loc: &Dict, t: &Dict) -> Vec<Pos> {
    let mut taken: Vec<Pos> = json::aget(t, "anchors")
        .iter()
        .filter_map(|a| a.as_object())
        .filter(|a| !json::s(a, "id").starts_with(IDLE_ANCHOR_PREFIX))
        .map(|a| (json::i(a, "x", -1), json::i(a, "y", -1)))
        .collect();
    taken.extend(explore::markers_of(loc).iter().map(|m| explore::marker_pos(t, m, 0)));
    let mut out: Vec<Pos> = Vec::new();
    let add = |p: Pos, out: &mut Vec<Pos>| {
        if explore::is_open(t, p.0, p.1) && !taken.contains(&p) && !out.contains(&p) {
            out.push(p);
        }
    };
    for a in json::aget(t, "anchors").iter().filter_map(|a| a.as_object()).filter(|a| json::s(a, "id").starts_with(IDLE_ANCHOR_PREFIX)) {
        add((json::i(a, "x", -1), json::i(a, "y", -1)), &mut out);
    }
    for p in json::aget(t, "props").iter().filter_map(|p| p.as_object()).filter(|p| json::s(p, "id").starts_with(TABLE_PROP_PREFIX)) {
        let (x, y, w, h) = (json::i(p, "x", 0), json::i(p, "y", 0), json::i(p, "w", 1).max(1), json::i(p, "h", 1).max(1));
        for dy in 0..h {
            add((x - 1, y + dy), &mut out);
        }
        for dy in 0..h {
            add((x + w, y + dy), &mut out);
        }
        for dx in 0..w {
            add((x + dx, y + h), &mut out);
        }
        for dx in 0..w {
            add((x + dx, y - 1), &mut out);
        }
    }
    out
}

/// BK9, BK12: where each member idles with `leader` leading: the active bunk members (roster order, the lead left out)
/// on the table spots in order; benched members sleep in their bunks (HUB-BUNKS2, `sleepers`), so they idle nowhere;
/// crypt members, `crypt_only` members and members past the spots are not shown.
pub fn place(gs: &GameState, loc: &Dict, t: &Dict, leader: &str) -> Spots {
    let mut out = Spots::new();
    let tables = table_spots(loc, t);
    let mut next_table = tables.into_iter();
    let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
    for m in gs.party.members.iter().filter(|m| m.id != leader && m.housing() == rpgkit_data::HOUSING[0] && active.contains(&m.id)) {
        if let Some(p) = next_table.next() {
            out.push((m.id.clone(), p));
        }
    }
    out
}

/// HUB-BUNKS2 (4): the bunks of map `t` whose member sleeps there: a benched member in the hub rests in their bunk
/// (state `Benched`), in bunk order.
pub fn sleepers(gs: &GameState, t: &Dict) -> Vec<Bunk> {
    bunks(gs, t).into_iter().filter(|b| b.state == BunkState::Benched).collect()
}

/// What `place` depends on (members, housing, who is active, the lead): a change re-places everyone.
pub fn signature(gs: &GameState, leader: &str) -> String {
    let active: Vec<String> = gs.party.active_members().iter().map(|m| m.id.clone()).collect();
    let members: Vec<String> = gs.party.members.iter().map(|m| format!("{}:{}", m.id, m.housing())).collect();
    format!("{}|{}|{}", members.join(","), active.join(","), leader)
}

/// BK10: the lead passes from `old` to `new` in the hub: `new` stands up from its spot onto the first open cell beside it
/// (up, right, down, left) that no one idles on, else trades cells with the party; `old` takes the freed spot. Moves the
/// party (`gs.position`) there and rewrites `spots`; no other member moves. False (nothing changes) when `new` idles
/// nowhere.
pub fn pass_lead(gs: &mut GameState, t: &Dict, spots: &mut Spots, old: &str, new: &str) -> bool {
    let Some(i) = spots.iter().position(|(id, _)| id == new) else { return false };
    let spot = spots[i].1;
    let busy = |p: Pos| spots.iter().any(|(_, q)| *q == p);
    let stand = explore::DIRS.iter().map(|d| (spot.0 + d.0, spot.1 + d.1)).find(|p| explore::is_open(t, p.0, p.1) && !busy(*p) && *p != gs.position);
    gs.position = stand.unwrap_or(gs.position);
    spots[i] = (old.to_string(), spot);
    true
}
