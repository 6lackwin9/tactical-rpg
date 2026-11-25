//! Headless sprite production pipeline (`aegis sprite ...`), 100% game-agnostic:
//!   aegis sprite chroma [palette=<palette.json>] [colors=#rrggbb,...] [mode=auto|green|magenta|blue] [json]
//!   aegis sprite key <in.png> out=<out.png> [cols=1] [rows=1] [json]
//!   aegis sprite slice <in.png> out=<out.png> [cols=4] [rows=1] [frame=112x96] [foot=93] [key=1] [inset=2] [nearest=0|1] [clips=<clips.json>] [clip=<name>] [fps=8] [json]
//!   aegis sprite check <sheet.png> [ref=<anchor.png>] [frame=112x96] [foot=93] [max_foot_drift=2] [max_chroma_pct=1.0] [out=<sprite_check.json>] [json]
//!   aegis sprite batch-sheet (dirs=a,b | root=<dir>) [out=<batch_review.html>] [title=<text>] [embed=0|1]

use crate::align::{load_image, save_png, RgbaImg};
use crate::batch_sheet;
use crate::Args;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

const USAGE: &str = "aegis sprite chroma [palette=<palette.json>] [colors=#rrggbb,...] [mode=auto|green|magenta|blue] [json]
aegis sprite key <in.png> out=<out.png> [cols=1] [rows=1] [json]
aegis sprite slice <in.png> out=<out.png> [cols=4] [rows=1] [frame=112x96] [foot=93] [key=1] [inset=2] [nearest=0|1] [clips=<clips.json>] [clip=<name>] [fps=8] [json]
aegis sprite check <sheet.png> [ref=<anchor.png>] [frame=112x96] [foot=93] [max_foot_drift=2] [max_chroma_pct=1.0] [out=<sprite_check.json>] [json]
aegis sprite batch-sheet (dirs=a,b | root=<dir>) [out=<batch_review.html>] [title=<text>] [embed=0|1]";

/// CLI entrypoint for `aegis sprite ...`.
pub fn command(args: &Args) -> Result<()> {
    match args.pos(1) {
        Some("chroma") => cmd_chroma(args),
        Some("key") => cmd_key(args),
        Some("slice") => cmd_slice(args),
        Some("check") => cmd_check(args),
        Some("batch-sheet") => batch_sheet::cmd_batch_sheet_with_defaults(
            args,
            "out/sprite/batch_review.html",
            "Sprite Batch Review Sheet",
        ),
        _ => {
            println!("{USAGE}");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// 1. Chroma key color auto-selection (`aegis sprite chroma`)
// ---------------------------------------------------------------------------

/// Result of resolving an optimal high-contrast chroma screen color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromaSpec {
    pub hex: &'static str,
    pub label: &'static str,
    pub short_label: &'static str,
    pub has_green: bool,
    pub has_magenta: bool,
}

/// Parses `#RRGGBB` or `RRGGBB` into `[r, g, b]`.
pub fn parse_hex_rgb(hex: &str) -> Option<[u8; 3]> {
    let cleaned = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if cleaned.len() != 6 || !cleaned.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&cleaned[0..2], 16).ok()?;
    let g = u8::from_str_radix(&cleaned[2..4], 16).ok()?;
    let b = u8::from_str_radix(&cleaned[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Recursively extracts `#RRGGBB` strings from any JSON shape (flat arrays, objects, or nested trees).
pub fn collect_hex_colors_from_json(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with('#') && parse_hex_rgb(trimmed).is_some() {
                out.push(trimmed.to_string());
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_hex_colors_from_json(item, out);
            }
        }
        Value::Object(map) => {
            for val in map.values() {
                collect_hex_colors_from_json(val, out);
            }
        }
        _ => {}
    }
}

/// Picks an optimal high-contrast chroma key screen color (`#00FF00` green by default,
/// switching to `#FF00FF` magenta or `#0047FF` blue in `auto` mode when palette colors collide).
pub fn resolve_chroma_key(palette_hexes: &[String], mode: &str) -> Result<ChromaSpec> {
    match mode.to_ascii_lowercase().as_str() {
        "green" => Ok(ChromaSpec {
            hex: "#00FF00",
            label: "Chroma Green",
            short_label: "Green #00FF00",
            has_green: false,
            has_magenta: false,
        }),
        "magenta" => Ok(ChromaSpec {
            hex: "#FF00FF",
            label: "Chroma Magenta",
            short_label: "Magenta #FF00FF",
            has_green: false,
            has_magenta: false,
        }),
        "blue" => Ok(ChromaSpec {
            hex: "#0047FF",
            label: "Chroma Blue",
            short_label: "Blue #0047FF",
            has_green: false,
            has_magenta: false,
        }),
        "auto" => {
            let mut has_green = false;
            let mut has_magenta = false;
            for hex in palette_hexes {
                if let Some([r, g, b]) = parse_hex_rgb(hex) {
                    let (r, g, b) = (r as i32, g as i32, b as i32);
                    if g > r + 24 && g > b + 24 && g > 70 {
                        has_green = true;
                    }
                    if r > g + 35 && b > g + 35 && r > 90 && b > 90 {
                        has_magenta = true;
                    }
                }
            }
            if !has_green {
                Ok(ChromaSpec {
                    hex: "#00FF00",
                    label: "Chroma Green",
                    short_label: "Auto (#00FF00)",
                    has_green,
                    has_magenta,
                })
            } else if !has_magenta {
                Ok(ChromaSpec {
                    hex: "#FF00FF",
                    label: "Chroma Magenta",
                    short_label: "Auto (#FF00FF)",
                    has_green,
                    has_magenta,
                })
            } else {
                Ok(ChromaSpec {
                    hex: "#0047FF",
                    label: "Chroma Blue",
                    short_label: "Auto (#0047FF)",
                    has_green,
                    has_magenta,
                })
            }
        }
        other => bail!("unknown chroma mode '{other}' (expected auto, green, magenta, or blue)"),
    }
}

fn cmd_chroma(args: &Args) -> Result<()> {
    let mode = args.key("mode", "auto");
    let mut colors = Vec::new();

    if let Some(path) = args.keys.get("palette") {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading palette {path}"))?;
        let val: Value = serde_json::from_str(&text).with_context(|| format!("parsing palette JSON {path}"))?;
        collect_hex_colors_from_json(&val, &mut colors);
    }
    if let Some(raw) = args.keys.get("colors") {
        for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            colors.push(part.to_string());
        }
    }

    let spec = resolve_chroma_key(&colors, &mode)?;
    if args.flag("json") {
        println!(
            "{}",
            json!({
                "hex": spec.hex,
                "label": spec.label,
                "short_label": spec.short_label,
                "mode": mode,
                "colors_checked": colors.len(),
                "has_green": spec.has_green,
                "has_magenta": spec.has_magenta,
            })
        );
    } else {
        println!(
            "chroma: {} ({}; {}, checked {} color(s))",
            spec.hex,
            spec.label,
            spec.short_label,
            colors.len()
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Studio Chroma Keyer + Despill + Flood-Fill Fallback (`aegis sprite key`)
// ---------------------------------------------------------------------------

/// Summary statistics from background keying.
#[derive(Debug, Clone)]
pub struct KeyStats {
    pub mode: &'static str,
    pub bg_rgb: [u8; 3],
    pub bg_saturation: f64,
    pub keyed_pixels: usize,
    pub despilled_pixels: usize,
    pub total_pixels: usize,
}

/// Removes a solid chroma or neutral background to transparent alpha in-place.
///
/// - When corners have high saturation (`>= 70`), uses global + interior-hole chroma keying
///   with broadcast edge unpremultiply and color-spill suppression (despill).
/// - Otherwise falls back to exterior + grid-seam border flood-fill with soft edge feathering.
pub fn remove_solid_background_to_alpha(
    img: &mut RgbaImg,
    grid_cols: usize,
    grid_rows: usize,
) -> KeyStats {
    let w = img.w;
    let h = img.h;
    let total = w * h;
    if total == 0 {
        return KeyStats {
            mode: "empty",
            bg_rgb: [0, 0, 0],
            bg_saturation: 0.0,
            keyed_pixels: 0,
            despilled_pixels: 0,
            total_pixels: 0,
        };
    }

    let cols = grid_cols.max(1);
    let rows = grid_rows.max(1);

    // Sample 4 corners (inset by 1px when possible to avoid edge DCT artifacts).
    let ix0 = 1.min(w - 1);
    let iy0 = 1.min(h - 1);
    let ix1 = w.saturating_sub(2);
    let iy1 = h.saturating_sub(2);
    let sample_indices = [
        (iy0 * w + ix0) * 4,
        (iy0 * w + ix1) * 4,
        (iy1 * w + ix0) * 4,
        (iy1 * w + ix1) * 4,
    ];

    let (mut bg_r, mut bg_g, mut bg_b) = (0.0f64, 0.0f64, 0.0f64);
    for idx in sample_indices {
        bg_r += img.px[idx] as f64;
        bg_g += img.px[idx + 1] as f64;
        bg_b += img.px[idx + 2] as f64;
    }
    bg_r /= 4.0;
    bg_g /= 4.0;
    bg_b /= 4.0;

    let max_bg = bg_r.max(bg_g).max(bg_b);
    let min_bg = bg_r.min(bg_g).min(bg_b);
    let bg_saturation = max_bg - min_bg;
    let bg_rgb = [
        bg_r.round().clamp(0.0, 255.0) as u8,
        bg_g.round().clamp(0.0, 255.0) as u8,
        bg_b.round().clamp(0.0, 255.0) as u8,
    ];

    let color_dist = |px: &[u8], p: usize| -> f64 {
        let i = p * 4;
        let dr = px[i] as f64 - bg_r;
        let dg = px[i + 1] as f64 - bg_g;
        let db = px[i + 2] as f64 - bg_b;
        (dr * dr + dg * dg + db * db).sqrt()
    };

    // Branch A: High-saturation Chroma Key screen (#00FF00, #FF00FF, #0047FF)
    if bg_saturation >= 70.0 {
        let is_green_key = bg_g > bg_r + 55.0 && bg_g > bg_b + 55.0;
        let is_magenta_key = bg_r > bg_g + 55.0 && bg_b > bg_g + 55.0;
        let is_blue_key = bg_b > bg_r + 55.0 && bg_b > bg_g + 45.0;
        let mode = if is_green_key {
            "chroma_green"
        } else if is_magenta_key {
            "chroma_magenta"
        } else if is_blue_key {
            "chroma_blue"
        } else {
            "chroma_other"
        };

        let inner_cut_dist = 96.0;
        let outer_soft_dist = 158.0;
        let mut transparent_mask = vec![0u8; total];
        let mut keyed_pixels = 0usize;

        // Pass 1: Global + interior hole chroma key extraction
        for p in 0..total {
            let idx = p * 4;
            let r = img.px[idx] as f64;
            let g = img.px[idx + 1] as f64;
            let b = img.px[idx + 2] as f64;
            let dist = color_dist(&img.px, p);

            let key_dominance = if is_green_key {
                g - r.max(b)
            } else if is_magenta_key {
                r.min(b) - g
            } else if is_blue_key {
                b - r.max(g)
            } else {
                0.0
            };

            if dist <= inner_cut_dist || (key_dominance >= 82.0 && dist <= 132.0) {
                img.px[idx + 3] = 0;
                transparent_mask[p] = 1;
                keyed_pixels += 1;
            } else if dist < outer_soft_dist {
                let alpha_ratio =
                    ((dist - inner_cut_dist) / (outer_soft_dist - inner_cut_dist)).clamp(0.0, 1.0);
                img.px[idx + 3] = ((img.px[idx + 3] as f64) * alpha_ratio)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                if alpha_ratio < 0.25 {
                    transparent_mask[p] = 1;
                }
            }
        }

        // Pass 2: Edge unpremultiply & broadcast chroma spill suppression (despill)
        let mut despilled_pixels = 0usize;
        for y in 0..h {
            for x in 0..w {
                let p = y * w + x;
                let idx = p * 4;
                let a = img.px[idx + 3];
                if a == 0 {
                    continue;
                }

                let near_keyed = a < 250
                    || (x > 0 && transparent_mask[p - 1] == 1)
                    || (x + 1 < w && transparent_mask[p + 1] == 1)
                    || (y > 0 && transparent_mask[p - w] == 1)
                    || (y + 1 < h && transparent_mask[p + w] == 1);

                if !near_keyed {
                    continue;
                }

                let mut r = img.px[idx] as f64;
                let mut g = img.px[idx + 1] as f64;
                let mut b = img.px[idx + 2] as f64;

                let alpha_ratio = a as f64 / 255.0;
                if alpha_ratio > 0.08 && alpha_ratio < 0.96 {
                    let inv = 1.0 - alpha_ratio;
                    r = ((r - bg_r * inv) / alpha_ratio).round().clamp(0.0, 255.0);
                    g = ((g - bg_g * inv) / alpha_ratio).round().clamp(0.0, 255.0);
                    b = ((b - bg_b * inv) / alpha_ratio).round().clamp(0.0, 255.0);
                }

                // Suppress chroma color fringe on sprite edges
                if is_green_key {
                    let cap = (r.max(b) * 1.03).round();
                    if g > cap {
                        g = cap;
                    }
                } else if is_magenta_key {
                    let cap = (g * 1.15 + 18.0).round();
                    if r > cap && b > cap {
                        r = ((r + cap) * 0.5).round();
                        b = ((b + cap) * 0.5).round();
                    }
                } else if is_blue_key {
                    let cap = (r.max(g) * 1.06).round();
                    if b > cap {
                        b = cap;
                    }
                }

                img.px[idx] = r.clamp(0.0, 255.0) as u8;
                img.px[idx + 1] = g.clamp(0.0, 255.0) as u8;
                img.px[idx + 2] = b.clamp(0.0, 255.0) as u8;
                despilled_pixels += 1;
            }
        }

        return KeyStats {
            mode,
            bg_rgb,
            bg_saturation,
            keyed_pixels,
            despilled_pixels,
            total_pixels: total,
        };
    }

    // Branch B: Neutral/white background fallback via exterior + grid-seam border flood-fill
    let mut visited = vec![0u8; total];
    let mut is_bg = vec![0u8; total];
    let mut queue = VecDeque::with_capacity(w * 2 + h * 2);

    let hard_threshold = 48.0;
    let soft_threshold = 76.0;

    let enqueue_if_bg = |x: usize,
                         y: usize,
                         px: &[u8],
                         visited: &mut [u8],
                         is_bg: &mut [u8],
                         queue: &mut VecDeque<usize>| {
        if x >= w || y >= h {
            return;
        }
        let p = y * w + x;
        if visited[p] != 0 {
            return;
        }
        visited[p] = 1;
        if color_dist(px, p) <= hard_threshold {
            is_bg[p] = 1;
            queue.push_back(p);
        }
    };

    for x in 0..w {
        enqueue_if_bg(x, 0, &img.px, &mut visited, &mut is_bg, &mut queue);
        enqueue_if_bg(x, h - 1, &img.px, &mut visited, &mut is_bg, &mut queue);
    }
    for y in 0..h {
        enqueue_if_bg(0, y, &img.px, &mut visited, &mut is_bg, &mut queue);
        enqueue_if_bg(w - 1, y, &img.px, &mut visited, &mut is_bg, &mut queue);
    }
    for c in 1..cols {
        let gx = ((c * w) as f64 / cols as f64).round() as usize;
        if gx < w {
            for y in 0..h {
                enqueue_if_bg(gx, y, &img.px, &mut visited, &mut is_bg, &mut queue);
            }
        }
    }
    for r in 1..rows {
        let gy = ((r * h) as f64 / rows as f64).round() as usize;
        if gy < h {
            for x in 0..w {
                enqueue_if_bg(x, gy, &img.px, &mut visited, &mut is_bg, &mut queue);
            }
        }
    }

    while let Some(curr) = queue.pop_front() {
        let cx = curr % w;
        let cy = curr / w;
        let neighbors = [
            (cx > 0).then(|| curr - 1),
            (cx + 1 < w).then(|| curr + 1),
            (cy > 0).then(|| curr - w),
            (cy + 1 < h).then(|| curr + w),
        ];
        for np in neighbors.into_iter().flatten() {
            if visited[np] == 0 {
                visited[np] = 1;
                if color_dist(&img.px, np) <= hard_threshold {
                    is_bg[np] = 1;
                    queue.push_back(np);
                }
            }
        }
    }

    let mut keyed_pixels = 0usize;
    let mut despilled_pixels = 0usize;
    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let idx = p * 4;
            if is_bg[p] != 0 {
                img.px[idx + 3] = 0;
                keyed_pixels += 1;
                continue;
            }
            let touches_bg = (x > 0 && is_bg[p - 1] != 0)
                || (x + 1 < w && is_bg[p + 1] != 0)
                || (y > 0 && is_bg[p - w] != 0)
                || (y + 1 < h && is_bg[p + w] != 0);
            if touches_bg {
                let dist = color_dist(&img.px, p);
                if dist < soft_threshold {
                    let denom = soft_threshold - hard_threshold * 0.65;
                    let alpha_ratio = ((dist - hard_threshold * 0.65) / denom).clamp(0.0, 1.0);
                    img.px[idx + 3] = ((img.px[idx + 3] as f64) * alpha_ratio)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    despilled_pixels += 1;
                }
            }
        }
    }

    KeyStats {
        mode: "flood_fill",
        bg_rgb,
        bg_saturation,
        keyed_pixels,
        despilled_pixels,
        total_pixels: total,
    }
}

fn cmd_key(args: &Args) -> Result<()> {
    let input = args
        .pos(2)
        .context("usage: aegis sprite key <in.png> out=<out.png> [cols=1] [rows=1] [json]")?;
    let out_path = PathBuf::from(
        args.keys
            .get("out")
            .context("sprite key requires out=<out.png>")?,
    );
    let cols: usize = args.key("cols", "1").parse().context("cols must be a positive integer")?;
    let rows: usize = args.key("rows", "1").parse().context("rows must be a positive integer")?;
    if cols == 0 || rows == 0 {
        bail!("cols and rows must be >= 1");
    }

    let mut img = load_image(Path::new(input))?;
    let stats = remove_solid_background_to_alpha(&mut img, cols, rows);
    save_png(&img, &out_path)?;

    if args.flag("json") {
        println!(
            "{}",
            json!({
                "in": input,
                "out": out_path.display().to_string(),
                "width": img.w,
                "height": img.h,
                "mode": stats.mode,
                "bg_rgb": stats.bg_rgb,
                "bg_saturation": (stats.bg_saturation * 100.0).round() / 100.0,
                "keyed_pixels": stats.keyed_pixels,
                "despilled_pixels": stats.despilled_pixels,
                "total_pixels": stats.total_pixels,
            })
        );
    } else {
        println!(
            "sprite key [{}]: {} -> {} ({}x{}, keyed {}/{} px, despilled {} px)",
            stats.mode,
            input,
            out_path.display(),
            img.w,
            img.h,
            stats.keyed_pixels,
            stats.total_pixels,
            stats.despilled_pixels
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Sheet Slicing, Resampling & Foot Alignment (`aegis sprite slice`)
// ---------------------------------------------------------------------------

/// Configuration for slicing an `N x M` input strip or sheet into a game-ready sheet.
#[derive(Debug, Clone)]
pub struct SliceOpts {
    pub cols: usize,
    pub rows: usize,
    pub frame_w: usize,
    pub frame_h: usize,
    pub foot: usize,
    pub run_key: bool,
    pub inset: usize,
    pub nearest: bool,
}

/// Summary of one sliced cell.
#[derive(Debug, Clone)]
pub struct SlicedCellInfo {
    pub row: usize,
    pub col: usize,
    pub empty: bool,
    pub src_bbox: Option<(usize, usize, usize, usize)>,
    pub dst_rect: Option<(usize, usize, usize, usize)>,
    pub bottom_y: Option<usize>,
}

/// Output of `slice_sheet`.
#[derive(Debug, Clone)]
pub struct SliceResult {
    pub image: RgbaImg,
    pub key_stats: Option<KeyStats>,
    pub cells: Vec<SlicedCellInfo>,
}

fn parse_frame_dims(raw: &str) -> Result<(usize, usize)> {
    let (w_s, h_s) = raw
        .split_once('x')
        .or_else(|| raw.split_once('X'))
        .with_context(|| format!("invalid frame size '{raw}' (expected WxH, e.g. 112x96)"))?;
    let w: usize = w_s.trim().parse().context("frame width")?;
    let h: usize = h_s.trim().parse().context("frame height")?;
    if w < 8 || h < 8 {
        bail!("frame dimensions must be at least 8x8 (got {w}x{h})");
    }
    Ok((w, h))
}

fn sample_bilinear(img: &RgbaImg, sx: f64, sy: f64, min_x: usize, max_x: usize, min_y: usize, max_y: usize) -> [u8; 4] {
    let cx = sx.clamp(min_x as f64, max_x as f64);
    let cy = sy.clamp(min_y as f64, max_y as f64);
    let x0 = cx.floor() as usize;
    let y0 = cy.floor() as usize;
    let x1 = (x0 + 1).min(max_x);
    let y1 = (y0 + 1).min(max_y);
    let fx = cx - x0 as f64;
    let fy = cy - y0 as f64;

    let c00 = img.get(x0, y0);
    let c10 = img.get(x1, y0);
    let c01 = img.get(x0, y1);
    let c11 = img.get(x1, y1);

    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;

    // Premultiplied-alpha bilinear filtering prevents dark RGB bleeding at transparent edges.
    let a00 = c00[3] as f64 / 255.0;
    let a10 = c10[3] as f64 / 255.0;
    let a01 = c01[3] as f64 / 255.0;
    let a11 = c11[3] as f64 / 255.0;

    let a = w00 * a00 + w10 * a10 + w01 * a01 + w11 * a11;
    if a <= 1e-6 {
        return [0, 0, 0, 0];
    }

    let r_pm = w00 * c00[0] as f64 * a00
        + w10 * c10[0] as f64 * a10
        + w01 * c01[0] as f64 * a01
        + w11 * c11[0] as f64 * a11;
    let g_pm = w00 * c00[1] as f64 * a00
        + w10 * c10[1] as f64 * a10
        + w01 * c01[1] as f64 * a01
        + w11 * c11[1] as f64 * a11;
    let b_pm = w00 * c00[2] as f64 * a00
        + w10 * c10[2] as f64 * a10
        + w01 * c01[2] as f64 * a01
        + w11 * c11[2] as f64 * a11;

    [
        (r_pm / a).round().clamp(0.0, 255.0) as u8,
        (g_pm / a).round().clamp(0.0, 255.0) as u8,
        (b_pm / a).round().clamp(0.0, 255.0) as u8,
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

/// Slices `src` into a `cols * frame_w` by `rows * frame_h` sheet with exact foot alignment.
pub fn slice_sheet(src: &RgbaImg, opts: &SliceOpts) -> Result<SliceResult> {
    if opts.cols == 0 || opts.rows == 0 {
        bail!("slice: cols and rows must be >= 1");
    }
    if opts.foot >= opts.frame_h {
        bail!(
            "slice: foot row {} must be < frame height {}",
            opts.foot,
            opts.frame_h
        );
    }

    let mut working = src.clone();
    let key_stats = if opts.run_key {
        Some(remove_solid_background_to_alpha(&mut working, opts.cols, opts.rows))
    } else {
        None
    };

    let out_w = opts.cols * opts.frame_w;
    let out_h = opts.rows * opts.frame_h;
    let mut out = RgbaImg::new(out_w, out_h, [0, 0, 0, 0]);
    let mut cells = Vec::with_capacity(opts.cols * opts.rows);

    let top_margin = 4usize.min(opts.foot);
    let max_h = (opts.foot + 1).saturating_sub(top_margin).max(1);
    let max_w = opts.frame_w.saturating_sub(8).max(1);

    for r in 0..opts.rows {
        for c in 0..opts.cols {
            let mut sx0 = (c * working.w) / opts.cols;
            let mut sx1 = ((c + 1) * working.w) / opts.cols;
            let mut sy0 = (r * working.h) / opts.rows;
            let mut sy1 = ((r + 1) * working.h) / opts.rows;

            // Trim inset inside internal cell seams only.
            if c > 0 {
                sx0 = sx0.saturating_add(opts.inset);
            }
            if c + 1 < opts.cols {
                sx1 = sx1.saturating_sub(opts.inset);
            }
            if r > 0 {
                sy0 = sy0.saturating_add(opts.inset);
            }
            if r + 1 < opts.rows {
                sy1 = sy1.saturating_sub(opts.inset);
            }

            if sx0 >= sx1 || sy0 >= sy1 {
                cells.push(SlicedCellInfo {
                    row: r,
                    col: c,
                    empty: true,
                    src_bbox: None,
                    dst_rect: None,
                    bottom_y: None,
                });
                continue;
            }

            // Compute opaque bounding box (alpha >= 16).
            let (mut min_x, mut min_y) = (usize::MAX, usize::MAX);
            let (mut max_x, mut max_y) = (0usize, 0usize);
            let mut found = false;
            for y in sy0..sy1 {
                for x in sx0..sx1 {
                    if working.get(x, y)[3] >= 16 {
                        found = true;
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
            }

            if !found {
                cells.push(SlicedCellInfo {
                    row: r,
                    col: c,
                    empty: true,
                    src_bbox: None,
                    dst_rect: None,
                    bottom_y: None,
                });
                continue;
            }

            let bw = max_x - min_x + 1;
            let bh = max_y - min_y + 1;
            let scale = (max_w as f64 / bw as f64).min(max_h as f64 / bh as f64);
            let dst_w = ((bw as f64) * scale).round().max(1.0) as usize;
            let dst_w = dst_w.min(opts.frame_w);
            let dst_h = ((bh as f64) * scale).round().max(1.0) as usize;
            let dst_h = dst_h.min(opts.foot + 1);

            // Compute horizontal foot centroid from the bottom band of the sprite (or bbox center if narrow).
            let centroid_x = if bw <= 4 {
                (min_x + max_x) as f64 * 0.5
            } else {
                let band_h = (bh / 10).max(2).min(bh);
                let band_y0 = max_y + 1 - band_h;
                let (mut sum_x, mut cnt) = (0.0f64, 0usize);
                for y in band_y0..=max_y {
                    for x in min_x..=max_x {
                        if working.get(x, y)[3] >= 16 {
                            sum_x += x as f64;
                            cnt += 1;
                        }
                    }
                }
                if cnt > 0 {
                    sum_x / cnt as f64
                } else {
                    (min_x + max_x) as f64 * 0.5
                }
            };

            let foot_cx_dst = ((centroid_x - min_x as f64) + 0.5) * (dst_w as f64 / bw as f64);
            let dst_x0 = (opts.frame_w as f64 * 0.5 - foot_cx_dst)
                .round()
                .clamp(0.0, opts.frame_w.saturating_sub(dst_w) as f64) as usize;
            let dst_y0 = (opts.foot + 1).saturating_sub(dst_h);

            let mut frame_buf = RgbaImg::new(opts.frame_w, opts.frame_h, [0, 0, 0, 0]);
            for dy in 0..dst_h {
                for dx in 0..dst_w {
                    let px = if opts.nearest {
                        let rx = (((dx as f64 + 0.5) * bw as f64 / dst_w as f64).floor() as usize)
                            .min(bw - 1);
                        let ry = (((dy as f64 + 0.5) * bh as f64 / dst_h as f64).floor() as usize)
                            .min(bh - 1);
                        working.get(min_x + rx, min_y + ry)
                    } else {
                        let sx = min_x as f64 + (dx as f64 + 0.5) * bw as f64 / dst_w as f64 - 0.5;
                        let sy = min_y as f64 + (dy as f64 + 0.5) * bh as f64 / dst_h as f64 - 0.5;
                        sample_bilinear(&working, sx, sy, min_x, max_x, min_y, max_y)
                    };
                    frame_buf.set(dst_x0 + dx, dst_y0 + dy, px);
                }
            }

            // Guarantee bottom-most opaque pixel row lands exactly on `opts.foot` even under heavy downscale.
            let mut actual_bottom = None;
            for y in (0..opts.frame_h).rev() {
                if (0..opts.frame_w).any(|x| frame_buf.get(x, y)[3] >= 16) {
                    actual_bottom = Some(y);
                    break;
                }
            }
            if let Some(ab) = actual_bottom {
                if ab != opts.foot {
                    let mut shifted = RgbaImg::new(opts.frame_w, opts.frame_h, [0, 0, 0, 0]);
                    let shift = opts.foot as isize - ab as isize;
                    for y in 0..opts.frame_h {
                        let ny = y as isize + shift;
                        if ny >= 0 && (ny as usize) < opts.frame_h {
                            for x in 0..opts.frame_w {
                                shifted.set(x, ny as usize, frame_buf.get(x, y));
                            }
                        }
                    }
                    frame_buf = shifted;
                    actual_bottom = Some(opts.foot);
                }
            }

            let fx0 = c * opts.frame_w;
            let fy0 = r * opts.frame_h;
            for y in 0..opts.frame_h {
                for x in 0..opts.frame_w {
                    out.set(fx0 + x, fy0 + y, frame_buf.get(x, y));
                }
            }

            cells.push(SlicedCellInfo {
                row: r,
                col: c,
                empty: false,
                src_bbox: Some((min_x, min_y, bw, bh)),
                dst_rect: Some((dst_x0, dst_y0, dst_w, dst_h)),
                bottom_y: actual_bottom,
            });
        }
    }

    Ok(SliceResult {
        image: out,
        key_stats,
        cells,
    })
}

/// Builds a valid `clips.json` payload matching `aegis_core::assets::Clips::parse`.
pub fn build_clips_json(
    frame_w: usize,
    frame_h: usize,
    foot: usize,
    cols: usize,
    rows: usize,
    clip_names_raw: &str,
    fps: f64,
) -> Value {
    let ms = (1000.0 / fps.max(0.5)).round().clamp(16.0, 2000.0) as u32;
    let requested: Vec<String> = clip_names_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(sanitize_snake_id)
        .collect();
    let base = requested.first().cloned().unwrap_or_else(|| "idle".to_string());
    let mut clips = Vec::with_capacity(rows);
    for r in 0..rows {
        let id = if let Some(name) = requested.get(r) {
            name.clone()
        } else if r == 0 {
            "idle".to_string()
        } else {
            format!("{base}_{r}")
        };
        clips.push(json!({
            "id": id,
            "frames": cols,
            "ms": vec![ms; cols],
            "loop": true,
        }));
    }
    json!({
        "frame": [frame_w, frame_h],
        "foot": foot,
        "painted": true,
        "clips": clips,
    })
}

fn sanitize_snake_id(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        let lc = ch.to_ascii_lowercase();
        if lc.is_ascii_lowercase() || lc.is_ascii_digit() || lc == '_' {
            out.push(lc);
        } else if lc == '-' || lc == ' ' {
            out.push('_');
        }
    }
    if out.is_empty() || !out.starts_with(|c: char| c.is_ascii_lowercase()) {
        format!("clip_{out}")
    } else {
        out
    }
}

fn cmd_slice(args: &Args) -> Result<()> {
    let input = args.pos(2).context(
        "usage: aegis sprite slice <in.png> out=<out.png> [cols=4] [rows=1] [frame=112x96] [foot=93] [key=1] [inset=2] [nearest=0|1] [clips=<clips.json>] [clip=<name>] [fps=8] [json]",
    )?;
    let out_path = PathBuf::from(
        args.keys
            .get("out")
            .context("sprite slice requires out=<out.png>")?,
    );
    let cols: usize = args.key("cols", "4").parse().context("cols")?;
    let rows: usize = args.key("rows", "1").parse().context("rows")?;
    let (frame_w, frame_h) = parse_frame_dims(&args.key("frame", "112x96"))?;
    let foot: usize = args.key("foot", "93").parse().context("foot")?;
    let run_key = args.key("key", "1") != "0";
    let inset: usize = args.key("inset", "2").parse().context("inset")?;
    let nearest = args.key("nearest", "0") == "1";
    let fps: f64 = args.key("fps", "8").parse().context("fps")?;
    if fps <= 0.0 {
        bail!("fps must be > 0");
    }

    let src = load_image(Path::new(input))?;
    let opts = SliceOpts {
        cols,
        rows,
        frame_w,
        frame_h,
        foot,
        run_key,
        inset,
        nearest,
    };
    let result = slice_sheet(&src, &opts)?;
    save_png(&result.image, &out_path)?;

    let clips_written = if let Some(clips_path) = args.keys.get("clips") {
        let clip_name = args.key("clip", "idle");
        let clips_val = build_clips_json(frame_w, frame_h, foot, cols, rows, &clip_name, fps);
        let p = Path::new(clips_path);
        if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(p, serde_json::to_string_pretty(&clips_val)? + "\n")
            .with_context(|| format!("writing {}", p.display()))?;
        Some(clips_path.clone())
    } else {
        None
    };

    let non_empty = result.cells.iter().filter(|c| !c.empty).count();
    if args.flag("json") {
        println!(
            "{}",
            json!({
                "in": input,
                "out": out_path.display().to_string(),
                "width": result.image.w,
                "height": result.image.h,
                "cols": cols,
                "rows": rows,
                "frame": [frame_w, frame_h],
                "foot": foot,
                "key_mode": result.key_stats.as_ref().map(|k| k.mode),
                "frames_total": result.cells.len(),
                "frames_non_empty": non_empty,
                "clips": clips_written,
            })
        );
    } else {
        println!(
            "sprite slice: {} -> {} ({}x{}, {}x{} frames of {}x{}, foot={}, {}/{} non-empty)",
            input,
            out_path.display(),
            result.image.w,
            result.image.h,
            cols,
            rows,
            frame_w,
            frame_h,
            foot,
            non_empty,
            result.cells.len()
        );
        if let Some(cp) = clips_written {
            println!("  wrote clips: {cp}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Headless Sprite Drift QA (`aegis sprite check`)
// ---------------------------------------------------------------------------

/// Options for `check_sprite_sheet`.
#[derive(Debug, Clone)]
pub struct CheckOpts {
    pub frame_w: usize,
    pub frame_h: usize,
    pub foot: usize,
    pub max_foot_drift: usize,
    pub max_chroma_pct: f64,
}

#[derive(Debug, Clone)]
struct RegionStats {
    mean_luma: f64,
    mean_sat: f64,
    fill_ratio: f64,
}

fn is_chroma_fringe_rgb(r: u8, g: u8, b: u8) -> bool {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    let neon_green = g > r + 60 && g > b + 60 && g >= 140;
    let neon_magenta = r > g + 60 && b > g + 60 && r >= 140 && b >= 140;
    let neon_blue = b > r + 65 && b > g + 55 && b >= 150;
    neon_green || neon_magenta || neon_blue
}

fn measure_anchor(anchor: &RgbaImg, frame_w: usize, frame_h: usize) -> Option<RegionStats> {
    // If anchor is a multi-frame sheet of exact frame multiples, measure its first non-empty frame.
    let (rx0, ry0, rw, rh) = if anchor.w >= frame_w
        && anchor.h >= frame_h
        && anchor.w % frame_w == 0
        && anchor.h % frame_h == 0
    {
        let cols = anchor.w / frame_w;
        let rows = anchor.h / frame_h;
        let mut chosen = (0, 0, frame_w, frame_h);
        'outer: for r in 0..rows {
            for c in 0..cols {
                let x0 = c * frame_w;
                let y0 = r * frame_h;
                for y in y0..y0 + frame_h {
                    for x in x0..x0 + frame_w {
                        if anchor.get(x, y)[3] >= 16 {
                            chosen = (x0, y0, frame_w, frame_h);
                            break 'outer;
                        }
                    }
                }
            }
        }
        chosen
    } else {
        (0, 0, anchor.w, anchor.h)
    };

    let area = (rw * rh) as f64;
    if area <= 0.0 {
        return None;
    }
    let (mut luma_sum, mut sat_sum, mut cnt) = (0.0f64, 0.0f64, 0usize);
    for y in ry0..ry0 + rh {
        for x in rx0..rx0 + rw {
            let [r, g, b, a] = anchor.get(x, y);
            if a >= 16 {
                luma_sum += 0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64;
                sat_sum += (r.max(g).max(b) - r.min(g).min(b)) as f64;
                cnt += 1;
            }
        }
    }
    if cnt == 0 {
        return None;
    }
    Some(RegionStats {
        mean_luma: luma_sum / cnt as f64,
        mean_sat: sat_sum / cnt as f64,
        fill_ratio: cnt as f64 / area,
    })
}

/// Runs deterministic headless drift QA on a sprite sheet and returns a structured JSON report.
pub fn check_sprite_sheet(
    sheet: &RgbaImg,
    sheet_label: &str,
    sprite_id: &str,
    anchor: Option<&RgbaImg>,
    opts: &CheckOpts,
) -> Value {
    let mut issues: Vec<Value> = Vec::new();
    let mut has_fail = false;
    let mut has_warn = false;

    let mut push_issue = |severity: &str, code: &str, frame: Option<(usize, usize)>, message: String| {
        if severity == "FAIL" {
            has_fail = true;
        } else if severity == "WARN" {
            has_warn = true;
        }
        issues.push(json!({
            "severity": severity,
            "code": code,
            "frame": frame.map(|(r, c)| json!([r, c])).unwrap_or(Value::Null),
            "message": message,
        }));
    };

    let dims_ok = sheet.w > 0
        && sheet.h > 0
        && sheet.w % opts.frame_w == 0
        && sheet.h % opts.frame_h == 0;
    if !dims_ok {
        push_issue(
            "FAIL",
            "bad_dimensions",
            None,
            format!(
                "sheet dimensions {}x{} are not an exact multiple of {}x{}",
                sheet.w, sheet.h, opts.frame_w, opts.frame_h
            ),
        );
    }

    let cols = sheet.w / opts.frame_w;
    let rows = sheet.h / opts.frame_h;
    let mut frames_json = Vec::with_capacity(cols * rows);
    let mut heights: Vec<(usize, usize, usize)> = Vec::new();
    let mut max_foot_drift_seen = 0usize;
    let mut max_chroma_pct_seen = 0.0f64;
    let (mut sum_luma, mut sum_sat, mut sum_fill, mut non_empty_count) = (0.0f64, 0.0f64, 0.0f64, 0usize);

    for r in 0..rows {
        for c in 0..cols {
            let fx0 = c * opts.frame_w;
            let fy0 = r * opts.frame_h;
            let (mut min_x, mut min_y) = (usize::MAX, usize::MAX);
            let (mut max_x, mut max_y) = (0usize, 0usize);
            let (mut opaque_cnt, mut edge_cnt, mut chroma_cnt) = (0usize, 0usize, 0usize);
            let (mut luma_sum, mut sat_sum) = (0.0f64, 0.0f64);

            for ly in 0..opts.frame_h {
                for lx in 0..opts.frame_w {
                    let [cr, cg, cb, ca] = sheet.get(fx0 + lx, fy0 + ly);
                    if ca >= 16 {
                        opaque_cnt += 1;
                        min_x = min_x.min(lx);
                        min_y = min_y.min(ly);
                        max_x = max_x.max(lx);
                        max_y = max_y.max(ly);
                        luma_sum += 0.2126 * cr as f64 + 0.7152 * cg as f64 + 0.0722 * cb as f64;
                        sat_sum += (cr.max(cg).max(cb) - cr.min(cg).min(cb)) as f64;
                    }
                    if ca >= 1 {
                        let is_semi = ca < 250;
                        let is_boundary = if is_semi {
                            true
                        } else {
                            let left_t = lx == 0 || sheet.get(fx0 + lx - 1, fy0 + ly)[3] < 16;
                            let right_t =
                                lx + 1 >= opts.frame_w || sheet.get(fx0 + lx + 1, fy0 + ly)[3] < 16;
                            let up_t = ly == 0 || sheet.get(fx0 + lx, fy0 + ly - 1)[3] < 16;
                            let down_t =
                                ly + 1 >= opts.frame_h || sheet.get(fx0 + lx, fy0 + ly + 1)[3] < 16;
                            left_t || right_t || up_t || down_t
                        };
                        if is_boundary {
                            edge_cnt += 1;
                            if is_chroma_fringe_rgb(cr, cg, cb) {
                                chroma_cnt += 1;
                            }
                        }
                    }
                }
            }

            if opaque_cnt == 0 {
                frames_json.push(json!({
                    "row": r,
                    "col": c,
                    "empty": true,
                }));
                continue;
            }

            non_empty_count += 1;
            let height = max_y - min_y + 1;
            let width = max_x - min_x + 1;
            let foot_drift = (max_y as isize - opts.foot as isize).unsigned_abs();
            max_foot_drift_seen = max_foot_drift_seen.max(foot_drift);

            let chroma_pct = if edge_cnt > 0 {
                (chroma_cnt as f64 / edge_cnt as f64) * 100.0
            } else {
                0.0
            };
            if chroma_pct > max_chroma_pct_seen {
                max_chroma_pct_seen = chroma_pct;
            }

            if foot_drift > opts.max_foot_drift {
                push_issue(
                    "FAIL",
                    "foot_drift",
                    Some((r, c)),
                    format!(
                        "frame (r{r},c{c}): bottom opaque row {max_y} drifts {foot_drift}px from foot={} (max {})",
                        opts.foot, opts.max_foot_drift
                    ),
                );
            }
            if min_y == 0 {
                push_issue(
                    "WARN",
                    "top_clip",
                    Some((r, c)),
                    format!("frame (r{r},c{c}): top-edge clipping (top_y=0)"),
                );
            }
            if chroma_pct > opts.max_chroma_pct {
                push_issue(
                    "FAIL",
                    "chroma_halo",
                    Some((r, c)),
                    format!(
                        "frame (r{r},c{c}): chroma halo residue {chroma_pct:.2}% exceeds max {:.2}%",
                        opts.max_chroma_pct
                    ),
                );
            }

            let mean_luma = luma_sum / opaque_cnt as f64;
            let mean_sat = sat_sum / opaque_cnt as f64;
            let fill_ratio = opaque_cnt as f64 / (opts.frame_w * opts.frame_h) as f64;
            sum_luma += mean_luma;
            sum_sat += mean_sat;
            sum_fill += fill_ratio;
            heights.push((r, c, height));

            frames_json.push(json!({
                "row": r,
                "col": c,
                "empty": false,
                "bbox": [min_x, min_y, width, height],
                "top_y": min_y,
                "bottom_y": max_y,
                "foot_drift": foot_drift,
                "chroma_pct": (chroma_pct * 100.0).round() / 100.0,
                "mean_luma": (mean_luma * 10.0).round() / 10.0,
                "mean_sat": (mean_sat * 10.0).round() / 10.0,
                "fill_ratio": (fill_ratio * 10000.0).round() / 10000.0,
            }));
        }
    }

    if non_empty_count == 0 {
        push_issue("FAIL", "empty_sheet", None, "sheet has zero non-empty frames".to_string());
    }

    // Height consistency across non-empty frames (>25% deviation from median -> WARN).
    let median_height = if heights.is_empty() {
        0usize
    } else {
        let mut sorted: Vec<usize> = heights.iter().map(|&(_, _, h)| h).collect();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    };
    if median_height > 0 {
        for &(r, c, h) in &heights {
            let dev = (h as f64 - median_height as f64).abs() / median_height as f64;
            if dev > 0.25 {
                push_issue(
                    "WARN",
                    "height_drift",
                    Some((r, c)),
                    format!(
                        "frame (r{r},c{c}): height {h}px deviates {:.1}% from median {median_height}px (>25%)",
                        dev * 100.0
                    ),
                );
            }
        }
    }

    let mean_luma = if non_empty_count > 0 { sum_luma / non_empty_count as f64 } else { 0.0 };
    let mean_sat = if non_empty_count > 0 { sum_sat / non_empty_count as f64 } else { 0.0 };
    let mean_fill = if non_empty_count > 0 { sum_fill / non_empty_count as f64 } else { 0.0 };

    // Optional Style Anchor comparison.
    let anchor_json = if let Some(anc_img) = anchor {
        if let Some(anc) = measure_anchor(anc_img, opts.frame_w, opts.frame_h) {
            let lum_diff = (mean_luma - anc.mean_luma).abs();
            let sat_diff = (mean_sat - anc.mean_sat).abs();
            let fill_rel_diff = (mean_fill - anc.fill_ratio).abs() / anc.fill_ratio.max(0.01);
            if lum_diff > 35.0 {
                push_issue(
                    "WARN",
                    "anchor_luma_drift",
                    None,
                    format!(
                        "average luminance {mean_luma:.1} drifts {lum_diff:.1} from anchor {:.1} (>35.0)",
                        anc.mean_luma
                    ),
                );
            }
            if sat_diff > 40.0 {
                push_issue(
                    "WARN",
                    "anchor_sat_drift",
                    None,
                    format!(
                        "average saturation {mean_sat:.1} drifts {sat_diff:.1} from anchor {:.1} (>40.0)",
                        anc.mean_sat
                    ),
                );
            }
            if fill_rel_diff > 0.40 {
                push_issue(
                    "WARN",
                    "anchor_scale_drift",
                    None,
                    format!(
                        "foreground fill ratio {:.3} deviates {:.1}% from anchor {:.3} (>40%)",
                        mean_fill,
                        fill_rel_diff * 100.0,
                        anc.fill_ratio
                    ),
                );
            }
            json!({
                "anchor_luma": (anc.mean_luma * 10.0).round() / 10.0,
                "anchor_sat": (anc.mean_sat * 10.0).round() / 10.0,
                "anchor_fill": (anc.fill_ratio * 10000.0).round() / 10000.0,
                "luma_diff": (lum_diff * 10.0).round() / 10.0,
                "sat_diff": (sat_diff * 10.0).round() / 10.0,
                "fill_rel_diff_pct": (fill_rel_diff * 1000.0).round() / 10.0,
            })
        } else {
            json!({"error": "anchor image is empty"})
        }
    } else {
        Value::Null
    };

    let status = if has_fail {
        "FAIL"
    } else if has_warn {
        "WARN"
    } else {
        "PASS"
    };

    json!({
        "sprite_id": sprite_id,
        "sheet": sheet_label,
        "status": status,
        "frame": [opts.frame_w, opts.frame_h],
        "foot": opts.foot,
        "issues": issues,
        "metrics": {
            "width": sheet.w,
            "height": sheet.h,
            "cols": cols,
            "rows": rows,
            "frames_total": cols * rows,
            "frames_non_empty": non_empty_count,
            "max_foot_drift": max_foot_drift_seen,
            "max_chroma_pct": (max_chroma_pct_seen * 100.0).round() / 100.0,
            "median_height": median_height,
            "mean_luma": (mean_luma * 10.0).round() / 10.0,
            "mean_sat": (mean_sat * 10.0).round() / 10.0,
            "mean_fill": (mean_fill * 10000.0).round() / 10000.0,
            "anchor": anchor_json,
        },
        "frames": frames_json,
    })
}

fn infer_sprite_id(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("sprite");
    if stem == "sheet" || stem == "sliced" || stem == "keyed" {
        if let Some(parent) = path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) {
            if !parent.is_empty() {
                return parent.to_string();
            }
        }
    }
    stem.to_string()
}

fn cmd_check(args: &Args) -> Result<()> {
    let sheet_path_s = args.pos(2).context(
        "usage: aegis sprite check <sheet.png> [ref=<anchor.png>] [frame=112x96] [foot=93] [max_foot_drift=2] [max_chroma_pct=1.0] [out=<sprite_check.json>] [json]",
    )?;
    let sheet_path = Path::new(sheet_path_s);
    let (frame_w, frame_h) = parse_frame_dims(&args.key("frame", "112x96"))?;
    let foot: usize = args.key("foot", "93").parse().context("foot")?;
    let max_foot_drift: usize = args.key("max_foot_drift", "2").parse().context("max_foot_drift")?;
    let max_chroma_pct: f64 = args.key("max_chroma_pct", "1.0").parse().context("max_chroma_pct")?;

    let sheet = load_image(sheet_path)?;
    let anchor = match args.keys.get("ref") {
        Some(p) => Some(load_image(Path::new(p))?),
        None => None,
    };

    let sprite_id = args
        .keys
        .get("id")
        .cloned()
        .unwrap_or_else(|| infer_sprite_id(sheet_path));

    let opts = CheckOpts {
        frame_w,
        frame_h,
        foot,
        max_foot_drift,
        max_chroma_pct,
    };
    let report = check_sprite_sheet(&sheet, sheet_path_s, &sprite_id, anchor.as_ref(), &opts);

    if let Some(out_s) = args.keys.get("out") {
        let out_p = Path::new(out_s);
        if let Some(parent) = out_p.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_p, serde_json::to_string_pretty(&report)? + "\n")
            .with_context(|| format!("writing {}", out_p.display()))?;
    }

    if args.flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let m = &report["metrics"];
        let issues = report["issues"].as_array().map(Vec::len).unwrap_or(0);
        println!(
            "sprite check [{}]: {} ({}x{}, {}x{} frames, {}/{} non-empty, max_foot_drift={}px, max_chroma={}%, issues={})",
            report["status"].as_str().unwrap_or("INFO"),
            sheet_path_s,
            m["width"],
            m["height"],
            m["cols"],
            m["rows"],
            m["frames_non_empty"],
            m["frames_total"],
            m["max_foot_drift"],
            m["max_chroma_pct"],
            issues,
        );
        if let Some(arr) = report["issues"].as_array() {
            for iss in arr {
                println!(
                    "  [{}] {}",
                    iss["severity"].as_str().unwrap_or("INFO"),
                    iss["message"].as_str().unwrap_or("")
                );
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests (100% game-agnostic)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fill_rect(img: &mut RgbaImg, x0: usize, y0: usize, w: usize, h: usize, c: [u8; 4]) {
        for y in y0..(y0 + h).min(img.h) {
            for x in x0..(x0 + w).min(img.w) {
                img.set(x, y, c);
            }
        }
    }

    /// Builds a synthetic sprite with a torso, two legs (creating an interior hole between legs),
    /// and an optional 1px semi-blended chroma fringe along the outer contour.
    fn make_synthetic_subject_on_bg(w: usize, h: usize, bg: [u8; 4], subject: [u8; 4], fringe: Option<[u8; 4]>) -> RgbaImg {
        let mut img = RgbaImg::new(w, h, bg);
        // Head + torso
        fill_rect(&mut img, 12, 8, 16, 18, subject);
        // Left leg & right leg with interior background gap at x=18..22, y=26..36
        fill_rect(&mut img, 12, 26, 6, 10, subject);
        fill_rect(&mut img, 22, 26, 6, 10, subject);
        // Optional chroma-blended edge pixels along the top/left border of the torso
        if let Some(fc) = fringe {
            for x in 12..28 {
                img.set(x, 7, fc);
            }
        }
        img
    }

    #[test]
    fn chroma_auto_selects_green_magenta_blue() {
        // Default neutral/warm palette -> green
        let warm = vec!["#884422".into(), "#d8a878".into(), "#303848".into()];
        let c1 = resolve_chroma_key(&warm, "auto").unwrap();
        assert_eq!(c1.hex, "#00FF00");
        assert!(!c1.has_green);

        // Palette containing green -> magenta
        let with_green = vec!["#228833".into(), "#d8a878".into()];
        let c2 = resolve_chroma_key(&with_green, "auto").unwrap();
        assert_eq!(c2.hex, "#FF00FF");
        assert!(c2.has_green);
        assert!(!c2.has_magenta);

        // Palette containing both green and magenta -> blue
        let with_both = vec!["#228833".into(), "#c830b8".into()];
        let c3 = resolve_chroma_key(&with_both, "auto").unwrap();
        assert_eq!(c3.hex, "#0047FF");
        assert!(c3.has_green);
        assert!(c3.has_magenta);

        // Explicit overrides
        assert_eq!(resolve_chroma_key(&with_both, "green").unwrap().hex, "#00FF00");
        assert_eq!(resolve_chroma_key(&warm, "magenta").unwrap().hex, "#FF00FF");
        assert_eq!(resolve_chroma_key(&warm, "blue").unwrap().hex, "#0047FF");

        // Nested JSON extraction
        let v = json!({
            "palette": {"primary": "#209030", "secondary": "#404040"},
            "extra": ["#c030b0", "not_a_color"]
        });
        let mut extracted = Vec::new();
        collect_hex_colors_from_json(&v, &mut extracted);
        assert_eq!(extracted.len(), 3);
        assert_eq!(resolve_chroma_key(&extracted, "auto").unwrap().hex, "#0047FF");
    }

    #[test]
    fn key_handles_green_magenta_blue_and_white_backgrounds() {
        let subject = [120, 90, 75, 255];

        // 1. Green screen with interior hole between legs + greenish fringe
        let mut green_img = make_synthetic_subject_on_bg(
            40,
            44,
            [0, 255, 0, 255],
            subject,
            Some([80, 175, 50, 255]), // soft-zone greenish fringe (dist ~123, between 96 and 158)
        );
        let st_g = remove_solid_background_to_alpha(&mut green_img, 1, 1);
        assert_eq!(st_g.mode, "chroma_green");
        // Exterior background keyed
        assert_eq!(green_img.get(0, 0)[3], 0);
        // Interior hole between legs (x=20, y=30) punched out globally
        assert_eq!(green_img.get(20, 30)[3], 0);
        // Subject center preserved opaque
        assert_eq!(green_img.get(20, 16), subject);
        // Fringe despilled: green channel capped near max(r, b)
        let fpx = green_img.get(20, 7);
        assert!(fpx[3] == 0 || fpx[1] <= ((fpx[0].max(fpx[2]) as f64) * 1.05).ceil() as u8);

        // 2. Magenta screen
        let mut mag_img = make_synthetic_subject_on_bg(40, 44, [255, 0, 255, 255], subject, None);
        let st_m = remove_solid_background_to_alpha(&mut mag_img, 1, 1);
        assert_eq!(st_m.mode, "chroma_magenta");
        assert_eq!(mag_img.get(0, 0)[3], 0);
        assert_eq!(mag_img.get(20, 30)[3], 0);
        assert_eq!(mag_img.get(20, 16), subject);

        // 3. Blue screen
        let mut blue_img = make_synthetic_subject_on_bg(40, 44, [0, 71, 255, 255], subject, None);
        let st_b = remove_solid_background_to_alpha(&mut blue_img, 1, 1);
        assert_eq!(st_b.mode, "chroma_blue");
        assert_eq!(blue_img.get(0, 0)[3], 0);
        assert_eq!(blue_img.get(20, 30)[3], 0);
        assert_eq!(blue_img.get(20, 16), subject);

        // 4. White background fallback (exterior flood-fill preserves interior white eye/highlight)
        let mut white_img = make_synthetic_subject_on_bg(40, 44, [250, 250, 250, 255], subject, None);
        // Put an enclosed white highlight inside the torso
        fill_rect(&mut white_img, 18, 12, 4, 4, [255, 255, 255, 255]);
        let st_w = remove_solid_background_to_alpha(&mut white_img, 1, 1);
        assert_eq!(st_w.mode, "flood_fill");
        assert_eq!(white_img.get(0, 0)[3], 0);
        // Enclosed white highlight inside torso must NOT be erased by exterior flood-fill
        assert_eq!(white_img.get(19, 13)[3], 255);
        assert_eq!(white_img.get(14, 10), subject);
    }

    #[test]
    fn slice_aligns_feet_and_writes_valid_clips_json() {
        // Build a 4-column strip on green screen (each cell 64x64 -> 256x64 total).
        let mut strip = RgbaImg::new(256, 64, [0, 255, 0, 255]);
        for c in 0..4 {
            let ox = c * 64;
            // Varying vertical offsets in input; slicer must align all feet to row 93.
            let y_off = 6 + c * 3;
            fill_rect(&mut strip, ox + 20, y_off, 24, 40, [150, 80, 60, 255]);
        }

        let opts = SliceOpts {
            cols: 4,
            rows: 1,
            frame_w: 112,
            frame_h: 96,
            foot: 93,
            run_key: true,
            inset: 2,
            nearest: false,
        };
        let res = slice_sheet(&strip, &opts).unwrap();
        assert_eq!(res.image.w, 4 * 112);
        assert_eq!(res.image.h, 96);
        assert_eq!(res.cells.len(), 4);
        for cell in &res.cells {
            assert!(!cell.empty);
            assert_eq!(cell.bottom_y, Some(93));
        }

        // Verify nearest-neighbor path as well
        let opts_nearest = SliceOpts { nearest: true, ..opts };
        let res_n = slice_sheet(&strip, &opts_nearest).unwrap();
        assert_eq!(res_n.image.w, 4 * 112);
        for cell in &res_n.cells {
            assert_eq!(cell.bottom_y, Some(93));
        }

        // Verify generated clips.json passes engine Clips::parse
        let clips_val = build_clips_json(112, 96, 93, 4, 1, "idle", 8.0);
        let parsed = aegis_core::assets::Clips::parse(&clips_val).expect("valid clips.json");
        assert_eq!(parsed.frame, (112, 96));
        assert_eq!(parsed.foot, Some(93));
        assert!(parsed.painted);
        assert_eq!(parsed.clips.len(), 1);
        assert_eq!(parsed.clips[0].id, "idle");
        assert_eq!(parsed.clips[0].frames, 4);
    }

    #[test]
    fn check_passes_clean_sheet_and_catches_defects() {
        // Build a clean 4-frame sliced sheet first
        let mut strip = RgbaImg::new(256, 64, [0, 255, 0, 255]);
        for c in 0..4 {
            fill_rect(&mut strip, c * 64 + 18, 10, 28, 44, [140, 100, 80, 255]);
        }
        let sliced = slice_sheet(
            &strip,
            &SliceOpts {
                cols: 4,
                rows: 1,
                frame_w: 112,
                frame_h: 96,
                foot: 93,
                run_key: true,
                inset: 2,
                nearest: false,
            },
        )
        .unwrap();

        let check_opts = CheckOpts {
            frame_w: 112,
            frame_h: 96,
            foot: 93,
            max_foot_drift: 2,
            max_chroma_pct: 1.0,
        };
        let clean = check_sprite_sheet(&sliced.image, "clean.png", "unit_a", Some(&sliced.image), &check_opts);
        assert_eq!(clean["status"], "PASS");
        assert_eq!(clean["metrics"]["max_foot_drift"], 0);
        assert_eq!(clean["metrics"]["frames_non_empty"], 4);

        // 1. Misaligned feet -> FAIL
        let mut bad_foot = RgbaImg::new(112, 96, [0, 0, 0, 0]);
        fill_rect(&mut bad_foot, 30, 10, 40, 70, [130, 90, 70, 255]); // bottom_y = 79 vs foot 93
        let r_foot = check_sprite_sheet(&bad_foot, "bad_foot.png", "unit_b", None, &check_opts);
        assert_eq!(r_foot["status"], "FAIL");
        assert!(r_foot["issues"].as_array().unwrap().iter().any(|i| i["code"] == "foot_drift"));

        // 2. Chroma halo residue -> FAIL
        let mut halo = RgbaImg::new(112, 96, [0, 0, 0, 0]);
        fill_rect(&mut halo, 30, 20, 40, 74, [130, 90, 70, 255]); // bottom_y = 93
        // Paint a neon green halo around the border
        for y in 20..94 {
            halo.set(30, y, [0, 250, 0, 200]);
            halo.set(69, y, [0, 250, 0, 200]);
        }
        let r_halo = check_sprite_sheet(&halo, "halo.png", "unit_c", None, &check_opts);
        assert_eq!(r_halo["status"], "FAIL");
        assert!(r_halo["issues"].as_array().unwrap().iter().any(|i| i["code"] == "chroma_halo"));

        // 3. Bad dimensions -> FAIL
        let bad_dims = RgbaImg::new(100, 90, [100, 100, 100, 255]);
        let r_dims = check_sprite_sheet(&bad_dims, "bad_dims.png", "unit_d", None, &check_opts);
        assert_eq!(r_dims["status"], "FAIL");
        assert!(r_dims["issues"].as_array().unwrap().iter().any(|i| i["code"] == "bad_dimensions"));
    }

    #[test]
    fn batch_sheet_recognizes_sprite_check_run_dir() {
        let scratch = aegis_core::scratch::Scratch::new("sprite_batch_sheet");
        let run_dir = scratch.path().join("batch/unit_alpha");
        std::fs::create_dir_all(&run_dir).unwrap();

        let sheet = RgbaImg::new(112, 96, [0, 0, 0, 0]);
        save_png(&sheet, &run_dir.join("sheet.png")).unwrap();
        let report = json!({
            "sprite_id": "unit_alpha",
            "status": "WARN",
            "metrics": {
                "frames_total": 4,
                "frames_non_empty": 4,
                "max_foot_drift": 1,
                "max_chroma_pct": 0.25
            },
            "issues": [
                {"severity": "WARN", "code": "top_clip", "message": "frame (r0,c0): top-edge clipping"}
            ]
        });
        std::fs::write(
            run_dir.join("sprite_check.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        )
        .unwrap();

        assert!(batch_sheet::is_run_dir(&run_dir));
        let items = batch_sheet::build_batch_items(&[run_dir.clone()]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "unit_alpha");
        assert_eq!(items[0].status, "WARN");
        assert_eq!(items[0].stage.as_deref(), Some("sprite"));
        assert!(!items[0].previews.is_empty());
        assert_eq!(items[0].previews[0].0, "sheet");
        assert!(items[0].issues.iter().any(|s| s.contains("top-edge clipping")));

        let out_html = scratch.path().join("review/sprite_batch.html");
        let summary = batch_sheet::write_batch_sheet(&items, &out_html, "Sprite Batch", true).unwrap();
        assert_eq!(summary["counts"]["total"], 1);
        assert_eq!(summary["counts"]["warn"], 1);
        assert!(out_html.is_file());
    }
}
