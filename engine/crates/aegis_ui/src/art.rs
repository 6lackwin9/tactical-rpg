//! Art from the game's assets/art folder (read-only), downsampled to pixel scale and cached. One folder per thing
//! (ASSETORG1): `<category>/<id>/<variant>.png`, e.g. `unit/warrior/token.png`, `tile/dirt/v2.png`, `ui/pin_town/icon.png`;
//! data names the id, the screen the category and variant.

use crate::canvas::Image;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Art {
    root: Option<PathBuf>,
    cache: HashMap<(String, String, String, i32), Option<Image>>,
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
    /// `root`: the project folder (holds assets/art); None draws fallbacks only.
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root, cache: HashMap::new() }
    }

    /// The file of `variant` of thing `id` in `category` (assets/art/<category>/<id>/<variant>.png).
    pub fn path(root: &Path, category: &str, id: &str, variant: &str) -> PathBuf {
        root.join("assets/art").join(category).join(id).join(format!("{variant}.png"))
    }

    /// `variant` of `id` in `category` (e.g. "unit", "warrior", "token") at `size` x `size` pixels.
    pub fn get(&mut self, category: &str, id: &str, variant: &str, size: i32) -> Option<&Image> {
        let key = (category.to_string(), id.to_string(), variant.to_string(), size);
        if !self.cache.contains_key(&key) {
            let path = self.root.as_ref().map(|r| Self::path(r, category, id, variant));
            let img = path.as_ref().and_then(|p| load_png(p)).map(|i| i.downsample(size, size));
            if let (None, Some(p)) = (&img, &path) {
                // The caller draws its fallback; the playtest trail notes which art was missing.
                crate::playtest::report_asset(category, id, variant, if p.exists() { "load_failed" } else { "missing_file" });
            }
            self.cache.insert(key.clone(), img);
        }
        self.cache.get(&key).and_then(|i| i.as_ref())
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

/// The project folder above `start` (the first ancestor holding assets/art).
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
