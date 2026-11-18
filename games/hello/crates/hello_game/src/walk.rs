//! The game's whole state: a walker on a walled field. Pure rules, no screen: the screens, the save and the command host
//! all go through `Walk`, so a scenario, a replay and a unit test see the same game.

use serde_json::{json, Value};

/// The field in tiles; the outer ring is wall.
pub const COLS: i64 = 20;
pub const ROWS: i64 = 11;
/// Where a new walk starts.
pub const START: (i64, i64) = (2, 5);
/// Stones inside the field (walls the walker bumps into), as (column, row).
pub const STONES: &[(i64, i64)] = &[(6, 3), (6, 4), (6, 5), (12, 6), (13, 6), (14, 6), (9, 8)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Walk {
    pub x: i64,
    pub y: i64,
    /// Steps taken since the walk began (bumps do not count).
    pub steps: i64,
}

impl Default for Walk {
    fn default() -> Self {
        Self { x: START.0, y: START.1, steps: 0 }
    }
}

/// Whether the tile at (x, y) blocks: the outer ring, or a stone.
pub fn blocked(x: i64, y: i64) -> bool {
    x <= 0 || y <= 0 || x >= COLS - 1 || y >= ROWS - 1 || STONES.contains(&(x, y))
}

impl Walk {
    /// One step by (dx, dy), each -1, 0 or 1 and not both 0; false (and nothing changes) on a bump or a bad step.
    pub fn step(&mut self, dx: i64, dy: i64) -> bool {
        if !(-1..=1).contains(&dx) || !(-1..=1).contains(&dy) || (dx == 0 && dy == 0) || blocked(self.x + dx, self.y + dy) {
            return false;
        }
        self.x += dx;
        self.y += dy;
        self.steps += 1;
        true
    }

    /// The saved and reported form.
    pub fn to_value(self) -> Value {
        json!({"x": self.x, "y": self.y, "steps": self.steps})
    }

    /// The walk a saved form holds; refused when a field is missing, not whole, or on a blocked tile.
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let field = |k: &str| v.get(k).and_then(Value::as_i64).ok_or_else(|| format!("'{k}' is missing or not a whole number"));
        let w = Self { x: field("x")?, y: field("y")?, steps: field("steps")? };
        if blocked(w.x, w.y) {
            return Err(format!("({}, {}) is not a free tile", w.x, w.y));
        }
        if w.steps < 0 {
            return Err("'steps' is below 0".into());
        }
        Ok(w)
    }
}

/// The command set the engine's host answers with (`hello serve`, a replay): `new`, `move` {dx, dy}, `state`.
#[derive(Debug, Default)]
pub struct Commands {
    pub walk: Walk,
}

impl aegis_host::Commands for Commands {
    fn mutating(&self, cmd: &str) -> bool {
        matches!(cmd, "new" | "move")
    }
    fn restarts(&self, cmd: &str) -> bool {
        cmd == "new"
    }
    fn handle(&mut self, cmd: &str, req: &Value) -> Value {
        match cmd {
            "new" => {
                self.walk = Walk::default();
                json!({"ok": true, "walk": self.walk.to_value()})
            }
            "move" => {
                let d = |k: &str| req.get(k).and_then(Value::as_i64).unwrap_or(0);
                if self.walk.step(d("dx"), d("dy")) {
                    json!({"ok": true, "walk": self.walk.to_value()})
                } else {
                    json!({"ok": false, "error": "blocked", "walk": self.walk.to_value()})
                }
            }
            "state" => json!({"ok": true, "walk": self.walk.to_value()}),
            other => json!({"ok": false, "error": format!("unknown cmd '{other}' (new, move, state, history)")}),
        }
    }
    fn checkpoint(&mut self) -> Option<Value> {
        Some(self.walk.to_value())
    }
}
