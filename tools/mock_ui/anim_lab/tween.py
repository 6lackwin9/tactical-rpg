"""Keyframe in-betweening lab: generated keyframes, in-between frames by code (ART_UI_STUDIO Experiment 5).

Builds the same clips three ways so they can be judged side by side:

- ``keys``: the generated keyframes only, hard cuts, held for the same total time.
- ``flow``: in-betweens from dense optical flow (OpenCV DIS) between neighbouring keyframes; no blending, each
  in-between takes the nearer keyframe's warped pixels, so nothing ghosts.
- ``hybrid``: the halberd is lifted out of every keyframe (inpainted), the body and arms are in-betweened by flow, and
  one canonical halberd (from ``active_drawn``) is drawn back as a rigid prop placed by the hand landmarks
  (``tween_landmarks.json``), its angle interpolated; the hands are laid back over the shaft.

Every frame is snapped to one shared palette and a hard alpha edge, so in-betweens stay pixel art.

Usage:
    python3 -m tools.mock_ui.anim_lab.tween --out /path/to/kaelen_tween_lab.html [--gif-dir art_src/vn_stage]
"""
import argparse
import base64
import io
import json
import math
from pathlib import Path

import cv2
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
SIZE = 340
GROUND_Y = 324  # boots plant here in every aligned pose (ART_UI_STUDIO 3.3)
BG = (34, 33, 44)


# ---------------------------------------------------------------- loading and clean-up

def load_keys() -> dict:
    uris = json.loads((HERE / "kaelen_4pose_uris.json").read_text(encoding="utf-8"))
    out = {}
    for key, uri in uris.items():
        im = Image.open(io.BytesIO(base64.b64decode(uri.split(",", 1)[1]))).convert("RGBA")
        out[key] = clean(np.array(im))
    return out


def clean(rgba: np.ndarray) -> np.ndarray:
    """Hard alpha edge and magenta despill (the keyframes were keyed from #FF00FF JPEGs)."""
    out = rgba.copy()
    a = out[..., 3]
    out[..., 3] = np.where(a >= 128, 255, 0).astype(np.uint8)
    rgb = out[..., :3].astype(np.int16)
    spill = np.minimum(rgb[..., 0], rgb[..., 2]) - rgb[..., 1]
    fix = (spill > 12) & (out[..., 3] > 0)
    for c in (0, 2):
        ch = rgb[..., c]
        ch[fix] -= spill[fix]
    out[..., :3] = np.clip(rgb, 0, 255).astype(np.uint8)
    out[out[..., 3] == 0, :3] = 0
    return out


def build_palette(frames: list, colours: int = 96) -> Image.Image:
    px = np.concatenate([f[f[..., 3] > 0][:, :3] for f in frames])
    strip = Image.fromarray(px.reshape(1, -1, 3))
    return strip.quantize(colors=colours, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)


def snap(rgba: np.ndarray, pal: Image.Image) -> np.ndarray:
    rgb = Image.fromarray(np.ascontiguousarray(rgba[..., :3]))
    q = np.array(rgb.quantize(palette=pal, dither=Image.Dither.NONE).convert("RGB"))
    out = np.dstack([q, np.where(rgba[..., 3] >= 128, 255, 0).astype(np.uint8)])
    out[out[..., 3] == 0, :3] = 0
    return out


# ---------------------------------------------------------------- optical-flow in-betweens

_DIS = None


def _flows(a: np.ndarray, b: np.ndarray):
    global _DIS
    if _DIS is None:
        _DIS = cv2.DISOpticalFlow_create(cv2.DISOPTICAL_FLOW_PRESET_MEDIUM)
    def grey(f):
        g = cv2.cvtColor(np.ascontiguousarray(f[..., :3]), cv2.COLOR_RGB2GRAY)
        g[f[..., 3] == 0] = 0
        return g
    ga, gb = grey(a), grey(b)
    return _DIS.calc(ga, gb, None), _DIS.calc(gb, ga, None)


def flow_tween(a: np.ndarray, b: np.ndarray, t: float, flows=None) -> np.ndarray:
    """Warp a forward by t and b back by 1 - t; take the nearer key's pixels, fill its holes from the other."""
    fab, fba = flows if flows is not None else _flows(a, b)
    h, w = a.shape[:2]
    gx, gy = np.meshgrid(np.arange(w, dtype=np.float32), np.arange(h, dtype=np.float32))
    wa = cv2.remap(a, gx - t * fab[..., 0], gy - t * fab[..., 1], cv2.INTER_NEAREST, borderValue=0)
    wb = cv2.remap(b, gx - (1 - t) * fba[..., 0], gy - (1 - t) * fba[..., 1], cv2.INTER_NEAREST, borderValue=0)
    near, far = (wa, wb) if t < 0.5 else (wb, wa)
    out = near.copy()
    # Fill only cracks: pixels within 2 px of the near key's warped silhouette. Filling every hole from the far key
    # brings its limbs along too (a third leg in the walk).
    close = cv2.dilate((near[..., 3] > 0).astype(np.uint8), np.ones((5, 5), np.uint8)) > 0
    hole = close & (near[..., 3] == 0) & (far[..., 3] > 0)
    out[hole] = far[hole]
    return out


# ---------------------------------------------------------------- hybrid: rigid halberd + flow body

class Axis:
    """Shaft axis of one keyframe: origin at the socket, d toward the blade, n its normal."""

    def __init__(self, lm: dict):
        self.butt = np.array(lm["butt"], float)
        self.socket = np.array(lm["socket"], float)
        v = self.socket - self.butt
        self.length = float(np.hypot(*v))
        self.d = v / self.length
        self.n = np.array([-self.d[1], self.d[0]])
        self.angle = math.atan2(self.d[1], self.d[0])

    def local(self, xs, ys):
        px, py = xs - self.socket[0], ys - self.socket[1]
        return px * self.d[0] + py * self.d[1], px * self.n[0] + py * self.n[1]

    def proj(self, p) -> np.ndarray:
        p = np.array(p, float)
        return self.butt + np.dot(p - self.butt, self.d) * self.d


BLADE_BACK = -14   # the blade sprite starts this far behind the socket (the collar)
BLADE_HALF = 36    # half-width of the blade region across the shaft
SHAFT_HALF = 3.0   # half-width of the drawn shaft (colour profile)
STRIP_HALF = 5.5   # half-width of the band lifted out of each keyframe (the shaft plus its dark outline)
STRIP_BACK = -24   # the collar starts this far behind the socket when lifting the blade out
HAND_R = 11        # hand disc kept over the shaft


def _hand_mask(f: np.ndarray, centres) -> np.ndarray:
    h, w = f.shape[:2]
    ys, xs = np.mgrid[0:h, 0:w]
    disc = np.zeros((h, w), bool)
    for c in centres:
        disc |= (xs - c[0]) ** 2 + (ys - c[1]) ** 2 <= HAND_R ** 2
    rgb = f[..., :3].astype(np.int16)
    skin_or_bracer = (rgb[..., 0] - rgb[..., 2] > 50) & (rgb[..., 0] - rgb[..., 1] > 22)
    return disc & skin_or_bracer & (f[..., 3] > 0)


def refine_landmarks(f: np.ndarray, lm: dict) -> dict:
    """Fit the shaft line to the painted shaft pixels near the hand-read line (a robust line fit), then put butt and
    socket on it, so the strip mask follows the real shaft instead of a grid estimate."""
    ax = Axis(lm)
    h, w = f.shape[:2]
    ys, xs = np.mgrid[0:h, 0:w].astype(float)
    u, v = ax.local(xs, ys)
    rgb = f[..., :3].astype(np.int16)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    brown = (r >= g) & (g >= b - 6) & (r - b >= 8) & (r - b <= 80) & (np.maximum(r, np.maximum(g, b)) < 150)
    hands = _hand_mask(f, [lm["grip_front"], lm["grip_back"]])
    cand = (np.abs(v) <= 7) & (u >= -ax.length - 6) & (u <= BLADE_BACK) & brown & ~hands & (f[..., 3] > 0)
    pts = np.column_stack([xs[cand], ys[cand]]).astype(np.float32)
    if len(pts) < 20:
        return lm
    vx, vy, x0, y0 = cv2.fitLine(pts, cv2.DIST_HUBER, 0, 0.01, 0.01).ravel()
    d = np.array([vx, vy], float)
    if np.dot(d, ax.d) < 0:
        d = -d
    p0 = np.array([x0, y0], float)
    on = lambda p: p0 + np.dot(np.array(p, float) - p0, d) * d
    s = (pts - p0) @ d
    out = dict(lm)
    out["butt"] = list(p0 + (s.min() - 1) * d)
    out["socket"] = list(on(lm["socket"]))
    return out


def strip_weapon(f: np.ndarray, lm: dict) -> np.ndarray:
    ax = Axis(lm)
    h, w = f.shape[:2]
    ys, xs = np.mgrid[0:h, 0:w].astype(float)
    u, v = ax.local(xs, ys)
    shaft = (u >= -ax.length - 7) & (u <= 0) & (np.abs(v) <= STRIP_HALF)
    blade = (u >= STRIP_BACK) & (np.abs(v) <= BLADE_HALF) & (u <= 90)
    hands = _hand_mask(f, [lm["grip_front"], lm["grip_back"]])
    remove = (shaft | blade) & ~hands & (f[..., 3] > 0)
    rgb = np.ascontiguousarray(f[..., :3])
    m = remove.astype(np.uint8) * 255
    rgb2 = cv2.inpaint(rgb, m, 3, cv2.INPAINT_TELEA)
    a2 = cv2.inpaint(np.ascontiguousarray(f[..., 3]), m, 3, cv2.INPAINT_TELEA)
    out = f.copy()
    out[remove, :3] = rgb2[remove]
    out[remove, 3] = np.where(a2[remove] >= 128, 255, 0)
    out[out[..., 3] == 0, :3] = 0
    return out


class Halberd:
    """The canonical halberd: blade pixels sampled from the canonical keyframe, shaft drawn from a colour profile."""

    def __init__(self, canon: np.ndarray, lm: dict, clean_back: bool = False, outline: bool = False):
        """``clean_back`` keeps only the shaft band behind the socket (drops hand or sleeve outline caught next to
        the collar); ``outline`` draws a 1 px dark edge along the shaft like the painted one. Both default off so
        the clips built before them stay byte for byte the same."""
        self.img = canon
        self.ax = Axis(lm)
        self.outline = outline
        h, w = canon.shape[:2]
        ys, xs = np.mgrid[0:h, 0:w].astype(float)
        u, v = self.ax.local(xs, ys)
        hands = _hand_mask(canon, [lm["grip_front"], lm["grip_back"]])
        self.blade_mask = (u >= BLADE_BACK) & (np.abs(v) <= BLADE_HALF) & (u <= 90) & (canon[..., 3] > 0) & ~hands
        if clean_back:
            self.blade_mask &= (u >= 0) | (np.abs(v) <= SHAFT_HALF + 1.5)
        # Shaft profile across the shaft: median colour per whole-pixel offset, sampled where no hand covers it.
        prof = {}
        free = (u > -self.ax.length + 4) & (u < BLADE_BACK - 2) & ~_hand_mask(canon, [lm["grip_front"], lm["grip_back"]])
        band = free & (np.abs(v) <= SHAFT_HALF) & (canon[..., 3] > 0)
        vr = np.rint(v).astype(int)
        for off in range(-3, 4):
            sel = band & (vr == off)
            if sel.sum() > 8:
                prof[off] = np.median(canon[sel][:, :3], axis=0).astype(np.uint8)
        self.profile = prof
        self.u_front = float(np.dot(self.ax.proj(lm["grip_front"]) - self.ax.socket, self.ax.d))

    def draw(self, canvas: np.ndarray, grip_front, angle: float, length: float | None = None) -> np.ndarray:
        """Draw the halberd so its canonical front-grip point sits on grip_front at the given angle; ``length`` (butt to
        socket) overrides the canonical shaft length."""
        length = self.ax.length if length is None else length
        d = np.array([math.cos(angle), math.sin(angle)])
        n = np.array([-d[1], d[0]])
        socket = np.array(grip_front, float) - self.u_front * d
        h, w = canvas.shape[:2]
        ys, xs = np.mgrid[0:h, 0:w].astype(float)
        px, py = xs - socket[0], ys - socket[1]
        u, v = px * d[0] + py * d[1], px * n[0] + py * n[1]
        out = canvas.copy()
        if self.outline:
            edge = (u >= -length - 1) & (u < BLADE_BACK + 2) & (np.abs(v) <= SHAFT_HALF + 1.5)
            out[edge, :3] = (26, 19, 17)
            out[edge, 3] = 255
        # shaft
        shaft = (u >= -length) & (u < BLADE_BACK + 2) & (np.abs(v) <= SHAFT_HALF + 0.5)
        vr = np.clip(np.rint(v), -3, 3).astype(int)
        for off, col in self.profile.items():
            sel = shaft & (vr == off)
            out[sel, :3] = col
            out[sel, 3] = 255
        # blade: map back into the canonical frame, nearest sample
        region = (u >= BLADE_BACK) & (u <= 90) & (np.abs(v) <= BLADE_HALF)
        cx = self.ax.socket[0] + u * self.ax.d[0] + v * self.ax.n[0]
        cy = self.ax.socket[1] + u * self.ax.d[1] + v * self.ax.n[1]
        ix, iy = np.rint(cx).astype(int), np.rint(cy).astype(int)
        ok = region & (ix >= 0) & (ix < w) & (iy >= 0) & (iy < h)
        sel = np.zeros_like(ok)
        sel[ok] = self.blade_mask[iy[ok], ix[ok]]
        out[sel] = self.img[iy[sel], ix[sel]]
        return out


def _lerp(a, b, t):
    return np.array(a, float) * (1 - t) + np.array(b, float) * t


def _lerp_angle(a, b, t):
    dlt = (b - a + math.pi) % (2 * math.pi) - math.pi
    return a + dlt * t


class Hybrid:
    def __init__(self, keys: dict, landmarks: dict):
        self.lm = {k: refine_landmarks(keys[k], lm) for k, lm in landmarks["keys"].items()}
        canon = landmarks["canonical"]
        self.halberd = Halberd(keys[canon], self.lm[canon])
        self.bodies = {k: strip_weapon(keys[k], self.lm[k]) for k in self.lm}
        # Stop-frame keys keep their own painted weapon on the key itself; in-betweens blend the shaft length from it.
        self.own = set(landmarks.get("own_weapon", []))
        self.keys = keys
        self._flows = {}

    def _length(self, k: str) -> float:
        return Axis(self.lm[k]).length if k in self.own else self.halberd.ax.length

    def _transplant_hand(self, body: np.ndarray, ka: str, kb: str, t: float, grip, ang: float) -> np.ndarray:
        """Erase the front hand wherever flow left it (near either key's grip) and inpaint, so the nearer key's
        painted hand can be laid back on the rigid shaft afterwards."""
        la, lb = self.lm[ka], self.lm[kb]
        h, w = body.shape[:2]
        ys, xs = np.mgrid[0:h, 0:w]
        rgb = body[..., :3].astype(np.int16)
        skin_or_bracer = (rgb[..., 0] - rgb[..., 2] > 50) & (rgb[..., 0] - rgb[..., 1] > 22) & (body[..., 3] > 0)
        near = np.zeros((h, w), bool)
        for c in (la["grip_front"], lb["grip_front"], grip):
            near |= (xs - c[0]) ** 2 + (ys - c[1]) ** 2 <= (HAND_R + 5) ** 2
        m = (near & skin_or_bracer).astype(np.uint8) * 255
        out = body.copy()
        out[..., :3] = cv2.inpaint(np.ascontiguousarray(body[..., :3]), m, 3, cv2.INPAINT_TELEA)
        return out

    def _hand_patch(self, canvas: np.ndarray, k: str, grip, ang: float) -> np.ndarray:
        """Lay key k's painted front hand onto the drawn shaft: rotate it about its grip by the shaft's turn and move
        its grip onto ``grip`` (inverse mapping, nearest sample)."""
        lm, key = self.lm[k], self.keys[k]
        ax = Axis(lm)
        src = ax.proj(lm["grip_front"])
        mask = _hand_mask(key, [lm["grip_front"]])
        da = ang - ax.angle
        c, s = math.cos(-da), math.sin(-da)
        h, w = canvas.shape[:2]
        r = HAND_R + 3
        out = canvas.copy()
        for ty in range(int(grip[1]) - r - 2, int(grip[1]) + r + 3):
            for tx in range(int(grip[0]) - r - 2, int(grip[0]) + r + 3):
                if not (0 <= tx < w and 0 <= ty < h):
                    continue
                dx, dy = tx - grip[0], ty - grip[1]
                sx, sy = int(round(src[0] + c * dx - s * dy)), int(round(src[1] + s * dx + c * dy))
                if 0 <= sx < w and 0 <= sy < h and mask[sy, sx]:
                    out[ty, tx] = key[sy, sx]
        return out

    def frame(self, ka: str, kb: str, t: float, hand: bool = False) -> np.ndarray:
        if t <= 0 and ka in self.own:
            return self.keys[ka]
        if t >= 1 and kb in self.own:
            return self.keys[kb]
        a, b = self.bodies[ka], self.bodies[kb]
        if t <= 0:
            body = a
        elif t >= 1:
            body = b
        else:
            if (ka, kb) not in self._flows:
                self._flows[(ka, kb)] = _flows(a, b)
            body = flow_tween(a, b, t, self._flows[(ka, kb)])
        la, lb = self.lm[ka], self.lm[kb]
        axa, axb = Axis(la), Axis(lb)
        grip = _lerp(axa.proj(la["grip_front"]), axb.proj(lb["grip_front"]), t)
        ang = _lerp_angle(axa.angle, axb.angle, t)
        length = (1 - t) * self._length(ka) + t * self._length(kb)
        if hand and 0 < t < 1:
            body = self._transplant_hand(body, ka, kb, t, grip, ang)
            withw = self.halberd.draw(body, grip, ang, length)
            back = _hand_mask(body, [_lerp(la["grip_back"], lb["grip_back"], t)])
            withw[back] = body[back]
            return self._hand_patch(withw, ka if t < 0.5 else kb, grip, ang)
        withw = self.halberd.draw(body, grip, ang, length)
        hands = _hand_mask(body, [_lerp(la["grip_front"], lb["grip_front"], t), _lerp(la["grip_back"], lb["grip_back"], t)])
        withw[hands] = body[hands]
        return withw


# ---------------------------------------------------------------- code-only passive motion

def breathe(f: np.ndarray, amp_px: float) -> np.ndarray:
    """Stretch everything above the boots by amp_px, anchored at the ground (PORTRAITFX breath, full body)."""
    top = int(np.nonzero(f[..., 3].any(axis=1))[0][0])
    span = GROUND_Y - top
    out = np.zeros_like(f)
    for y in range(SIZE):
        if y >= GROUND_Y - 30:  # boots and shins stay planted
            out[y] = f[y]
            continue
        # target row y samples the source row that the stretch moved here
        k = (GROUND_Y - 30 - y) / (span - 30)
        src = y + amp_px * k
        sy = int(round(src))
        if 0 <= sy < SIZE:
            out[y] = f[sy]
    return out


# ---------------------------------------------------------------- clips

def ease(t: float) -> float:
    return t * t * (3 - 2 * t)


def clip_frames(method: str, plan: list, keys: dict, hyb: Hybrid, flow_cache: dict):
    """plan: list of (key_a, key_b, [(t, ms), ...]). Returns [(rgba, ms)].

    For ``keys`` each in-between's time is added to the nearer keyframe's hold, so every method runs the same length.
    """
    out = []
    for ka, kb, steps in plan:
        for t, ms in steps:
            if method == "keys":
                k = ka if t < 0.5 else kb
                if out and out[-1][2] == k:
                    out[-1] = (out[-1][0], out[-1][1] + ms, k)
                else:
                    out.append((keys[k], ms, k))
                continue
            if method in ("hybrid", "hybrid_hand"):
                img = hyb.frame(ka, kb, t, hand=method == "hybrid_hand")
            elif t <= 0:
                img = keys[ka]
            elif t >= 1:
                img = keys[kb]
            else:
                if (ka, kb) not in flow_cache:
                    flow_cache[(ka, kb)] = _flows(keys[ka], keys[kb])
                img = flow_tween(keys[ka], keys[kb], t, flow_cache[(ka, kb)])
            out.append((img, ms, None))
    return [(img, ms) for img, ms, _ in out]


def tweens(n: int, ms: int, eased=True):
    return [((ease if eased else (lambda x: x))(i / (n + 1)), ms) for i in range(1, n + 1)]


def sheet_plan(prefix: str, n: int, step_ms: int, hold_ms: int = 0, cycle: bool = False):
    """Plan through <prefix>_f1.._fN with one flow in-between per gap. ``cycle`` loops back from fN to f1 (walk,
    sneak); otherwise the action holds on f1 for ``hold_ms`` and returns to f1 at the end (attacks)."""
    half = step_ms // 2
    keys = [f"{prefix}_f{i}" for i in range(1, n + 1)]
    pairs = list(zip(keys, keys[1:] + ([keys[0]] if cycle else [])))
    plan = [(a, b, [(0, half), (0.5, half)]) for a, b in pairs]
    if not cycle:
        plan[0] = (keys[0], keys[1], [(0, hold_ms), (0.5, half)])
        plan.append((keys[-1], keys[0], [(0, half), (0.5, half)]))
    return plan


GUARD, WIND, POKE, CLEAVE = "active_drawn", "drawn_kf_windup", "drawn_kf_poke", "drawn_kf_cleave"
# generated breakdown keys (ART_UI_STUDIO 3.5); drawn_kf_halfdraw (first take) copied the guard stance, the retake r2 is used
MID, HALF = "drawn_kf_midswing", "drawn_kf_halfdraw_r2"
CLIPS = {
    "poke": {
        "label": "Drawn: guard, wind-up, poke, recover",
        "note": "Small arm moves. Watch the halberd's blade: the generated keys draw it differently from key to key.",
        "methods": ["keys", "flow", "hybrid"],
        "plan": [
            (GUARD, WIND, [(0, 500)] + tweens(2, 70)),
            (WIND, POKE, [(0, 260)] + [(0.5, 45)]),
            (POKE, GUARD, [(0, 320)] + tweens(3, 80)),
        ],
    },
    "cleave": {
        "label": "Drawn: guard, wind-up, cleave, recover",
        "note": "Stress test: the front hand travels about 90 px between wind-up and cleave, too far for flow alone.",
        "methods": ["keys", "flow", "hybrid"],
        "plan": [
            (GUARD, WIND, [(0, 500)] + tweens(2, 70)),
            (WIND, CLEAVE, [(0, 280)] + [(0.33, 40), (0.67, 40)]),
            (CLEAVE, GUARD, [(0, 340)] + tweens(3, 85)),
        ],
    },
    "cleave_bd": {
        "label": "Drawn: cleave with the generated mid-swing breakdown key",
        "note": "Same timing as the cleave above; the swing passes through drawn_kf_midswing instead of guessing it.",
        "methods": ["keys", "flow", "hybrid"],
        "plan": [
            (GUARD, WIND, [(0, 500)] + tweens(2, 70)),
            (WIND, MID, [(0, 280)] + [(0.5, 40)]),
            (MID, CLEAVE, [(0, 40)]),
            (CLEAVE, GUARD, [(0, 340)] + tweens(3, 85)),
        ],
    },
    "walk": {
        "label": "Walk, one-hand carry: 4 keys against 8 frames",
        "note": "Only the legs differ between the walk keys, so flow fills the passing frames.",
        "methods": ["keys", "flow"],
        "plan": [(f"walk_1h_kf{i}", f"walk_1h_kf{(i + 1) % 4}", [(0, 110), (0.5, 110)]) for i in range(4)],
    },
    "draw_weapon": {
        "label": "Passive to drawn: drawing the weapon",
        "note": "Stress test: stance, facing and weapon all change. This is the case for a generated breakdown key or local video in-betweening.",
        "methods": ["keys", "flow"],
        "plan": [
            ("passive_relaxed", GUARD, [(0, 700)] + tweens(3, 90)),
            (GUARD, "passive_relaxed", [(0, 700)] + tweens(3, 90)),
        ],
    },
    "draw_weapon_bd": {
        "label": "Passive to drawn with the generated half-draw breakdown key",
        "note": "Same timing as above, through drawn_kf_halfdraw_r2: head a little lower, back boot already out, front boot halfway. hybrid_hand lays the nearer key's painted front hand back on the rigid shaft.",
        "methods": ["keys", "flow", "hybrid", "hybrid_hand"],
        "plan": [
            ("passive_relaxed", HALF, [(0, 700)] + tweens(1, 90)),
            (HALF, GUARD, [(0, 90)] + tweens(1, 90)),
            (GUARD, HALF, [(0, 700)] + tweens(1, 90)),
            (HALF, "passive_relaxed", [(0, 90)] + tweens(1, 90)),
        ],
    },
    "draw_weapon_sheet": {
        "label": "Passive to drawn from one generated 6-frame sheet",
        "note": "The six frames of kaelen_draw_sheet_6frame (one scale, planted on the ground line) plus the generated lift frame (kaelen_draw_sheet_lift_r2) between frames 1 and 2: one halberd, hands on the shaft.",
        "methods": ["keys", "flow"],
        "plan": (
            [("sheet_draw_f1", "sheet_draw_lift", [(0, 700), (0.5, 45)]),
             ("sheet_draw_lift", "sheet_draw_f2", [(0, 45), (0.5, 45)])]
            + [(f"sheet_draw_f{i}", f"sheet_draw_f{i + 1}", [(0, 45), (0.5, 45)]) for i in range(2, 6)]
            + [("sheet_draw_f6", "sheet_draw_f5", [(0, 700), (0.5, 45)])]
            + [(f"sheet_draw_f{i}", f"sheet_draw_f{i - 1}", [(0, 45), (0.5, 45)]) for i in range(5, 2, -1)]
            + [("sheet_draw_f2", "sheet_draw_lift", [(0, 45), (0.5, 45)]),
               ("sheet_draw_lift", "sheet_draw_f1", [(0, 45), (0.5, 45)])]
        ),
    },
    "slash_sheet": {
        "label": "Drawn: slash from the generated 6-frame sheet",
        "note": "kaelen_slash_sheet_6frame: guard, raise, wind-up, (mid-swing), cleave, recover. Frame 4 came out as a second wind-up, so frame 4 to 5 is a 180 degree jump.",
        "methods": ["keys", "flow"],
        "plan": sheet_plan("sheet_slash", 6, 110, hold_ms=500),
    },
    "poke_sheet": {
        "label": "Drawn: poke from the generated 6-frame sheet",
        "note": "kaelen_poke_sheet_6frame: guard, chamber, drive, full thrust, retract, recover.",
        "methods": ["keys", "flow"],
        "plan": sheet_plan("sheet_poke", 6, 110, hold_ms=500),
    },
    "walk_sheet": {
        "label": "Walk from the generated 8-frame sheet",
        "note": "kaelen_walk_sheet_8frame: contact, down, passing, up and their mirrors, looped.",
        "methods": ["keys", "flow"],
        "plan": sheet_plan("sheet_walk", 8, 110, cycle=True),
    },
    "sneak_sheet": {
        "label": "Sneak from the generated 6-frame sheet",
        "note": "kaelen_sneak_sheet_6frame: three crouched steps and their mirrors, looped.",
        "methods": ["keys", "flow"],
        "plan": sheet_plan("sheet_sneak", 6, 180, cycle=True),
    },
}


def passive_frames(keys: dict):
    """Passive idle by code only: a slow breath (2 px, 3.6 s) on the stop frame."""
    f = keys["passive_relaxed"]
    n = 12
    frames = []
    for i in range(n):
        b = (1 - math.cos(2 * math.pi * i / n)) / 2
        frames.append((breathe(f, 2.0 * b), 300))
    return frames


CODE_GUARD = "sheet_draw_f6"   # the signed-off draw ends here, so code attacks start from it
HIP_Y = 205                    # rows below stay planted when the upper body leans
# (lean px at the head, slide px along the shaft, turn deg, ms): wind-up, thrust, hold, recover
POKE_PLAN = [(0, 0, 0, 500), (-2, -5, -1, 60), (-4, -10, -2, 60), (-5, -12, -2, 140),
             (3, 12, 1, 35), (9, 30, 2, 35), (10, 34, 2, 260),
             (7, 22, 1, 60), (4, 12, 0, 60), (1, 4, 0, 60), (0, 0, 0, 400)]


def code_poke(keys: dict, landmarks: dict):
    """A poke from one guard frame by code only: the upper body leans from the hips (boots planted) and the rigid
    halberd, lifted out of the frame, slides along its own axis through both hands, which are laid back over it.
    No generated attack keys (ART_UI_STUDIO 3.5: small attacks by code, arcs by sprites)."""
    f = keys[CODE_GUARD]
    lm = refine_landmarks(f, landmarks["keys"][CODE_GUARD])
    body = strip_weapon(f, lm)
    hal = Halberd(f, lm, clean_back=True, outline=True)
    ax = Axis(lm)
    top = int(np.nonzero(f[..., 3].any(axis=1))[0][0])

    def shift(y: float, lean: float) -> float:
        return 0.0 if y >= HIP_Y else lean * (HIP_Y - y) / (HIP_Y - top)

    def leaned(img: np.ndarray, lean: float) -> np.ndarray:
        out = np.zeros_like(img)
        for y in range(SIZE):
            dx = int(round(shift(y, lean)))
            if dx >= 0:
                out[y, dx:] = img[y, :SIZE - dx]
            else:
                out[y, :SIZE + dx] = img[y, -dx:]
        return out

    frames = []
    for lean, slide, turn, ms in POKE_PLAN:
        b = leaned(body, lean)
        g = ax.proj(lm["grip_front"])
        g = g + np.array([shift(g[1], lean), 0.0]) + slide * ax.d
        img = hal.draw(b, g, ax.angle + math.radians(turn))
        hands = _hand_mask(b, [np.array(lm[k], float) + [shift(lm[k][1], lean), 0.0] for k in ("grip_front", "grip_back")])
        img[hands] = b[hands]
        frames.append((img, ms))
    return frames


# ---------------------------------------------------------------- output

def to_uri(rgba: np.ndarray) -> str:
    buf = io.BytesIO()
    Image.fromarray(rgba).save(buf, format="PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()


def on_bg(rgba: np.ndarray) -> Image.Image:
    bg = Image.new("RGBA", (SIZE, SIZE), BG + (255,))
    bg.alpha_composite(Image.fromarray(rgba))
    return bg.convert("RGB")


def build(out: Path, gif_dir: Path | None) -> None:
    keys = load_keys()
    landmarks = json.loads((HERE / "tween_landmarks.json").read_text(encoding="utf-8"))
    hyb = Hybrid(keys, landmarks)
    # sheet_* frames snap to the palette but do not shape it, so adding a sheet leaves every other clip unchanged
    pal = build_palette([keys[k] for k in keys
                         if (not k.startswith("drawn_") or k.startswith("drawn_kf")) and not k.startswith("sheet_")])
    flow_cache = {}
    images, index = [], {}

    def add(img):
        img = snap(img, pal)
        h = img.tobytes().__hash__()
        if h not in index:
            index[h] = len(images)
            images.append(img)
        return index[h]

    data = {"clips": []}
    data["clips"].append({"id": "passive", "label": "Passive stop frame: breath by code",
                          "note": "One generated frame, no in-betweens: everything above the shins rises 2 px over 3.6 s.", "rows": [
        {"method": "code", "frames": [[add(img), ms] for img, ms in passive_frames(keys)]}]})
    for cid, c in CLIPS.items():
        rows = []
        for m in c["methods"]:
            fr = clip_frames(m, c["plan"], keys, hyb, flow_cache)
            rows.append({"method": m, "frames": [[add(img), ms] for img, ms in fr]})
        data["clips"].append({"id": cid, "label": c["label"], "note": c["note"], "rows": rows})
    data["clips"].append({"id": "poke_code", "label": "Drawn: poke by code only, from the sheet's guard",
                          "note": "Hip lean plus the rigid halberd sliding through the hands; no generated attack keys.",
                          "rows": [{"method": "code", "frames": [[add(img), ms] for img, ms in code_poke(keys, landmarks)]}]})
    data["images"] = [to_uri(im) for im in images]
    html = (HERE / "tween_template.html").read_text(encoding="utf-8").replace("__TWEEN_JSON__", json.dumps(data))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html, encoding="utf-8")
    print(f"Built tween lab: {len(images)} unique frames, {len(html):,} bytes -> {out}")
    if gif_dir:
        gif_dir.mkdir(parents=True, exist_ok=True)
        for clip in data["clips"]:
            for row in clip["rows"]:
                seq = [on_bg(images[i]) for i, _ in row["frames"]]
                dur = [ms for _, ms in row["frames"]]
                p = gif_dir / f"kaelen_tween_{clip['id']}_{row['method']}.gif"
                seq[0].save(p, save_all=True, append_images=seq[1:], duration=dur, loop=0, disposal=1)
        print(f"GIFs -> {gif_dir}")


def slice_sheet(sheet: Path, cols: int, rows: int, prefix: str, size_like: str, x_shift: int = 0) -> dict:
    """Cut a generated sprite sheet (flat #FF00FF, uniform cells) into aligned 340 px frames for the key bundle.

    One scale for every frame: frame 1's height matches the ``size_like`` key's height. Every frame is planted on
    GROUND_Y and keeps its x inside its cell, shifted so frame 1's boots centre on the ``size_like`` key's boots, so
    steps and stance changes survive. Returns {f"{prefix}_f{i}": rgba} in reading order."""
    a = np.asarray(Image.open(sheet).convert("RGB")).astype(int)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    bg = (r > 150) & (b > 150) & (g < 110) & (np.abs(r - b) < 90)
    rgba = np.dstack([a, np.where(bg, 0, 255)]).astype(np.uint8)
    h, w = rgba.shape[:2]
    cw, ch = w // cols, h // rows
    # A halberd may reach past its cell edge, so frames are cut by connected pieces, not by cell lines: every piece
    # (8-connected after a small dilation, so a thin shaft stays joined) belongs to the cell holding its centroid,
    # and each frame is read from a padded window around its cell holding only its own pieces.
    fg = (rgba[..., 3] > 0).astype(np.uint8)
    n, labels, _, cents = cv2.connectedComponentsWithStats(cv2.dilate(fg, np.ones((3, 3), np.uint8)), connectivity=8)
    labels = np.where(fg > 0, labels, 0)
    owner = np.full(n, -1)
    for i in range(1, n):
        cx, cy = cents[i]
        owner[i] = min(int(cy) // ch, rows - 1) * cols + min(int(cx) // cw, cols - 1)
    px, py = cw // 2, ch // 4
    cells = []
    for idx in range(rows * cols):
        y, x = divmod(idx, cols)
        y0, y1 = max(0, y * ch - py), min(h, (y + 1) * ch + py)
        x0, x1 = max(0, x * cw - px), min(w, (x + 1) * cw + px)
        win = rgba[y0:y1, x0:x1].copy()
        mine = owner[labels[y0:y1, x0:x1]] == idx
        win[~mine] = 0
        pad = np.zeros(((ch + 2 * py), (cw + 2 * px), 4), np.uint8)  # same size for every frame, cell at the same spot
        oy, ox = y0 - (y * ch - py), x0 - (x * cw - px)
        pad[oy:oy + win.shape[0], ox:ox + win.shape[1]] = win
        cells.append(pad)
    cw, ch = cw + 2 * px, ch + 2 * py

    def extent(img):
        m = img[..., 3] > 0
        ys = np.nonzero(m.any(axis=1))[0]
        low = int(ys.max())
        xs = np.nonzero(m[max(0, low - 12):low + 1].any(axis=0))[0]
        return int(ys.min()), low, int(xs.min()), int(xs.max())

    ref = load_keys()[size_like]
    rt, rl, rx0, rx1 = extent(ref)
    t0, l0, x0, x1 = extent(cells[0])
    s = (rl - rt) / (l0 - t0)
    # x_shift moves every frame left (negative) or right: a canvas for a reference that must not clip a long blade
    dx = int(round((rx0 + rx1) / 2 - (x0 + x1) / 2 * s)) + x_shift
    W, H = round(cw * s), round(ch * s)
    out = {}
    for i, c in enumerate(cells, 1):
        im = clean(np.array(Image.fromarray(c).resize((W, H), Image.LANCZOS)))
        m = im[..., 3] > 0
        dy = GROUND_Y - int(np.nonzero(m.any(axis=1))[0].max())
        frame = np.zeros((SIZE, SIZE, 4), np.uint8)
        for yy in range(H):
            ty = yy + dy
            if 0 <= ty < SIZE:
                xa, xb = max(0, -dx), min(W, SIZE - dx)
                frame[ty, xa + dx:xb + dx] = im[yy, xa:xb]
        lost = int(m.sum()) - int((frame[..., 3] > 0).sum())
        if lost:
            print(f"  {prefix}_f{i}: {lost} px fall outside the 340 px frame")
        out[f"{prefix}_f{i}"] = frame
    print(f"{sheet.name}: {len(out)} frames, scale {s:.4f}, x shift {dx}")
    return out


def add_to_bundle(frames: dict) -> None:
    """Write frames into kaelen_4pose_uris.json as PNG data URIs, keeping every existing entry as it is."""
    p = HERE / "kaelen_4pose_uris.json"
    raw = p.read_text(encoding="utf-8")
    d = json.loads(raw)
    for k, f in frames.items():
        buf = io.BytesIO()
        Image.fromarray(f).save(buf, format="PNG", optimize=True)
        d[k] = "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()
    p.write_text(json.dumps(d) + ("\n" if raw.endswith("\n") else ""), encoding="utf-8", newline="\n")


REFS = {  # reference image name -> aligned key, for generated breakdown keys (art_src/vn_stage/prompts/)
    "kaelen_ref_passive_relaxed": "passive_relaxed",
    "kaelen_ref_active_drawn": "active_drawn",
    "kaelen_ref_drawn_windup": "drawn_kf_windup",
    "kaelen_ref_drawn_cleave": "drawn_kf_cleave",
    "kaelen_ref_halfdraw_r2": "drawn_kf_halfdraw_r2",
    "kaelen_ref_sheet_f1": "sheet_draw_f1",
    "kaelen_ref_sheet_f2": "sheet_draw_f2",
    "kaelen_ref_sheet_f6": "sheet_draw_f6",
    "kaelen_ref_walk_1h": "walk_1h_kf0",
    "kaelen_ref_sneak": "active_sneak",
    "kaelen_ref_drawn_poke": "drawn_kf_poke",
}


def export_refs(out_dir: Path) -> None:
    """The aligned 340 px keys on flat #FF00FF with one shared 96-colour palette, as image-model references.
    Rebuilt from kaelen_4pose_uris.json, so no binary needs to travel through git or Drive by hand."""
    keys = load_keys()
    # The palette comes from the first four references only, so adding a reference never changes the ones already
    # on Drive (the halfdraw_r2 reference is snapped to the same palette).
    basis = ["passive_relaxed", "active_drawn", "drawn_kf_windup", "drawn_kf_cleave"]
    pal = build_palette([keys[k] for k in basis], colours=96)
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, key in REFS.items():
        bg = Image.new("RGBA", (SIZE, SIZE), (255, 0, 255, 255))
        bg.alpha_composite(Image.fromarray(snap(keys[key], pal)))
        bg.convert("RGB").quantize(colors=128, dither=Image.Dither.NONE).save(out_dir / f"{name}.png", optimize=True)
    print(f"{len(REFS)} references -> {out_dir}")


PIPELINE = Path("G:/My Drive/image_pipeline")  # tools/portraits/artqueue.json's Drive queue


def stage_refs(pipeline: Path = PIPELINE) -> None:
    """Refresh <pipeline>/refs/_active/ with copies of exactly the references the requests waiting in queue/ need,
    so the web-app run can attach the whole folder. The folder only ever holds these copies; it is emptied first."""
    import shutil
    active = pipeline / "refs" / "_active"
    active.mkdir(parents=True, exist_ok=True)
    for old in active.iterdir():
        if old.is_file():
            old.unlink()
    need = {}
    for q in sorted((pipeline / "queue").glob("*.json")):
        req = json.loads(q.read_text(encoding="utf-8"))
        for ref in req.get("references", []):
            need.setdefault(Path(ref).name, Path(ref))
        print(f"{req['metadata']['asset_id']}: {', '.join(Path(r).name for r in req.get('references', []))}")
    for name, src in sorted(need.items()):
        shutil.copy2(src, active / name)
    print(f"{len(need)} references -> {active}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Build the keyframe in-betweening comparison lab.")
    ap.add_argument("--out", type=Path, default=HERE.parent.parent.parent / "kaelen_tween_lab.html")
    ap.add_argument("--gif-dir", type=Path, default=None, help="Also write one GIF per clip and method here.")
    ap.add_argument("--export-refs", type=Path, default=None, metavar="DIR",
                    help="Only write the breakdown-key reference PNGs into DIR (no lab).")
    ap.add_argument("--slice", type=Path, default=None, metavar="SHEET",
                    help="Cut a generated sprite sheet into the key bundle (with --grid, --prefix, --size-like).")
    ap.add_argument("--grid", default="3x2", help="Columns x rows of the sheet (default 3x2).")
    ap.add_argument("--prefix", default=None, help="Bundle key prefix, frames become <prefix>_f1.._fN.")
    ap.add_argument("--size-like", default="sheet_draw_f6", help="Bundle key whose height frame 1 is scaled to.")
    ap.add_argument("--stage-refs", action="store_true",
                    help="Copy the references the queued requests need into the queue's refs/_active/ folder.")
    args = ap.parse_args()
    if args.stage_refs:
        stage_refs()
        return
    if args.slice:
        cols, rows = (int(x) for x in args.grid.lower().split("x"))
        add_to_bundle(slice_sheet(args.slice, cols, rows, args.prefix, args.size_like))
        return
    if args.export_refs:
        export_refs(args.export_refs)
        return
    build(args.out, args.gif_dir)


if __name__ == "__main__":
    main()
