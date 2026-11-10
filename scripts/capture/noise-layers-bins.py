#!/usr/bin/env python3
"""Per-position deviation of a signed grain image (noise layers design §7.2, 5).

Usage: noise-layers-bins.py <raw 16-bit MSB gray> <width> <height> <scale>

Prints aggregate_sd (0..1 units), min_bin_ratio and max_bin_ratio (each
position class's deviation over the aggregate) and lf_ratio (the deviation of
4x4 block means over the pixels'). Pixels are classed by (x mod scale,
y mod scale): one class per position inside the lattice cell, whatever the
crop's phase or the capture's orientation. Exit 2 on malformed input.
"""
import math
import struct
import sys


def sd(values):
    mean = sum(values) / len(values)
    return math.sqrt(sum((v - mean) ** 2 for v in values) / len(values))


def main():
    if len(sys.argv) != 5:
        print(__doc__, file=sys.stderr)
        return 2
    path, w, h, s = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
    with open(path, 'rb') as f:
        data = f.read()
    if len(data) != 2 * w * h or s < 1 or w < 4 or h < 4:
        print(f'{path}: {len(data)} bytes for {w}x{h}, scale {s}', file=sys.stderr)
        return 2
    values = struct.unpack(f'>{w * h}H', data)
    aggregate = sd(values)
    if aggregate == 0:
        print(f'{path}: no grain', file=sys.stderr)
        return 2
    classes = [[] for _ in range(s * s)]
    for y in range(h):
        for x in range(w):
            classes[(y % s) * s + x % s].append(values[y * w + x])
    ratios = [sd(c) / aggregate for c in classes]
    means = [
        sum(values[(by * 4 + dy) * w + bx * 4 + dx] for dy in range(4) for dx in range(4)) / 16
        for by in range(h // 4)
        for bx in range(w // 4)
    ]
    print(f'aggregate_sd={aggregate / 65535:.6f}')
    print(f'min_bin_ratio={min(ratios):.4f}')
    print(f'max_bin_ratio={max(ratios):.4f}')
    print(f'lf_ratio={sd(means) / aggregate:.4f}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
