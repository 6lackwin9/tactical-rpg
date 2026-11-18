"""AI-21 battle mock take 16 (Tushar 2026-11-18 22:49): the combined battle screen at true size, 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take16_mock.py <playtest_dir> <out_dir>
Everything signed off in takes 9-15 on one screen: the kind-coloured strip with the ghost faded and the chips on the
tile edge; party plates with a status column inside on the right, a Guests block; foe plates with the status column
inside on the left, no "who hits it" ruler (the card and the timeline carry that); the foe panel header with the count
and the next reinforcement; the objective under the round; the docked foe card (take 14); a ground hazard and a push
preview on the board. Icons use integer scales of the 64 px sources (skills 32, statuses 16) to stay crisp at 1:1.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take13_mock as T13  # noqa: E402
import battle_take14_mock as T14  # noqa: E402
import battle_take15_mock as T15  # noqa: E402

ST = T.ST
FOE_RIM = (170, 60, 50)
STC = 26                      # the status column inside a plate


def icon16(status):
    return Image.open(ST / f"{status}.png").convert("RGBA").resize((16, 16), Image.BOX)


def tab(img, d, x, y, st, n, inc):
    """A status tab inside the plate's status column: 16 px icon, rim by kind, rounds as up to 3 pips."""
    kind = T12.KIND[st]
    col = T12.rim(kind)
    d.rectangle((x, y, x + 24, y + 19), fill=(16, 18, 25))
    if inc:
        for k in range(0, 25, 3):
            d.point((x + k, y), fill=col); d.point((x + k, y + 19), fill=col)
        for k in range(0, 20, 3):
            d.point((x, y + k), fill=col); d.point((x + 24, y + k), fill=col)
    else:
        d.rectangle((x, y, x + 24, y + 19), outline=col)
    ic = icon16(st)
    img.paste(ic, (x + 2, y + 2), ic)
    if n < 0:
        d.rectangle((x + 19, y + 2, x + 23, y + 17), fill=col)
    else:
        for k in range(3):                       # 5x5 round pips, bottom up (take 16 legibility fix)
            yy = y + 2 + (2 - k) * 6
            d.rectangle((x + 19, yy, x + 23, yy + 4), fill=col if k < n else B.AP_OFF)


def status_col(img, d, x, y, h, sts):
    o = T12.ordered(sts)
    fit = (h - 2) // 21
    show = o if len(o) <= fit else o[:fit - 1]
    for i, (st, n, inc) in enumerate(show):
        tab(img, d, x, y + 1 + i * 21, st, n, inc)
    if len(o) > len(show):
        yy = y + 1 + len(show) * 21
        d.rectangle((x, yy, x + 24, yy + 19), fill=(16, 18, 25), outline=B.IRON_L)
        d.text((x + 4, yy + 3), f"+{len(o) - len(show)}", font=B.MONO, fill=B.PARCH)


def hero_plate(img, d, x, y, name, hp, mx, ap_spent, sts, sel=False, loss=0):
    w, h = 152, 84
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=B.GOLD if sel else B.IRON)
    img.paste(T.face({"Mara": "mara_voss"}.get(name, name.lower()), 44), (x + 5, y + 5))
    d.text((x + 54, y + 3), name, font=B.BODY, fill=B.PARCH)
    bw = w - STC - 58
    B.bar(d, x + 54, y + 26, bw, 6, hp / mx, B.HP)
    if loss:
        l = round(bw * loss / mx)
        e = x + 54 + round(bw * hp / mx)
        d.rectangle((e - l, y + 26, e, y + 32), fill=B.IMPACT)
    d.text((x + 54, y + 35), f"{hp}/{mx}", font=B.MONO, fill=B.PARCH)
    B.ap_pips(d, x + 54, y + 54, 10, 10, size=5, gap=2, spent=ap_spent)
    B.intent_tile(d, x + 5, y + 52, s=16)
    B.poise(d, x + 5, y + 75, 5, 5, w=(w - STC - 14) // 5 - 2, h=4)
    d.line((x + w - STC - 2, y + 4, x + w - STC - 2, y + h - 4), fill=B.BASALT)
    status_col(img, d, x + w - STC + 1, y + 2, h - 4, sts)


def guest_plate(img, d, x, y, face, name, hp, mx, protect):
    w, h = 152, 52
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=T15.GUEST)
    img.paste(face.resize((40, 40)), (x + 5, y + 6))
    d.text((x + 50, y + 3), name, font=B.SMALL, fill=B.PARCH)
    B.bar(d, x + 50, y + 21, w - STC - 56, 5, hp / mx, B.HP)
    d.rounded_rectangle((x + 50, y + 31, x + 68, y + 45), radius=3, fill=(30, 60, 60))
    d.text((x + 53, y + 31), "AI", font=T12.TINY, fill=B.PARCH)
    if protect:
        d.rounded_rectangle((x + 72, y + 31, x + 120, y + 45), radius=3, outline=B.GOLD)
        d.text((x + 76, y + 31), "Protect", font=T12.TINY, fill=B.GOLD)
    d.line((x + w - STC - 2, y + 4, x + w - STC - 2, y + h - 4), fill=B.BASALT)
    status_col(img, d, x + w - STC + 1, y + 4, h - 8, [("fortify", 1, False)])


def foe_plate(img, d, x, y, face, name, hp, mx, pips, sts, sel=False, compact=False):
    w = 152
    h = 52 if compact else 70
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=FOE_RIM if sel else B.IRON)
    status_col(img, d, x + 2, y + 2, h - 4, sts)
    d.line((x + STC + 2, y + 4, x + STC + 2, y + h - 4), fill=B.BASALT)
    px = 40 if compact else 44
    fx = x + STC + 5
    img.paste(face.resize((px, px)), (fx, y + 5))
    tx = fx + px + 5
    fnt = B.BODY if not compact and d.textlength(name, font=B.BODY) <= x + w - 6 - tx else B.SMALL
    d.text((tx, y + 2 if fnt == B.BODY else y + 4), name, font=fnt, fill=B.PARCH)
    bw = x + w - 6 - tx
    yb = y + 20 if compact else y + 24
    B.bar(d, tx, yb, bw, 5, hp / mx, (150, 50, 45))
    d.text((tx, yb + 7), f"{hp}/{mx}", font=T12.TINY if compact else B.MONO, fill=B.PARCH)
    B.poise(d, tx, y + h - 8, 5, pips, w=(bw - 8) // 5, h=3)


def tile(img, d, x, icon, blocks, ap, ghost=False, on=False, chips=()):
    """The top bar's strip tile: kind colour (ghost faded), 32 px icon, AP pips, the block ruler under it."""
    W, G = 54, 6
    w = blocks * W + (blocks - 1) * G
    base = T13.KIND_COL[T13.SKILL_KIND[icon]]
    fill = T13.mix(base, B.BASALT, T13.GHOST) if ghost else base
    d.rectangle((x, 6, x + w, 52), fill=fill)
    if on:
        d.rectangle((x, 6, x + w, 52), outline=B.GOLD, width=2)
    t = B.slot_icon(icon, 64).resize((32, 32), Image.BOX)
    if ghost:
        t.putalpha(t.getchannel("A").point(lambda a: round(a * 150 / 255)))
    ix = x + (w - 32) // 2
    img.paste(t, (ix, 8), t)
    pw = ap * 9 - 2
    for i in range(ap):
        px = x + (w - pw) // 2 + i * 9
        if ghost:
            d.rectangle((px, 43, px + 6, 49), outline=T13.mix(B.AP_ON, fill, 0.7))
        else:
            d.rectangle((px, 43, px + 6, 49), fill=B.AP_ON, outline=(12, 12, 16))
    for b in range(blocks):
        bx = x + b * (W + G)
        c = fill if ghost else (B.GOLD if on else B.IRON_L)
        d.rectangle((bx, 56, bx + W, 60), fill=c)
        if b < blocks - 1:
            d.rectangle((bx + W, 57, bx + W + G, 59), fill=c)
    cx = x + w - 3 - len(chips) * 18
    for st in chips:
        if st == "heal":
            d.rectangle((cx, 1, cx + 16, 17), fill=B.SLATE, outline=T.HEAL)
            B.heart(d, cx + 2, 4, 12, T.HEAL)
        else:
            d.rectangle((cx, 1, cx + 16, 17), fill=B.SLATE, outline=T12.rim(T12.KIND[st]))
            ic = Image.open(ST / f"{st}.png").convert("RGBA").resize((16, 16), Image.BOX).resize((14, 14), Image.LANCZOS)
            img.paste(ic, (cx + 2, 3), ic)
        cx += 18
    return x + w + G


def screen(shot):
    img = B.mock10(shot)
    d = ImageDraw.Draw(img)
    # top bar: objective under the round, the strip in kind colours
    d.rectangle((66, 30, 190, 60), fill=B.BASALT)
    d.text((70, 36), "Rout them", font=B.SMALL, fill=B.PARCH)
    d.text((70, 50), "Protect Hollis", font=T12.TINY, fill=B.GOLD)
    d.rectangle((192, 0, 585, 63), fill=B.BASALT)
    x = 196
    for p in [("arrow", 1, 1, False, False, ()), ("pneumatic_hammer_strike", 3, 5, False, True, ()),
              ("pommel_strike", 1, 2, True, False, ()), ("pommel_strike", 1, 2, True, False, ("weaken",))]:
        x = tile(img, d, x, p[0], p[1], p[2], ghost=p[3], on=p[4], chips=p[5])
    # party panel
    d.rectangle((12, 76, 172, 636), fill=B.BASALT)
    heroes = [("Kaelen", 242, 242, 6, [("bleed", 1, True), ("fortify", 2, False), ("haste", 1, False), ("regen", 2, False),
                                        ("slow", 3, False)], True, 22),
              ("Vesper", 150, 190, 0, [("weaken", 1, False)], False, 0),
              ("Mara", 200, 210, 0, [], False, 0), ("Dunstan", 260, 300, 0, [("haste", 1, False)], False, 0)]
    y = 80
    for name, hp, mx, sp, sts, sel, loss in heroes:
        hero_plate(img, d, 16, y, name, hp, mx, sp, sts, sel, loss)
        y += 90
    d.line((16, y + 2, 168, y + 2), fill=T15.GUEST)
    d.text((16, y + 5), "Guests", font=B.SMALL, fill=T15.GUEST)
    guest_plate(img, d, 16, y + 24, T15.portrait("heroes/hollis.png", 40), "Hollis", 60, 90, True)
    # foe panel
    d.rectangle((1108, 76, 1268, 636), fill=B.BASALT)
    d.text((1114, 80), "Hostiles 4", font=B.SMALL, fill=B.PARCH)
    d.rounded_rectangle((1186, 79, 1264, 95), radius=3, outline=B.AMBER)
    d.text((1191, 80), "+2 round 3", font=B.SMALL, fill=B.AMBER)
    f = T15.foe_faces(shot)
    foes = [(f["footman"], "Footman", 64, 144, 5, [("stagger", 1, False), ("bleed", 3, True)], True, False),
            (f["footman"], "Footman", 144, 144, 5, [], False, False),
            (f["crawler"], "Ash Crawler", 60, 80, 4, [("burn", 2, False)], False, False),
            (f["bolt"], "Bolt Ranger", 70, 70, 5, [], False, False)]
    y = 100
    for face, name, hp, mx, pips, sts, sel, comp in foes:
        foe_plate(img, d, 1112, y, face, name, hp, mx, pips, sts, sel, comp)
        y += 76
    # board: a burn hazard on two tiles, a push preview on the target
    ov = Image.new("RGBA", img.size, (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for bx, by in ((232, 440), (328, 440)):
        od.rectangle((bx, by, bx + 92, by + 68), fill=(200, 90, 30, 70), outline=(220, 120, 50, 170))
    img.paste(ov, (0, 0), ov)
    burn = icon16("burn")
    for bx, by in ((232, 440), (328, 440)):
        img.paste(burn, (bx + 4, by + 4), burn)
        d.text((bx + 22, by + 4), "2", font=B.MONO, fill=B.PARCH)
    for k in range(0, 80, 8):
        d.line((900 + k, 470, 904 + k, 470), fill=B.GOLD, width=2)
    d.polygon([(986, 470), (976, 463), (976, 477)], fill=B.GOLD)
    for k in range(0, 92, 6):
        d.point((904 + k, 442), fill=B.GOLD); d.point((904 + k, 508), fill=B.GOLD)
    for k in range(0, 66, 6):
        d.point((904, 442 + k), fill=B.GOLD); d.point((996, 442 + k), fill=B.GOLD)
    # docked foe card (take 14, Kaelen selected)
    card = T14.card_hero(shot)
    img.paste(card.crop((6, 6, card.width - 6, card.height - 6)), (788, 76))
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    s = screen(shot)
    s.save(out / "battle_take16_720p.png")
    s.crop((0, 0, 640, 360)).resize((1280, 720), Image.NEAREST).save(out / "battle_take16_zoom_topleft.png")


if __name__ == "__main__":
    main()
