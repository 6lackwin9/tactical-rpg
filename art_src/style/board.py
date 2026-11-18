# Style-lock board (2026-11-17): python art_src/style/board.py OUT.png. Sections 1-5 are the options Tushar picked from.
import sprites as S, pals, glob
from PIL import Image, ImageDraw, ImageFont, ImageEnhance, ImageOps
import pathlib as _p
R = str(_p.Path(__file__).resolve().parents[2]) + '/'
GA = R + 'games/emberwake/assets/'
BG = (14, 15, 20, 255); INK = (217, 203, 180); DIM = (156, 163, 175); AMBER = (217, 119, 6)
try:
    F = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 18); FB = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 26); FS = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 14)
except OSError:
    F = FB = FS = ImageFont.load_default()

def sat(im, k):
    a = im.split()[3]; rgb = ImageEnhance.Color(im.convert('RGB')).enhance(k); rgb.putalpha(a); return rgb

def tallow(im):
    """-25 % saturation, shadows pulled to warm umber, highlights to parchment (the Tallow and Iron grade)"""
    im = sat(im, 0.75); a = im.split()[3]
    g = ImageOps.grayscale(im.convert('RGB'))
    tint = ImageOps.colorize(g, black=(18, 12, 12), mid=(120, 96, 78), white=(236, 224, 200))
    out = Image.blend(im.convert('RGB'), tint, 0.28); out.putalpha(a); return out

def frame(im, kind):
    w, h = im.size; pad = 10
    c = Image.new('RGBA', (w + 2 * pad, h + 2 * pad), BG); d = ImageDraw.Draw(c)
    c.alpha_composite(im, (pad, pad))
    if kind == 'brass':
        d.rectangle([pad - 4, pad - 4, w + pad + 3, h + pad + 3], outline=(150, 116, 60), width=3)
        for x, y in ((pad - 4, pad - 4), (w + pad - 3, pad - 4), (pad - 4, h + pad - 3), (w + pad - 3, h + pad - 3)):
            d.rectangle([x, y, x + 6, y + 6], fill=(196, 160, 90))
    elif kind == 'iron':
        d.rectangle([pad - 2, pad - 2, w + pad + 1, h + pad + 1], outline=(75, 85, 99), width=2)
        d.line([pad - 2, pad - 2, w + pad + 1, pad - 2], fill=(153, 27, 27), width=2)
    else:  # vignette, no frame
        m = Image.new('L', im.size, 255); md = ImageDraw.Draw(m)
        for k in range(24): md.rectangle([k, k, w - 1 - k, h - 1 - k], outline=round(255 * k / 24))
        v = Image.new('RGBA', im.size, BG); v.putalpha(ImageOps.invert(m)); c.alpha_composite(v, (pad, pad))
    return c

board = Image.new('RGBA', (1680, 2480), BG); d = ImageDraw.Draw(board)
def H(y, t, sub=''):
    d.text((40, y), t, font=FB, fill=AMBER)
    if sub: d.text((40, y + 34), sub, font=F, fill=DIM)
def L(x, y, t, f=None, c=INK): d.text((x, y), t, font=f or F, fill=c)

d.text((40, 24), 'Emberwake style lock board, 2026-11-17', font=FB, fill=INK)
L(40, 60, 'Pick one option per section. Letters match the cards in the thread.', c=DIM)

# 1 portraits: saturation
y = 110; H(y, '1. Portrait colour', 'Current founder portraits graded three ways (Day 1 re-renders come after the lock).')
P = [Image.open(GA + f'portraits/heroes/{n}.png').convert('RGBA') for n in ('kaelen', 'vesper', 'elyse')]
for i, (lab, fn) in enumerate((('P1  as painted', lambda im: im), ('P2  25 % less saturated', lambda im: sat(im, 0.75)), ('P3  Tallow and Iron grade', tallow))):
    x = 40 + i * 540; L(x, y + 70, lab)
    for j, im in enumerate(P):
        board.alpha_composite(fn(im).resize((168, 168), Image.NEAREST), (x + j * 172, y + 100))
# frames
y = 400; H(y, '2. Portrait frame', 'Kaelen at in-game size, P2 colour, on the panel colour.')
for i, (lab, k) in enumerate((('F1  brass corners', 'brass'), ('F2  thin iron, red top line', 'iron'), ('F3  no frame, soft vignette', 'none'))):
    x = 40 + i * 540; L(x, y + 70, lab); board.alpha_composite(frame(sat(P[0], 0.75), k), (x, y + 100))

# 3 sprites
y = 780; H(y, '3. Battle sprite paint', 'Kaelen (own head: shoulder-length blond hair) and a Castellan Watch footman, idle frame, 1x and 4x.')
PP = R + 'games/emberwake/assets/units/warrior/1h_polearm/palettes/'
k = S.base_unit('bare', braid=True, pal=dict(hair=(214, 170, 80), hair_lo=(160, 120, 50)))
w = S.base_unit('kettle')
tile = Image.open(sorted(glob.glob(R + 'art_src/sunk_ward/tiles/ground_flagstone_q*_2.png'))[-1]).convert('RGB').resize((96, 96), Image.LANCZOS)
tile = ImageEnhance.Brightness(tile).enhance(0.55)
opts = (('S1  today: class colours, only base slots swapped', S.today(k, PP + 'kaelen.json'), S.today(w, PP + 'castellan_watch.json')),
        ('S2  muted, full 3-tone ramp per slot', S.grim(k, pals.KAELEN_B), S.grim(w, pals.WATCH_B)),
        ('S3  low-key: S2 darker and greyer', S.grim(k, pals.KAELEN_C), S.grim(w, pals.WATCH_C)))
for i, (lab, a, b) in enumerate(opts):
    x = 40 + i * 540; L(x, y + 70, lab, FS)
    g = Image.new('RGBA', (96, 48)); g.paste(tile.crop((0, 0, 96, 48)))
    g.alpha_composite(a, (0, 0)); g.alpha_composite(b, (48, 0))
    board.alpha_composite(g, (x, y + 100))
    board.alpha_composite(g.resize((384, 192), Image.NEAREST), (x + 110, y + 100))

# 4 icons
y = 1110; H(y, '4. Icon template', 'Six skills each, at 32 px (battle bar) and 64 px (menus).')
red = ['bleeding_cut', 'assassinate', 'crushing_blow', 'firebomb', 'envenom', 'deadeye']
slate = ['aimed_bolt', 'billhook_hook', 'bind_parry', 'body_blow', 'anoint_flare', 'bodkin_twist']
flat = ['attack', 'defend', 'move', 'focus', 'support', 'target']
for i, (lab, names, root) in enumerate((('I1  painted, full tile on red', red, GA + 'skills/'), ('I2  slate tile, painted glyph', slate, GA + 'skills/'),
                                         ('I3  flat glyph, no tile', flat, R + 'kits/rpg/assets/skills/kinds/'))):
    x = 40 + i * 540; L(x, y + 70, lab)
    for j, n in enumerate(names):
        im = Image.open(root + n + '.png').convert('RGBA')
        board.alpha_composite(im.resize((32, 32), Image.LANCZOS), (x + j * 40, y + 100))
        board.alpha_composite(im.resize((64, 64), Image.NEAREST), (x + (j % 3) * 72, y + 145 + (j // 3) * 72))

# 5 battle scene
y = 1460; H(y, '5. In context: a battle on the Undercroft', 'Left: S2 paint. Right: S3 paint. Same backdrop, ground and UI colours.')
bd = Image.open(GA + 'backgrounds/battle/undercroft.png').convert('RGBA')
def scene(ka, wa):
    sc = Image.new('RGBA', (384, 200), BG)
    sc.alpha_composite(bd, (0, 0))
    gt = Image.open(sorted(glob.glob(R + 'art_src/sunk_ward/tiles/ground_flagstone_q*_2.png'))[-1]).convert('RGB').resize((128, 128), Image.LANCZOS)
    gt = ImageEnhance.Brightness(gt).enhance(0.5)
    for gx in range(0, 384, 32):
        for gy in range(52, 200, 32):
            sc.paste(gt.crop((gx % 128, gy % 128, gx % 128 + 32, gy % 128 + 32)), (gx, gy))
    dd = ImageDraw.Draw(sc)
    for gx in range(0, 384, 32): dd.line([gx, 52, gx, 200], fill=(0, 0, 0, 60))
    for gy in range(52, 200, 32): dd.line([0, gy, 384, gy], fill=(0, 0, 0, 60))
    sc.alpha_composite(ka, (60, 92)); sc.alpha_composite(ka.transpose(Image.FLIP_LEFT_RIGHT) if False else ka, (100, 124))
    for px, py in ((250, 84), (290, 116), (260, 148)):
        sc.alpha_composite(wa.transpose(Image.FLIP_LEFT_RIGHT), (px, py))
    dd.rectangle([0, 186, 384, 199], fill=(10, 11, 16, 240)); dd.line([0, 186, 384, 186], fill=(75, 85, 99))
    return sc
for i, (ka, wa) in enumerate(((opts[1][1], opts[1][2]), (opts[2][1], opts[2][2]))):
    board.alpha_composite(scene(ka, wa).resize((768, 400), Image.NEAREST), (40 + i * 808, y + 100))

# 6 ground tile
y = 2020; H(y, 'Reference: one ground tile at three darknesses (no pick needed; set by the colour pick above)')
for i, kf in enumerate((0.8, 0.55, 0.4)):
    board.paste(ImageEnhance.Brightness(Image.open(sorted(glob.glob(R + 'art_src/sunk_ward/tiles/ground_flagstone_q*_2.png'))[-1]).convert('RGB').resize((256, 256), Image.LANCZOS)).enhance(kf).crop((0, 0, 256, 128)).resize((512, 256), Image.NEAREST), (40 + i * 540, y + 60))
import sys; board.convert('RGB').save(sys.argv[1] if len(sys.argv) > 1 else 'style_board_v1.png')
print('ok')
