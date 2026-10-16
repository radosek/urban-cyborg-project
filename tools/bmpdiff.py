"""Compares two 8-bit BMP frame dumps and writes a side-by-side PNG (2x).

usage: python3 tools/bmpdiff.py a.bmp b.bmp out.png
"""
import struct
import sys
import zlib


def read(path):
    d = open(path, 'rb').read()
    off = int.from_bytes(d[10:14], 'little')
    w, h = struct.unpack('<ii', d[18:26])
    pal = d[54:54 + 1024]
    row = (w + 3) & ~3
    px = [d[off + (h - 1 - y) * row: off + (h - 1 - y) * row + w] for y in range(h)]
    return w, h, pal, px


def main():
    wa, ha, pa, a = read(sys.argv[1])
    wb, hb, pb, b = read(sys.argv[2])
    w, h = min(wa, wb), min(ha, hb)
    pts = [(x, y) for y in range(h) for x in range(w) if a[y][x] != b[y][x]]
    print('diff pixels', len(pts), 'pal same', pa == pb, 'sizes', (wa, ha), (wb, hb))
    if pts:
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        print('bbox', min(xs), min(ys), max(xs), max(ys))

    def row(img, pal, y, width):
        out = b''
        for x in range(width):
            c = img[y][x]
            out += bytes([pal[c * 4 + 2], pal[c * 4 + 1], pal[c * 4]]) * 2
        return out

    raw = b''
    for y in range(max(ha, hb)):
        la = row(a, pa, y, wa) if y < ha else b'\0' * wa * 6
        lb = row(b, pb, y, wb) if y < hb else b'\0' * wb * 6
        line = b'\0' + la + b'\xff\x00\xff' * 4 + lb
        raw += line + line
    W, H = wa * 2 + 4 + wb * 2, max(ha, hb) * 2

    def chunk(t, d):
        return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t + d))

    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', W, H, 8, 2, 0, 0, 0))
    png += chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b'')
    open(sys.argv[3], 'wb').write(png)


if __name__ == '__main__':
    main()
