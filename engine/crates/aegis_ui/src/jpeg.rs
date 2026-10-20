//! A small baseline JPEG encoder (JFIF, 4:4:4, the standard quantisation and Huffman tables of ITU T.81 Annex K),
//! dependency free: playtest note screenshots (decision "Playtest feedback loop": JPG at quality 0.85).

const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61,
    54, 47, 55, 62, 63,
];

const LUMA_Q: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56, 14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92, 49, 64, 78, 87,
    103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

const CHROMA_Q: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99, 47, 66, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

const DC_LUMA_BITS: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const DC_CHROMA_BITS: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
const DC_VALUES: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const AC_LUMA_BITS: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
const AC_LUMA_VALUES: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62,
    0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57,
    0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
    0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    0xd9, 0xda, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
];
const AC_CHROMA_BITS: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77];
const AC_CHROMA_VALUES: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0, 0x15, 0x62, 0x72,
    0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19, 0x1a, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56,
    0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6,
    0xd7, 0xd8, 0xd9, 0xda, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
];

/// A Huffman table: (code, length) per symbol.
struct Table([(u16, u8); 256]);

impl Table {
    fn new(bits: &[u8; 16], values: &[u8]) -> Self {
        let mut t = [(0u16, 0u8); 256];
        let (mut code, mut k) = (0u16, 0usize);
        for (len, n) in bits.iter().enumerate() {
            for _ in 0..*n {
                t[values[k] as usize] = (code, len as u8 + 1);
                code += 1;
                k += 1;
            }
            code <<= 1;
        }
        Table(t)
    }
}

struct Bits {
    out: Vec<u8>,
    acc: u32,
    n: u32,
}

impl Bits {
    fn put(&mut self, code: u32, len: u32) {
        for i in (0..len).rev() {
            self.acc = (self.acc << 1) | ((code >> i) & 1);
            self.n += 1;
            if self.n == 8 {
                let b = self.acc as u8;
                self.out.push(b);
                if b == 0xff {
                    self.out.push(0);
                }
                self.acc = 0;
                self.n = 0;
            }
        }
    }

    fn flush(&mut self) {
        while self.n != 0 {
            self.put(1, 1);
        }
    }
}

/// The scaled quantisation table of `base` for `quality` 1..100 (IJG scaling), natural order.
fn scaled(base: &[u16; 64], quality: u32) -> [u16; 64] {
    let q = quality.clamp(1, 100);
    let scale = if q < 50 { 5000 / q } else { 200 - 2 * q };
    let mut out = [0u16; 64];
    for (o, b) in out.iter_mut().zip(base) {
        *o = ((*b as u32 * scale + 50) / 100).clamp(1, 255) as u16;
    }
    out
}

fn category(v: i32) -> u32 {
    32 - v.unsigned_abs().leading_zeros()
}

fn encode_block(bits: &mut Bits, block: &[f32; 64], q: &[u16; 64], dc: &Table, ac: &Table, prev: &mut i32, cos: &[[f32; 8]; 8]) {
    // Forward DCT, separable: rows then columns.
    let mut tmp = [0f32; 64];
    for y in 0..8 {
        for u in 0..8 {
            let s: f32 = (0..8).map(|x| block[y * 8 + x] * cos[u][x]).sum();
            tmp[y * 8 + u] = s;
        }
    }
    let mut coef = [0i32; 64];
    for u in 0..8 {
        for v in 0..8 {
            let s: f32 = (0..8).map(|y| tmp[y * 8 + u] * cos[v][y]).sum();
            let cu = if u == 0 { std::f32::consts::FRAC_1_SQRT_2 } else { 1.0 };
            let cv = if v == 0 { std::f32::consts::FRAC_1_SQRT_2 } else { 1.0 };
            let f = 0.25 * cu * cv * s;
            coef[v * 8 + u] = (f / q[v * 8 + u] as f32).round() as i32;
        }
    }
    let emit = |bits: &mut Bits, t: &Table, sym: usize, v: i32, cat: u32| {
        let (code, len) = t.0[sym];
        bits.put(code as u32, len as u32);
        if cat > 0 {
            let raw = if v < 0 { v - 1 } else { v };
            bits.put((raw as u32) & ((1 << cat) - 1), cat);
        }
    };
    let d = coef[0] - *prev;
    *prev = coef[0];
    let cat = category(d);
    emit(bits, dc, cat as usize, d, cat);
    let mut run = 0;
    let last = (1..64).rev().find(|&k| coef[ZIGZAG[k]] != 0).unwrap_or(0);
    for k in 1..=last {
        let v = coef[ZIGZAG[k]];
        if v == 0 {
            run += 1;
            continue;
        }
        while run > 15 {
            emit(bits, ac, 0xf0, 0, 0);
            run -= 16;
        }
        let cat = category(v);
        emit(bits, ac, ((run << 4) | cat) as usize, v, cat);
        run = 0;
    }
    if last < 63 {
        emit(bits, ac, 0x00, 0, 0);
    }
}

/// The JPEG file of an RGBA image (`px` row-major, 4 bytes a pixel) at `quality` 1..100.
pub fn encode_rgba(w: usize, h: usize, px: &[u8], quality: u32) -> Vec<u8> {
    let (ql, qc) = (scaled(&LUMA_Q, quality), scaled(&CHROMA_Q, quality));
    let mut out: Vec<u8> = vec![0xff, 0xd8, 0xff, 0xe0, 0, 16, b'J', b'F', b'I', b'F', 0, 1, 1, 0, 0, 1, 0, 1, 0, 0];
    out.extend_from_slice(&[0xff, 0xdb, 0, 132, 0]);
    out.extend(ZIGZAG.iter().map(|&i| ql[i] as u8));
    out.push(1);
    out.extend(ZIGZAG.iter().map(|&i| qc[i] as u8));
    out.extend_from_slice(&[0xff, 0xc0, 0, 17, 8, (h >> 8) as u8, h as u8, (w >> 8) as u8, w as u8, 3, 1, 0x11, 0, 2, 0x11, 1, 3, 0x11, 1]);
    let tables: [(u8, &[u8; 16], &[u8]); 4] = [(0x00, &DC_LUMA_BITS, &DC_VALUES), (0x10, &AC_LUMA_BITS, &AC_LUMA_VALUES), (0x01, &DC_CHROMA_BITS, &DC_VALUES), (0x11, &AC_CHROMA_BITS, &AC_CHROMA_VALUES)];
    let len: usize = 2 + tables.iter().map(|(_, _, v)| 17 + v.len()).sum::<usize>();
    out.extend_from_slice(&[0xff, 0xc4, (len >> 8) as u8, len as u8]);
    for (class, bits, values) in tables {
        out.push(class);
        out.extend_from_slice(bits);
        out.extend_from_slice(values);
    }
    out.extend_from_slice(&[0xff, 0xda, 0, 12, 3, 1, 0x00, 2, 0x11, 3, 0x11, 0, 63, 0]);
    let (dcl, acl, dcc, acc) = (Table::new(&DC_LUMA_BITS, &DC_VALUES), Table::new(&AC_LUMA_BITS, &AC_LUMA_VALUES), Table::new(&DC_CHROMA_BITS, &DC_VALUES), Table::new(&AC_CHROMA_BITS, &AC_CHROMA_VALUES));
    let mut cos = [[0f32; 8]; 8];
    for (u, row) in cos.iter_mut().enumerate() {
        for (x, c) in row.iter_mut().enumerate() {
            *c = (((2 * x + 1) * u) as f32 * std::f32::consts::PI / 16.0).cos();
        }
    }
    let mut bits = Bits { out: Vec::new(), acc: 0, n: 0 };
    let mut prev = [0i32; 3];
    for by in (0..h).step_by(8) {
        for bx in (0..w).step_by(8) {
            let mut blocks = [[0f32; 64]; 3];
            for y in 0..8 {
                for x in 0..8 {
                    // Edge blocks repeat the last row and column.
                    let i = ((by + y).min(h - 1) * w + (bx + x).min(w - 1)) * 4;
                    let (r, g, b) = (px[i] as f32, px[i + 1] as f32, px[i + 2] as f32);
                    blocks[0][y * 8 + x] = 0.299 * r + 0.587 * g + 0.114 * b - 128.0;
                    blocks[1][y * 8 + x] = -0.168736 * r - 0.331264 * g + 0.5 * b;
                    blocks[2][y * 8 + x] = 0.5 * r - 0.418688 * g - 0.081312 * b;
                }
            }
            encode_block(&mut bits, &blocks[0], &ql, &dcl, &acl, &mut prev[0], &cos);
            encode_block(&mut bits, &blocks[1], &qc, &dcc, &acc, &mut prev[1], &cos);
            encode_block(&mut bits, &blocks[2], &qc, &dcc, &acc, &mut prev[2], &cos);
        }
    }
    bits.flush();
    out.extend(bits.out);
    out.extend_from_slice(&[0xff, 0xd9]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_huffman_tables_are_complete() {
        for (bits, values) in [(&DC_LUMA_BITS, &DC_VALUES[..]), (&DC_CHROMA_BITS, &DC_VALUES[..]), (&AC_LUMA_BITS, &AC_LUMA_VALUES[..]), (&AC_CHROMA_BITS, &AC_CHROMA_VALUES[..])] {
            assert_eq!(bits.iter().map(|b| *b as usize).sum::<usize>(), values.len());
            let mut seen = values.to_vec();
            seen.sort();
            seen.dedup();
            assert_eq!(seen.len(), values.len(), "every symbol once");
        }
    }

    #[test]
    fn a_file_has_the_jfif_frame() {
        let (w, h) = (13, 9);
        let px: Vec<u8> = (0..w * h).flat_map(|i| [(i * 7) as u8, (i * 3) as u8, 200, 255]).collect();
        let jpg = encode_rgba(w, h, &px, 85);
        assert_eq!(&jpg[..4], &[0xff, 0xd8, 0xff, 0xe0]);
        assert_eq!(&jpg[jpg.len() - 2..], &[0xff, 0xd9]);
        let sof = jpg.windows(2).position(|p| p == [0xff, 0xc0]).unwrap();
        assert_eq!((jpg[sof + 5], jpg[sof + 6], jpg[sof + 7], jpg[sof + 8]), (0, 9, 0, 13), "height then width");
    }
}

#[cfg(test)]
mod pillow_check {
    /// Writes a test image as JPEG for an outside decoder check (`AEGIS_JPEG_OUT=<dir> cargo test`).
    #[test]
    fn write_for_an_outside_decoder() {
        let Ok(dir) = std::env::var("AEGIS_JPEG_OUT") else { return };
        let (w, h) = (101usize, 67usize);
        let px: Vec<u8> = (0..w * h).flat_map(|i| {
            let (x, y) = (i % w, i / w);
            [(x * 255 / w) as u8, (y * 255 / h) as u8, if (x / 8 + y / 8) % 2 == 0 { 40 } else { 220 }, 255]
        }).collect();
        std::fs::write(format!("{dir}/check.rgba"), &px).unwrap();
        std::fs::write(format!("{dir}/check.jpg"), super::encode_rgba(w, h, &px, 85)).unwrap();
    }
}
