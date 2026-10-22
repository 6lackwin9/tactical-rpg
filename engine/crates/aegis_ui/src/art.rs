//! Art over the asset layers (ASSETORG2: the game's assets, its kit's, the engine's; `aegis_core::assets::Layers`),
//! read-only, scaled to pixel size and cached. A file is named by its path relative to a layer root (`image`), a
//! body by its key (`<class>` or `<class>/<variant>`: a sheet of clips, `body_frame`, `body_token`, with an optional
//! palette), a tile by its set chain (`tile_in`, variants by the files present). The game names the files through
//! its own lookup chains (ASSETORG2 section 4). SPRITE1-L: a body's clips play by time (`played` over an
//! `aegis_core::assets::ClipPlayer`), a frame's silhouette outline is cached (`body_outline`, highlights) and a body
//! is checked frame by frame (`body_problems`); `played`, `body_outline` and `body_clip_flipped` take a `flip` to
//! show a frame mirrored (a sheet facing right drawn facing left). PROPS1-E: a file's own size (`size_of`), mirrored images
//! (`image_mirrored`) and a variant picked over a chain of folders (`varied`, which `tile_in` wraps). DIR4-E: a body
//! played for a four-way `Facing` (`played_facing`: front and back rows of the sheet, side rows mirrored for left).

use crate::canvas::{Image, Rgba, OUTLINE_ALPHA};
use aegis_core::assets::{self, ClipFrame, ClipPlayer, Clips, Facing, Layers};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A body's sheet (palette applied) and its clips.
struct Body {
    sheet: Image,
    clips: Clips,
}

/// (body key, palette id, clip, frame, size, flipped) of a cached fitted frame.
type FrameKey = (String, String, String, u32, i32, bool);
/// (body key, palette id, clip, frame, size, colour, flipped) of a cached outline.
type OutlineKey = (String, String, String, u32, i32, Rgba, bool);

pub struct Art {
    layers: Layers,
    /// (rel, w, h) -> the scaled image.
    files: HashMap<(String, i32, i32), Option<Image>>,
    /// (body key, palette id) -> the body.
    bodies: HashMap<(String, String), Option<Body>>,
    /// (body key, palette id, clip, frame, size, flipped) -> the scaled frame (mirrored left to right when flipped).
    frames: HashMap<FrameKey, Option<Image>>,
    /// (body key, palette id, clip, frame, size, colour, flipped) -> the frame's silhouette outline.
    outlines: HashMap<OutlineKey, Option<Image>>,
    /// (rel, w, h) -> the scaled image mirrored left to right.
    mirrors: HashMap<(String, i32, i32), Option<Image>>,
    /// rel -> the PNG's own size.
    sizes: HashMap<String, Option<(i32, i32)>>,
    /// rel -> whether a layer holds it, and tile folder -> its variant count (asked every frame, looked up once).
    present: HashMap<String, bool>,
    variant_counts: HashMap<String, usize>,
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
        Self { layers, files: HashMap::new(), bodies: HashMap::new(), frames: HashMap::new(), outlines: HashMap::new(), mirrors: HashMap::new(), sizes: HashMap::new(), present: HashMap::new(), variant_counts: HashMap::new() }
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    /// Whether a layer holds `rel` (cached: screens ask every frame).
    pub fn has(&mut self, rel: &str) -> bool {
        if let Some(&hit) = self.present.get(rel) {
            return hit;
        }
        let hit = self.layers.has(rel);
        self.present.insert(rel.to_string(), hit);
        hit
    }

    fn variants(&mut self, dir: &str) -> usize {
        if let Some(&n) = self.variant_counts.get(dir) {
            return n;
        }
        let n = self.layers.variants(dir);
        self.variant_counts.insert(dir.to_string(), n);
        n
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

    /// `image` mirrored left to right (cached apart).
    pub fn image_mirrored(&mut self, rel: &str, w: i32, h: i32) -> Option<&Image> {
        let key = (rel.to_string(), w, h);
        if !self.mirrors.contains_key(&key) {
            let img = self.image(rel, w, h).map(|i| i.mirrored());
            self.mirrors.insert(key.clone(), img);
        }
        self.mirrors.get(&key).and_then(|i| i.as_ref())
    }

    /// The pixel size of the PNG at `rel` (in the first layer holding it) as stored, read from its header; cached.
    pub fn size_of(&mut self, rel: &str) -> Option<(i32, i32)> {
        if let Some(&s) = self.sizes.get(rel) {
            return s;
        }
        let size = self.layers.find(rel).and_then(|p| png_size(&p));
        self.sizes.insert(rel.to_string(), size);
        size
    }

    /// The first of `rels` a layer holds (a lookup chain), downsampled to `w` x `h`; when none is there, the trail
    /// notes the first (the caller draws its placeholder).
    pub fn first(&mut self, rels: &[String], w: i32, h: i32) -> Option<&Image> {
        match rels.iter().find(|r| self.has(r)).cloned() {
            Some(rel) => self.image(&rel, w, h),
            None => {
                if let (Some(rel), false) = (rels.first(), self.layers.roots.is_empty()) {
                    crate::playtest::report_asset("file", rel, "", "missing_file");
                }
                None
            }
        }
    }

    /// The token of the first body of `keys` that loads (a body chain), with `palette`.
    pub fn first_token(&mut self, keys: &[String], palette: &str, size: i32) -> Option<&Image> {
        let key = keys.iter().find(|k| self.has_body(k))?.clone();
        self.body_token(&key, palette, size)
    }

    /// Body `key` with `palette` ("" = none): its sheet with the palette's slots swapped and its clips.
    fn body(&mut self, key: &str, palette: &str) -> Option<&Body> {
        let k = (key.to_string(), palette.to_string());
        if !self.bodies.contains_key(&k) {
            let body = self.load_body(key, palette);
            if body.is_none() && !self.layers.roots.is_empty() {
                let there = self.layers.has(&assets::body_file(key, "clips.json"));
                crate::playtest::report_asset("body", key, palette, if there { "load_failed" } else { "missing_file" });
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
        self.body_clip_flipped(key, palette, clip, n, size, false)
    }

    /// `body_clip`, mirrored left to right when `flip` (a sheet drawn facing one way shown facing the other; the
    /// fitted square is mirrored, so the figure stays on its spot). Cached apart.
    pub fn body_clip_flipped(&mut self, key: &str, palette: &str, clip: &str, n: u32, size: i32, flip: bool) -> Option<&Image> {
        let k = (key.to_string(), palette.to_string(), clip.to_string(), n, size, flip);
        if !self.frames.contains_key(&k) {
            if flip {
                let img = self.body_clip_flipped(key, palette, clip, n, size, false).map(|f| f.mirrored());
                self.frames.insert(k.clone(), img);
                return self.frames.get(&k).and_then(|i| i.as_ref());
            }
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

    /// The first body of `keys` that loads (a body chain).
    pub fn first_body(&mut self, keys: &[String]) -> Option<String> {
        keys.iter().find(|k| self.has_body(k)).cloned()
    }

    /// Body `key`'s clips (None: the body does not load).
    pub fn clips(&mut self, key: &str) -> Option<&Clips> {
        self.body(key, "").map(|b| &b.clips)
    }

    /// Whether body `key` has its own clip `clip` (no fallback).
    pub fn has_clip(&mut self, key: &str, clip: &str) -> bool {
        self.clips(key).is_some_and(|c| c.row(clip).is_some())
    }

    /// The frame `player` shows at `now` (SPRITE1-L: the shot or the base loop, missing clips falling back to `idle`),
    /// fitted to a `size` square like `body_clip` (mirrored left to right when `flip`), and which frame it is.
    pub fn played(&mut self, key: &str, palette: &str, player: &ClipPlayer, now: u64, size: i32, flip: bool) -> Option<(ClipFrame, &Image)> {
        let f = player.frame(self.clips(key)?, now)?;
        let img = self.body_clip_flipped(key, palette, &f.clip, f.frame, size, flip)?;
        Some((f, img))
    }

    /// `played` for a four-way `facing` (DIR4 3.2): the player's frame in `facing.view` (`ClipPlayer::frame_facing`:
    /// the clip's view row, else its side row, else idle in that view, else side idle), mirrored only when it is a
    /// side row and the facing is left (`Facing::mirrors`; pass the same to `body_outline`). Cached as `played`.
    pub fn played_facing(&mut self, key: &str, palette: &str, player: &ClipPlayer, now: u64, size: i32, facing: Facing) -> Option<(ClipFrame, &Image)> {
        let f = player.frame_facing(self.clips(key)?, now, facing.view)?;
        let img = self.body_clip_flipped(key, palette, &f.clip, f.frame, size, facing.mirrors(&f))?;
        Some((f, img))
    }

    /// The silhouette outline (`Image::outline`, `col`) of frame `n` of `clip` in a `size` square; (size + 2) square,
    /// drawn one pixel up and left of the frame; of the mirrored frame when `flip` (as `played` draws it). Cached.
    #[allow(clippy::too_many_arguments)]
    pub fn body_outline(&mut self, key: &str, palette: &str, clip: &str, n: u32, size: i32, col: Rgba, flip: bool) -> Option<&Image> {
        let k = (key.to_string(), palette.to_string(), clip.to_string(), n, size, col, flip);
        if !self.outlines.contains_key(&k) {
            let img = self.body_clip_flipped(key, palette, clip, n, size, flip).map(|f| f.outline(col));
            self.outlines.insert(k.clone(), img);
        }
        self.outlines.get(&k).and_then(|i| i.as_ref())
    }

    /// What is wrong with body `key` (empty: it loads and every clip frame holds pixels): clips.json problems, a sheet
    /// too small for the clips, frames with no solid pixel (`blank_frames`).
    pub fn body_problems(&mut self, key: &str) -> Vec<String> {
        let Some(v) = assets::read_json(&self.layers, &assets::body_file(key, "clips.json")) else {
            return vec![format!("{} is missing or not JSON.", assets::body_file(key, "clips.json"))];
        };
        let clips = match Clips::parse(&v) {
            Ok(c) => c,
            Err(p) => return p,
        };
        let Some(sheet) = self.layers.find(&assets::body_file(key, "sheet.png")).and_then(|p| load_png(&p)) else {
            return vec![format!("{} is missing or not a PNG.", assets::body_file(key, "sheet.png"))];
        };
        let size = clips.sheet_problem(sheet.w as u32, sheet.h as u32);
        if !size.is_empty() {
            return vec![size];
        }
        blank_frames(&clips, &sheet).into_iter().map(|(clip, n)| format!("clip '{clip}' frame {n} is empty.")).collect()
    }

    /// The file a variant pick over `dirs` names: the first folder holding `v0.png` in any layer, then `variant`
    /// (at or above that layer's count n: `v0`) or, with None, (x x 7 + y x 13) mod n (Euclidean). None: no folder
    /// has a `v0.png`.
    pub fn pick(&mut self, dirs: &[&str], variant: Option<u32>, x: i32, y: i32) -> Option<String> {
        let (dir, n) = dirs.iter().map(|d| (*d, self.variants(d))).find(|(_, n)| *n > 0)?;
        let v = match variant {
            Some(v) if (v as usize) < n => v as i32,
            Some(_) => 0,
            None => (x * 7 + y * 13).rem_euclid(n as i32),
        };
        Some(format!("{dir}/v{v}.png"))
    }

    /// The image of `pick` (`dirs`, `variant`, `x`, `y`) downsampled to `w` x `h`.
    pub fn varied(&mut self, dirs: &[&str], variant: Option<u32>, x: i32, y: i32, w: i32, h: i32) -> Option<&Image> {
        let rel = self.pick(dirs, variant, x, y)?;
        self.image(&rel, w, h)
    }

    /// Tile `tile` from the first set of `sets` a layer holds (`tilesets/<set>/<tile>/v0.png`); the variant is
    /// (x x 7 + y x 13) mod n over that layer's v0..v(n-1) (`varied` by position).
    pub fn tile_in(&mut self, sets: &[&str], tile: &str, x: i32, y: i32, size: i32) -> Option<&Image> {
        let dirs: Vec<String> = sets.iter().map(|s| format!("tilesets/{s}/{tile}")).collect();
        let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
        self.varied(&dirs, None, x, y, size, size)
    }

}

/// A PNG's width and height from its header (None: not a readable PNG).
pub fn png_size(path: &Path) -> Option<(i32, i32)> {
    let file = std::fs::File::open(path).ok()?;
    let reader = png::Decoder::new(std::io::BufReader::new(file)).read_info().ok()?;
    let info = reader.info();
    Some((info.width as i32, info.height as i32))
}

/// The frames of `sheet` (laid out by `clips`) with no pixel at `OUTLINE_ALPHA` or above: (clip, frame), sheet order.
pub fn blank_frames(clips: &Clips, sheet: &Image) -> Vec<(String, u32)> {
    let (w, h) = (clips.frame.0 as i32, clips.frame.1 as i32);
    let mut out = Vec::new();
    for (row, c) in clips.clips.iter().enumerate() {
        for n in 0..c.frames {
            let f = sheet.crop(n as i32 * w, row as i32 * h, w, h);
            if !f.px.iter().skip(3).step_by(4).any(|a| *a >= OUTLINE_ALPHA) {
                out.push((c.id.clone(), n));
            }
        }
    }
    out
}

/// The game folder at or above `start` (the first holding game.json and assets/).
pub fn find_project(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("game.json").is_file() && dir.join("assets").is_dir() {
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
    fn chains_take_the_first_file_or_body_present() {
        let root = body_tree("chains");
        write_png(&root.join("portraits/common/knight_base.png"), 2, 2, &[7u8; 16]);
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        let chain = vec!["portraits/heroes/nobody.png".to_string(), "portraits/common/knight_base.png".to_string()];
        assert_eq!(art.first(&chain, 2, 2).map(|i| i.px[0]), Some(7), "the second of the chain");
        assert!(art.first(&["portraits/heroes/nobody.png".to_string()], 2, 2).is_none());
        let bodies = vec!["knight/goblin".to_string(), "knight".to_string()];
        assert_eq!(art.first_token(&bodies, "", 8).map(|i| (i.w, i.h)), Some((8, 8)), "a missing variant falls back to its class body");
        assert!(art.has("units/knight/base/sheet.png") && !art.has("units/knight/goblin/sheet.png"));
        let game = root.join("games/demo");
        std::fs::create_dir_all(game.join("assets/units")).unwrap();
        std::fs::write(game.join("game.json"), "{}").unwrap();
        assert_eq!(find_project(&game.join("assets/units")).as_deref(), Some(game.as_path()));
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
