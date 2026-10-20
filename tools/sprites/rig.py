# Shared pixel-art rig: one skeleton, swappable outfit (head/torso/legs), weapon family and palette.
# Frames are W x W, facing right. Units are dicts: proportions, part names and a palette.
import math
from PIL import Image

W = 48
GY = 43.0
OUT = (20, 17, 15)
WHITE = (250, 244, 230)


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
    if h == 'goblin':
        for k in range(6): f.disc(hx - 2.5 - k * 0.9, hy - 0.6 - k * 0.45, 1.3 - k * 0.15, P['skin'] if k < 5 else P['skin_lo'])
        f.disc(hx, hy, 3.4, P['skin']); f.rect(hx - 3, hy + 1.5, hx + 2, hy + 3, P['skin_lo'])
        f.rect(hx + 2, hy + 0.5, hx + 4.2, hy + 1.6, P['skin']); f.dot(hx + 4.2, hy + 1.6, P['skin_lo'])
        f.dot(hx + 1.6, hy - 0.6, OUT if hurt else P['eye']); f.dot(hx + 2.3, hy - 0.6, OUT)
        f.dot(hx + 2.2, hy + 2.4, WHITE); f.dot(hx + 0.8, hy + 2.6, WHITE)
        f.rect(hx - 2.5, hy - 3.8, hx + 1.2, hy - 2.8, P['accent']); f.dot(hx - 3.6, hy - 2.2, P['accent']); f.dot(hx - 4.2, hy - 1.6, P['accent_lo'])
        return
    # a face every human head shares
    f.disc(hx, hy, 3.4, P['skin'])
    f.rect(hx - 3, hy + 1.2, hx - 1.5, hy + 3, P['skin_lo'])
    f.dot(hx + 2, hy + 0.2, OUT if hurt else P['eye']); f.dot(hx + 3.2, hy + 1.6, P['skin_lo'])
    if h == 'helm':      # warrior: full helm, visor slit, plume
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
         bow_a=0.0, pull=0.0, nock=False, arrow=None, glow=0.0, fist_f=False, fist_b=False, tails=0.0, swap=False, leg_trail=None):
    """hand_f/hand_b: hand targets relative to the shoulder (None = rest pose). wa/wb: weapon angles
    (0 up, 90 forward, 180 down). ff/fb: foot offsets (x from pelvis, lift). loose: (x, y, angle) of a dropped weapon."""
    P = u['pal']; L, A, T = u['leg'], u['arm'], u['tl']
    f = F()
    px, py = 20 + dx + fall * 10, GY - 2 * L + bob
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
    hf = (sh[0] + 4, sh[1] + 6.5) if hand_f is None else (sh[0] + hand_f[0], sh[1] + hand_f[1] + bob * 0.0)
    hb = (sh[0] - 2.5, sh[1] + 8.5 - hurt * 2) if hand_b is None else (sh[0] + hand_b[0], sh[1] + hand_b[1])
    if swap:  # main weapon in the far hand (behind the body), off-hand item (shield) in the near hand
        eb, hb = ik((sh[0] - 2, sh[1] - 0.5), A, A, hf, 1)
        ef, hf = ik((sh[0] + 2, sh[1] - 0.5), A, A, (sh[0] + 4.5, sh[1] + 3) if hand_b is None else (sh[0] + hand_b[0] + 2, sh[1] + hand_b[1]), -1)
    else:
        ef, hf = ik((sh[0] + 2, sh[1] - 0.5), A, A, hf, u.get('fbend', -1))
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
            draw_weapon(f, u, P, wf, (hfx, hfy), wa, two, glow)
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
    return f.im


# ------------------------------------------------------------------ shared clips (any unit)
def idle(u, frames=8, hf_amp=0.8, alt=0.0, **kw):
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
        out.append(pose(u, bob=k.pop('bob', 0.0) + b * 0.9, head_dy=lag * 0.8 - 0.2, lean=k.pop('lean', 0.0) + math.sin(t) * 0.4,
                        hand_f=hf, hand_b=hb, fb=(-3.5 - b * 0.4, 0), ff=(3.5 + b * 0.4, 0), tails=t, **k))
    return out, [160] * frames


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
