# Shared pixel-art rig: one skeleton, swappable outfit (head/torso/legs), weapon family and palette.
# CREATURE1: two creature skeletons beside it, a quadruped and a segmented crawler (the end of this file).
# WINDUP1: the wind-up, stagger and dazed helpers and the strike and loop marks (after `impact`).
# Frames are W x W, facing right. Units are dicts: proportions, part names and a palette.
import math
from PIL import Image

W = 48
GY = 43.0
OUT = (20, 17, 15)
WHITE = (250, 244, 230)
GOLDEN, GOLDEN_HI = (240, 200, 90), (255, 240, 180)  # effect colours (no palette slot): focus and buff motes


class F:
    def __init__(s):
        s.im = Image.new("RGBA", (W, W), (0, 0, 0, 0)); s.px = s.im.load()

    def dot(s, x, y, c):
        x, y = int(round(x)), int(round(y))
        if 0 <= x < W and 0 <= y < W: s.px[x, y] = tuple(c[:3]) + (255,)

    def disc(s, cx, cy, r, c):
        for y in range(int(cy - r - 1), int(cy + r + 2)):
            for x in range(int(cx - r - 1), int(cx + r + 2)):
                if (x - cx) ** 2 + (y - cy) ** 2 <= r * r: s.dot(x, y, c)

    def limb(s, a, b, r, c, lo=None):
        n = int(max(abs(b[0] - a[0]), abs(b[1] - a[1])) * 2) + 1
        for i in range(n + 1):
            t = i / n; s.disc(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, r, c)
        if lo:
            for i in range(n + 1):
                t = i / n; x = a[0] + (b[0] - a[0]) * t; y = a[1] + (b[1] - a[1]) * t
                s.dot(x, y + r - 0.2, lo); s.dot(x - 0.5, y + r - 0.2, lo)

    def rect(s, x0, y0, x1, y1, c):
        for y in range(int(min(y0, y1)), int(max(y0, y1)) + 1):
            for x in range(int(min(x0, x1)), int(max(x0, x1)) + 1): s.dot(x, y, c)

    def poly(s, pts, c):  # filled polygon, scanline
        ys = [p[1] for p in pts]
        for y in range(int(min(ys)), int(max(ys)) + 1):
            xs = []
            for i in range(len(pts)):
                (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % len(pts)]
                if (y0 <= y + 0.5 < y1) or (y1 <= y + 0.5 < y0):
                    xs.append(x0 + (y + 0.5 - y0) * (x1 - x0) / (y1 - y0))
            xs.sort()
            for i in range(0, len(xs) - 1, 2):
                for x in range(int(round(xs[i])), int(round(xs[i + 1])) + 1): s.dot(x, y, c)

    def outline(s):
        sp = s.im.copy().load()
        for y in range(W):
            for x in range(W):
                if sp[x, y][3] == 0:
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < W and 0 <= ny < W and sp[nx, ny][3] and sp[nx, ny][:3] != WHITE:
                            s.px[x, y] = OUT + (255,); break


def ik(root, la, lb, target, bend=1):
    dx, dy = target[0] - root[0], target[1] - root[1]
    d = max(0.01, min(math.hypot(dx, dy), la + lb - 0.01))
    base = math.atan2(dy, dx)
    a = base + bend * math.acos(max(-1, min(1, (la * la + d * d - lb * lb) / (2 * la * d))))
    return (root[0] + la * math.cos(a), root[1] + la * math.sin(a)), (root[0] + d * math.cos(base), root[1] + d * math.sin(base))


def arc_pts(cx, cy, r, a0, a1, n=16):
    return [(cx + r * math.sin(math.radians(a0 + (a1 - a0) * i / (n - 1))), cy - r * math.cos(math.radians(a0 + (a1 - a0) * i / (n - 1)))) for i in range(n)]


def gait(u, stride=6.0, lift=4.2):
    u %= 1.0
    if u < 0.5: return (stride - 2 * stride * (u / 0.5), 0.0)
    v = (u - 0.5) / 0.5
    return (-stride + 2 * stride * v, lift * math.sin(math.pi * v))


def udir(a_deg):  # 0 = up, 90 = forward (right), 180 = down
    r = math.radians(a_deg); return math.sin(r), -math.cos(r)


# ------------------------------------------------------------------ outfit parts
def draw_head(f, u, P, hx, hy, hurt, tilt=0.0):
    h = u['head']
    if h == 'goblin':   # SPRITE2: big skull, long ear swept back, hooked nose, tusked underbite; u['hat'] adds headgear
        hat = u.get('hat')
        if hat == 'hood':  # the hood goes on first; the ear pokes out through it
            f.disc(hx - 0.6, hy - 0.5, 3.9, P['main']); f.rect(hx - 4.2, hy - 0.5, hx - 2.6, hy + 3.5, P['main_lo'])
        else:
            f.disc(hx, hy, 3.4, P['skin'])
        f.disc(hx + 0.9, hy + 0.5, 2.7, P['skin'])
        for k in range(7):  # ear: from behind the skull, back and up to a point
            f.disc(hx - 2.0 - k * 0.95, hy - 0.4 - k * 0.62, 1.5 - k * 0.16, P['skin'] if k < 5 else P['skin_lo'])
        f.dot(hx - 2.6, hy - 0.5, P['skin_lo']); f.dot(hx - 3.5, hy - 1.0, P['skin_lo'])
        f.rect(hx - 2.0, hy + 1.8, hx + 2.6, hy + 3.2, P['skin_lo'])                       # heavy jaw
        f.rect(hx + 2.2, hy + 0.0, hx + 4.4, hy + 1.0, P['skin']); f.dot(hx + 4.6, hy + 1.4, P['skin_lo'])  # hooked nose
        f.rect(hx + 0.6, hy - 1.6, hx + 2.6, hy - 1.6, P['skin_lo'])                      # brow
        f.dot(hx + 1.5, hy - 0.6, OUT if hurt else P['eye']); f.dot(hx + 2.4, hy - 0.6, OUT)
        f.dot(hx + 2.4, hy + 1.8, WHITE); f.dot(hx + 1.0, hy + 2.0, WHITE)                  # tusks
        if hat == 'band':      # headband with trailing tails
            f.rect(hx - 2.8, hy - 2.6, hx + 2.8, hy - 1.9, P['accent'])
            for k in range(3): f.dot(hx - 3.6 - k, hy - 2.0 + k * 0.7 + math.sin(tilt + k) * 0.5, P['accent'] if k < 2 else P['accent_lo'])
        elif hat == 'bandana':  # cloth tied over the skull, knot at the back
            f.poly([(hx - 3.4, hy - 0.8), (hx - 2.8, hy - 3.2), (hx - 0.4, hy - 4.2), (hx + 2.0, hy - 3.6), (hx + 3.0, hy - 1.8), (hx - 1.0, hy - 1.6)], P['accent'])
            f.rect(hx - 0.6, hy - 2.1, hx + 2.8, hy - 1.8, P['accent_lo'])
            f.dot(hx - 3.6, hy - 1.0, P['accent_lo']); f.dot(hx - 4.2, hy - 0.2 + math.sin(tilt) * 0.5, P['accent'])
        elif hat == 'hood':
            f.dot(hx + 1.0, hy - 3.4, P['main_hi']); f.dot(hx - 0.2, hy - 3.8, P['main_hi'])
        elif hat == 'mohawk':  # a crest of spikes front to back
            for k in range(5):
                x0 = hx + 1.4 - k * 1.1
                f.limb((x0, hy - 2.8), (x0 - 0.9, hy - 5.6 + abs(k - 1.5) * 0.45), 0.55, P['hair'] if k % 2 == 0 else P['hair_lo'])
        elif hat == 'skull':   # an animal skull worn as a cap, horns back, a feather
            f.disc(hx - 0.2, hy - 2.4, 2.7, P['bone']); f.rect(hx + 1.0, hy - 2.2, hx + 3.4, hy - 1.4, P['bone'])
            f.dot(hx + 1.6, hy - 2.6, OUT); f.dot(hx - 0.6, hy - 3.6, P['bone_lo'])
            f.limb((hx - 2.0, hy - 3.6), (hx - 4.4, hy - 5.8), 0.6, P['bone']); f.dot(hx - 4.8, hy - 6.4, P['bone_lo'])
            f.limb((hx - 2.8, hy - 2.2), (hx - 5.0, hy - 1.0), 0.5, P['accent'])
        elif hat == 'horns':   # iron cap with two curved horns
            f.poly([(hx - 3.6, hy - 1.0), (hx - 2.9, hy - 3.4), (hx - 0.4, hy - 4.5), (hx + 2.1, hy - 3.8), (hx + 3.2, hy - 1.6), (hx + 3.2, hy - 1.0)], P['metal'])
            f.rect(hx - 3.6, hy - 1.6, hx + 3.2, hy - 1.0, P['metal_lo']); f.dot(hx + 0.4, hy - 3.6, P['metal_hi'])
            f.limb((hx - 2.2, hy - 2.6), (hx - 4.6, hy - 4.6), 0.75, P['bone']); f.limb((hx - 4.6, hy - 4.6), (hx - 4.4, hy - 7.0), 0.55, P['bone'])
            f.limb((hx + 1.6, hy - 3.0), (hx + 3.2, hy - 5.2), 0.75, P['bone']); f.limb((hx + 3.2, hy - 5.2), (hx + 2.6, hy - 7.4), 0.55, P['bone_lo'])
        elif hat == 'crown':   # an iron band with a crown of rusted nails
            f.rect(hx - 3.2, hy - 2.9, hx + 2.9, hy - 1.9, P['metal_lo'])
            for k in range(6):
                x0 = hx - 2.8 + k * 1.15; f.limb((x0, hy - 2.9), (x0 + 0.3 * (k - 2.5) / 2.5, hy - 5.2 - (0.6 if k in (2, 3) else 0)), 0.4, P['rust'] if k % 2 else P['metal'])
            f.dot(hx - 0.3, hy - 2.4, P['trim'])
        return
    # a face every human head shares
    f.disc(hx, hy, 3.4, P['skin'])
    f.rect(hx - 3, hy + 1.2, hx - 1.5, hy + 3, P['skin_lo'])
    f.dot(hx + 2, hy + 0.2, OUT if hurt else P['eye']); f.dot(hx + 3.2, hy + 1.6, P['skin_lo'])
    if h in ('bare', 'kettle', 'cap'):  # SPRITE2 named humans: hair, an iron kettle hat or a leather cap, beard, braid
        f.rect(hx - 3, hy - 3.6, hx + 2.5, hy - 2.2, P['hair']); f.rect(hx - 3.5, hy - 2.2, hx - 2.3, hy + 0.8, P['hair'])
        if u.get('braid'):
            for k in range(5): f.disc(hx - 3.8 - k * 0.3, hy + 1.2 + k * 1.3, 1.0 - k * 0.08, P['hair'] if k % 2 == 0 else P['hair_lo'])
        if h == 'kettle':
            f.rect(hx - 2.8, hy - 5.0, hx + 2.6, hy - 2.6, P['metal']); f.dot(hx + 1.2, hy - 4.4, P['metal_hi']); f.dot(hx - 2.2, hy - 3.2, P['metal_lo'])
            f.rect(hx - 4.8, hy - 2.6, hx + 4.8, hy - 1.9, P['metal_lo'])
        elif h == 'cap':
            f.rect(hx - 3.2, hy - 4.2, hx + 2.4, hy - 2.0, P['cap']); f.rect(hx + 1.6, hy - 2.4, hx + 4.4, hy - 1.8, P['cap_lo'])
            f.dot(hx - 0.6, hy - 3.8, P.get('cap_hi', P['cap']))
        if u.get('beard'):
            f.rect(hx + 0.4, hy + 1.8, hx + 3.2, hy + 3.8, P['beard']); f.rect(hx - 1.4, hy + 0.6, hx - 0.2, hy + 3.2, P['beard'])
            f.dot(hx + 1.8, hy + 4.4, P['beard']); f.dot(hx + 2.8, hy + 1.6, OUT)
    elif h == 'helm':      # warrior: full helm, visor slit, plume
        f.disc(hx, hy - 0.2, 3.7, P['metal']); f.rect(hx - 3, hy + 0.6, hx + 3, hy + 3, P['skin'])
        f.rect(hx + 0.5, hy - 0.5, hx + 3.6, hy + 0.5, OUT); f.dot(hx + 2, hy + 1.8, OUT if hurt else WHITE)
        f.dot(hx - 1, hy - 2, P['metal_hi']); f.dot(hx, hy - 2.6, P['metal_hi']); f.rect(hx - 3, hy - 0.5, hx - 2, hy + 2.5, P['metal_lo'])
        f.rect(hx - 1, hy - 4.8, hx, hy - 3.4, P['accent']); f.dot(hx - 2, hy - 4, P['accent']); f.dot(hx - 3, hy - 3.2, P['accent_lo'])
    elif h == 'hood':    # ranger/rogue: hood with a deep front edge, optional mask
        f.disc(hx - 0.6, hy - 0.4, 3.9, P['main']); f.disc(hx + 0.8, hy + 0.6, 2.7, P['skin'])
        f.rect(hx - 4, hy - 0.5, hx - 2.5, hy + 3.5, P['main_lo']); f.dot(hx - 4.5, hy + 3.5, P['main_lo'])
        f.dot(hx + 1.4, hy - 2.6, P['main_hi']); f.dot(hx + 0.4, hy - 3.2, P['main_hi'])
        f.dot(hx + 2, hy + 0.2, OUT if hurt else P['eye'])
        if u.get('mask'): f.rect(hx - 0.5, hy + 1.6, hx + 3.4, hy + 3.2, P['accent'])
    elif h == 'headband':  # brawler: short hair, headband with trailing tails
        f.rect(hx - 3, hy - 3.6, hx + 2.5, hy - 2.2, P['hair']); f.rect(hx - 3.5, hy - 2.2, hx - 2.3, hy + 0.5, P['hair'])
        f.rect(hx - 3, hy - 2.0, hx + 3, hy - 1.2, P['accent'])
        for k in range(4): f.dot(hx - 4 - k, hy - 1.6 + k * 0.6 + math.sin(tilt + k) * 0.5, P['accent'] if k < 3 else P['accent_lo'])
        f.rect(hx + 1.2, hy + 2.6, hx + 2.6, hy + 2.6, P['skin_lo'])  # set jaw
    elif h == 'cowl':    # priest: cloth cowl that opens at the face, gold circlet
        f.disc(hx - 0.8, hy - 0.2, 3.9, P['main']); f.disc(hx + 0.9, hy + 0.7, 2.6, P['skin'])
        f.rect(hx - 4.2, hy + 0.5, hx - 1.5, hy + 4.5, P['main_lo'])
        f.rect(hx - 1, hy - 2.8, hx + 2.4, hy - 2.2, P['trim']); f.dot(hx + 2, hy + 0.2, OUT if hurt else P['eye'])
    elif h == 'hat':     # mage: wide brim and a tall bent cone
        f.rect(hx - 3, hy - 3.2, hx + 2, hy - 1.8, P['hair']); f.rect(hx - 3.5, hy - 1.8, hx - 2.4, hy + 2.5, P['hair'])
        f.rect(hx - 5.2, hy - 3.4, hx + 5.2, hy - 2.6, P['main_lo'])
        f.poly([(hx - 3.2, hy - 3.2), (hx + 2.8, hy - 3.2), (hx - 1.2, hy - 10.5)], P['main'])
        f.limb((hx - 1.2, hy - 10.2), (hx - 4, hy - 11.5), 0.6, P['main']); f.rect(hx - 3, hy - 4.4, hx + 2.6, hy - 3.6, P['trim'])
        f.rect(hx + 0.5, hy + 2.8, hx + 2.8, hy + 4.6, P['hair']); f.dot(hx + 1.5, hy + 5.2, P['hair'])  # beard


def draw_torso(f, u, P, R, px, py, lx, T):
    t_ = u['torso']
    for i in range(12):
        t = i / 11.0; cx = px + lx * t; cy = py - T * t
        w0, w1 = -4.2 + t * 0.5, 3.8 - t * 0.3
        if u.get('girth'): w0, w1 = w0 - u['girth'], w1 + u['girth'] * (0.6 + 0.4 * t)  # a broader chest (SPRITE2 warlords)
        if t_ == 'plate':   c = P['metal'] if t > 0.55 else P['accent']
        elif t_ == 'leather': c = P['main'] if t > 0.25 else P['leg']
        elif t_ == 'bare':  c = P['skin'] if t > 0.22 else P['leg']; w0 -= 0.6; w1 += 0.6
        elif t_ == 'robe':  c = P['main'] if t > 0.15 else P['main_lo']
        elif t_ == 'vest':  c = P['accent'] if t < 0.3 else (P['skin_lo'] if t < 0.45 else P['skin'])
        f.rect(*R((cx + w0, cy - 0.5)), *R((cx + w1, cy + 0.5)), c)
    if t_ == 'plate':
        for i in range(7):
            cx = px + lx * (i / 11.0); cy = py - T * (i / 11.0)
            f.rect(*R((cx - 1.5, cy - 0.4)), *R((cx + 1.5, cy + 0.6)), P['accent_lo'] if i % 2 else P['accent'])
        f.rect(*R((px - 4, py - 1.2)), *R((px + 4, py - 0.2)), P['leather']); f.dot(*R((px, py - 0.7)), P['trim'])
    elif t_ == 'leather':  # cross belt and a short cloak edge
        for k in range(9):
            t = k / 8.0; f.disc(*R((px + lx * (0.4 + 0.6 * t) + 3 - 6 * t, py - T * (0.35 + 0.6 * t))), 0.5, P['leather'])
        f.rect(*R((px - 4, py - 1.2)), *R((px + 4, py - 0.2)), P['leather']); f.dot(*R((px + 1, py - 0.7)), P['trim'])
    elif t_ == 'bare':  # muscle line, sash belt
        f.dot(*R((px + lx * 0.6 + 1, py - T * 0.6)), P['skin_lo']); f.dot(*R((px + lx * 0.6 + 1.5, py - T * 0.45)), P['skin_lo'])
        f.dot(*R((px + lx * 0.8 - 1, py - T * 0.78)), P['skin_lo']); f.dot(*R((px + lx * 0.8 + 0.5, py - T * 0.8)), P['skin_lo'])
        f.rect(*R((px - 4.6, py - 2.2)), *R((px + 4.6, py - 0.8)), P['accent'])
        f.limb(R((px - 4, py - 1.5)), R((px - 6.5, py + 3)), 0.6, P['accent_lo'])
    elif t_ == 'robe':  # trim down the front, rope belt
        for i in range(12):
            t = i / 11.0; f.dot(*R((px + lx * t + 2.6 - t * 0.3, py - T * t)), P['trim'])
        f.rect(*R((px - 4, py - 3.2)), *R((px + 4, py - 2.4)), P['main_lo']); f.dot(*R((px + 2, py - 2.8)), P['trim'])
    elif t_ == 'vest':
        for k in range(9):
            t = k / 8.0; f.disc(*R((px + lx * (0.4 + 0.6 * t) + 3 - 6 * t, py - T * (0.35 + 0.6 * t))), 0.6, P['leather'])
        f.rect(*R((px - 3, py - 0.5)), *R((px + 3, py + 2.5)), P['accent_lo']); f.rect(*R((px - 3.5, py - 1.5)), *R((px + 3.5, py - 0.5)), P['leather'])


def draw_back_gear(f, u, P, R, sh, px, py, T, lx):
    g = u.get('back')
    if g == 'quiver':
        qx, qy = sh[0] - 4.5, sh[1] + 1
        f.rect(*R((qx - 1, qy)), *R((qx + 1, qy + 6)), P['leather_lo'])
        for k in range(3): f.dot(*R((qx - 1 + k, qy - 1 - (k % 2))), WHITE if k != 1 else P['accent'])
    elif g == 'cape':
        f.poly([R((sh[0] - 2.5, sh[1])), R((sh[0] - 1, sh[1])), R((px - 3, py + 6)), R((px - 7.5, py + 5))], P['accent_lo'])


def draw_robe_skirt(f, u, P, R, px, py, kb, kf, fb, ff):
    if u['legs'] != 'robe': return
    lowb = (min(fb[0], kb[0]) - 1.2, fb[1] - 3.0); lowf = (max(ff[0], kf[0]) + 1.2, ff[1] - 3.0)
    pts = [R((px - 4.2, py - 1)), R((px + 4.0, py - 1)), R(lowf), R(((lowb[0] + lowf[0]) / 2, max(lowb[1], lowf[1]) + 0.5)), R(lowb)]
    f.poly(pts, P['main'])
    f.limb(R(lowb), R(lowf), 0.5, P['main_lo']); f.dot(*R((px + 2.5, py + 2)), P['trim'])


# ------------------------------------------------------------------ weapons
def draw_weapon(f, u, P, kind, hand, a_deg, two=False, glow=0.0):
    hx, hy = hand; ux, uy = udir(a_deg)
    if kind in ('sword', 'sword2'):
        grip = 5.0 if two else 2.5; L = 15 if kind == 'sword2' else 12
        f.limb((hx - ux * grip, hy - uy * grip), (hx + ux * 0.5, hy + uy * 0.5), 0.8, P['leather_lo'])
        f.disc(hx - ux * (grip + 0.6), hy - uy * (grip + 0.6), 1.0, P['trim'])
        f.limb((hx + ux * 0.9 - uy * 2.4, hy + uy * 0.9 + ux * 2.4), (hx + ux * 0.9 + uy * 2.4, hy + uy * 0.9 - ux * 2.4), 0.7, P['trim'])
        tip = (hx + ux * L, hy + uy * L)
        f.limb((hx + ux * 2, hy + uy * 2), tip, 1.0, P['metal_hi']); f.limb((hx + ux * 2, hy + uy * 2), tip, 0.4, WHITE)
    elif kind == 'mace':
        # gripped a third of the way up: the shaft shows below the fist, with a small pommel
        f.limb((hx - ux * 4, hy - uy * 4), (hx + ux * 7.5, hy + uy * 7.5), 0.8, P['wood'])
        f.disc(hx - ux * 4.4, hy - uy * 4.4, 0.8, P['metal_lo'])
        cx, cy = hx + ux * 8.6, hy + uy * 8.6
        f.disc(cx, cy, 2.2, P['metal']); f.dot(cx - 1, cy - 1, P['metal_hi'])
        for k in range(4):
            a = math.radians(a_deg + k * 90 + 45); f.dot(cx + math.sin(a) * 2.8, cy - math.cos(a) * 2.8, P['metal_lo'])
    elif kind == 'dagger':
        f.limb((hx - ux * 1.5, hy - uy * 1.5), (hx + ux * 0.4, hy + uy * 0.4), 0.7, P['leather_lo'])
        f.limb((hx + ux * 0.9 - uy * 1.5, hy + uy * 0.9 + ux * 1.5), (hx + ux * 0.9 + uy * 1.5, hy + uy * 0.9 - ux * 1.5), 0.5, P['trim'])
        f.limb((hx + ux * 1.6, hy + uy * 1.6), (hx + ux * 6.5, hy + uy * 6.5), 0.6, P['metal_hi']); f.dot(hx + ux * 6.8, hy + uy * 6.8, WHITE)
    elif kind == 'staff':  # held near the middle; gem on the upper end
        a, b = (hx - ux * 11, hy - uy * 11), (hx + ux * 10, hy + uy * 10)
        f.limb(a, b, 0.8, P['wood']); f.limb((b[0] - ux * 1.5, b[1] - uy * 1.5), b, 1.1, P['wood_lo'])
        gx, gy = hx + ux * 11.8, hy + uy * 11.8
        if glow > 0: f.disc(gx, gy, 2.2 + glow * 1.8, P['magic_lo'])
        f.disc(gx, gy, 1.6, P['magic']); f.dot(gx - 0.5, gy - 0.6, WHITE)
    elif kind == 'shield':  # kite shield on its own outlined layer so it reads apart from the armour; a_deg rotates it
        g = F(); cx, cy = hx + 3.0, hy + 0.5
        r = math.radians(a_deg); ca, sa = math.cos(r), math.sin(r)
        rot = lambda p: (cx + p[0] * ca - p[1] * sa, cy + p[0] * sa + p[1] * ca)
        face = [(-3.6, -7.5), (3.6, -7.5), (4.4, -2.0), (0.5, 9.0), (-0.5, 9.0), (-4.4, -2.0)]
        g.poly([rot(p) for p in face], P.get('shield', (70, 96, 150)))
        g.poly([rot(p) for p in [(1.2, -7.0), (3.4, -7.0), (4.0, -2.0), (0.6, 8.0)]], P.get('shield_lo', (48, 66, 110)))   # shaded far half
        for i in range(len(face)): g.limb(rot(face[i]), rot(face[(i + 1) % len(face)]), 0.45, P['metal_hi'] if i in (0, 5) else P['metal'])
        g.limb(rot((0, -5.5)), rot((0, 6.0)), 0.6, P.get('emblem', WHITE)); g.limb(rot((-2.6, -2.2)), rot((2.6, -2.2)), 0.6, P.get('emblem', WHITE))
        g.outline(); f.im.alpha_composite(g.im); f.px = f.im.load()
    elif kind == 'buckler':  # a small round fist-shield strapped over the far hand: leather face, iron rim and boss
        g = F(); cx, cy = hx + 1.0, hy
        g.disc(cx, cy, 3.6, P['metal']); g.disc(cx, cy, 2.8, P['leather'])
        g.poly([(cx + 0.4, cy - 2.6), (cx + 2.6, cy - 1.0), (cx + 2.6, cy + 1.0), (cx + 0.4, cy + 2.6)], P['leather_lo'])  # shaded far half
        g.dot(cx - 2.6, cy - 2.0, P['metal_hi']); g.dot(cx - 3.2, cy - 0.8, P['metal_hi'])
        g.disc(cx, cy, 1.1, P['metal']); g.dot(cx - 0.4, cy - 0.5, P['metal_hi'])
        g.outline(); f.im.alpha_composite(g.im); f.px = f.im.load()
    elif kind in DRAWERS:
        DRAWERS[kind](f, P, (hx, hy), (ux, uy))


# ------------------------------------------------------------------ weapon drawers (WAR2H1)
# Each drawer gets the (upper) hand and the weapon's direction u; on the 2H grip the lower hand holds the grip `GRIP2`
# px behind it (pose(grip2=...)). n = (-uy, ux) is the side the head faces: forward when held upright, and the
# direction of travel in a forward (clockwise) swing, so an axe's edge leads the chop. Only rig palette colours
# (no new slots). DRAWERS names them for unit['weapon'].
GRIP2 = 3.2


def _at(h, u, s, n=(0.0, 0.0), k=0.0):
    return (h[0] + u[0] * s + n[0] * k, h[1] + u[1] * s + n[1] * k)


def _haft(f, P, h, u, back, fwd):
    f.limb(_at(h, u, -back), _at(h, u, fwd), 0.75, P['leather'])
    f.limb(_at(h, u, -back), _at(h, u, -1.0), 0.75, P['leather_lo'])  # the wrapped grip
    f.disc(*_at(h, u, -back - 0.4), 0.8, P['metal_lo'])                # butt cap


def _greatsword(f, P, h, u):
    n = (-u[1], u[0])
    f.limb(_at(h, u, -5.5), _at(h, u, 0.5), 0.8, P['leather_lo'])      # a grip long enough for two hands
    f.disc(*_at(h, u, -6.1), 1.1, P['trim'])
    f.limb(_at(h, u, 1.0, n, -3.0), _at(h, u, 1.0, n, 3.0), 0.7, P['trim'])
    f.limb(_at(h, u, 2.0), _at(h, u, 16.0), 1.2, P['metal_hi']); f.limb(_at(h, u, 2.0), _at(h, u, 15.5), 0.45, WHITE)
    f.dot(*_at(h, u, 16.8), P['metal_hi'])


def _battle_axe(f, P, h, u):
    n = (-u[1], u[0])
    _haft(f, P, h, u, 5.0, 13.5)
    f.poly([_at(h, u, 8.6, n, 0.8), _at(h, u, 7.0, n, 4.6), _at(h, u, 10.8, n, 5.6), _at(h, u, 14.6, n, 4.6), _at(h, u, 13.0, n, 0.8)], P['metal'])
    for a, b in ((_at(h, u, 7.0, n, 4.6), _at(h, u, 10.8, n, 5.6)), (_at(h, u, 10.8, n, 5.6), _at(h, u, 14.6, n, 4.6))):
        f.limb(a, b, 0.45, P['metal_hi'])                                   # the honed edge
    f.limb(_at(h, u, 9.4, n, 1.4), _at(h, u, 12.2, n, 1.4), 0.4, P['metal_lo'])
    f.poly([_at(h, u, 9.8, n, -0.6), _at(h, u, 11.0, n, -3.4), _at(h, u, 12.2, n, -0.6)], P['metal_lo'])  # back spike
    f.dot(*_at(h, u, 14.0), P['metal_lo'])


def _warhammer(f, P, h, u):
    n = (-u[1], u[0])
    _haft(f, P, h, u, 5.0, 12.0)
    f.poly([_at(h, u, 10.0, n, -3.0), _at(h, u, 10.0, n, 3.8), _at(h, u, 13.4, n, 3.8), _at(h, u, 13.4, n, -3.0)], P['metal'])
    f.limb(_at(h, u, 10.0, n, 4.2), _at(h, u, 13.4, n, 4.2), 0.5, P['metal_hi'])   # the striking face
    f.limb(_at(h, u, 10.4, n, -0.4), _at(h, u, 13.0, n, -0.4), 0.4, P['metal_lo'])
    f.poly([_at(h, u, 10.6, n, -3.0), _at(h, u, 11.7, n, -5.4), _at(h, u, 12.8, n, -3.0)], P['metal_lo'])  # back spike
    f.limb(_at(h, u, 13.4), _at(h, u, 15.0), 0.6, P['metal'])           # top spike


def _double_axe(f, P, h, u):
    """A two-handed double-bitted axe: two broad crescent heads, one each side of the haft, read at 28 px."""
    n = (-u[1], u[0])
    _haft(f, P, h, u, 5.0, 15.0)
    for k in (1.0, -1.0):
        f.poly([_at(h, u, 8.4, n, 0.8 * k), _at(h, u, 6.4, n, 5.4 * k), _at(h, u, 10.9, n, 6.4 * k), _at(h, u, 15.4, n, 5.4 * k),
                _at(h, u, 13.4, n, 0.8 * k)], P['metal'])
        for a, b in ((_at(h, u, 6.4, n, 5.4 * k), _at(h, u, 10.9, n, 6.5 * k)), (_at(h, u, 10.9, n, 6.5 * k), _at(h, u, 15.4, n, 5.4 * k))):
            f.limb(a, b, 0.5, P['metal_hi'])                                # both edges honed
        f.limb(_at(h, u, 9.4, n, 2.2 * k), _at(h, u, 12.4, n, 2.2 * k), 0.4, P['metal_lo'])
    f.poly([_at(h, u, 9.6, n, -1.1), _at(h, u, 9.6, n, 1.1), _at(h, u, 12.2, n, 1.1), _at(h, u, 12.2, n, -1.1)], P['metal_lo'])  # socket
    f.limb(_at(h, u, 13.4), _at(h, u, 16.4), 0.6, P['metal'])           # top spike


def _hand_axe(f, P, h, u):
    """A one-handed hatchet: a short haft and a small bearded head."""
    n = (-u[1], u[0])
    f.limb(_at(h, u, -2.5), _at(h, u, 9.5), 0.7, P['leather']); f.limb(_at(h, u, -2.5), _at(h, u, 0.5), 0.7, P['leather_lo'])
    f.poly([_at(h, u, 6.4, n, 0.6), _at(h, u, 5.2, n, 3.6), _at(h, u, 7.8, n, 4.3), _at(h, u, 10.2, n, 3.6), _at(h, u, 9.2, n, 0.6)], P['metal'])
    f.limb(_at(h, u, 5.2, n, 3.7), _at(h, u, 7.8, n, 4.4), 0.45, P['metal_hi']); f.limb(_at(h, u, 7.8, n, 4.4), _at(h, u, 10.2, n, 3.7), 0.45, P['metal_hi'])
    f.poly([_at(h, u, 7.2, n, -0.6), _at(h, u, 7.9, n, -1.9), _at(h, u, 8.6, n, -0.6)], P['metal_lo'])


DRAWERS = {'greatsword': _greatsword, 'battle_axe': _battle_axe, 'warhammer': _warhammer, 'double_axe': _double_axe, 'hand_axe': _hand_axe}
# WINDUP1: how far each weapon reaches past the hand along its angle (px; the debuff wind-up's glint sits at its tip)
WEAPON_TIP = {'sword': 12.0, 'sword2': 15.0, 'mace': 10.8, 'dagger': 6.8, 'staff': 11.8, 'greatsword': 16.8,
              'battle_axe': 14.0, 'warhammer': 15.0, 'double_axe': 16.4, 'hand_axe': 9.5}


def draw_bow(f, P, hand, bow_a, pull_hand=None, nock=False):
    bx, by = hand; r = math.radians(bow_a); ca, sa = math.cos(r), math.sin(r)
    def bp(al, out): return (bx + out * ca - al * sa, by - al * ca + out * sa)
    pts = [bp(t, 2.6 * math.cos(t / 9 * math.pi / 2) - 0.3) for t in [x * 0.5 for x in range(-18, 19)]]
    top, bot = bp(9, -0.3), bp(-9, -0.3)
    if pull_hand: f.limb(top, pull_hand, 0.3, WHITE); f.limb(bot, pull_hand, 0.3, WHITE)
    else: f.limb(top, bot, 0.3, WHITE)
    for i in range(len(pts) - 1): f.limb(pts[i], pts[i + 1], 0.8, P['wood'] if abs(i - 18) > 3 else P['wood_lo'])
    if nock and pull_hand:
        sx, sy = pull_hand; f.limb((sx, sy), (bx + 4, by), 0.4, P['wood']); f.dot(bx + 4.5, by, P['metal_hi']); f.dot(bx + 5.2, by, P['metal_hi'])
        f.dot(sx - 0.5, sy - 0.8, P['accent']); f.dot(sx - 0.5, sy + 0.8, P['accent'])


# ------------------------------------------------------------------ the pose
def pose(u, bob=0.0, lean=0.0, dx=0.0, fb=(-3.5, 0), ff=(3.5, 0), hand_f=None, hand_b=None, wa=25.0, wb=None,
         head_dy=0.0, hurt=0.0, fall=0.0, flash=False, two=False, loose=None, loose_b=None, fx=None, slash=None,
         bow_a=0.0, pull=0.0, nock=False, arrow=None, glow=0.0, fist_f=False, fist_b=False, tails=0.0, swap=False, leg_trail=None,
         grip2=0.0):
    """hand_f/hand_b: hand targets relative to the shoulder (None = rest pose). wa/wb: weapon angles
    (0 up, 90 forward, 180 down). ff/fb: foot offsets (x from pelvis, lift). loose: (x, y, angle) of a dropped weapon.
    grip2 > 0 (WAR2H1, a two-handed weapon): the far hand holds the grip grip2 px behind the near hand, along the
    weapon (hand_b is ignored), and is drawn over the grip."""
    P = u['pal']; L, A, T = u['leg'], u['arm'], u['tl']
    s = u.get('scale', 1.0)  # a body variant's size against its class (goblins ~0.83, a warlord ~1.1): every offset scales
    if s != 1.0:
        sc = lambda p: None if p is None else (p[0] * s, p[1] * s)
        about = lambda x, y: (20 + (x - 20) * s, GY - (GY - y) * s)  # absolute effect positions scale about the feet
        bob, lean, dx, head_dy, grip2 = bob * s, lean * s, dx * s, head_dy * s, grip2 * s
        fb, ff, hand_f, hand_b = sc(fb), sc(ff), sc(hand_f), sc(hand_b)
        fx = [about(x, y) + (r, c) for x, y, r, c in fx] if fx else fx
        slash = [about(x, y) for x, y in slash] if slash else slash
        arrow = about(*arrow) if arrow else arrow
        loose = about(loose[0], loose[1]) + (loose[2],) if loose else loose
        loose_b = about(loose_b[0], loose_b[1]) + (loose_b[2],) if loose_b else loose_b
        leg_trail = [sc(p) for p in leg_trail] if leg_trail else leg_trail
    f = F()
    px, py = 20 + dx + fall * 10 * s, GY - 2 * L + bob
    lx = lean + hurt * -2.5
    sh = (px + lx, py - T + fall * 3)
    head = (sh[0] + 1 + lx * 0.3 - fall * 3, sh[1] - u.get('neck', 6) + head_dy + fall * 2)
    ang = -fall * math.radians(84)
    piv = (px, GY - fall * 3.5)
    def R(p):
        if not fall: return p
        ddx, ddy = p[0] - piv[0], p[1] - piv[1]
        return (piv[0] + ddx * math.cos(ang) - ddy * math.sin(ang), piv[1] + ddx * math.sin(ang) + ddy * math.cos(ang))
    foot_b = (px + fb[0], GY - fb[1]); foot_f = (px + ff[0], GY - ff[1])
    if fall: foot_b, foot_f = (px - 3, GY), (px + 3, GY)
    kb, fbp = ik((px - 1, py), L, L, foot_b, -1)
    kf, ffp = ik((px + 1, py), L, L, foot_f, -1)
    hf = (sh[0] + 4 * s, sh[1] + 6.5 * s) if hand_f is None else (sh[0] + hand_f[0], sh[1] + hand_f[1] + bob * 0.0)
    hb = (sh[0] - 2.5 * s, sh[1] + 8.5 * s - hurt * 2 * s) if hand_b is None else (sh[0] + hand_b[0], sh[1] + hand_b[1])
    if swap:  # main weapon in the far hand (behind the body), off-hand item (shield) in the near hand
        eb, hb = ik((sh[0] - 2, sh[1] - 0.5), A, A, hf, 1)
        ef, hf = ik((sh[0] + 2, sh[1] - 0.5), A, A, (sh[0] + 4.5 * s, sh[1] + 3 * s) if hand_b is None else (sh[0] + hand_b[0] + 2 * s, sh[1] + hand_b[1]), -1)
    else:
        ef, hf = ik((sh[0] + 2, sh[1] - 0.5), A, A, hf, u.get('fbend', -1))
        if grip2 and not loose:
            gx, gy = udir(wa); hb = (hf[0] - gx * grip2, hf[1] - gy * grip2)
        eb, hb = ik((sh[0] - 2, sh[1] - 0.5), A, A, hb, 1)
    wf, wbk = u['weapon'], u.get('offhand')
    sleeve, sleeve_lo, fore, fore_lo = P.get('sleeve', P['main']), P.get('sleeve_lo', P['main_lo']), P.get('fore', P['skin']), P.get('fore_lo', P['skin_lo'])
    if fall and not loose: wa = wa + math.degrees(ang)
    # behind the body
    draw_back_gear(f, u, P, R, sh, px, py, T, lx)
    if wbk == 'dagger' and not loose_b: draw_weapon(f, u, P, 'dagger', R(hb), (wb if wb is not None else 160) + (math.degrees(ang) if fall else 0))
    if swap and not loose: draw_weapon(f, u, P, wf, R(hb), wa, two, glow)
    f.limb(R((sh[0] - 2, sh[1])), R(eb), 1.7, sleeve_lo); f.limb(R(eb), R(hb), 1.5, fore_lo)
    f.disc(*R(hb), 1.9 if fist_b else 1.5, P.get('glove_lo', fore_lo))
    # WAR2H1 grips ('2h', 'buckler'): a weapon pointing back (a wind-up, a low guard) goes behind the head and body
    back2 = u.get('grip') in ('2h', 'buckler') and not loose and wa < 0
    if back2:
        draw_weapon(f, u, P, wf, R(hf), wa, two, glow)
        if grip2: f.disc(*R(hb), 1.5, P.get('glove_lo', P['leather_lo']))

    f.limb(R((px - 1, py)), R(kb), 2.2, P['leg_lo']); f.limb(R(kb), R(fbp), 2.0, P['leg_lo'])
    f.rect(*R((fbp[0] - 1, fbp[1] - 1)), *R((fbp[0] + 3, fbp[1])), P['boot_lo'])
    draw_torso(f, u, P, R, px, py, lx, T)
    if u['torso'] == 'plate' or u.get('pauldron'):
        f.disc(*R((sh[0] + 2.2, sh[1] + 0.5)), 2.6, P['metal']); f.dot(*R((sh[0] + 1.4, sh[1] - 0.7)), P['metal_hi'])
    f.rect(*R((head[0] - 1, head[1] + 2)), *R((head[0] + 1, head[1] + 4)), P['skin_lo'])
    hx, hy = R(head)
    draw_head(f, u, P, hx, hy, hurt, tails)
    f.limb(R((px + 1, py)), R(kf), 2.4, P['leg']); f.limb(R(kf), R(ffp), 2.2, P['leg'], P['leg_lo'])
    f.rect(*R((ffp[0] - 1, ffp[1] - 1)), *R((ffp[0] + 3, ffp[1])), P['boot'])
    draw_robe_skirt(f, u, P, R, px, py, kb, kf, fbp, ffp)
    if wbk == 'shield' and not loose_b and not swap:  # shield on the far arm, carried in front of the chest
        draw_weapon(f, u, P, 'shield', R(hb), math.degrees(ang) if fall else 0)
    elif wbk == 'buckler' and not loose_b and not swap:  # buckler strapped over the far fist, in front of the chest
        draw_weapon(f, u, P, 'buckler', R(hb), 0)
    # front arm + weapon
    f.limb(R((sh[0] + 2, sh[1])), R(ef), 1.9, sleeve); f.limb(R(ef), R(hf), 1.7, fore)
    hfx, hfy = R(hf)
    if wf == 'bow':
        pull_hand = None
        if pull > 0 and not loose: pull_hand = R(hb)
        if loose: draw_bow(f, P, loose[:2], loose[2]); f.disc(hfx, hfy, 1.4, P['skin'])
        else:
            draw_bow(f, P, (hfx, hfy), bow_a + (math.degrees(ang) if fall else 0), pull_hand, nock); f.disc(hfx, hfy, 1.5, P.get('glove', P['leather_lo']))
            if pull_hand: f.disc(*pull_hand, 1.4, P.get('glove', P['leather_lo']))
    elif wf == 'fists':
        f.disc(hfx, hfy, 2.0 if fist_f else 1.6, P['glove']); f.dot(hfx + 0.8, hfy - 0.8, P.get('glove_hi', P['glove']))
    elif swap:
        f.disc(hfx, hfy, 1.6, P.get('glove', P['leather']))
        if wbk == 'shield': draw_weapon(f, u, P, 'shield', (hfx, hfy), 0)
    else:
        if loose: draw_weapon(f, u, P, wf, loose[:2], loose[2]); f.disc(hfx, hfy, 1.4, P['skin'])
        else:
            if not back2:
                draw_weapon(f, u, P, wf, (hfx, hfy), wa, two, glow)
            if grip2 and not back2:  # the far hand over the lower grip, in shade
                f.disc(*R(hb), 1.5, P.get('glove_lo', P['leather_lo']))
            if wf == 'mace':  # a smaller fist round the shaft, knuckles shaded
                f.disc(hfx, hfy, 1.3, P.get('glove', P['leather'])); f.dot(hfx + 0.6, hfy + 0.4, P.get('glove_lo', P['leather_lo']))
            else:
                f.disc(hfx, hfy, 1.6, P.get('glove', P['leather']))
    if loose_b and wbk: draw_weapon(f, u, P, wbk, loose_b[:2], loose_b[2])
    if slash:
        for i, (x, y) in enumerate(slash):
            f.disc(x, y, 0.5 + 1.0 * i / len(slash), P.get('slash', P['metal_hi']) if i < len(slash) * 0.6 else WHITE)
    f.outline()
    trail = leg_trail or []
    for i, (tx, tl) in enumerate(trail):  # motion smear of the near leg: pale shin ghosts, newest brightest, plus the foot's path
        k = (i + 1) / len(trail)
        col = tuple(int(a + (b - a) * k) for a, b in zip((86, 94, 120), (196, 204, 228)))
        tk, tf = ik((px + 1, py), L, L, (px + tx, GY - tl), -1)
        sx, sy = tf[0] + (tk[0] - tf[0]) * 0.3, tf[1] + (tk[1] - tf[1]) * 0.3      # only the lower shin and foot
        f.limb(R((sx, sy)), R(tf), 0.5 + 0.5 * k, col)
        if i + 1 < len(trail):
            _, nf = ik((px + 1, py), L, L, (px + trail[i + 1][0], GY - trail[i + 1][1]), -1)
            f.limb(R(tf), R(nf), 0.5, col)
    for (x, y, r, c) in (fx or []): f.disc(x, y, r, c)
    if arrow is not None:
        ax, ay = arrow
        for k in range(7): f.dot(ax - k, ay, P['wood'])
        f.dot(ax + 1, ay, P['metal_hi']); f.dot(ax - 6, ay - 1, P['accent']); f.dot(ax - 6, ay + 1, P['accent'])
    if flash:
        for y in range(W):
            for x in range(W):
                if f.px[x, y][3]: f.px[x, y] = (255, 255, 255, 255) if f.px[x, y][:3] != OUT else (255, 220, 220, 255)
    # WINDUP1: where the main hand and the main weapon's tip are (image metadata, for the wind-up effects)
    f.im.info['hand'] = (hfx, hfy)
    if wf in WEAPON_TIP and not loose and not swap:
        ux, uy = udir(wa); f.im.info['tip'] = (hfx + ux * WEAPON_TIP[wf], hfy + uy * WEAPON_TIP[wf])
    return f.im


# ------------------------------------------------------------------ shared clips (any unit)
def idle_keys(frames=8, hf_amp=0.8, alt=0.0, **kw):
    """The pose keys of each idle frame (idle draws them; WINDUP1's wind-up intros start from frame 0's)."""
    out = []
    for i in range(frames):
        t = i / frames * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t - 0.6); lag = 0.5 + 0.5 * math.sin(t - 1.4)
        k = dict(kw); base_f = k.pop('hand_f', None); base_b = k.pop('hand_b', None)
        hf = None if base_f is None else (base_f[0], base_f[1] + lag * hf_amp)
        hb = None if base_b is None else (base_b[0], base_b[1] + lag * 0.6)
        if alt:  # fists bob up/down in opposite phase (boxer 1-2 bounce) instead of moving together
            a = math.sin(t)
            hf = (base_f[0], base_f[1] - a * alt)
            hb = (base_b[0], base_b[1] + a * alt)
        out.append(dict(bob=k.pop('bob', 0.0) + b * 0.9, head_dy=lag * 0.8 - 0.2, lean=k.pop('lean', 0.0) + math.sin(t) * 0.4,
                        hand_f=hf, hand_b=hb, fb=(-3.5 - b * 0.4, 0), ff=(3.5 + b * 0.4, 0), tails=t, **k))
    return out


def idle(u, frames=8, hf_amp=0.8, alt=0.0, **kw):
    return [pose(u, **k) for k in idle_keys(frames, hf_amp, alt, **kw)], [160] * frames


def walk(u, frames=8, stride=6.0, hf_amp=0.5, **kw):
    out = []
    for i in range(frames):
        ph = i / frames; fa = gait(ph, stride); fbk = gait(ph + 0.5, stride); s = math.sin(2 * math.pi * ph)
        bob = 0.9 - 1.8 * abs(s)
        k = dict(kw); base_f = k.pop('hand_f', (4, 6.5)); base_b = k.pop('hand_b', (-2.5, 8.5)); swing = k.pop('swing', 1.0)
        hf = (base_f[0] + s * 1.2 * swing, base_f[1] + abs(s) * hf_amp)
        hb = (base_b[0] - s * 1.8 * swing, base_b[1] + abs(s) * 0.4)
        out.append(pose(u, bob=k.pop('bob', 0.0) + bob, lean=k.pop('lean', 1.4), ff=fa, fb=fbk, head_dy=bob * 0.25,
                        hand_f=hf, hand_b=hb, tails=ph * 6, **k))
    return out, [90] * frames


def hurt(u, **kw):
    k = dict(kw)
    return ([pose(u, bob=1.2, hurt=1.0, lean=-1.5, dx=-1.0, flash=True, head_dy=0.8, fb=(-4, 0), ff=(4, 0), **k),
             pose(u, bob=1.6, hurt=1.0, lean=-2.5, dx=-2.0, head_dy=1.2, fb=(-5, 0), ff=(4, 0), **k),
             pose(u, bob=0.8, hurt=0.5, lean=-1.0, dx=-1.0, fb=(-4, 0), ff=(4, 0), **k),
             pose(u, bob=0.3, **k)], [60, 120, 90, 90])


def death(u, drop=(30, 45, 92), drop_b=None, **kw):
    k = dict(kw)
    lb = lambda y: None if drop_b is None else (drop_b[0], y, drop_b[2])
    return ([pose(u, bob=1.5, hurt=1.0, lean=-1.5, dx=-1.5, head_dy=1, flash=True, **k),
             pose(u, bob=1.8, hurt=1.0, lean=-2.5, dx=-2.5, head_dy=1.2, loose=(drop[0], 24, drop[2] - 50), loose_b=lb(26), **k),
             pose(u, bob=1.2, hurt=1.0, fall=0.22, lean=-1.0, dx=-2.0, loose=(drop[0] + 1, 33, drop[2] - 12), loose_b=lb(34), **k),
             pose(u, bob=0.8, hurt=1.0, fall=0.5, dx=-1.5, loose=(drop[0], 41, drop[2] + 8), loose_b=lb(41), **k),
             pose(u, bob=0.3, hurt=1.0, fall=0.82, dx=-1.0, loose=drop, loose_b=lb(drop_b[1]) if drop_b else None, **k),
             pose(u, bob=0.0, hurt=1.0, fall=1.0, dx=-1.0, loose=drop, loose_b=lb(drop_b[1]) if drop_b else None, **k)], [60, 140, 100, 90, 90, 600])


def impact(im):
    """FX1-L1 (decision "Attack timing on the board" A): marks `im` as its attack clip's impact frame, where the blow
    lands or the shot leaves (the frame with the hit spark). The exporter writes its index as the clip's `impact` in
    clips.json; the mark is image metadata only (no pixel changes). Returns `im`."""
    im.info['impact'] = True
    return im


def impact_frame(frames):
    """The index of the first frame marked by `impact` (None: the clip marks none)."""
    return next((i for i, fr in enumerate(frames) if fr.info.get('impact')), None)


# ------------------------------------------------------------------ wind-ups, stagger and dazed (WINDUP1 section 7)
# games/emberwake/docs/specs/WINDUP1.md: a unit whose action spans several blocks raises its weapon (an intro), holds
# the pose breathing until the impact block, then its attack plays from the `strike` frame. Per body (one set per grip)
# the rig draws six rows: the four wind-ups (attack, heavy, buff, debuff; 7 frames, the loop from frame 3), `stagger`
# (the reaction to a break, 5 frames once) and `dazed` (the loop while Staggered, 8 frames). The helpers here take a
# draw function (pose keys -> frame), so the side rig (pose) and the views (views.pose_view) share them:
#   K1 the intro lerps the pose keys (numbers and tuples) from the ready pose (idle frame 0) to the hold pose at
#      k = INTRO; the hold is four frames bobbing HOLD_BOB px; windup_heavy trembles TREMBLE px across; windup_buff
#      adds two gold motes (GOLDEN, GOLDEN_HI) rising 1 px a frame from the main hand; windup_debuff a 1 px WHITE
#      glint at the weapon's tip on hold frames 0 and 2.
#   K3 / K4 stagger: four keys of the grip, then dazed frame 0; dazed: the slumped keys swaying dx 0.8 sin and
#      head_dy 0.5 cos over its 8 frames. No flash and no new colour (K5).
WINDUP_MS = {'windup': [90, 110, 120, 180, 180, 180, 180], 'windup_heavy': [100, 120, 140, 160, 160, 160, 160],
             'windup_buff': [90, 110, 120, 180, 180, 180, 180], 'windup_debuff': [90, 110, 120, 180, 180, 180, 180]}
WINDUPS = tuple(WINDUP_MS)
STAGGER_MS, DAZED_MS = [60, 90, 110, 120, 160], [150] * 8
REACTIONS = ('stagger', 'dazed')
WINDUP_ROWS = WINDUPS + REACTIONS     # the six rows, in sheet order (K7)
LOOP_FROM = 3                         # the first looped frame of a wind-up (after its three intro frames)
INTRO = (0.4, 0.75, 1.0)
HOLD_BOB = (0.0, 0.5, 1.0, 0.5)
TREMBLE = (0.0, 0.5, 0.0, -0.5)
# the values pose() draws for a key it is not given (None hands: the rest pose), so an intro can lerp from them
SIDE_REST = dict(bob=0.0, lean=0.0, dx=0.0, fb=(-3.5, 0), ff=(3.5, 0), hand_f=(4.0, 6.5), hand_b=(-2.5, 8.5), wa=25.0,
                 wb=160.0, head_dy=0.0, glow=0.0, tails=0.0, bow_a=0.0, pull=0.0)


def strike(im):
    """WINDUP1 (table 7.2): marks `im` as its attack clip's `strike` frame, the first frame after the clip's own wind-up
    frames, where the clip starts after a held wind-up. The exporter writes its index as the clip's `strike`;
    metadata only (no pixel changes). Returns `im`."""
    im.info['strike'] = True
    return im


def strike_frame(frames):
    """The index of the first frame marked by `strike` (None: the clip marks none)."""
    return next((i for i, fr in enumerate(frames) if fr.info.get('strike')), None)


def loop_start(im):
    """WINDUP1: marks `im` as its looping clip's first looped frame (the frames before it are the intro); the exporter
    writes its index as `loop_from`. Metadata only. Returns `im`."""
    im.info['loop_from'] = True
    return im


def loop_frame(frames):
    """The index of the frame marked by `loop_start` (None: the clip loops whole)."""
    return next((i for i, fr in enumerate(frames) if fr.info.get('loop_from')), None)


ANGLES = ('wa', 'wb', 'bow_a')  # weapon angles lerp the short way round


def lerp_kw(a, b, k, rest=SIDE_REST):
    """Pose keys `k` of the way from `a` to `b`: numbers and number tuples lerp (a key missing, or None, on one side
    takes `rest`'s value; an angle of ANGLES the short way round); any other value (flags, layers, effects) is `b`'s,
    left out when only `a` has it."""
    out = {}
    for key in list(a) + [x for x in b if x not in a]:
        va, vb = a.get(key), b.get(key)
        va = rest.get(key) if va is None else va
        vb = rest.get(key) if vb is None else vb
        if isinstance(vb, bool) or isinstance(va, bool) or va is None or vb is None:
            if key in b or key in rest:  # a flag or a layer only the ready pose sets ends with the intro
                out[key] = b.get(key, rest.get(key))
        elif isinstance(va, (int, float)) and isinstance(vb, (int, float)):
            d = (vb - va + 180.0) % 360.0 - 180.0 if key in ANGLES else vb - va
            out[key] = vb if k == 1.0 else va + d * k
        elif isinstance(va, tuple) and isinstance(vb, tuple) and len(va) == len(vb):
            out[key] = tuple(x + (y - x) * k for x, y in zip(va, vb))
        elif key in b:
            out[key] = b[key]
    return out


def overlay(im, dots):
    """Draws (x, y, colour) one-pixel dots on frame `im` (over the outline, as pose's fx); returns `im`."""
    for x, y, c in dots:
        x, y = int(round(x)), int(round(y))
        if 0 <= x < im.width and 0 <= y < im.height:
            im.putpixel((x, y), tuple(c[:3]) + (255,))
    return im


def shake_dx(kw, d):
    """The heavy wind-up's tremble on the side rig: the figure moved `d` px across (dx)."""
    kw['dx'] = kw.get('dx', 0.0) + d


def windup(kind, ready, hold, draw, rest=SIDE_REST, shake=shake_dx):
    """(frames, ms) of wind-up `kind` (WINDUPS) from the `ready` keys to the `hold` keys (K1). `draw(keys)` draws one
    frame and puts the main hand (and the weapon tip, when it has one) in the frame's info ('hand', 'tip'), as pose
    and views.pose_view do; `shake(keys, d)` applies the heavy tremble of d px."""
    frames = [draw(lerp_kw(ready, hold, k, rest)) for k in INTRO]
    for i in range(len(HOLD_BOB)):
        kw = dict(hold)
        kw['bob'] = kw.get('bob', 0.0) + HOLD_BOB[i]
        if kind == 'windup_heavy':
            shake(kw, TREMBLE[i])
        im = draw(kw)
        hx, hy = im.info['hand']
        if kind == 'windup_buff':  # two motes rise from the hand, a frame apart in their 4-frame climb
            overlay(im, [(hx - 1.5, hy - 2 - i, GOLDEN), (hx + 1.5, hy - 2 - (i + 2) % 4, GOLDEN_HI)])
        if kind == 'windup_debuff' and i in (0, 2) and im.info.get('tip'):
            tx, ty = im.info['tip']; d = math.hypot(tx - hx, ty - hy) or 1.0
            overlay(im, [(tx + (tx - hx) / d, ty + (ty - hy) / d, WHITE)])  # just past the tip, over the outline
        frames.append(im)
    loop_start(frames[LOOP_FROM])
    return frames, list(WINDUP_MS[kind])


def dazed(keys, draw):
    """(frames, ms) of `dazed` (K4): the slumped `keys` swaying dx 0.8 sin and head_dy 0.5 cos over 8 frames."""
    out = []
    for i in range(8):
        t = i / 8 * 2 * math.pi
        kw = dict(keys)
        kw['dx'] = kw.get('dx', 0.0) + 0.8 * math.sin(t)
        kw['head_dy'] = kw.get('head_dy', 0.0) + 0.5 * math.cos(t)
        out.append(draw(kw))
    return out, list(DAZED_MS)


def windup_rows(draw, ready, holds, stagger_keys, dazed_keys, rest=SIDE_REST, shake=shake_dx):
    """The six WINDUP1 rows of one grip (WINDUP_ROWS order): {id: frames}, {id: ms}. holds: {wind-up id: hold keys}
    (a grip may leave a wind-up out; the screen falls back); stagger_keys: its first four keys (K3; frame 4 is dazed
    frame 0); dazed_keys: the slumped pose (K4)."""
    A, T = {}, {}
    for kind in WINDUPS:
        if kind in holds:
            A[kind], T[kind] = windup(kind, ready, holds[kind], draw, rest, shake)
    dz, dzt = dazed(dazed_keys, draw)
    first = dazed(dazed_keys, lambda k: k)[0][0]  # dazed frame 0's keys: the stagger ends in it
    A['stagger'], T['stagger'] = [draw(dict(k)) for k in stagger_keys] + [draw(first)], list(STAGGER_MS)
    A['dazed'], T['dazed'] = dz, dzt
    return A, T


def mirror(im, dx=0.0):
    from PIL import ImageOps
    m = ImageOps.mirror(im); out = Image.new("RGBA", im.size); shift = int(round(2 * (20 + dx) - (W - 1)))
    out.alpha_composite(m, (shift, 0)) if shift >= 0 else out.alpha_composite(m.crop((-shift, 0, W, W)), (0, 0))
    return out


def export(name, A, T, out, bgc=(46, 52, 44), Z=6):
    names = list(A); cols = max(len(v) for v in A.values())
    sheet = Image.new("RGBA", (cols * W, len(names) * W))
    prev = Image.new("RGBA", (cols * W * Z, len(names) * W * Z), bgc + (255,))
    def bg(im):
        b = Image.new("RGBA", (W, W), bgc + (255,)); b.alpha_composite(im); return b.resize((W * Z, W * Z), Image.NEAREST)
    for r, n in enumerate(names):
        for c, fr in enumerate(A[n]):
            sheet.paste(fr, (c * W, r * W)); prev.paste(bg(fr), (c * W * Z, r * W * Z))
        g = [bg(fr).convert("P", palette=Image.ADAPTIVE) for fr in A[n]]
        g[0].save(f"{out}/{name}_{n}.gif", save_all=True, append_images=g[1:], duration=T[n], loop=0)
    sheet.save(f"{out}/{name}_sheet.png"); prev.convert("RGB").save(f"{out}/{name}_preview.png")
    return {n: len(v) for n, v in A.items()}



# ------------------------------------------------------------------ creature skeletons (CREATURE1): non-human bodies
# Two generic skeletons beside the humanoid one, in the same frame (W x W, facing right, feet on GY) with the same
# clips (idle, walk, attack, hurt, death). A creature is a dict like a rig unit: proportions, feature options and a
# palette `pal` whose keys are the body's palette slots (a palette file recolours them as it does a humanoid's).
#   quadruped: a spine from hip to shoulder, a neck and a head with a hinged jaw, four legs on IK (front: elbow back;
#     hind: knee forward, hock back), a tail when tail[0] > 0. quad_pose draws one frame, quad_clips the clip set.
#   crawler: a chain of n segments laid along a path from the head (chain_along), a pair of legs per segment whose
#     step phases alternate along the chain and between the near and far side. crawler_pose draws one frame,
#     crawler_clips the clip set (plus `spit`, an acid spit, when the creature has spit=True).
def mix(a, b, t):
    """Colour `a` moved `t` of the way to `b` (a computed shade step between two palette hues)."""
    return tuple(int(round(x + (y - x) * t)) for x, y in zip(a[:3], b[:3]))


def _flash(f):
    for y in range(W):
        for x in range(W):
            if f.px[x, y][3]: f.px[x, y] = (255, 255, 255, 255) if f.px[x, y][:3] != OUT else (255, 220, 220, 255)


def _v(p, *terms):
    """p plus each (vector, factor) of `terms`."""
    x, y = p
    for v, k in terms:
        x += v[0] * k; y += v[1] * k
    return (x, y)


def _line(f, a, b, c, behind=False):  # a one-pixel line (behind: only where nothing is drawn yet)
    n = int(max(abs(b[0] - a[0]), abs(b[1] - a[1]))) + 1
    for i in range(n + 1):
        t = i / n; x, y = int(round(a[0] + (b[0] - a[0]) * t)), int(round(a[1] + (b[1] - a[1]) * t))
        if not behind or (0 <= x < W and 0 <= y < W and f.px[x, y][3] == 0): f.dot(x, y, c)


P_RED = (225, 29, 72)   # a bite's spray (arterial red), an effect colour like the humanoids' sparks

# A quadruped's proportions (px) and options; a creature overrides what it needs. stance: the pose every clip starts
# from (pitch: degrees the spine tips, front down when positive; neck_a: the neck's angle, 0 forward, negative up;
# head_a: the head's, nose down when positive; bob: px the body sits lower; paws: rest offsets of the paws).
QUAD = dict(cx=20.0, body=14.0, hip_h=12.5, sh_h=13.0, arch=1.0, haunch=3.4, waist=2.4, chest=4.2,
            fore=(5.2, 5.0), hind=(5.0, 4.6, 3.0), neck=5.0, head=2.7, snout=4.8, jaw_len=4.4,
            tail=(5, 2.0), tail_a=-25.0, tail_curl=6.0, ears='pointed', ridges=0, ribs=0, wire=False,
            stance=dict(pitch=0.0, neck_a=-30.0, head_a=10.0, bob=0.0, paws={}))


def quad_pose(q, dx=0.0, bob=0.0, pitch=0.0, neck_a=-30.0, head_a=10.0, jaw=0.0, paws=None, tail=0.0, tail_a=None,
              hurt=False, flash=False, fx=None):
    """One frame of quadruped `q`. paws: 'nf' / 'ff' / 'nh' / 'fh' (near and far, front and hind) -> (x offset from
    the paw's rest spot, lift); jaw: degrees open; tail: the tail's sway (radians of phase); tail_a: its angle
    (degrees above straight back; default q's); fx: discs (x, y, r, colour) drawn last."""
    P = q['pal']; paws = paws or {}
    hide, lo, dk = P['hide'], P['hide_lo'], P.get('hide_dk', P['hide_lo'])
    f = F()
    a = math.radians(pitch); along = (math.cos(a), math.sin(a)); up = (math.sin(a), -math.cos(a))
    mid = (q['cx'] + dx, GY - (q['hip_h'] + q['sh_h']) / 2 + bob)
    H = _v(mid, (along, -q['body'] / 2), (up, (q['hip_h'] - q['sh_h']) / 2))
    S = _v(mid, (along, q['body'] / 2), (up, (q['sh_h'] - q['hip_h']) / 2))

    def spine(t):
        return _v(H, ((S[0] - H[0], S[1] - H[1]), t), (up, q['arch'] * math.sin(math.pi * t)))

    def rad(t):
        pts = ((0.0, q['haunch']), (0.5, q['waist']), (0.82, q['chest']), (1.0, q['chest'] * 0.86))
        for (t0, r0), (t1, r1) in zip(pts, pts[1:]):
            if t <= t1: return r0 + (r1 - r0) * (t - t0) / (t1 - t0)
        return pts[-1][1]

    def centre(t):
        return _v(spine(t), (up, -(rad(t) - 0.7)))

    def leg(name, front, col, col_lo):
        ox, lift = paws.get(name, (0.0, 0.0))
        if front:
            root = _v(centre(0.8), (along, 0.6))
            tgt = (S[0] + 0.5 + ox, GY - 0.5 - lift)
            knee, foot = ik(root, q['fore'][0], q['fore'][1], tgt, 1)
            f.limb(root, knee, 1.8, col); f.limb(knee, foot, 1.1, col, col_lo)
        else:
            root = _v(centre(0.12), (along, -0.4))
            tgt = (H[0] - 0.5 + ox, GY - 0.5 - lift)
            m = q['hind'][2]; hock = (tgt[0] - m * 0.42, tgt[1] - m * 0.9)
            knee, hk = ik(root, q['hind'][0], q['hind'][1], hock, -1)
            f.limb(root, knee, 2.3, col); f.limb(knee, hk, 1.2, col); f.limb(hk, tgt, 0.9, col, col_lo)
            foot = tgt
        f.disc(foot[0] + 0.8, foot[1] - 0.1, 1.0, col)
        f.dot(foot[0] + 1.9, foot[1] + 0.4, P.get('claw', col_lo))

    # far side: legs in the dark shade, behind the body
    leg('fh', False, dk, dk); leg('ff', True, dk, dk)
    # tail
    if q['tail'][0]:
        n, sl = q['tail']; p = _v(spine(0.0), (along, -0.8), (up, -0.4))
        ang = q['tail_a'] if tail_a is None else tail_a
        for k in range(n):
            ak = math.radians(ang - q['tail_curl'] * k + 9 * math.sin(tail + k * 0.8) * k / n) + a
            nxt = (p[0] - math.cos(ak) * sl, p[1] - math.sin(ak) * sl)
            f.limb(p, nxt, max(0.5, 1.2 - 0.7 * k / n), hide if k < n - 1 else lo); p = nxt
    # torso: the shade, the body over it one px up, a lit back line
    ts = [i / 24 for i in range(25)]
    for t in ts: f.disc(*centre(t), rad(t), lo)
    for t in ts: f.disc(*_v(centre(t), (up, 1.1)), rad(t) - 1.1, hide)
    for t in ts[3:-3]: f.dot(*_v(spine(t), (up, -0.9)), P.get('hide_hi', hide))
    for k in range(q['ribs']):  # a starved chest: the ribs show as short dark lines
        t = 0.5 + k * 0.08; top = _v(spine(t), (up, -2.0)); bot = _v(centre(t), (up, -rad(t) * 0.55))
        _line(f, top, bot, lo)
    for k in range(q['ridges']):  # bone ridges along the spine, raked back, tallest over the shoulders
        t = 0.1 + 0.8 * k / max(1, q['ridges'] - 1); b = spine(t); h = 1.8 + 1.4 * t
        tip = _v(b, (up, h), (along, -1.1))
        f.poly([_v(b, (along, -1.0)), _v(b, (along, 1.0)), tip], P['bone']); f.dot(*_v(b, (along, -0.6), (up, 0.3)), P.get('bone_lo', lo))
    # neck and head
    nb = _v(centre(0.94), (up, 0.6))
    na = math.radians(neck_a); hd = (nb[0] + math.cos(na) * q['neck'], nb[1] + math.sin(na) * q['neck'])
    f.limb(nb, hd, 2.0, hide, lo)
    ha = math.radians(head_a); fw = (math.cos(ha), math.sin(ha)); hu = (math.sin(ha), -math.cos(ha))
    r, sn = q['head'], q['snout']
    if q['ears'] == 'pointed':
        f.poly([_v(hd, (hu, 1.6), (fw, -0.2)), _v(hd, (hu, 4.4), (fw, -2.6)), _v(hd, (hu, 0.6), (fw, -2.4))], hide)
        f.dot(*_v(hd, (hu, 2.2), (fw, -1.6)), lo)
    elif q['ears'] == 'bat':  # big bare ears swept back
        f.poly([_v(hd, (hu, 1.4), (fw, 0.2)), _v(hd, (hu, 4.8), (fw, -3.4)), _v(hd, (hu, 0.4), (fw, -2.6))], hide)
        f.poly([_v(hd, (hu, 1.6), (fw, -0.8)), _v(hd, (hu, 3.6), (fw, -2.7)), _v(hd, (hu, 1.0), (fw, -2.2))], lo)
    f.disc(*hd, r, hide)
    J = _v(hd, (hu, -1.3), (fw, -0.2))
    ja = ha + math.radians(jaw); jf = (math.cos(ja), math.sin(ja)); ju = (math.sin(ja), -math.cos(ja))
    jl = q['jaw_len']
    lower = [_v(J, (ju, 0.6)), _v(J, (jf, jl), (ju, 0.2)), _v(J, (jf, jl - 0.7), (ju, -0.9)), _v(J, (jf, 0.6), (ju, -1.3))]
    upper = [_v(hd, (hu, 1.9), (fw, 0.4)), _v(hd, (fw, sn), (hu, 0.9)), _v(hd, (fw, sn + 0.5), (hu, -0.3)),
             _v(hd, (fw, sn * 0.9), (hu, -1.0)), _v(hd, (fw, 0.6), (hu, -1.1))]
    if jaw > 6:  # the open mouth: gums between the jaws, teeth on both edges
        f.poly([_v(hd, (fw, 0.6), (hu, -1.0)), _v(hd, (fw, sn * 0.9), (hu, -0.9)), _v(J, (jf, jl * 0.95), (ju, 0.3)), _v(J, (ju, 0.6))],
               P.get('mouth', lo))
    f.poly(lower, lo)
    f.poly(upper, hide)
    f.dot(*_v(hd, (fw, sn + 0.4), (hu, 0.2)), P.get('nose', lo))
    if jaw > 6:
        for k in (0.45, 0.7, 0.92): f.dot(*_v(hd, (fw, sn * k), (hu, -1.6)), P.get('teeth', WHITE))
        for k in (0.55, 0.85): f.dot(*_v(J, (jf, jl * k), (ju, 0.9)), P.get('teeth', WHITE))
    if q['wire']:  # iron wire bound round each jaw; a strand between them pulls taut when the jaw opens
        m = sn * 0.55; ml = jl * 0.55
        top, bot = _v(hd, (fw, m), (hu, 1.0)), _v(hd, (fw, m), (hu, -1.0))
        _line(f, top, bot, P['wire']); f.dot(*top, P.get('wire_hi', P['wire']))
        jt, jb = _v(J, (jf, ml), (ju, 0.4)), _v(J, (jf, ml), (ju, -1.0))
        _line(f, jt, jb, P['wire'])
        if jaw > 6: _line(f, bot, jt, P.get('wire_hi', P['wire']))
    f.dot(*_v(hd, (fw, 1.1), (hu, 0.7)), OUT if hurt else P['eye'])
    f.dot(*_v(hd, (fw, 0.6), (hu, 1.5)), lo)  # brow
    # near side: legs in the body colour, over it
    leg('nh', False, hide, lo); leg('nf', True, hide, lo)
    f.outline()
    for (x, y, rr, c) in (fx or []): f.disc(x, y, rr, c)
    if flash: _flash(f)
    return f.im


def quad_clips(q):
    """(frames, ms) of quadruped `q`: idle, walk (a trot), attack (a lunge and bite), hurt, death."""
    st = q['stance']; A, T = {}, {}
    base = dict(pitch=st['pitch'], neck_a=st['neck_a'], head_a=st['head_a'], bob=st['bob'])
    rest = st['paws']

    def P(paws=None, **kw):
        k = dict(base); k.update(kw)
        pw = dict(rest)
        pw.update(paws or {})
        return quad_pose(q, paws=pw, **k)

    fr = []
    for i in range(8):  # breathing: the chest rises, the head sways, the tail twitches
        t = i / 8 * 2 * math.pi; b = math.sin(t)
        fr.append(P(bob=base['bob'] + 0.5 + 0.5 * b, neck_a=base['neck_a'] - 2 * math.sin(t - 0.8), jaw=3 + 3 * (0.5 + 0.5 * b), tail=t))
    A['idle'], T['idle'] = fr, [160] * 8
    fr = []
    for i in range(8):  # a trot: the near front and far hind paws step together, then the other pair
        ph = i / 8; s = math.sin(2 * math.pi * ph)
        g1, g2 = gait(ph, 3.4, 2.4), gait(ph + 0.5, 3.4, 2.4)
        pw = {n: (rest.get(n, (0, 0))[0] + g[0], g[1]) for n, g in (('nf', g1), ('fh', g1), ('ff', g2), ('nh', g2))}
        fr.append(P(paws=pw, bob=base['bob'] + 0.6 - 1.2 * abs(s), neck_a=base['neck_a'] + 3 * abs(s), jaw=8, tail=ph * 6.28))
    A['walk'], T['walk'] = fr, [90] * 8
    hit = [(46, 31, 1.4, WHITE), (44, 28, 0.7, P_RED), (45, 35, 0.7, P_RED), (47, 27, 0.5, WHITE)]
    A['attack'], T['attack'] = [
        P(dx=-1, bob=base['bob'] + 2.0, pitch=base['pitch'] + 6, neck_a=12, head_a=18, jaw=4,
          paws={'nh': (3.0, 0), 'fh': (3.5, 0), 'nf': (-1.5, 0), 'ff': (-1.0, 0)}),                        # coil on the haunches
        strike(P(dx=3, bob=base['bob'] - 2.0, pitch=-10, neck_a=-12, head_a=0, jaw=42,
                 paws={'nf': (5.0, 5.0), 'ff': (4.0, 6.0), 'nh': (-4.0, 0.5), 'fh': (-5.0, 1.0)})),         # spring, jaws wide
        impact(P(dx=7, bob=base['bob'] - 0.5, pitch=2, neck_a=-4, head_a=8, jaw=4, fx=hit,
                 paws={'nf': (6.0, 1.0), 'ff': (5.0, 0.5), 'nh': (-3.0, 0), 'fh': (-4.0, 0)})),             # the bite
        P(dx=6.5, bob=base['bob'], pitch=4, neck_a=-2, head_a=16, jaw=2, fx=hit[1:2],
          paws={'nf': (5.0, 0), 'ff': (4.5, 0), 'nh': (-2.0, 0), 'fh': (-3.0, 0)}),                         # hold, worrying
        P(dx=3, jaw=12, paws={'nf': (2.5, 1.5), 'ff': (2.0, 0), 'nh': (-1.0, 0), 'fh': (-1.0, 0)}),
        P(jaw=4)], [120, 150, 50, 150, 110, 110]
    A['hurt'], T['hurt'] = [
        P(dx=-1, bob=base['bob'] - 0.5, pitch=-4, neck_a=-45, head_a=-5, jaw=30, hurt=True, flash=True),
        P(dx=-2.5, bob=base['bob'] - 1.0, pitch=-6, neck_a=-50, head_a=-10, jaw=34, hurt=True, paws={'nf': (-1.0, 2.0), 'ff': (-0.5, 1.0)}),
        P(dx=-1.5, pitch=base['pitch'] - 2, neck_a=base['neck_a'] - 8, jaw=10),
        P(jaw=4)], [60, 120, 90, 90]
    lie = q['sh_h'] - q['chest'] * 1.7  # how far the body drops to lie on its side
    sprawl = {'nf': (7.0, 1.5), 'ff': (8.0, 3.5), 'nh': (-6.5, 1.0), 'fh': (-7.0, 3.0)}
    A['death'], T['death'] = [
        P(dx=-1, bob=base['bob'] - 0.5, pitch=-4, neck_a=-45, head_a=-5, jaw=30, hurt=True, flash=True),
        P(dx=-2, bob=base['bob'] - 1.5, pitch=-12, neck_a=-60, head_a=-20, jaw=40, hurt=True, paws={'nf': (-1.0, 3.0), 'ff': (0.0, 4.0)}),
        P(dx=-1.5, bob=lie * 0.45, pitch=4, neck_a=20, head_a=25, jaw=20, hurt=True,
          paws={'nf': (2.0, 0), 'ff': (2.5, 0.5), 'nh': (-2.0, 0), 'fh': (-1.5, 0.5)}),
        P(dx=-0.5, bob=lie * 0.8, pitch=2, neck_a=25, head_a=30, jaw=16, hurt=True, tail_a=-5,
          paws={n: (v[0] * 0.7, v[1] * 0.7) for n, v in sprawl.items()}),
        P(dx=0.5, bob=lie, pitch=0, neck_a=18, head_a=22, jaw=14, hurt=True, tail_a=0, paws=sprawl),
        P(dx=0.5, bob=lie, pitch=0, neck_a=18, head_a=24, jaw=12, hurt=True, tail_a=0, paws=sprawl)], [60, 140, 110, 100, 100, 600]
    return A, T


# A crawler's proportions (px) and options. head_at: the raised head's rest spot; run: px of body along the ground
# behind the neck; coil: how far the tail end curls up and forward (0 = straight); belly: the segment range drawn as
# a swollen translucent abdomen (None: all chitin); spit: also a `spit` clip; ride: px the body's centre line rides
# above the ground on its legs.
CRAWL = dict(n=13, gap=2.6, r=2.2, head=2.6, belly=None, belly_r=3.0, leg=(2.6, 2.6), antenna=6.0, spit=False,
             head_at=(33.0, 25.0), run=18.0, coil=0.0, ride=4.6)


def chain_along(path, n, gap):
    """`n` points `gap` px apart along the polyline `path` (head first), measured along the line; past its end the
    last piece runs on straight. A segmented body follows its head this way."""
    lens = [math.hypot(b[0] - a[0], b[1] - a[1]) for a, b in zip(path, path[1:])]
    out = []
    for k in range(n):
        d = k * gap
        for i, L in enumerate(lens):
            if d <= L or i == len(lens) - 1:
                a, b = path[i], path[i + 1]; t = d / L if L else 0.0
                out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)); break
            d -= L
    return out


def crawl_path(head, ground_x, run, lift=0.0, wave=0.0, amp=0.0, coil=0.0, floor=None):
    """A crawler's path, head first: from `head` an S-curve down to the ground at `ground_x` (head raised; `lift`
    bows the neck forward), then `run` px back along the ground (a travelling hump `amp` px high at phase `wave`),
    then the tail end curling up and forward by `coil` (0 = straight on)."""
    gy = floor if floor is not None else GY - 2.4
    p0, p3 = head, (ground_x, gy)
    p1 = (head[0] - 4.0, head[1] + 0.5)  # the head looks forward off the top of the curve
    p2 = (ground_x + 3.0 + lift, gy - 1.0)
    pts = []
    for i in range(10):
        t = i / 9; u = 1 - t
        pts.append((u ** 3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t ** 3 * p3[0],
                    u ** 3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t ** 3 * p3[1]))
    for i in range(1, 10):
        x = ground_x - run * i / 9
        pts.append((x, gy - amp * max(0.0, math.sin(2 * math.pi * (i / 9 * 1.5 + wave)))))
    if coil:  # round a circle standing on the ground behind the run: back, up and over towards the head
        cx, cy, r = pts[-1][0], gy - 3.2, 3.2
        for i in range(1, 7):
            a = math.pi / 2 + coil * math.pi * 1.2 * i / 6
            pts.append((cx - math.cos(a) * r, cy + math.sin(a) * r))
    pts.append((2 * pts[-1][0] - pts[-2][0], 2 * pts[-1][1] - pts[-2][1]))
    return pts


def crawler_pose(c, path, phase=0.0, jaw=0.2, bubbles=0.0, sway=0.0, dead=False, hurt=False, flash=False, fx=None,
                 dim=False):
    """One frame of crawler `c` laid along `path` (head first). phase: the leg cycle (legs step in a wave from the
    head back, the far side half a cycle off); jaw: radians the fangs open; bubbles: the abdomen's bubble cycle;
    sway: the antennae's; dead: legs curled up; dim: the abdomen's glow out."""
    P = c['pal']; f = F(); n = c['n']
    pts = chain_along(path, n, c['gap'])
    dirs = []
    for i in range(n):
        a, b = (pts[0], pts[1]) if i == 0 else (pts[i - 1], pts[i])
        L = math.hypot(a[0] - b[0], a[1] - b[1]) or 1.0
        dirs.append(((a[0] - b[0]) / L, (a[1] - b[1]) / L))
    bel = c['belly']

    def radius(i):
        if i == 0: return c['head']
        if bel and bel[0] <= i <= bel[1]:
            return c['r'] + (c['belly_r'] - c['r']) * math.sin(math.pi * (i - bel[0] + 1) / (bel[1] - bel[0] + 2))
        if i >= n - 3: return c['r'] - 0.35 * (i - (n - 4))
        return c['r']

    def legs(far):  # one-pixel legs drawn over the outlined body, so a row of them reads as legs, not a band
        col = P['leg_lo'] if far else P['leg']
        for i in range(1, n - 1):
            p, d = pts[i], dirs[i]; up = (d[1], -d[0]); r = radius(i)
            root = _v(p, (up, -(r - 0.6)))
            ph = phase + i * 0.29 + (0.5 if far else 0.0)
            if dead:
                tgt = _v(root, (up, 3.6 + 0.6 * math.sin(i * 1.7)), (d, 1.2 if far else -1.2))
            elif p[1] > GY - c['ride'] - 1.5:  # on the ground: the foot steps
                g = gait(ph, 1.4, 1.2)
                tgt = (root[0] + g[0] + (0.8 if far else -0.4), GY - 0.5 - g[1])
            else:  # raised: the legs paddle in the air
                tgt = _v(root, (up, -3.4), (d, 0.4 + 0.9 * math.sin(2 * math.pi * ph)))
            knee, foot = ik(root, c['leg'][0], c['leg'][1], tgt, 1)
            _line(f, root, knee, col, far); _line(f, knee, foot, col, far)
        p, d = pts[-1], dirs[-1]; up = (d[1], -d[0]); s = 0.5 * math.sin(2 * math.pi * phase)  # the long trailing legs
        _line(f, p, _v(p, (d, -4.6), (up, 1.2 + s + (0.8 if far else 0))), col, far)

    for i in range(n - 1, -1, -1):
        p, d = pts[i], dirs[i]; up = (d[1], -d[0]); r = radius(i)
        acid = bel and bel[0] <= i <= bel[1]
        if acid:
            f.disc(*p, r, P['acid_lo'])
            f.disc(*_v(p, (up, 0.3)), r - 0.8, P['acid_lo'] if dim else P['acid'])
            if not dim:
                b = (bubbles + i * 0.37) % 1.0  # a bubble rising through the acid
                f.dot(*_v(p, (up, -r * 0.5 + r * 1.0 * b), (d, 0.9 * math.sin(i * 2.1))), P['glow'])
                f.dot(*_v(p, (up, 0.4), (d, -0.6)), P['glow'])
            f.disc(*_v(p, (up, r - 0.8)), 1.1, P['chitin'])  # the dorsal plate over the acid
        else:
            f.disc(*p, r, P['chitin_lo'])
            f.disc(*_v(p, (up, 0.5)), r - 0.6, P['chitin'])
            f.dot(*_v(p, (up, r - 0.9)), P['chitin_hi'])
        if i:  # the plate's back edge over the segment behind it
            for k in range(-int(r), int(r) + 1):
                f.dot(*_v(p, (d, -(r - 0.4)), (up, k * 0.9)), P['chitin_lo'])
    # head: fangs, antennae, eye
    p, d = pts[0], dirs[0]; up = (d[1], -d[0])
    base = _v(p, (d, 1.4), (up, -0.9))
    for s in (-1, 1):
        ang = math.atan2(d[1], d[0]) + s * jaw + 0.35
        tip = (base[0] + math.cos(ang) * 3.0, base[1] + math.sin(ang) * 3.0)
        mid = _v(base, ((tip[0] - base[0], tip[1] - base[1]), 0.55), (up, -0.5 * s))
        f.limb(base, mid, 0.55, P['fang']); f.limb(mid, tip, 0.45, P['fang'])
        f.dot(*tip, P['glow'])
    for k in (0, 1):
        a0 = math.atan2(d[1], d[0]) - 0.75 - k * 0.35; q = _v(p, (up, 1.2), (d, 0.8))  # swept forward over the fangs
        seg = c['antenna'] / 3
        for j in range(3):
            a0 += 0.6 if dead else 0.2 + 0.15 * math.sin(sway + j + k)
            nq = (q[0] + math.cos(a0) * seg, q[1] + math.sin(a0) * seg)
            _line(f, q, nq, P['leg']); q = nq
    f.dot(*_v(p, (d, 0.9), (up, 0.7)), OUT if hurt else P['glow'])
    f.outline()
    legs(True); legs(False)
    for (x, y, rr, col) in (fx or []): f.disc(x, y, rr, col)
    if flash: _flash(f)
    return f.im


def crawler_clips(c):
    """(frames, ms) of crawler `c`: idle (head raised, swaying), walk, attack (rear and strike), hurt, death; `spit`
    (rear and spit acid) when c['spit']."""
    A, T = {}, {}
    hx, hy = c['head_at']; gx = hx - 6.0; run = c['run']; coil = c['coil']; n_len = c['n'] * c['gap']

    def path(dhx=0.0, dhy=0.0, dg=0.0, **kw):
        kw.setdefault('coil', coil); kw.setdefault('floor', GY - c['ride'])
        return crawl_path((hx + dhx, hy + dhy), gx + dg, run, **kw)

    def pose(pth, **kw):
        return crawler_pose(c, pth, **kw)

    fr = []
    for i in range(8):  # the raised head sways, the abdomen bubbles
        t = i / 8 * 2 * math.pi
        fr.append(pose(path(0.5 * math.sin(t), 0.7 * math.sin(t + 0.6)), phase=0.04 * math.sin(t), jaw=0.18 + 0.08 * math.sin(t),
                       bubbles=i / 8, sway=t))
    A['idle'], T['idle'] = fr, [160] * 8
    fr = []
    for i in range(8):  # a ripple runs down the legs; a hump travels back along the body
        ph = i / 8
        fr.append(pose(path(0.4 * math.sin(4 * math.pi * ph), 0.6 * math.sin(4 * math.pi * ph), wave=-ph, amp=0.8),
                       phase=ph, jaw=0.2, bubbles=ph, sway=ph * 6.28))
    A['walk'], T['walk'] = fr, [90] * 8
    acid = c['pal'].get('acid', P_RED); glow = c['pal'].get('glow', WHITE)
    hx2 = min(46.0, hx + 10.5)  # where the shut fangs land
    hit = [(hx2, hy + 8.5, 1.4, WHITE), (hx2 - 1.0, hy + 6.0, 0.7, glow), (hx2 - 0.5, hy + 11.0, 0.7, glow)]
    A['attack'], T['attack'] = [
        pose(path(-3.0, -5.0, dg=-1.0, lift=-2.0), phase=0.1, jaw=0.7, bubbles=0.1),            # rear back
        pose(path(-4.0, -6.0, dg=-1.5, lift=-3.0), phase=0.15, jaw=0.9, bubbles=0.2),           # poised
        strike(pose(path(5.5, 7.0, dg=4.0, lift=4.0), phase=0.3, jaw=0.9, bubbles=0.3)),        # the strike
        impact(pose(path(6.0, 8.0, dg=4.5, lift=4.0), phase=0.35, jaw=0.05, bubbles=0.4, fx=hit)),  # fangs shut
        pose(path(4.0, 3.0, dg=2.0, lift=1.5), phase=0.2, jaw=0.3, bubbles=0.5),
        pose(path(), phase=0.0, jaw=0.2, bubbles=0.6)], [130, 150, 50, 150, 110, 110]
    A['hurt'], T['hurt'] = [
        pose(path(-2.0, -2.0, dg=-1.0), jaw=0.8, hurt=True, flash=True, bubbles=0.2),
        pose(path(-3.5, -3.0, dg=-2.0, lift=-2.0), phase=0.25, jaw=0.9, hurt=True, bubbles=0.3),
        pose(path(-1.5, -1.0, dg=-1.0), phase=0.1, jaw=0.4, bubbles=0.4),
        pose(path(), jaw=0.2, bubbles=0.5)], [60, 120, 90, 90]
    flat = (min(38.0, max(gx + 6.0, n_len + 3.0)), GY - 2.6)  # dead and curled up, kept in the frame
    A['death'], T['death'] = [
        pose(path(-2.0, -2.0, dg=-1.0), jaw=0.8, hurt=True, flash=True),
        pose(path(-3.0, -6.0, dg=-1.5, lift=-3.0), phase=0.4, jaw=1.0, hurt=True, bubbles=0.3),   # thrashes up
        pose(path(2.0, 4.0, dg=1.0, lift=2.0), phase=0.7, jaw=0.6, hurt=True, bubbles=0.5),
        pose(crawl_path((flat[0], GY - 6.0), flat[0] - 4, run + 2, coil=max(coil, 0.6) * 0.8), jaw=0.4, hurt=True, dead=True, bubbles=0.7),
        pose(crawl_path(flat, flat[0] - 3, run + 2, coil=max(coil, 0.6)), jaw=0.3, hurt=True, dead=True, dim=True),
        pose(crawl_path(flat, flat[0] - 3, run + 2, coil=max(coil, 0.6)), jaw=0.3, hurt=True, dead=True, dim=True)], [60, 140, 110, 100, 100, 600]
    if c['spit']:
        sx = lambda x: min(46.0, x)
        A['spit'], T['spit'] = [
            pose(path(-2.0, -4.0, dg=-1.0, lift=-2.0), phase=0.1, jaw=0.4, bubbles=0.1),
            pose(path(-3.0, -5.0, dg=-1.0, lift=-2.5), phase=0.1, jaw=1.0, bubbles=0.3, fx=[(hx - 0.5, hy - 4.5, 1.0, acid)]),  # acid wells up
            strike(impact(pose(path(1.5, 0.0, lift=1.0), phase=0.15, jaw=1.1, bubbles=0.5,             # the acid leaves
                               fx=[(sx(hx + 6.0), hy - 1.0, 1.6, acid), (sx(hx + 6.4), hy - 1.4, 0.6, glow), (sx(hx + 3.5), hy, 0.6, acid)]))),
            pose(path(1.0, 0.5, lift=0.5), phase=0.2, jaw=0.6, bubbles=0.7,
                 fx=[(sx(hx + 11.5), hy + 0.5, 1.7, acid), (sx(hx + 12.0), hy, 0.6, glow), (sx(hx + 8.0), hy + 1.0, 0.7, acid)]),
            pose(path(0.5, 0.0), phase=0.1, jaw=0.4, bubbles=0.9,
                 fx=[(sx(hx + 13.5), hy + 2.5, 1.2, acid), (sx(hx + 12.0), hy + 4.0, 0.7, glow), (sx(hx + 14.1), hy + 0.5, 0.6, acid)]),
            pose(path(), jaw=0.2, bubbles=0.0)], [120, 160, 60, 70, 120, 110]
    return A, T
