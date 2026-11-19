"""AI-23: top-level drain brick interior wall set (tilesets/drain_brick/{wall,wall_face}/v0.png).

Same shape as tilesets/interior_stone (wall 128x96, face 128x48): the interior_stone brick courses re-graded onto the
colour ramp of the Sunk Ward boundary drain brick (tilesets/sunk_ward/boundary/drain_brick), plus wet streaks and a
moss line along the foot of the face. Deterministic (fixed seed). Usage: python3 art_src/style/drain_brick_walls.py
"""
import random
from pathlib import Path
from PIL import Image

T = Path(__file__).resolve().parents[2] / "games/emberwake/assets/tilesets"


def ramp_of(img: Image.Image):
    px = [p[:3] for p in img.convert("RGBA").getdata() if p[3] > 0]
    px.sort(key=lambda c: sum(c))
    n = len(px)
    pick = lambda f: tuple(sum(c[i] for c in px[int(n * f): int(n * f) + max(1, n // 20)]) // max(1, n // 20) for i in range(3))
    return pick(0.05), pick(0.5), pick(0.9)


def regrade(src: Image.Image, ramp, wet: bool, seed: int) -> Image.Image:
    # keep interior_stone's contrast; shift its hue to the drain brick's (damp olive-grey), scaled per channel
    s = src.convert("RGBA")
    px = [p for p in s.get_flattened_data()] if hasattr(s, "get_flattened_data") else list(s.getdata())
    m = [sum(p[k] for p in px) / len(px) for k in range(3)]
    tgt = ramp[1]
    tl = sum(tgt) / 3 or 1
    ml = sum(m) / 3
    tint = [(tgt[k] / tl) * ml / max(1, m[k]) for k in range(3)]
    out = Image.new("RGBA", s.size)
    rnd = random.Random(seed)
    streaks = {rnd.randrange(s.width) for _ in range(4)} if wet else set()
    for i, (r, g, b, a) in enumerate(px):
        x, y = i % s.width, i // s.width
        c = [r * tint[0], g * tint[1], b * tint[2]]
        if any(abs(x - sx) <= 0 for sx in streaks) and y > 4:
            c = [v * 0.88 for v in c]          # damp run-off down the face
        if wet and y >= s.height - 5:
            c = [c[0] * 0.8, c[1] * 0.92, c[2] * 0.75]   # moss and silt line at the foot
        out.putpixel((x, y), tuple(int(max(0, min(255, v))) for v in c) + (a,))
    return out


def main():
    ramp = ramp_of(Image.open(T / "sunk_ward/boundary/drain_brick/wall/v0.png"))
    face_ramp = ramp_of(Image.open(T / "sunk_ward/boundary/drain_brick/wall_face/v0.png"))
    for part, rp, wet in (("wall", ramp, False), ("wall_face", face_ramp, True)):
        src = Image.open(T / "interior_stone" / part / "v0.png")
        (T / "drain_brick" / part).mkdir(parents=True, exist_ok=True)
        regrade(src, rp, wet, 7).save(T / "drain_brick" / part / "v0.png")
        print(part, rp)


if __name__ == "__main__":
    main()
