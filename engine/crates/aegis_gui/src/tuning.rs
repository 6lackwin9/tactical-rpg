//! Feel tuning (engine/data/tuning.json): every timing, size and feedback colour the GUI animates with. Reloaded when
//! the file changes, so the user can tune while playing and the agent reads the exact values.

use crate::canvas::{rgb, Rgba};
use serde_json::Value;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Tuning {
    pub step_ms: u64,
    pub round_pause_ms: u64,
    pub float_ms: u64,
    pub float_rise_px: i32,
    pub hit_flash_ms: u64,
    pub hit_shake_px: i32,
    pub heal_color: Rgba,
    pub damage_color: Rgba,
    pub crit_color: Rgba,
    pub intent_arrows: bool,
    pub token_px: i32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            step_ms: 420,
            round_pause_ms: 900,
            float_ms: 900,
            float_rise_px: 16,
            hit_flash_ms: 180,
            hit_shake_px: 2,
            heal_color: rgb(96, 220, 120),
            damage_color: rgb(255, 96, 80),
            crit_color: rgb(255, 210, 60),
            intent_arrows: true,
            token_px: 28,
        }
    }
}

fn color(v: &Value, d: Rgba) -> Rgba {
    match v.as_array() {
        Some(a) if a.len() >= 3 => rgb(a[0].as_u64().unwrap_or(0) as u8, a[1].as_u64().unwrap_or(0) as u8, a[2].as_u64().unwrap_or(0) as u8),
        _ => d,
    }
}

impl Tuning {
    pub fn from_json(v: &Value) -> Self {
        let d = Tuning::default();
        let u = |k: &str, def: u64| v.get(k).and_then(Value::as_u64).unwrap_or(def);
        let i = |k: &str, def: i32| v.get(k).and_then(Value::as_i64).map(|x| x as i32).unwrap_or(def);
        Self {
            step_ms: u("step_ms", d.step_ms),
            round_pause_ms: u("round_pause_ms", d.round_pause_ms),
            float_ms: u("float_ms", d.float_ms),
            float_rise_px: i("float_rise_px", d.float_rise_px),
            hit_flash_ms: u("hit_flash_ms", d.hit_flash_ms),
            hit_shake_px: i("hit_shake_px", d.hit_shake_px),
            heal_color: v.get("heal_color").map(|c| color(c, d.heal_color)).unwrap_or(d.heal_color),
            damage_color: v.get("damage_color").map(|c| color(c, d.damage_color)).unwrap_or(d.damage_color),
            crit_color: v.get("crit_color").map(|c| color(c, d.crit_color)).unwrap_or(d.crit_color),
            intent_arrows: v.get("intent_arrows").and_then(Value::as_bool).unwrap_or(d.intent_arrows),
            token_px: i("token_px", d.token_px).clamp(8, 32),
        }
    }
}

/// The tuning file plus change detection.
pub struct TuningFile {
    pub path: Option<PathBuf>,
    pub value: Tuning,
    pub raw: Value,
    stamp: Option<SystemTime>,
}

impl TuningFile {
    pub fn new(path: Option<PathBuf>) -> Self {
        let mut t = Self { path, value: Tuning::default(), raw: Value::Null, stamp: None };
        t.reload_if_changed();
        t
    }

    /// Re-reads the file when its timestamp moved; true when the values changed.
    pub fn reload_if_changed(&mut self) -> bool {
        let Some(p) = &self.path else { return false };
        let stamp = std::fs::metadata(p).and_then(|m| m.modified()).ok();
        if stamp.is_none() || stamp == self.stamp {
            return false;
        }
        self.stamp = stamp;
        let Ok(text) = std::fs::read_to_string(p) else { return false };
        let Ok(v) = serde_json::from_str::<Value>(&text) else { return false };
        self.value = Tuning::from_json(&v);
        self.raw = v;
        true
    }
}
