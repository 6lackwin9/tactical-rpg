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

    def __init__(self, canon: np.ndarray, lm: dict):
        self.img = canon
        self.ax = Axis(lm)
        h, w = canon.shape[:2]
        ys, xs = np.mgrid[0:h, 0:w].astype(float)
        u, v = self.ax.local(xs, ys)
        hands = _hand_mask(canon, [lm["grip_front"], lm["grip_back"]])
        self.blade_mask = (u >= BLADE_BACK) & (np.abs(v) <= BLADE_HALF) & (u <= 90) & (canon[..., 3] > 0) & ~hands
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

    def draw(self, canvas: np.ndarray, grip_front, angle: float) -> np.ndarray:
        """Draw the halberd so its canonical front-grip point sits on grip_front at the given angle."""
        d = np.array([math.cos(angle), math.sin(angle)])
        n = np.array([-d[1], d[0]])
        socket = np.array(grip_front, float) - self.u_front * d
        h, w = canvas.shape[:2]
        ys, xs = np.mgrid[0:h, 0:w].astype(float)
        px, py = xs - socket[0], ys - socket[1]
        u, v = px * d[0] + py * d[1], px * n[0] + py * n[1]
        out = canvas.copy()
        # shaft
        shaft = (u >= -self.ax.length) & (u < BLADE_BACK + 2) & (np.abs(v) <= SHAFT_HALF + 0.5)
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
        self._flows = {}

    def frame(self, ka: str, kb: str, t: float) -> np.ndarray:
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
        withw = self.halberd.draw(body, grip, ang)
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
            if method == "hybrid":
                img = hyb.frame(ka, kb, t)
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


GUARD, WIND, POKE, CLEAVE = "active_drawn", "drawn_kf_windup", "drawn_kf_poke", "drawn_kf_cleave"
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
    pal = build_palette([keys[k] for k in keys if not k.startswith("drawn_") or k.startswith("drawn_kf")])
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


REFS = {  # reference image name -> aligned key, for generated breakdown keys (art_src/vn_stage/prompts/)
    "kaelen_ref_passive_relaxed": "passive_relaxed",
    "kaelen_ref_active_drawn": "active_drawn",
    "kaelen_ref_drawn_windup": "drawn_kf_windup",
    "kaelen_ref_drawn_cleave": "drawn_kf_cleave",
}


def export_refs(out_dir: Path) -> None:
    """The aligned 340 px keys on flat #FF00FF with one shared 96-colour palette, as image-model references.
    Rebuilt from kaelen_4pose_uris.json, so no binary needs to travel through git or Drive by hand."""
    keys = load_keys()
    pal = build_palette([keys[k] for k in REFS.values()], colours=96)
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, key in REFS.items():
        bg = Image.new("RGBA", (SIZE, SIZE), (255, 0, 255, 255))
        bg.alpha_composite(Image.fromarray(snap(keys[key], pal)))
        bg.convert("RGB").quantize(colors=128, dither=Image.Dither.NONE).save(out_dir / f"{name}.png", optimize=True)
    print(f"{len(REFS)} references -> {out_dir}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Build the keyframe in-betweening comparison lab.")
    ap.add_argument("--out", type=Path, default=HERE.parent.parent.parent / "kaelen_tween_lab.html")
    ap.add_argument("--gif-dir", type=Path, default=None, help="Also write one GIF per clip and method here.")
    ap.add_argument("--export-refs", type=Path, default=None, metavar="DIR",
                    help="Only write the breakdown-key reference PNGs into DIR (no lab).")
    args = ap.parse_args()
    if args.export_refs:
        export_refs(args.export_refs)
        return
    build(args.out, args.gif_dir)


if __name__ == "__main__":
    main()
