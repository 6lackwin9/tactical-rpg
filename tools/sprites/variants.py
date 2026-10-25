# Body variants on the class rig (SPRITE2; ASSETORG2 section 4.1): the goblin variant of each class body (kit), the
# game's named bodies (Grakka Ashmaw, the Horde Warchief, the NPCs), the training dummy (a prop body, its own drawing)
# and the palette files that recolour a body for a character or a mob family (ash goblins, named goblins).
# tools/sprites/export_units.py writes them; tools/gen_art.py never writes a body listed here.
import copy
import math

import classes
import rig
from rig import F, OUT, W

GOBLIN_SKIN = dict(skin=(110, 150, 70), skin_lo=(76, 108, 48), eye=(240, 208, 60), hair=(196, 46, 34), hair_lo=(134, 28, 24),
                   bone=(226, 216, 186), bone_lo=(168, 156, 126), rust=(156, 78, 40))
# palette slots a goblin body exposes (first key wins when two share a colour; at most 16)
GOBLIN_SLOTS = ("skin", "skin_lo", "main", "main_lo", "main_hi", "accent", "accent_lo", "leg", "leg_lo", "glove", "glove_lo",
                "fore", "sleeve", "sleeve_lo", "boot_lo", "hair")
# the named humans' slots (the class rule would give Hest 17)
HUMAN_SLOTS = ("main", "accent", "sleeve", "fore", "metal", "leg", "skin", "hair", "beard", "cap", "glove", "shield", "emblem",
               "trim", "leather", "boot")


def variant(cls, scale=1.0, pal=None, **kw):
    """A copy of class `cls`'s rig unit, resized by `scale` (legs, arms, torso, neck; the rig scales every pose offset by
    the same factor) with outfit keys `kw` and palette changes `pal`."""
    base = classes.UNITS[cls]
    u = copy.deepcopy(base)
    u.update(leg=base['leg'] * scale, arm=base['arm'] * scale, tl=base['tl'] * scale, neck=6 * scale, scale=scale)
    u.update(kw)
    u['pal'] = dict(base['pal'])
    u['pal'].update(pal or {})
    return u


def goblin(cls, hat, pal, scale=0.84, **kw):
    """The goblin variant of a class body: shorter (scale), green skin, the goblin head (ears, nose, tusks), headgear."""
    p = dict(GOBLIN_SKIN)
    p.update(pal)
    kw.setdefault('neck', 5.0 * scale / 0.84)
    u = variant(cls, scale, p, head='goblin', hat=hat, **kw)
    u.pop('mask', None)
    return u


# WAR2H1 look matrix (DIR4 section 7.1, W1: bake every look). A look is a grip (a pose set) plus a weapon drawing.
# TWO_HANDED: item family -> its 2H drawing (rig.DRAWERS). LOOKS: class -> the looks the exporter bakes for the class
# base, each a body of its own at units/<class>/<look>/ in the base's layer: `1h_buckler` (the base's one-hand weapon
# and a buckler) and `2h_<family>`; the base's own sheet is its 1H + kite shield look. Every look of a body shares the
# body's palette slots and palette files (DIR4 L3). The game picks the look from the equipped gear
# (rpgkit_content::art::gear_body_chain). A new family on the 2H grip: one drawer in rig.DRAWERS, one entry here.
TWO_HANDED = {'sword': 'greatsword', 'axe': 'battle_axe', 'hammer': 'warhammer'}
LOOKS = {'warrior': ('1h_buckler', '2h_sword', '2h_axe', '2h_hammer')}


def two_handed(u, family, drawing=None):
    """Unit `u` on the 2H grip with `family`'s drawing (or `drawing`, a rig.DRAWERS key) and no off-hand."""
    u = copy.deepcopy(u)
    u.update(grip='2h', weapon=drawing or TWO_HANDED[family], offhand=None)
    return u


def with_buckler(u, weapon=None):
    """Unit `u` on the buckler grip: its one-hand weapon (or `weapon`, a rig drawing) and a buckler on the far fist."""
    u = copy.deepcopy(u)
    u.update(grip='buckler', offhand='buckler')
    if weapon:
        u['weapon'] = weapon
    return u


def look_unit(u, look):
    """Unit `u` wearing look `look` (a LOOKS id)."""
    if look == '1h_buckler':
        return with_buckler(u)
    return two_handed(u, look[len('2h_'):])


def looks():
    """Look body key -> (base body key, look id)."""
    return {'%s/%s' % (cls, look): (cls, look) for cls, ids in LOOKS.items() for look in ids}


def look_base(key):
    """The body a look belongs to (None: `key` is not a look)."""
    return looks().get(key, (None, None))[0]


# The named warrior bodies (WAR2H1, user 2026-10-23): one look each, the gear they wield (their gear in the data follows in a seed):
# Hest, Brask and Orsa a hatchet (wpn_hand_axe) and a buckler; Grakka a two-handed double-bitted axe, no shield.


# Ash goblins of the Horde (ASSETORG2: the ash_* mobs wear the goblin bodies): ashen skin, soot cloth, ember trim.
ASH = dict(skin=(132, 138, 114), skin_lo=(96, 102, 82), main=(70, 64, 66), main_lo=(50, 46, 48), main_hi=(98, 92, 94),
           accent=(214, 104, 42), accent_lo=(150, 66, 28), leg=(64, 58, 58), leg_lo=(46, 42, 42), leather=(84, 66, 52),
           glove=(92, 88, 86), glove_lo=(68, 64, 62), fore=(92, 88, 86), sleeve=(118, 124, 102), sleeve_lo=(104, 110, 90), boot_lo=(78, 84, 66), hair=(220, 110, 44))

# body key -> layer, class, rig unit, palette slots (None = the class rule: base colours present in the sheet)
BODIES = {
    # ---- kit: the goblin variant of each class body (catalog units goblin_*, fleeing_scout)
    'brawler/goblin': ('kit', 'brawler', goblin('brawler', 'band', dict(
        main=(110, 150, 70), main_lo=(76, 108, 48), accent=(150, 66, 42), accent_lo=(108, 44, 28),
        sleeve=(98, 136, 62), sleeve_lo=(84, 118, 54), fore=(200, 184, 150), fore_lo=(160, 146, 116),
        glove=(200, 184, 150), glove_lo=(176, 160, 128), glove_hi=(232, 220, 190), leg=(112, 84, 56), leg_lo=(80, 58, 38),
        boot=(76, 108, 48), boot_lo=(60, 86, 38))), GOBLIN_SLOTS),
    'brawler/goblin_berserker': ('kit', 'brawler', goblin('brawler', 'mohawk', dict(
        main=(110, 150, 70), main_lo=(76, 108, 48), accent=(170, 40, 32), accent_lo=(120, 26, 22),
        sleeve=(98, 136, 62), sleeve_lo=(84, 118, 54), fore=(116, 84, 58), fore_lo=(90, 64, 44),
        glove=(116, 84, 58), glove_lo=(96, 68, 48), glove_hi=(146, 110, 80), leg=(70, 52, 44), leg_lo=(50, 36, 30),
        boot=(76, 108, 48), boot_lo=(60, 86, 38)), scale=0.9, girth=0.5), GOBLIN_SLOTS),
    'rogue/goblin': ('kit', 'rogue', goblin('rogue', 'bandana', dict(
        main=(92, 68, 48), main_lo=(66, 48, 34), main_hi=(124, 94, 66), accent=(172, 44, 40), accent_lo=(120, 28, 28),
        sleeve=(92, 68, 48), sleeve_lo=(66, 48, 34), leg=(70, 60, 50), leg_lo=(50, 42, 36), glove=(56, 46, 40), glove_lo=(42, 34, 30))),
        GOBLIN_SLOTS),
    'rogue/goblin_scout': ('kit', 'rogue', goblin('rogue', 'hood', dict(
        main=(88, 78, 62), main_lo=(62, 54, 42), main_hi=(118, 106, 86), accent=(124, 92, 54), accent_lo=(92, 66, 38),
        sleeve=(88, 78, 62), sleeve_lo=(62, 54, 42), leg=(92, 76, 54), leg_lo=(66, 54, 38), glove=(70, 56, 40), glove_lo=(52, 42, 30)),
        back=None), GOBLIN_SLOTS),
    'ranger/goblin': ('kit', 'ranger', goblin('ranger', 'band', dict(
        main=(112, 84, 54), main_lo=(80, 60, 38), main_hi=(146, 114, 76), accent=(228, 122, 42), accent_lo=(162, 82, 26),
        sleeve=(112, 84, 54), sleeve_lo=(80, 60, 38), leg=(86, 72, 52), leg_lo=(62, 52, 38), glove=(70, 50, 34))), GOBLIN_SLOTS),
    'priest/goblin': ('kit', 'priest', goblin('priest', 'skull', dict(
        main=(106, 78, 126), main_lo=(76, 54, 94), main_hi=(142, 112, 162), accent=(206, 84, 52), accent_lo=(150, 58, 36),
        trim=(226, 216, 186), sleeve=(106, 78, 126), sleeve_lo=(76, 54, 94), fore=(106, 78, 126), fore_lo=(76, 54, 94),
        leg=(76, 54, 94), leg_lo=(58, 40, 72), glove=(110, 150, 70))), GOBLIN_SLOTS),
    # ---- game: named bodies
    # Grakka Ashmaw, Chieftain of the Horde: a large goblin warlord in dark iron, a crown of rusted nails, a red cape;
    # a two-handed double-bitted axe, no shield (WAR2H1)
    'warrior/grakka_ashmaw': ('game', 'warrior', two_handed(goblin('warrior', 'crown', dict(
        skin=(94, 128, 60), skin_lo=(66, 92, 42), metal=(104, 108, 116), metal_hi=(156, 162, 172), metal_lo=(66, 68, 76),
        main=(140, 36, 30), main_lo=(96, 24, 22), main_hi=(176, 60, 48), accent=(140, 36, 30), accent_lo=(96, 24, 22),
        sleeve=(140, 36, 30), sleeve_lo=(96, 24, 22), fore=(104, 108, 116), fore_lo=(66, 68, 76),
        leg=(104, 108, 116), leg_lo=(66, 68, 76), glove=(70, 54, 40), glove_lo=(50, 38, 28),
        shield=(84, 66, 52), shield_lo=(60, 46, 36), emblem=(200, 64, 44)), scale=1.1, girth=1.0, neck=5.0, back='cape'), 'axe', 'double_axe'), None),
    # the Horde Warchief (patrol leader): a big ash goblin brawler, horned iron cap, iron gauntlets, a shoulder plate
    'brawler/ash_warchief': ('game', 'brawler', goblin('brawler', 'horns', dict(
        ASH, main=(132, 138, 114), main_lo=(96, 102, 82), sleeve=(118, 124, 102), sleeve_lo=(104, 110, 90),
        fore=(112, 118, 128), fore_lo=(80, 86, 96), glove=(112, 118, 128), glove_lo=(90, 96, 106), glove_hi=(160, 166, 176),
        metal=(72, 74, 84), metal_hi=(122, 126, 138), metal_lo=(50, 52, 60), boot=(58, 52, 50), boot_lo=(44, 40, 38)),
        scale=0.97, girth=0.7, pauldron=True), None),
    # NPCs: class bodies in their own outfit and colours (the warrior NPCs: a hatchet and a buckler, WAR2H1)
    'warrior/brask_toller': ('game', 'warrior', with_buckler(variant('warrior', 1.0, dict(
        main=(58, 82, 128), main_lo=(40, 56, 92), main_hi=(84, 110, 160), accent=(58, 82, 128), accent_lo=(40, 56, 92),
        sleeve=(58, 82, 128), sleeve_lo=(40, 56, 92), hair=(104, 72, 46), beard=(112, 78, 50),
        shield=(58, 82, 128), shield_lo=(40, 56, 92), emblem=(224, 180, 60)), head='kettle', beard=True), 'hand_axe'), HUMAN_SLOTS),
    'warrior/hest_the_quartermaster': ('game', 'warrior', with_buckler(variant('warrior', 1.0, dict(
        main=(112, 86, 58), main_lo=(80, 60, 40), main_hi=(144, 114, 80), accent=(96, 104, 70), accent_lo=(70, 76, 50),
        sleeve=(112, 86, 58), sleeve_lo=(80, 60, 40), fore=(217, 166, 121), fore_lo=(176, 126, 88),
        leg=(92, 84, 70), leg_lo=(66, 60, 50), hair=(184, 180, 172), beard=(226, 222, 212), cap=(96, 70, 46), cap_lo=(70, 50, 32),
        cap_hi=(124, 94, 64), shield=(110, 84, 56), shield_lo=(80, 60, 40), emblem=(96, 104, 70)),
        head='cap', beard=True, torso='leather'), 'hand_axe'), HUMAN_SLOTS),
    'warrior/orsa_tallow': ('game', 'warrior', with_buckler(variant('warrior', 1.0, dict(
        metal=(178, 188, 200), metal_hi=(230, 236, 244), metal_lo=(112, 122, 136), main=(48, 98, 82), main_lo=(32, 68, 56),
        main_hi=(76, 130, 110), accent=(48, 98, 82), accent_lo=(32, 68, 56), sleeve=(48, 98, 82), sleeve_lo=(32, 68, 56),
        fore=(178, 188, 200), fore_lo=(112, 122, 136), leg=(178, 188, 200), leg_lo=(112, 122, 136),
        hair=(150, 72, 42), hair_lo=(110, 50, 30), shield=(48, 98, 82), shield_lo=(32, 68, 56), emblem=(224, 180, 60)),
        head='bare', braid=True), 'hand_axe'), HUMAN_SLOTS),
    'ranger/wenn_tallow': ('game', 'ranger', variant('ranger', 1.0, dict(
        main=(152, 122, 80), main_lo=(112, 88, 56), main_hi=(184, 154, 106), accent=(176, 58, 46), accent_lo=(120, 40, 32),
        sleeve=(152, 122, 80), sleeve_lo=(112, 88, 56), leg=(84, 76, 96), leg_lo=(60, 54, 70), hair=(122, 82, 46)),
        head='bare'), HUMAN_SLOTS),
}

# palette files: (layer, body, id) -> slot colours; each slot must be one of the body's palette slots, except in a
# shared palette (one colour set over several bodies), which keeps the slots each body has
SHARED_PALETTES = {'ash'}
PALETTES = {}
for _body in ('brawler/goblin', 'brawler/goblin_berserker', 'rogue/goblin', 'rogue/goblin_scout', 'ranger/goblin', 'priest/goblin'):
    PALETTES[('game', _body, 'ash')] = dict(ASH)  # the Ashlands Horde mobs (ash_brawler ... ash_berserker)
PALETTES.update({
    ('game', 'brawler/goblin', 'skarn_ironjaw'): dict(skin=(96, 132, 62), skin_lo=(66, 94, 42), accent=(120, 128, 140), accent_lo=(84, 90, 100),
                                                      glove=(140, 146, 156), leg=(62, 66, 84), leg_lo=(44, 46, 62)),
    ('game', 'rogue/goblin', 'vex_cutpurse'): dict(accent=(104, 46, 116), accent_lo=(70, 30, 80), main=(50, 44, 54), main_lo=(36, 32, 40),
                                                   leg=(42, 38, 46), leg_lo=(30, 28, 34)),
    ('game', 'ranger/goblin', 'nok_flamebow'): dict(accent=(246, 140, 40), accent_lo=(196, 84, 24), main=(74, 58, 50), main_lo=(54, 42, 36),
                                                    leg=(62, 52, 46), leg_lo=(44, 38, 34)),
    ('game', 'brawler/goblin_berserker', 'bolg_redhand'): dict(glove=(178, 38, 30), glove_lo=(132, 26, 20),
                                                               accent=(92, 30, 30), accent_lo=(66, 20, 20)),
    ('game', 'priest/goblin', 'gruk_smokecaller'): dict(main=(114, 112, 118), main_lo=(84, 82, 88), leg_lo=(64, 62, 68),
                                                        accent=(222, 112, 42)),
})


# ------------------------------------------------------------------ the training dummy (a prop body: idle, hurt, death)
DUMMY_PAL = dict(burlap=(200, 172, 120), burlap_lo=(158, 132, 88), straw=(232, 204, 112), straw_lo=(190, 160, 76),
                 wood=(138, 98, 58), wood_lo=(96, 66, 40), rope=(110, 84, 52), target=(178, 52, 42))


def _dummy(lean=0.0, head=None, flash=False, bits=(), snap=0.0):
    """One dummy frame: a post on a cross foot, a burlap body with a painted target, a crossbar for arms, a sack head
    with stitched X eyes. lean: degrees the upper part tips back (negative) or forward about the post's foot (or about the
    snap point when snap > 0); head: (x, y, angle) of a knocked-off head, else on the body; bits: straw (x, y)."""
    P = DUMMY_PAL
    g = F()  # the upper part, drawn upright, then turned about its pivot
    g.rect(19, 12, 21, 40, P['wood']); g.dot(21, 20, P['wood_lo']); g.dot(21, 32, P['wood_lo'])
    g.rect(8, 22, 32, 24, P['wood']); g.rect(8, 24, 32, 24, P['wood_lo'])
    for x0, d in ((8, -1), (32, 1)):  # straw tufts at the crossbar ends
        g.dot(x0 + d, 22, P['straw']); g.dot(x0 + d, 24, P['straw_lo']); g.dot(x0 + 2 * d, 23, P['straw'])
    g.poly([(14, 19), (26, 19), (27, 30), (25, 35), (15, 35), (13, 30)], P['burlap'])
    g.poly([(14, 30), (27, 30), (25, 35), (15, 35)], P['burlap_lo'])
    g.rect(13, 30, 27, 31, P['rope']); g.dot(23, 31, P['straw'])
    g.disc(21, 25, 3.0, P['target']); g.disc(21, 25, 1.8, P['burlap']); g.disc(21, 25, 0.8, P['target'])
    if head is None:
        g.disc(20, 13, 4.4, P['burlap']); g.disc(19, 15, 2.6, P['burlap']); g.rect(16, 16, 23, 17, P['burlap_lo'])
        for (ex, ey) in ((19, 12), (23, 12)):
            g.dot(ex - 1, ey - 1, OUT); g.dot(ex + 1, ey + 1, OUT); g.dot(ex - 1, ey + 1, OUT); g.dot(ex + 1, ey - 1, OUT); g.dot(ex, ey, OUT)
    piv = (20, 36) if snap else (20, 42)
    up = g.im.rotate(-lean, resample=0, center=piv) if lean else g.im
    f = F()
    f.rect(13, 42, 27, 43, P['wood_lo']); f.rect(15, 41, 25, 41, P['wood'])  # cross foot
    if snap:
        f.rect(19, 36, 21, 41, P['wood']); f.dot(19, 35, P['wood_lo']); f.dot(21, 35, P['straw'])
    else:
        f.rect(19, 40, 21, 41, P['wood'])
    f.im.alpha_composite(up); f.px = f.im.load()
    if head is not None:
        hx, hy, _a = head
        f.disc(hx, hy, 4.0, P['burlap']); f.rect(hx - 3, hy + 2, hx + 3, hy + 3, P['burlap_lo'])
        f.dot(hx - 1, hy - 1, OUT); f.dot(hx + 1, hy + 1, OUT); f.dot(hx - 1, hy + 1, OUT); f.dot(hx + 1, hy - 1, OUT)
    f.outline()
    for (bx, by) in bits:
        f.dot(bx, by, P['straw']); f.dot(bx + 1, by, P['straw_lo'])
    if flash:
        for y in range(W):
            for x in range(W):
                if f.px[x, y][3]: f.px[x, y] = (255, 255, 255, 255) if f.px[x, y][:3] != OUT else (255, 220, 220, 255)
    return f.im


def dummy_clips():
    A, T = {}, {}
    A['idle'] = [_dummy(lean=round(1.5 * math.sin(i / 6 * 2 * math.pi))) for i in range(6)]
    T['idle'] = [220] * 6
    A['hurt'] = [_dummy(lean=-6, flash=True), _dummy(lean=-9, bits=[(30, 18), (33, 24), (28, 13)]),
                 _dummy(lean=4, bits=[(34, 16), (37, 25)]), _dummy(lean=-2), _dummy(lean=0)]
    T['hurt'] = [60, 110, 100, 90, 90]
    A['death'] = [_dummy(lean=-8, flash=True),
                  _dummy(lean=-14, head=(14, 9, 0), snap=1, bits=[(30, 16), (33, 22), (26, 10)]),
                  _dummy(lean=-24, head=(9, 20, 0), snap=1, bits=[(34, 20), (36, 30), (12, 14)]),
                  _dummy(lean=-32, head=(7, 34, 0), snap=1, bits=[(36, 38), (11, 26)]),
                  _dummy(lean=-36, head=(7, 39, 0), snap=1, bits=[(36, 42), (12, 42), (30, 42)])]
    T['death'] = [60, 140, 110, 110, 600]
    return A, T


PROPS = {'dummy/base': ('kit', dummy_clips, DUMMY_PAL)}


def clips(key):
    """(frames, ms, unit palette, slot keys) of a variant body or a prop body."""
    if key in PROPS:
        _, fn, pal = PROPS[key]
        A, T = fn()
        return A, T, pal, tuple(pal)
    if key in looks():  # the exporter gives a look its base body's palette slots
        cls, look = looks()[key]
        u = look_unit(classes.UNITS[cls], look)
        A, T = classes.clips(cls, unit=u)
        return A, T, u['pal'], None
    _, cls, u, slots = BODIES[key]
    A, T = classes.clips(cls, unit=u)
    return A, T, u['pal'], slots


def layer(key):
    if key in looks():
        return 'kit'  # a class base's looks live with the class base, in the kit
    return (BODIES.get(key) or PROPS[key])[0]


def all_bodies():
    return list(BODIES) + list(looks()) + list(PROPS)


if __name__ == "__main__":
    import sys
    out = sys.argv[1]
    for k in sys.argv[2:] or all_bodies():
        A, T, _, _ = clips(k)
        print(k, rig.export(k.replace('/', '_'), A, T, out))
