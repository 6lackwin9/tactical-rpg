import sys, json, colorsys, copy
import pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / 'tools/sprites'))
import classes, variants, rig
from PIL import Image
import pathlib as _p
R = str(_p.Path(__file__).resolve().parents[2]) + '/'

def hx(h): return tuple(int(h[i:i+2], 16) for i in (1, 3, 5))

def ramp(c, lo_v=0.62, hi_v=1.22, shift=0.035, sat_lo=1.1, sat_hi=0.85):
    """three-tone ramp: shadow cooler/darker, light warmer/brighter (hue shift)"""
    h, s, v = colorsys.rgb_to_hsv(*[x/255 for x in c])
    def mk(hh, ss, vv): return tuple(round(min(1, max(0, x))*255) for x in colorsys.hsv_to_rgb(hh % 1, min(1, ss), min(1, vv)))
    # shadow toward blue-violet (0.66), light toward amber (0.1)
    def toward(h0, tgt, k):
        d = ((tgt - h0 + 0.5) % 1) - 0.5
        return h0 + max(-k, min(k, d))
    return mk(toward(h, 0.68, shift), s*sat_lo, v*lo_v), c, mk(toward(h, 0.09, shift), s*sat_hi, min(1, v*hi_v))

def base_unit(head, **kw):
    u = variants.variant('warrior', head=head, **kw)
    return variants.look_unit(u, '1h_polearm')

def frame(u, clip='idle', i=0):
    A, T = classes.clips('warrior', unit=u)
    return A[clip][i]

def swap(im, pairs):
    im = im.copy(); px = im.load()
    m = {a: b for a, b in pairs}
    for y in range(im.height):
        for x in range(im.width):
            p = px[x, y]
            if p[3] and p[:3] in m: px[x, y] = m[p[:3]] + (p[3],)
    return im

def today(u, palfile):
    """what the engine does today: render with the class palette, then swap only the base slot colours"""
    slots = json.load(open(palfile))['slots']
    im = frame(u)
    pairs = [(tuple(u['pal'][k]), hx(v)) for k, v in slots.items() if k in u['pal']]
    return swap(im, pairs)

def grim(u, slots, **rk):
    """the proposal: every slot ships a full three-tone ramp (base, _lo, _hi) from the palette file"""
    u = copy.deepcopy(u)
    for k, v in slots.items():
        lo, b, hi = ramp(hx(v) if isinstance(v, str) else v, **rk)
        u['pal'][k] = b; u['pal'][k + '_lo'] = lo; u['pal'][k + '_hi'] = hi
    return frame(u)
