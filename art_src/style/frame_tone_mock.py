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


def corner_piece(px: int) -> Image.Image:
    src = Image.open(ROOT / "games/emberwake/assets/ui/frames/ornate_full.png").convert("RGBA")
    star = src.crop((8, 8, 136, 136))
    out = Image.new("RGBA", star.size)
    for y in range(star.height):
        for x in range(star.width):
            r, g, b, a = star.getpixel((x, y))
            if a == 0:
                continue
            t = min(1.0, (0.3 * r + 0.59 * g + 0.11 * b) / 200)
            c = tuple(int(BRONZE_DK[i] + (BRONZE[i] - BRONZE_DK[i]) * t) for i in range(3))
            out.putpixel((x, y), c + (a,))
    return out.resize((px, px), Image.LANCZOS)


def toned_settings(shot: Path, title: Path) -> Image.Image:
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
    d.rectangle((L, T, R, B), outline=BRONZE, width=1)
    d.rectangle((L, T, R, T + 1), fill=BLOOD)
    c = corner_piece(26)
    for flip, (x, y) in ((None, (L + 2, T + 2)), (Image.FLIP_LEFT_RIGHT, (R - 27, T + 2)),
                         (Image.FLIP_TOP_BOTTOM, (L + 2, B - 27)), (Image.ROTATE_180, (R - 27, B - 27))):
        p = c if flip is None else c.transpose(flip)
        img.paste(p, (x, y), p)
    d.rectangle((L, T, R, T + 1), fill=BLOOD)
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
