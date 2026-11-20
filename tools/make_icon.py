#!/usr/bin/env python3
"""Draws SunnyMods' icon.png (128x128): a 32x32 pixel picture, each pixel 4x4.
Three bevelled mod tiles and a sun in the fourth corner, on a dark rounded panel."""
import struct, sys, zlib, math

G = 32
px = [[(0, 0, 0, 0)] * G for _ in range(G)]

def put(x, y, c):
    if 0 <= x < G and 0 <= y < G:
        px[y][x] = c if len(c) == 4 else (*c, 255)

def rect(x0, y0, x1, y1, c):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, c)

def rounded(x0, y0, x1, y1, c, r=1, bg=(0, 0, 0, 0)):
    rect(x0, y0, x1, y1, c)
    for (cx, cy) in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)):
        for i in range(r):
            for j in range(r - i):
                put(cx + (i if cx == x0 else -i), cy + (j if cy == y0 else -j), bg)

def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3])

# The panel: dark navy, a lighter rim.
rounded(0, 0, 31, 31, (30, 34, 52), 3)
rounded(1, 1, 30, 30, (24, 27, 42), 2, (30, 34, 52, 255))

def tile(x, y, s, c):
    rounded(x, y, x + s - 1, y + s - 1, shade(c, 0.55), 1, (24, 27, 42, 255))          # outline / shadow
    rect(x + 1, y + 1, x + s - 2, y + s - 3, c)                       # face
    rect(x + 1, y + 1, x + s - 2, y + 1, shade(c, 1.3))               # lit top edge
    rect(x + 1, y + 2, x + 1, y + s - 3, shade(c, 1.15))              # lit left edge
    rect(x + 1, y + s - 2, x + s - 2, y + s - 2, shade(c, 0.75))      # lower lip

S = 12
tile(4, 4, S, (58, 179, 218))     # sky
tile(4, 17, S, (128, 199, 31))    # lime
tile(17, 17, S, (176, 92, 214))   # violet

# Little marks on the tiles: a list, a gear-ish dot, a plus.
W = (245, 247, 250)
for i, y in enumerate((7, 10, 13)):
    rect(7, y, 7, y, W); rect(9, y, 12 - (i % 2), y, W)
rect(9, 20, 11, 25, W); rect(7, 22, 13, 23, W)                   # plus on lime
rect(21, 21, 24, 24, W); rect(22, 20, 23, 25, W); rect(20, 22, 25, 23, W)
rect(22, 22, 23, 23, (176, 92, 214))                              # gear's hole

# The sun in the top right tile's place.
cx, cy = 22.5, 9.5
for y in range(G):
    for x in range(G):
        dx, dy = x + 0.5 - cx, y + 0.5 - cy
        r = math.hypot(dx, dy)
        a = math.degrees(math.atan2(dy, dx)) % 45
        if r <= 3.6:
            put(x, y, (254, 216, 61) if r <= 2.6 else (249, 128, 29))
            if r <= 1.5 and dx < 0 and dy < 0:
                put(x, y, (255, 244, 180))
        elif 4.6 <= r <= 6.6 and min(a, 45 - a) < 7:
            put(x, y, (249, 160, 40))

out = []
N = 128
for y in range(N):
    row = bytearray([0])
    for x in range(N):
        row += bytes(px[y // 4][x // 4])
    out.append(bytes(row))
raw = b"".join(out)
def chunk(t, d):
    return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", N, N, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
open(sys.argv[1] if len(sys.argv) > 1 else "icon.png", "wb").write(png)
