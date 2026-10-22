//! A draw list sorted by feet (engine-only, PROPS1 4.1): items pushed with the y of their feet, a group (which of
//! two at the same feet draws later) and their left edge come back in back-to-front order. It draws nothing itself;
//! a screen (a region map with standing props and characters, later the battle grid) sorts what it draws with it.

/// Items with their sort keys, in push order until `into_sorted`.
pub struct DepthList<T> {
    items: Vec<(f32, u8, f32, T)>,
}

impl<T> Default for DepthList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DepthList<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds `item` whose feet are at `foot_y`; at equal feet the lower `group` draws first, then the smaller `x`.
    pub fn push(&mut self, foot_y: f32, group: u8, x: f32, item: T) {
        self.items.push((foot_y, group, x, item));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The items in draw order: by foot_y, then group, then x, then push order (a stable sort).
    pub fn into_sorted(mut self) -> Vec<T> {
        self.items.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.total_cmp(&b.2)));
        self.items.into_iter().map(|(_, _, _, item)| item).collect()
    }
}

