"""LOOK1-T: deterministic ragged corner masks 1-14 for pieces of 32 x 32, 32 x 24 and 64 x 48.

Bit 1 north-west, 2 north-east, 4 south-west, 8 south-east is upper (LOOK1 5.4). Each quarter is bounded by the
piece's two middle lines; the lines are moved by 1D value noise (seed the mask number, amplitude 2 art px, period 6 art
px) that fades to 0 within 3 art px of the piece border, so neighbouring pieces meet.
"""
AMPLITUDE = 2
PERIOD = 6
FADE = 3
SIZES = [(32, 32), (32, 24), (64, 48)]


def _lattice(seed, k):
    """Deterministic value in [-1, 1] for lattice point k."""
    n = (seed * 0x9E3779B1 + k * 0x85EBCA6B + 0x27D4EB2F) & 0xFFFFFFFF
    n ^= n >> 15
    n = (n * 0x2C1B3C6D) & 0xFFFFFFFF
    n ^= n >> 12
    n = (n * 0x297A2D39) & 0xFFFFFFFF
    n ^= n >> 15
    return (n & 0xFFFF) / 32767.5 - 1.0


def displacement(seed, i, length):
    """Integer shift of a middle line at position `i` along it (0 at both ends)."""
    t = i / PERIOD
    k = int(t)
    f = t - k
    f = f * f * (3 - 2 * f)
    v = _lattice(seed, k) * (1 - f) + _lattice(seed, k + 1) * f
    fade = min(min(i, length - 1 - i), FADE) / FADE
    return round(v * AMPLITUDE * fade)


def mask_grid(mask, w, h):
    """Rows of booleans, True where the upper tile shows."""
    if not 0 <= mask <= 15:
        raise ValueError(f"mask {mask} outside 0-15")
    dx = [displacement(mask * 2, y, h) for y in range(h)]  # vertical line, by row
    dy = [displacement(mask * 2 + 1, x, w) for x in range(w)]  # horizontal line, by column
    rows = []
    for y in range(h):
        row = []
        for x in range(w):
            east = x >= w // 2 + dx[y]
            south = y >= h // 2 + dy[x]
            row.append(bool(mask & (1 << (south * 2 + east))))
        rows.append(row)
    return rows


def plain_quarters(mask, w, h):
    """The mask without noise (the boundary exactly on the middle lines)."""
    return [[bool(mask & (1 << ((y >= h // 2) * 2 + (x >= w // 2)))) for x in range(w)] for y in range(h)]


if __name__ == "__main__":
    import sys

    m = int(sys.argv[1]) if len(sys.argv) > 1 else 1
    for r in mask_grid(m, 32, 24):
        print("".join("#" if v else "." for v in r))
