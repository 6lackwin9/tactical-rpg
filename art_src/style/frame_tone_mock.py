"""UX-36/37 frame tone mock: current ornate shell vs a toned-down one, painted over playtest shots.

Usage: python3 art_src/style/frame_tone_mock.py <playtest_dir> <out_dir>
Tier 1 (main panels): one dim bronze line, half-size corners kept inside the edge, 2 px oxblood top line.
Tier 2 (popups): one thin iron line, 2 px oxblood top line, no corners.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageEnhance

ROOT = Path(__file__).resolve().parents[2]
BRONZE = (116, 84, 60)      # rust #92400e mixed toward iron, about half the title gold's brightness
BRONZE_DK = (34, 26, 22)
IRON = (75, 85, 99)
BLOOD = (153, 27, 27)


def corner_piece(px: int, lo=None, hi=None) -> Image.Image:
    lo, hi = lo or BRONZE_DK, hi or BRONZE
    src = Image.open(ROOT / "games/emberwake/assets/ui/frames/ornate_full.png").convert("RGBA")
    star = src.crop((8, 8, 136, 136))
    out = Image.new("RGBA", star.size)
    for y in range(star.height):
        for x in range(star.width):
            r, g, b, a = star.getpixel((x, y))
            if a == 0:
                continue
            t = min(1.0, (0.3 * r + 0.59 * g + 0.11 * b) / 200)
            c = tuple(int(lo[i] + (hi[i] - lo[i]) * t) for i in range(3))
            out.putpixel((x, y), c + (a,))
    return out.resize((px, px), Image.LANCZOS)


def toned_settings(shot: Path, title: Path, line=BRONZE, lw=1, cpx=26, chi=None) -> Image.Image:
    cur = Image.open(shot).convert("RGB")
    ttl = Image.open(title).convert("RGB")
    # background outside the panel: the title art dimmed like the shot
    f = sum(cur.getpixel((1250, 400))) / max(1, sum(ttl.getpixel((1250, 400))))
    bg = ImageEnhance.Brightness(ttl).enhance(min(1.0, f))
    img = bg.copy()
    L, T, R, B = 61, 33, 1218, 686
    inner = cur.crop((L + 6, T + 6, R - 5, B - 5))
    d = ImageDraw.Draw(img)
    d.rectangle((L, T, R, B), fill=(9, 10, 15))
    img.paste(inner, (L + 6, T + 6))
    d.rectangle((L, T, R, B), outline=line, width=lw)
    d.rectangle((L, T, R, T + 1), fill=BLOOD)
    c = corner_piece(cpx, hi=chi)
    o = cpx + 1
    for flip, (x, y) in ((None, (L + 2, T + 2)), (Image.FLIP_LEFT_RIGHT, (R - o, T + 2)),
                         (Image.FLIP_TOP_BOTTOM, (L + 2, B - o)), (Image.ROTATE_180, (R - o, B - o))):
        p = c if flip is None else c.transpose(flip)
        img.paste(p, (x, y), p)
    d.rectangle((L, T, R, T + 1), fill=BLOOD)
    return img


def middle_settings(shot: Path, title: Path) -> Image.Image:
    """Take 2 middle: the current corner star itself, dimmed to bronze, kept inside the panel edge; one line."""
    cur = Image.open(shot).convert("RGB")
    ttl = Image.open(title).convert("RGB")
    f = sum(cur.getpixel((1250, 400))) / max(1, sum(ttl.getpixel((1250, 400))))
    img = ImageEnhance.Brightness(ttl).enhance(min(1.0, f))
    X0, Y0, X1, Y1 = 40, 12, 100, 72          # current top-left star box; frame line crosses at (62, 34)
    star = Image.new("RGBA", (X1 - X0, Y1 - Y0))
    for y in range(Y0, Y1):
        for x in range(X0, X1):
            r, g, b = cur.getpixel((x, y))
            if (x >= 80 and y >= 48) or (x >= 68 and y >= 40) or max(r, g, b) < 34 \
                    or (31 <= y <= 40 and x >= 76) or (59 <= x <= 68 and y >= 50):
                continue
            v = (0.3 * r + 0.59 * g + 0.11 * b) / 255
            c = (int(40 + 150 * v), int(30 + 112 * v), int(24 + 70 * v))   # gold to medium bronze
            star.putpixel((x - X0, y - Y0), c + (255,))
    cx, cy = 62 - X0, 34 - Y0                 # star centre inside its box
    # panel edge = star tips; line runs through the star centres, so nothing pokes past the edge
    L, T, R, B = 61, 33, 1218, 686
    Lo, To, Ro, Bo = L - cx, T - cy, R + cx, B + cy
    inner = cur.crop((L + 6, T + 6, R - 5, B - 5))
    d = ImageDraw.Draw(img)
    d.rectangle((Lo, To, Ro, Bo), fill=(9, 10, 15))
    img.paste(inner, (L + 6, T + 6))
    line = (150, 114, 74)
    d.rectangle((L, T, R, B), outline=line, width=2)
    w, h = star.size
    d.rectangle((L + w - cx, T, R - w + cx, T + 1), fill=BLOOD)
    for flip, (x, y) in ((None, (L - cx, T - cy)), (Image.FLIP_LEFT_RIGHT, (R + cx - w + 1, T - cy)),
                         (Image.FLIP_TOP_BOTTOM, (L - cx, B + cy - h + 1)), (Image.ROTATE_180, (R + cx - w + 1, B + cy - h + 1))):
        st = star if flip is None else star.transpose(flip)
        img.paste(st, (x, y), st)
    d.rectangle((Lo, To, Ro, Bo), outline=(30, 26, 24), width=1)
    return img


def plain_card(shot: Path) -> Image.Image:
    img = Image.open(shot).convert("RGB")
    L, T, R, B = 341, 225, 939, 495
    d = ImageDraw.Draw(img)
    fill = img.getpixel((L + 20, T + 200))
    for k in range(0, 8):
        d.rectangle((L + k, T + k, R - k, B - k), outline=fill)
    d.rectangle((L, T, R, B), outline=IRON, width=1)
    d.rectangle((L, T, R, T + 1), fill=BLOOD)
    return img


def popup_border(shot: Path, style: str) -> Image.Image:
    """Tier 2 take 2 (Tushar 2026-11-18: plain line too weak): a proper border, no ornate corners."""
    img = Image.open(shot).convert("RGB")
    L, T, R, B = 341, 225, 939, 495
    d = ImageDraw.Draw(img)
    fill = img.getpixel((L + 20, T + 200))
    for k in range(0, 8):
        d.rectangle((L + k, T + k, R - k, B - k), outline=fill)
    if style == "double":      # bronze outer line, dark gap, iron inner line
        d.rectangle((L, T, R, B), outline=BRONZE, width=2)
        d.rectangle((L + 4, T + 4, R - 4, B - 4), outline=IRON, width=1)
    elif style == "bevel":     # 4 px bronze band, lit top-left, shaded bottom-right
        lit, mid, dk = (150, 112, 80), BRONZE, (62, 45, 34)
        d.rectangle((L, T, R, B), outline=mid, width=4)
        d.line((L, T, R, T), fill=lit); d.line((L, T, L, B), fill=lit)
        d.line((L, B, R, B), fill=dk); d.line((R, T, R, B), fill=dk)
        d.rectangle((L + 4, T + 4, R - 4, B - 4), outline=(14, 12, 12), width=1)
    elif style == "band":      # 3 px iron band between two dark bronze hairlines
        d.rectangle((L, T, R, B), outline=BRONZE_DK, width=1)
        d.rectangle((L + 1, T + 1, R - 1, B - 1), outline=IRON, width=3)
        d.rectangle((L + 4, T + 4, R - 4, B - 4), outline=BRONZE, width=1)
    d.rectangle((L, T, R, T + 1), fill=BLOOD)
    return img


def label(img: Image.Image, text: str) -> Image.Image:
    out = Image.new("RGB", (img.width, img.height + 34), (20, 23, 31))
    out.paste(img, (0, 34))
    ImageDraw.Draw(out).text((14, 10), text, fill=(217, 203, 180))
    return out


def main() -> None:
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    cur = Image.open(pt / "003_click_menu_settings.png").convert("RGB")
    new = toned_settings(pt / "003_click_menu_settings.png", pt / "002_title.png")
    new.save(out / "settings_toned.png")
    card_cur = Image.open(pt / "249_click_bunk_bunk_1.png").convert("RGB")
    card_new = plain_card(pt / "249_click_bunk_bunk_1.png")
    card_new.save(out / "bunk_card_plain.png")
    MID = (168, 128, 82)   # medium bronze-gold: clearly visible, below the title gold
    mid = middle_settings(pt / "003_click_menu_settings.png", pt / "002_title.png")
    mid.save(out / "settings_middle.png")
    A, Bm = label(cur, "Now: double gold line, stars poke out past the edge"), label(
        mid, "Middle: same stars and size, dimmed to bronze, inside the panel edge; one bronze line; oxblood top")
    sheet = Image.new("RGB", (A.width + Bm.width + 12, A.height), (10, 11, 16))
    sheet.paste(A, (0, 0)); sheet.paste(Bm, (A.width + 12, 0))
    sheet.save(out / "frame_main_middle_side_by_side.png")
    zc = (30, 10, 230, 140)
    za = label(cur.crop(zc).resize((800, 520), Image.NEAREST), "Now, top-left corner x4")
    zb = label(mid.crop(zc).resize((800, 520), Image.NEAREST), "Middle, top-left corner x4")
    z = Image.new("RGB", (za.width * 2 + 12, za.height), (10, 11, 16))
    z.paste(za, (0, 0)); z.paste(zb, (za.width + 12, 0))
    z.save(out / "frame_main_middle_corner_zoom.png")
    crop = (300, 190, 980, 530)
    takes = [label(card_cur.crop(crop), "Now")] + [
        label(popup_border(pt / "249_click_bunk_bunk_1.png", st).crop(crop), f"{k}: {txt}")
        for k, st, txt in (("A", "double", "double line, bronze outside, iron inside"),
                           ("B", "bevel", "4 px bevelled bronze edge"),
                           ("C", "band", "3 px iron band in bronze hairlines"))]
    w, h = takes[0].size
    sheet = Image.new("RGB", (w * 2 + 12, h * 2 + 12), (10, 11, 16))
    for i, t in enumerate(takes):
        sheet.paste(t, ((i % 2) * (w + 12), (i // 2) * (h + 12)))
    sheet.save(out / "frame_popup_take2.png")
    for name, a, b, la, lb in (
        ("frame_settings_side_by_side.png", cur, new, "Now: bright double gold line, stars past the edge",
         "UX-36: one dim bronze line, small corners inside, oxblood top line"),
        ("frame_popup_side_by_side.png", card_cur.crop((300, 190, 980, 530)), card_new.crop((300, 190, 980, 530)),
         "Now: popup", "UX-37: plain iron line, oxblood top line, no corners")):
        A, Bm = label(a, la), label(b, lb)
        sheet = Image.new("RGB", (A.width + Bm.width + 12, A.height), (10, 11, 16))
        sheet.paste(A, (0, 0)); sheet.paste(Bm, (A.width + 12, 0))
        sheet.save(out / name)


if __name__ == "__main__":
    main()
