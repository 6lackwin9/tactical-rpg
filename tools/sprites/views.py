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

# part kinds in drawing order within a layer (back gear after the torso garment, so a cape seen from behind covers
# the tunic on the same layer); FLAGS are the unit's boolean extras that need a drawing of their own
KINDS = ('legs', 'torso', 'back', 'offhand', 'weapon', 'head', 'hat', 'flag')
FLAGS = ('mask', 'beard', 'braid', 'pauldron')
PARTS = {k: {} for k in KINDS}
GRIPS = {}


def parts_of(u):
    """The (kind, name) parts unit `u` uses, in KINDS order."""
    out = [('legs', u['legs']), ('torso', u['torso'])]
    if u.get('back'):
        out.append(('back', u['back']))
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


def grip_of(cls, u):
    """The view keys of class `cls` worn as unit `u` (None: none): GRIPS[grip_key], with the entries its 'weapons'
    table gives for u's weapon laid over it (DIR4-R2: the 2H grip's battle axe and warhammer chop, the greatsword
    cuts)."""
    g = GRIPS.get(grip_key(cls, u))
    if g is not None and u.get('weapon') in g.get('weapons', {}):
        g = dict(g, **g['weapons'][u['weapon']])
    return g


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
        s.tip = None                               # a drawer may set the main weapon's tip (a nocked arrow, a fist)


def joints(u, view, bob=0.0, head_dy=0.0, lift_r=0.0, lift_l=0.0, hand_r=None, hand_l=None, dy=0.0, dx=0.0):
    """The skeleton of one frame in `view`, scaled about the feet as rig.pose: pelvis, hips, knees, feet, shoulders,
    elbows, hands (r = the unit's right, the main hand), neck base and head centre. hand_r / hand_l: hand targets
    (outward, down) from the shoulder (None = the rest pose, hanging at the side); lift_r / lift_l: foot lifts
    (a lifted foot rises, its knee bends); dy: the body (not the feet) moved down the screen; dx (WINDUP1: a sway or a
    tremble): the whole figure moved across the screen."""
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
    if dx:  # whole pixels: a half-pixel shift of the symmetric figure blurs the legs together
        j = {k: (p[0] + round(dx * s), p[1]) for k, p in j.items()}
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
              flash=False, fx=None, tails=0.0, stand_in=False, dx=0.0, **e):
    """One frame of unit `u` in `view` ('down' or 'up'), a W x W RGBA image (R1, R2, R6). The keys are joints'; fx:
    (x, y, r, colour) effect discs at side-frame positions scaled about the feet; e: extra keys for the part drawers
    (Ctx.e). stand_in: draw STAND_INS for parts without a view drawer (review boards only). The frame's info holds
    the main hand ('hand') and, for a weapon of rig.WEAPON_TIP, its tip ('tip'), as rig.pose's (WINDUP1)."""
    if view not in VIEWS:
        raise ValueError("view must be down or up, not %r" % (view,))
    s = u.get('scale', 1.0)
    j = joints(u, view, bob, head_dy, lift_r, lift_l, hand_r, hand_l, dy, dx)
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
    if e.get('arrow'):  # DIR4-R2: a loosed arrow (as rig.pose's, 1 x 5 px plus head and fletching) flying down or up
        _arrow(f, c, *e['arrow'])
    if flash:
        rig._flash(f)
    f.im.info['hand'] = j['hand_r']
    f.im.info['off'] = j['hand_l']  # the off hand (a ranger's string hand, for the buff motes), as rig.pose's
    if c.tip is not None:
        f.im.info['tip'] = c.tip
    elif u.get('weapon') in rig.WEAPON_TIP:
        d = rig.udir(e.get('wa', 0.0)); k = e.get('wk', 1.0) * rig.WEAPON_TIP[u['weapon']]
        f.im.info['tip'] = (j['hand_r'][0] + d[0] * k, j['hand_r'][1] + d[1] * k)
    return f.im


def _arrow(f, c, x, y, sign):
    """A loosed arrow at frame position (x, y) (scaled about the feet), head `sign` (+1 down the screen, -1 up)."""
    s, P = c.s, c.P
    x, y = round(CX + (x - CX) * s), round(GY - (GY - y) * s)
    for k in range(5):
        f.dot(x, y - sign * k, P['wood'])
    f.dot(x, y + sign, P['metal_hi'])
    f.dot(x - 1, y - sign * 4, P['accent']); f.dot(x + 1, y - sign * 4, P['accent'])


# ------------------------------------------------------------------ clip shapes (DIR4 6.3)
def idle_keys(u, frames=8, hf_amp=0.8, **kw):
    """The pose_view keys of each idle frame (idle draws them; WINDUP1's view wind-ups start from frame 0's)."""
    out = []
    for i in range(frames):
        t = i / frames * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t - 0.6); lag = 0.5 + 0.5 * math.sin(t - 1.4)
        k = dict(kw); hr = k.pop('hand_r', None); hl = k.pop('hand_l', None)
        hr = None if hr is None else (hr[0], hr[1] + lag * hf_amp)
        hl = None if hl is None else (hl[0], hl[1] + lag * 0.6)
        if hr is None and hl is None:  # rest pose: the hands breathe with the body
            hr = hl = (1.2, 2 * u['arm'] / u.get('scale', 1.0) - 1.0 + lag * 0.4)
        out.append(dict(bob=k.pop('bob', 0.0) + b * 0.9, head_dy=lag * 0.8 - 0.2, hand_r=hr, hand_l=hl, tails=t, **k))
    return out


def idle(u, view, frames=8, hf_amp=0.8, stand_in=False, **kw):
    """idle_<view>: rig.idle's bob and breath (8 frames, 160 ms), hands from the grip's ready pose."""
    return [pose_view(u, view, stand_in=stand_in, **k) for k in idle_keys(u, frames, hf_amp, **kw)], [160] * frames


def step_height(ph, lift=3.4, rise=1.2):
    """DIR4-R1F: the height of one foot at gait phase `ph` as the front and back views show it (the stride runs along the
    view axis, so it reads as height, not as x): in its stance half the foot recedes and rises `rise` px; in its swing half
    it comes back from `rise` to the ground with a `lift` px arc on top, so one foot is planted (fixed on screen) while
    the other lifts, clearly, in turn."""
    ph %= 1.0
    if ph < 0.5:
        return rise * (ph / 0.5)
    v = (ph - 0.5) / 0.5
    return rise * (1 - v) + lift * math.sin(math.pi * v)


def walk(u, view, frames=8, stride=6.0, lift=3.4, rise=1.2, stand_in=False, **kw):
    """walk_<view>: rig.walk's timing (8 frames, 90 ms), the legs alternating with a visible lift and plant (DIR4-R1F:
    step_height, the lifted knee bending outward); the body bobs 1.1 - 2.2 |sin| in time with the steps; the arms swing
    as 1 px vertical offsets opposite to the legs."""
    out = []
    rest = 2 * u['arm'] / u.get('scale', 1.0) - 1.0
    for i in range(frames):
        ph = i / frames; s = math.sin(2 * math.pi * ph)
        bob = 1.1 - 2.2 * abs(s)
        k = dict(kw); hr = k.pop('hand_r', (1.2, rest)); hl = k.pop('hand_l', (1.2, rest)); swing = k.pop('swing', 1.0)
        hr = (hr[0], hr[1] + s * swing)                                 # the right arm swings with the left leg
        hl = (hl[0], hl[1] - s * swing)
        out.append(pose_view(u, view, bob=k.pop('bob', 0.0) + bob, head_dy=bob * 0.25,
                             lift_r=step_height(ph + 0.5, lift, rise), lift_l=step_height(ph, lift, rise),
                             hand_r=hr, hand_l=hl, tails=ph * 6, stand_in=stand_in, **k))
    return out, [90] * frames


# DASH1: rig.dash's five frames seen along the run: the stride reads as height, so the coil and the plant sit low and the
# flight has both feet off the ground; `lean` tips the head toward the run (down the screen in `down`, up it in `up`).
DASH_KEYS = (dict(bob=4.0, dy=0.5, head=2.0, lift_r=0.0, lift_l=0.0, arm=(0.0, 1.0)),
             dict(bob=1.5, dy=0.0, head=1.5, lift_r=0.0, lift_l=6.5, arm=(0.0, 0.0)),
             dict(bob=-1.5, dy=0.0, head=1.0, lift_r=4.5, lift_l=7.5, arm=(0.5, -0.5)),
             dict(bob=0.5, dy=0.0, head=1.0, lift_r=7.0, lift_l=2.5, arm=(0.0, 0.0)),
             dict(bob=4.0, dy=0.5, head=2.0, lift_r=1.5, lift_l=0.0, arm=(0.0, 1.0)))


def dash_trail(P, view, f):
    """Frame `f`'s dash marks in `view`: a puff on both sides of the feet at the drive and the plant, as rig.dash_trail's;
    no speed lines (user 2026-11-02: the side view's only)."""
    lo = P['metal_lo']
    if f in (1, 4):
        return [(CX + 8.0, 41.5, 1.8, lo), (CX - 8.0, 41.5, 1.8, lo), (CX + 12.0, 40.0, 1.0, lo), (CX - 12.0, 40.0, 1.0, lo)]
    return []


def dash(u, view, stand_in=False, hand_r=None, hand_l=None, **kw):
    """dash_<view>: rig.dash's frames and ms; `hand_r`, `hand_l` and `kw` are the grip's carry (grip_kw's form)."""
    rest = 2 * u['arm'] / u.get('scale', 1.0) - 1.0
    hr, hl = hand_r or (1.2, rest), hand_l or (1.2, rest)
    out = []
    for i, key in enumerate(DASH_KEYS):
        k = dict(kw, **key); ax, ay = k.pop('arm'); k['head_dy'] = k.pop('head')   # the key's body beats a carry's
        out.append(pose_view(u, view, hand_r=(hr[0] + ax, hr[1] + ay), hand_l=(hl[0] + ax, hl[1] + ay), tails=i * 2.0,
                             fx=dash_trail(u['pal'], view, i), stand_in=stand_in, **k))
    return out, list(rig.DASH_MS)


def clips(cls, u, stand_in=False):
    """The view clips of class `cls` worn as unit `u`: {id: frames}, {id: ms} with ids '<side clip>_<view>' (idle,
    walk and, when the grip has one, its attack). stand_in: a body without view support is drawn with STAND_INS and the
    rest pose (review boards only)."""
    g = grip_of(cls, u)
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
    if 'windups' in g:  # WINDUP1 K6: the six wind-up, stagger and dazed rows per view
        for view in VIEWS:
            W, WT = windup_rows(u, view, g, stand_in)
            for n in W:
                A['%s_%s' % (n, view)], T['%s_%s' % (n, view)] = W[n], WT[n]
    if 'dash' in g:  # DASH1: the dash per view
        for view in VIEWS:
            A['dash_' + view], T['dash_' + view] = dash(u, view, stand_in, **grip_kw(g, 'dash', view))
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


# the weapons whose side drawing has gold fittings (guard, pommel): a belt buckle in a view matches them (trim); with
# any other weapon the side rows never show the trim colour (the side's buckle is always covered), so the view's
# buckle is steel (metal_hi) and adds no palette slot (R4; DIR4-R2: the ranger, the 2H axe and hammer)
GOLD_FITTINGS = ('sword', 'sword2', 'greatsword', 'dagger')


def _buckle(c):
    return c.P['trim'] if c.u.get('weapon') in GOLD_FITTINGS else c.P['metal_hi']


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
            f.rect(px - 4, py - 1.2, px + 4, py - 0.2, P['leather']); f.rect(px - 0.5, py - 1.2, px + 0.5, py - 0.2, _buckle(c))
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
    """The screen point `length` px along the main weapon of a frame posed with `kw` (for a spark at the tip).
    DIR4-R2: length ('l', n) is n px along the off-hand weapon (wb, wbk); ('hand', dx, dy) a point dx, dy px from the
    main hand (a fist's spark)."""
    j = joints(u, view, **{k: kw[k] for k in ('bob', 'head_dy', 'hand_r', 'hand_l', 'dy', 'dx') if k in kw})
    s = u.get('scale', 1.0)
    if isinstance(length, tuple) and length[0] == 'hand':
        h = j['hand_r']
        return (CX + (h[0] + length[1] * s - CX) / s, GY - (GY - h[1] - length[2] * s) / s)
    if isinstance(length, tuple):
        d = rig.udir(kw.get('wb', 180.0)); k = kw.get('wbk', 1.0); h = j['hand_l']; length = length[1]
    else:
        d = rig.udir(kw.get('wa', 0.0)); k = kw.get('wk', 1.0); h = j['hand_r']
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
    return _frames(u, view, warrior_attack_keys(view), stand_in), [110, 160, 45, 140, 110, 110]


def warrior_attack_keys(view):
    """The keys of attack_<view> of the 1H + kite look (frame 1, pulled back, is the `windup` hold, WINDUP1 K2)."""
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
    return keys


# 2H (WAR2H1 grip '2h', the greatsword): both hands on the grip; down: before the torso, the blade over the screen-left
# shoulder; up: hands, grip and pommel hidden by the body (held at the chest), only the blade over the right shoulder
W2_READY = {'down': dict(hand_r=(-2.5, 6.5), wa=335, grip2=rig.GRIP2),
            'up': dict(hand_r=(-2.5, 3.5), wa=25, grip2=rig.GRIP2, fore_z={'r': RAISED_BACK, 'l': RAISED_BACK})}
UP_FORE = {'r': RAISED_BACK, 'l': RAISED_BACK}


def warrior_2h_attack(u, view, stand_in=False):
    """attack_<view> of the 2H look (the side clip's seven keys and ms, the impact on its frame 3): a two-handed cut
    across the chest; the blade lifts, cocks back over the right shoulder, sweeps across (a slash arc) and lands level
    at chest height on the far side with the impact spark. up: the hands, grip and pommel stay behind the body."""
    return _frames(u, view, warrior_2h_attack_keys(view), stand_in, _impact), [120, 190, 45, 150, 120, 110, 110]


def warrior_2h_attack_keys(view):
    """The keys of attack_<view> of the 2H look (frame 1, cocked, is the `windup` hold, WINDUP1 K2)."""
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
    return keys


# ------------------------------------------------------------------ wind-ups, stagger and dazed (WINDUP1 K6)
# Every new side clip gets a `_down` and an `_up` row with the side row's frames, ms and loop_from (rig.windup_rows
# draws them through pose_view). A grip's 'windups' entry holds per view: 'attack' (its attack keys; frame
# strike - 1 is the `windup` hold, K2), 'holds' (the other wind-ups' held keys), 'stagger' (four keys) and 'dazed'.
# The side keys turn as DIR4 6.3: a knock-back moves the body 1 px away from the facing (up the screen in `down`,
# down it in `up`); the dazed sway moves the figure across the screen in whole pixels (dx), the heavy tremble the
# weapon hand (shake_hand).
def view_rest(u):
    """The values pose_view draws for a key it is not given (hands: the rest pose), for rig.lerp_kw."""
    rest = (1.2, 2 * u['arm'] / u.get('scale', 1.0) - 1.0)
    return dict(bob=0.0, head_dy=0.0, lift_r=0.0, lift_l=0.0, dy=0.0, dx=0.0, wk=1.0, hand_r=rest, hand_l=rest, tails=0.0,
                wbk=1.0, glow=0.0, pull=0.0, bk=1.0, fr=2.0, fl=2.0)


def windup_rows(u, view, g, stand_in=False):
    """The six WINDUP1 rows of unit `u` in `view` ({side id: frames}, {side id: ms}) from grip keys `g`; the grip's
    windups 'opts' are rig.windup_rows's motes and steps (WINDUP1-R2: the ranger's motes from the string hand, the
    mage's circling the gem and its pulsing glow)."""
    w = g['windups']
    ready = idle_keys(u, **grip_kw(g, 'idle', view))[0]
    holds = dict(w['holds'][view], windup=w['attack'](view)[w['strike'] - 1])
    def draw(kw):
        return pose_view(u, view, stand_in=stand_in, **kw)
    return rig.windup_rows(draw, ready, holds, w['stagger'][view], w['dazed'][view], view_rest(u), shake_hand,
                           **w.get('opts', {}))


def shake_hand(kw, d):
    """The heavy wind-up's tremble in a view: the main hand (and the weapon) moved `d` px across the screen, the
    body still (a half-pixel shift of the whole figure blurs its legs)."""
    hx, hy = kw.get('hand_r') or (0.0, 0.0)
    kw['hand_r'] = (hx + d, hy)


# 1H + kite shield: the windup is attack frame 1 (the sword pulled back by the right shoulder)
W1_LOW = dict(hand_l=(1.0, 8.5))                      # the shield lowered (buff, dazed)
W1_WINDUPS = dict(attack=warrior_attack_keys, strike=2, holds={
    'down': {'windup_heavy': dict(bob=3.2, dy=-0.5, hand_r=(3.0, -2.5), wa=335, wk=0.85, hand_l=(-0.5, 5.0)),
             'windup_buff': dict(bob=0.6, hand_r=(-2.5, 2.5), wa=0, **W1_LOW),
             'windup_debuff': dict(bob=1.0, dy=0.5, hand_r=(2.0, 2.0), wa=212, wk=0.6, hand_l=(0.5, 7.0))},
    'up': {'windup_heavy': dict(bob=3.2, dy=0.5, hand_r=(3.0, -2.5), wa=20, wk=0.85, hand_l=(-0.5, 5.0)),
           'windup_buff': dict(bob=0.6, hand_r=(-1.5, 1.0), wa=12, fore_z={'r': RAISED_BACK}, **W1_LOW),
           'windup_debuff': dict(bob=1.0, dy=-0.5, hand_r=(2.0, 1.0), wa=40, wk=0.55, hand_l=(0.5, 7.0))}},
    stagger={
    'down': [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(6.0, 1.0), wa=300, hand_l=(-0.5, 6.0)),
             dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(5.0, 3.0), wa=320, hand_l=(0.0, 6.5)),
             dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(1.5, 8.5), wa=195, wk=0.8, hand_l=(1.0, 8.0)),
             dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(1.5, 9.0), wa=190, wk=0.8, hand_l=(1.2, 9.0))],
    'up': [dict(bob=0.5, dy=1.0, head_dy=-1.0, hand_r=(6.0, 1.0), wa=60, hand_l=(-0.5, 6.0)),
           dict(bob=1.5, dy=1.0, head_dy=-0.5, lift_r=1.5, hand_r=(5.0, 3.0), wa=40, hand_l=(0.0, 6.5)),
           dict(bob=2.5, dy=0.5, head_dy=1.0, hand_r=(1.5, 8.5), wa=165, wk=0.8, hand_l=(1.0, 8.0)),
           dict(bob=2.2, dy=1.0, head_dy=1.5, hand_r=(1.5, 9.0), wa=170, wk=0.7, hand_l=(1.2, 9.0))]},
    dazed={'down': dict(bob=2.0, head_dy=1.5, hand_r=(1.5, 9.5), wa=185, wk=0.8, hand_l=(1.2, 9.0)),
           'up': dict(bob=2.0, head_dy=1.5, hand_r=(1.5, 9.5), wa=170, wk=0.6, hand_l=(1.2, 9.0))})

# 2H (the greatsword): the windup is attack frame 1 (cocked over the right shoulder); both hands on the grip
G2 = dict(grip2=rig.GRIP2)
W2_WINDUPS = dict(attack=warrior_2h_attack_keys, strike=2, holds={
    'down': {'windup_heavy': dict(bob=3.2, dy=-0.5, hand_r=(3.0, -2.5), wa=318, wk=0.8, wz=RAISED_BACK, **G2),
             'windup_buff': dict(bob=1.0, head_dy=0.8, hand_r=(-4.0, 8.0), wa=180, wz=FRONT, **G2),
             'windup_debuff': dict(bob=1.0, dy=0.5, hand_r=(0.5, 2.5), wa=212, wk=0.6, **G2)},
    'up': {'windup_heavy': dict(bob=3.2, dy=0.5, hand_r=(0.5, -1.0), wa=38, wk=0.75, fore_z=UP_FORE, **G2),
           'windup_buff': dict(bob=1.0, head_dy=0.8, hand_r=(-2.5, 3.5), wa=10, wk=1.0, fore_z=UP_FORE, **G2),
           'windup_debuff': dict(bob=1.0, dy=-0.5, hand_r=(2.0, 1.0), wa=40, wk=0.55, fore_z=UP_FORE, **G2)}},
    stagger={
    'down': [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(5.0, -1.0), wa=315, **G2),
             dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(4.0, 2.0), wa=330, **G2),
             dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(-2.0, 8.0), wa=195, wk=0.8, **G2),
             dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(-2.0, 8.5), wa=190, wk=0.8, **G2)],
    'up': [dict(bob=0.5, dy=1.0, head_dy=-1.0, hand_r=(5.0, -1.0), wa=45, **G2),
           dict(bob=1.5, dy=1.0, head_dy=-0.5, lift_r=1.5, hand_r=(4.0, 2.0), wa=30, **G2),
           dict(bob=2.5, dy=0.5, head_dy=1.0, hand_r=(-3.5, 8.0), wa=170, wk=0.6, fore_z=UP_FORE, **G2),
           dict(bob=2.2, dy=1.0, head_dy=1.5, hand_r=(-3.5, 8.5), wa=172, wk=0.6, fore_z=UP_FORE, **G2)]},
    dazed={'down': dict(bob=2.0, head_dy=1.5, hand_r=(-2.5, 9.0), wa=188, wk=0.75, **G2),
           'up': dict(bob=2.0, head_dy=1.5, hand_r=(-3.0, 5.0), wa=32, wk=0.85, fore_z=UP_FORE, **G2)})

GRIPS['warrior'] = dict(idle=W1_READY, walk={v: dict(W1_READY[v], swing=0.4) for v in VIEWS},
                        attack=('attack', warrior_attack), windups=W1_WINDUPS)
# 1H + buckler (WAR2H1 grip 'buckler'; WINDUP1-R1 review, user 2026-10-29): the sword raised upright in front and
# back views as the kite look's (the DIR4-R1 review note for a shield or buckler warrior), not hanging along the leg.
# DIR4-R3 (DIR4-R1F's buckler part, user 2026-10-29) adds the buckler drawing and, per weapon, the attack and the
# wind-up rows (GRIPS['warrior/buckler']['weapons'], at the end of this module).
WB_READY = {v: dict(W1_READY[v], hand_l=(1.0, 6.0)) for v in VIEWS}   # DIR4-R3: the buckler a little out, so it shows from behind
GRIPS['warrior/buckler'] = dict(idle=WB_READY, walk={v: dict(WB_READY[v], swing=0.4) for v in VIEWS})
GRIPS['warrior/2h'] = dict(idle=W2_READY, walk={v: dict(W2_READY[v], swing=0.3) for v in VIEWS},
                           attack=('attack', warrior_2h_attack), windups=W2_WINDUPS)


# ------------------------------------------------------------------ the other classes' parts (DIR4-R2)
# Decision "Front and back views follow the side model" (user 2026-10-29): each drawer is the front and back of the side
# rig's own part (rig.draw_head, draw_torso, draw_back_gear, draw_robe_skirt, draw_weapon, draw_bow): the same shapes,
# colours and placement (the hood's drape, the cowl's circlet, the hat's band and bent cone, the mage's white hair and
# beard, the brawler's headband and its tails, the sash, the cross belt, the robe's trim and rope belt, the cape, the
# quiver, the weapons' parts). Weapon keys beyond the warrior's: wb / wbk / oz the off-hand weapon's angle, length and
# layer (the rogue's second dagger); bow_a / bk / bs / pull / nock the bow (its tilt, the bulge's foreshortening and
# side, the draw and the nocked arrow); glow the staff's gem; fr / fl the fists' size (0: an open hand); wn the side a
# 2H head's edge faces (+1 the unit's right side, the default; -1 the other).
def _side(c, side):
    """The screen x sign of the unit's `side` ('r' or 'l') in this view."""
    return c.rs if side == 'r' else -c.rs


def _chest(f, c, colour, widen=0.0):
    """The torso's twelve rows (as rig.draw_torso's), widening to the shoulders; colour(t) gives a row's colour from the
    waist (t = 0) to the shoulders (t = 1)."""
    px, py = c.j['pelvis']; T = c.u['tl']; g = c.u.get('girth', 0.0)
    for i in range(12):
        t = i / 11.0; y = py - T * t
        half = 3.8 + 1.0 * t + g * (0.6 + 0.4 * t) + widen
        f.rect(px - half, y - 0.5, px + half, y + 0.5, colour(t))


def headband(f, c, layer):
    """The brawler's head (rig 'headband'): short dark hair, the headband round the brow, its tails trailing from the
    knot at the back and a set jaw. down: the face, the hair over the brow and short at the temples, the band across the
    forehead, the tails' ends showing past the unit's left side; up: the hair over the skull to the nape, the band round
    the back, its knot and both tails hanging, swaying."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; t = c.tails; k = _side(c, 'l')
    if c.view == 'down':
        for i in range(3):   # the tails behind the head, their ends past its left side
            f.dot(hx + k * (3.6 + i * 0.9), hy - 1.4 + i * 0.7 + math.sin(t + i) * 0.5, P['accent'] if i < 2 else P['accent_lo'])
        f.disc(hx, hy, 3.4, P['skin'])
        f.rect(hx - 3, hy - 3.6, hx + 3, hy - 2.2, P['hair'])
        f.rect(hx - 3.5, hy - 2.2, hx - 3.0, hy - 0.4, P['hair']); f.rect(hx + 3.0, hy - 2.2, hx + 3.5, hy - 0.4, P['hair'])
        f.rect(hx - 3.4, hy - 2.0, hx + 3.4, hy - 1.2, P['accent'])
        f.dot(hx - 1, hy + 0.2, OUT if c.hurt else P['eye']); f.dot(hx + 1, hy + 0.2, OUT if c.hurt else P['eye'])
        f.rect(hx - 1, hy + 2.4, hx + 1, hy + 2.4, P['skin_lo'])                              # set jaw
    else:
        f.disc(hx, hy, 3.4, P['skin'])
        f.disc(hx, hy - 0.6, 3.3, P['hair']); f.rect(hx - 3, hy - 0.6, hx + 3, hy + 0.8, P['hair'])
        f.dot(hx - 3.6, hy + 0.4, P['skin_lo']); f.dot(hx + 3.6, hy + 0.4, P['skin_lo'])   # ears
        f.rect(hx - 3.4, hy - 2.0, hx + 3.4, hy - 1.2, P['accent'])
        f.rect(hx - 0.5, hy - 2.2, hx + 0.5, hy - 1.0, P['accent_lo'])                       # the knot
        for i in range(3):                                                                    # the tails, swaying
            f.dot(hx + k * (0.6 + 0.5 * i) + math.sin(t + i) * 0.4, hy - 0.4 + i * 0.9, P['accent'] if i < 2 else P['accent_lo'])


def hood(f, c, layer):
    """The ranger's and the rogue's hood (rig 'hood'): the cowl of cloth with its highlights and the drape falling to
    the shoulders. down: the face deep in the opening, the shadowed brow edge, two eyes; up: the hood's back and its
    drape coming to a point between the shoulders, a seam."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    if c.view == 'down':
        f.rect(hx - 4.2, hy + 0.5, hx - 2.8, hy + 3.6, P['main_lo']); f.rect(hx + 2.8, hy + 0.5, hx + 4.2, hy + 3.6, P['main_lo'])
        f.disc(hx, hy - 0.4, 3.9, P['main'])
        f.disc(hx, hy + 0.9, 2.5, P['skin'])
        f.rect(hx - 2, hy - 1.4, hx + 2, hy - 1.4, P['main_lo'])                              # the deep front edge
        f.dot(hx - 1.6, hy - 3.0, P['main_hi']); f.dot(hx - 0.6, hy - 3.6, P['main_hi'])
        f.dot(hx - 1, hy + 0.4, OUT if c.hurt else P['eye']); f.dot(hx + 1, hy + 0.4, OUT if c.hurt else P['eye'])
    else:  # the drape covers the nape down to the shoulders
        f.poly([(hx - 3.8, hy + 0.2), (hx + 3.8, hy + 0.2), (hx + 2.4, hy + 6.0), (hx - 2.4, hy + 6.0)], P['main_lo'])
        f.disc(hx, hy - 0.4, 3.9, P['main'])
        f.rect(hx, hy - 3.4, hx, hy + 1.0, P['main_lo'])                                      # the seam
        f.dot(hx - 1.6, hy - 3.0, P['main_hi']); f.dot(hx - 0.6, hy - 3.6, P['main_hi'])


def mask(f, c, layer):
    """The rogue's mask (rig 'hood' + mask): the cloth over the lower face, below the eyes (down only; from the back
    the hood covers its tie)."""
    if layer != HEAD or c.view != 'down':
        return
    hx, hy = c.j['head']
    f.rect(hx - 2.4, hy + 1.4, hx + 2.4, hy + 3.0, c.P['accent'])


def cowl(f, c, layer):
    """The priest's cowl (rig 'cowl'): cloth over the head and falling over the shoulders, the gold circlet on it.
    down: the face in the opening, its shaded rim, the circlet across the brow, two eyes; up: the cowl's back, its fold
    and the circlet's band round the back."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    low = 4.6 if c.view == 'down' else 6.0                                                    # from behind: to the shoulders
    f.poly([(hx - 3.6, hy + 0.2), (hx + 3.6, hy + 0.2), (hx + 4.6, hy + low), (hx - 4.6, hy + low)], P['main_lo'])
    f.disc(hx, hy - 0.2, 3.9, P['main'])
    if c.view == 'down':
        f.disc(hx, hy + 0.9, 2.4, P['skin'])
        f.rect(hx - 2.8, hy + 0.8, hx - 2.8, hy + 3.2, P['main_lo']); f.rect(hx + 2.8, hy + 0.8, hx + 2.8, hy + 3.2, P['main_lo'])
        f.rect(hx - 2.0, hy - 2.8, hx + 2.0, hy - 2.2, P['trim'])
        f.dot(hx - 1, hy + 0.6, OUT if c.hurt else P['eye']); f.dot(hx + 1, hy + 0.6, OUT if c.hurt else P['eye'])
    else:
        f.rect(hx - 3.5, hy - 2.2, hx + 3.5, hy - 1.6, P['trim'])
        f.rect(hx, hy - 1.6, hx, hy + 3.8, P['main_lo'])                                      # the fold down the back


def hat(f, c, layer):
    """The mage's hat and hair (rig 'hat'): the wide brim, the gold band, the tall cone bent at its tip; long white hair
    under the brim and the beard. down: the face, the hair falling at both sides, the beard to a point; up: the hair
    over the back of the head down to the shoulders. The cone's tip bends over the unit's left side (the side view
    bends it back)."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; k = _side(c, 'l')
    lock = P.get('metal_hi', P['hair'])                                                       # the strands' sheen
    if c.view == 'down':
        for s in (-1, 1):                                                                     # locks to the shoulders
            f.poly([(hx + s * 3.9, hy - 2.6), (hx + s * 2.6, hy - 2.6), (hx + s * 2.4, hy + 3.8), (hx + s * 3.7, hy + 3.0)], P['hair'])
            f.dot(hx + s * 3.2, hy + 0.4, lock)
        f.disc(hx, hy, 3.2, P['skin'])
        f.poly([(hx - 2.4, hy + 1.2), (hx + 2.4, hy + 1.2), (hx + 1.8, hy + 4.2), (hx, hy + 5.4), (hx - 1.8, hy + 4.2)], P['hair'])
        f.dot(hx - 0.6, hy + 3.2, lock); f.dot(hx + 0.6, hy + 2.4, lock)
        f.dot(hx - 1, hy - 0.4, OUT if c.hurt else P['eye']); f.dot(hx + 1, hy - 0.4, OUT if c.hurt else P['eye'])
    else:  # the hair falls from under the brim to the shoulders in locks
        f.disc(hx, hy, 3.4, P['hair'])
        f.poly([(hx - 3.6, hy - 1.0), (hx + 3.6, hy - 1.0), (hx + 3.4, hy + 4.8), (hx + 1.8, hy + 3.8), (hx, hy + 5.4),
                (hx - 1.8, hy + 3.8), (hx - 3.4, hy + 4.8)], P['hair'])
        f.limb((hx - 1.6, hy - 0.6), (hx - 1.9, hy + 3.0), 0.3, lock); f.limb((hx + 1.4, hy - 0.2), (hx + 1.7, hy + 3.4), 0.3, lock)
    f.rect(hx - 5.2, hy - 3.4, hx + 5.2, hy - 2.6, P['main_lo'])
    f.poly([(hx - 3.2, hy - 3.2), (hx + 3.2, hy - 3.2), (hx + k * 0.4, hy - 10.5)], P['main'])
    f.limb((hx + k * 0.4, hy - 10.2), (hx + k * 2.6, hy - 11.4), 0.6, P['main'])
    f.rect(hx - 3.0, hy - 4.4, hx + 3.0, hy - 3.6, P['trim'])


def bare(f, c, layer):
    """The brawler's bare torso (rig 'bare'): skin to the waist, the trousers' top below, the orange sash with its end
    hanging at the left hip. down: the chest and belly lines; up: the spine and the shoulder blades."""
    if layer != TORSO:
        return
    P = c.P; px, py = c.j['pelvis']; T = c.u['tl']
    _chest(f, c, lambda t: P['skin'] if t > 0.22 else P['leg'], 0.6)
    if c.view == 'down':
        f.rect(px - 3.2, py - T * 0.62, px - 1.0, py - T * 0.62, P['skin_lo']); f.rect(px + 1.0, py - T * 0.62, px + 3.2, py - T * 0.62, P['skin_lo'])
        f.dot(px, py - T * 0.5, P['skin_lo']); f.dot(px - 1, py - T * 0.38, P['skin_lo']); f.dot(px + 1, py - T * 0.38, P['skin_lo'])
    else:
        f.rect(px, py - T * 0.85, px, py - T * 0.35, P['skin_lo'])
        f.dot(px - 2.4, py - T * 0.72, P['skin_lo']); f.dot(px + 2.4, py - T * 0.72, P['skin_lo'])
    f.rect(px - 4.6, py - 2.2, px + 4.6, py - 0.8, P['accent'])
    k = _side(c, 'l')
    f.limb((px + k * 3.8, py - 1.5), (px + k * 4.8, py + 3.0), 0.6, P['accent_lo'])


def leather(f, c, layer):
    """The ranger's and the rogue's leather tunic (rig 'leather'): the tunic over the trousers' top, the cross belt
    from the left shoulder to the right hip (the quiver's strap), the belt; down: its buckle; up: the belt line."""
    if layer != TORSO:
        return
    P = c.P; px, py = c.j['pelvis']; T = c.u['tl']
    _chest(f, c, lambda t: P['main'] if t > 0.25 else P['leg'])
    l, r = _side(c, 'l'), _side(c, 'r')
    for i in range(9):
        t = i / 8.0
        f.disc(px + l * 3.2 * (1 - t) + r * 3.0 * t, py - T * (0.95 - 0.6 * t), 0.5, P['leather'])
    f.rect(px - 4, py - 1.2, px + 4, py - 0.2, P['leather'])
    if c.view == 'down':
        f.dot(px, py - 0.7, _buckle(c))
    else:
        f.rect(px - 4, py - 0.2, px + 4, py - 0.2, P['leather_lo'])


def robe(f, c, layer):
    """The priest's and the mage's robe (rig 'robe'): the robe to the waist (darker below the rope belt's line), the
    rope belt; down: the trim down the front and the belt's knot; up: the back seam."""
    if layer != TORSO:
        return
    P = c.P; px, py = c.j['pelvis']; T = c.u['tl']
    _chest(f, c, lambda t: P['main'] if t > 0.15 else P['main_lo'])
    if c.view == 'down':
        f.rect(px, py - T + 1, px, py - 3.4, P['trim'])
    else:
        f.rect(px, py - T * 0.9, px, py - T * 0.3, P['main_lo'])
    f.rect(px - 4, py - 3.2, px + 4, py - 2.4, P['main_lo'])
    if c.view == 'down':
        f.dot(px + 1.5, py - 2.8, P['trim'])


def robe_skirt(f, c, layer):
    """The robe's skirt (rig draw_robe_skirt): from the waist to 2.6 px above each foot, its hem line; down: the
    front trim continues to the hem; up: a fold down the back."""
    if layer != SKIRT:
        return
    P, j = c.P, c.j; px, py = j['pelvis']
    corner = {}
    for side in ('r', 'l'):
        fx_, fy_ = j['foot_' + side]; out = 1.0 if fx_ > CX else -1.0
        corner[side] = (fx_ + out * 1.8, fy_ - 2.6)
    lo, hi = (corner['r'], corner['l']) if corner['r'][0] < corner['l'][0] else (corner['l'], corner['r'])
    mid = ((lo[0] + hi[0]) / 2, max(lo[1], hi[1]) + 0.5)
    f.poly([(px - 4.2, py - 1), (px + 4.2, py - 1), hi, mid, lo], P['main'])
    f.limb(lo, mid, 0.5, P['main_lo']); f.limb(mid, hi, 0.5, P['main_lo'])
    if c.view == 'down':
        f.rect(px, py - 1, px, mid[1] - 1.2, P['trim'])
    else:
        f.rect(px, py + 1, px, mid[1] - 1.5, P['main_lo'])


def quiver(f, c, layer):
    """The ranger's quiver (rig 'quiver') slung on the back from the right hip to the left shoulder. down: behind the
    body, only its fletchings (white, one red) over the left shoulder; up: the whole quiver over the back, its mouth
    and the fletchings at the left shoulder."""
    P, j = c.P, c.j; px, py = j['pelvis']; T = c.u['tl']
    sl = j['sh_l']; k = _side(c, 'l')
    top = (sl[0] - k * 1.0, sl[1] - 2.2); bot = (px - k * 2.4, py - T * 0.3)
    if c.view == 'down' and layer == BEHIND:
        f.limb(bot, top, 1.1, P['leather_lo'])
        for i in range(3):
            f.dot(top[0] - 1 + i, top[1] - 1.5 - (i % 2), rig.WHITE if i != 1 else P['accent'])
    elif c.view == 'up' and layer == FRONT:
        f.limb(bot, top, 1.2, P['leather_lo'])
        f.limb((top[0] - 1.2, top[1] + 0.6), (top[0] + 1.2, top[1] - 0.6), 0.5, P['leather'])
        for i in range(3):
            f.dot(top[0] - 1 + i, top[1] - 1.5 - (i % 2), rig.WHITE if i != 1 else P['accent'])


def cape(f, c, layer):
    """The rogue's cape (rig 'cape'), hanging from the shoulders to the thighs. down: behind the body, showing at its
    sides and between the legs, the cords over the shoulders and the clasp at the throat; up: over the back and the
    upper legs, two folds, swaying with the walk."""
    P, j = c.P, c.j; px, py = j['pelvis']; sy = j['neck'][1]; sw = 0.5 * math.sin(c.tails)
    if c.view == 'down' and layer == BEHIND:
        f.poly([(px - 5.0, sy + 0.5), (px + 5.0, sy + 0.5), (px + 6.8 + sw, py + 6), (px - 6.8 + sw, py + 6)], P['accent_lo'])
    elif c.view == 'down' and layer == TORSO:
        for side in ('r', 'l'):
            sx, sy_ = j['sh_' + side]
            f.limb((sx, sy_ - 0.4), (px + (1.4 if sx > px else -1.4), sy + 2.0), 0.6, P['accent_lo'])
        f.dot(px, sy + 2.2, P['trim'])
    elif c.view == 'up' and layer == TORSO:
        f.poly([(px - 4.8, sy - 0.2), (px + 4.8, sy - 0.2), (px + 6.0 + sw, py + 6), (px - 6.0 + sw, py + 6)], P['accent_lo'])
        f.limb((px - 1.8, sy + 3), (px - 2.6 + sw, py + 5), 0.3, P['accent'])
        f.limb((px + 1.8, sy + 3), (px + 2.6 + sw, py + 5), 0.3, P['accent'])


def fists(f, c, layer):
    """The brawler's wrapped fists (rig 'fists'), on each hand's layer: a fist of radius fr / fl (default 2, the side's
    clenched fist; 0 leaves the open hand) with the wrap's highlight. Sets the tip (the right fist's knuckles, along the
    forearm) for the debuff glint."""
    P, j = c.P, c.j
    for side, key in (('r', 'fr'), ('l', 'fl')):
        r = c.e.get(key, 2.0)
        if arm_layer(c, side, True) != layer or r <= 0:
            continue
        hx, hy = j['hand_' + side]
        f.disc(hx, hy, r, P['glove']); f.dot(hx + 0.6, hy - 0.8, P.get('glove_hi', P['glove']))
    if c.tip is None:
        (ex, ey), (hx, hy) = j['elbow_r'], j['hand_r']; d = math.hypot(hx - ex, hy - ey) or 1.0
        c.tip = (hx + (hx - ex) / d * 2.0, hy + (hy - ey) / d * 2.0)


def bow(f, c, layer):
    """The ranger's bow (rig draw_bow) in the right hand: the stave's curve (wood, the grip darker) tilted by bow_a (0
    upright), its bulge foreshortened by bk and toward the screen side bs (default the unit's right: away from the
    body); the white string, pulled to the left hand by `pull` with a nocked arrow (wood, a steel head at the bow, red
    fletching at the hand) when `nock`; the arrow's head is the tip."""
    if layer != _wz(c, 'wz'):
        return
    P, j, e = c.P, c.j, c.e
    bx, by = j['hand_r']; r = math.radians(e.get('bow_a', 0.0)); ca, sa = math.cos(r), math.sin(r)
    bk, bs = e.get('bk', 1.0), e.get('bs', c.rs)

    def bp(al, out):
        out *= bs
        return (bx + out * ca - al * sa, by - al * ca + out * sa)
    pts = [bp(t, bk * 2.6 * math.cos(t / 9 * math.pi / 2) - 0.3) for t in [x * 0.5 for x in range(-18, 19)]]
    top, bot = bp(9, -0.3), bp(-9, -0.3)
    ph = j['hand_l'] if e.get('pull', 0.0) > 0 else None
    if ph:
        f.limb(top, ph, 0.3, rig.WHITE); f.limb(bot, ph, 0.3, rig.WHITE)
    else:
        f.limb(top, bot, 0.3, rig.WHITE)
    for i in range(len(pts) - 1):
        f.limb(pts[i], pts[i + 1], 0.8, P['wood'] if abs(i - 18) > 3 else P['wood_lo'])
    if e.get('nock') and ph:
        d = math.hypot(bx - ph[0], by - ph[1]) or 1.0; ux, uy = (bx - ph[0]) / d, (by - ph[1]) / d
        f.limb(ph, (bx + ux * 1.0, by + uy * 1.0), 0.4, P['wood'])
        f.dot(bx + ux * 1.8, by + uy * 1.8, P['metal_hi']); f.dot(bx + ux * 2.6, by + uy * 2.6, P['metal_hi'])
        f.dot(ph[0] - uy * 0.8, ph[1] + ux * 0.8, P['accent']); f.dot(ph[0] + uy * 0.8, ph[1] - ux * 0.8, P['accent'])
        c.tip = (bx + ux * 2.6, by + uy * 2.6)
    _regrip(f, c, layer, ('r', 'l') if ph else ('r',))


def _dagger(f, c, layer, side, ak, kk, zk, default):
    if layer != _wz(c, zk):
        return
    P = c.P; h = c.j['hand_' + side]; d = rig.udir(c.e.get(ak, default)); n = (-d[1], d[0])
    at = _along(h, d, c.e.get(kk, 1.0), n)
    f.limb(at(-1.5), at(0.4), 0.7, P['leather_lo'])
    f.limb(at(0.9, -1.5), at(0.9, 1.5), 0.5, P['trim'])
    f.limb(at(1.6), at(6.5), 0.6, P['metal_hi']); f.dot(*at(6.8), rig.WHITE)
    _regrip(f, c, layer, (side,))


def dagger(f, c, layer):
    """The rogue's main dagger (rig draw_weapon 'dagger': grip, a small guard, the blade and its white point) in the
    right hand along wa, foreshortened by wk, on layer wz."""
    _dagger(f, c, layer, 'r', 'wa', 'wk', 'wz', 180.0)


def off_dagger(f, c, layer):
    """The rogue's second dagger in the left hand along wb, foreshortened by wbk, on layer oz."""
    _dagger(f, c, layer, 'l', 'wb', 'wbk', 'oz', 180.0)


def mace(f, c, layer):
    """The priest's mace (rig draw_weapon 'mace'), gripped a third up the shaft: the shaft and pommel below the fist,
    the head with its highlight and four flanges, along wa, foreshortened by wk; the smaller fist round the shaft (and
    the left hand below it with grip2)."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; wa = c.e.get('wa', 0.0); d = rig.udir(wa)
    at = _along(h, d, c.e.get('wk', 1.0))
    f.limb(at(-4), at(7.5), 0.8, P['wood']); f.disc(*at(-4.4), 0.8, P['metal_lo'])
    cx, cy = at(8.6)
    f.disc(cx, cy, 2.2, P['metal']); f.dot(cx - 1, cy - 1, P['metal_hi'])
    for k in range(4):
        a = math.radians(wa + k * 90 + 45); f.dot(cx + math.sin(a) * 2.8, cy - math.cos(a) * 2.8, P['metal_lo'])
    if c.e.get('grip2'):
        _regrip(f, c, layer, ('l',))
    if arm_layer(c, 'r', True) <= layer:
        f.disc(h[0], h[1], 1.3, P.get('glove', P['leather'])); f.dot(h[0] + 0.6, h[1] + 0.4, P.get('glove_lo', P['leather_lo']))


def staff(f, c, layer):
    """The mage's staff (rig draw_weapon 'staff'), held near its middle: the shaft, its darker upper end, the gem with
    its glint and, at `glow`, the glow round it; along wa, foreshortened by wk."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 0.0))
    at = _along(h, d, c.e.get('wk', 1.0))
    f.limb(at(-11), at(10), 0.8, P['wood']); f.limb(at(8.5), at(10), 1.1, P['wood_lo'])
    gx, gy = at(11.8); glow = c.e.get('glow', 0.0)
    if glow > 0:
        f.disc(gx, gy, 2.2 + glow * 1.8, P['magic_lo'])
    f.disc(gx, gy, 1.6, P['magic']); f.dot(gx - 0.5, gy - 0.6, rig.WHITE)
    _regrip(f, c, layer, ('r', 'l') if c.e.get('grip2') else ('r',))


def _head_side(c, d):
    """The across-direction of a 2H head (its edge or striking face): toward the unit's right side, or the other with
    wn = -1."""
    n = (-d[1], d[0])
    if n[0] * c.rs < 0 or (abs(n[0]) < 1e-9 and n[1] > 0):
        n = (-n[0], -n[1])
    w = c.e.get('wn', 1.0)
    return (n[0] * w, n[1] * w)


def battle_axe(f, c, layer):
    """The battle axe in both hands (rig._battle_axe: the haft with its wrapped grip and butt cap, the bearded head
    with its honed edge and the back spike), along wa, foreshortened by wk; the head keeps its width. In `up` behind the
    body only the head past the body shows (the grip stays hidden, as the greatsword's)."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 0.0)); n = _head_side(c, d)
    at = _along(h, d, c.e.get('wk', 1.0), n)
    f.limb(at(-5), at(13.5), 0.75, P['leather']); f.limb(at(-5), at(-1), 0.75, P['leather_lo']); f.disc(*at(-5.4), 0.8, P['metal_lo'])
    f.poly([at(8.6, 0.8), at(7.0, 4.6), at(10.8, 5.6), at(14.6, 4.6), at(13.0, 0.8)], P['metal'])
    f.limb(at(7.0, 4.6), at(10.8, 5.6), 0.45, P['metal_hi']); f.limb(at(10.8, 5.6), at(14.6, 4.6), 0.45, P['metal_hi'])
    f.limb(at(9.4, 1.4), at(12.2, 1.4), 0.4, P['metal_lo'])
    f.poly([at(9.8, -0.6), at(11.0, -3.4), at(12.2, -0.6)], P['metal_lo'])
    f.dot(*at(14.0), P['metal_lo'])
    _regrip(f, c, layer, ('r', 'l'))


def warhammer(f, c, layer):
    """The warhammer in both hands (rig._warhammer: the haft, the square head with its striking face, the back and
    top spikes), along wa, foreshortened by wk; the head keeps its width."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 0.0)); n = _head_side(c, d)
    at = _along(h, d, c.e.get('wk', 1.0), n)
    f.limb(at(-5), at(12), 0.75, P['leather']); f.limb(at(-5), at(-1), 0.75, P['leather_lo']); f.disc(*at(-5.4), 0.8, P['metal_lo'])
    f.poly([at(10.0, -3.0), at(10.0, 3.8), at(13.4, 3.8), at(13.4, -3.0)], P['metal'])
    f.limb(at(10.0, 4.2), at(13.4, 4.2), 0.5, P['metal_hi'])
    f.limb(at(10.4, -0.4), at(13.0, -0.4), 0.4, P['metal_lo'])
    f.poly([at(10.6, -3.0), at(11.7, -5.4), at(12.8, -3.0)], P['metal_lo'])
    f.limb(at(13.4), at(15.0), 0.6, P['metal'])
    _regrip(f, c, layer, ('r', 'l'))


PARTS['head'].update(headband=headband, hood=hood, cowl=cowl, hat=hat)
PARTS['flag']['mask'] = mask
PARTS['torso'].update(bare=bare, leather=leather, robe=robe)
PARTS['legs']['robe'] = robe_skirt
PARTS['back'].update(quiver=quiver, cape=cape)
PARTS['weapon'].update(fists=fists, bow=bow, dagger=dagger, mace=mace, staff=staff, battle_axe=battle_axe, warhammer=warhammer)
PARTS['offhand']['dagger'] = off_dagger


# ------------------------------------------------------------------ the other classes' grips (DIR4-R2)
# Each grip keys idle and walk per view, its main attack (the side clip's frames and ms; DIR4 6.3: a lunge moves the
# body 1 to 2 px toward the facing, down the screen in `down` and up it in `up`; a motion along the facing axis shows
# foreshortened) and the WINDUP1 rows (K6: the windup hold is the attack's frame strike - 1; the heavy holds rest over
# the shoulder, decision "WINDUP1-R1 heavy hold, front and back"; stagger knocks the body 1 px away from the facing;
# dazed slumps with the weapon low). The weapon stays visible in both views: held at the side, raised, or showing past
# the body from behind (decision "Front and back views follow the side model").
def flip(kw, **over):
    """The `up` keys of `down` keys `kw` for a pose across the facing (holds, stagger, dazed): screen angles (wa, wb,
    bow_a) and screen offsets (dx, dy) mirrored, a lifted foot kept on the same screen side, the layer keys dropped
    (each view's defaults) unless given in `over`, which is laid over the result."""
    out = {k: v for k, v in kw.items() if k not in ('wz', 'oz', 'fore_z', 'arm_z')}
    for k in ('wa', 'wb'):
        if k in out:
            out[k] = (360.0 - out[k]) % 360.0
    for k in ('bow_a', 'dx', 'dy'):
        if k in out:
            out[k] = -out[k]
    if 'lift_l' in out:
        out['lift_r'] = out.pop('lift_l')
    out.update(over)
    return out


def _qarc(a, ctrl, b, n=14):
    """A swing's arc for the slash key: n points on the quadratic curve from `a` through control point `ctrl` to `b`."""
    return [((1 - t) ** 2 * a[0] + 2 * (1 - t) * t * ctrl[0] + t * t * b[0], (1 - t) ** 2 * a[1] + 2 * (1 - t) * t * ctrl[1] + t * t * b[1])
            for t in (i / (n - 1) for i in range(n))]


def _windups(attack, strike, holds, stagger, dazed, opts=None, up=None):
    """A grip's windups entry: holds, stagger and dazed given for `down`; `up` ({key: up keys}) overrides the flipped
    `down` keys per hold (by wind-up id), 'stagger' (a list) and 'dazed'."""
    up = up or {}
    w = dict(attack=attack, strike=strike,
             holds={'down': holds, 'up': {k: up.get(k, flip(v)) for k, v in holds.items()}},
             stagger={'down': stagger, 'up': up.get('stagger', [flip(k) for k in stagger])},
             dazed={'down': dazed, 'up': up.get('dazed', flip(dazed))})
    if opts:
        w['opts'] = opts
    return w


def _attack(keys_fn, ms, *sparks):
    """An attack entry's draw function: the keys of `keys_fn(view)` with the side clip's ms; a key's spark (a length
    along its weapon, see _tip) is drawn with sparks[fxk] (the key's fxk, default 0; default _spark)."""
    fns = sparks or (_spark,)

    def draw(u, view, stand_in=False):
        out = []
        for kw in keys_fn(view):
            kw = dict(kw)
            sp = kw.pop('spark', None); fn = fns[kw.pop('fxk', 0)]
            if sp is not None:
                kw['fx'] = fn(*_tip(u, view, kw, sp))
            out.append(pose_view(u, view, stand_in=stand_in, **kw))
        return out, list(ms)
    return draw


# Brawler (fists): the guard holds both fists before the chin (down: in front of the face; up: just showing beside the
# head); the jab drives the right fist 3 px toward the facing (down: toward the camera, the fist grown by 1 px; up: up
# beside the head, away from the camera), the left fist bobbing at the chin.
FZ = {'r': FRONT, 'l': FRONT}
FB = {'r': RAISED_BACK, 'l': RAISED_BACK}
B_GUARD = {'down': dict(hand_r=(0.5, -3.5), hand_l=(0.5, -3.5), fore_z=FZ),   # fists beside the jaw
           'up': dict(hand_r=(0.5, -2.5), hand_l=(0.5, -2.5), fore_z=FB)}


def _fist_spark(x, y):
    """The brawler's jab spark (the side clip's colours)."""
    return [(x, y, 1.6, (255, 255, 255)), (x - 2, y + 2, 0.7, (255, 220, 160)), (x + 2, y + 2, 0.7, (255, 220, 160))]


def brawler_jab_keys(view):
    """The keys of jab_<view> (frame 1, the fist cocked at the chest, is the `windup` hold)."""
    if view == 'down':
        return [dict(bob=1.2, hand_r=(0.5, -3.5), hand_l=(0.5, -3.8), fore_z=FZ),
                dict(bob=1.3, hand_r=(-0.5, 0.5), hand_l=(0.5, -2.6), fore_z=FZ),                           # cocked
                dict(bob=1.0, dy=0.8, hand_r=(-3.5, 1.5), fr=3.0, hand_l=(0.5, -3.8), fore_z=FZ, spark=('hand', 0.0, 2.5)),
                dict(bob=1.1, dy=0.8, hand_r=(-3.5, 1.5), fr=2.8, hand_l=(0.5, -3.5), fore_z=FZ),
                dict(bob=1.2, dy=0.3, hand_r=(-1.0, -1.0), hand_l=(0.5, -2.6), fore_z=FZ),
                dict(bob=1.0, hand_r=(0.5, -3.5), hand_l=(0.5, -3.8), fore_z=FZ)]
    return [dict(bob=1.2, hand_r=(0.5, -2.5), hand_l=(0.5, -2.8), fore_z=FB),
            dict(bob=1.3, hand_r=(-0.5, 0.5), hand_l=(0.5, -1.6), fore_z=FB),                                # cocked
            dict(bob=1.0, dy=-0.8, hand_r=(0.0, -5.5), hand_l=(0.5, -2.8), fore_z=FB, spark=('hand', 0.5, -2.5)),
            dict(bob=1.1, dy=-0.8, hand_r=(0.0, -5.5), hand_l=(0.5, -2.5), fore_z=FB),
            dict(bob=1.2, dy=-0.3, hand_r=(0.0, -1.5), hand_l=(0.5, -1.6), fore_z=FB),
            dict(bob=1.0, hand_r=(0.5, -2.5), hand_l=(0.5, -2.8), fore_z=FB)]


GRIPS['brawler'] = dict(
    idle={v: dict(B_GUARD[v], bob=1.0) for v in VIEWS},
    walk={v: dict(hand_r=(0.8, 8.5), hand_l=(0.8, 8.5), swing=1.5) for v in VIEWS},
    attack=('jab', _attack(brawler_jab_keys, [80, 60, 50, 100, 70, 100], _fist_spark)),
    windups=_windups(brawler_jab_keys, 2, {
        'windup_heavy': dict(bob=2.0, hand_r=(3.5, 0.5), hand_l=(0.5, -3.5), fore_z={'l': FRONT}),   # wound far back
        'windup_buff': dict(bob=1.2, head_dy=1.6, hand_r=(-3.0, 3.0), hand_l=(-3.0, 3.0), fore_z=FZ),
        'windup_debuff': dict(bob=1.2, dy=0.5, hand_r=(-3.0, 2.0), fr=2.6, hand_l=(0.5, -3.5), fore_z=FZ)},
        [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(3.5, -3.5), hand_l=(3.0, -2.0)),
         dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(3.0, 0.5), hand_l=(3.0, 2.0)),
         dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(1.0, 9.5), hand_l=(1.0, 10.0), fr=1.6, fl=1.6),
         dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(1.0, 10.0), hand_l=(1.0, 10.4), fr=0.0, fl=0.0)],
        dict(bob=2.0, head_dy=1.5, hand_r=(1.0, 10.2), hand_l=(1.0, 10.6), fr=0.0, fl=0.0), up={   # dazed: hands open
            'windup_heavy': dict(bob=2.0, hand_r=(3.5, 0.5), hand_l=(0.5, -2.5), fore_z={'l': RAISED_BACK}),
            'windup_buff': dict(bob=1.2, head_dy=1.6, hand_r=(-3.0, 3.0), hand_l=(-3.0, 3.0), fore_z=FB),
            'windup_debuff': dict(bob=1.2, dy=-0.5, hand_r=(0.0, -5.0), hand_l=(0.5, -2.5), fore_z=FB)}))


# Ranger (bow): idle and walk carry the bow low at the right side, its curve showing; the attack raises it before the
# chest (down) or forward over the right shoulder (up), nocks and draws to the chin, and looses a 1 x 5 px arrow down
# past the feet (down) or up above the head (up).
R_BOW = dict(bk=1.0, bow_a=15)
R_BOW_UP = dict(bk=1.0, bow_a=-10)


def ranger_attack_keys(view):
    """The keys of attack_<view> (frame 3, the full draw, is the `windup` hold)."""
    if view == 'down':
        return [dict(bob=0.5, hand_r=(-1.0, 5.0), bow_a=-10, bk=0.8),
                dict(bob=0.8, hand_r=(-1.0, 3.0), hand_l=(-3.5, 2.0), pull=0.2, nock=True, **R_BOW),
                dict(bob=1.0, hand_r=(-1.0, 3.0), hand_l=(-3.0, -0.5), pull=0.6, nock=True, **R_BOW),
                dict(bob=1.2, hand_r=(-1.0, 3.0), hand_l=(-2.5, -2.5), pull=1.0, nock=True, **R_BOW),   # full draw
                dict(bob=1.0, dy=-0.5, hand_r=(-1.0, 3.0), hand_l=(-1.5, -3.5), arrow=(22, 45, 1), **R_BOW),  # loosed
                dict(bob=0.8, dy=-0.4, hand_r=(-1.0, 3.2), hand_l=(-1.0, -3.0), **R_BOW),
                dict(bob=0.4, hand_r=(-1.0, 5.0), bow_a=-10, bk=0.8)]
    return [dict(bob=0.5, hand_r=(0.5, 3.0), bow_a=10, bk=0.8),
            dict(bob=0.8, hand_r=(0.5, -1.0), hand_l=(-2.5, 1.0), pull=0.2, nock=True, **R_BOW_UP),
            dict(bob=1.0, hand_r=(0.5, -1.0), hand_l=(-2.5, -1.0), pull=0.6, nock=True, **R_BOW_UP),
            dict(bob=1.2, hand_r=(0.5, -1.0), hand_l=(-2.0, -2.5), pull=1.0, nock=True, **R_BOW_UP),       # full draw
            dict(bob=1.0, dy=0.5, hand_r=(0.5, -1.0), hand_l=(-1.0, -3.5), arrow=(30, 6, -1), **R_BOW_UP),  # loosed
            dict(bob=0.8, dy=0.4, hand_r=(0.5, -0.8), hand_l=(-1.0, -3.0), **R_BOW_UP),
            dict(bob=0.4, hand_r=(0.5, 3.0), bow_a=10, bk=0.8)]


R_LOW = {'down': dict(hand_r=(3.0, 8.0), bow_a=-25), 'up': dict(hand_r=(3.0, 8.0), bow_a=25)}
GRIPS['ranger'] = dict(
    idle=R_LOW, walk={v: dict(R_LOW[v], swing=0.5) for v in VIEWS},
    attack=('attack', _attack(ranger_attack_keys, [100, 110, 90, 260, 50, 140, 120])),
    windups=_windups(ranger_attack_keys, 4, {
        'windup_heavy': dict(bob=2.4, dy=-0.5, hand_r=(-1.0, 0.5), hand_l=(-2.5, -4.5), pull=1.0, nock=True, bow_a=22, bk=1.0),
        'windup_buff': dict(bob=0.6, hand_r=(1.2, 8.5), bow_a=-30, hand_l=(1.5, -3.5)),
        'windup_debuff': dict(bob=1.0, dy=0.5, hand_r=(-1.5, 6.0), hand_l=(-3.0, 3.0), pull=0.5, nock=True, **R_BOW)},
        [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(4.0, 1.0), bow_a=-40, hand_l=(2.5, 4.0)),
         dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(3.0, 4.0), bow_a=-30, hand_l=(2.0, 6.0)),
         dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(1.0, 8.5), bow_a=-25, hand_l=(1.0, 8.5)),
         dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(1.2, 9.0), bow_a=-30, hand_l=(1.2, 9.0))],
        dict(bob=2.0, head_dy=1.5, hand_r=(1.2, 9.5), bow_a=-35, hand_l=(1.2, 9.0)), dict(motes='off'), up={
            'windup_heavy': dict(bob=2.4, dy=0.5, hand_r=(0.5, -2.5), hand_l=(-2.0, -4.0), pull=1.0, nock=True, bow_a=12, bk=0.6),
            'windup_debuff': dict(bob=1.0, dy=-0.5, hand_r=(0.5, 2.0), hand_l=(-2.5, 1.5), pull=0.5, nock=True, **R_BOW_UP)}))


# Rogue (two daggers): held low at the sides, the blades angled out past the hips, in both views; the attack stabs the
# right, then the left dagger toward the facing past the hip (down: toward the camera, foreshortened; up: away, the
# blade rising beside the hip), a spark each (the side clip's).
GZ = dict(wz=FRONT, oz=FRONT)


def _stab(x, y):
    return [(x, y, 1.2, (255, 255, 255))]


def _stab2(x, y):
    return [(x, y, 1.2, (255, 255, 255)), (x + 2, y - 1, 0.7, (255, 200, 200))]


def rogue_attack_keys(view):
    """The keys of attack_<view> (frame 0, crouched with the daggers drawn back low, is the `windup` hold)."""
    if view == 'down':
        return [dict(bob=1.6, hand_r=(0.5, 7.5), wa=200, hand_l=(0.5, 7.5), wb=160, **GZ),
                dict(bob=1.2, dy=1.0, hand_r=(-1.5, 6.0), wa=195, wk=0.7, hand_l=(0.5, 7.5), wb=160, spark=6.8, **GZ),
                dict(bob=1.4, dy=1.5, hand_r=(0.5, 6.5), wa=205, hand_l=(-1.5, 6.0), wb=165, wbk=0.7, spark=('l', 6.8), fxk=1, **GZ),
                dict(bob=1.0, dy=0.5, hand_r=(0.8, 7.0), wa=210, hand_l=(0.8, 7.0), wb=150, **GZ),
                dict(bob=0.8, hand_r=(1.0, 6.5), wa=215, hand_l=(1.0, 6.5), wb=145, **GZ)]
    return [dict(bob=1.6, hand_r=(0.5, 7.5), wa=160, hand_l=(0.5, 7.5), wb=200, **GZ),
            dict(bob=1.2, dy=-1.0, hand_r=(1.5, 4.5), wa=20, wk=0.7, hand_l=(0.5, 7.5), wb=200, spark=6.8, **GZ),
            dict(bob=1.4, dy=-1.5, hand_r=(0.5, 6.5), wa=155, hand_l=(1.5, 4.5), wb=340, wbk=0.7, spark=('l', 6.8), fxk=1, **GZ),
            dict(bob=1.0, dy=-0.5, hand_r=(0.8, 7.0), wa=150, hand_l=(0.8, 7.0), wb=210, **GZ),
            dict(bob=0.8, hand_r=(1.0, 6.5), wa=145, hand_l=(1.0, 6.5), wb=215, **GZ)]


G_READY = {'down': dict(bob=0.8, hand_r=(1.0, 8.5), wa=215, hand_l=(1.0, 8.5), wb=145, **GZ),
           'up': dict(bob=0.8, hand_r=(1.0, 8.5), wa=145, hand_l=(1.0, 8.5), wb=215, **GZ)}
G_STAGGER = [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(4.0, 2.0), wa=250, hand_l=(2.0, 6.0), wb=140, **GZ),
             dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(3.0, 4.5), wa=230, hand_l=(1.5, 7.0), wb=150, **GZ),
             dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(1.0, 8.5), wa=190, hand_l=(1.0, 8.5), wb=170, **GZ),
             dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(1.2, 9.0), wa=188, hand_l=(1.2, 9.0), wb=172, **GZ)]
G_DAZED = dict(bob=2.0, head_dy=1.5, hand_r=(1.2, 9.5), wa=186, hand_l=(1.2, 9.5), wb=174, **GZ)
GRIPS['rogue'] = dict(
    idle=G_READY, walk={v: dict(G_READY[v], swing=0.8) for v in VIEWS},
    attack=('attack', _attack(rogue_attack_keys, [120, 50, 70, 120, 120], _stab, _stab2)),
    windups=_windups(rogue_attack_keys, 1, {
        'windup_heavy': dict(bob=3.4, hand_r=(-1.0, 2.0), wa=180, hand_l=(-1.0, 2.0), wb=180, **GZ),     # reversed, raised
        'windup_buff': dict(bob=0.6, hand_r=(1.2, 9.0), wa=185, hand_l=(-3.0, 2.5), wb=150, **GZ),
        'windup_debuff': dict(bob=1.0, dy=0.5, hand_r=(-1.5, 4.0), wa=225, wk=0.8, hand_l=(0.5, 7.5), wb=160, **GZ)},
        G_STAGGER, G_DAZED, up={
            'windup_heavy': dict(bob=3.4, hand_r=(1.0, 2.0), wa=180, hand_l=(1.0, 2.0), wb=180, **GZ),
            'windup_buff': dict(bob=0.6, hand_r=(1.2, 9.0), wa=175, hand_l=(-3.0, 2.5), wb=210, wz=FRONT, oz=RAISED_BACK,
                                fore_z={'l': RAISED_BACK}),
            'windup_debuff': dict(bob=1.0, dy=-0.5, hand_r=(1.5, 3.0), wa=25, wk=0.8, hand_l=(0.5, 7.5), wb=200, **GZ),
            'stagger': [flip(k, **GZ) for k in G_STAGGER], 'dazed': flip(G_DAZED, **GZ)}))


# Priest (mace): held upright at the right side (as the warrior's sword); the attack raises it over the head (down:
# behind the head; up: above it), smashes it down in front and flashes before the feet (down) or at both sides of the
# body (up, the mace hidden in front of him).
def _flash(x, y):
    """The priest's smash flash (the side clip's colours), centred at (x, y)."""
    return [(x, y, 2.0, (255, 255, 255)), (x - 3, y + 1, 1.0, rig.GOLDEN_HI), (x + 3, y + 1, 1.0, rig.GOLDEN_HI), (x, y - 2.5, 0.8, rig.GOLDEN)]


def priest_attack_keys(view):
    """The keys of attack_<view> (frame 1, the mace raised over the head, is the `windup` hold)."""
    if view == 'down':
        return [dict(bob=0.8, hand_r=(0.5, 1.0), wa=5, hand_l=(-1.0, 3.0)),
                dict(bob=1.2, dy=-0.5, hand_r=(0.5, -7.0), wa=10, wk=0.6, wz=RAISED_BACK, hand_l=(-1.5, -3.5)),    # raised
                dict(bob=0.8, dy=0.5, hand_r=(-2.5, 1.5), wa=195, wk=0.55, slash=_qarc((21, 2), (13, 14), (22, 30))),
                dict(bob=2.2, dy=1.5, hand_r=(-3.0, 8.5), wa=180, wk=0.9, hand_l=(-2.0, 7.5), fx=_flash(23, 42)),    # smash
                dict(bob=2.0, dy=1.5, hand_r=(-3.0, 8.5), wa=180, wk=0.9, hand_l=(-2.0, 7.5)),
                dict(bob=0.4, hand_r=(1.5, 7.0), wa=345)]
    hid = dict(fore_z={'r': RAISED_BACK, 'l': RAISED_BACK})
    return [dict(bob=0.8, hand_r=(0.5, 1.0), wa=355, hand_l=(-1.0, 3.0)),
            dict(bob=1.2, dy=0.5, hand_r=(0.5, -7.0), wa=350, wk=0.6, wz=TOP, hand_l=(-1.5, -3.5)),                # raised
            dict(bob=0.8, dy=-0.5, hand_r=(-2.5, 1.5), wa=10, wk=0.5, slash=_qarc((27, 1), (35, 8), (28, 16))),
            dict(bob=2.2, dy=-1.5, hand_r=(-3.0, 8.5), wa=180, wk=0.9, hand_l=(-2.0, 7.5), fx=_flash(15, 41) + _flash(33, 41), **hid),
            dict(bob=2.0, dy=-1.5, hand_r=(-3.0, 8.5), wa=180, wk=0.9, hand_l=(-2.0, 7.5), **hid),
            dict(bob=0.4, hand_r=(1.5, 7.0), wa=15)]


P_READY = {'down': dict(hand_r=(1.5, 7.0), wa=345), 'up': dict(hand_r=(1.5, 7.0), wa=15)}
GRIPS['priest'] = dict(
    idle=P_READY, walk={v: dict(P_READY[v], swing=0.8) for v in VIEWS},
    attack=('attack', _attack(priest_attack_keys, [120, 180, 50, 150, 110, 120])),
    windups=_windups(priest_attack_keys, 2, {
        'windup_heavy': dict(bob=2.4, dy=-0.5, hand_r=(1.5, -3.0), wa=320, wk=0.9, grip2=2.5, wz=RAISED_BACK),  # over the shoulder
        'windup_buff': dict(bob=0.6, hand_r=(-2.0, 4.0), wa=0, hand_l=(0.5, -4.0)),
        'windup_debuff': dict(bob=1.0, dy=0.5, hand_r=(-1.0, 3.0), wa=230, wk=0.7, hand_l=(2.5, 4.0))},
        [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(5.0, 1.0), wa=300, hand_l=(1.0, 7.0)),
         dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(4.0, 3.0), wa=320, hand_l=(1.2, 8.0)),
         dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(1.5, 8.5), wa=195, wk=0.8, hand_l=(1.2, 8.5)),
         dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(1.5, 9.0), wa=190, wk=0.8, hand_l=(1.2, 9.0))],
        dict(bob=2.0, head_dy=1.5, hand_r=(1.5, 9.5), wa=185, wk=0.8, hand_l=(1.2, 9.0)), up={
            'windup_buff': dict(bob=0.6, hand_r=(-1.0, 3.0), wa=5, hand_l=(0.5, -4.0), fore_z={'r': RAISED_BACK}),
            'windup_debuff': dict(bob=1.0, dy=-0.5, hand_r=(0.5, 2.0), wa=40, wk=0.6, hand_l=(2.5, 4.0))}))


# Mage (staff): upright at the right side; the attack raises it and charges the gem, then points it toward the facing
# (down: the gem before the chest, foreshortened; up: forward over the right shoulder) and the bolt leaves down past
# the feet (down) or up beside the hat (up).
def _bolt(x, y, sign):
    """The mage's bolt (the side clip's colours) at (x, y), flying down (sign 1) or up (-1), its trail behind it."""
    return [(x, y, 2.0, (150, 220, 255)), (x, y - sign * 2.5, 1.3, (80, 140, 230)), (x, y - sign * 4.5, 0.8, (80, 140, 230)),
            (x, y + sign * 0.5, 0.8, (255, 255, 255))]


def mage_attack_keys(view):
    """The keys of attack_<view> (frame 2, the gem charged, is the `windup` hold)."""
    if view == 'down':
        return [dict(bob=0.4, hand_r=(0.5, 4.5), wa=0, hand_l=(-2.0, 5.0)),
                dict(bob=0.8, dy=-0.3, hand_r=(0.0, 1.5), wa=0, glow=0.4, hand_l=(-2.0, 3.5)),
                dict(bob=1.0, dy=-0.5, hand_r=(-0.5, 0.0), wa=0, glow=1.0, hand_l=(-2.0, 2.5)),                    # charged
                dict(bob=0.8, dy=0.8, hand_r=(-3.0, 3.0), wa=180, wk=0.3, glow=0.6, hand_l=(-2.5, 4.5), fx=_bolt(23, 36, 1)),
                dict(bob=0.6, dy=0.6, hand_r=(-3.0, 3.5), wa=180, wk=0.3, hand_l=(-2.5, 4.5), fx=_bolt(23, 44, 1)),
                dict(bob=0.3, hand_r=(1.0, 5.5), wa=0)]
    return [dict(bob=0.4, hand_r=(0.5, 4.5), wa=0, hand_l=(-2.0, 5.0)),
            dict(bob=0.8, dy=0.3, hand_r=(0.0, 1.5), wa=0, glow=0.4, hand_l=(-2.0, 3.5)),
            dict(bob=1.0, dy=0.5, hand_r=(-0.5, 0.0), wa=0, glow=1.0, hand_l=(-2.0, 2.5)),                          # charged
            dict(bob=0.8, dy=-0.8, hand_r=(0.0, 1.5), wa=10, wk=0.5, glow=0.6, hand_l=(-2.0, 3.5), fx=_bolt(30, 6, -1)),
            dict(bob=0.6, dy=-0.6, hand_r=(0.0, 1.5), wa=10, wk=0.5, hand_l=(-2.0, 3.5), fx=_bolt(31, 2, -1)),
            dict(bob=0.3, hand_r=(1.0, 5.5), wa=0)]


M_READY = {v: dict(hand_r=(1.0, 5.5), wa=0) for v in VIEWS}
GRIPS['mage'] = dict(
    idle=M_READY, walk={v: dict(M_READY[v], swing=0.5) for v in VIEWS},
    attack=('attack', _attack(mage_attack_keys, [120, 140, 220, 60, 120, 140])),
    windups=_windups(mage_attack_keys, 3, {
        'windup_heavy': dict(bob=2.2, dy=-0.5, hand_r=(0.5, -3.0), wa=350, grip2=3.0, glow=0.6),
        'windup_buff': dict(bob=0.6, hand_r=(-2.5, 1.5), wa=0, glow=0.3),
        'windup_debuff': dict(bob=1.0, dy=0.5, hand_r=(-2.0, 3.5), wa=215, wk=0.6, glow=0.5)},
        [dict(bob=0.5, dy=-1.0, head_dy=-1.0, hand_r=(4.0, 1.0), wa=330),
         dict(bob=1.5, dy=-1.0, head_dy=-0.5, lift_l=1.5, hand_r=(3.0, 3.5), wa=345),
         dict(bob=2.5, dy=-0.5, head_dy=1.0, hand_r=(1.2, 8.0), wa=190, wk=0.7),
         dict(bob=2.2, dy=-1.0, head_dy=1.5, hand_r=(1.2, 8.5), wa=188, wk=0.7)],
        dict(bob=2.0, head_dy=1.5, hand_r=(1.2, 8.5), wa=192, wk=0.75),
        dict(motes='tip', steps={'windup_heavy': {'glow': (0.6, 0.8, 1.0, 0.8)}}), up={
            'windup_buff': dict(bob=0.6, hand_r=(0.5, 1.0), wa=0, glow=0.3),
            'windup_debuff': dict(bob=1.0, dy=-0.5, hand_r=(0.5, 1.5), wa=35, wk=0.6, glow=0.5)}))


# Warrior 2H, battle axe and warhammer (WAR2H1; DIR4 7.1 L5, DIR4-R2): the greatsword's grip keys (idle, walk and the
# held poses), with the side view's chop instead of the cut: raised over the right shoulder (the windup hold), swung
# down in front, the head landing low before the feet (down) or past the right hip (up) with the impact spark.
def chop_keys(view, tip):
    """The keys of the axe's and the hammer's attack_<view> (frame 1, raised over the right shoulder, is the `windup`
    hold); `tip`: the weapon's reach (rig.WEAPON_TIP) for the spark."""
    if view == 'down':
        keys = [dict(bob=1.2, dy=-0.5, hand_r=(-1.0, 1.0), wa=340, **G2),
                dict(bob=1.6, dy=-0.5, hand_r=(1.0, -5.0), wa=330, wk=0.75, wz=RAISED_BACK, **G2),             # raised
                dict(bob=0.8, dy=0.3, hand_r=(-2.0, 0.5), wa=195, wk=0.6, slash=_qarc((17, 2), (10, 14), (23, 32)), **G2),
                dict(bob=2.4, dy=1.5, hand_r=(-3.0, 5.5), wa=200, wk=0.9, spark=tip, **G2),                    # impact
                dict(bob=2.2, dy=1.5, hand_r=(-3.0, 5.5), wa=200, wk=0.9, **G2),
                dict(bob=1.2, dy=0.5, hand_r=(-3.0, 5.5), wa=300, wk=0.9, **G2)]
    else:
        hid = dict(fore_z=UP_FORE, **G2)
        keys = [dict(bob=1.2, dy=0.5, hand_r=(-1.0, 1.0), wa=20, **hid),
                dict(bob=1.6, dy=0.5, hand_r=(1.0, -5.0), wa=30, wk=0.75, **G2),                              # raised
                dict(bob=0.8, dy=-0.3, hand_r=(-2.0, 0.5), wa=345, wk=0.6, slash=_qarc((31, 2), (38, 12), (27, 22)), **hid),
                dict(bob=2.4, dy=-1.5, hand_r=(-3.0, 6.0), wa=160, wk=0.9, spark=tip, **hid),                  # impact
                dict(bob=2.2, dy=-1.5, hand_r=(-3.0, 6.0), wa=160, wk=0.9, **hid),
                dict(bob=1.2, dy=-0.5, hand_r=(-3.0, 4.5), wa=60, wk=0.9, **hid)]
    keys.append(dict(W2_READY[view], bob=0.3))
    return keys


def _chop(weapon):
    def keys(view):
        return chop_keys(view, rig.WEAPON_TIP[weapon])
    return dict(attack=('attack', _attack(keys, [120, 190, 45, 150, 120, 110, 110], _impact)),
                windups=dict(W2_WINDUPS, attack=keys))


GRIPS['warrior/2h']['weapons'] = {'battle_axe': _chop('battle_axe'), 'warhammer': _chop('warhammer')}


# ------------------------------------------------------------------ the variants' parts (DIR4-R3)
# Decision "Front and back views follow the side model" (user 2026-10-29) on the goblin and named bodies: the goblin
# head is bald as in the side view (no hair stand-in), its long ears swept out and up, yellow eyes, the hooked nose,
# the heavy jaw and the tusks; each hat is the side view's (band, bandana, hood, mohawk, skull, horns, crown); the
# named humans keep their hair, kettle, cap, beard and braid. The buckler (DIR4-R1F, user 2026-10-29) and the hand axe
# and double axe are the side rig's drawings.
def _goblin_ears(f, c):
    P = c.P; hx, hy = c.j['head']
    for s in (-1, 1):  # swept back and up: from the front and the back they stand out to both sides
        for k in range(7):
            f.disc(hx + s * (2.8 + k * 0.95), hy - 0.4 - k * 0.45, 1.4 - k * 0.14, P['skin'] if k < 5 else P['skin_lo'])
        f.dot(hx + s * 3.6, hy - 0.6, P['skin_lo'])


def goblin(f, c, layer):
    """The goblin head (rig 'goblin'): a bald, big skull and the long ears standing out to both sides. down: the brow,
    two yellow eyes, the hooked nose, the heavy jaw of the underbite and two tusks; up: the back of the skull and the
    nape. Under the goblin hood (hat 'hood') the hood goes on first and the ears poke out through it."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; hood = c.u.get('hat') == 'hood'
    if hood:
        low = 3.6 if c.view == 'down' else 6.0
        f.poly([(hx - 3.8, hy + 0.2), (hx + 3.8, hy + 0.2), (hx + 2.6, hy + low), (hx - 2.6, hy + low)], P['main_lo'])
        f.disc(hx, hy - 0.5, 3.9, P['main'])
    _goblin_ears(f, c)
    if hood and c.view == 'up':
        f.rect(hx, hy - 3.6, hx, hy + 1.0, P['main_lo'])
        return
    if hood:
        f.disc(hx, hy + 0.6, 2.7, P['skin'])
    else:
        f.disc(hx, hy, 3.4, P['skin'])
    if c.view == 'down':
        f.rect(hx - 2.2, hy - 1.6, hx + 2.2, hy - 1.6, P['skin_lo'])                         # brow
        f.dot(hx - 1.2, hy - 0.6, OUT if c.hurt else P['eye']); f.dot(hx + 1.2, hy - 0.6, OUT if c.hurt else P['eye'])
        f.rect(hx - 0.5, hy - 0.2, hx + 0.5, hy + 1.0, P['skin']); f.dot(hx, hy + 1.2, P['skin_lo'])  # hooked nose
        f.rect(hx - 2.6, hy + 1.8, hx + 2.6, hy + 3.2, P['skin_lo'])                         # heavy jaw
        f.dot(hx - 1.6, hy + 1.8, rig.WHITE); f.dot(hx + 1.6, hy + 1.8, rig.WHITE)          # tusks
    elif not hood:
        f.rect(hx - 2, hy + 2.6, hx + 2, hy + 3.0, P['skin_lo'])                             # the nape


def g_band(f, c, layer):
    """A goblin's headband (rig hat 'band') with its trailing tails: across the brow and round the back, its knot and
    tails at the back (their ends past the unit's left side from the front)."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; t = c.tails; k = _side(c, 'l')
    f.rect(hx - 3.4, hy - 2.6, hx + 3.4, hy - 1.9, P['accent'])
    if c.view == 'down':
        for i in range(2):
            f.dot(hx + k * (3.8 + i * 0.9), hy - 2.0 + i * 0.7 + math.sin(t + i) * 0.5, P['accent'] if i < 1 else P['accent_lo'])
    else:
        f.dot(hx, hy - 2.2, P['accent_lo'])
        for i in range(3):
            f.dot(hx + k * (0.6 + 0.5 * i) + math.sin(t + i) * 0.4, hy - 1.4 + i * 0.9, P['accent'] if i < 2 else P['accent_lo'])


def g_bandana(f, c, layer):
    """A goblin's bandana (rig hat 'bandana'): cloth tied over the skull down to the brow, its edge; from the back
    the knot and its hanging end."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; k = _side(c, 'l')
    f.poly([(hx - 3.5, hy - 1.2), (hx - 2.8, hy - 3.4), (hx, hy - 4.3), (hx + 2.8, hy - 3.4), (hx + 3.5, hy - 1.2)], P['accent'])
    f.rect(hx - 3.4, hy - 1.6, hx + 3.4, hy - 1.2, P['accent_lo'])
    if c.view == 'up':
        f.dot(hx, hy - 1.4, P['accent_lo'])
        f.dot(hx + k * 0.6, hy - 0.4 + math.sin(c.tails) * 0.4, P['accent']); f.dot(hx + k * 1.0, hy + 0.5, P['accent_lo'])


def g_hood(f, c, layer):
    """A goblin's hood (rig hat 'hood'; the goblin head draws the hood itself so the ears poke through): the hood's
    highlights on top."""
    if layer != HEAD:
        return
    hx, hy = c.j['head']
    f.dot(hx - 1.2, hy - 3.4, c.P['main_hi']); f.dot(hx - 0.2, hy - 3.9, c.P['main_hi'])


def g_mohawk(f, c, layer):
    """The goblin berserker's mohawk (rig hat 'mohawk'): a crest of spikes front to back, alternating the hair and its
    shade; seen end-on from the front and the back, a fan of thin spikes over the skull's middle."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    for k in range(5):
        d = k - 2
        f.limb((hx + d * 0.3, hy - 2.8), (hx + d * 0.75, hy - 5.8 + abs(d) * 0.7), 0.5, P['hair'] if k % 2 == 0 else P['hair_lo'])
    f.rect(hx - 0.5, hy - 3.2, hx + 0.5, hy - 1.8, P['hair_lo'])                             # the shaved stripe's root


def g_skull(f, c, layer):
    """The goblin priest's skull cap (rig hat 'skull'): an animal skull worn over the head, its horns sweeping up and
    out, a feather at the left. down: the skull's face (eye sockets, the snout over the brow); up: its back."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; k = _side(c, 'l')
    for s in (-1, 1):
        f.limb((hx + s * 2.0, hy - 3.4), (hx + s * 4.0, hy - 5.6), 0.6, P['bone']); f.dot(hx + s * 4.3, hy - 6.1, P['bone_lo'])
    f.limb((hx + k * 2.6, hy - 2.2), (hx + k * 4.8, hy - 1.0), 0.5, P['accent'])
    f.disc(hx, hy - 2.4, 2.9, P['bone'])
    if c.view == 'down':
        f.dot(hx - 1.0, hy - 2.6, OUT); f.dot(hx + 1.0, hy - 2.6, OUT)
        f.rect(hx - 0.8, hy - 1.8, hx + 0.8, hy - 1.0, P['bone']); f.dot(hx, hy - 1.0, P['bone_lo'])
    else:
        f.dot(hx - 0.6, hy - 3.6, P['bone_lo']); f.dot(hx + 0.8, hy - 2.0, P['bone_lo'])


def g_horns(f, c, layer):
    """The Ash Warchief's horned iron cap (rig hat 'horns'): the cap over the skull, its rim and highlight, two bone
    horns curving out and up."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    for s in (-1, 1):
        f.limb((hx + s * 2.2, hy - 2.8), (hx + s * 4.6, hy - 4.6), 0.75, P['bone'])
        f.limb((hx + s * 4.6, hy - 4.6), (hx + s * 4.4, hy - 7.0), 0.55, P['bone'] if s < 0 else P['bone_lo'])
    f.poly([(hx - 3.6, hy - 1.0), (hx - 2.9, hy - 3.4), (hx, hy - 4.5), (hx + 2.9, hy - 3.4), (hx + 3.6, hy - 1.0)], P['metal'])
    f.rect(hx - 3.6, hy - 1.6, hx + 3.6, hy - 1.0, P['metal_lo']); f.dot(hx - 0.8, hy - 3.6, P['metal_hi'])


def g_crown(f, c, layer):
    """Grakka's crown (rig hat 'crown'): an iron band with a crown of rusted nails, the gold stud at the front."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    f.rect(hx - 3.3, hy - 2.9, hx + 3.3, hy - 1.9, P['metal_lo'])
    for k in range(6):
        x0 = hx - 2.9 + k * 1.16
        f.limb((x0, hy - 2.9), (x0 + 0.3 * (k - 2.5) / 2.5, hy - 5.2 - (0.6 if k in (2, 3) else 0)), 0.4,
               P['rust'] if k % 2 else P['metal'])
    if c.view == 'down':
        f.dot(hx, hy - 2.4, P['trim'])


def _human_hair(f, c, top=True):
    """The named humans' hair (rig draw_head 'bare'): down, over the brow and short at the temples; up, over the back
    of the head to the nape, a few strands in the hair's shade when it has one."""
    P = c.P; hx, hy = c.j['head']; lo = P.get('hair_lo', P['hair'])
    if c.view == 'down':
        if top:
            f.rect(hx - 3, hy - 3.6, hx + 3, hy - 2.2, P['hair'])
        f.rect(hx - 3.5, hy - 2.6, hx - 2.6, hy + 0.4, P['hair']); f.rect(hx + 2.6, hy - 2.6, hx + 3.5, hy + 0.4, P['hair'])
    else:
        f.disc(hx, hy - 0.6, 3.4, P['hair']); f.rect(hx - 3.4, hy - 0.6, hx + 3.4, hy + 0.8, P['hair'])
        f.dot(hx - 1.4, hy - 1.0, lo); f.dot(hx + 1.2, hy - 0.2, lo); f.dot(hx - 0.2, hy + 0.4, lo)
        f.dot(hx - 3.6, hy + 0.2, P['skin_lo']); f.dot(hx + 3.6, hy + 0.2, P['skin_lo'])    # ears


def _human_face(f, c):
    P = c.P; hx, hy = c.j['head']
    f.disc(hx, hy, 3.4, P['skin'])
    if c.view == 'down':
        f.dot(hx - 1, hy + 0.2, OUT if c.hurt else P['eye']); f.dot(hx + 1, hy + 0.2, OUT if c.hurt else P['eye'])
        f.rect(hx - 0.5, hy + 2.0, hx + 0.5, hy + 2.0, P['skin_lo'])


def bare_head(f, c, layer):
    """A named human's bare head (rig 'bare'): the face and the hair."""
    if layer == HEAD:
        _human_face(f, c); _human_hair(f, c)


def kettle(f, c, layer):
    """Brask's iron kettle hat (rig 'kettle') over his hair: the dome with its highlight and shade, the wide brim."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    _human_face(f, c); _human_hair(f, c, top=False)
    f.rect(hx - 2.8, hy - 5.0, hx + 2.8, hy - 2.6, P['metal']); f.dot(hx - 1.2, hy - 4.4, P['metal_hi']); f.dot(hx + 2.2, hy - 3.2, P['metal_lo'])
    f.rect(hx - 4.8, hy - 2.6, hx + 4.8, hy - 1.9, P['metal_lo'])


def cap(f, c, layer):
    """Hest's leather cap (rig 'cap') over his hair: the crown with its highlight and, from the front, the visor."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    _human_face(f, c); _human_hair(f, c, top=False)
    f.rect(hx - 3.2, hy - 4.2, hx + 3.2, hy - 2.0, P['cap']); f.dot(hx - 0.8, hy - 3.8, P.get('cap_hi', P['cap']))
    if c.view == 'down':
        f.rect(hx - 3.0, hy - 2.4, hx + 3.0, hy - 1.8, P['cap_lo'])
    else:
        f.rect(hx - 3.2, hy - 2.2, hx + 3.2, hy - 2.0, P['cap_lo'])


def beard(f, c, layer):
    """A named human's beard (rig flag 'beard'): down, over the jaw and the chin with the sideburns and the mouth's
    line; up, its edge showing at both sides of the jaw."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']
    if c.view == 'down':
        f.rect(hx - 3.0, hy - 0.4, hx - 2.4, hy + 1.6, P['beard']); f.rect(hx + 2.4, hy - 0.4, hx + 3.0, hy + 1.6, P['beard'])
        f.rect(hx - 2.6, hy + 1.2, hx + 2.6, hy + 3.8, P['beard']); f.dot(hx, hy + 4.4, P['beard'])
        f.rect(hx - 1, hy + 1.8, hx + 1, hy + 1.8, OUT)
    else:
        f.dot(hx - 3.2, hy + 2.0, P['beard']); f.dot(hx + 3.2, hy + 2.0, P['beard'])


def braid(f, c, layer):
    """Orsa's braid (rig flag 'braid'): hanging down the back from the nape, its plaits alternating the hair and its
    shade; from the front its end shows past the left shoulder."""
    if layer != HEAD:
        return
    P = c.P; hx, hy = c.j['head']; lo = P.get('hair_lo', P['hair'])
    if c.view == 'up':
        for k in range(5):
            f.disc(hx + math.sin(c.tails + k) * 0.2, hy + 1.2 + k * 1.3, 1.0 - k * 0.08, P['hair'] if k % 2 == 0 else lo)


def pauldron(f, c, layer):
    """The Ash Warchief's shoulder plate (rig flag 'pauldron', the side view's near shoulder): an iron plate on the
    right shoulder, on that arm's layer."""
    if arm_layer(c, 'r') != layer:
        return
    P = c.P; sx, sy = c.j['sh_r']; out = 1.0 if sx > CX else -1.0
    f.disc(sx + out * 0.3, sy + 0.2, 2.6, P['metal'])
    f.rect(sx + out * 0.3 - 2, sy + 2.0, sx + out * 0.3 + 2, sy + 2.0, P['metal_lo'])
    f.dot(sx + out * 0.3 - 0.8, sy - 0.9, P['metal_hi'])


def hand_axe(f, c, layer):
    """The hatchet in the right hand (rig._hand_axe: a short haft, its grip, a small bearded head and the back spur),
    along wa, foreshortened by wk; the head keeps its width, its edge to the unit's right side."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 0.0)); n = _head_side(c, d)
    at = _along(h, d, c.e.get('wk', 1.0), n)
    f.limb(at(-2.5), at(9.5), 0.7, P['leather']); f.limb(at(-2.5), at(0.5), 0.7, P['leather_lo'])
    f.poly([at(6.4, 0.6), at(5.2, 3.6), at(7.8, 4.3), at(10.2, 3.6), at(9.2, 0.6)], P['metal'])
    f.limb(at(5.2, 3.7), at(7.8, 4.4), 0.45, P['metal_hi']); f.limb(at(7.8, 4.4), at(10.2, 3.7), 0.45, P['metal_hi'])
    f.poly([at(7.2, -0.6), at(7.9, -1.9), at(8.6, -0.6)], P['metal_lo'])
    _regrip(f, c, layer, ('r',))


def double_axe(f, c, layer):
    """Grakka's two-handed double-bitted axe (rig._double_axe: two broad crescent heads, honed edges, the socket and
    the top spike), along wa, foreshortened by wk; both heads keep their width."""
    if layer != _wz(c, 'wz'):
        return
    P = c.P; h = c.j['hand_r']; d = rig.udir(c.e.get('wa', 0.0)); n = _head_side(c, d)
    at = _along(h, d, c.e.get('wk', 1.0), n)
    f.limb(at(-5), at(15), 0.75, P['leather']); f.limb(at(-5), at(-1), 0.75, P['leather_lo']); f.disc(*at(-5.4), 0.8, P['metal_lo'])
    for k in (1.0, -1.0):
        f.poly([at(8.4, 0.8 * k), at(6.4, 5.4 * k), at(10.9, 6.4 * k), at(15.4, 5.4 * k), at(13.4, 0.8 * k)], P['metal'])
        f.limb(at(6.4, 5.4 * k), at(10.9, 6.5 * k), 0.5, P['metal_hi']); f.limb(at(10.9, 6.5 * k), at(15.4, 5.4 * k), 0.5, P['metal_hi'])
        f.limb(at(9.4, 2.2 * k), at(12.4, 2.2 * k), 0.4, P['metal_lo'])
    f.poly([at(9.6, -1.1), at(9.6, 1.1), at(12.2, 1.1), at(12.2, -1.1)], P['metal_lo'])
    f.limb(at(13.4), at(16.4), 0.6, P['metal'])
    _regrip(f, c, layer, ('r', 'l'))


def buckler(f, c, layer):
    """The buckler strapped over the left fist (rig draw_weapon 'buckler'), on its own outlined layer. down: its face
    (the iron rim, the leather face with its shaded far half, the glints and the boss); up: its back (the rim, the
    leather in shade, the strap), a little outward so it shows past the body."""
    if layer != _wz(c, 'oz'):
        return
    P = c.P; hx, hy = c.j['hand_l']
    cx, cy = hx + c.rs * (0.5 if c.view == 'down' else -1.5), hy
    g = F()
    g.disc(cx, cy, 3.6, P['metal'])
    if c.view == 'down':
        g.disc(cx, cy, 2.8, P['leather'])
        g.poly([(cx + 0.4, cy - 2.6), (cx + 2.6, cy - 1.0), (cx + 2.6, cy + 1.0), (cx + 0.4, cy + 2.6)], P['leather_lo'])
        g.dot(cx - 2.6, cy - 2.0, P['metal_hi']); g.dot(cx - 3.2, cy - 0.8, P['metal_hi'])
        g.disc(cx, cy, 1.1, P['metal']); g.dot(cx - 0.4, cy - 0.5, P['metal_hi'])
    else:
        g.disc(cx, cy, 2.8, P['leather_lo'])
        g.limb((cx - 2.2, cy), (cx + 2.2, cy), 0.6, P['leather'])
        g.dot(cx - 2.6, cy - 2.0, P['metal_hi'])
    g.outline(); f.im.alpha_composite(g.im); f.px = f.im.load()


PARTS['head'].update(goblin=goblin, bare=bare_head, kettle=kettle, cap=cap)
PARTS['hat'].update(band=g_band, bandana=g_bandana, hood=g_hood, mohawk=g_mohawk, skull=g_skull, horns=g_horns, crown=g_crown)
PARTS['flag'].update(beard=beard, braid=braid, pauldron=pauldron)
PARTS['weapon'].update(hand_axe=hand_axe, double_axe=double_axe)
PARTS['offhand']['buckler'] = buckler


# ------------------------------------------------------------------ the buckler grip's attack (DIR4-R3)
# The side clip's overhand cut (classes.warrior_buckler): the weapon raised behind the head (the windup hold), the
# buckler forward; cut down and across before the chest, the spark at the tip (down: toward the screen-left, before
# the chest; up: past the right hip); back to the ready stance (the weapon raised, as the kite look's).
def _buckler_spark(x, y):
    """The buckler look's cut spark (the side clip's colours)."""
    return [(x, y, 2.0, (255, 255, 255)), (x - 2, y + 1.5, 0.8, (255, 240, 200)), (x + 2, y + 1.5, 0.8, (255, 240, 200))]


def buckler_attack_keys(view, tip):
    """The keys of the buckler look's attack_<view> (frame 1, the weapon cocked behind the head, is the `windup`
    hold); `tip`: the weapon's reach (rig.WEAPON_TIP) for the spark."""
    if view == 'down':
        keys = [dict(bob=1.0, dy=-0.3, hand_r=(0.5, -4.0), wa=350, wk=0.8, wz=RAISED_BACK, hand_l=(1.0, 5.5)),
                dict(bob=1.4, dy=-0.5, hand_r=(-1.0, -6.5), wa=25, wk=0.7, wz=RAISED_BACK, hand_l=(1.0, 5.5)),      # cocked
                dict(bob=0.8, dy=0.5, hand_r=(-2.5, 1.0), wa=200, wk=0.6, slash=_qarc((25, 3), (14, 10), (17, 28)), hand_l=(1.0, 5.5)),
                dict(bob=2.0, dy=1.0, hand_r=(-3.0, 3.5), wa=230, wk=0.85, spark=tip, hand_l=(1.0, 6.0)),           # impact
                dict(bob=1.8, dy=1.0, hand_r=(-3.0, 3.5), wa=230, wk=0.85, hand_l=(1.0, 6.0)),
                dict(bob=1.0, dy=0.4, hand_r=(-1.0, 4.5), wa=300, hand_l=(1.0, 6.0))]
    else:
        keys = [dict(bob=1.0, dy=0.3, hand_r=(0.5, -4.0), wa=10, wk=0.8, wz=TOP, hand_l=(1.0, 5.5)),
                dict(bob=1.4, dy=0.5, hand_r=(-1.0, -6.5), wa=335, wk=0.7, wz=TOP, hand_l=(1.0, 5.5)),             # cocked
                dict(bob=0.8, dy=-0.5, hand_r=(-2.5, 1.0), wa=20, wk=0.5, slash=_qarc((23, 2), (34, 8), (30, 20)), hand_l=(1.0, 5.5)),
                dict(bob=2.0, dy=-1.0, hand_r=(0.5, 3.0), wa=135, wk=0.85, spark=tip, hand_l=(1.0, 6.0)),           # impact
                dict(bob=1.8, dy=-1.0, hand_r=(0.5, 3.0), wa=135, wk=0.85, hand_l=(1.0, 6.0)),
                dict(bob=1.0, dy=-0.4, hand_r=(1.0, 4.5), wa=60, hand_l=(1.0, 6.0))]
    keys.append(dict(WB_READY[view], bob=0.3))
    return keys


def _buckler_look(weapon):
    def keys(view):
        return buckler_attack_keys(view, rig.WEAPON_TIP[weapon])
    return dict(attack=('attack', _attack(keys, [110, 170, 45, 140, 110, 110, 110], _buckler_spark)),
                windups=dict(W1_WINDUPS, attack=keys, strike=2))


GRIPS['warrior/buckler']['weapons'] = {'sword': _buckler_look('sword'), 'hand_axe': _buckler_look('hand_axe')}
# the double axe's two broad heads would cover the face held as the greatsword's: its ready pose leans it further out
DA_READY = {'down': dict(W2_READY['down'], hand_r=(-1.5, 6.5), wa=318), 'up': W2_READY['up']}
GRIPS['warrior/2h']['weapons']['double_axe'] = dict(_chop('double_axe'), idle=DA_READY,
                                                    walk={v: dict(DA_READY[v], swing=0.3) for v in VIEWS})


# ------------------------------------------------------------------ the dash's carry (DASH1)
# Per grip, what the hands hold during dash_<view>: the warrior looks sweep the weapon out and back, foreshortened, with the
# shield or buckler kept up; the other classes run in their ready pose. dash() applies its own body keys over these.
W_SWEEP = {'down': dict(hand_r=(4.0, 6.0), wa=235, wk=0.75), 'up': dict(hand_r=(4.0, 6.0), wa=125, wk=0.75)}
GRIPS['warrior']['dash'] = {'down': dict(W_SWEEP['down'], hand_l=(-3.0, 3.0), oz=FRONT), 'up': dict(W_SWEEP['up'], hand_l=(4.0, 3.5))}
GRIPS['warrior/buckler']['dash'] = {'down': dict(W_SWEEP['down'], hand_l=(-2.0, 4.0), oz=FRONT), 'up': dict(W_SWEEP['up'], hand_l=(4.0, 3.5))}
# 2H from the back: carried higher and flatter, so the hilt stays off the legs even in the coil (pilot review, 2026-10-28)
GRIPS['warrior/2h']['dash'] = {'down': dict(W_SWEEP['down'], **G2), 'up': dict(hand_r=(4.5, 1.0), wa=105, wk=0.75, **G2)}
# the double axe's broad heads on its large bearer run off the frame at that reach: held closer and shorter
GRIPS['warrior/2h']['weapons']['double_axe']['dash'] = {'down': dict(W_SWEEP['down'], **G2), 'up': dict(hand_r=(2.5, 1.0), wa=105, wk=0.6, **G2)}
GRIPS['brawler']['dash'] = {v: dict(B_GUARD[v]) for v in VIEWS}
GRIPS['ranger']['dash'] = {v: dict(R_LOW[v]) for v in VIEWS}
GRIPS['rogue']['dash'] = {v: dict(G_READY[v]) for v in VIEWS}
GRIPS['priest']['dash'] = {v: dict(P_READY[v]) for v in VIEWS}
GRIPS['mage']['dash'] = {v: dict(M_READY[v]) for v in VIEWS}
