# DIR4 front and back views on the rig (games/emberwake/docs/specs/DIR4.md section 6): the same skeleton and unit dict
# as rig.pose, projected for the two views a unit shows when it faces the camera or away from it:
#   down: the front (face to the camera); the unit's right hand (the main weapon) is screen-left.
#   up:   the back (face away);           the unit's right hand is screen-right.
# Never mirrored (DIR4 section 1). rig.py's side code is not used to draw here and is not changed by this module.
#
# Rules (DIR4 6.1):
#   R1 pose_view(u, view, ...) reads the keys rig.pose reads (leg, arm, tl, neck, scale, girth, pal, and the part names
#      head, hat, torso, legs, back, weapon, offhand, mask, beard, braid, pauldron).
#   R2 48 x 48 frame, feet on rig.GY, pelvis centred at x = CX, every offset scaled by `scale` about the feet; the
#      figure (weapons excluded) stays within x 12..36.
#   R3 a view clip has its side clip's frame count and ms (idle and walk here use rig.idle's and rig.walk's).
#   R4 drawn only with the unit's `pal` colours plus rig.OUT and rig.WHITE (export_units checks the sheet).
#   R5 supports(u, cls): a body gets view clips only when every part it uses has a view drawer (PARTS) and its grip
#      has view keys (GRIPS); missing(u, cls) names what is missing.
#   R6 outline and white hit flash as the side rig (rig.F.outline).
#
# The core (DIR4-R1 slice 1) draws the skeleton: legs, arms, hands and neck, plus the clip shapes idle and walk.
# The warrior's parts and grips (DIR4-R1 slice 2, the pilot) are at the end: helm, plate, trousers, sword, greatsword,
# kite shield; GRIPS 'warrior' (1H + kite shield) and 'warrior/2h' (WAR2H1's 2H grip; the greatsword only so far).
# Outfit parts plug in through PARTS[kind][name] = drawer(f, c, layer): the drawer is called once per depth layer
# (DIR4 6.2, LAYERS) with the frame `f` (rig.F) and the pose context `c` (Ctx) and draws what belongs on that layer.
# Grips plug in through GRIPS['<class>' or '<class>/<grip>'] = dict(idle=kw, walk=kw, attack=(side clip id, fn)) with
# fn(u, view, stand_in) -> (frames, ms). STAND_INS are plain drawings (a face or a hair back, a tunic) used only for the
# review boards of bodies whose parts have no view drawer yet; the exporter never writes them.
import math

from PIL import Image

import rig
from rig import F, GY, OUT

VIEWS = ('down', 'up')
CX = 24.0
X_MIN, X_MAX = 12, 36                 # R2: the figure's columns (weapons excluded)
TOKEN = 28                            # the battle's token size (Art::body_clip downsamples the 48 px frame to it)
READABLE = 0.10                       # 6.4: idle_down and idle_up frame 0 differ in at least this share at TOKEN px
REACH = 0.01                          # rig.ik clamps a leg to its length less this, so a side foot sits just above GY

# depth layers, back to front (DIR4 6.2); a drawer reads the view from c.view
BEHIND, RAISED_BACK, LEGS, SKIRT, TORSO, ARMS, HEAD, FRONT, TOP = range(1, 10)
LAYERS = (BEHIND, RAISED_BACK, LEGS, SKIRT, TORSO, ARMS, HEAD, FRONT, TOP)

# part kinds in drawing order within a layer; FLAGS are the unit's boolean extras that need a drawing of their own
KINDS = ('back', 'legs', 'torso', 'offhand', 'weapon', 'head', 'hat', 'flag')
FLAGS = ('mask', 'beard', 'braid', 'pauldron')
PARTS = {k: {} for k in KINDS}
GRIPS = {}


def parts_of(u):
    """The (kind, name) parts unit `u` uses, in KINDS order."""
    out = []
    if u.get('back'):
        out.append(('back', u['back']))
    out.append(('legs', u['legs']))
    out.append(('torso', u['torso']))
    if u.get('offhand'):
        out.append(('offhand', u['offhand']))
    if u.get('weapon'):
        out.append(('weapon', u['weapon']))
    out.append(('head', u['head']))
    if u.get('hat'):
        out.append(('hat', u['hat']))
    out += [('flag', k) for k in FLAGS if u.get(k)]
    return out


def grip_key(cls, u):
    """The GRIPS key of class `cls` worn as unit `u`: the class id, or '<class>/<grip>' for a WAR2H1 grip."""
    return cls if not u.get('grip') else '%s/%s' % (cls, u['grip'])


def missing(u, cls):
    """What keeps unit `u` (of class `cls`) from view clips (R5): 'kind name' per part without a view drawer, and
    'grip <key>' when its grip has no view keys. Empty: the body is supported."""
    out = ['%s %s' % (k, n) for k, n in parts_of(u) if n not in PARTS[k]]
    if grip_key(cls, u) not in GRIPS:
        out.append('grip %s' % grip_key(cls, u))
    return out


def supports(u, cls):
    """R5: every part of `u` has a view drawer and its grip has view keys."""
    return not missing(u, cls)


class Ctx:
    """One frame's pose: the unit, its palette, the view, the joints (screen positions, already scaled) and the clip's
    extra keys `e` (weapon angles, glow and the like, for the part drawers)."""

    def __init__(s, u, view, j, tails, hurt, e):
        s.u, s.P, s.view, s.j, s.tails, s.hurt, s.e = u, u['pal'], view, j, tails, hurt, e
        s.rs = -1.0 if view == 'down' else 1.0     # screen x sign of the unit's right side
        s.s = u.get('scale', 1.0)


def joints(u, view, bob=0.0, head_dy=0.0, lift_r=0.0, lift_l=0.0, hand_r=None, hand_l=None, dy=0.0):
    """The skeleton of one frame in `view`, scaled about the feet as rig.pose: pelvis, hips, knees, feet, shoulders,
    elbows, hands (r = the unit's right, the main hand), neck base and head centre. hand_r / hand_l: hand targets
    (outward, down) from the shoulder (None = the rest pose, hanging at the side); lift_r / lift_l: foot lifts
    (a lifted foot rises, its knee bends); dy: the body (not the feet) moved down the screen."""
    L, A, T = u['leg'], u['arm'], u['tl']
    s = u.get('scale', 1.0)
    rs = -1.0 if view == 'down' else 1.0
    bob, head_dy, dy, lift_r, lift_l = bob * s, head_dy * s, dy * s, lift_r * s, lift_l * s
    hand_r = None if hand_r is None else (hand_r[0] * s, hand_r[1] * s)
    hand_l = None if hand_l is None else (hand_l[0] * s, hand_l[1] * s)
    py = GY - 2 * L + bob + dy
    g = u.get('girth', 0.0)
    j = {'pelvis': (CX, py)}
    for side, sx, lift in (('r', rs, lift_r), ('l', -rs, lift_l)):
        hip = (CX + sx * 2.3, py)
        foot = (CX + sx * 2.6, GY - REACH - lift)  # the side's ik stops REACH short of the ground: same boot rows
        k = 0.5 + (0.08 * lift / max(s, 0.01) if view == 'down' else 0.0)  # a lifted knee comes toward the camera
        knee = (hip[0] + (foot[0] - hip[0]) * 0.5 + sx * 0.25 * lift, hip[1] + (foot[1] - hip[1]) * k)
        j['hip_' + side], j['knee_' + side], j['foot_' + side] = hip, knee, foot
    sh_y = py - T
    for side, sx, hand in (('r', rs, hand_r), ('l', -rs, hand_l)):
        sh = (CX + sx * (4.0 + g), sh_y + 0.5)
        tgt = (sh[0] + sx * 1.2 * s, sh[1] + 2 * A - 1.0 * s) if hand is None else (sh[0] + sx * hand[0], sh[1] + hand[1])
        elbow, hand_p = rig.ik(sh, A, A, tgt, -sx)                     # the elbow bends outward
        j['sh_' + side], j['elbow_' + side], j['hand_' + side] = sh, elbow, hand_p
    j['neck'] = (CX, sh_y)
    j['head'] = (CX, sh_y - u.get('neck', 6) + head_dy)
    return j


def _call(f, c, layer, stand_in):
    for kind, name in parts_of(c.u):
        d = PARTS[kind].get(name) or (STAND_INS.get(kind) if stand_in else None)
        if d:
            d(f, c, layer)


def _legs(f, c):
    P, j = c.P, c.j
    for side in ('r', 'l'):
        hip, knee, foot = j['hip_' + side], j['knee_' + side], j['foot_' + side]
        f.limb(hip, knee, 2.2, P['leg']); f.limb(knee, foot, 2.0, P['leg'])
        inner = -1.0 if hip[0] > CX else 1.0                            # shade the inner edge
        f.limb((hip[0] + inner * 1.6, hip[1] + 1.0), (foot[0] + inner * 1.4, foot[1] - 2.0), 0.3, P['leg_lo'])
        if knee[1] - hip[1] < foot[1] - knee[1] - 0.2:                  # a lifted, bent knee
            f.dot(knee[0], knee[1], P['leg_lo'])
        f.rect(foot[0] - 1.5, foot[1] - 1, foot[0] + 1.5, foot[1], P['boot'])
        f.rect(foot[0] - 1.5, foot[1], foot[0] + 1.5, foot[1], P['boot_lo'])


def arm_layer(c, side, fore=False):
    """The depth layer of an arm (side 'r' or 'l'): ARMS, or the clip's arm_z[side]; fore=True: its forearm and hand,
    fore_z[side] when given (a forearm before the chest in `up`, hidden by the body), else the arm's."""
    upper = c.e.get('arm_z', {}).get(side, ARMS)
    return c.e.get('fore_z', {}).get(side, upper) if fore else upper


def _arms(f, c, layer):
    P, j = c.P, c.j
    sleeve, fore = P.get('sleeve', P['main']), P.get('fore', P['skin'])
    for side in ('r', 'l'):
        sh, el, hd = j['sh_' + side], j['elbow_' + side], j['hand_' + side]
        if arm_layer(c, side) == layer:
            f.limb(sh, el, 1.8, sleeve)
        if arm_layer(c, side, True) == layer:
            f.limb(el, hd, 1.6, fore)
            f.disc(hd[0], hd[1], 1.5, P.get('glove', P['leather']))


def _neck(f, c):
    hx, hy = c.j['head']; nx, ny = c.j['neck']
    f.rect(hx - 1, hy + 2, hx + 1, ny, c.P['skin_lo'])


def pose_view(u, view, bob=0.0, head_dy=0.0, lift_r=0.0, lift_l=0.0, hand_r=None, hand_l=None, dy=0.0, hurt=0.0,
              flash=False, fx=None, tails=0.0, stand_in=False, **e):
    """One frame of unit `u` in `view` ('down' or 'up'), a W x W RGBA image (R1, R2, R6). The keys are joints'; fx:
    (x, y, r, colour) effect discs at side-frame positions scaled about the feet; e: extra keys for the part drawers
    (Ctx.e). stand_in: draw STAND_INS for parts without a view drawer (review boards only)."""
    if view not in VIEWS:
        raise ValueError("view must be down or up, not %r" % (view,))
    s = u.get('scale', 1.0)
    j = joints(u, view, bob, head_dy, lift_r, lift_l, hand_r, hand_l, dy)
    if e.get('grip2'):  # a two-handed grip: the left hand holds the grip grip2 px below the right, along the weapon
        dx_, dy_ = rig.udir(e.get('wa', 0.0)); g = e['grip2'] * e.get('wk', 1.0) * s; hr = j['hand_r']
        j['elbow_l'], j['hand_l'] = rig.ik(j['sh_l'], u['arm'], u['arm'], (hr[0] - dx_ * g, hr[1] - dy_ * g),
                                           -1.0 if view == 'down' else 1.0)  # the elbow bends outward
    c = Ctx(u, view, j, tails, hurt, e)
    f = F()
    for layer in LAYERS:
        if layer == LEGS:
            _legs(f, c)
        elif layer == HEAD:
            _neck(f, c)
        _arms(f, c, layer)
        _call(f, c, layer, stand_in)
        if layer == TOP and e.get('slash'):  # a swing's arc (frame positions scaled about the feet), as rig.pose's
            sl = e['slash']
            for i, (x, y) in enumerate(sl):
                f.disc(CX + (x - CX) * s, GY - (GY - y) * s, 0.5 + 1.0 * i / len(sl),
                       c.P.get('slash', c.P['metal_hi']) if i < len(sl) * 0.6 else rig.WHITE)
    f.outline()
    for x, y, r, col in fx or []:
        f.disc(CX + (x - CX) * s, GY - (GY - y) * s, r, col)
    if flash:
        rig._flash(f)
    return f.im


# ------------------------------------------------------------------ clip shapes (DIR4 6.3)
def idle(u, view, frames=8, hf_amp=0.8, stand_in=False, **kw):
    """idle_<view>: rig.idle's bob and breath (8 frames, 160 ms), hands from the grip's ready pose."""
    out = []
    for i in range(frames):
        t = i / frames * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t - 0.6); lag = 0.5 + 0.5 * math.sin(t - 1.4)
        k = dict(kw); hr = k.pop('hand_r', None); hl = k.pop('hand_l', None)
        hr = None if hr is None else (hr[0], hr[1] + lag * hf_amp)
        hl = None if hl is None else (hl[0], hl[1] + lag * 0.6)
        if hr is None and hl is None:  # rest pose: the hands breathe with the body
            hr = hl = (1.2, 2 * u['arm'] / u.get('scale', 1.0) - 1.0 + lag * 0.4)
        out.append(pose_view(u, view, bob=k.pop('bob', 0.0) + b * 0.9, head_dy=lag * 0.8 - 0.2, hand_r=hr, hand_l=hl,
                             tails=t, stand_in=stand_in, **k))
    return out, [160] * frames


def walk(u, view, frames=8, stride=6.0, lift=0.45, stand_in=False, **kw):
    """walk_<view>: rig.walk's gait (8 frames, 90 ms): the feet alternate, each lifting and bending its knee in its
    half of the gait; the body bobs 0.9 - 1.8 |sin|; the arms swing as 1 px vertical offsets opposite to the legs."""
    out = []
    rest = 2 * u['arm'] / u.get('scale', 1.0) - 1.0
    for i in range(frames):
        ph = i / frames; s = math.sin(2 * math.pi * ph)
        bob = 0.9 - 1.8 * abs(s)
        _, lr = rig.gait(ph, stride); _, ll = rig.gait(ph + 0.5, stride)
        k = dict(kw); hr = k.pop('hand_r', (1.2, rest)); hl = k.pop('hand_l', (1.2, rest)); swing = k.pop('swing', 1.0)
        hr = (hr[0], hr[1] + s * swing)                                 # the right arm swings with the left leg
        hl = (hl[0], hl[1] - s * swing)
        out.append(pose_view(u, view, bob=k.pop('bob', 0.0) + bob, head_dy=bob * 0.25, lift_r=lr * lift, lift_l=ll * lift,
                             hand_r=hr, hand_l=hl, tails=ph * 6, stand_in=stand_in, **k))
    return out, [90] * frames


def clips(cls, u, stand_in=False):
    """The view clips of class `cls` worn as unit `u`: {id: frames}, {id: ms} with ids '<side clip>_<view>' (idle,
    walk and, when the grip has one, its attack). stand_in: a body without view support is drawn with STAND_INS and the
    rest pose (review boards only)."""
    g = GRIPS.get(grip_key(cls, u))
    if g is None:
        if not stand_in:
            raise ValueError("no view keys for grip %s" % grip_key(cls, u))
        g = {}
    A, T = {}, {}
    for view in VIEWS:
        A['idle_' + view], T['idle_' + view] = idle(u, view, stand_in=stand_in, **grip_kw(g, 'idle', view))
    for view in VIEWS:
        A['walk_' + view], T['walk_' + view] = walk(u, view, stand_in=stand_in, **grip_kw(g, 'walk', view))
    if 'attack' in g:
        name, fn = g['attack']
        for view in VIEWS:
            A['%s_%s' % (name, view)], T['%s_%s' % (name, view)] = fn(u, view, stand_in)
    return A, T


def grip_kw(g, clip, view):
    """A grip's keys for `clip` ('idle' or 'walk') in `view`: g[clip][view] when the grip keys each view apart, else
    g[clip] for both."""
    kw = g.get(clip, {})
    return kw[view] if view in kw else kw


def of(view_id):
    """(side clip id, view) of a view clip id '<of>_<view>'."""
    base, _, view = view_id.rpartition('_')
    return base, view


# ------------------------------------------------------------------ readability (DIR4 6.4, section 8)
def downsample(im, w, h):
    """aegis_ui Image::downsample: a box filter with alpha-weighted colour, integer maths as the engine's."""
    src = im.convert("RGBA"); sw, sh = src.size; sp = src.load()
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0)); op = out.load()
    for y in range(h):
        y0 = y * sh // h; y1 = max((y + 1) * sh // h, y0 + 1)
        for x in range(w):
            x0 = x * sw // w; x1 = max((x + 1) * sw // w, x0 + 1)
            r = g = b = a = n = 0
            for yy in range(y0, min(y1, sh)):
                for xx in range(x0, min(x1, sw)):
                    pr, pg, pb, pa = sp[xx, yy]
                    r += pr * pa; g += pg * pa; b += pb * pa; a += pa; n += 1
            op[x, y] = (r // a, g // a, b // a, a // max(n, 1)) if a else (0, 0, 0, 0)
    return out


def token(im, size=TOKEN):
    """A frame at the battle's token size, as Art::body_clip fits a 48 px frame into a smaller square."""
    return downsample(im, size, size)


def difference(a, b, size=TOKEN, tol=16):
    """The share of the opaque pixels (in either) of frames `a` and `b` at `size` px that differ: opaque in one only,
    or a channel apart by more than `tol`."""
    ta, tb = token(a, size).load(), token(b, size).load()
    opaque = differ = 0
    for y in range(size):
        for x in range(size):
            pa, pb = ta[x, y], tb[x, y]
            if not pa[3] and not pb[3]:
                continue
            opaque += 1
            if bool(pa[3]) != bool(pb[3]) or max(abs(pa[i] - pb[i]) for i in range(3)) > tol:
                differ += 1
    return differ / opaque if opaque else 0.0


# ------------------------------------------------------------------ stand-ins (review boards only)
def plain_head(f, c, layer):
    """A bare head: face, two eyes 2 px apart and a hair line (down); hair over the skull and the nape (up)."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    f.disc(hx, hy, 3.4, P['skin'])
    if c.view == 'down':
        f.rect(hx - 3, hy - 3.6, hx + 3, hy - 2.2, P['hair'])
        f.rect(hx - 3.5, hy - 2.2, hx - 3.0, hy + 0.5, P['hair']); f.rect(hx + 3.0, hy - 2.2, hx + 3.5, hy + 0.5, P['hair'])
        f.dot(hx - 1, hy + 0.2, OUT if c.hurt else P['eye']); f.dot(hx + 1, hy + 0.2, OUT if c.hurt else P['eye'])
        f.rect(hx - 0.5, hy + 2.0, hx + 0.5, hy + 2.0, P['skin_lo'])
    else:
        f.disc(hx, hy - 0.4, 3.4, P['hair'])
        f.rect(hx - 3, hy, hx + 3, hy + 2.4, P['hair'])
        f.dot(hx - 3.6, hy + 0.6, P['skin_lo']); f.dot(hx + 3.6, hy + 0.6, P['skin_lo'])  # ears


def plain_torso(f, c, layer):
    """A tunic: chest in `main` over a waist in `leg`, a belt; the buckle and an open collar (down), a back seam (up)."""
    if layer != TORSO:
        return
    P, u = c.P, c.u
    px, py = c.j['pelvis']; T = u['tl']; g = u.get('girth', 0.0)
    for i in range(12):
        t = i / 11.0; y = py - T * t
        half = 3.8 + 1.0 * t + g * (0.6 + 0.4 * t)
        f.rect(px - half, y - 0.5, px + half, y + 0.5, P['main'] if t > 0.25 else P['leg'])
    f.rect(px - 4.0, py - 1.2, px + 4.0, py - 0.2, P['leather'])
    if c.view == 'down':
        f.dot(px, py - 0.7, P['trim'])
        f.poly([(px - 1.6, py - T - 0.4), (px + 1.6, py - T - 0.4), (px, py - T + 2.6)], P['skin'])  # open collar
    else:
        f.rect(px, py - T * 0.9, px, py - T * 0.3, P['main_lo'])


STAND_INS = {'head': plain_head, 'torso': plain_torso}


# ------------------------------------------------------------------ the warrior's parts (DIR4-R1 slice 2)
# Clip keys the weapon drawers read (Ctx.e): wa the main weapon's screen angle (rig.udir: 0 up, 90 screen right,
# 180 down), wk its length on screen (1 = drawn flat; less = foreshortened, pointing toward or away from the camera),
# wz its layer (default FRONT in `down`, BEHIND in `up`), oz the off-hand's layer (same defaults); arm_z / fore_z
# ({'r' or 'l': layer}) move an arm, or only its forearm and hand, off ARMS (pose_view); grip2 (pose_view) puts the
# left hand on the grip below the right; slash (pose_view) a swing's arc on TOP.
def _wz(c, key):
    return c.e.get(key, FRONT if c.view == 'down' else BEHIND)


def _along(h, d, k, n=(0.0, 0.0)):
    """A point on a weapon held at hand `h` with screen direction `d` and length factor `k`: at(a, b) is `a` px along
    the weapon (foreshortened by k) and `b` px across it (not foreshortened)."""
    return lambda a, b=0.0: (h[0] + d[0] * a * k + n[0] * b, h[1] + d[1] * a * k + n[1] * b)


def _regrip(f, c, layer, sides):
    """The hands over a weapon drawn on `layer` (as rig.pose draws the fist over the grip), for the hands drawn below."""
    for side in sides:
        if arm_layer(c, side, True) <= layer:
            h = c.j['hand_' + side]
            f.disc(h[0], h[1], 1.5, c.P.get('glove', c.P['leather']))


def helm(f, c, layer):
    """The warrior's full helm. down: the dome, an open face under a dark visor slit with two eye glints, cheek guards
    and a crest plume; up: the dome, the plume running down the back and a neck guard."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    f.disc(hx, hy - 0.2, 3.7, P['metal'])
    if c.view == 'down':
        ys = int(hy)                                                                          # the visor slit's row
        f.rect(hx - 2, ys + 1, hx + 2, hy + 3, P['skin'])                                     # the face
        f.rect(hx - 3, ys, hx - 3, hy + 2.6, P['metal_lo']); f.rect(hx + 3, ys, hx + 3, hy + 2.6, P['metal_lo'])
        f.rect(hx - 2, ys, hx + 2, ys, OUT)                                                   # visor slit
        f.dot(hx - 1, ys, OUT if c.hurt else rig.WHITE); f.dot(hx + 1, ys, OUT if c.hurt else rig.WHITE)
        f.dot(hx, hy + 2.2, P['skin_lo'])                                                     # mouth
        f.dot(hx - 1.5, hy - 2.2, P['metal_hi']); f.dot(hx - 0.5, hy - 2.8, P['metal_hi'])
        f.rect(hx - 0.5, hy - 5.0, hx + 0.5, hy - 3.6, P['accent']); f.dot(hx, hy - 5.4, P['accent_lo'])
    else:
        f.rect(hx - 3, hy + 1.4, hx + 3, hy + 3.0, P['metal_lo'])                             # neck guard
        f.dot(hx - 1.5, hy - 2.2, P['metal_hi']); f.dot(hx - 0.5, hy - 2.8, P['metal_hi'])
        f.rect(hx - 0.5, hy - 5.0, hx + 0.5, hy + 0.8, P['accent'])                           # plume down the back
        f.rect(hx - 0.5, hy - 0.4, hx + 0.5, hy + 0.8, P['accent_lo']); f.dot(hx, hy - 5.4, P['accent_lo'])


def plate(f, c, layer):
    """The warrior's plate: a breastplate over a red tabard, belt and buckle, a tabard flap below the belt (down); the
    back plate with a spine line and the belt line, no buckle (up); a shoulder plate on each shoulder (the one the
    side rig adds), on its arm's layer."""
    P, u, j = c.P, c.u, c.j
    px, py = j['pelvis']; T = u['tl']; g = u.get('girth', 0.0)
    if layer == SKIRT and c.view == 'down':
        f.poly([(px - 2.2, py - 0.5), (px + 2.2, py - 0.5), (px + 1.8, py + 3.4), (px - 1.8, py + 3.4)], P['accent'])
        f.rect(px - 1.6, py + 3.0, px + 1.6, py + 3.0, P['accent_lo'])
    if layer == TORSO:
        for i in range(12):
            t = i / 11.0; y = py - T * t
            half = 3.8 + 1.0 * t + g * (0.6 + 0.4 * t)
            f.rect(px - half, y - 0.5, px + half, y + 0.5, P['metal'] if t > 0.55 else P['accent'])
        top, low = py - T, py - T * 0.6
        f.rect(px - 4.4 - g, low, px + 4.4 + g, low, P['metal_lo'])                          # the plate's lower rim
        f.rect(px + 3.8 + g, top + 1, px + 4.6 + g, low, P['metal_lo'])                       # shade on the screen-right edge
        if c.view == 'down':
            f.dot(px - 2.5, top + 1.5, P['metal_hi']); f.dot(px - 2.0, top + 2.5, P['metal_hi']); f.dot(px - 1.5, top + 3.5, P['metal_hi'])
            f.rect(px - 0.5, low + 1, px + 0.5, py - 1.5, P['accent_lo'])                    # the tabard's centre fold
            f.rect(px - 4, py - 1.2, px + 4, py - 0.2, P['leather']); f.rect(px - 0.5, py - 1.2, px + 0.5, py - 0.2, P['trim'])
        else:
            f.rect(px, top + 1, px, low - 1, P['metal_lo'])                                   # spine
            f.dot(px - 2.5, top + 1.5, P['metal_hi']); f.dot(px + 2.0, top + 1.5, P['metal_hi'])
            f.rect(px - 4, py - 1.2, px + 4, py - 0.2, P['leather'])
            f.rect(px - 4, py - 0.2, px + 4, py - 0.2, P['leather_lo'])
    for side in ('r', 'l'):
        if arm_layer(c, side) != layer:
            continue
        sx, sy = j['sh_' + side]; out = 1.0 if sx > CX else -1.0
        f.disc(sx + out * 0.3, sy + 0.2, 2.6, P['metal'])
        f.rect(sx + out * 0.3 - 2, sy + 2.0, sx + out * 0.3 + 2, sy + 2.0, P['metal_lo'])
        f.dot(sx + out * 0.3 - 0.8, sy - 0.9, P['metal_hi'])


def trousers(f, c, layer):
    """Trousers: the inseam between the legs (leg_lo) and, from the back, a seat seam."""
    if layer != LEGS:
        return
    px, py = c.j['pelvis']
    f.rect(px, py + 1.0, px, py + 2.5, c.P['leg_lo'])
    if c.view == 'up':
        f.rect(px - 2, py - 0.2, px + 2, py - 0.2, c.P['leg_lo'])


def sword(f, c, layer):
    """The one-hand sword in the right hand: grip, pommel, crossguard and blade (rig.draw_weapon's, 12 px), along wa,
    foreshortened by wk."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 180.0)); n = (-d[1], d[0])
    at = _along(h, d, c.e.get('wk', 1.0), n)
    f.limb(at(-2.5), at(0.5), 0.8, P['leather_lo']); f.disc(*at(-3.1), 1.0, P['trim'])
    f.limb(at(0.9, -2.4), at(0.9, 2.4), 0.7, P['trim'])
    f.limb(at(2), at(12), 1.0, P['metal_hi']); f.limb(at(2), at(12), 0.4, rig.WHITE)
    _regrip(f, c, layer, ('r',))


def greatsword(f, c, layer):
    """The greatsword in both hands (rig._greatsword's: a two-hand grip, a wide guard, a 16 px blade), along wa,
    foreshortened by wk. In `up` behind the body the grip and pommel stay hidden; only the blade past the body shows
    (pilot review, user 2026-10-28: no hilt below the torso; this replaces L4's pommel below the hands)."""
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 0.0)); n = (-d[1], d[0])
    at = _along(h, d, c.e.get('wk', 1.0), n)
    if layer != _wz(c, 'wz'):
        return
    f.limb(at(-5.5), at(0.5), 0.8, P['leather_lo']); f.disc(*at(-6.1), 1.1, P['trim'])
    f.limb(at(1.0, -3.0), at(1.0, 3.0), 0.7, P['trim'])
    f.limb(at(2.0), at(16.0), 1.2, P['metal_hi']); f.limb(at(2.0), at(15.5), 0.45, rig.WHITE)
    f.dot(*at(16.8), P['metal_hi'])
    _regrip(f, c, layer, ('r', 'l'))


def kite_shield(f, c, layer):
    """The kite shield on the left arm, on its own outlined layer (as rig.draw_weapon's). down: its face (blue, the
    screen-right half shaded, the rim and the emblem cross); up: its back (shade, rim, two straps), behind the body."""
    if layer != _wz(c, 'oz'):
        return
    P = c.P; hx, hy = c.j['hand_l']
    # down: a little inward of the fist, over the chest; up: a little outward, so its rim shows past the body
    cx, cy = hx + c.rs * (1.0 if c.view == 'down' else -1.5), hy + 0.5
    face = [(-3.6, -7.5), (3.6, -7.5), (4.4, -2.0), (0.5, 9.0), (-0.5, 9.0), (-4.4, -2.0)]
    at = lambda p: (cx + p[0], cy + p[1])
    g = F()
    if c.view == 'down':
        g.poly([at(p) for p in face], P['shield'])
        g.poly([at(p) for p in [(1.2, -7.0), (3.4, -7.0), (4.0, -2.0), (0.6, 8.0)]], P['shield_lo'])
        for i in range(len(face)):
            g.limb(at(face[i]), at(face[(i + 1) % len(face)]), 0.45, P['metal_hi'] if i in (0, 5) else P['metal'])
        g.limb(at((0, -5.5)), at((0, 6.0)), 0.6, P['emblem']); g.limb(at((-2.6, -2.2)), at((2.6, -2.2)), 0.6, P['emblem'])
    else:
        g.poly([at(p) for p in face], P['shield_lo'])
        for i in range(len(face)):
            g.limb(at(face[i]), at(face[(i + 1) % len(face)]), 0.45, P['metal'])
        g.limb(at((-3.4, -4.0)), at((3.4, -4.0)), 0.6, P['leather']); g.limb(at((-3.0, 1.5)), at((3.0, 1.5)), 0.6, P['leather'])
    g.outline(); f.im.alpha_composite(g.im); f.px = f.im.load()


PARTS['head']['helm'] = helm
PARTS['torso']['plate'] = plate
PARTS['legs']['trousers'] = trousers
PARTS['weapon']['sword'] = sword
PARTS['weapon']['greatsword'] = greatsword
PARTS['offhand']['shield'] = kite_shield


# ------------------------------------------------------------------ the warrior's grips (DIR4-R1 slice 2)
def _tip(u, view, kw, length):
    """The screen point `length` px along the main weapon of a frame posed with `kw` (for a spark at the tip)."""
    j = joints(u, view, **{k: kw[k] for k in ('bob', 'head_dy', 'hand_r', 'dy') if k in kw})
    d = rig.udir(kw.get('wa', 0.0)); k = kw.get('wk', 1.0); s = u.get('scale', 1.0)
    h = j['hand_r']
    return (CX + (h[0] + d[0] * length * k - CX) / s, GY - (GY - h[1] - d[1] * length * k) / s)  # frame -> fx units


def _spark(x, y):
    """The 1H side attack's thrust spark (its colours), centred at (x, y)."""
    return [(x, y, 1.6, (255, 255, 255)), (x - 1.5, y - 1.0, 0.6, (230, 236, 245)), (x + 1.5, y - 1.0, 0.6, (230, 236, 245))]


def _impact(x, y):
    """The 2H side attack's impact spark (classes.warrior_2h's tip, its colours), centred at (x, y)."""
    return [(x, y, 2.2, (255, 255, 255)), (x - 2.5, y + 1.5, 0.9, (255, 240, 200)), (x + 2, y + 1.5, 0.9, (255, 240, 200)),
            (x - 1, y - 2.5, 0.7, (255, 220, 160))]


def _frames(u, view, keys, stand_in, spark=_spark):
    """pose_view per key dict; a key dict's 'spark' (a length along the weapon) adds `spark` at that point."""
    out = []
    for kw in keys:
        kw = dict(kw)
        sp = kw.pop('spark', None)
        if sp is not None:
            kw['fx'] = spark(*_tip(u, view, kw, sp))
        out.append(pose_view(u, view, stand_in=stand_in, **kw))
    return out


def _arc(a, b, bulge, n=14):
    """A swing's arc for the slash key: n points from `a` (where the cut starts) to `b` (where it lands), bowed
    `bulge` px up the screen at the middle."""
    c = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2 - 2 * bulge)
    return [((1 - t) ** 2 * a[0] + 2 * (1 - t) * t * c[0] + t * t * b[0], (1 - t) ** 2 * a[1] + 2 * (1 - t) * t * c[1] + t * t * b[1])
            for t in (i / (n - 1) for i in range(n))]


# Attacks (pilot review, user 2026-10-28): at chest-to-shoulder height, never below the upper body, the spark at the
# tip at chest height on the side clip's impact frame: the 1H + kite look stabs (as its side clip), the 2H look cuts
# (wind-up at the right shoulder, a level cut across the chest, as its side clip since the review).

# 1H + kite shield: the sword raised upright in the right hand (as the side view), the shield up on the left arm
# (classes.clips('warrior') variant A: carried lower)
W1_SHIELD = dict(hand_l=(-0.5, 6.0))
W1_READY = {'down': dict(hand_r=(1.5, 7.0), wa=350, **W1_SHIELD),
            'up': dict(hand_r=(1.5, 7.0), wa=10, **W1_SHIELD)}


def warrior_attack(u, view, stand_in=False):
    """attack_<view> of the 1H + kite look (the side clip's six keys and ms, the stab landing on its impact frame 2):
    a thrust at chest height (pilot review round 2, user 2026-10-28: the shield look pokes as the side view does).
    The sword levelled toward the facing, pulled back to the right shoulder, stabbed forward at chest height with a
    tip spark; the shield stays up. down: the blade points at the camera (foreshortened) by the shield's edge; up:
    the stab goes away from the camera, the forearm and hilt hidden by the body, the tip and spark past his right
    side."""
    sh = dict(hand_l=(-0.5, 5.0))
    if view == 'down':
        keys = [dict(bob=1.0, hand_r=(1.5, 3.0), wa=180, wk=0.35, hand_l=(-0.5, 5.5)),                # levelled
                dict(bob=2.0, dy=-0.5, hand_r=(2.5, 0.0), wa=175, wk=0.3, **sh),                      # pulled back
                dict(bob=1.6, dy=0.5, hand_r=(-1.5, 0.0), wa=170, wk=0.4, spark=13.5, **sh),         # the stab
                dict(bob=1.8, dy=0.5, hand_r=(-1.5, 0.0), wa=170, wk=0.4, **sh),                     # hold
                dict(bob=1.2, dy=0.2, hand_r=(1.0, 3.5), wa=330, **sh)]
    else:
        hid = dict(fore_z={'r': RAISED_BACK})
        keys = [dict(bob=1.0, hand_r=(1.5, 3.0), wa=30, wk=0.6, hand_l=(-0.5, 5.5)),                  # levelled
                dict(bob=2.0, dy=0.5, hand_r=(2.5, 0.0), wa=40, wk=0.4, **sh),                        # pulled back
                dict(bob=1.6, dy=-1.0, hand_r=(-1.0, 2.0), wa=70, wk=0.8, spark=13.5, **dict(hid, **sh)),  # the stab
                dict(bob=1.8, dy=-1.0, hand_r=(-1.0, 2.0), wa=70, wk=0.8, **dict(hid, **sh)),        # hold
                dict(bob=1.2, dy=-0.4, hand_r=(1.0, 4.0), wa=15, **sh)]
    keys.append(dict(W1_READY[view], bob=0.3))
    return _frames(u, view, keys, stand_in), [110, 160, 45, 140, 110, 110]


# 2H (WAR2H1 grip '2h', the greatsword): both hands on the grip; down: before the torso, the blade over the screen-left
# shoulder; up: hands, grip and pommel hidden by the body (held at the chest), only the blade over the right shoulder
W2_READY = {'down': dict(hand_r=(-2.5, 6.5), wa=335, grip2=rig.GRIP2),
            'up': dict(hand_r=(-2.5, 3.5), wa=25, grip2=rig.GRIP2, fore_z={'r': RAISED_BACK, 'l': RAISED_BACK})}
UP_FORE = {'r': RAISED_BACK, 'l': RAISED_BACK}


def warrior_2h_attack(u, view, stand_in=False):
    """attack_<view> of the 2H look (the side clip's seven keys and ms, the impact on its frame 3): a two-handed cut
    across the chest; the blade lifts, cocks back over the right shoulder, sweeps across (a slash arc) and lands level
    at chest height on the far side with the impact spark. up: the hands, grip and pommel stay behind the body."""
    g2 = dict(grip2=rig.GRIP2)
    if view == 'down':
        keys = [dict(bob=1.2, dy=-0.5, hand_r=(-2.0, 3.0), wa=345, **g2),                              # lift
                dict(bob=1.6, dy=-0.5, hand_r=(1.0, -3.5), wa=300, wk=0.7, wz=RAISED_BACK, **g2),      # cocked
                dict(bob=0.8, dy=0.3, hand_r=(-2.0, 2.5), wa=50, slash=_arc((12, 10), (40, 22), 2.5), **g2),  # the sweep
                dict(bob=2.0, dy=1.0, hand_r=(-5.0, 2.0), wa=95, wk=0.95, spark=16.8, **g2),           # impact
                dict(bob=1.8, dy=1.0, hand_r=(-5.0, 2.0), wa=95, wk=0.95, **g2),                       # hold
                dict(bob=1.2, dy=0.4, hand_r=(-3.0, 5.0), wa=30, **g2)]
    else:
        hid = dict(fore_z=UP_FORE, **g2)
        keys = [dict(bob=1.2, dy=0.5, hand_r=(-2.0, 3.0), wa=15, **hid),                               # lift
                dict(bob=1.6, dy=0.5, hand_r=(-1.0, 2.0), wa=50, wk=0.8, **hid),                       # cocked
                dict(bob=0.8, dy=-0.3, hand_r=(-5.0, 3.0), wa=320, slash=_arc((36, 10), (8, 22), 2.5), **hid),  # the sweep
                dict(bob=2.0, dy=-1.0, hand_r=(-6.5, 2.0), wa=265, wk=0.95, spark=16.8, **hid),       # impact
                dict(bob=1.8, dy=-1.0, hand_r=(-6.5, 2.0), wa=265, wk=0.95, **hid),                   # hold
                dict(bob=1.2, dy=-0.4, hand_r=(-5.0, 3.0), wa=330, **hid)]
    keys.append(dict(W2_READY[view], bob=0.3))
    return _frames(u, view, keys, stand_in, _impact), [120, 190, 45, 150, 120, 110, 110]


GRIPS['warrior'] = dict(idle=W1_READY, walk={v: dict(W1_READY[v], swing=0.4) for v in VIEWS},
                        attack=('attack', warrior_attack))
GRIPS['warrior/2h'] = dict(idle=W2_READY, walk={v: dict(W2_READY[v], swing=0.3) for v in VIEWS},
                           attack=('attack', warrior_2h_attack))
