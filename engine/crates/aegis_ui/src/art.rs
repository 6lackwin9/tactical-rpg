//! Art over the asset layers (ASSETORG2: the game's assets, its kit's, the engine's; `aegis_core::assets::Layers`),
//! read-only, scaled to pixel size and cached. A file is named by its path relative to a layer root (`image`), a
//! body by its key (`<class>` or `<class>/<variant>`: a sheet of clips, `body_frame`, `body_token`, with an optional
//! palette), a tile by its set chain (`tile_in`, variants by the files present). The ASSETORG1 tree
//! (`art/<category>/<id>/<variant>.png` in the game root) is still read by `get` until the game moves its files.

use crate::canvas::Image;
use aegis_core::assets::{self, Clips, Layers};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A body's sheet (palette applied) and its clips.
struct Body {
    sheet: Image,
    clips: Clips,
}

pub struct Art {
    layers: Layers,
    cache: HashMap<(String, String, String, i32), Option<Image>>,
    /// (rel, w, h) -> the scaled image.
    files: HashMap<(String, i32, i32), Option<Image>>,
    /// (body key, palette id) -> the body.
    bodies: HashMap<(String, String), Option<Body>>,
    /// (body key, palette id, clip, frame, size) -> the scaled frame.
    frames: HashMap<(String, String, String, u32, i32), Option<Image>>,
}

/// Decodes a PNG into RGBA (palette, grey and RGB inputs are expanded).
pub fn load_png(path: &Path) -> Option<Image> {
    let file = std::fs::File::open(path).ok()?;
    let mut dec = png::Decoder::new(file);
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as i32, info.height as i32);
    let src = &buf[..info.buffer_size()];
    let px: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => src.to_vec(),
        png::ColorType::Rgb => src.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => src.chunks_exact(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
        png::ColorType::Grayscale => src.iter().flat_map(|g| [*g, *g, *g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    Some(Image { w, h, px })
}

impl Art {
    /// `root`: the game folder (holds game.json and assets/): its asset layers; None draws fallbacks only.
    pub fn new(root: Option<PathBuf>) -> Self {
        Self::layered(root.map(|r| Layers::for_game(&r)).unwrap_or_default())
    }

    pub fn layered(layers: Layers) -> Self {
        Self { layers, cache: HashMap::new(), files: HashMap::new(), bodies: HashMap::new(), frames: HashMap::new() }
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    /// The ASSETORG1 path of `variant` of thing `id` in `category`, relative to a layer: art/<category>/<id>/<variant>.png.
    pub fn rel(category: &str, id: &str, variant: &str) -> String {
        format!("art/{category}/{id}/{variant}.png")
    }

    /// The file of `variant` of thing `id` in `category` under the game folder `root` (ASSETORG1 tree).
    pub fn path(root: &Path, category: &str, id: &str, variant: &str) -> PathBuf {
        root.join("assets").join(Self::rel(category, id, variant))
    }

    /// `variant` of `id` in `category` (e.g. "unit", "warrior", "token") at `size` x `size` pixels (ASSETORG1 tree).
    pub fn get(&mut self, category: &str, id: &str, variant: &str, size: i32) -> Option<&Image> {
        let key = (category.to_string(), id.to_string(), variant.to_string(), size);
        if !self.cache.contains_key(&key) {
            let rel = Self::rel(category, id, variant);
            let path = self.layers.find(&rel);
            let img = path.as_ref().and_then(|p| load_png(p)).map(|i| i.downsample(size, size));
            if img.is_none() && !self.layers.roots.is_empty() {
                // The caller draws its fallback; the playtest trail notes which art was missing.
                crate::playtest::report_asset(category, id, variant, if path.is_some() { "load_failed" } else { "missing_file" });
            }
            self.cache.insert(key.clone(), img);
        }
        self.cache.get(&key).and_then(|i| i.as_ref())
    }

    /// The file at `rel` (in the first layer holding it) downsampled to `w` x `h`.
    pub fn image(&mut self, rel: &str, w: i32, h: i32) -> Option<&Image> {
        let key = (rel.to_string(), w, h);
        if !self.files.contains_key(&key) {
            let path = self.layers.find(rel);
            let img = path.as_ref().and_then(|p| load_png(p)).map(|i| i.downsample(w, h));
            if img.is_none() && !self.layers.roots.is_empty() {
                crate::playtest::report_asset("file", rel, "", if path.is_some() { "load_failed" } else { "missing_file" });
            }
            self.files.insert(key.clone(), img);
        }
        self.files.get(&key).and_then(|i| i.as_ref())
    }

    /// The first of `rels` a layer holds (a lookup chain), downsampled to `w` x `h`; no missing-art report.
    pub fn first(&mut self, rels: &[String], w: i32, h: i32) -> Option<&Image> {
        let rel = rels.iter().find(|r| self.layers.has(r))?.clone();
        self.image(&rel, w, h)
    }

    /// Body `key` with `palette` ("" = none): its sheet with the palette's slots swapped and its clips.
    fn body(&mut self, key: &str, palette: &str) -> Option<&Body> {
        let k = (key.to_string(), palette.to_string());
        if !self.bodies.contains_key(&k) {
            let body = self.load_body(key, palette);
            if body.is_none() && !self.layers.roots.is_empty() {
                crate::playtest::report_asset("body", key, palette, "missing_file");
            }
            self.bodies.insert(k.clone(), body);
        }
        self.bodies.get(&k).and_then(|b| b.as_ref())
    }

    fn load_body(&self, key: &str, palette: &str) -> Option<Body> {
        let clips = Clips::parse(&assets::read_json(&self.layers, &assets::body_file(key, "clips.json"))?).ok()?;
        let mut sheet = load_png(&self.layers.find(&assets::body_file(key, "sheet.png"))?)?;
        if !clips.sheet_problem(sheet.w as u32, sheet.h as u32).is_empty() {
            return None;
        }
        if !palette.is_empty() {
            if let Some(slots) = assets::read_json(&self.layers, &assets::body_file(key, &format!("palettes/{palette}.json"))).and_then(|v| assets::parse_palette(&v).ok()) {
                let pairs: Vec<([u8; 3], [u8; 3])> = slots.iter().filter_map(|(slot, to)| clips.palette.get(slot).map(|from| (*from, *to))).collect();
                sheet.swap_colours(&pairs);
            }
        }
        Some(Body { sheet, clips })
    }

    /// Whether body `key` loads (its clips.json is valid and its sheet big enough).
    pub fn has_body(&mut self, key: &str) -> bool {
        self.body(key, "").is_some()
    }

    /// Frame `n` of `clip` of body `key` at full size (`None`: no such body, clip or frame).
    pub fn body_frame(&mut self, key: &str, palette: &str, clip: &str, n: u32) -> Option<Image> {
        let b = self.body(key, palette)?;
        let row = b.clips.row(clip)?;
        if n >= b.clips.clips[row].frames {
            return None;
        }
        let (w, h) = (b.clips.frame.0 as i32, b.clips.frame.1 as i32);
        Some(b.sheet.crop(n as i32 * w, row as i32 * h, w, h))
    }

    /// Frame `n` of `clip` fitted to a `size` square (spec 4.1): scaled by k = floor(size / max(w, h)) with nearest
    /// pixels (downsampled when k is 0), centred across and standing on the bottom edge.
    pub fn body_clip(&mut self, key: &str, palette: &str, clip: &str, n: u32, size: i32) -> Option<&Image> {
        let k = (key.to_string(), palette.to_string(), clip.to_string(), n, size);
        if !self.frames.contains_key(&k) {
            let img = self.body_frame(key, palette, clip, n).map(|f| {
                let scale = size / f.w.max(f.h).max(1);
                let fitted = if scale >= 1 {
                    f.scale_nearest(scale)
                } else {
                    let m = f.w.max(f.h).max(1);
                    f.downsample((f.w * size / m).max(1), (f.h * size / m).max(1))
                };
                fitted.on_square(size)
            });
            self.frames.insert(k.clone(), img);
        }
        self.frames.get(&k).and_then(|i| i.as_ref())
    }

    /// A body's token: its `idle` clip's first frame in a `size` square.
    pub fn body_token(&mut self, key: &str, palette: &str, size: i32) -> Option<&Image> {
        self.body_clip(key, palette, "idle", 0, size)
    }

    /// Tile `tile` from the first set of `sets` a layer holds (`tilesets/<set>/<tile>/v0.png`); the variant is
    /// (x x 7 + y x 13) mod n over that layer's v0..v(n-1).
    pub fn tile_in(&mut self, sets: &[&str], tile: &str, x: i32, y: i32, size: i32) -> Option<&Image> {
        let (dir, n) = sets.iter().map(|s| format!("tilesets/{s}/{tile}")).map(|d| (d.clone(), self.layers.variants(&d))).find(|(_, n)| *n > 0)?;
        let v = (x * 7 + y * 13).rem_euclid(n as i32);
        self.image(&format!("{dir}/v{v}.png"), size, size)
    }

    /// A unit's token (unit/<art>/token).
    pub fn token(&mut self, art: &str, size: i32) -> Option<&Image> {
        self.get("unit", art, "token", size)
    }

    /// A unit's portrait (unit/<art>/portrait).
    pub fn portrait(&mut self, art: &str, size: i32) -> Option<&Image> {
        self.get("unit", art, "portrait", size)
    }

    /// Terrain tile `id` (tile/<id>/v<n>, the variant chosen by the cell, like TileArt).
    pub fn tile(&mut self, id: &str, x: i32, y: i32, size: i32) -> Option<&Image> {
        let v = ((x * 7 + y * 13) % 4 + 4) % 4;
        self.get("tile", id, &format!("v{v}"), size)
    }
}

/// The project folder above `start` (the first ancestor holding assets/art; the ASSETORG1 tree).
pub fn find_project(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("assets/art").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = std::fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(px).unwrap();
    }

    /// A 2-frame idle row and a 1-frame walk row of 8x8 frames: idle frame 0 red with a gold top-left pixel.
    fn body_tree(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("aegis_art_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (w, h) = (16u32, 16u32);
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..8 {
            for x in 0..8 {
                let i = ((y * w + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[0xa8, 0x30, 0x28, 255]);
            }
        }
        px[0..4].copy_from_slice(&[0xe0, 0xb4, 0x3c, 255]);
        let dir = root.join("units/knight/base");
        write_png(&dir.join("sheet.png"), w, h, &px);
        std::fs::write(dir.join("clips.json"), r##"{"frame": [8, 8], "palette": {"main": "#a83028", "trim": "#e0b43c"},
            "clips": [{"id": "idle", "frames": 2, "ms": [100, 100]}, {"id": "walk", "frames": 1, "ms": [100]}]}"##).unwrap();
        std::fs::create_dir_all(dir.join("palettes")).unwrap();
        std::fs::write(dir.join("palettes/blue.json"), r##"{"slots": {"main": "#2040c0"}}"##).unwrap();
        root
    }

    #[test]
    fn a_body_gives_frames_a_token_and_palette_swaps() {
        let root = body_tree("body");
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        let f = art.body_frame("knight", "", "idle", 0).expect("idle frame 0");
        assert_eq!((f.w, f.h, &f.px[0..4], &f.px[4..8]), (8, 8, &[0xe0, 0xb4, 0x3c, 255][..], &[0xa8, 0x30, 0x28, 255][..]));
        assert!(art.body_frame("knight", "", "idle", 2).is_none() && art.body_frame("knight", "", "run", 0).is_none());
        let blue = art.body_frame("knight", "blue", "idle", 0).unwrap();
        assert_eq!((&blue.px[0..4], &blue.px[4..8]), (&[0xe0, 0xb4, 0x3c, 255][..], &[0x20, 0x40, 0xc0, 255][..]), "only the main slot's colour changes");
        let t = art.body_token("knight", "", 20).unwrap().clone();
        assert_eq!((t.w, t.h), (20, 20));
        // k = 20 / 8 = 2: a 16x16 frame centred across (x 2..18) on the bottom (y 4..20)
        let at = |x: i32, y: i32| &t.px[((y * 20 + x) * 4) as usize..((y * 20 + x) * 4 + 4) as usize];
        assert_eq!((at(0, 19)[3], at(2, 4), at(3, 5), at(10, 3)[3]), (0, &[0xe0, 0xb4, 0x3c, 255][..], &[0xe0, 0xb4, 0x3c, 255][..], 0));
        assert!(art.has_body("knight") && !art.has_body("rogue"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn tiles_come_from_the_first_set_with_files_and_vary_by_their_count() {
        let root = std::env::temp_dir().join(format!("aegis_art_tiles_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (set, n) in [("common", 3), ("world_a", 0)] {
            for v in 0..n {
                write_png(&root.join(format!("tilesets/{set}/floor/v{v}.png")), 1, 1, &[v as u8, 0, 0, 255]);
            }
        }
        write_png(&root.join("tilesets/world_a/wall/v0.png"), 1, 1, &[9, 9, 9, 255]);
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        let red = |art: &mut Art, x: i32| art.tile_in(&["world_a", "common"], "floor", x, 0, 1).map(|i| i.px[0]);
        assert_eq!((red(&mut art, 0), red(&mut art, 1), red(&mut art, 2)), (Some(0), Some(1), Some(2)), "(x * 7) mod 3 over common's three");
        assert_eq!(art.tile_in(&["world_a", "common"], "wall", 5, 5, 1).map(|i| i.px[0]), Some(9), "the world's own set first");
        assert!(art.tile_in(&["world_a", "common"], "lava", 0, 0, 1).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }
}
