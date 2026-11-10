//! Unit tracks (PK2-S1, `docs/sessions/PK2_GRAFTS.md` section 2; engine-only, no Godot original): a per-member
//! integer stage on a track row (kind `unit_tracks`). A row names its stages in order (stage 1 is the first row of
//! `stages`; stage 0 = not on the track); each stage carries a slotless passive the member holds from that stage on
//! (the passives of every stage up to the member's apply: a stage adds its symptoms to the earlier ones, so a battle
//! that advances the stage only adds the new stage's passive) and the last stage may carry a departure (`lost`, for
//! the row's `cause`, an id the game names in its data). A battle mark whose id is the row's `mark` advances the stage (in battle at once, saved at the
//! settle). The kind ships without rows until the asset lane fills it, so no unit holds a track.

use aegis_core::value::*;
use serde_json::Value;

/// The catalog kind (file data/catalog/unit_tracks.json; a missing file is an empty table).
pub const UNIT_TRACKS_KIND: &str = "unit_tracks";
/// A stage's departure values ("" = none): the member leaves the party with that life when it settles at the stage.
pub const TRACK_DEPARTURES: [&str; 1] = ["lost"];
/// The most stages a track row holds.
pub const TRACK_STAGES_MAX: usize = 9;
/// A stage row's keys.
pub const TRACK_STAGE_KEYS: [&str; 3] = ["name", "passive", "departure"];

/// One stage of a track row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackStage {
    pub name: String,
    /// The slotless passive the stage grants ("" = none).
    pub passive: String,
    /// The life the member departs with at this stage ("" = none; only the last stage may set one).
    pub departure: String,
}

/// One track row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnitTrackDef {
    pub id: String,
    pub name: String,
    /// The battle mark id that advances the stage by its value ("" = no mark does).
    pub mark: String,
    /// The departure's cause id (the game's word; a save's departure accepts it beside `DEPART_CAUSES`).
    pub cause: String,
    pub stages: Vec<TrackStage>,
}

impl UnitTrackDef {
    pub fn from_obj(o: &Obj) -> Self {
        let stages = array(o, "stages")
            .iter()
            .filter_map(Value::as_object)
            .map(|s| TrackStage { name: string(s, "name", ""), passive: string(s, "passive", ""), departure: string(s, "departure", "") })
            .collect();
        Self { id: string(o, "id", ""), name: string(o, "name", ""), mark: string(o, "mark", ""), cause: string(o, "cause", ""), stages }
    }

    /// The highest stage.
    pub fn top(&self) -> i32 {
        self.stages.len() as i32
    }

    /// `stage` held within the row (0..=top).
    pub fn clamp(&self, stage: i64) -> i32 {
        stage.clamp(0, i64::from(self.top())) as i32
    }

    /// Stage `n` (1-based; None for 0 or past the top).
    pub fn stage(&self, n: i32) -> Option<&TrackStage> {
        usize::try_from(n).ok().filter(|&n| n >= 1).and_then(|n| self.stages.get(n - 1))
    }

    /// The passives a member at stage `n` holds: every stage's up to `n`, in stage order.
    pub fn passives_to(&self, n: i32) -> Vec<String> {
        (1..=n.min(self.top())).filter_map(|s| self.stage(s)).map(|s| s.passive.clone()).filter(|p| !p.is_empty()).collect()
    }

    /// The departure life at stage `n` ("" = none).
    pub fn departure_at(&self, n: i32) -> &str {
        self.stage(n).map_or("", |s| s.departure.as_str())
    }
}
