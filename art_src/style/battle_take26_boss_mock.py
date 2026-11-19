"""AI-35 battle mock take 26 (BOSSUI1, Tushar 2026-11-19 18:09 on take 24), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take26_boss_mock.py <playtest_dir> <out_dir>
  - BU1 the boss bar shows the current phase's HP span as a full bar and refills when the next phase starts; the
    number beside it is the span left ("160/450"), the unit's whole HP shows on hover;
  - BU2 phase markers, one per phase (done filled, current lit, coming hollow): option A left of the name
    (recommended), option B a thin segment strip under the bar;
  - BU3 the title is the phase's name or form; phase 3 here brings a different boss unit (illustrative: "the Ash
    Effigy" on the calcified_amalgam portrait), so the bar follows it (name, portrait, HP, poise);
  - BU4 passive tags on the title row; hover shows the passive's description; no phase-change or enrage tags
    (user 18:12): enrage is the baseline rule for every enemy (ENRAGE2), shown by the round counter a round ahead
    and on an enraged unit (amber tint, raised numbers);
  - BU5 the intent and impact row as in take 24; BU6 icon-only status tabs, rounds only in the expanded list;
  - BU7 boss poise as 10 pips of the ruleset's pip size (the tag shows the poise per pip);
  - the phase change: drain, gold flash with the marker filling, name (and portrait) crossfade, refill sweep.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take15_mock as T15  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take20_mock as T20  # noqa: E402
import battle_take24_mock as T24  # noqa: E402

T24.MODE = "none"                                  # BU6: icon-only tabs everywhere
TAG_W = {}
GRAKKA, EFFIGY = "bosses/grakka_ashmaw.png", "bosses/calcified_amalgam.png"
PASSIVES = {
    "Steady Nerves": "Hard to rattle: 25% resistance to Stagger and Stun.",
    "Warchief's Call": "When a Horde ally falls, Grakka gains 2 AP next round.",
    "Crown of Nails": "Reckless: deals 20% more damage and takes 10% more. Targets the weakest hero.",
    "Ash Body": "Fire heals it instead of hurting. Immune to Burn.",
    "Hollow": "Has no blood: immune to Bleed and Poison.",
}
# (name, title, portrait, span top, span bottom, unit max HP, HP now, planned loss, poise on, per-pip, passives, statuses)
PHASES = [
    ("Grakka Ashmaw", "Chieftain of the Horde", GRAKKA, 900, 450, 900, 610, 96, 6, 26,
     ["Steady Nerves", "Warchief's Call"], [("haste", 1, False), ("bleed", 3, True), ("burn", 2, False)]),
    ("Grakka, Crown of Nails", "Chieftain of the Horde", GRAKKA, 450, 0, 900, 380, 64, 10, 26,
     ["Steady Nerves", "Crown of Nails"], [("fortify", 3, False), ("regen", 3, False), ("bleed", 2, False)]),
    ("The Ash Effigy", "What the Horde Burned", EFFIGY, 600, 0, 600, 600, 0, 10, 30,
     ["Ash Body", "Hollow"], []),
]
NEXT_CUE = ["Done: filled. Now: lit. Coming: hollow.", "Hover: \"Phase 2 of 3: Crown of Nails\""]
ENR = (232, 140, 40)


def round_counter(shot, rnd, note, col):
    """ENRAGE2 EN5: the round counter warns a round ahead, then names the enrage step (top-left block, 1:1)."""
    img = T24.screen.__globals__["B"].mock10(shot).crop((0, 0, 200, 64))
    d = ImageDraw.Draw(img)
    d.rectangle((66, 4, 196, 62), fill=(20, 23, 31))
    d.text((70, 8), f"Round {rnd} of 8", font=B.TITLE_S, fill=B.GOLD)
    d.text((70, 30), "Rout them", font=B.SMALL, fill=B.PARCH)
    if note:
        d.rectangle((66, 46, 196, 60), fill=(44, 30, 14))
        d.rectangle((66, 46, 67, 60), fill=col)
        d.text((72, 47), note, font=T12.TINY, fill=col)
    return img


def enraged_plate(shot, stacks):
    """An enraged foe: amber tint on the plate and portrait rim, its intent number raised with the up arrow."""
    img = Image.new("RGB", (190, 92), B.BASALT)
    d = ImageDraw.Draw(img)
    T20.foe_plate(img, d, 6, 6, T15.portrait("common/brawler_cohort_breaker.png", 64), "Brawler", 88, 120, 5, [],
                  ("kaelen", 1), w=172)
    tint = Image.new("RGB", img.size, ENR)
    img = Image.blend(img, tint, 0.10)
    d = ImageDraw.Draw(img)
    d.rectangle((4, 4, 181, 86), outline=ENR)
    d.rounded_rectangle((141, 9, 180, 22), radius=3, fill=(44, 30, 14), outline=ENR)
    d.polygon([(144, 19), (147, 12), (150, 19)], fill=ENR)
    d.text((152, 10), f"+{10 * stacks}%", font=T12.TINY, fill=ENR)
    return img


def tag(d, x, y, text, col, lit=False):
    w = int(d.textlength(text, font=B.SMALL)) + 14
    d.rounded_rectangle((x, y, x + w, y + 16), radius=3, fill=(40, 34, 22) if lit else (16, 18, 25),
                        outline=B.GOLD if lit else col)
    d.text((x + 7, y + 1), text, font=B.SMALL, fill=B.GOLD if lit else (B.PARCH if col == B.IRON_L else col))
    return w


def markers(d, x, y, n, cur, s=10):
    """BU2 option A: one diamond per phase; done filled gold, current lit, coming hollow iron."""
    for i in range(n):
        cx, cy = x + i * (s + 4) + s // 2, y + s // 2
        pts = [(cx, cy - s // 2), (cx + s // 2, cy), (cx, cy + s // 2), (cx - s // 2, cy)]
        if i < cur:
            d.polygon(pts, fill=B.GOLD, outline=(90, 64, 20))
        elif i == cur:
            d.polygon(pts, fill=(120, 84, 24), outline=B.GOLD)
            d.point((cx, cy), fill=(255, 236, 170))
        else:
            d.polygon(pts, fill=(16, 18, 25), outline=B.IRON)
    return x + n * (s + 4)


def strip(d, x, y, w, n, cur):
    """BU2 option B: a 3 px segment strip under the bar, one segment per phase."""
    seg = (w - (n - 1) * 3) / n
    for i in range(n):
        sx = round(x + i * (seg + 3))
        col = B.GOLD if i < cur else ((180, 130, 40) if i == cur else (40, 44, 56))
        d.rectangle((sx, y, round(sx + seg), y + 2), fill=col)


def boss_bar(img, x0, x1, y, ph=0, opt="A", hover=None, frac=None, flash=False, title_img=None, por_img=None):
    name, title, por, top, bot, mx, hp, loss, pon, per, pas, sts = PHASES[ph]
    h = 84
    T24.shadow(img, (x0, y, x1, y + h))
    T24.nine(img, (x0, y, x1, y + h), "hud")
    d = ImageDraw.Draw(img)
    img.paste(por_img or T15.portrait(por, 56), (x0 + 10, y + 12))
    d.rectangle((x0 + 9, y + 11, x0 + 66, y + 68), outline=B.GOLD)
    T20.crown(d, x0 + 8, y + 7, 12)
    tx = x0 + 76
    nx = markers(d, tx, y + 12, len(PHASES), ph) + 6 if opt == "A" else tx
    if title_img is not None:
        img.paste(title_img, (nx, y + 6))
    else:
        d.text((nx, y + 8), name, font=B.TITLE_S, fill=B.GOLD)
        d.text((nx + d.textlength(name, font=B.TITLE_S) + 10, y + 11), title, font=B.SMALL, fill=B.IRON_L)
    # BU4: passive tags, right-aligned on the title row (no enrage or phase-change tag, ENRAGE2 EN5)
    cx = x1 - 12
    for t in pas[::-1]:
        w = int(d.textlength(t, font=B.SMALL)) + 14
        cx -= w
        TAG_W[t] = (cx, y + 9, w)
        tag(d, cx, y + 9, t, B.IRON_L, lit=t == hover)
        cx -= 6
    # BU1: the phase span as a full bar
    bx, bw, by = tx, x1 - 14 - 84 - tx, y + 32
    span = top - bot
    left = hp - bot
    f = left / span if frac is None else frac
    B.bar(d, bx, by, bw, 10, f, (150, 50, 45))
    if flash:
        d.rectangle((bx, by, bx + bw, by + 10), fill=(255, 226, 150))
    elif loss and frac is None:
        e = bx + round(bw * f)
        d.rectangle((e - round(bw * loss / span), by, e, by + 10), fill=B.IMPACT)
    if opt == "B":
        strip(d, bx, by + 14, bw, len(PHASES), ph)
    shown = round(span * f) if frac is not None else left
    d.text((bx + bw + 8, by - 3), f"{shown}/{span}", font=B.MONO, fill=B.PARCH)
    if loss and frac is None:
        d.text((bx + bw + 8, by + 11), f"-{loss}", font=B.MONO, fill=B.IMPACT)
    # row 3: 10 poise pips, the per-pip tag, icon-only statuses, the intent and its impact (take 24)
    ry = y + 56
    B.poise_glyph(d, tx, ry + 2, 12)
    B.poise(d, tx + 16, ry + 6, 10, pon, w=9, h=5, gap=2)
    B.pip_tag(d, tx + 132, ry + 2, per)
    sx = tx + 184
    for st, n, inc in T12.ordered(sts):
        T24.tab(img, d, sx, ry, st, n, inc)
        sx += 28
    ix = max(sx + 18, tx + 300)
    d.line((ix - 10, ry, ix - 10, ry + 18), fill=B.IRON)
    if ph == 2:
        d.text((ix, ry + 1), "Rising: acts from round 7", font=B.SMALL, fill=B.IRON_L)
        return y + h
    B.intent_tile(d, ix, ry + 1, s=16)
    skill = "Cleaving Roar" if ph == 0 else "Nail Rain"
    d.text((ix + 22, ry + 1), skill, font=B.SMALL, fill=B.PARCH)
    gx = ix + 26 + int(d.textlength(skill, font=B.SMALL))
    B.glyph(d, "sword", gx, ry + 3, 12, (200, 160, 120))
    img.paste(T17.T.face("kaelen" if ph == 0 else "vesper", 16), (gx + 16, ry + 1))
    for b in range(6):
        d.rectangle((gx + 38 + b * 13, ry + 7, gx + 38 + b * 13 + 10, ry + 12), fill=B.IMPACT if b == 1 + ph else B.AP_OFF)
    d.text((gx + 38 + 6 * 13 + 4, ry + 1), "-42" if ph == 0 else "-55", font=B.MONO, fill=B.IMPACT)
    return y + h


def tooltip(img, x, y, head, lines, w=300):
    h = 30 + 17 * len(lines) + 6
    T24.shadow(img, (x, y, x + w, y + h))
    T24.nine(img, (x, y, x + w, y + h), "hud")
    d = ImageDraw.Draw(img)
    d.text((x + 12, y + 7), head, font=B.TITLE_S, fill=B.GOLD)
    for i, ln in enumerate(lines):
        d.text((x + 12, y + 30 + i * 17), ln, font=B.SMALL, fill=B.PARCH if i == 0 else B.IRON_L)
    return h


def wrap(d, text, w):
    out, cur = [], ""
    for word in text.split():
        t = (cur + " " + word).strip()
        if d.textlength(t, font=B.SMALL) > w and cur:
            out.append(cur); cur = word
        else:
            cur = t
    return out + [cur]


def status_list(img, x, y, sts):
    """BU6: the expanded list (press the plate or hover a tab) is the one place with the rounds left as a number."""
    o = T12.ordered(sts)
    w, h = 300, 36 + len(o) * 24 + 8
    T24.shadow(img, (x, y, x + w, y + h))
    T24.nine(img, (x, y, x + w, y + h), "hud")
    d = ImageDraw.Draw(img)
    d.text((x + 12, y + 7), "Grakka: statuses", font=B.TITLE_S, fill=B.GOLD)
    d.text((x + 244, y + 11), "rounds", font=B.SMALL, fill=B.IRON_L)
    for i, (st, n, inc) in enumerate(o):
        ry = y + 36 + i * 24
        T24.tab(img, d, x + 10, ry - 1, st, n, inc)
        d.text((x + 42, ry + 1), st.capitalize(), font=B.SMALL, fill=B.PARCH)
        d.text((x + 104, ry + 1), T12.LABEL.get(st, ""), font=B.SMALL, fill=B.IRON_L)
        d.text((x + 262, ry + 1), "new" if inc else str(n), font=B.MONO, fill=B.IRON_L if inc else B.PARCH)
    return h


def screen(shot, ph=0, hover=None):
    T24.boss_bar = lambda img, x0, x1, y: boss_bar(img, x0, x1, y, ph=ph, hover=hover)
    return T24.screen(shot)


def crop2(img, box, k=2):
    c = img.crop(box)
    return c.resize((c.width * k, c.height * k), Image.NEAREST)


BAR = (204, 66, 1076, 166)


def bar_only(shot, **kw):
    base = T24.screen.__globals__["B"].mock10(shot)
    boss_bar(base, 212, 1068, 74, **kw)
    return base.crop(BAR)


def title_blend(shot, a, b, t):
    """The name (and portrait) crossfade between phases a and b."""
    ia, ib = bar_only(shot, ph=a), bar_only(shot, ph=b)
    box = (8, 0, 600, 34)
    ta = ia.crop((282 - 204 + 0, 74 - 66 + 4, 282 - 204 + 560, 74 - 66 + 26))
    return Image.blend(ia, ib, t), ta


def fade(shot, a, b, t):
    """The bar in phase b with only the portrait and the title blended from phase a (tags, statuses swap at the flash)."""
    ia, ib = bar_only(shot, ph=a, frac=0.0), bar_only(shot, ph=b, frac=0.0)
    out = ib.copy()
    for box in ((14, 8, 66, 66), (100, 4, 600, 28)):
        out.paste(Image.blend(ia.crop(box), ib.crop(box), t), box[:2])
    return out


def transition(shot):
    """Five beats inside the phase transition: drain, flash and marker fill, crossfade, refill sweep, settled."""
    return [("1  Phase 1's span drains to 0", bar_only(shot, ph=0, frac=0.0)),
            ("2  Gold flash; marker 1 fills, phase 2's tags and statuses swap in (0.1 s)",
             bar_only(shot, ph=1, frac=0.0, flash=True)),
            ("3  The name crossfades to the new form (0.3 s)", fade(shot, 0, 1, 0.5)),
            ("4  The bar refills, sweeping left to right (0.4 s)", bar_only(shot, ph=1, frac=0.55)),
            ("5  Settled: phase 2 of 3", bar_only(shot, ph=1)),
            ("A different unit: the portrait crossfades with the name, then the same sweep", fade(shot, 1, 2, 0.5))]


def sheet(rows, w=1760, title=""):
    hs = [r[1].height for r in rows]
    img = Image.new("RGB", (w, 56 + sum(hs) + 34 * len(rows)), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((16, 14), title, font=B.TITLE, fill=B.GOLD)
    y = 56
    for label, im in rows:
        d.text((16, y), label, font=B.BODY, fill=B.PARCH)
        img.paste(im, (16, y + 22))
        y += im.height + 34
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    # full screens: phase 1 (card open), phase 2 with a passive hovered, phase 3 (a different unit)
    s1 = screen(shot, 0)
    s1.save(out / "battle_take26_phase1_720p.png")
    s2 = screen(shot, 1, hover="Crown of Nails")
    d = ImageDraw.Draw(s2)
    x, y, w = TAG_W["Crown of Nails"]
    tooltip(s2, min(x + w - 300, 960 - 300), y + 22, "Crown of Nails", wrap(d, PASSIVES["Crown of Nails"], 276) + ["Passive, from phase 2"])
    s2.save(out / "battle_take26_phase2_hover_720p.png")
    s3 = screen(shot, 2)
    s3.save(out / "battle_take26_phase3_new_unit_720p.png")
    # 2x sheet: the three phases, the marker options, the hover tips and the expanded status list
    rows = [("Phase 1 of 3: the span 900 to 450 HP as a full bar (160 left) and two passive tags",
             crop2(s1, BAR)),
            ("Phase 2 of 3: refilled (450 to 0), the form's name, Crown of Nails hovered",
             crop2(s2, (204, 66, 1076, 232))),
            ("Phase 3 of 3: a different boss unit; the bar follows it (name, portrait, HP 600/600, poise 10 at 30)",
             crop2(s3, BAR)),
            ("Markers option B (not recommended): a segment strip under the bar instead of the diamonds",
             crop2(Image.new("RGB", (1, 1)), (0, 0, 1, 1)) if False else bar_only(shot, ph=0, opt="B").resize(
                 (872 * 2, 200), Image.NEAREST))]
    sheet(rows, title="Take 26: boss bar by phase (BOSSUI1), 2x").save(out / "battle_take26_phases_2x.png")
    # hover tips and the expanded list, 2x
    tips = Image.new("RGB", (880, 210), B.VOID)
    td = ImageDraw.Draw(tips)
    tooltip(tips, 10, 10, "Steady Nerves", wrap(td, PASSIVES["Steady Nerves"], 276) + ["Passive"])
    tooltip(tips, 10, 104, "Phase markers", NEXT_CUE, w=300)
    status_list(tips, 330, 10, PHASES[0][11] + [("weaken", 2, False)])
    tips.crop((0, 0, 650, 210)).resize((1300, 420), Image.NEAREST).save(out / "battle_take26_tips_2x.png")
    en = Image.new("RGB", (900, 100), B.VOID)
    en.paste(round_counter(shot, 4, "", ENR), (0, 0))
    en.paste(round_counter(shot, 5, "Enrage next round", ENR), (210, 0))
    en.paste(round_counter(shot, 6, "Foes enraged +1", ENR), (420, 0))
    en.paste(enraged_plate(shot, 1), (640, 4))
    ImageDraw.Draw(en).text((4, 76), "Round 4: nothing. Round 5: the warning. Round 6+: the step count. Right: an "
                            "enraged foe plate (standard step 10%).", font=B.SMALL, fill=B.IRON_L)
    en.resize((1800, 200), Image.NEAREST).save(out / "battle_take26_enrage_round_2x.png")
    rows = [(lab, im.resize((im.width * 2, im.height * 2), Image.NEAREST)) for lab, im in transition(shot)]
    sheet(rows, title="Take 26: the phase change, inside the existing phase transition (about 0.9 s), 2x").save(
        out / "battle_take26_transition_2x.png")


if __name__ == "__main__":
    main()
