//! The typed pool facade (U1.2 slice 1, `FB-04`). HP, AP and poise keep their typed fields (`Unit.hp` and `max_hp`,
//! `Unit.ap` and `max_ap`, the battle's `World.poise`), so AI and planner callers do not change; a pool is read through
//! this one view by its id. Writes stay on the fields' own paths (damage, heals, AP costs and regen, poise refill and
//! stagger). Pools that later come from data use a generic store beside these.

use crate::unit::Unit;

/// The pools with typed fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolId {
    Hp,
    Ap,
    Poise,
}

impl PoolId {
    pub const ALL: [PoolId; 3] = [PoolId::Hp, PoolId::Ap, PoolId::Poise];

    pub fn name(self) -> &'static str {
        match self {
            PoolId::Hp => "hp",
            PoolId::Ap => "ap",
            PoolId::Poise => "poise",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == s)
    }
}

/// A pool's current value and its range. The current value is the field's own: it is not clamped here (AP may sit
/// above its max after a refund, poise above its max while a windup bonus holds).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PoolView {
    pub current: i32,
    pub min: i32,
    pub max: i32,
}

impl PoolView {
    /// At or below its min (HP 0: knocked).
    pub fn is_empty(&self) -> bool {
        self.current <= self.min
    }

    /// At or above its max.
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Room left below the max (0 when full or above).
    pub fn room(&self) -> i32 {
        (self.max - self.current).max(0)
    }
}

impl Unit {
    /// The unit's own pools: HP (0 to `max_hp`) and AP (0 to `max_ap`). Poise lives in the battle, so None here (the
    /// battle's `World::pool` reads it).
    pub fn pool(&self, id: PoolId) -> Option<PoolView> {
        match id {
            PoolId::Hp => Some(PoolView { current: self.hp, min: 0, max: self.max_hp }),
            PoolId::Ap => Some(PoolView { current: self.ap, min: 0, max: self.max_ap }),
            PoolId::Poise => None,
        }
    }
}
