#!/usr/bin/env python3
"""Draw the Fly on the Wall app icon: a housefly seen from directly above.

Written by hand rather than cropped from a screenshot because an icon is read
at sixteen pixels as often as at a thousand, and what survives that is a
silhouette. So: a dark plate, pale wings held out to either side, and a body
light enough to be a shape rather than a picture.

**Straight alpha, transparent corners.** The first version wrote RGB, which
gave the rounded plate square black corners — wrong for a macOS icon, where the
bundle is expected to supply its own shape, and wrong again for the launcher
tile, which sits the same art on its own background.

The colour outside the plate is the plate's own colour with the alpha taken to
zero rather than black-with-alpha-zero. That is what keeps the supersampled
edge from averaging toward a dark fringe: only the alpha steps across the
boundary, never the colour.

No PIL on this machine, so the PNG is assembled from zlib and a CRC, which is
all a PNG actually is.
"""

import math
import struct
import zlib

SIZE = 1024
SS = 3  # supersample factor per axis


def png(path, w, h, rows):
    raw = b"".join(b"\x00" + bytes(v for px in row for v in px) for row in rows)

    def chunk(tag, data):
        body = tag + data
        return (
            struct.pack(">I", len(data))
            + body
            + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)
        )

    # Colour type 6: 8-bit RGBA.
    head = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    with open(path, "wb") as f:
        f.write(
            b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", head)
            + chunk(b"IDAT", zlib.compress(raw, 9))
            + chunk(b"IEND", b"")
        )


def ellipse(px, py, cx, cy, rx, ry, angle):
    """Is (px,py) inside an ellipse centred (cx,cy), rotated by `angle`?"""
    dx, dy = px - cx, py - cy
    ca, sa = math.cos(-angle), math.sin(-angle)
    ux, uy = dx * ca - dy * sa, dx * sa + dy * ca
    return (ux / rx) ** 2 + (uy / ry) ** 2 <= 1.0


def rounded(px, py, x0, y0, x1, y1, r):
    if px < x0 or px > x1 or py < y0 or py > y1:
        return False
    cx = min(max(px, x0 + r), x1 - r)
    cy = min(max(py, y0 + r), y1 - r)
    return (px - cx) ** 2 + (py - cy) ** 2 <= r * r


def mix(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


GROUND_TOP = (26, 29, 37)
GROUND_BOT = (13, 15, 20)
WING = (184, 198, 216)
ABDOMEN = (58, 64, 78)
THORAX = (78, 86, 103)
HEAD = (68, 75, 91)
EYE = (170, 54, 44)


def shade(x, y):
    """Straight-alpha RGBA at a point in 0..SIZE space."""
    base = mix(GROUND_TOP, GROUND_BOT, y / SIZE)

    # The plate. macOS wants the rounded square baked in — and the corners
    # outside it genuinely absent, not painted black.
    inset = SIZE * 0.055
    if not rounded(x, y, inset, inset, SIZE - inset, SIZE - inset, SIZE * 0.225):
        return (*base, 0)

    cx = SIZE * 0.5
    cy = SIZE * 0.5
    u = SIZE / 1024.0

    # Order is front-to-back, first match wins. The eyes come first so they
    # read as the leading edge of the animal rather than as two crescents
    # peering out from behind its head, which is what drawing them last got.
    for side in (-1, 1):
        if ellipse(x, y, cx + side * 72 * u, cy - 212 * u, 58 * u, 70 * u, side * 0.28):
            return (*EYE, 255)
    if ellipse(x, y, cx, cy - 198 * u, 96 * u, 78 * u, 0.0):
        return (*HEAD, 255)
    if ellipse(x, y, cx, cy - 30 * u, 122 * u, 140 * u, 0.0):
        return (*THORAX, 255)
    if ellipse(x, y, cx, cy + 168 * u, 100 * u, 178 * u, 0.0):
        return (*ABDOMEN, 255)

    # Wings last, so the body sits over their roots: a wing hangs off a
    # shoulder and disappears under it, which is the same thing the game's own
    # model had to be taught.
    for side in (-1, 1):
        if ellipse(x, y, cx + side * 208 * u, cy + 88 * u, 196 * u, 74 * u, side * 0.55):
            return (*mix(base, WING, 0.9), 255)

    return (*base, 255)


def main():
    rows = []
    step = 1.0 / SS
    n = SS * SS
    for py in range(SIZE):
        row = []
        for px in range(SIZE):
            r = g = b = a = 0
            for sy in range(SS):
                for sx in range(SS):
                    c = shade(px + (sx + 0.5) * step, py + (sy + 0.5) * step)
                    r += c[0]
                    g += c[1]
                    b += c[2]
                    a += c[3]
            row.append((r // n, g // n, b // n, a // n))
        rows.append(row)
    png("icon.png", SIZE, SIZE, rows)
    print("wrote icon.png")


main()
