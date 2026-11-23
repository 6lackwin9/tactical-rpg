//! The region map's ground (REGIONGROUND1-L, `games/emberwake/docs/specs/REGIONGROUND1.md` section 2): glue from a map
//! template to the engine's composer. `plan` from the kit gives the layers and cells; each layer's art comes from
//! `ground_dirs` through `Art::pick` and `Art::image`, reduced once per view from its 256 px file, or the layer draws its
//! flat `colour`. Presentation only: no rule, save, walk or click reads any of it.
//!
//! Everything a job depends on is read here only when something it depends on changed, compared cheaply each frame
//! (`Seen`): the map, the view, its tile sets, the data and the art. So a frame reads no file, hashes no texture and
//! builds no job; it passes the kept key to the composer's cache, which composes on a miss.

use crate::mapview::{MapGeom, View};
use crate::screens::Cx;
use aegis_core::json::{self, Dict};
use aegis_ui::canvas::{Image, Rect, Rgba};
use aegis_ui::ground::{self as composer, GroundJob, GroundKey, GroundLayer, PlacedPatch, ART_PX_MAX, CELL_MAX, CELL_PX, TEX_CELLS, TILTED_CELL};
use aegis_ui::painter::Painter;
use emberwake_content::ground::{plan, GroundPlan};
use emberwake_world::Data;
use serde_json::{json, Value};
use std::sync::Arc;

/// The region map's largest art cell: 32 (the sprites' own density); the composer's `CELL_MAX` is higher for the battle
/// board (DIORAMA5 R1), and a region map keeps composing at most this.
pub const REGION_CELL_MAX: i32 = 32;

/// The composer's art px per cell in a view at `q` quarters of zoom: one art px is one logical px, so these equal
/// `MapGeom::cell_size`, up to `REGION_CELL_MAX` (past it, the picture at that size stretches on the blit).
fn cell_of(view: View, q: i32) -> (i32, i32) {
    let (w, h) = match view {
        View::Square => (CELL_PX, CELL_PX),
        View::Tilted => TILTED_CELL,
    };
    const _: () = assert!(REGION_CELL_MAX <= CELL_MAX);
    let q = q.min(4 * REGION_CELL_MAX / CELL_PX);
    (w * q / 4, h * q / 4)
}

/// What a job was made from. `data` is held, not its address, so a freed catalog's address can never come back as a
/// false match.
struct Seen {
    data: Arc<Data>,
    art: u64,
    map: String,
    sets: Vec<String>,
    view: View,
    q: i32,
}

/// One map's job, owning what the composer's job borrows. A texture is a layer's reduced art; `layers` is in `plan`'s
/// order, the one fixed order, since a texture's offset comes from its place in the list.
#[derive(Clone)]
struct Picture {
    plan: GroundPlan,
    textures: Vec<Option<Image>>,
    colours: Vec<Rgba>,
    under_textures: Vec<Option<Image>>,
    under_colours: Vec<Option<Rgba>>,
    placed: Vec<PlacedPatch>,
    cell: (i32, i32),
}

impl Picture {
    fn layers(&self) -> Vec<GroundLayer<'_>> {
        self.plan
            .layers
            .iter()
            .enumerate()
            .map(|(i, def)| GroundLayer {
                order: def.order,
                tex: self.textures[i].as_ref(),
                colour: self.colours[i],
                wear: def.wear as f32,
                strays: def.strays,
                tone: def.tone as f32,
                patches: def.patches as f32,
                sunken: def.is_water(),
                patchy: def.patchy as f32,
                under_tex: self.under_textures[i].as_ref(),
                under_colour: self.under_colours[i].unwrap_or([0, 0, 0, 0]),
                clamp_water: def.is_water(),
                stone_rim: def.is_water(),
                // PR7-36 (ART-20): a 1 px dark line where two floor types meet (water keeps its stone rim)
                contact: !def.is_water(),
                marks: Vec::new(),
                marks_per_100: 0,
            })
            .collect()
    }

    fn job<'a>(&'a self, layers: &'a [GroundLayer<'a>]) -> GroundJob<'a> {
        GroundJob { seed: self.plan.seed, cols: self.plan.cols, rows: self.plan.rows, cell: self.cell, cells: &self.plan.cells, blocked: &self.plan.blocked, layers, placed: &self.placed }
    }

    fn compose(&self) -> Image {
        composer::compose(&self.job(&self.layers()))
    }

    fn key(&self, map: &str) -> GroundKey {
        GroundKey::of(map, &self.job(&self.layers()))
    }
}

/// A row's flat colour: `#rrggbb` only (the row check's form), so a stand-in is always opaque.
fn flat_colour(text: &str) -> Option<Rgba> {
    if text.len() != 7 {
        return None;
    }
    aegis_ui::theme::parse_colour(text)
}

/// The picture of template `t` in `view` at `q` quarters of zoom; None (the map draws as if it had no ground) for what `plan` refuses, for a row
/// the checks refuse by its colour, or for a map too large for the composer, so a map never loses its tiles for nothing.
fn picture(cx: &mut Cx, t: &Dict, sets: &[String], view: View, q: i32) -> Option<Picture> {
    let d = cx.data.clone();
    let plan = plan(t, &d.cat.grounds)?;
    let cell = cell_of(view, q);
    if (plan.cols * cell.0) as i64 * (plan.rows * cell.1) as i64 > ART_PX_MAX {
        return None;
    }
    let colours: Vec<Rgba> = plan.layers.iter().map(|l| flat_colour(&l.colour)).collect::<Option<_>>()?;
    let (tw, th) = (TEX_CELLS * cell.0, TEX_CELLS * cell.1);
    let mut load_tex = |id: &str| -> Option<Image> {
        let dirs = emberwake_content::art::ground_dirs(sets, id);
        let refs: Vec<&str> = dirs.iter().map(String::as_str).collect();
        // only `v0` is read: a ground has one texture
        let rel = cx.art.pick(&refs, Some(0), 0, 0)?;
        cx.art.image(&rel, tw, th).cloned()
    };
    let textures = plan.layers.iter().map(|l| load_tex(&l.id)).collect();
    let (mut under_textures, mut under_colours) = (Vec::with_capacity(plan.layers.len()), Vec::with_capacity(plan.layers.len()));
    for l in &plan.layers {
        let u = if l.patchy > 0.0 && !l.under.is_empty() {
            plan.layers.iter().chain(plan.under.iter()).find(|x| x.id == l.under).or_else(|| d.cat.grounds.get(&l.under))
        } else {
            None
        };
        under_textures.push(u.and_then(|x| load_tex(&x.id)));
        under_colours.push(u.and_then(|x| flat_colour(&x.colour)));
    }
    let placed = plan.placed.iter().map(|p| PlacedPatch { x: p.x, y: p.y, w: p.w, h: p.h, strength: p.strength }).collect();
    Some(Picture { plan, textures, colours, under_textures, under_colours, placed, cell })
}

/// A region map screen's ground: the kept job and key.
#[derive(Default)]
pub struct MapGround {
    seen: Option<Seen>,
    job: Option<(Picture, GroundKey)>,
    /// Times a job was made (tests: never per frame).
    #[cfg(test)]
    pub(crate) builds: u32,
}

impl MapGround {
    /// Makes the job and key if something they depend on changed since the last frame; else only compares.
    pub fn sync(&mut self, cx: &mut Cx, t: &Dict, sets: &[String], view: View, q: i32) {
        let map = json::s(t, "id");
        let same = self.seen.as_ref().is_some_and(|s| s.view == view && s.q == q && s.art == cx.art.generation() && Arc::ptr_eq(&s.data, &cx.data) && s.map == map && s.sets == sets);
        if same {
            return;
        }
        // ENGINE-54: a warm draw whose textures are not all loaded yet keeps no job (a later slice makes it)
        let left = cx.art.left();
        let pic = picture(cx, t, sets, view, q);
        if cx.art.left() != left {
            self.job = None;
            self.seen = None;
            return;
        }
        self.job = pic.map(|p| {
            let key = p.key(&map);
            (p, key)
        });
        self.seen = Some(Seen { data: cx.data.clone(), art: cx.art.generation(), map, sets: sets.to_vec(), view, q });
        #[cfg(test)]
        {
            self.builds += 1;
        }
    }

    /// Draws the map's ground over its whole rect (the composer copies what the clip shows). False: this map has none.
    pub fn draw(&self, p: &mut Painter, geo: &MapGeom) -> bool {
        let Some((picture, key)) = &self.job else {
            return false;
        };
        let (cw, ch) = geo.cell_size();
        let r = Rect::new(geo.ox, geo.oy, picture.plan.cols * cw, picture.plan.rows * ch);
        // ENGINE-54 (WARM-SPREAD1): a warm draw composes the map's ground on a thread
        if composer::deferring() && !composer::held(key) {
            let picture = picture.clone();
            composer::compose_later(key.clone(), move || picture.compose());
            return true;
        }
        composer::draw(p, key.clone(), r, || picture.compose());
        true
    }

    /// ENGINE-50 (MAP-FRAME1): whether this map has a composed ground (`draw` draws it).
    pub fn has(&self) -> bool {
        self.job.is_some()
    }

    /// The agent link's `state.ground`: `{"layers": n}`, or null on a map without ground.
    pub fn state(&self) -> Value {
        self.job.as_ref().map_or(Value::Null, |(p, _)| json!({"layers": p.plan.layers.len()}))
    }

    /// The key in force (tests compare what recomposes).
    #[cfg(test)]
    pub(crate) fn key(&self) -> Option<&GroundKey> {
        self.job.as_ref().map(|(_, k)| k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::game_cx;
    use crate::mapview::{self, CELL_W};
    use aegis_ui::art::{load_png, Art};
    use serde_json::Map;

    fn template(cols: usize, rows: usize) -> Dict {
        let line = |c: char| Value::Array(vec![Value::String(c.to_string().repeat(cols)); rows]);
        let mut t = Map::new();
        t.insert("id".into(), json!("zz_map"));
        t.insert("rows".into(), line('.'));
        t.insert("ground".into(), line('a'));
        t.insert("ground_legend".into(), json!({"a": "zz_ground"}));
        t
    }

    fn cx_with(colour: &str) -> Cx {
        let mut cx = game_cx();
        let mut data = (*cx.app.data).clone();
        let row = emberwake_data::GroundDef::from_obj(json!({"id": "zz_ground", "name": "G", "order": 1, "colour": colour}).as_object().unwrap());
        data.edit_cat(|cat| {
            cat.grounds.insert(row.id.clone(), row);
        });
        cx.app.data = Arc::new(data);
        cx
    }

    /// Scratch art: `tilesets/common/ground/zz_ground/v0.png`, 256 px of busy noise, so two ways of reducing it differ.
    fn noisy_art(tag: &str) -> (aegis_core::scratch::Scratch, std::path::PathBuf) {
        let root = aegis_core::scratch::Scratch::vacant(&format!("emberwake_ground_tex_{tag}"));
        let file = root.join("tilesets/common/ground/zz_ground/v0.png");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&file).unwrap()), 256, 256);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let px: Vec<u8> = (0..256u64 * 256).flat_map(|i| { let b = aegis_ui::ground::splitmix64(i).to_le_bytes(); [b[0], b[1], b[2], 255] }).collect();
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        (root, file)
    }

    #[test]
    fn the_composers_cell_is_the_views_cell() {
        for (name, _) in mapview::VIEWS {
            let view = View::of_setting(&json!(name));
            assert_eq!(cell_of(view, 4), (CELL_W, view.cell_h()), "{name}: one art px is one logical px");
            for z in mapview::zoom_steps() {
                let q = mapview::zoom_of(&json!(z));
                let (art, cell) = (cell_of(view, q), mapview::cell_of(view, q));
                assert!(art.0 <= REGION_CELL_MAX && art.1 <= REGION_CELL_MAX, "{name} {z}: the composer takes it");
                assert_eq!(art.0 * cell.1, art.1 * cell.0, "{name} {z}: the same shape as the cell");
                if q <= 8 {
                    assert_eq!(art, cell, "{name} {z}: one art px is one logical px up to 2x");
                }
            }
        }
    }

    /// Spec 4.2: each view's texture is one box step from the 256 px file, never a squash of the other's reduction.
    #[test]
    fn each_view_reduces_the_file_once_to_its_own_size() {
        let (root, file) = noisy_art("reduce");
        let master = load_png(&file).expect("the file");
        let mut cx = cx_with("#102030");
        cx.art = Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
        let t = template(4, 3);
        for (view, want) in [(View::Square, (64, 64)), (View::Tilted, (64, 48))] {
            let p = picture(&mut cx, &t, &["common".to_string()], view, 4).expect("a picture");
            let tex = p.textures[0].as_ref().expect("the texture");
            assert_eq!((tex.w, tex.h), want);
            assert!(*tex == master.downsample(want.0, want.1), "{view:?}: one step from the master");
        }
        assert!(master.downsample(64, 48) != master.downsample(64, 64).downsample(64, 48), "the fixture tells the two apart");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// REGIONVIEW1's measurement on the shipped keep: making the job, composing and expanding at the zoom's steps at 1080p.
    #[test]
    #[ignore = "a measurement; run by hand in a release build"]
    fn the_keeps_ground_time_at_each_zoom() {
        let mut cx = game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let loc = d.merged_entries(&world, "locations").into_iter().find(|l| json::s(l, "map") == "ashlands_warden_keep").expect("the keep");
        let t = emberwake_world::explore::template_of(&d, loc).clone();
        let mut lineage = d.lineage(&world);
        lineage.reverse();
        let sets = emberwake_content::art::region_sets(&lineage, &json::s(loc, "tileset"));
        for view in [View::Square, View::Tilted] {
            for z in [1.0, 2.0, 3.0] {
                let q = mapview::zoom_of(&json!(z));
                let t0 = aegis_core::clock::Instant::now();
                let p = picture(&mut cx, &t, &sets, view, q).expect("a picture");
                let job = t0.elapsed();
                let t1 = aegis_core::clock::Instant::now();
                let key = p.key("keep");
                let keyed = t1.elapsed();
                let t2 = aegis_core::clock::Instant::now();
                let art = p.compose();
                let composed = t2.elapsed();
                let (cw, ch) = mapview::cell_of(view, q);
                let (w, h) = ((p.plan.cols * cw) as f32 * 3.0, (p.plan.rows * ch) as f32 * 3.0);
                let bytes = (w * h * 4.0) as usize;
                let t3 = aegis_core::clock::Instant::now();
                let copy = if bytes <= composer::PHYSICAL_MAX { composer::expand(&art, w as i32, h as i32).len() } else { 0 };
                let expanded = t3.elapsed();
                println!("{view:?} {z}x: art {}x{} ({} MB), job {:.1} ms (first read of the art sizes), key {:.1} ms, compose {:.1} ms, physical copy {} MB in {:.1} ms{}; {:?}", art.w, art.h, art.px.len() / 1_000_000, job.as_secs_f64() * 1e3, keyed.as_secs_f64() * 1e3, composed.as_secs_f64() * 1e3, bytes / 1_000_000, expanded.as_secs_f64() * 1e3, if copy == 0 { " (past the limit: not kept)" } else { "" }, key.cell);
            }
        }
    }

    /// ENGINE-52 (COMPOSE-PAR1): two shipped maps' grounds (both views) compose to the same pixels on one thread and on
    /// many (the rows in bands).
    #[test]
    fn a_maps_ground_is_the_same_on_one_thread_and_many() {
        let mut cx = game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let d = cx.data.clone();
        for (world, map) in [("ashlands", "ashlands_warden_keep"), ("sunk_ward", "sunk_ward_junction_7")] {
            let loc = d.merged_entries(world, "locations").into_iter().find(|l| json::s(l, "map") == map).expect("the map's location");
            let t = emberwake_world::explore::template_of(&d, loc).clone();
            let mut lineage = d.lineage(world);
            lineage.reverse();
            let sets = emberwake_content::art::region_sets(&lineage, &json::s(loc, "tileset"));
            for view in [View::Square, View::Tilted] {
                let p = picture(&mut cx, &t, &sets, view, mapview::zoom_of(&json!(1.0))).expect("a picture");
                let one = composer::with_compose_threads(1, || p.compose());
                let many = composer::with_compose_threads(composer::COMPOSE_THREADS, || p.compose());
                assert!(one.w > 0 && one.h > 0, "{map} {view:?}: a ground");
                assert!(one == many, "{map} {view:?}: the same pixels");
            }
        }
    }

    /// ENGINE-53 (SWITCH-AHEAD1), ENGINE-54 (WARM-SPREAD1): a map's ground drawn in a warm slice is composed on a
    /// thread, never on the frame: its textures are decoded off the frame first (the slice keeps no job made without
    /// them), then the job goes to `compose_later`; once kept it is the ground a real draw finds (the same key and
    /// pixels), so that draw composes nothing.
    #[test]
    fn a_warm_draw_composes_the_maps_ground_on_a_thread() {
        let mut cx = game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let d = cx.data.clone();
        let (world, map) = ("sunk_ward", "sunk_ward_junction_7");
        let loc = d.merged_entries(world, "locations").into_iter().find(|l| json::s(l, "map") == map).expect("the map's location");
        let t = emberwake_world::explore::template_of(&d, loc).clone();
        let mut lineage = d.lineage(world);
        lineage.reverse();
        let sets = emberwake_content::art::region_sets(&lineage, &json::s(loc, "tileset"));
        let (view, q) = (View::Square, mapview::zoom_of(&json!(1.0)));
        let geo = MapGeom::default();
        let mut g = crate::hud::hud_tests::gfx();
        let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        let composes = composer::ground_composes();
        let mut warm = MapGround::default();
        let t0 = std::time::Instant::now();
        let key = loop {
            cx.art.warm_poll();
            composer::poll_later();
            cx.art.defer(std::time::Duration::from_millis(4));
            composer::set_deferring(true);
            warm.sync(&mut cx, &t, &sets, view, q);
            warm.draw(&mut p, &geo);
            let left = cx.art.undefer() + composer::left_later();
            composer::set_deferring(false);
            if left == 0 {
                break warm.key().cloned().expect("a key");
            }
            assert!(t0.elapsed().as_secs() < 60, "the warm draws finish");
        };
        assert!(composer::held(&key), "kept");
        assert_eq!(composer::ground_composes(), composes, "nothing composed on the frame");
        let mut real = MapGround::default();
        real.sync(&mut cx, &t, &sets, view, q);
        assert_eq!(real.key(), Some(&key), "the map's own key");
        real.draw(&mut p, &geo);
        assert_eq!(composer::ground_composes(), composes, "the real draw finds it");
    }

    /// A row's numbers reach the composer as they are, in `plan`'s order (a texture's offset depends on it), water sunken.
    #[test]
    fn a_rows_numbers_reach_the_composer() {
        let mut cx = game_cx();
        let row = |id: &str, order: i32, kind: &str, wear: f64, strays: bool, tone: f64, patches: f64| emberwake_data::GroundDef::from_obj(json!({"id": id, "name": id, "order": order, "kind": kind, "colour": "#102030", "wear": wear, "strays": strays, "tone": tone, "patches": patches}).as_object().unwrap());
        let mut data = (*cx.app.data).clone();
        data.edit_cat(|cat| {
            for r in [row("zz_late", 80, "land", 0.1, true, 0.08, 0.2), row("zz_water", 5, "water", 0.0, false, 0.04, 0.0)] {
                cat.grounds.insert(r.id.clone(), r);
            }
        });
        cx.app.data = Arc::new(data);
        let mut t = template(2, 1);
        t.insert("rows".into(), json!([".~"]));
        t.insert("ground".into(), json!(["lw"]));
        t.insert("ground_legend".into(), json!({"l": "zz_late", "w": "zz_water"}));
        let p = picture(&mut cx, &t, &[], View::Square, 4).expect("a picture");
        let layers = p.layers();
        let got: Vec<(i32, bool, bool, bool)> = layers.iter().map(|l| (l.order, l.sunken, l.strays, l.tex.is_none())).collect();
        assert_eq!(got, [(5, true, false, true), (80, false, true, true)], "low order first");
        assert_eq!(p.plan.cells, [1, 0], "cells index that list");
        let close = |a: f32, b: f64| (a as f64 - b).abs() < 1e-6;
        assert!(close(layers[1].wear, 0.1) && close(layers[1].tone, 0.08) && close(layers[1].patches, 0.2), "the row's numbers");
        assert!(close(layers[0].wear, 0.0) && close(layers[0].tone, 0.04), "the water's");
        assert_eq!(layers[0].colour, [0x10, 0x20, 0x30, 255], "opaque");
        assert!(layers[0].clamp_water && layers[0].stone_rim && !layers[1].clamp_water && !layers[1].stone_rim);
        assert!(!layers[0].contact && layers[1].contact, "PR7-36 (ART-20): land draws the contact line, water its rim");
        assert!(close(layers[1].patchy, 0.0) && layers[1].under_tex.is_none() && layers[1].under_colour[3] == 0);
    }

    #[test]
    fn patchy_and_under_reach_composer_even_when_under_is_not_placed_on_cells() {
        let mut cx = game_cx();
        let under = emberwake_data::GroundDef::from_obj(json!({"id": "zz_under", "name": "Under", "order": 1, "kind": "land", "colour": "#304050"}).as_object().unwrap());
        let top = emberwake_data::GroundDef::from_obj(json!({"id": "zz_top", "name": "Top", "order": 20, "kind": "land", "colour": "#101214", "patchy": 0.52, "under": "zz_under"}).as_object().unwrap());
        let mut data = (*cx.app.data).clone();
        data.edit_cat(|cat| {
            cat.grounds.insert(under.id.clone(), under);
            cat.grounds.insert(top.id.clone(), top);
        });
        cx.app.data = Arc::new(data);
        let mut t = template(2, 1);
        t.insert("rows".into(), json!([".."]));
        t.insert("ground".into(), json!(["tt"]));
        t.insert("ground_legend".into(), json!({"t": "zz_top"}));
        let p = picture(&mut cx, &t, &[], View::Square, 4).expect("a picture");
        let layers = p.layers();
        assert_eq!(layers.len(), 1);
        assert!((layers[0].patchy - 0.52).abs() < 1e-6);
        assert_eq!(layers[0].under_colour, [0x30, 0x40, 0x50, 255]);
        assert!(layers[0].under_tex.is_none());
    }

    /// A row the checks would refuse, and a map past the composer's limit, keep the map's tiles; a limit-size map plans.
    #[test]
    fn what_the_composer_cannot_draw_is_refused_not_cut() {
        for colour in ["", "102030", "#10203", "#1020300", "#10203040", "#10203g"] {
            let mut cx = cx_with(colour);
            assert!(picture(&mut cx, &template(4, 3), &[], View::Square, 4).is_none(), "'{colour}'");
        }
        let mut cx = cx_with("#102030");
        let side = (ART_PX_MAX as f64).sqrt() as usize / CELL_PX as usize;
        assert!(picture(&mut cx, &template(side, side), &[], View::Square, 4).is_some(), "at the limit");
        assert!(picture(&mut cx, &template(side + 1, side + 1), &[], View::Square, 4).is_none(), "past it");
    }
}
