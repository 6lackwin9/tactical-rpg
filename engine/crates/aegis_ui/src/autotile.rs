//! Tile neighbourhood masks (LOOK1-E2), game-agnostic and naming no terrain: which transition piece a tile corner
//! takes (a dual grid: the four cells around a corner give its mask) and which edges of a block are open. The caller's
//! closures say what a cell is, off-grid cells included. Drawing is the caller's (the Kit page's Tiles page; a game's
//! views draw the pieces from their own sets).

/// The corner at the top-left of cell (`cx`, `cy`): bit 1 cell (cx - 1, cy - 1), 2 (cx, cy - 1), 4 (cx - 1, cy), 8
/// (cx, cy) is upper. 0 and 15 take no piece.
pub fn corner_mask(is_upper: impl Fn(i32, i32) -> bool, cx: i32, cy: i32) -> u8 {
    let bit = |x: i32, y: i32, b: u8| if is_upper(x, y) { b } else { 0 };
    bit(cx - 1, cy - 1, 1) | bit(cx, cy - 1, 2) | bit(cx - 1, cy, 4) | bit(cx, cy, 8)
}

/// Every corner of a `w` x `h` grid (0..=w by 0..=h, row by row) that takes a piece: (cx, cy, mask) with mask 1-14.
pub fn corner_pieces(w: i32, h: i32, is_upper: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32, u8)> {
    let mut out = Vec::new();
    for cy in 0..=h {
        for cx in 0..=w {
            let mask = corner_mask(&is_upper, cx, cy);
            if (1..=14).contains(&mask) {
                out.push((cx, cy, mask));
            }
        }
    }
    out
}

/// The open edges of a block cell: true where that neighbour is not a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEdges {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

/// The open edges of the block at (`x`, `y`); the closure answers off-grid cells.
pub fn block_edges(is_block: impl Fn(i32, i32) -> bool, x: i32, y: i32) -> BlockEdges {
    BlockEdges { north: !is_block(x, y - 1), east: !is_block(x + 1, y), south: !is_block(x, y + 1), west: !is_block(x - 1, y) }
}
