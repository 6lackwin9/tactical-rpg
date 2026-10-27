# Portrait rig (ART2): head-and-shoulders pixel portraits from a few parts and a palette, at a chosen detail level.
# Parts are drawn as shapes on a supersampled canvas in unit coordinates (0..1, y down, the figure faces right),
# reduced to the level's pixel grid, snapped to the palette, outlined, then scaled up to the portrait size.
# Pillow only, so the rig can be ported to Rust (decision "Art tools in Python, portable to Rust").
import math
from PIL import Image, ImageDraw

SS = 8                 # supersampling per grid pixel
OUT = (24, 16, 18)     # outline


def ramp(base, n, lo=0.55, hi=1.25):
    """n shades of `base` from dark to light (index n // 2 is close to the base)."""
    out = []
    for i in range(n):
        k = lo + (hi - lo) * i / max(1, n - 1)
        out.append(tuple(max(0, min(255, int(c * k))) for c in base))
    return out


class Canvas:
    def __init__(s, grid):
        s.g = grid; s.n = grid * SS
        s.im = Image.new("RGBA", (s.n, s.n), (0, 0, 0, 0)); s.d = ImageDraw.Draw(s.im)

    def P(s, pts): return [(x * s.n, y * s.n) for x, y in pts]

    def poly(s, pts, c): s.d.polygon(s.P(pts), fill=c + (255,))

    def ell(s, cx, cy, rx, ry, c): s.d.ellipse([(cx - rx) * s.n, (cy - ry) * s.n, (cx + rx) * s.n, (cy + ry) * s.n], fill=c + (255,))

    def line(s, pts, c, w=1.0): s.d.line(s.P(pts), fill=c + (255,), width=max(1, int(w * SS)), joint="curve")

    def dot(s, x, y, c, r=0.6): s.d.ellipse([x * s.n - r * SS, y * s.n - r * SS, x * s.n + r * SS, y * s.n + r * SS], fill=c + (255,))


def reduce(cv, palette, outline=True):
    """Supersampled canvas -> grid pixels: box-average, snap every pixel to the nearest palette colour, outline."""
    small = cv.im.resize((cv.g, cv.g), Image.BOX)
    px = small.load(); pal = list(dict.fromkeys(palette))
    for y in range(cv.g):
        for x in range(cv.g):
            r, g, b, a = px[x, y]
            if a < 110:
                px[x, y] = (0, 0, 0, 0); continue
            r, g, b = (int(r * 255 / a), int(g * 255 / a), int(b * 255 / a)) if a < 255 else (r, g, b)
            best = min(pal, key=lambda p: (p[0] - r) ** 2 * 3 + (p[1] - g) ** 2 * 4 + (p[2] - b) ** 2 * 2)
            px[x, y] = best + (255,)
    if outline:
        ring = []
        for y in range(cv.g):
            for x in range(cv.g):
                if px[x, y][3] == 0 and any(0 <= x + dx < cv.g and 0 <= y + dy < cv.g and px[x + dx, y + dy][3] and px[x + dx, y + dy][:3] != OUT
                                            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    ring.append((x, y))
        for x, y in ring:
            px[x, y] = OUT + (255,)
    return small


def backdrop_step(g, x, y, n, style="glow"):
    """Where grid pixel (x, y) of a dithered backdrop sits on its ramp of `n` colours: 0 the first (darkest), n - 1
    the last; between two steps the frame dithers. "corner": 0 out to a quarter of the way from the face point to
    the far corner, then up to n - 1 there; "glow": n - 1 at the face point, falling to 0 at 0.62 of the grid."""
    cx, cy = g * 0.58, g * 0.40
    if style == "corner":
        far = max(math.hypot(px - cx, py - cy) for px, py in ((0, 0), (g, 0), (0, g), (g, g)))
        d = math.hypot(x - cx, y - cy) / far
        return min(1.0, max(0.0, (d - 0.25) / 0.75)) * (n - 1)
    d = math.hypot(x - cx, y - cy) / (g * 0.62)
    return (1 - min(1.0, d)) * (n - 1)


def backdrop_pixel(g, x, y, bg, style="glow"):
    """The dithered backdrop's colour at grid pixel (x, y), as frame draws it: between two ramp steps a checker."""
    k = backdrop_step(g, x, y, len(bg), style)
    lo = int(k); fr = k - lo
    hi = min(lo + 1, len(bg) - 1)
    return bg[hi] if (fr > 0.5) != ((x + y) % 2 == 0 and abs(fr - 0.5) < 0.25) else bg[lo]


def frame(fig, size, bg, dither=False, border=None, style="glow"):
    """The reduced figure on its background, scaled up (nearest) to `size`. dither: the background steps through its
    ramp `bg` (dark to light) by checker dither instead of hard rings, in one of two styles: "glow" (lightest behind
    the head, falling off to the dark edges) or "corner" (PIXEL3: darkest behind the face, brighter toward the
    frame's corners and edges); border: (dark, light) for a framed edge."""
    g = fig.width
    back = Image.new("RGBA", (g, g), bg[0] + (255,)); bd = ImageDraw.Draw(back)
    cx, cy = g * 0.58, g * 0.40
    if dither:
        bp = back.load()
        for y in range(g):
            for x in range(g):
                bp[x, y] = backdrop_pixel(g, x, y, bg, style) + (255,)
    else:
        for i, c in enumerate(bg[1:]):
            r = g * (0.62 - i * 0.12)
            bd.ellipse([cx - r, cy - r, cx + r, cy + r], fill=c + (255,))
    back.alpha_composite(fig)
    if border:
        dark, light = border
        bd = ImageDraw.Draw(back)
        bd.rectangle([0, 0, g - 1, g - 1], outline=dark + (255,)); bd.rectangle([1, 1, g - 2, g - 2], outline=light + (255,))
        bd.rectangle([2, 2, g - 3, g - 3], outline=dark + (255,))
        for x, y in ((3, 3), (g - 4, 3), (3, g - 4), (g - 4, g - 4)):
            bd.rectangle([x - 1, y - 1, x + 1, y + 1], outline=light + (255,))
    return back.resize((size, size), Image.NEAREST)


# ---- the parts ---------------------------------------------------------------------------------------------------

def draw(cv, L, c):
    """Mara-style warrior bust. `L` is the detail level: 1 clean, 2 detailed, 3 rich. `c` holds the ramps."""
    skin, hair, arm, steel, metal = c["skin"], c["hair"], c["armour"], c["steel"], c["brass"]
    n = len(skin) - 1
    # hair mass behind the head and the tail down the back
    cv.ell(0.47, 0.37, 0.205, 0.215, hair[1])
    cv.poly([(0.30, 0.38), (0.36, 0.30), (0.33, 0.62), (0.27, 0.74), (0.22, 0.70)], hair[1])
    # shoulders and the brigandine
    cv.poly([(0.02, 1.0), (0.08, 0.84), (0.24, 0.76), (0.44, 0.72), (0.66, 0.73), (0.84, 0.79), (0.96, 0.90), (1.0, 1.0)], arm[2])
    cv.poly([(0.02, 1.0), (0.08, 0.84), (0.24, 0.76), (0.30, 0.86), (0.22, 1.0)], arm[1])
    cv.poly([(0.66, 0.73), (0.84, 0.79), (0.96, 0.90), (1.0, 1.0), (0.80, 1.0), (0.70, 0.84)], arm[1])
    # pauldrons
    # pauldrons: a dark rim, the plate, a lit crown (the near one in light, the far one in shade)
    cv.ell(0.15, 0.85, 0.14, 0.09, steel[0]); cv.ell(0.15, 0.84, 0.125, 0.075, steel[1]); cv.ell(0.13, 0.825, 0.09, 0.05, steel[2])
    cv.ell(0.89, 0.87, 0.12, 0.085, steel[0]); cv.ell(0.89, 0.86, 0.105, 0.07, steel[1])
    if L >= 2:
        cv.ell(0.11, 0.815, 0.05, 0.025, steel[3]); cv.dot(0.22, 0.86, metal[-1], 0.5); cv.dot(0.83, 0.88, metal[1], 0.5)
    if L >= 3:
        cv.line([(0.03, 0.88), (0.27, 0.90)], steel[1], 0.7)
    # neck and the gorget
    cv.poly([(0.46, 0.56), (0.60, 0.56), (0.61, 0.73), (0.45, 0.73)], skin[n - 1])
    cv.poly([(0.46, 0.56), (0.52, 0.56), (0.50, 0.73), (0.45, 0.73)], skin[max(0, n - 2)])
    cv.poly([(0.38, 0.70), (0.66, 0.70), (0.70, 0.78), (0.34, 0.78)], steel[2])
    cv.line([(0.36, 0.74), (0.68, 0.74)], steel[0 if L == 1 else 1], 0.8)
    if L >= 2:
        cv.line([(0.40, 0.71), (0.64, 0.71)], steel[-1], 0.6)
    # brigandine rivets and the ember trim
    if L >= 2:
        for i in range(7):
            x = 0.30 + i * 0.065; cv.dot(x, 0.86 + 0.01 * (i % 2), metal[-1], 0.45 if L == 2 else 0.5)
            cv.dot(x + 0.03, 0.95, metal[1], 0.45)
        cv.line([(0.28, 0.80), (0.70, 0.80)], metal[1], 0.7)
    # head: skull, jaw, ear
    cv.ell(0.53, 0.40, 0.165, 0.195, skin[n - 1])
    cv.poly([(0.38, 0.42), (0.69, 0.40), (0.68, 0.52), (0.62, 0.61), (0.55, 0.645), (0.47, 0.62), (0.40, 0.53)], skin[n - 1])
    # shade: the far cheek and under the jaw (light from the upper left)
    cv.poly([(0.64, 0.40), (0.70, 0.40), (0.68, 0.52), (0.62, 0.61), (0.58, 0.62), (0.63, 0.52)], skin[n - 2])
    cv.poly([(0.44, 0.60), (0.55, 0.645), (0.62, 0.61), (0.60, 0.635), (0.54, 0.665), (0.46, 0.64)], skin[max(0, n - 3)])
    if L >= 3:
        cv.poly([(0.41, 0.36), (0.47, 0.30), (0.50, 0.40), (0.45, 0.50)], skin[n])   # lit forehead and cheek
    cv.ell(0.405, 0.46, 0.028, 0.045, skin[n - 2]); cv.ell(0.41, 0.46, 0.012, 0.025, skin[max(0, n - 3)])
    if c.get("stamps"):   # the features are placed pixel by pixel after the reduction (face_stamps)
        cv.poly([(0.655, 0.43), (0.705, 0.505), (0.68, 0.52), (0.645, 0.515)], skin[n - 1])
        draw_hair(cv, L, c, hair)
        return
    # eyes and brows: the near eye larger, the far eye toward the nose; a stern set
    for (ex, w) in ((0.52, 0.038), (0.625, 0.026)):
        cv.ell(ex, 0.445, w, 0.014, (236, 230, 220))
        cv.ell(ex + w * 0.25, 0.445, w * 0.45, 0.013, c["eye"])
        if L >= 2:
            cv.dot(ex + w * 0.15, 0.440, (255, 255, 255), 0.35)
        cv.line([(ex - w * 1.1, 0.418 - (0.006 if ex < 0.6 else 0)), (ex + w * 1.1, 0.405)], hair[0], 1.0 if L == 1 else 1.2)
        if L >= 2:
            cv.line([(ex - w * 1.0, 0.456), (ex + w, 0.458)], skin[max(0, n - 3)], 0.6)
    # nose and mouth
    cv.poly([(0.655, 0.43), (0.705, 0.505), (0.68, 0.52), (0.645, 0.515)], skin[n - 1])
    cv.line([(0.66, 0.44), (0.70, 0.505), (0.665, 0.52)], skin[max(0, n - 3)], 0.7)
    cv.line([(0.555, 0.567), (0.64, 0.56)], c["lip"], 1.2)
    if L >= 2:
        cv.line([(0.57, 0.585), (0.62, 0.583)], skin[n - 2], 0.6)
    # the scar across the near cheek (too fine for the clean level)
    if L >= 2:
        cv.line([(0.47, 0.49), (0.53, 0.535)], c["scar"], 0.7)
    draw_hair(cv, L, c, hair)


def draw_hair(cv, L, c, hair):
    skin = c["skin"]; n = len(skin) - 1
    # hair: swept back from a side part, the hairline dark, a loose lock over the near brow and the temple
    cv.poly([(0.36, 0.42), (0.36, 0.29), (0.44, 0.205), (0.57, 0.19), (0.67, 0.23), (0.705, 0.31), (0.66, 0.30), (0.60, 0.275),
             (0.54, 0.29), (0.49, 0.33), (0.45, 0.39), (0.42, 0.47)], hair[2])
    cv.line([(0.49, 0.33), (0.54, 0.29), (0.60, 0.275), (0.66, 0.30)], hair[0], 0.8)
    cv.poly([(0.47, 0.31), (0.53, 0.30), (0.50, 0.39), (0.47, 0.41)], hair[1])             # the lock
    cv.poly([(0.40, 0.40), (0.44, 0.38), (0.43, 0.52), (0.40, 0.50)], hair[1])             # temple
    if L >= 2:
        for k in range(4 if L == 2 else 7):
            t = k / (3 if L == 2 else 6)
            x0 = 0.40 + t * 0.26; cv.line([(x0, 0.25 + 0.02 * math.sin(t * 6)), (x0 - 0.06, 0.36)], hair[3 if L == 3 else 1], 0.6)
        cv.line([(0.47, 0.22), (0.62, 0.21)], hair[-1], 0.8)
    # rim light from the burning keep behind her (right side)
    if L >= 2:
        cv.line([(0.70, 0.31), (0.705, 0.40), (0.69, 0.50)], c["rim"], 0.8)
    if L >= 3:
        cv.line([(0.62, 0.62), (0.56, 0.655)], c["rim"], 0.6)
        for x, y in ((0.50, 0.50), (0.535, 0.505), (0.515, 0.515)):
            cv.dot(x, y, skin[n - 2], 0.3)   # freckles


def palette(L):
    shades = {1: 3, 2: 4, 3: 5}[L]
    c = {
        "skin": ramp((214, 160, 120), shades + 1, 0.6, 1.15),
        "hair": ramp((150, 52, 40), shades, 0.45, 1.25),
        "armour": ramp((132, 36, 34), shades, 0.5, 1.2),
        "steel": ramp((150, 162, 178), shades, 0.45, 1.3),
        "brass": ramp((200, 156, 70), 3, 0.6, 1.25),
        "eye": (64, 112, 84), "lip": (140, 70, 64), "lip_lo": (178, 106, 92), "scar": (176, 110, 92), "rim": (255, 170, 80),
    }
    flat = [OUT] + [v for k in ("skin", "hair", "armour", "steel", "brass") for v in c[k]] + [c["eye"], c["lip"], c["scar"], c["rim"], (236, 230, 220), (255, 255, 255)]
    return c, flat


def stamp(img, x, y, rows, key):
    """Pixel pattern `rows` (strings; '.' leaves the pixel) with its top-left at grid (x, y)."""
    px = img.load()
    for j, row in enumerate(rows):
        for i, ch in enumerate(row):
            if ch != "." and 0 <= x + i < img.width and 0 <= y + j < img.height:
                px[x + i, y + j] = key[ch] + (255,)


# The face at the 96 px grid, pixel by pixel (a 3/4 view facing right, light from the upper left).
FACE96 = {
    # stern brows sloping down to the nose; the socket under each brow in shade
    "brow_near": (43, 37, ["HHHH.....", ".HHHHHHH.", "....HHHHH", ".sssss..."]),
    "brow_far": (58, 37, ["HHHH.", ".HHHH", "..ss."]),
    # eyes: a thick dark upper lid, the iris with its pupil and a glint, the lower lid in shade
    "eye_near": (45, 41, ["..kkkkk.", ".kwwgipk", "kwwwiipk", ".sssss.."]),
    "eye_far": (59, 41, ["kkkk.", "wgpk.", "wiik.", ".ss.."]),
    # the nose: a shadow wedge down the far side of the bridge, the lit tip, a dark nostril, the cast shadow below
    "nose": (60, 42, ["S....", "S....", ".S...", ".S...", ".SS..", "..S.l", "..Sll", ".SOS.", "..SSs"]),
    # the mouth: a dark parting line, the upper lip in shade, the lower lip lit, its shadow below
    "mouth": (52, 54, ["...ssss.", "SOOOOOO.", ".RRRRR..", "..ssss.."]),
    # the cheekbone ridge on the near side and the hollow under it
    "cheek": (44, 48, [".ll.....", "..lll...", "....ss..", ".....ss."]),
}


def shade_face(img, c, top=27, chin=58):
    """Clean shading planes on the face from its own outline: base tone, a lit near cheek and brow, a shadow band down
    the far side, the jawline where the chin meets the neck, the neck in shade with a deep band under the chin."""
    skin = c["skin"]; n = len(skin) - 1
    base, lit, shade, deep = skin[n - 1], skin[n], skin[n - 2], skin[n - 3]
    px = img.load(); ramp_set = set(skin)
    def is_skin(x, y): return 0 <= x < img.width and 0 <= y < img.height and px[x, y][3] and px[x, y][:3] in ramp_set
    for y in range(top, chin + 9):
        xs = [x for x in range(img.width) if is_skin(x, y)]
        if not xs:
            continue
        xl, xr = min(xs), max(xs)
        for x in xs:
            jy = jaw(x, chin)
            if y > jy:                                      # neck, a deep band under the jaw
                tone = deep if y <= jy + 2 else shade
            elif y == jy:                                   # the jawline
                tone = deep
            elif x >= xr - 5 or y >= jy - 1:                # far side band (a strong split light), the jaw's underside
                tone = shade
            elif (x - 47) ** 2 / 9 + (y - 46) ** 2 / 4 <= 1 or ((x - 51) / 6) ** 2 + ((y - 30) / 2.2) ** 2 <= 1:
                tone = lit                                  # near cheekbone, brow
            else:
                tone = base
            px[x, y] = tone + (255,)


def jaw(x, chin):
    """The jaw's lowest face row at column x: slanting from below the ear to the chin, then up the far side."""
    if x < 54:
        return int(chin - 7 + (x - 42) * 0.55)
    if x <= 59:
        return chin
    return chin - (x - 59) * 2


def face_stamps(img, c, face=FACE96):
    skin = c["skin"]; n = len(skin) - 1
    shade_face(img, c)
    key = {"k": (58, 30, 30), "H": c["hair"][0], "w": (222, 212, 200), "i": c["eye"], "p": (28, 26, 30), "g": (255, 255, 255),
           "s": skin[n - 2], "S": skin[n - 3], "l": skin[n], "r": c["lip"], "R": c["lip_lo"], "O": (74, 34, 32)}
    for _, (x, y, rows) in face.items():
        stamp(img, x, y, rows, key)


LEVELS = {1: ("Clean", 64), 2: ("Detailed", 96), 3: ("Rich", 120)}
BG = [(34, 22, 24), (58, 30, 26), (86, 42, 30), (120, 60, 34)]


def portrait(L, size=240, stamps=False):
    _, grid = LEVELS[L]
    c, flat = palette(L)
    c["stamps"] = stamps and grid == 96
    cv = Canvas(grid); draw(cv, L, c)
    fig = reduce(cv, flat)
    if c["stamps"]:
        face_stamps(fig, c)
        return frame(fig, size, [(18, 14, 18), (34, 22, 24), (58, 30, 26), (86, 42, 30)], dither=True, border=((24, 16, 18), c["brass"][1]))
    bg = BG[:2] if L == 1 else BG[:3] if L == 2 else BG
    return frame(fig, size, bg)


if __name__ == "__main__":
    import sys
    out = sys.argv[1]
    for L, (name, grid) in LEVELS.items():
        portrait(L).save(f"{out}/mara_L{L}_{name.lower()}_{grid}.png")
        print(L, name, grid)
