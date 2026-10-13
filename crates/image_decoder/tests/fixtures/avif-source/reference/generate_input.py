"""Deterministic full/limited 400/420/422/444 input for libyuv_oracle.cc."""
import pathlib
import struct
import sys

data = bytearray()
state = 0x5eeda71f
sizes = [(1, 1), (1, 5), (2, 2), (3, 3), (7, 5), (16, 4), (17, 9),
         (31, 18), (32, 17), (33, 3), (64, 2), (65, 7), (257, 1)]
edge = [0, 1, 15, 16, 17, 63, 64, 127, 128, 129, 234, 235, 236, 254, 255]
for fmt in (400, 420, 422, 444):
    for matrix in (1, 2, 5, 6):
        for full in (0, 1):
            for alpha in (0, 1):
                for w, h in sizes:
                    cw = w if fmt == 444 else (w + 1) // 2
                    ch = (h + 1) // 2 if fmt == 420 else h
                    n = w * h * (1 + alpha) + (0 if fmt == 400 else 2 * cw * ch)
                    data += struct.pack('<6I', w, h, fmt, matrix, full, alpha)
                    for i in range(n):
                        state ^= (state << 13) & 0xffffffff
                        state ^= state >> 17
                        state ^= (state << 5) & 0xffffffff
                        data.append(edge[i % len(edge)] if i % 3 == 0 else state & 255)
pathlib.Path(sys.argv[1]).write_bytes(data)
