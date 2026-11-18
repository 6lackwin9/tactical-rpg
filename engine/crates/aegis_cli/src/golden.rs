//! Scenario image goldens (PR7-4, SCENARIO-GOLDENS1): a `shot` step with `golden` compares the frame it took against a
//! committed baseline PNG, `<game folder>/scenarios/baselines/<scenario id>/<WxH>/<name>.png`. A pixel differs when one
//! channel (RGBA) is more than `tolerance` apart; the step fails when more than `pixels` differ, and the bundle keeps
//! `<name>.diff.png` (the differing pixels red over the baseline darkened). `record=1` writes the baselines instead: a
//! baseline change needs the user's approval, so it is a named re-record in the commit and `HISTORY.md`.

use std::path::Path;

/// How far a frame may drift from its baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tolerance {
    /// Per channel, 0 to 255.
    pub channel: u8,
    /// Pixels past `channel` allowed.
    pub pixels: u64,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self { channel: 0, pixels: 0 }
    }
}

/// An image as RGBA8 rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

pub fn read(path: &Path) -> Result<Rgba, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut dec = png::Decoder::new(std::io::BufReader::new(file));
    dec.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut reader = dec.read_info().map_err(|e| format!("{}: not a PNG: {e}", path.display()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("{}: {e}", path.display()))?;
    buf.truncate(info.buffer_size());
    let (w, h) = (info.width, info.height);
    let px = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::GrayscaleAlpha => buf.chunks(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
        other => return Err(format!("{}: colour type {other:?} after normalising", path.display())),
    };
    Ok(Rgba { w, h, px })
}

pub fn write(path: &Path, img: &Rgba) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.w, img.h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(|e| e.to_string())?;
    w.write_image_data(&img.px).map_err(|e| e.to_string())
}

/// What a comparison found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// Pixels past the channel tolerance.
    pub pixels: u64,
    /// The largest channel difference seen.
    pub max: u8,
    /// Their bounding box: x, y, w, h (None when none differ).
    pub bbox: Option<(u32, u32, u32, u32)>,
    /// The baseline darkened, differing pixels red.
    pub mask: Rgba,
}

/// `Err` when the sizes differ.
pub fn compare(shot: &Rgba, base: &Rgba, tol: Tolerance) -> Result<Diff, String> {
    if (shot.w, shot.h) != (base.w, base.h) {
        return Err(format!("the frame is {}x{}, the baseline {}x{}", shot.w, shot.h, base.w, base.h));
    }
    let mut mask = base.clone();
    let (mut pixels, mut max) = (0u64, 0u8);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (i, (a, b)) in shot.px.chunks(4).zip(base.px.chunks(4)).enumerate() {
        let d = a.iter().zip(b).map(|(p, q)| p.abs_diff(*q)).max().unwrap_or(0);
        max = max.max(d);
        let m = &mut mask.px[i * 4..i * 4 + 4];
        if d > tol.channel {
            pixels += 1;
            let (x, y) = (i as u32 % shot.w, i as u32 / shot.w);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            m.copy_from_slice(&[255, 0, 0, 255]);
        } else {
            for c in &mut m[..3] {
                *c /= 3;
            }
            m[3] = 255;
        }
    }
    let bbox = (pixels > 0).then(|| (x0, y0, x1 - x0 + 1, y1 - y0 + 1));
    Ok(Diff { pixels, max, bbox, mask })
}

/// One shot against its baseline: `Ok(None)` when it matches, `Ok(Some(message))` when it does not (the mask written to
/// `diff_out`), `Err` when either cannot be read.
pub fn check(shot: &Path, baseline: &Path, tol: Tolerance, diff_out: &Path) -> Result<Option<String>, String> {
    if !baseline.is_file() {
        return Err(format!("no baseline at {} (record it with record=1; a new or changed baseline needs the user's approval)", baseline.display()));
    }
    let d = compare(&read(shot)?, &read(baseline)?, tol)?;
    if d.pixels <= tol.pixels {
        return Ok(None);
    }
    let see = match write(diff_out, &d.mask) {
        Ok(()) => format!("see {}", diff_out.display()),
        Err(e) => format!("no diff image: {e}"),
    };
    let (x, y, w, h) = d.bbox.unwrap_or_default();
    Ok(Some(format!("{} px differ from the baseline by more than {} (largest {}), within x {x} y {y} w {w} h {h}; allowed {}; {see}", d.pixels, tol.channel, d.max, tol.pixels)))
}
