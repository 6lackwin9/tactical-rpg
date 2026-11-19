"""Writes games/hello/assets/sprites/walker.png: the sample game's 16 x 16 walker (a round lamp-bot), drawn from the
rows below. Run from the repository: python3 games/hello/tools/walker_png.py"""
from pathlib import Path
from PIL import Image

ROWS = [
    "......kkkk......",
    ".....kyyyyk.....",
    "......kkkk......",
    "....kkbbbbkk....",
    "...kbbbbbbbbk...",
    "..kbbwwbbwwbbk..",
    "..kbbwkbbwkbbk..",
    "..kbbbbbbbbbbk..",
    "..kbbbbppbbbbk..",
    "..kbbbbbbbbbbk..",
    "...kbbbbbbbbk...",
    "....kkbbbbkk....",
    ".....kddddk.....",
    "....kdk..kdk....",
    "....kk....kk....",
    "................",
]
COLOURS = {"k": (24, 28, 40, 255), "y": (255, 214, 90, 255), "b": (88, 168, 232, 255), "w": (245, 245, 250, 255),
           "p": (232, 110, 130, 255), "d": (60, 92, 140, 255), ".": (0, 0, 0, 0)}

img = Image.new("RGBA", (16, 16))
for y, row in enumerate(ROWS):
    for x, c in enumerate(row):
        img.putpixel((x, y), COLOURS[c])
out = Path(__file__).resolve().parents[1] / "assets" / "sprites" / "walker.png"
img.save(out)
print(out)
