//! Placeholder art from the Godot project (assets/art, read-only), downsampled to pixel scale and cached.

use crate::canvas::Image;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Art {
    root: Option<PathBuf>,
    cache: HashMap<(String, i32), Option<Image>>,
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
    /// `root`: the Godot project folder (holds assets/art); None draws fallbacks only.
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root, cache: HashMap::new() }
    }

    /// `rel` (e.g. "unit/warrior_token") at `size` x `size` pixels.
    pub fn get(&mut self, rel: &str, size: i32) -> Option<&Image> {
        let key = (rel.to_string(), size);
        if !self.cache.contains_key(&key) {
            let img = self.root.as_ref().and_then(|r| load_png(&r.join("assets/art").join(format!("{rel}.png")))).map(|i| i.downsample(size, size));
            self.cache.insert(key.clone(), img);
        }
        self.cache.get(&key).and_then(|i| i.as_ref())
    }

    pub fn token(&mut self, art: &str, size: i32) -> Option<&Image> {
        let rel = format!("unit/{art}_token");
        if self.get(&rel, size).is_some() {
            return self.get(&rel, size);
        }
        None
    }

    /// Terrain tile `id` (variant chosen by the cell, like TileArt).
    pub fn tile(&mut self, id: &str, x: i32, y: i32, size: i32) -> Option<&Image> {
        let v = ((x * 7 + y * 13) % 4 + 4) % 4;
        self.get(&format!("tile/{id}_v{v}"), size)
    }
}

/// The Godot project folder above `start` (the first ancestor holding project.godot).
pub fn find_project(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("project.godot").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
