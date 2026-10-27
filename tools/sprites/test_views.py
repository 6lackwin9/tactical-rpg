# Tests of views.py and the exporter's view hook (DIR4-R1, games/emberwake/docs/specs/DIR4.md 6.1 - 6.5, 7.1 L2 - L5).
# Run from anywhere:
#   python tools/sprites/test_views.py
# The exporter hook is proved on a test-only body registered in memory here; the pilot bodies (warrior, its 1H + kite
# look, and warrior/2h_sword, the greatsword on the 2H grip) are checked against the view rules. Nothing is written
# outside a temporary folder.
import contextlib
import copy
import io
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import classes  # noqa: E402
import export_units  # noqa: E402
import rig  # noqa: E402
import variants  # noqa: E402
import views  # noqa: E402

TEST_BODY = "warrior/_viewtest"
PILOT = ("warrior", "warrior/2h_sword")   # DIR4-R1 (L5): both warrior grips, the 2H one with the greatsword
VIEW_IDS = ['idle_down', 'idle_up', 'walk_down', 'walk_up', 'attack_down', 'attack_up']
SPARK = (255, 255, 255)                   # the attack sparks' core (fx only; the rig's own white is rig.WHITE)


def rig_bodies():
    """Every body drawn on the humanoid rig: (key, class, unit)."""
    out = [(k, k, u) for k, u in classes.UNITS.items()]
    out += [(k, b[1], b[2]) for k, b in variants.BODIES.items()]
    out += [(k, cls, variants.look_unit(classes.UNITS[cls], look)) for k, (cls, look) in variants.looks().items()]
    return out


def solid(im):
    """(x, y) of the pixels drawn by the figure: opaque and not the outline."""
    px = im.load()
    return [(x, y) for y in range(im.height) for x in range(im.width) if px[x, y][3] == 255 and px[x, y][:3] != rig.OUT]


def quiet(fn, *a):
    with contextlib.redirect_stdout(io.StringIO()):
        return fn(*a)


def nothing(f, c, layer):
    pass


class Registry(unittest.TestCase):
    """Saves and restores the view registries and the variant bodies around each test."""

    def setUp(self):
        self._parts = copy.copy({k: dict(v) for k, v in views.PARTS.items()})
        self._grips = dict(views.GRIPS)
        self._bodies = dict(variants.BODIES)

    def tearDown(self):
        views.PARTS.clear(); views.PARTS.update(self._parts)
        views.GRIPS.clear(); views.GRIPS.update(self._grips)
        variants.BODIES.clear(); variants.BODIES.update(self._bodies)

    def register_test_body(self, head=views.plain_head, torso=views.plain_torso):
        """A warrior with a bare head and a leather torso (both drawn by the side rig), its parts given stand-in view
        drawings, its sword and shield drawn as nothing, and the warrior grip given the core's rest pose."""
        u = variants.variant('warrior', head='bare', torso='leather')
        variants.BODIES[TEST_BODY] = ('kit', 'warrior', u, None)
        views.PARTS['head']['bare'] = head
        views.PARTS['torso']['leather'] = torso
        views.PARTS['legs']['trousers'] = nothing
        views.PARTS['weapon']['sword'] = nothing
        views.PARTS['offhand']['shield'] = nothing
        views.GRIPS['warrior'] = {}
        return u


class Support(unittest.TestCase):
    def test_no_shipped_body_has_views_yet(self):
        # R5: only the pilot bodies (DIR4-R1) have view drawings for every part and their grip; every other body keeps
        # its side rows only, and the exporter writes no view clip for it
        for key, cls, u in rig_bodies():
            self.assertEqual(views.supports(u, cls), key in PILOT, (key, views.missing(u, cls)))
        clips = quiet(export_units.body, 'ranger', 'all')[4]['clips']
        self.assertFalse([c for c in clips if 'view' in c or 'of' in c])

    def test_other_warrior_looks_keep_side_views(self):
        # L2 / L5: a look gets views only when its grip and its weapon have view drawings; the battle axe and the
        # warhammer wait for DIR4-R2, the buckler look for a buckler drawing
        for key, (cls, look) in variants.looks().items():
            u = variants.look_unit(classes.UNITS[cls], look)
            miss = views.missing(u, cls)
            if key in PILOT:
                self.assertEqual(miss, [], key)
                continue
            self.assertTrue(miss, key)
            if u.get('weapon') not in views.PARTS['weapon']:
                self.assertIn('weapon %s' % u['weapon'], miss)
            if u.get('offhand') and u['offhand'] not in views.PARTS['offhand']:
                self.assertIn('offhand %s' % u['offhand'], miss)

    def test_missing_names_each_part(self):
        u = classes.UNITS['ranger']
        miss = views.missing(u, 'ranger')
        for kind, name in (('head', u['head']), ('torso', u['torso']), ('weapon', u['weapon']), ('back', u['back'])):
            self.assertIn('%s %s' % (kind, name), miss)
        self.assertIn('grip ranger', miss)

    def test_dummy_and_creatures_get_no_views(self):
        for key in list(variants.PROPS) + list(variants.CREATURES):
            self.assertIsNone(export_units.unit_of(key), key)


class Frame(unittest.TestCase):
    def units(self):
        out = [(cls, u) for cls, u in classes.UNITS.items()]
        out += [(b[1], b[2]) for b in variants.BODIES.values() if b[2].get('scale', 1.0) != 1.0][:3]  # scaled variants
        out += [('warrior', variants.look_unit(classes.UNITS['warrior'], '2h_sword'))]                # the 2H pilot look
        return out

    def test_frame_feet_and_columns(self):
        # R2: 48 x 48, feet on GY as the side view (its boot rows), the figure (weapons excluded: drawn here without
        # its weapon and off-hand, and without the swing's effects (slash and sparks), in every pose of its grip)
        # within x 12..36, the feet centred on CX
        pose_view = views.pose_view
        views.pose_view = lambda *a, **kw: pose_view(*a, **{k: v for k, v in kw.items() if k not in ('slash', 'fx')})
        try:
            bodies = [(cls, u) + views.clips(cls, dict(u, weapon=None, offhand=None), stand_in=True)[:1] for cls, u in self.units()]
        finally:
            views.pose_view = pose_view
        for cls, u, A in bodies:
            side = classes.clips(cls, unit=u)[0]['idle'][0]
            boot = {tuple(u['pal']['boot']), tuple(u['pal']['boot_lo'])}
            side_boot = max(y for x, y in solid(side) if side.getpixel((x, y))[:3] in boot)
            for vid, frames in A.items():
                for n, im in enumerate(frames):
                    self.assertEqual(im.size, (rig.W, rig.W))
                    pts = solid(im)
                    xs = [x for x, _ in pts]
                    self.assertGreaterEqual(min(xs), views.X_MIN, (cls, vid, n))
                    self.assertLessEqual(max(xs), views.X_MAX, (cls, vid, n))
                    if vid.startswith('idle'):
                        boots = [(x, y) for x, y in pts if im.getpixel((x, y))[:3] in boot]
                        low = max(y for _, y in boots)
                        # a side foot sits on GY or a row above it (rig.ik stops a straight leg just short)
                        self.assertLessEqual(abs(low - side_boot), 1, (cls, vid, n))
                        feet = [x for x, y in boots if y == low]
                        self.assertLessEqual(abs((min(feet) + max(feet)) / 2 - views.CX), 1.0, (cls, vid, n))

    def test_main_hand_side(self):
        # section 1: the right (main) hand is screen-left in down and screen-right in up; never mirrored
        for cls, u in self.units():
            down, up = views.joints(u, 'down'), views.joints(u, 'up')
            self.assertLess(down['hand_r'][0], views.CX); self.assertGreater(down['hand_l'][0], views.CX)
            self.assertGreater(up['hand_r'][0], views.CX); self.assertLess(up['hand_l'][0], views.CX)
            self.assertAlmostEqual(down['foot_r'][1], rig.GY, delta=0.02); self.assertAlmostEqual(up['foot_l'][1], rig.GY, delta=0.02)

    def test_scale_about_the_feet(self):
        # R2: a scaled body keeps its feet on GY and its head lower by the scale
        base = classes.UNITS['brawler']
        small = variants.variant('brawler', 0.84)
        jb, js = views.joints(base, 'down'), views.joints(small, 'down')
        self.assertAlmostEqual(js['foot_r'][1], rig.GY, delta=0.02)
        self.assertAlmostEqual(rig.GY - js['pelvis'][1], (rig.GY - jb['pelvis'][1]) * 0.84, places=6)

    def test_walk_feet_alternate(self):
        # 6.3: each foot lifts in its half of the gait
        u = classes.UNITS['ranger']
        for view in views.VIEWS:
            frames, _ = views.walk(u, view, stand_in=True)
            lifted = set()
            for im in frames:
                pts = solid(im)
                low_l = max(y for x, y in pts if x < views.CX)
                low_r = max(y for x, y in pts if x > views.CX)
                if low_l < low_r:
                    lifted.add('screen-left')
                if low_r < low_l:
                    lifted.add('screen-right')
            self.assertEqual(lifted, {'screen-left', 'screen-right'}, view)

    def test_unknown_view(self):
        with self.assertRaises(ValueError):
            views.pose_view(classes.UNITS['warrior'], 'side')


class Clips(unittest.TestCase):
    def test_timing_matches_side(self):
        # R3: a view clip has its side clip's frame count and ms
        for cls, u in classes.UNITS.items():
            A, T = classes.clips(cls)
            VA, VT = views.clips(cls, u, stand_in=True)
            attack = views.GRIPS.get(views.grip_key(cls, u), {}).get('attack')
            self.assertEqual(sorted(VA), sorted(['idle_down', 'idle_up', 'walk_down', 'walk_up']
                                                + (['%s_down' % attack[0], '%s_up' % attack[0]] if attack else [])))
            for vid in VA:
                base, view = views.of(vid)
                self.assertIn(view, views.VIEWS)
                self.assertEqual(len(VA[vid]), len(A[base]), (cls, vid))
                self.assertEqual(list(VT[vid]), list(T[base]), (cls, vid))

    def test_palette_only(self):
        # R4: the views draw with the unit's palette, OUT and WHITE only, plus the effect colours its side rows use
        # (the attack sparks)
        for cls, u in classes.UNITS.items():
            allowed = {tuple(v[:3]) for v in u['pal'].values()} | {rig.OUT, rig.WHITE}
            side = classes.clips(cls)[0]
            allowed |= export_units.colours(export_units.paste(side, list(side)))
            VA, _ = views.clips(cls, u, stand_in=True)
            for vid, frames in VA.items():
                for im in frames:
                    used = {c[:3] for _, c in im.getcolors(1 << 16) if c[3]}
                    self.assertFalse(used - allowed, (cls, vid))

    def test_front_and_back_read_apart(self):
        # 6.4: idle_down and idle_up frame 0 differ in at least READABLE of their opaque pixels at the token size
        for cls, u in classes.UNITS.items():
            VA, _ = views.clips(cls, u, stand_in=True)
            self.assertGreaterEqual(views.difference(VA['idle_down'][0], VA['idle_up'][0]), views.READABLE, cls)
            self.assertEqual(views.difference(VA['idle_down'][0], VA['idle_down'][0]), 0.0)

    def test_downsample_keeps_a_solid_colour(self):
        from PIL import Image
        t = views.token(Image.new("RGBA", (rig.W, rig.W), (10, 200, 30, 255)))
        self.assertEqual(t.size, (views.TOKEN, views.TOKEN))
        self.assertEqual(t.getcolors(), [(views.TOKEN * views.TOKEN, (10, 200, 30, 255))])
        self.assertEqual(views.token(Image.new("RGBA", (rig.W, rig.W))).getcolors(), [(views.TOKEN * views.TOKEN, (0, 0, 0, 0))])


class Exporter(Registry):
    def test_view_rows_follow_the_side_rows(self):
        # 3.1, 6.4, 6.5 checks 1-3: the view rows come after the side rows with of, view, frames, ms and loop of
        # their side clip; the side rows are pixel-identical to the body without views; the palette is unchanged
        self.register_test_body()
        _, _, frames, sheet, out = quiet(export_units.body, TEST_BODY, 'all')
        views.GRIPS.pop('warrior')
        _, _, side_frames, side_sheet, side_out = quiet(export_units.body, TEST_BODY, 'all')
        n = len(side_out['clips'])
        self.assertEqual(out['clips'][:n], side_out['clips'])
        self.assertEqual([c['id'] for c in out['clips'][n:]], ['idle_down', 'idle_up', 'walk_down', 'walk_up'])
        by_id = {c['id']: c for c in side_out['clips']}
        for c in out['clips'][n:]:
            side = by_id[c['of']]
            self.assertEqual(c['id'], '%s_%s' % (c['of'], c['view']))
            self.assertEqual((c['frames'], c['ms'], c['loop']), (side['frames'], side['ms'], side['loop']))
        self.assertEqual(out['palette'], side_out['palette'])
        self.assertEqual(sheet.width, side_sheet.width)
        self.assertEqual(sheet.height, side_sheet.height + 4 * rig.W)
        self.assertEqual(sheet.crop((0, 0, side_sheet.width, side_sheet.height)).tobytes(), side_sheet.tobytes())

    def test_written_clips_json(self):
        import json
        self.register_test_body()
        with tempfile.TemporaryDirectory() as tmp:
            argv = sys.argv
            sys.argv = ['export_units.py', TEST_BODY, '--root', tmp]
            try:
                quiet(export_units.main)
            finally:
                sys.argv = argv
            with open(os.path.join(tmp, 'kits', 'rpg', 'assets', 'units', 'warrior', '_viewtest', 'clips.json')) as f:
                clips = json.load(f)['clips']
        views_ = [c for c in clips if 'view' in c]
        self.assertEqual(len(views_), 4)
        self.assertTrue(all('view' not in c for c in clips[:len(clips) - 4]))

    def test_r3_unequal_timing_exits(self):
        self.register_test_body()
        views.GRIPS['warrior'] = {'walk': {'frames': 6}}
        with self.assertRaises(SystemExit) as e:
            quiet(export_units.body, TEST_BODY, 'all')
        self.assertIn('R3', str(e.exception))

    def test_r4_added_colour_exits_naming_it(self):
        def odd(f, c, layer):
            if layer == views.HEAD:
                f.disc(c.j['head'][0], c.j['head'][1], 1.0, (1, 2, 3))
        self.register_test_body()
        views.PARTS['hat']['odd'] = odd
        u = variants.BODIES[TEST_BODY][2]
        u['hat'] = 'odd'
        with self.assertRaises(SystemExit) as e:
            quiet(export_units.body, TEST_BODY, 'all')
        self.assertIn('#010203', str(e.exception))

    def test_r4_added_slot_exits_naming_it(self):
        u = self.register_test_body()
        views.GRIPS.pop('warrior')
        side_slots = quiet(export_units.body, TEST_BODY, 'all')[4]['palette']
        views.GRIPS['warrior'] = {}
        spare = next(k for k in u['pal'] if not k.endswith(('_lo', '_hi')) and k not in side_slots
                     and tuple(u['pal'][k]) not in {tuple(u['pal'][s]) for s in side_slots})

        def spare_dot(f, c, layer):
            if layer == views.FRONT:
                f.disc(c.j['pelvis'][0], c.j['pelvis'][1] - 5, 1.0, c.P[spare])
        views.PARTS['legs']['trousers'] = spare_dot
        with self.assertRaises(SystemExit) as e:
            quiet(export_units.body, TEST_BODY, 'all')
        self.assertIn(spare, str(e.exception))

    def test_unreadable_views_exit(self):
        def same_head(f, c, layer):
            if layer == views.HEAD:
                f.disc(c.j['head'][0], c.j['head'][1], 3.4, c.P['skin'])

        def same_torso(f, c, layer):
            if layer == views.TORSO:
                px, py = c.j['pelvis']
                f.rect(px - 4, py - c.u['tl'], px + 4, py, c.P['main'])
        self.register_test_body(same_head, same_torso)
        with self.assertRaises(SystemExit) as e:
            quiet(export_units.body, TEST_BODY, 'all')
        self.assertIn('6.4', str(e.exception))

    def test_turnaround_board(self):
        with tempfile.TemporaryDirectory() as tmp:
            frames, times = classes.clips('warrior')
            paths = export_units.turnaround(tmp, 'warrior', frames, times)
            self.assertEqual([os.path.basename(p) for p in paths],
                             ['warrior_turnaround.png', 'warrior_walk_4dir.gif', 'warrior_attack_views.gif'])
            frames, times = classes.clips('ranger')
            paths = export_units.turnaround(tmp, 'ranger', frames, times)   # stand-ins, no attack view yet
            self.assertEqual([os.path.basename(p) for p in paths], ['ranger_turnaround.png', 'ranger_walk_4dir.gif'])
            self.assertTrue(all(os.path.getsize(p) > 0 for p in paths))
            self.assertEqual(export_units.turnaround(tmp, 'dummy/base', *variants.dummy_clips()), [])


def pixels(im, colour):
    """(x, y) of the opaque pixels of `im` in `colour`."""
    px = im.load()
    return [(x, y) for y in range(im.height) for x in range(im.width) if px[x, y][3] == 255 and px[x, y][:3] == tuple(colour)]


def mean_x(pts):
    return sum(x for x, _ in pts) / len(pts)


class Pilot(Registry):
    """DIR4-R1's bodies against 6.5 and the view rules (section 1, 6.2, 6.3, L4)."""

    def views_of(self, key):
        cls, u = export_units.unit_of(key)
        return u, views.clips(cls, u)[0]

    def test_six_view_clips_after_the_side_rows(self):
        # 6.5 check 1, R3: the six view clips after the side clips, each with of, view and its side clip's frames,
        # ms and loop
        for key in PILOT:
            clips = quiet(export_units.body, key, 'all')[4]['clips']
            side = [c for c in clips if 'view' not in c]
            self.assertEqual([c['id'] for c in clips[len(side):]], VIEW_IDS, key)
            by_id = {c['id']: c for c in side}
            for c in clips[len(side):]:
                s = by_id[c['of']]
                self.assertEqual((c['view'], c['frames'], c['ms'], c['loop']),
                                 (c['id'].rsplit('_', 1)[1], s['frames'], s['ms'], s['loop']), (key, c['id']))

    def test_side_rows_and_palette_unchanged(self):
        # 6.5 checks 2 and 3, R4: the side rows are pixel-identical to the body exported without views and the
        # palette slots are the same
        for key in PILOT:
            _, _, _, sheet, out = quiet(export_units.body, key, 'all')
            cls, u = export_units.unit_of(key)
            grips = dict(views.GRIPS)
            views.GRIPS.pop(views.grip_key(cls, u))
            _, _, _, side_sheet, side_out = quiet(export_units.body, key, 'all')
            views.GRIPS.clear(); views.GRIPS.update(grips)
            self.assertEqual(out['palette'], side_out['palette'], key)
            self.assertEqual(sheet.height, side_sheet.height + len(VIEW_IDS) * rig.W, key)
            self.assertEqual(sheet.crop((0, 0) + side_sheet.size).tobytes(), side_sheet.tobytes(), key)

    def test_no_blank_frame(self):
        # 6.5 check 4: every view frame draws the figure
        for key in PILOT:
            for vid, frames in self.views_of(key)[1].items():
                for n, im in enumerate(frames):
                    self.assertGreater(len(solid(im)), 150, (key, vid, n))

    def test_front_and_back_read_apart(self):
        # 6.4: idle_down and idle_up frame 0 differ in at least READABLE at the token size
        for key in PILOT:
            A = self.views_of(key)[1]
            self.assertGreaterEqual(views.difference(A['idle_down'][0], A['idle_up'][0]), views.READABLE, key)

    def test_main_weapon_in_the_right_hand(self):
        # section 1: the main weapon is in the right hand, screen-left in down and screen-right in up (the blade's
        # white core; the visor's eye glints excluded); L4: the 2H blade rises over the shoulder on that side
        for key in PILOT:
            u, A = self.views_of(key)
            for view, side in (('down', -1), ('up', 1)):
                j = views.joints(u, view)
                hx, hy = j['head']
                blade = [(x, y) for x, y in pixels(A['idle_' + view][0], rig.WHITE) if abs(x - hx) >= 4 or y > hy + 1]
                self.assertTrue(blade, (key, view))
                self.assertGreater((mean_x(blade) - views.CX) * side, 0, (key, view))
                if key == 'warrior/2h_sword':
                    self.assertLess(min(y for _, y in blade), j['sh_r'][1], (key, view))

    def test_kite_shield_on_the_left_arm(self):
        # 6.2: down, the shield's face on the left arm (screen-right) before the body; up, its back behind the body,
        # only its rim past the left side (screen-left)
        u, A = self.views_of('warrior')
        P = u['pal']
        down, up = A['idle_down'][0], A['idle_up'][0]
        self.assertGreater(mean_x(pixels(down, P['shield'])), views.CX)
        self.assertGreater(len(pixels(down, P['emblem'])), 10)
        self.assertEqual(pixels(up, P['shield']), [])
        back = pixels(up, P['shield_lo'])
        self.assertTrue(back)
        self.assertLess(mean_x(back), views.CX)

    def blade(self, u, view, im):
        """The white blade core (and slash) pixels of a frame, the helm's eye glints left out."""
        hx, hy = views.joints(u, view)['head']
        return [(x, y) for x, y in pixels(im, rig.WHITE) if abs(x - hx) >= 4 or y > hy + 1]

    def test_attack_stays_at_chest_height(self):
        # pilot review (user 2026-10-28): a sword cut, not a hammer swing: the blade never drops below the upper body
        # (the pelvis), and the impact spark lands at chest height on the side clip's impact frame
        for key in PILOT:
            u, A = self.views_of(key)
            cls, _ = export_units.unit_of(key)
            side = classes.clips(cls, unit=u)[0]['attack']
            impact = [n for n, im in enumerate(side) if pixels(im, SPARK)]
            for view in views.VIEWS:
                j = views.joints(u, view)
                frames = A['attack_' + view]
                self.assertEqual([n for n, im in enumerate(frames) if pixels(im, SPARK)], impact, (key, view))
                sparks = pixels(frames[impact[0]], SPARK)
                self.assertGreaterEqual(min(y for _, y in sparks), j['sh_r'][1] - 3, (key, view))
                self.assertLess(max(y for _, y in sparks), j['pelvis'][1], (key, view))
                for n, im in enumerate(frames):
                    self.assertLess(max([y for _, y in self.blade(u, view, im)] or [0]), j['pelvis'][1], (key, view, n))

    def test_greatsword_side_attack_cuts_at_chest_height(self):
        # pilot review round 2 (user 2026-10-28): the greatsword's side attack is the level cut the views use (its
        # blade and spark above the pelvis, the spark on frame 3, the chop's frame count and ms kept); the battle axe and
        # the warhammer keep the chop (their spark lands low, by the feet)
        pelvis = rig.GY - 2 * classes.UNITS['warrior']['leg']
        for look in ('2h_sword', '2h_axe', '2h_hammer'):
            u = variants.look_unit(classes.UNITS['warrior'], look)
            A, T = classes.clips('warrior', unit=u)
            self.assertEqual((len(A['attack']), list(T['attack'])), (7, [120, 190, 45, 150, 120, 110, 110]), look)
            self.assertEqual([n for n, im in enumerate(A['attack']) if pixels(im, SPARK)], [3], look)
            spark = max(y for _, y in pixels(A['attack'][3], SPARK))
            if look == '2h_sword':
                self.assertLess(spark, pelvis)
                for n, im in enumerate(A['attack']):
                    self.assertLess(max([y for _, y in pixels(im, rig.WHITE)] or [0]), pelvis, n)
            else:
                self.assertGreater(spark, pelvis, look)

    def test_one_hand_sword_raised(self):
        # pilot review: in idle and walk the 1H sword is held upright (as the side view): its blade rises above the
        # shoulder and stays above the pelvis
        u, A = self.views_of('warrior')
        for view in views.VIEWS:
            j = views.joints(u, view)
            for clip in ('idle_', 'walk_'):
                for n, im in enumerate(A[clip + view]):
                    ys = [y for _, y in self.blade(u, view, im)]
                    self.assertLess(min(ys), j['sh_r'][1], (clip + view, n))
                    self.assertLess(max(ys), j['pelvis'][1], (clip + view, n))

    def test_two_hand_back_hides_the_hilt(self):
        # pilot review: from the back the greatsword's grip, guard and pommel stay behind the body (no trim colour
        # shows in any up frame); its blade shows over the right shoulder (screen-right)
        u, A = self.views_of('warrior/2h_sword')
        trim = u['pal']['trim']
        for vid in ('idle_up', 'walk_up', 'attack_up'):
            for n, im in enumerate(A[vid]):
                self.assertEqual(pixels(im, trim), [], (vid, n))
        self.assertGreater(mean_x(self.blade(u, 'up', A['idle_up'][0])), views.CX)


if __name__ == "__main__":
    unittest.main()
