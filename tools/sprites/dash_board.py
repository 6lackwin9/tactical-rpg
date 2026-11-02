#!/usr/bin/env python3
# DASH1 review: one sheet with the dash's frame strip per grip (side, front, back) at 3x, its 40 px token under each.
#   python tools/sprites/dash_board.py OUT [body ...]
# Writes OUT/dash1_grips.png from the rig (not from the shipped sheets); nothing in the repository is written. Default
# bodies: one per grip.
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from PIL import Image, ImageDraw  # noqa: E402

import export_units  # noqa: E402
import rig  # noqa: E402
import views  # noqa: E402

TOKEN_PX = 40      # tuning.json token_px
ZOOM = 3
BG, INK, GROUND = (46, 52, 44), (232, 226, 210), (90, 100, 86)
GRIPS = ('warrior', 'warrior/1h_buckler', 'warrior/2h_sword', 'warrior/2h_axe', 'ranger', 'brawler', 'priest', 'rogue', 'mage')
ROWS = ('dash', 'dash_down', 'dash_up')


def board(out, bodies):
    size = rig.W * ZOOM
    frames = len(rig.DASH_MS)
    label = 130
    strip = frames * (size + 2)
    cell_h = size + TOKEN_PX + 4
    im = Image.new('RGB', (label + len(ROWS) * (strip + 10), 24 + len(bodies) * (cell_h + 8)), BG)
    d = ImageDraw.Draw(im)
    d.text((10, 6), 'DASH1 per grip: side, front, back (%s ms); the 40 px token under each frame' % '-'.join(map(str, rig.DASH_MS)), fill=INK)
    for r, key in enumerate(bodies):
        rows = export_units.body(key, 'all')[2]
        y = 24 + r * (cell_h + 8)
        d.text((10, y + 10), key, fill=INK)
        for c, name in enumerate(ROWS):
            for i, f in enumerate(rows[name]):
                x = label + c * (strip + 10) + i * (size + 2)
                d.line([(x, y + (rig.GY + 1) * ZOOM), (x + size, y + (rig.GY + 1) * ZOOM)], fill=GROUND)
                big = f.resize((size, size), Image.NEAREST)
                im.paste(big, (x, y), big)
                tok = views.downsample(f, TOKEN_PX, TOKEN_PX)
                im.paste(tok, (x + (size - TOKEN_PX) // 2, y + size), tok)
    path = os.path.join(out, 'dash1_grips.png')
    im.save(path)
    return path


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    print(board(out, sys.argv[2:] or list(GRIPS)))


if __name__ == '__main__':
    main()
