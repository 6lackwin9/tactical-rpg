//! Map props on the region map (game-only, PROPS1 4.2-4.6): where each prop of a map template draws (its drawn rect
//! from the art's PNG size, bottom-centred on its footprint), the feet-sorted order of standing props, markers and the
//! party (`aegis_ui::depth::DepthList`), and whether a standing prop hides the party (its outline then draws over the
//! props). Pure placement and order; `region.rs` draws. Props never change walking (the template `rows` do).

use aegis_core::json::{self, Dict};
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::depth::DepthList;

/// Logical px of art per tile at tile size 32 (PROPS1 3.1: 32 px of art per tile; the map draws at `ts` px a tile).
pub const ART_TILE: i32 = 32;
/// A standing prop without art (4.6): its footprint filled, a 1 px border, and (a 3/4 view hint) a darker south face.
pub const BOX_FILL: Rgba = rgb(52, 46, 42);
pub const BOX_EDGE: Rgba = rgb(96, 84, 72);
pub const BOX_FACE: Rgba = rgb(40, 35, 32);
/// The south face's height (logical px; at most a quarter of the block).
pub const BOX_FACE_H: i32 = 4;
/// The party's colour (its ring, and its outline over a prop that hides it, 4.5).
pub const PARTY: Rgba = rgb(250, 214, 60);

/// One prop to draw: where, which art, how.
#[derive(Debug, Clone, PartialEq)]
pub struct PropDraw {
    pub id: String,
    /// The drawn rect (logical px): the art's PNG size x ts / 32, bottom-centred on the footprint; the footprint when
    /// there is no art.
    pub rect: Rect,
    /// The footprint (logical px).
    pub foot: Rect,
    /// The art file the variant pick chose (None: no `v0.png` along the chain; standing props draw their box).
    pub file: Option<String>,
    pub flip: bool,
    /// Flat, under every character (`layer` "ground"); else standing, sorted by its feet.
    pub ground: bool,
    /// The footprint's bottom edge (logical px), the sort key of a standing prop (4.4).
    pub foot_y: i32,
}

/// What the sorted pass draws (4.4), in `sorted` order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    /// A standing prop: its index in the `layout` list.
    Prop(usize),
    /// A marker: its index in the caller's marker list.
    Marker(usize),
    Party,
}

/// A prop's art file as the variant pick chose it, with its PNG size (None: no art along the chain).
pub type Picked = Option<(String, (i32, i32))>;

/// Sort groups at equal feet (4.4): a prop, then a marker, then the party.
const GROUP_PROP: u8 = 0;
const GROUP_MARKER: u8 = 1;
const GROUP_PARTY: u8 = 2;

/// The props of template `t` that meet `area`, in list order (4.3): the map's origin `origin` and tile size `ts`
/// (logical px). `art(dirs, variant, x, y)` picks a prop's file over its folders (`Art::pick`) and gives it with its
/// PNG size; `dirs` are the `props/<set>/<art>` folders over `lineage` (the world lineage, child first). A prop is
/// culled by its drawn rect, not its footprint, so an overhang whose footprint is off screen still draws.
pub fn layout(t: &Dict, lineage: &[String], origin: (i32, i32), ts: i32, area: Rect, mut art: impl FnMut(&[&str], Option<u32>, i32, i32) -> Picked) -> Vec<PropDraw> {
    let (ox, oy) = origin;
    let mut out = Vec::new();
    for raw in json::aget(t, "props") {
        let Some(p) = raw.as_object() else { continue };
        let (x, y) = (json::i(p, "x", 0) as i32, json::i(p, "y", 0) as i32);
        let (w, h) = (json::i(p, "w", 1).max(1) as i32, json::i(p, "h", 1).max(1) as i32);
        let variant = u32::try_from(json::i(p, "variant", -1)).ok();
        let dirs = emberwake_content::art::prop_dirs(lineage, &json::s(p, "art"));
        let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
        let foot = Rect::new(ox + x * ts, oy + y * ts, w * ts, h * ts);
        let file = art(&dirs, variant, x, y);
        let rect = match &file {
            Some((_, (pw, ph))) => {
                let (dw, dh) = (scaled(*pw, ts), scaled(*ph, ts));
                Rect::new(foot.x + (foot.w - dw) / 2, foot.y + foot.h - dh, dw, dh)
            }
            None => foot,
        };
        let seen = rect.intersect(area);
        if seen.w <= 0 || seen.h <= 0 {
            continue;
        }
        out.push(PropDraw {
            id: json::s(p, "id"),
            rect,
            foot,
            file: file.map(|(f, _)| f),
            flip: json::b(p, "flip", false),
            ground: json::s(p, "layer") == "ground",
            foot_y: foot.y + foot.h,
        });
    }
    out
}

/// Art px at `ts` logical px a tile (rounded).
fn scaled(px: i32, ts: i32) -> i32 {
    (px * ts + ART_TILE / 2) / ART_TILE
}

/// The sorted pass (4.4): the standing props of `props`, the `markers` (their tile rects) and the party (its drawn
/// rect), back to front by feet, then group (prop, marker, party), then left edge, then push order.
pub fn sorted(props: &[PropDraw], markers: &[Rect], party: Rect) -> Vec<Item> {
    let mut list = DepthList::new();
    for (i, p) in props.iter().enumerate().filter(|(_, p)| !p.ground) {
        list.push(p.foot_y as f32, GROUP_PROP, p.rect.x as f32, Item::Prop(i));
    }
    for (i, r) in markers.iter().enumerate() {
        list.push((r.y + r.h) as f32, GROUP_MARKER, r.x as f32, Item::Marker(i));
    }
    list.push((party.y + party.h) as f32, GROUP_PARTY, party.x as f32, Item::Party);
    list.into_sorted()
}

/// 4.5: a standing prop drawn after the party (in `order`) whose drawn rect meets the party's rect hides it.
pub fn party_hidden(props: &[PropDraw], order: &[Item], party: Rect) -> bool {
    let Some(at) = order.iter().position(|i| *i == Item::Party) else { return false };
    order[at + 1..].iter().any(|i| match i {
        Item::Prop(n) => {
            let m = props[*n].rect.intersect(party);
            m.w > 0 && m.h > 0
        }
        _ => false,
    })
}

/// PROPS1 section 8 (L2): placement, culling, sort order and the party outline over a test template (not the shipped
/// maps): `keep_hall`, 10 x 6 at (4, 3) with 320 x 256 px art, as the spec's example.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TS: i32 = 16;
    const O: (i32, i32) = (7, 50);

    fn template(props: serde_json::Value) -> Dict {
        json::dict(&json!({"rows": [], "props": props})).clone()
    }

    fn hall(flip: bool, variant: i64) -> serde_json::Value {
        json!({"id": "keep_hall", "art": "keep_hall", "x": 4, "y": 3, "w": 10, "h": 6, "layer": "sort", "variant": variant, "flip": flip})
    }

    fn everywhere() -> Rect {
        Rect::new(-10_000, -10_000, 20_000, 20_000)
    }

    /// Art of `w` x `h` px for every prop (the first folder's `v0.png`).
    fn with_art(w: i32, h: i32) -> impl FnMut(&[&str], Option<u32>, i32, i32) -> Picked {
        move |dirs, _, _, _| Some((format!("{}/v0.png", dirs[0]), (w, h)))
    }

    fn no_art(_: &[&str], _: Option<u32>, _: i32, _: i32) -> Picked {
        None
    }

    fn tile(x: i32, y: i32) -> Rect {
        Rect::new(O.0 + x * TS, O.1 + y * TS, TS, TS)
    }

    #[test]
    fn a_prop_stands_bottom_centred_on_its_footprint() {
        let t = template(json!([hall(false, -1)]));
        let lineage = vec!["ashlands".to_string()];
        let mut asked = Vec::new();
        let drawn = layout(&t, &lineage, O, TS, everywhere(), |dirs: &[&str], v, x, y| {
            asked.push((dirs.iter().map(|d| d.to_string()).collect::<Vec<_>>(), v, x, y));
            Some((format!("{}/v0.png", dirs[0]), (320, 256)))
        });
        assert_eq!(asked, vec![(vec!["props/ashlands/keep_hall".to_string(), "props/common/keep_hall".to_string()], None, 4, 3)], "the chain, a pick by position");
        assert_eq!(drawn.len(), 1);
        let p = &drawn[0];
        assert_eq!(p.rect, Rect::new(O.0 + 64, O.1 + 16, 160, 128), "the spec's example rect");
        assert_eq!(p.foot, Rect::new(O.0 + 4 * TS, O.1 + 3 * TS, 10 * TS, 6 * TS));
        assert_eq!(p.foot_y, O.1 + 9 * TS, "the footprint's bottom edge");
        assert_eq!(p.file.as_deref(), Some("props/ashlands/keep_hall/v0.png"));
        assert!(!p.ground && !p.flip);
        // flip mirrors the image around the same centre: the rect stays
        let flipped = layout(&template(json!([hall(true, 2)])), &lineage, O, TS, everywhere(), |_: &[&str], v, _, _| {
            assert_eq!(v, Some(2), "an explicit variant goes to the pick");
            Some(("x/v2.png".to_string(), (320, 256)))
        });
        assert!(flipped[0].flip);
        assert_eq!(flipped[0].rect, p.rect);
        // an odd art width stays centred within a pixel; a narrow art is centred, not left-aligned
        let narrow = layout(&t, &lineage, O, TS, everywhere(), with_art(32 * 10 - 64, 256));
        assert_eq!((narrow[0].rect.x, narrow[0].rect.w), (O.0 + 64 + 16, 160 - 32));
        // no art: the footprint is the drawn rect (the plain box, 4.6)
        let bare = layout(&t, &lineage, O, TS, everywhere(), no_art);
        assert_eq!((bare[0].rect, bare[0].file.clone()), (bare[0].foot, None));
    }

    #[test]
    fn props_are_culled_by_their_drawn_rect() {
        let t = template(json!([hall(false, -1)]));
        let lineage = vec!["ashlands".to_string()];
        // the area ends between the roof's top (row 1) and the footprint's top (row 3)
        let above = Rect::new(-10_000, -10_000, 20_000, 10_000 + O.1 + 40);
        let kept = layout(&t, &lineage, O, TS, above, with_art(320, 256));
        assert_eq!(kept.len(), 1, "the overhang meets the area although the footprint does not");
        assert!(kept[0].foot.intersect(above).h == 0);
        assert!(layout(&t, &lineage, O, TS, above, no_art).is_empty(), "without art the footprint is all there is");
        let right = Rect::new(O.0 + 64 + 160, -10_000, 10_000, 20_000);
        assert!(layout(&t, &lineage, O, TS, right, with_art(320, 256)).is_empty(), "a prop left of the area is culled");
    }

    #[test]
    fn the_sorted_pass_goes_by_feet_then_group_then_left_edge() {
        let mut props = vec![hall(false, -1)];
        props.push(json!({"id": "fire", "art": "fire_pit", "x": 2, "y": 2, "w": 1, "h": 1, "layer": "ground", "variant": -1, "flip": false}));
        let drawn = layout(&template(json!(props)), &[], O, TS, everywhere(), with_art(320, 256));
        assert_eq!(drawn.len(), 2);
        // north of the hall (feet on row 3's top edge, 48 px from the origin) the party draws before it; south after it
        assert_eq!(sorted(&drawn, &[], tile(8, 2)), vec![Item::Party, Item::Prop(0)], "the ground prop is not in the sorted pass");
        assert_eq!(sorted(&drawn, &[], tile(8, 9)), vec![Item::Prop(0), Item::Party]);
        // at equal feet: the prop, then markers, then the party; markers at equal feet left to right
        let markers = [tile(20, 8), tile(15, 8)];
        assert_eq!(sorted(&drawn, &markers, tile(1, 8)), vec![Item::Prop(0), Item::Marker(1), Item::Marker(0), Item::Party]);
    }

    #[test]
    fn the_party_outline_shows_only_when_a_prop_in_front_covers_it() {
        let drawn = layout(&template(json!([hall(false, -1)])), &[], O, TS, everywhere(), with_art(320, 256));
        let hidden = |at: Rect| party_hidden(&drawn, &sorted(&drawn, &[], at), at);
        assert!(hidden(tile(8, 2)), "under the roof's overhang, north of the hall");
        assert!(!hidden(tile(24, 2)), "beside the hall, clear of its art");
        assert!(!hidden(tile(8, 9)), "south of the hall the party draws over it");
        // the plain box covers only walls, so the party (on open tiles) is never behind it
        let bare = layout(&template(json!([hall(false, -1)])), &[], O, TS, everywhere(), no_art);
        assert!(!party_hidden(&bare, &sorted(&bare, &[], tile(8, 2)), tile(8, 2)));
    }
}
