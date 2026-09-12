#!/usr/bin/env python3
"""Measure grain or additive-light invariance in aligned 8-bit captures.

Exit 0: measured gate passes (or grain reported); 1: additive gate fails;
2: invalid or uninformative input. Capture refusals are not metric failures.
"""
import argparse
import json
import math
from pathlib import Path
import statistics
import subprocess
import sys


def decode(x):
    return x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4


def grain_sd(on, off):
    if not on or len(on) != len(off):
        raise ValueError('grain needs nonempty, equally sized images')
    residuals = [sum(w * (a - b) for w, a, b in zip((0.2126, 0.7152, 0.0722), p, q)) / 255
                 for p, q in zip(on, off)]
    return statistics.pstdev(residuals)


def additive_interval(codes):
    bounds = [(decode(max(0, c - 0.5) / 255),
               decode(min(255, c + 0.5) / 255)) for c in codes]
    lo = bounds[0][0] - bounds[1][1] - bounds[2][1] + bounds[3][0]
    hi = bounds[0][1] - bounds[1][0] - bounds[2][0] + bounds[3][1]
    return lo, hi


def additive_report(frames):
    if len(frames) != 4 or not frames[0] or any(len(f) != len(frames[0]) for f in frames):
        raise ValueError('additive needs four nonempty, equally sized images')
    valid = informative = clipped = failures = 0
    maximum = 0.0
    for pixels in zip(*frames):
        for codes in zip(*pixels):
            if any(c == 0 or c == 255 for c in codes):
                clipped += 1
                continue
            valid += 1
            informative += abs(codes[2] - codes[3]) > 2
            lo, hi = additive_interval(codes)
            failures += lo > 1e-7 or hi < -1e-7
            a, b, c, d = (decode(code / 255) for code in codes)
            maximum = max(maximum, abs(a - b - c + d))
    if valid < 1000 or informative < 100:
        raise ValueError(f'uninformative additive fixture: {valid} valid, '
                         f'{informative} informative, {clipped} clipped channels')
    return dict(valid_channels=valid, informative_channels=informative,
                clipped_channels=clipped, failures=failures, max_abs_residual=maximum)


def check_rect(rect, width, height):
    x, y, w, h = rect
    if x < 0 or y < 0 or w <= 0 or h <= 0 or x + w > width or y + h > height:
        raise ValueError(f'crop {rect} is outside {width}x{height} or empty')
    return f'{w}x{h}+{x}+{y}'


def read_rgb(path, rect):
    path = str(Path(path).resolve(strict=True))
    size = subprocess.run(['magick', 'identify', '-format', '%w %h', path],
                          check=True, capture_output=True, text=True).stdout.split()
    if len(size) != 2:
        raise ValueError('expected one image')
    width, height = map(int, size)
    crop = check_rect(rect, width, height)
    raw = subprocess.run(['magick', path, '-crop', crop, '+repage', '-alpha', 'off',
                          '-depth', '8', 'rgb:-'], check=True, capture_output=True).stdout
    if len(raw) != rect[2] * rect[3] * 3:
        raise ValueError('decoded RGB byte count does not match crop')
    return (width, height), list(zip(raw[::3], raw[1::3], raw[2::3]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    grain = commands.add_parser('grain')
    grain.add_argument('images', nargs=2, metavar=('ON', 'OFF'))
    grain.add_argument('--rect', nargs=4, type=int, required=True, metavar=('X', 'Y', 'W', 'H'))
    additive = commands.add_parser('additive')
    additive.add_argument('images', nargs=4, metavar='IMAGE')
    additive.add_argument('--rect', nargs=4, type=int, required=True, metavar=('X', 'Y', 'W', 'H'))
    args = parser.parse_args()
    try:
        decoded = [read_rgb(path, args.rect) for path in args.images]
        if len({size for size, _ in decoded}) != 1:
            raise ValueError('capture dimensions differ')
        frames = [pixels for _, pixels in decoded]
        report = (dict(sd=grain_sd(*frames), pixels=len(frames[0])) if args.command == 'grain'
                  else additive_report(frames))
        if not all(math.isfinite(value) for value in report.values()):
            raise ValueError('non-finite metric')
        print(json.dumps(report, allow_nan=False, sort_keys=True))
        return int(report.get('failures', 0) > 0)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'metrics: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
