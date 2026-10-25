//! The game's feel tuning (games/emberwake/tuning.json): every timing, size and feedback colour the battle screen animates
//! with, read by the engine's live tuning file (`aegis_ui::tuning::TuningFile`, reloaded when the file changes).

use aegis_ui::canvas::{rgb, Rgba};
use aegis_ui::tuning::Tuned;
use serde_json::Value;

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
    /// STAGE1: the Stage view's unit sprite square (24-64).
    pub token_px: i32,
    /// STAGE1: the Flat view's unit sprite square (24-64; the view and the key go after the user's review, STAGE1-C).
    pub flat_token_px: i32,
    /// STAGE1: the Stage foot point's height above its cell's bottom edge (2-10).
    pub foot_inset_px: i32,
    /// STAGE1: the opacity of a Stage unit's contact shadow.
    pub shadow_alpha: u8,
    /// STAGE1: the opacity of a Stage unit's side-coloured foot ring.
    pub foot_ring_alpha: u8,
    /// STAGE1: the opacity of the Stage grid lines (1-255; always on).
    pub grid_alpha: u8,
    /// SPRITE1-U: how long a moving unit slides to its new tile playing `walk` (0: it jumps).
    pub walk_ms: u64,
    /// SPRITE1-U: how long a fallen unit shows the death clip's last frame before the dead marker.
    pub death_hold_ms: u64,
    /// SPRITE1-U: the opacity (0-255) of the side-coloured ellipse under a unit's feet (the Flat view).
    pub foot_alpha: u8,
    /// SPRITE1-U: the opacity of the white outline around the hovered unit's sprite.
    pub hover_outline_alpha: u8,
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
            token_px: 40,
            flat_token_px: 28,
            foot_inset_px: 5,
            shadow_alpha: 120,
            foot_ring_alpha: 200,
            grid_alpha: 34,
            walk_ms: 360,
            death_hold_ms: 600,
            foot_alpha: 110,
            hover_outline_alpha: 110,
        }
    }
}

fn color(v: &Value, d: Rgba) -> Rgba {
    match v.as_array() {
        Some(a) if a.len() >= 3 => rgb(a[0].as_u64().unwrap_or(0) as u8, a[1].as_u64().unwrap_or(0) as u8, a[2].as_u64().unwrap_or(0) as u8),
        _ => d,
    }
}

impl Tuned for Tuning {
    fn from_json(v: &Value) -> Self {
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
            token_px: i("token_px", d.token_px).clamp(24, 64),
            flat_token_px: i("flat_token_px", d.flat_token_px).clamp(24, 64),
            foot_inset_px: i("foot_inset_px", d.foot_inset_px).clamp(2, 10),
            shadow_alpha: u("shadow_alpha", d.shadow_alpha as u64).min(255) as u8,
            foot_ring_alpha: u("foot_ring_alpha", d.foot_ring_alpha as u64).min(255) as u8,
            grid_alpha: u("grid_alpha", d.grid_alpha as u64).clamp(1, 255) as u8,
            walk_ms: u("walk_ms", d.walk_ms).min(2000),
            death_hold_ms: u("death_hold_ms", d.death_hold_ms).min(5000),
            foot_alpha: u("foot_alpha", d.foot_alpha as u64).min(255) as u8,
            hover_outline_alpha: u("hover_outline_alpha", d.hover_outline_alpha as u64).min(255) as u8,
        }
    }
}

