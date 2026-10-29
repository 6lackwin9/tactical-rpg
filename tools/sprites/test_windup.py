# Tests of the wind-up, stagger and dazed rows (WINDUP1-R1 and R2, games/emberwake/docs/specs/WINDUP1.md sections 4 and 7).
# Run from anywhere:
#   python tools/sprites/test_windup.py
# The rig core (rig.lerp_kw, rig.windup, rig.dazed, rig.windup_rows, the strike and loop marks) is checked on its own;
# every humanoid body (the warrior's three grips and the views of warrior/base and warrior/2h_sword, R1; the ranger,
# brawler, priest, rogue and mage with their goblin and named bodies, R2) against the clip set of section 4, the strike
# frames of table 7.2 and the exporter checks X1 - X6 of 7.3. Nothing is written outside a temporary folder.
import contextlib
import io
import json
import math
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from PIL import Image  # noqa: E402

import classes  # noqa: E402
import export_units  # noqa: E402
import rig  # noqa: E402
import variants  # noqa: E402
import views  # noqa: E402

# section 4: frames, ms, loop and loop_from of the six rows
CLIP_SET = {'windup': (7, [90, 110, 120, 180, 180, 180, 180], True, 3),
            'windup_heavy': (7, [100, 120, 140, 160, 160, 160, 160], True, 3),
            'windup_buff': (7, [90, 110, 120, 180, 180, 180, 180], True, 3),
            'windup_debuff': (7, [90, 110, 120, 180, 180, 180, 180], True, 3),
            'stagger': (5, [60, 90, 110, 120, 160], False, None),
            'dazed': (8, [150] * 8, True, None)}
# table 7.2: the strike frame of each attack-like clip, per class (the creatures by kind)
STRIKES = {'warrior': {'attack': 2, 'shield_bash': 2}, 'ranger': {'attack': 4}, 'priest': {'attack': 2}, 'rogue': {'attack': 1},
           'mage': {'attack': 3}, 'brawler': {'jab': 2, 'haymaker': 2, 'kick': 2, 'sweep_kick': 2},
           'crawler': {'attack': 2, 'spit': 2}, 'beast': {'attack': 1}}
# the bodies with views (K6): DIR4-R1's pilot, and DIR4-R2's five classes and the 2H axe and hammer looks
PILOT = ('warrior', 'warrior/2h_sword', 'warrior/2h_axe', 'warrior/2h_hammer', 'brawler', 'ranger', 'rogue', 'priest', 'mage')


def quiet(fn, *a):
    with contextlib.redirect_stdout(io.StringIO()):
        return fn(*a)


def all_bodies():
    return list(classes.UNITS) + variants.all_bodies()


def windup_bodies():
    """Every body of a class with wind-up rows (classes.WINDUP_CLASSES): the class bodies, looks, goblins and named."""
    return [k for k in all_bodies() if kind_of(k) in classes.WINDUP_CLASSES]


def kind_of(key):
    return key.split('/')[0]


def pixels(im, colour):
    px = im.load()
    return [(x, y) for y in range(im.height) for x in range(im.width) if px[x, y][3] == 255 and px[x, y][:3] == tuple(colour)]


def same(a, b):
    return a.tobytes() == b.tobytes()


def changed(a, b):
    """How many pixels of two equal-sized RGBA frames differ."""
    x, y = a.tobytes(), b.tobytes()
    return sum(x[i:i + 4] != y[i:i + 4] for i in range(0, len(x), 4))


NEW_IDS = set(rig.WINDUP_ROWS) | {'%s_%s' % (n, v) for n in rig.WINDUP_ROWS for v in views.VIEWS}


_CACHE = {}


def exported(key):
    """export_units.body(key, 'all'), once per run."""
    if key not in _CACHE:
        _CACHE[key] = quiet(export_units.body, key, 'all')
    return _CACHE[key]


class Core(unittest.TestCase):
    """The rig helpers, on a plain warrior frame."""

    def draw(self, kw):
        return rig.pose(classes.UNITS['warrior'], **kw)

    def test_lerp_numbers_tuples_angles_and_flags(self):
        a = dict(bob=0.0, hand_f=(0.0, 4.0), wa=350.0, fore_z={'r': 2}, glow=0.0)
        b = dict(bob=2.0, hand_f=(4.0, 0.0), wa=10.0, fist_f=True)
        m = rig.lerp_kw(a, b, 0.5, rest={})
        self.assertEqual((m['bob'], m['hand_f']), (1.0, (2.0, 2.0)))
        self.assertAlmostEqual(m['wa'] % 360.0, 0.0)          # the short way round, not through 180
        self.assertIs(m['fist_f'], True)                      # a flag is the hold's
        self.assertNotIn('fore_z', m)                         # a layer only the ready pose sets ends with the intro
        self.assertEqual(rig.lerp_kw(a, b, 1.0, rest={})['wa'], 10.0)  # k = 1 is the hold exactly
        # a key missing (or None) on one side lerps from the rest pose pose() draws for it
        r = rig.lerp_kw(dict(hand_f=None), dict(hand_f=(8.0, 6.5)), 0.5)
        self.assertEqual(r['hand_f'], ((rig.SIDE_REST['hand_f'][0] + 8.0) / 2, 6.5))

    def test_windup_shape_and_marks(self):
        ready = rig.idle_keys(**classes.W1_READY)[0]
        hold = classes.W1_HOLDS['windup_debuff']
        for kind in rig.WINDUPS:
            frames, ms = rig.windup(kind, ready, hold, self.draw)
            self.assertEqual((len(frames), ms, rig.loop_frame(frames)), (7, CLIP_SET[kind][1], 3), kind)
            # K1: the third intro frame is the hold pose (k = 1), the first breathing frame too (bob offset 0)
            plain = self.draw(dict(hold))
            if kind in ('windup', 'windup_heavy'):
                self.assertTrue(same(frames[2], plain) and same(frames[3], plain), kind)
            self.assertFalse(same(frames[0], frames[2]), kind)

    def test_buff_motes_and_debuff_glint(self):
        ready = rig.idle_keys(**classes.W1_READY)[0]
        hold = classes.W1_HOLDS['windup_buff']
        frames, _ = rig.windup('windup_buff', ready, hold, self.draw)
        for i, im in enumerate(frames):
            gold = pixels(im, rig.GOLDEN) + pixels(im, rig.GOLDEN_HI)
            self.assertEqual(bool(gold), i >= 3, i)          # motes only while holding
            if gold:
                hx, hy = im.info['hand']
                self.assertTrue(all(y < hy for _, y in gold), i)  # above the hand, rising
        rise = [frames[i].info['hand'][1] - min(y for _, y in pixels(frames[i], rig.GOLDEN)) for i in range(3, 7)]
        self.assertTrue(all(0.5 <= b - a <= 1.5 for a, b in zip(rise, rise[1:])), rise)  # 1 px a frame above the hand
        frames, _ = rig.windup('windup_debuff', ready, classes.W1_HOLDS['windup_debuff'], self.draw)
        for i in range(4):
            plain = dict(classes.W1_HOLDS['windup_debuff']); plain['bob'] += rig.HOLD_BOB[i]
            self.assertEqual(changed(frames[3 + i], self.draw(plain)), 1 if i in (0, 2) else 0, i)  # a 1 px glint on hold frames 0 and 2

    def test_stagger_ends_in_dazed_and_dazed_sways(self):
        A, T = rig.windup_rows(self.draw, rig.idle_keys(**classes.W1_READY)[0], {}, classes.W1_STAGGER, classes.W1_DAZED)
        self.assertEqual(list(A), ['stagger', 'dazed'])      # a grip may leave wind-ups out
        self.assertEqual((T['stagger'], T['dazed']), (CLIP_SET['stagger'][1], CLIP_SET['dazed'][1]))
        self.assertTrue(same(A['stagger'][4], A['dazed'][0]))  # K3: k4 = dazed frame 0
        self.assertGreater(len({im.tobytes() for im in A['dazed']}), 4)  # K4: it sways
        for im in A['stagger'] + A['dazed']:                # no flash: the figure keeps its outline colour
            self.assertTrue(pixels(im, rig.OUT))

    def test_marks_are_metadata_only(self):
        im = Image.new("RGBA", (4, 4), (1, 2, 3, 4))
        before = im.tobytes()
        self.assertIs(rig.strike(im), im); self.assertIs(rig.loop_start(im), im)
        self.assertEqual(im.tobytes(), before)
        self.assertEqual((rig.strike_frame([Image.new("RGBA", (1, 1)), im]), rig.loop_frame([im])), (1, 0))
        self.assertIsNone(rig.strike_frame([Image.new("RGBA", (1, 1))]))


class Strike(unittest.TestCase):
    def test_every_attack_like_clip_marks_its_strike(self):
        # table 7.2, X4: every attack-like clip of every body (view rows included) has its strike, at most its impact
        for key in all_bodies():
            clips = exported(key)[4]['clips']
            side = {c['id']: c for c in clips if 'of' not in c}
            want = STRIKES.get(kind_of(key), {})
            for c in clips:
                if 'of' in c:
                    for f in ('frames', 'ms', 'loop', 'loop_from', 'strike', 'impact'):
                        self.assertEqual(c.get(f), side[c['of']].get(f), (key, c['id'], f))
                elif c['id'] in export_units.ATTACKS:
                    self.assertEqual(c.get('strike'), want[c['id']], (key, c['id']))
                    self.assertLessEqual(c['strike'], c['impact'], (key, c['id']))
                else:
                    self.assertNotIn('strike', c, (key, c['id']))

    def test_every_row_meets_the_engine_rules(self):
        # Clips::parse L1, L2, S1, S2 on every shipped body
        for key in all_bodies():
            for c in exported(key)[4]['clips']:
                if 'loop_from' in c:
                    self.assertTrue(c['loop'] and 1 <= c['loop_from'] <= c['frames'] - 1, (key, c['id']))
                if 'strike' in c:
                    self.assertTrue(0 <= c['strike'] <= c['frames'] - 1, (key, c['id']))
                    self.assertLessEqual(c['strike'], c.get('impact', c['strike']), (key, c['id']))


class Bodies(unittest.TestCase):
    """Every body of the six classes (R1 the warrior's three grips, R2 the other five) against section 4 and 7.3."""

    def test_every_class_has_windups(self):
        self.assertEqual(set(classes.WINDUP_CLASSES), set(classes.UNITS))
        self.assertGreaterEqual(len([k for k in windup_bodies() if not kind_of(k) == 'warrior']), 13)

    def test_six_rows_with_the_clip_set(self):
        # X1, K7: every body has the six rows after today's side rows, frames, ms, loop and loop_from as section 4
        for key in windup_bodies():
            clips = exported(key)[4]['clips']
            side = [c for c in clips if 'of' not in c]
            self.assertEqual([c['id'] for c in side[-6:]], list(rig.WINDUP_ROWS), key)
            for c in side[-6:]:
                self.assertEqual((c['frames'], c['ms'], c['loop'], c.get('loop_from')), CLIP_SET[c['id']], (key, c['id']))

    def test_windup_holds_the_frame_before_the_strike(self):
        # X2, K2: the windup hold is the main attack's frame strike - 1 (pixel for pixel), side and views
        for key in windup_bodies():
            _, _, rows, _, out = exported(key)
            attack = export_units.main_attack(rows)
            strike = next(c['strike'] for c in out['clips'] if c['id'] == attack)
            for sfx in [''] + (['_down', '_up'] if key in PILOT else []):
                self.assertTrue(same(rows['windup' + sfx][3], rows[attack + sfx][strike - 1]), (key, sfx))

    def test_poses_read_apart(self):
        # X3: the four holds, dazed and idle differ pairwise in at least 6 % at 28 px (side, and both views)
        for key in windup_bodies():
            rows = exported(key)[2]
            for sfx in [''] + (['_down', '_up'] if key in PILOT else []):
                self.assertEqual(export_units.told_apart(rows, sfx), [], (key, sfx))

    def test_today_rows_unchanged(self):
        # X5: with the wind-up rows left out, the sheet is today's: the rows before them pixel-identical, the palette
        # and the width the same; growth +6 rows, +18 with views
        bodies = windup_bodies()
        real, real_classes = rig.windup_rows, classes.WINDUP_CLASSES
        rig.windup_rows = lambda *a, **kw: ({}, {})
        classes.WINDUP_CLASSES = ()
        try:
            before = {k: quiet(export_units.body, k, 'all') for k in bodies}
        finally:
            rig.windup_rows, classes.WINDUP_CLASSES = real, real_classes
        for key in bodies:
            _, _, _, sheet, out = exported(key)
            _, _, _, old_sheet, old = before[key]
            ids = [c['id'] for c in out['clips']]
            old_ids = [c['id'] for c in old['clips']]
            self.assertEqual([i for i in ids if i in old_ids], old_ids, key)  # today's order kept
            self.assertEqual(len(ids) - len(old_ids), 18 if key in PILOT else 6, key)
            self.assertEqual((out['palette'], sheet.width), (old['palette'], old_sheet.width), key)
            for r, cid in enumerate(old_ids):
                n = ids.index(cid)
                self.assertTrue(same(sheet.crop((0, n * rig.W, sheet.width, (n + 1) * rig.W)),
                                     old_sheet.crop((0, r * rig.W, sheet.width, (r + 1) * rig.W))), (key, cid))
            self.assertEqual(sheet.height, (len(old_ids) + (18 if key in PILOT else 6)) * rig.W, key)

    def test_no_blank_frame_and_effect_colours_only(self):
        # X5, K5: every new frame draws the figure; the new rows use the palette, OUT, WHITE and the gold motes only
        for key in windup_bodies():
            _, _, rows, _, out = exported(key)
            pal = export_units.source(key)[4]
            allowed = {tuple(v[:3]) for v in pal.values()} | {rig.OUT, rig.WHITE, rig.GOLDEN, rig.GOLDEN_HI}
            today = [c['id'] for c in out['clips'] if c['id'] not in NEW_IDS]
            allowed |= export_units.colours(export_units.paste(rows, today))
            for cid in rows:
                if cid not in NEW_IDS:
                    continue
                for i, im in enumerate(rows[cid]):
                    self.assertGreater(sum(1 for a in im.getchannel('A').tobytes() if a), 150, (key, cid, i))
                    self.assertFalse({c[:3] for _, c in im.getcolors(1 << 16) if c[3]} - allowed, (key, cid, i))

    def test_two_hand_back_keeps_the_hilt_off_the_legs(self):
        # pilot review (user 2026-10-28): from the back the greatsword's hilt never shows below the torso
        cls, u = export_units.unit_of('warrior/2h_sword')
        rows = exported('warrior/2h_sword')[2]
        trim = u['pal']['trim']
        for cid in rows:
            if cid.endswith('_up'):
                for i, im in enumerate(rows[cid]):
                    pelvis = views.joints(u, 'up')['pelvis'][1]
                    self.assertFalse([p for p in pixels(im, trim) if p[1] > pelvis + 1], (cid, i))

    def test_shipped_sheets_are_the_exporters(self):
        # the shipped sheet and clips.json of every body with wind-ups are what the exporter writes
        for key in windup_bodies():
            layer, folder, _, sheet, out = exported(key)
            base = os.path.join(export_units.WORKSPACE, *export_units.LAYERS[layer], 'units', *folder.split('/'))
            with open(os.path.join(base, 'clips.json'), encoding='utf-8') as f:
                self.assertEqual(json.load(f), out, '%s: re-export the body' % key)
            self.assertTrue(same(Image.open(os.path.join(base, 'sheet.png')).convert('RGBA'), sheet), key)


class Grips(unittest.TestCase):
    """WINDUP1-R2's grips (table 7.2) and the rig options they use, and the 2H heavy hold's views (R1 review fix)."""

    def hold_plain(self, cls, kind, i):
        """Hold frame i of class `cls`'s wind-up `kind` drawn without its effects (the bob only)."""
        kw = dict(classes.CLASS_WINDUPS[cls][1][kind]); kw['bob'] = kw.get('bob', 0.0) + rig.HOLD_BOB[i]
        return rig.pose(classes.UNITS[cls], **kw)

    def test_brawler_heavy_holds_the_haymaker(self):
        # table 7.2: the heavy hold is the haymaker's frame strike - 1 (the clip a heavy blow plays from its strike)
        for key in windup_bodies():
            if kind_of(key) == 'brawler':
                rows = exported(key)[2]
                self.assertTrue(same(rows['windup_heavy'][3], rows['haymaker'][rig.strike_frame(rows['haymaker']) - 1]), key)

    def test_steps_set_a_key_per_hold_frame(self):
        # the mage's heavy hold: the gem's glow 0.6 - 1.0 over the hold (rig.windup steps)
        seen = []

        def draw(kw):
            seen.append(kw.get('glow'))
            im = Image.new('RGBA', (rig.W, rig.W)); im.info['hand'] = (20.0, 20.0)
            return im
        ready = rig.idle_keys(**classes.CLASS_WINDUPS['mage'][0])[0]
        rig.windup('windup_heavy', ready, classes.M_HOLDS['windup_heavy'], draw, steps=classes.M_STEPS['windup_heavy'])
        self.assertEqual(seen[3:], [0.6, 0.8, 1.0, 0.8])
        self.assertEqual(seen[2], classes.M_HOLDS['windup_heavy']['glow'])  # the intro ends on the hold's own keys

    def test_mage_buff_motes_circle_the_gem(self):
        rows = classes.clips('mage')[0]
        spots = []
        for i in range(3, 7):
            im, plain = rows['windup_buff'][i], self.hold_plain('mage', 'windup_buff', i - 3)
            tx, ty = im.info['tip']
            a, p = im.load(), plain.load()  # the motes: the pixels the effect changed (the robe's trim is gold too)
            gold = [(x, y) for y in range(im.height) for x in range(im.width) if a[x, y] != p[x, y]]
            self.assertEqual(len(gold), 2, i)
            self.assertTrue(all(abs(math.hypot(x - tx, y - ty) - rig.MOTE_ORBIT) <= 1.0 for x, y in gold), (i, gold))
            spots.append(tuple(sorted((x - tx, y - ty, a[x, y]) for x, y in gold)))
        self.assertEqual(len(set(spots)), 4)                  # they move round the gem (each its colour)

    def test_ranger_buff_motes_rise_from_the_string_hand(self):
        rows = classes.clips('ranger')[0]
        for i in range(3, 7):
            im = rows['windup_buff'][i]
            ox, oy = im.info['off']
            gold = pixels(im, rig.GOLDEN) + pixels(im, rig.GOLDEN_HI)
            self.assertTrue(gold and all(abs(x - ox) <= 2.5 and y < oy for x, y in gold), (i, gold))

    def test_debuff_glint_on_arrow_and_fist(self):
        # K1: a 1 px glint past the weapon's point on hold frames 0 and 2: the nocked arrow's head, the near fist
        for cls in ('ranger', 'brawler', 'priest', 'rogue', 'mage'):
            rows = classes.clips(cls)[0]
            for i in range(4):
                n = changed(rows['windup_debuff'][3 + i], self.hold_plain(cls, 'windup_debuff', i))
                self.assertEqual(n, 1 if i in (0, 2) else 0, (cls, i))

    def test_new_frames_fit_the_frame_as_today(self):
        # a wind-up or reaction frame touches the 48 px frame's edge only where today's rows of the body already do
        for key in windup_bodies():
            rows = exported(key)[2]

            def edges(im):
                a = im.getchannel('A'); w = im.width
                return {e for e, pts in (('top', [(x, 0) for x in range(w)]), ('left', [(0, y) for y in range(w)]),
                                         ('right', [(w - 1, y) for y in range(w)])) if any(a.getpixel(p) for p in pts)}
            today = set().union(*(edges(im) for cid in rows if cid not in NEW_IDS for im in rows[cid]))
            for cid in rows:
                if cid in NEW_IDS:
                    for i, im in enumerate(rows[cid]):
                        self.assertLessEqual(edges(im), today, (key, cid, i))

    def test_two_hand_heavy_rests_over_the_shoulder_in_views(self):
        # R1 review fix (user 2026-10-29): front and back, the heavy hold rests the weapon over the shoulder as the side
        # view (hands and hilt at shoulder height, clear of the face) and inside the frame; not upright behind the head
        cls, u = export_units.unit_of('warrior/2h_sword')
        rows = exported('warrior/2h_sword')[2]
        for view in views.VIEWS:
            hold = views.W2_WINDUPS['holds'][view]['windup_heavy']
            for i in range(3, 7):
                sh_y = views.joints(u, view, bob=hold['bob'] + rig.HOLD_BOB[i - 3], dy=hold.get('dy', 0.0))['sh_r'][1]
                im = rows['windup_heavy_' + view][i]
                hx, hy = im.info['hand']; tx, ty = im.info['tip']
                self.assertTrue(sh_y - 6 <= hy <= sh_y + 2, (view, i, hy, sh_y))       # at shoulder height
                self.assertGreaterEqual(abs(hx - views.CX), 4.0, (view, i, hx))          # beside the head
                self.assertGreater(abs(tx - hx), 3.0, (view, i))                         # angled, not upright
                self.assertTrue(rig.W > tx > 0 and ty > 0, (view, i))


class Checks(unittest.TestCase):
    """The exporter exits when a body breaks a rule (7.3)."""

    def exits(self, key, needle):
        _CACHE.pop(key, None)
        with self.assertRaises(SystemExit) as e:
            quiet(export_units.body, key, 'all')
        self.assertIn(needle, str(e.exception))

    def test_x1_a_class_with_windups_needs_all_six(self):
        real = classes.CLASS_WINDUPS
        classes.CLASS_WINDUPS = {k: v for k, v in real.items() if k != 'ranger'}  # the ranger's rows left out
        try:
            self.exits('ranger', 'X1')
        finally:
            classes.CLASS_WINDUPS = real
            _CACHE.pop('ranger', None)

    def test_x2_a_hold_far_from_the_strike_exits(self):
        real = rig.windup_rows

        def swapped(draw, ready, holds, *a, **kw):  # the windup holds the heavy pose instead
            return real(draw, ready, dict(holds, windup=holds['windup_heavy']), *a, **kw)
        rig.windup_rows = swapped
        try:
            self.exits('warrior/1h_buckler', 'X2')
        finally:
            rig.windup_rows = real

    def test_x3_alike_poses_exit(self):
        real = classes.WB_HOLDS
        classes.WB_HOLDS = dict(real, windup_buff=real['windup_debuff'])
        try:
            self.exits('warrior/1h_buckler', 'X3')
        finally:
            classes.WB_HOLDS = real

    def test_a_missing_strike_exits(self):
        real = rig.strike_frame
        rig.strike_frame = lambda frames: None
        try:
            self.exits('ranger', 'strike')
        finally:
            rig.strike_frame = real


class Review(unittest.TestCase):
    def test_review_board_and_gifs(self):
        # X6: a pose board and three GIFs per body with wind-up rows; nothing for a body without them
        with tempfile.TemporaryDirectory() as tmp:
            for key, views_too in (('warrior', True), ('warrior/1h_buckler', False)):
                frames, times = variants.clips(key)[:2] if '/' in key else classes.clips(key)
                paths = export_units.windup_review(tmp, key, frames, times)
                name = key.replace('/', '_')
                self.assertEqual([os.path.basename(p) for p in paths],
                                 ['%s_windup_board.png' % name, '%s_windup_strike.gif' % name, '%s_windups.gif' % name,
                                  '%s_stagger_dazed.gif' % name])
                with Image.open(paths[0]) as board:
                    self.assertEqual(board.height, 18 + (12 if views_too else 4) * rig.W * 3, key)
                with Image.open(paths[1]) as g:
                    self.assertEqual(g.width, rig.W * 4 * (3 if views_too else 1), key)
                    # the intro, two holds, then the attack from its strike frame (the GIF merges equal frames and
                    # keeps time in 10 ms steps: the total time, to a step a frame)
                    total = 0
                    for i in range(g.n_frames):
                        g.seek(i); total += g.info['duration']
                    ms = CLIP_SET['windup'][1]
                    want = sum(ms[:3]) + 2 * sum(ms[3:]) + sum(times['attack'][rig.strike_frame(frames['attack']):])
                    self.assertLessEqual(abs(total - want), 10 * g.n_frames, key)
            self.assertEqual(export_units.windup_review(tmp, 'dummy/base', *variants.clips('dummy/base')[:2]), [])


if __name__ == "__main__":
    unittest.main()
