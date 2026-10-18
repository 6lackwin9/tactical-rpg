use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// 2D Integer Coordinate Vector (matching Godot's `Vector2i`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const ONE: Self = Self { x: 1, y: 1 };
    pub const NONE: Self = Self { x: -1, y: -1 };
    pub const UP: Self = Self { x: 0, y: -1 };
    pub const DOWN: Self = Self { x: 0, y: 1 };
    pub const LEFT: Self = Self { x: -1, y: 0 };
    pub const RIGHT: Self = Self { x: 1, y: 0 };

    /// Cardinal directions in clockwise order (matching Godot GridPath::DIRS).
    pub const DIRS: [Self; 4] = [Self::RIGHT, Self::UP, Self::LEFT, Self::DOWN];

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance (|x1 - x2| + |y1 - y2|).
    pub const fn manhattan_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Chebyshev distance (max(|x1 - x2|, |y1 - y2|)).
    pub const fn chebyshev_distance(self, other: Self) -> i32 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        if dx > dy {
            dx
        } else {
            dy
        }
    }

    /// Returns the 4 cardinal neighboring coordinates.
    pub fn cardinal_neighbors(self) -> [Self; 4] {
        [
            self + Self::RIGHT,
            self + Self::UP,
            self + Self::LEFT,
            self + Self::DOWN,
        ]
    }

    /// Checks if the coordinate is within grid bounds [0, width) x [0, height).
    pub const fn is_in_bounds(self, width: i32, height: i32) -> bool {
        self.x >= 0 && self.x < width && self.y >= 0 && self.y < height
    }

    /// Reference game 12x9 grid centre distance calculation (CombatGrid::centre_distance).
    /// Used for deterministic tie-breaking in pathfinding and deployment.
    pub const fn centre_distance_12x9(self) -> i32 {
        (2 * self.x - 11).abs() + (2 * self.y - 8).abs()
    }

    /// Reference game 12x9 party half check (left side: columns 0..5).
    pub const fn is_party_half_12x9(self) -> bool {
        self.x * 2 < 12
    }
}

impl Add for IVec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for IVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for IVec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for IVec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Neg for IVec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl fmt::Display for IVec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<(i32, i32)> for IVec2 {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivec2_math() {
        let a = IVec2::new(3, 4);
        let b = IVec2::new(1, 2);
        assert_eq!(a + b, IVec2::new(4, 6));
        assert_eq!(a - b, IVec2::new(2, 2));
        assert_eq!(-a, IVec2::new(-3, -4));
        assert_eq!(a.manhattan_distance(b), 4);
        assert_eq!(a.chebyshev_distance(b), 2);
    }

    #[test]
    fn test_bounds_and_centre() {
        let inside = IVec2::new(5, 4);
        assert!(inside.is_in_bounds(12, 9));
        assert!(inside.is_party_half_12x9());
        assert_eq!(inside.centre_distance_12x9(), 1); // 2*5-11 = -1 -> 1; 2*4-8 = 0 -> 0 -> sum = 1
    }
}
