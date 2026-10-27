# The six party classes on the shared rig: outfit + standard weapon + palette, and their base clips.
import math, sys
import rig
from rig import pose, idle, walk, hurt, death, arc_pts, export

BASE = dict(leather=(122, 82, 51), leather_lo=(80, 52, 32), wood=(138, 98, 58), wood_lo=(96, 66, 40), trim=(224, 180, 60),
            metal=(154, 166, 180), metal_hi=(214, 224, 235), metal_lo=(92, 102, 120), skin=(217, 166, 121), skin_lo=(176, 126, 88),
            eye=(40, 30, 30), hair=(70, 48, 36), boot=(92, 62, 40), boot_lo=(64, 42, 28))
def pal(**kw): p = dict(BASE); p.update(kw); return p

BLUE, BLUE_HI = (110, 170, 230), (200, 230, 255)
GOLDEN, GOLDEN_HI = (240, 200, 90), (255, 240, 180)

UNITS = {
    'warrior': dict(leg=8.5, arm=5.5, tl=10.0, head='helm', torso='plate', legs='trousers', weapon='sword', offhand='shield',
                    pal=pal(main=(168, 48, 40), main_lo=(110, 30, 30), main_hi=(200, 80, 70), accent=(168, 48, 40), accent_lo=(110, 30, 30),
                            sleeve=(168, 48, 40), sleeve_lo=(110, 30, 30), fore=(154, 166, 180), fore_lo=(92, 102, 120),
                            leg=(154, 166, 180), leg_lo=(92, 102, 120), glove=(122, 82, 51), glove_lo=(80, 52, 32),
                            shield=(66, 92, 150), shield_lo=(44, 62, 108), emblem=(232, 226, 210))),
    'ranger': dict(leg=8.5, arm=5.5, tl=10.0, head='hood', torso='leather', legs='trousers', weapon='bow', back='quiver',
                   pal=pal(main=(72, 110, 62), main_lo=(46, 74, 40), main_hi=(110, 150, 90), accent=(190, 60, 50), accent_lo=(120, 40, 36),
                           sleeve=(72, 110, 62), sleeve_lo=(46, 74, 40), leg=(110, 86, 60), leg_lo=(78, 60, 42), glove=(80, 52, 32))),
    'brawler': dict(fbend=1, leg=8.8, arm=5.8, tl=10.0, head='headband', torso='bare', legs='trousers', weapon='fists',
                    pal=pal(skin=(196, 140, 98), skin_lo=(150, 100, 68), main=(196, 140, 98), main_lo=(150, 100, 68),
                            accent=(220, 120, 40), accent_lo=(160, 80, 26), sleeve=(172, 118, 80), sleeve_lo=(140, 92, 62),
                            fore=(236, 226, 206), fore_lo=(176, 166, 146), leg=(70, 70, 96), leg_lo=(48, 48, 70), hair=(30, 24, 22),
                            glove=(236, 226, 206), glove_lo=(214, 204, 184), glove_hi=(255, 250, 240), boot=(150, 100, 68), boot_lo=(120, 80, 54))),
    'priest': dict(leg=8.2, arm=5.4, tl=10.0, head='cowl', torso='robe', legs='robe', weapon='mace',
                   pal=pal(main=(226, 220, 200), main_lo=(176, 168, 148), main_hi=(250, 246, 232), accent=(220, 180, 60), accent_lo=(170, 130, 40),
                           sleeve=(226, 220, 200), sleeve_lo=(176, 168, 148), fore=(226, 220, 200), fore_lo=(176, 168, 148),  # robe sleeves to the wrist
                           leg=(176, 168, 148), leg_lo=(140, 132, 116), glove=(217, 166, 121))),
    'rogue': dict(leg=8.3, arm=5.4, tl=9.6, head='hood', mask=True, torso='leather', legs='trousers', weapon='dagger', offhand='dagger', back='cape',
                  pal=pal(main=(70, 62, 86), main_lo=(46, 40, 60), main_hi=(110, 100, 130), accent=(120, 40, 60), accent_lo=(80, 26, 42),
                          sleeve=(70, 62, 86), sleeve_lo=(46, 40, 60), leg=(52, 46, 62), leg_lo=(36, 32, 44), glove=(46, 40, 60), glove_lo=(36, 32, 44))),
    'mage': dict(leg=8.2, arm=5.3, tl=9.8, head='hat', torso='robe', legs='robe', weapon='staff',
                 pal=pal(main=(60, 76, 160), main_lo=(40, 50, 110), main_hi=(100, 120, 200), accent=(220, 180, 60), accent_lo=(170, 130, 40),
                         sleeve=(60, 76, 160), sleeve_lo=(40, 50, 110), leg=(40, 50, 110), leg_lo=(30, 36, 80), hair=(200, 196, 190),
                         magic=(150, 220, 255), magic_lo=(80, 140, 230), glove=(217, 166, 121))),
}


def barrier(k, hit=False, col=BLUE, hi=BLUE_HI, cx=24, cy=26):
    pts = arc_pts(cx, cy, 15, 40, 140, 14)
    out = [(x, y, 0.5 + 0.4 * k, col if (i + int(k * 4)) % 3 else hi) for i, (x, y) in enumerate(pts) if i % 2 == 0 or k > 0.6]
    if hit: out += [(38, 20, 1.4, (255, 255, 255)), (40, 18, 0.8, hi), (40, 23, 0.8, hi)]
    return out


HUNCH = dict(bob=1.8, lean=2.2, head_dy=1.8)   # rule: defensive stances hunch the body behind the guard
GUARDS = {
    'ranger': dict(bow_a=30, hand_f=(6, 2)),
    'priest': dict(wa=55, hand_f=(6, 1), hand_b=(5, 3)),
    'rogue': dict(wa=35, wb=60, hand_f=(6, -0.5), hand_b=(6, 2.5)),
    'mage': dict(wa=35, hand_f=(5, 2.5), hand_b=(4.5, 4.5), glow=0.3),
    'brawler': dict(hand_f=(4.5, -3.5), hand_b=(6, -1.5), fist_f=True, fist_b=True),
}


def defensive_set(u, g, A, T):
    def gp(k=0.0, **kw):
        p = dict(g); p.update(kw)
        for key, v in HUNCH.items(): p[key] = p.get(key, 0.0) + v * k
        p.setdefault('fb', (-6.5, 0)); p.setdefault('ff', (5, 0))
        return pose(u, **p)
    A['defend'] = [gp(0.0, fb=(-4.5, 0), ff=(4, 0)), gp(0.5), gp(1.0, fx=barrier(0.5)), gp(1.0, fx=barrier(0.8)), gp(1.0, fx=barrier(1.0))]
    T['defend'] = [90, 80, 100, 120, 200]
    fr = []
    for i in range(8):
        t = i / 8 * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t)
        fr.append(gp(1.0, bob=b * 0.5, fx=barrier(0.55 + 0.35 * b)))
    A['idle_guarded'], T['idle_guarded'] = fr, [150] * 8
    A['hurt_guarded'] = [gp(1.0, fx=barrier(1.0, True)), gp(1.0, dx=-2.2, bob=0.4, fb=(-7.5, 0), fx=barrier(0.9)),
                         gp(1.0, dx=-1.5, bob=0.2, fb=(-7, 0), fx=barrier(0.8)), gp(1.0, fx=barrier(0.8))]
    T['hurt_guarded'] = [60, 100, 110, 140]


def warrior_2h(u):
    """WAR2H1: the warrior's two-handed look (grip '2h', no shield): u['weapon'] is a rig.DRAWERS drawing; both hands
    hold the grip (pose grip2). Clips as the 1H look minus shield_bash: idle, walk, attack (an overhead chop),
    defend / idle_guarded / hurt_guarded (a weapon block: the haft upright before the face), hurt, death."""
    A, T = {}, {}
    def put(k, v): A[k], T[k] = v
    g2 = rig.GRIP2
    def p(**kw): return pose(u, grip2=g2, **kw)
    ready = dict(hand_f=(5, 4.5), wa=35, grip2=g2)
    put('idle', idle(u, **ready)); put('walk', walk(u, wa=40, hand_f=(5, 5), grip2=g2, swing=0.3))
    # attack: lift the weapon high behind the head, chop down and forward with the whole body, recover to guard
    tip = lambda x, y: [(x, y, 2.2, (255, 255, 255)), (x - 2.5, y + 1.5, 0.9, (255, 240, 200)), (x + 2, y + 1.5, 0.9, (255, 240, 200)),
                        (x - 1, y - 2.5, 0.7, (255, 220, 160))]
    st = dict(fb=(-5.5, 0), ff=(4.5, 0))
    put('attack', ([p(bob=1.2, lean=-1.0, wa=-55, hand_f=(1.5, -5), **st),
                    p(bob=1.6, lean=-2.5, dx=-1.5, wa=-95, hand_f=(-0.5, -6.5), fb=(-6.5, 0), ff=(4, 0)),                  # wind-up
                    p(bob=0.8, lean=2.0, dx=1.0, wa=60, hand_f=(8, -3), fb=(-6.5, 0), ff=(5.5, 0),
                      slash=arc_pts(23, 19, 17, -60, 58, 14)),                                                          # the swing
                    p(bob=2.4, lean=4.5, dx=3.0, wa=150, hand_f=(8, 5), fb=(-7, 0), ff=(6.5, 0), fx=tip(43, 40)),     # impact
                    p(bob=2.2, lean=4.0, dx=3.0, wa=152, hand_f=(8, 5.5), fb=(-7, 0), ff=(6.5, 0)),                   # hold
                    p(bob=1.2, lean=1.5, dx=1.0, wa=80, hand_f=(6, 4.5), fb=(-5.5, 0), ff=(5, 0)),
                    pose(u, bob=0.3, **ready)], [120, 190, 45, 150, 120, 110, 110]))
    if u['weapon'] == 'greatsword':
        # DIR4-R1 pilot review (user 2026-10-28): the greatsword cuts level at chest height instead of the chop: raised
        # to the shoulder, cocked back over it, a level sweep, the blade straight out at chest height with the tip
        # spark (the same frame count, ms and impact frame 3 as the chop, for FX1); the axe and hammer keep the chop
        put('attack', ([p(bob=1.2, lean=-0.5, wa=-35, hand_f=(2, 1.5), **st),
                        p(bob=1.6, lean=-2.0, dx=-1.0, wa=-70, hand_f=(0, -2), fb=(-6.5, 0), ff=(4, 0)),             # cocked
                        p(bob=1.0, lean=1.5, dx=0.5, wa=75, hand_f=(4, 1), fb=(-6.5, 0), ff=(5.5, 0),
                          slash=arc_pts(22, 50, 30, -25, 25, 14)),                                                    # the sweep
                        p(bob=1.8, lean=3.0, dx=2.0, wa=95, hand_f=(3, 2.5), fb=(-7, 0), ff=(6.5, 0), fx=tip(44.5, 21.5)),  # impact
                        p(bob=1.6, lean=3.0, dx=2.0, wa=95, hand_f=(3, 2.5), fb=(-7, 0), ff=(6.5, 0)),             # hold
                        p(bob=1.2, lean=1.5, dx=1.0, wa=60, hand_f=(5, 3.5), fb=(-5.5, 0), ff=(5, 0)),
                        pose(u, bob=0.3, **ready)], [120, 190, 45, 150, 120, 110, 110]))
    # weapon block: hunch behind the weapon held upright before the face, the barrier on its far side
    def sb(k, hit=False, cx=24, cy=22):
        return barrier(k, hit, cx=cx, cy=cy) if not hit else barrier(k, False, cx=cx, cy=cy) + [(37, 15, 1.0, BLUE_HI), (38, 24, 1.0, BLUE_HI)]
    gd = dict(bob=2.6, dx=-0.5, lean=1.5, head_dy=1.0, wa=8, hand_f=(8, 2), fb=(-6.5, 0), ff=(5, 0))
    put('defend', ([pose(u, **ready),
                    p(bob=1.6, dx=-0.5, lean=0.8, wa=20, hand_f=(6.5, 3.5), fb=(-6, 0), ff=(5, 0)),
                    p(**dict(gd, hand_f=(8.5, 1.5)), fx=[(33, 12, 1.2, (255, 255, 255))]),                 # weapon snaps up
                    p(fx=sb(0.5), **gd), p(fx=sb(0.8), **gd), p(fx=sb(1.0), **gd)], [90, 80, 60, 100, 120, 200]))
    fr = []
    for i in range(8):  # stance: guarded idle, weapon held up, barrier breathing on its far side
        t = i / 8 * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t)
        fr.append(p(fx=sb(0.55 + 0.35 * b), **dict(gd, bob=gd['bob'] + b * 0.5)))
    put('idle_guarded', (fr, [150] * 8))
    # hit on the weapon: spark on the haft, the arms give, he slides back but keeps the guard
    put('hurt_guarded', ([p(fx=sb(1.0) + [(33, 14, 2.4, (255, 255, 255)), (35, 11, 1.0, (255, 240, 200)), (35, 18, 1.0, (255, 240, 200))], **gd),
                          p(fx=sb(0.9, True), **dict(gd, dx=-2.4, bob=3.0, lean=0.6, hand_f=(7, 2.8), fb=(-7.5, 0))),
                          p(fx=sb(0.8), **dict(gd, dx=-1.8, bob=2.8, lean=1.0, hand_f=(7.5, 2.4), fb=(-7, 0))),
                          p(fx=sb(0.8), **gd)], [60, 100, 110, 140]))
    put('hurt', hurt(u, **ready)); put('death', death(u, drop=(31, 44, 95)))
    return A, T


def warrior_buckler(u):
    """WAR2H1: the warrior's 1H + buckler look (grip 'buckler'): a one-hand weapon (u['weapon']) and a small round
    buckler on the far fist, held forward like a duelist. Clips as the 1H + kite look: idle, walk, attack (an overhand
    cut), shield_bash (a buckler punch), defend / idle_guarded / hurt_guarded (the buckler raised before the face, the
    weapon drawn back low), hurt, death (the buckler stays strapped on)."""
    A, T = {}, {}
    def put(k, v): A[k], T[k] = v
    ready = dict(hand_f=(2.5, 6.5), wa=20, hand_b=(7, 5.5))  # buckler forward at the waist, weapon raised behind it
    put('idle', idle(u, **ready)); put('walk', walk(u, wa=25, hand_f=(2.5, 7), hand_b=(6, 6), swing=0.4))
    spark = lambda x, y, r=2.0: [(x, y, r, (255, 255, 255)), (x - 2, y + 1.5, 0.8, (255, 240, 200)), (x + 2, y + 1.5, 0.8, (255, 240, 200))]
    st = dict(fb=(-5.5, 0), ff=(4.5, 0))
    # attack: raise the weapon behind the head, buckler forward; cut down and across, then back to the ready stance
    put('attack', ([pose(u, bob=1.0, lean=-1.0, wa=-25, hand_f=(0.5, -5), hand_b=(6, 3), **st),
                    pose(u, bob=1.4, lean=-2.0, dx=-1.0, wa=-60, hand_f=(-1, -5.5), hand_b=(6.5, 2.5), fb=(-6.5, 0), ff=(4, 0)),
                    pose(u, bob=0.8, lean=2.5, dx=1.5, wa=70, hand_f=(9, -1.5), hand_b=(4, 5), fb=(-6.5, 0), ff=(5.5, 0),
                         slash=arc_pts(23, 20, 13, -50, 70, 12)),
                    pose(u, bob=2.0, lean=4.0, dx=2.5, wa=125, hand_f=(9.5, 5), hand_b=(3, 6), fb=(-7, 0), ff=(6.5, 0), fx=spark(43, 31)),
                    pose(u, bob=1.8, lean=3.8, dx=2.5, wa=128, hand_f=(9.5, 5.5), hand_b=(3, 6), fb=(-7, 0), ff=(6.5, 0)),
                    pose(u, bob=1.0, lean=1.5, dx=1.0, wa=60, hand_f=(6, 4.5), hand_b=(5, 4), fb=(-5.5, 0), ff=(5, 0)),
                    pose(u, bob=0.3, **ready)], [110, 170, 45, 140, 110, 110, 110]))
    # shield bash: weapon held back, punch the buckler straight out at face height
    put('shield_bash', ([pose(u, bob=1.4, lean=-1.5, dx=-1.0, wa=-30, hand_f=(-1, 4), hand_b=(4.5, 3.5), fb=(-6, 0), ff=(4, 0)),
                         pose(u, bob=1.8, lean=-2.0, dx=-1.5, wa=-40, hand_f=(-1.5, 4), hand_b=(3.5, 3), fb=(-6.5, 0), ff=(3.5, 0)),
                         pose(u, bob=1.2, lean=4.0, dx=3.0, wa=-30, hand_f=(1, 3.5), hand_b=(11, 0), fb=(-7.5, 0), ff=(7, 0), fx=spark(42, 18, 2.2)),
                         pose(u, bob=1.4, lean=4.0, dx=3.0, wa=-30, hand_f=(1, 3.5), hand_b=(10.5, 0.5), fb=(-7.5, 0), ff=(7, 0)),
                         pose(u, bob=0.8, lean=1.5, dx=1.0, wa=10, hand_f=(3, 5), hand_b=(6, 3), fb=(-5.5, 0), ff=(5, 0)),
                         pose(u, bob=0.3, **ready)], [100, 150, 45, 150, 110, 110]))
    # buckler block: the buckler up before the face, weapon drawn back low ready to riposte; the barrier on the buckler
    def sb(k, hit=False, cx=24, cy=20):
        return barrier(k, hit, cx=cx, cy=cy) if not hit else barrier(k, False, cx=cx, cy=cy) + [(37, 14, 1.0, BLUE_HI), (38, 22, 1.0, BLUE_HI)]
    gd = dict(bob=2.4, dx=-0.5, lean=1.5, head_dy=1.0, wa=-120, hand_f=(-0.5, 6), hand_b=(7, -1), fb=(-6.5, 0), ff=(5, 0))
    put('defend', ([pose(u, **ready),
                    pose(u, bob=1.4, dx=-0.5, lean=0.8, wa=-40, hand_f=(1.5, 5.5), hand_b=(6.5, 1.5), fb=(-6, 0), ff=(5, 0)),
                    pose(u, **dict(gd, hand_b=(7.5, -1.5)), fx=[(33, 13, 1.2, (255, 255, 255))]),            # buckler snaps up
                    pose(u, fx=sb(0.5), **gd), pose(u, fx=sb(0.8), **gd), pose(u, fx=sb(1.0), **gd)], [90, 80, 60, 100, 120, 200]))
    fr = []
    for i in range(8):  # stance: guarded idle, buckler up, barrier breathing on its face
        t = i / 8 * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t)
        fr.append(pose(u, fx=sb(0.55 + 0.35 * b), **dict(gd, bob=gd['bob'] + b * 0.5)))
    put('idle_guarded', (fr, [150] * 8))
    put('hurt_guarded', ([pose(u, fx=sb(1.0) + spark(32, 15, 2.4), **gd),
                          pose(u, fx=sb(0.9, True), **dict(gd, dx=-2.4, bob=2.8, lean=0.6, hand_b=(6, -0.2), fb=(-7.5, 0))),
                          pose(u, fx=sb(0.8), **dict(gd, dx=-1.8, bob=2.6, lean=1.0, hand_b=(6.5, -0.6), fb=(-7, 0))),
                          pose(u, fx=sb(0.8), **gd)], [60, 100, 110, 140]))
    put('hurt', hurt(u, **ready)); put('death', death(u, drop=(30, 45, 92)))
    return A, T


def clips(name, variant='', unit=None):
    # unit: a body variant of class `name` (tools/sprites/variants.py: goblins, named bodies); default the class's own
    u = unit or UNITS[name]; A, T = {}, {}
    def put(k, v): A[k], T[k] = v
    if name == 'warrior' and u.get('grip') == '2h':  # WAR2H1: the two-handed look (the class base keeps 1H + shield)
        return warrior_2h(u)
    if name == 'warrior' and u.get('grip') == 'buckler':  # WAR2H1: one hand + buckler
        return warrior_buckler(u)
    if name == 'warrior':  # one-handed sword, kite shield on the far arm
        # variant: '' = current, 'A' = shield carried lower, 'B' = hunched behind the shield
        def wp(**kw):
            hb = kw.get('hand_b')
            raised = hb is not None and hb[1] < 3 and hb[0] >= 5
            if variant == 'A' and raised:
                kw['hand_b'] = (hb[0], hb[1] + 2.8)
            if variant == 'B' and raised:
                kw['bob'] = kw.get('bob', 0) + 1.8; kw['lean'] = kw.get('lean', 0) + 2.2; kw['head_dy'] = kw.get('head_dy', 0) + 1.8
                kw['hand_b'] = (hb[0] - 0.5, hb[1] + 2.0)
                if 'hand_f' in kw and kw['hand_f'] is not None: kw['hand_f'] = (kw['hand_f'][0], kw['hand_f'][1] + 0.5)
            return pose(u, **kw)
        ready = dict(hand_b=(3.5, 5), wa=25)
        put('idle', idle(u, **ready)); put('walk', walk(u, wa=30, hand_b=(3.0, 5.5), swing=0.4))
        # attack: a thrust from behind the raised shield; the shield never drops
        sh_up = dict(hand_b=(7.5, 0.5))
        put('attack', ([wp(bob=1.0, lean=0.5, wa=60, hand_f=(3, 5), hand_b=(6, 2), fb=(-5, 0), ff=(4, 0)),
                        wp(bob=2.0, lean=-1.0, dx=-1.5, wa=88, hand_f=(-2.5, 5.5), fb=(-6.5, 0), ff=(4, 0), **sh_up),          # draw back
                        wp(bob=1.6, lean=3.5, dx=2.5, wa=92, hand_f=(10.5, 4), fb=(-7, 0), ff=(6.5, 0), **sh_up,
                             fx=[(45, 24.5, 1.6, (255, 255, 255)), (44, 22.5, 0.6, (230, 236, 245)), (44, 26.5, 0.6, (230, 236, 245))]),  # thrust
                        wp(bob=1.8, lean=3.5, dx=2.5, wa=92, hand_f=(10.5, 4), fb=(-7, 0), ff=(6.5, 0), **sh_up),                 # hold
                        wp(bob=1.2, lean=1.5, dx=1.0, wa=80, hand_f=(5, 5), fb=(-5.5, 0), ff=(5, 0), **sh_up),
                        wp(bob=0.3, **ready)], [110, 160, 45, 140, 110, 110]))
        # shield bash: sword pulled back low, drive the shield forward with the whole body
        put('shield_bash', ([wp(bob=1.6, lean=-1.5, dx=-1.5, wa=-110, hand_f=(-0.5, 6.5), hand_b=(4, 2), fb=(-6, 0), ff=(4, 0)),
                             wp(bob=2.2, lean=-2.5, dx=-2.5, wa=-115, hand_f=(-1, 6.5), hand_b=(3, 2.5), fb=(-6.5, 0), ff=(3.5, 0)),
                             wp(bob=1.4, lean=4.5, dx=3.5, wa=-110, hand_f=(0.5, 6.5), hand_b=(10, 0.5), fb=(-7.5, 0), ff=(7, 0),
                                  fx=[(43, 19, 2.4, (255, 255, 255)), (45, 15, 1.0, (255, 240, 200)), (45, 24, 1.0, (255, 240, 200)), (41, 13, 0.8, (255, 220, 160))]),
                             wp(bob=1.6, lean=4.5, dx=3.5, wa=-110, hand_f=(0.5, 6.5), hand_b=(10, 0.5), fb=(-7.5, 0), ff=(7, 0)),
                             wp(bob=1.0, lean=2.0, dx=1.5, wa=-60, hand_f=(1.5, 6), hand_b=(6, 2), fb=(-5.5, 0), ff=(5, 0)),
                             wp(bob=0.3, **ready)], [110, 170, 45, 160, 110, 110]))
        # shield block: shoulder behind the raised shield, sword drawn back low, head tucked; the barrier hugs the shield face
        def sb(k, hit=False, cx=23, cy=22):
            return barrier(k, hit, cx=cx, cy=cy) if not hit else barrier(k, False, cx=cx, cy=cy) + [(36, 17, 1.0, BLUE_HI), (37, 26, 1.0, BLUE_HI)]
        gd = dict(bob=2.6, dx=-0.5, lean=1.5, head_dy=1.0, wa=-115, hand_f=(-0.5, 6.5), hand_b=(7.5, 0.5), fb=(-6.5, 0), ff=(5, 0))
        put('defend', ([wp(**ready),
                        wp(bob=1.6, dx=-0.5, lean=0.8, wa=-40, hand_f=(1, 6), hand_b=(6, 3), fb=(-6, 0), ff=(5, 0)),
                        wp(**dict(gd, hand_b=(8, 0)), fx=[(33, 20, 1.2, (255, 255, 255))]),                # shield snaps up
                        wp(fx=sb(0.5), **gd), wp(fx=sb(0.8), **gd), wp(fx=sb(1.0), **gd)], [90, 80, 60, 100, 120, 200]))
        fr = []
        for i in range(8):  # stance: guarded idle, shield held up, barrier breathing on its face
            t = i / 8 * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t)
            fr.append(wp(fx=sb(0.55 + 0.35 * b), **dict(gd, bob=gd['bob'] + b * 0.5)))
        put('idle_guarded', (fr, [150] * 8))
        # hit on the shield: spark on its face, the shield arm gives, he slides back but never drops guard
        put('hurt_guarded', ([wp(fx=sb(1.0) + [(33, 20, 2.4, (255, 255, 255)), (35, 17, 1.0, (255, 240, 200)), (35, 24, 1.0, (255, 240, 200))], **gd),
                              wp(fx=sb(0.9, True), **dict(gd, dx=-2.4, bob=3.0, lean=0.6, hand_b=(6.5, 1.2), fb=(-7.5, 0))),
                              wp(fx=sb(0.8), **dict(gd, dx=-1.8, bob=2.8, lean=1.0, hand_b=(7, 0.8), fb=(-7, 0))),
                              wp(fx=sb(0.8), **gd)], [60, 100, 110, 140]))
        put('hurt', hurt(u, **ready)); put('death', death(u, drop=(30, 45, 92)))
    elif name == 'ranger':
        rd = dict(bow_a=35, hand_f=(5, 7.5))
        put('idle', idle(u, **rd)); put('walk', walk(u, bow_a=40, hand_f=(5, 7.5), swing=0.5))
        st = dict(fb=(-5.5, 0), ff=(4.5, 0))
        put('attack', ([pose(u, bob=0.5, bow_a=20, hand_f=(6, 4), **st),
                        pose(u, bob=0.8, lean=-0.3, bow_a=0, hand_f=(9.5, 0.5), pull=0.2, nock=True, hand_b=(4.5, 0.5), **st),
                        pose(u, bob=1.0, lean=-0.8, bow_a=0, hand_f=(10, 0.5), pull=0.6, nock=True, hand_b=(1.0, 0.0), **st),
                        pose(u, bob=1.2, lean=-1.2, bow_a=0, hand_f=(10.2, 0.5), pull=1.0, nock=True, hand_b=(-2.0, -0.5), **st),
                        pose(u, bob=1.0, lean=-1.6, dx=-0.6, bow_a=-4, hand_f=(10, 0), hand_b=(-3.5, -1), arrow=(44, 22.0), **st),
                        pose(u, bob=0.8, lean=-1.0, dx=-0.4, bow_a=-2, hand_f=(9.8, 0.3), hand_b=(-3, 0), **st),
                        pose(u, bob=0.4, bow_a=20, hand_f=(6, 4), **st)], [100, 110, 90, 260, 50, 140, 120]))
        # stance: focus (aim) idle: bow half-raised, eye glint, slow gold motes at the head
        fr = []
        for i in range(8):
            t = i / 8 * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t)
            fr.append(pose(u, bob=1.0 + b * 0.5, lean=-0.3, bow_a=10, hand_f=(8, 2.5), fb=(-5.5, 0), ff=(4.5, 0),
                           fx=[(26.5 + 4 * math.cos(t + j * 2.1), 12 + 2 * math.sin(t + j * 2.1), 0.6 + 0.3 * b, GOLDEN if j % 2 else GOLDEN_HI) for j in range(3)]))
        put('idle_focus', (fr, [150] * 8))
        put('hurt', hurt(u, **rd)); put('death', death(u, drop=(31, 43, 90)))
    elif name == 'priest':  # mace overhead smash
        rd = dict(wa=30, hand_f=(5, 6.5))  # mace raised forward, head up (user 2026-10-20, ART2 pick B3)
        put('idle', idle(u, **rd)); put('walk', walk(u, wa=35, swing=0.8))
        st = dict(fb=(-5.5, 0), ff=(4.5, 0))
        put('attack', ([pose(u, bob=0.8, lean=-1.0, wa=-10, hand_f=(1, -6), hand_b=(-1, -3), **st),
                        pose(u, bob=1.2, lean=-2.0, dx=-1, wa=-40, hand_f=(-1, -7.5), hand_b=(-2, -4), **st),
                        pose(u, bob=0.8, lean=2.0, dx=1, wa=60, hand_f=(7, -2), slash=arc_pts(22, 18, 12, -20, 80, 12), **st),
                        pose(u, bob=2.2, lean=4.0, dx=2.5, wa=140, hand_f=(8, 5), fb=(-6.5, 0), ff=(6, 0),
                             fx=[(37, 38, 2.0, (255, 255, 255)), (34, 40, 1.0, GOLDEN_HI), (40, 40, 1.0, GOLDEN_HI), (37, 35, 0.8, GOLDEN)]),
                        pose(u, bob=2.0, lean=3.5, dx=2.5, wa=150, hand_f=(8, 5.5), fb=(-6.5, 0), ff=(6, 0)),
                        pose(u, bob=0.4, **rd)], [120, 180, 50, 150, 110, 120]))
        put('hurt', hurt(u, **rd)); put('death', death(u, drop=(31, 44, 95)))
    elif name == 'rogue':  # quick double stab with both daggers
        rd = dict(wa=110, wb=110, hand_f=(5, 5), hand_b=(2, 6))
        put('idle', idle(u, bob=0.8, lean=1.5, **rd)); put('walk', walk(u, wa=110, wb=120, hand_f=(5, 5), hand_b=(1, 6.5), lean=2.0))
        put('attack', ([pose(u, bob=1.6, lean=1.0, dx=-1, wa=100, wb=100, hand_f=(3, 6), hand_b=(0, 6), fb=(-6, 0), ff=(3, 0)),
                        pose(u, bob=1.2, lean=4.0, dx=3, wa=90, wb=110, hand_f=(10, 2.5), hand_b=(1, 6), fb=(-7, 0), ff=(6, 0),
                             fx=[(40, 22, 1.2, (255, 255, 255))]),
                        pose(u, bob=1.4, lean=4.0, dx=3.5, wa=110, wb=95, hand_f=(5, 5), hand_b=(8, 3.5), fb=(-7, 0), ff=(6.5, 0),
                             fx=[(39, 24, 1.2, (255, 255, 255)), (41, 23, 0.7, (255, 200, 200))]),
                        pose(u, bob=1.0, lean=2.5, dx=1.5, wa=110, wb=110, hand_f=(6, 4.5), hand_b=(3, 5.5), fb=(-5, 0), ff=(4.5, 0)),
                        pose(u, bob=0.8, lean=1.5, **rd)], [120, 50, 70, 120, 120]))
        put('hurt', hurt(u, **rd)); put('death', death(u, drop=(29, 44, 88), drop_b=(15, 44, 95)))
    elif name == 'mage':  # raise staff, charge the gem, thrust, bolt flies
        rd = dict(wa=5, hand_f=(4.5, 4))
        put('idle', idle(u, **rd)); put('walk', walk(u, wa=12, hand_f=(4.5, 4), swing=0.5))
        bolt = lambda x: [(x, 20, 2.0, (150, 220, 255)), (x - 2.5, 20, 1.3, (80, 140, 230)), (x - 4.5, 20, 0.8, (80, 140, 230)), (x + 0.5, 19.5, 0.8, (255, 255, 255))]
        put('attack', ([pose(u, bob=0.4, wa=0, hand_f=(4.5, 2.5), hand_b=(4, 4)),
                        pose(u, bob=0.8, lean=-1.2, wa=-8, hand_f=(3, -0.5), hand_b=(4, 2), glow=0.4),
                        pose(u, bob=1.0, lean=-1.6, wa=-12, hand_f=(2.5, -1.5), hand_b=(4.5, 1.5), glow=1.0),
                        pose(u, bob=0.8, lean=3.0, dx=1.5, wa=75, hand_f=(9, 1), hand_b=(5, 3), fb=(-6, 0), ff=(5, 0), glow=0.6, fx=bolt(41)),
                        pose(u, bob=0.6, lean=2.0, dx=1.0, wa=80, hand_f=(8.5, 1.5), hand_b=(5, 3.5), fb=(-6, 0), ff=(5, 0), fx=bolt(52)),
                        pose(u, bob=0.3, **rd)], [120, 140, 220, 60, 120, 140]))
        fr = []
        for i in range(8):  # stance: focused/charged idle, gem glows and runes circle the feet
            t = i / 8 * 2 * math.pi; b = 0.5 + 0.5 * math.sin(t)
            fr.append(pose(u, bob=0.5 + b * 0.5, wa=5, hand_f=(4.5, 4), glow=0.3 + 0.5 * b,
                           fx=[(20 + 9 * math.cos(t + j * 1.57), 43.5 + 1.2 * math.sin(t + j * 1.57), 0.6, (150, 220, 255) if j % 2 else (80, 140, 230)) for j in range(4)]))
        put('idle_focus', (fr, [150] * 8))
        put('hurt', hurt(u, **rd)); put('death', death(u, drop=(26, 44, 95)))
    elif name == 'brawler':
        # guard: near fist up at the chin; the far (power) fist chambered at the hip behind the body throws the punches
        G_F, G_B = (9.5, -6), (7, -4)
        guard = dict(hand_f=G_F, hand_b=G_B, fist_f=True, fist_b=True)
        bobf = lambda k: (G_F[0] - 0.5 * (k % 2), G_F[1] + (0.9 if k % 2 else -0.3))
        gb = lambda k: dict(guard, hand_f=bobf(k))
        put('idle', idle(u, bob=1.0, lean=0.8, alt=1.3, **dict(guard, hand_f=(8.5, -4.5), hand_b=(7, -3.5))))  # L-shaped arms: elbow forward, forearm up
        # walk: relaxed, guard dropped to chest/waist, arms swing loosely with the stride
        put('walk', walk(u, hand_f=(6, 8), hand_b=(3.5, 9), fist_f=True, fist_b=True, swing=2.2, hf_amp=0.3))
        st = dict(fb=(-5.5, 0), ff=(5, 0), fist_f=True, fist_b=True)
        spark = lambda x, y, r=1.6: [(x, y, r, (255, 255, 255)), (x + 2, y - 2, 0.7, (255, 220, 160)), (x + 2, y + 2, 0.7, (255, 220, 160))]
        # jab / body blow: chamber a little deeper, drive the hip fist straight out past the chest, snap it back
        # jab: short and straight; the hip fist cocks up to the chest, then drives straight out at face height and back.
        # the chin fist keeps bobbing through every action frame
        bobf = lambda k: (G_F[0] - 0.5 * (k % 2), G_F[1] + (0.9 if k % 2 else -0.3))
        put('jab', ([pose(u, bob=1.2, lean=0.8, hand_f=bobf(0), hand_b=G_B, **st),
                     pose(u, bob=1.3, lean=1.0, hand_f=bobf(1), hand_b=(3, -0.5), **st),
                     pose(u, bob=1.0, lean=2.5, dx=1.5, hand_f=(3.5, -3), hand_b=(12, -3.5), fx=spark(39, 13), **st),
                     pose(u, bob=1.1, lean=2.2, dx=1.5, hand_f=(3.5, -2.5), hand_b=(11.5, -3.2), **st),
                     pose(u, bob=1.2, lean=1.2, dx=0.5, hand_f=bobf(1), hand_b=(3.5, 0), **st),
                     pose(u, bob=1.0, lean=0.8, hand_f=bobf(0), hand_b=G_B, **st)], [80, 60, 50, 100, 70, 100]))
        # haymaker: far fist winds way back, the body twists, a wide hook swings round and lands
        put('haymaker', ([pose(u, bob=1.6, lean=-2.0, dx=-1.5, hand_f=bobf(1), hand_b=(-6, 5), **st),
                          pose(u, bob=2.0, lean=-3.0, dx=-2.5, hand_f=bobf(2), hand_b=(-8, 3), **st),
                          pose(u, bob=1.2, lean=1.5, dx=0.5, hand_f=(4, -1.5), hand_b=(2, -1), fx=[(x, y, 0.6, (255, 220, 160)) for x, y in arc_pts(20, 21, 10, -100, 20, 9)], **st),
                          pose(u, bob=2.0, lean=5.0, dx=3.5, hand_f=(3, -3.5), hand_b=(12, 0), fx=spark(39, 17, 2.4) + [(38, 15, 0.8, (255, 200, 120))], **st),
                          pose(u, bob=1.6, lean=3.0, dx=2.0, hand_f=(4.5, -2), hand_b=(8, 2.5), **st),
                          pose(u, bob=1.0, lean=0.8, **guard)], [110, 200, 40, 160, 110, 120]))
        # front kick: chamber the knee high, snap the foot out at chest height, recoil
        put('kick', ([pose(u, bob=1.0, ff=(4.5, 0), fb=(-4.5, 0), **gb(1)),
                      pose(u, bob=0.0, lean=-2.0, dx=-1.0, ff=(5, 13), fb=(-3, 0), **gb(2)),
                      pose(u, bob=-0.5, lean=-4.5, dx=-2.0, ff=(15.5, 21), fb=(-3.5, 0), hand_f=(4, -1), hand_b=(-4, 6), fist_f=True, fist_b=True,
                           fx=[(37, 21, 2.0, (255, 255, 255)), (39, 19, 0.8, (255, 220, 160)), (39, 24, 0.8, (255, 220, 160))]),
                      pose(u, bob=-0.5, lean=-4.5, dx=-2.0, ff=(15.5, 21), fb=(-3.5, 0), hand_f=(4, -1), hand_b=(-4, 6), fist_f=True, fist_b=True),
                      pose(u, bob=0.0, lean=-2.0, dx=-1.0, ff=(5, 13), fb=(-3, 0), **gb(3)),
                      pose(u, bob=1.0, lean=0.8, ff=(5, 0), fb=(-5, 0), **gb(4))], [90, 90, 50, 160, 90, 110]))
        # sweep kick: drop low and spin a full turn on the planted foot; the leg sweeps behind, the body turns
        # (mirrored frames), then the leg scythes through the front; a dust ring marks the circle
        def ring(k, n=12):
            out = []
            for j in range(n):
                a = j / n * 2 * math.pi
                if (j / n) <= k: out.append((20 + 14 * math.cos(a + math.pi), 42.5 - 2.6 * math.sin(a + math.pi), 0.8, (230, 220, 200) if j % 2 else (200, 186, 160)))
            return out
        low = dict(hand_f=(5, 8), hand_b=(-3, 10), fist_f=True, fist_b=True)
        tr = lambda *xs: [(x, 1.0) for x in xs]
        put('sweep_kick', ([pose(u, bob=2.0, lean=0.5, ff=(5, 0), fb=(-5, 0), **guard),
                            pose(u, bob=7.0, lean=1.5, ff=(4, 0), fb=(-8, 0), hand_f=(5, 6), hand_b=(4, 8), fist_f=True, fist_b=True),
                            rig.mirror(pose(u, bob=7.5, lean=2.0, ff=(15, 1), fb=(-5, 0), leg_trail=tr(8, 12), fx=ring(0.3), **low)),   # turned away: leg sweeps behind
                            rig.mirror(pose(u, bob=7.5, lean=1.0, ff=(9, 1), fb=(-4, 0), leg_trail=tr(15, 13), fx=ring(0.45), **low)),  # coming round
                            pose(u, bob=7.5, lean=0.0, ff=(2, 1), fb=(-2, 0), hand_f=(3, 9), hand_b=(-1, 9), fist_f=True, fist_b=True, leg_trail=tr(-8, -12, -15), fx=ring(0.6)),
                            pose(u, bob=7.5, lean=1.5, ff=(10, 1), fb=(-6, 0), leg_trail=tr(-2, 3, 6), fx=ring(0.8), **low),
                            pose(u, bob=7.5, lean=2.0, ff=(16, 1.5), fb=(-6, 0), leg_trail=tr(8, 11, 13.5), fx=ring(1.0) + [(38, 41, 2.0, (255, 255, 255)), (41, 39, 0.8, (255, 220, 160))], **low),
                            pose(u, bob=7.5, lean=2.0, ff=(16, 1.5), fb=(-6, 0), fx=ring(1.0), **low),
                            pose(u, bob=6.0, lean=1.5, ff=(8, 0), fb=(-7, 0), hand_f=(5, 6), hand_b=(3, 8), fist_f=True, fist_b=True),
                            pose(u, bob=1.0, lean=0.8, ff=(5, 0), fb=(-5, 0), **guard)], [110, 80, 50, 50, 50, 40, 60, 140, 110, 120]))
        put('hurt', hurt(u, **guard))
        # death: no weapon to drop, so the guard breaks: arms fly up and back on the hit, then fall limp and splayed
        put('death', ([pose(u, bob=1.0, lean=0.8, **guard),
                       pose(u, bob=1.5, hurt=1.0, lean=-1.5, dx=-1.5, head_dy=1, flash=True, hand_f=(6, -4), hand_b=(4, -2.5)),
                       pose(u, bob=1.8, hurt=1.0, lean=-2.5, dx=-2.5, head_dy=1.2, hand_f=(4, -8), hand_b=(2, -7)),
                       pose(u, bob=1.8, hurt=1.0, lean=-3.0, dx=-2.8, head_dy=1.4, hand_f=(3, -11), hand_b=(-2, -10)),
                       pose(u, bob=1.2, hurt=1.0, fall=0.22, lean=-1.0, dx=-2.0, hand_f=(5, -6), hand_b=(-6, -10)),
                       pose(u, bob=0.8, hurt=1.0, fall=0.5, dx=-1.5, hand_f=(3, 5), hand_b=(-6, -10)),
                       pose(u, bob=0.3, hurt=1.0, fall=0.82, dx=-1.0, hand_f=(-2, 10), hand_b=(-4.5, -11)),
                       pose(u, bob=0.0, hurt=1.0, fall=1.0, dx=-1.0, hand_f=(-2.5, 10.5), hand_b=(-4.5, -11))],
                      [200, 60, 90, 130, 100, 90, 90, 600]))
    if name in GUARDS: defensive_set(u, GUARDS[name], A, T)
    if name == 'warrior' and variant == '':
        a_, at_ = clips('warrior', 'A', unit); b_, bt_ = clips('warrior', 'B', unit)
        A['attack'], T['attack'] = a_['attack'], at_['attack']
        for k in ('shield_bash', 'defend', 'idle_guarded', 'hurt_guarded'): A[k], T[k] = b_[k], bt_[k]
    return A, T


if __name__ == "__main__":
    out = sys.argv[1]; names = sys.argv[2:] or list(UNITS)
    for n in names: print(n, export(n, *clips(n), out))
