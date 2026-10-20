//! Godot's number formatting (String.sprintf `%.Nf`; `json::roundi` rounds for `%d`): the exact binary value rounded to N
//! decimals with ties away from zero (0.125 prints 0.13, 2.5 prints 3), where Rust's `{:.N}` rounds ties to even.
//! Tables and log lines that must read as Godot printed them format through here.

/// `%.<decimals>f`: `v` with `decimals` digits after the point (ties away from zero; a negative value keeps its sign,
/// also when it rounds to zero).
pub fn fixed(v: f64, decimals: usize) -> String {
    if v.is_nan() {
        return "nan".into();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-inf" } else { "inf" }.into();
    }
    let sign = if v.is_sign_negative() { "-" } else { "" };
    let a = v.abs();
    let plain = format!("{a:.decimals$}");
    if !is_tie(a, decimals) {
        return format!("{sign}{plain}");
    }
    // An exact tie: round the truncated digits up (away from zero) instead of to even.
    let exact = format!("{a:.prec$}", prec = decimals + 1);
    let truncated = &exact[..exact.len() - 1];
    let truncated = truncated.strip_suffix('.').unwrap_or(truncated);
    format!("{sign}{}", add_last_digit(truncated))
}

/// True when `a` (not negative) lies exactly halfway between two `decimals`-digit numbers.
fn is_tie(a: f64, decimals: usize) -> bool {
    let scaled = a * 10f64.powi(decimals as i32);
    if (scaled - scaled.floor() - 0.5).abs() > 1e-6 {
        return false;
    }
    // The exact decimal expansion (a double has at most 1074 digits after the point): "5" then only zeros.
    let exact = format!("{a:.1074}");
    let dot = exact.find('.').unwrap_or(exact.len());
    let rest = &exact[dot + 1 + decimals..];
    rest.starts_with('5') && rest[1..].bytes().all(|b| b == b'0')
}

/// "0.12" -> "0.13", "0.99" -> "1.00", "9" -> "10".
fn add_last_digit(digits: &str) -> String {
    let mut out: Vec<u8> = digits.bytes().collect();
    let mut i = out.len();
    while i > 0 {
        i -= 1;
        match out[i] {
            b'.' => continue,
            b'9' => out[i] = b'0',
            d => {
                out[i] = d + 1;
                return String::from_utf8(out).unwrap();
            }
        }
    }
    let mut s = String::from("1");
    s.push_str(std::str::from_utf8(&out).unwrap());
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_like_godot() {
        // Recorded from Godot 4.7 ("%.2f" / "%.1f" / "%.0f"). Not here: 1.115 and 12.345, which Godot printed 1.12 and
        // 12.35 because its own parser read those literals one unit in the last place high (known issues).
        for (v, d, want) in [
            (0.595, 2, "0.59"),
            (0.125, 2, "0.13"),
            (0.135, 2, "0.14"),
            (2.675, 2, "2.67"),
            (1.005, 2, "1.00"),
            (0.005, 2, "0.01"),
            (0.015, 2, "0.01"),
            (0.025, 2, "0.03"),
            (0.565, 2, "0.56"),
            (0.125, 1, "0.1"),
            (0.595, 1, "0.6"),
            (0.125, 0, "0"),
            (0.595, 0, "1"),
            (2.5, 0, "3"),
            (0.5, 0, "1"),
            (9.95, 1, "9.9"),
            (99.5, 0, "100"),
            (0.0, 2, "0.00"),
        ] {
            assert_eq!(fixed(v, d), want, "{v} at {d}");
        }
        assert_eq!(fixed(-2.5, 0), "-3");
        assert_eq!(fixed(-0.001, 2), "-0.00");
    }
}
