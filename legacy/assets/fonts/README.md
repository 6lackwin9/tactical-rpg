# Web fallback fonts

Loaded only in the browser build by `scripts/ui/runner/web_fonts.gd` (`WebFonts.install`), as fallbacks of the default font, so emoji and symbol glyphs do not render as boxes. Desktop builds use the OS fonts instead.

| File | Source | Coverage | License |
| --- | --- | --- | --- |
| `dejavu_sans_symbols.ttf` | DejaVu Sans, subset U+2190-23FF, 2500-27BF, 27F0-27FF, 2900-297F, 2B00-2BFF | arrows, box drawing, shapes, dingbats | `DEJAVU_LICENSE.txt` |
| `noto_emoji.ttf` | Noto Emoji (google/fonts), full | emoji, monochrome | `OFL.txt` |
| `noto_symbols2_sub.ttf` | Noto Sans Symbols 2, subset U+2190-27BF, 2B00-2BFF | technical symbols such as ⏻ | `OFL.txt` |
| `noto_symbols_sub.ttf` | Noto Sans Symbols, subset U+2600-27BF | ⛶ | `OFL.txt` |
| `noto_math_sub.ttf` | Noto Sans Math, subset U+2900-2AFF | ⧉ | `OFL.txt` |

Subsets were made with `pyftsubset --unicodes=...`. When a new glyph shows as a box in the browser, add its range to a subset (or a new fallback) and extend the glyph list in `runner_test_suite.gd`.
