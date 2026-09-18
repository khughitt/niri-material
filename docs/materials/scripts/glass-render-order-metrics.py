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


def profile_report(on, off):
    """Return the peak and contiguous half-maximum width of a one-pixel profile."""
    if not on or len(on) != len(off):
        raise ValueError('profile needs nonempty, equally sized samples')
    deltas = [max(abs(a - b) for a, b in zip(p, q)) for p, q in zip(on, off)]
    peak = max(deltas)
    if peak <= 1:
        raise ValueError('uninformative profile')
    at_peak = deltas.index(peak)
    threshold = peak / 2
    left = at_peak
    while left and deltas[left - 1] >= threshold:
        left -= 1
    right = at_peak
    while right + 1 < len(deltas) and deltas[right + 1] >= threshold:
        right += 1
    return dict(profile_peak_delta=peak, profile_fwhm=right - left + 1)


def attenuation_interval(on, off):
    """The decoded-light interval for an on-minus-off 8-bit channel pair."""
    return (decode(max(0, on - .5) / 255) - decode(min(255, off + .5) / 255),
            decode(min(255, on + .5) / 255) - decode(max(0, off - .5) / 255))


def attenuation_report(frames, expected):
    """Check dense/white additive-light ratios with quantization intervals."""
    if len(frames) != 4 or not frames[0] or any(len(f) != len(frames[0]) for f in frames):
        raise ValueError('attenuation needs four nonempty, equally sized images')
    if len(expected) != 3 or any(not math.isfinite(x) or x <= 0 for x in expected):
        raise ValueError('attenuation needs three positive finite expected ratios')
    valid = clipped = uninformative = failures = 0
    for pixels in zip(*frames):
        for channel, codes in enumerate(zip(*pixels)):
            if any(code == 0 or code == 255 for code in codes):
                clipped += 1
                continue
            dense_lo, dense_hi = attenuation_interval(codes[0], codes[1])
            white_lo, white_hi = attenuation_interval(codes[2], codes[3])
            if dense_lo <= 0 or white_lo <= 0:
                uninformative += 1
                continue
            valid += 1
            ratio_lo, ratio_hi = dense_lo / white_hi, dense_hi / white_lo
            failures += not (ratio_lo <= expected[channel] <= ratio_hi)
    if valid < 100:
        raise ValueError(f'uninformative attenuation fixture: {valid} valid, '
                         f'{uninformative} uninformative, {clipped} clipped channels')
    return dict(valid_channels=valid, uninformative_channels=uninformative,
                clipped_channels=clipped, failures=failures)


def ring_bound(thickness, inset, width, scatter=0.0):
    epsilon = 0.5 / (255 * 12.92)
    gain = 1.7
    core_width = width + 6 * scatter
    halo_width = 9 + 18 * scatter
    core = inset + core_width * math.sqrt(0.5 * math.log(2 * gain / epsilon))
    halo = inset + 2 + halo_width * math.sqrt(0.5 * math.log(0.6 * gain / epsilon))
    shift = 0.5 * inset + 2 * (0.2 * thickness)
    return math.ceil(max(core, halo) + shift)


def rounded_slab_distance(x, y, window, bevel, offset_x=0.0, offset_y=0.0,
                          corner_radius=0.0):
    wx, wy, ww, wh = window
    if ww <= 0 or wh <= 0 or bevel < 0 or corner_radius < 0:
        raise ValueError('invalid slab geometry')
    inflate = bevel - max(abs(offset_x), abs(offset_y))
    if inflate < 0:
        raise ValueError('offset must not exceed bevel')
    sx = wx - inflate + offset_x
    sy = wy - inflate + offset_y
    sw = ww + 2 * inflate
    sh = wh + 2 * inflate
    chamfer = min(bevel, max(min(sw, sh) / 2 - 1, 0))
    radius = min(corner_radius, ww / 2, wh / 2)
    inner_radius = min(radius, max(sw - 2 * chamfer, 0) / 2,
                       max(sh - 2 * chamfer, 0) / 2)
    outer_radius = inner_radius + chamfer
    px = x + 0.5 - (sx + sw / 2)
    py = y + 0.5 - (sy + sh / 2)
    qx = abs(px) - sw / 2 + outer_radius
    qy = abs(py) - sh / 2 + outer_radius
    return min(max(qx, qy), 0) + math.hypot(max(qx, 0), max(qy, 0)) - outer_radius


def reach_report(on, off, window, bevel, thickness, inset, width, scatter=0.0,
                 offset_x=0.0, offset_y=0.0, corner_radius=0.0, profile=None):
    if not on or len(on) != len(off) or any(len(row) != len(off_row)
                                            for row, off_row in zip(on, off)):
        raise ValueError('reach needs nonempty, equally sized images')
    geometry = (*window, bevel, thickness, inset, width, scatter, offset_x, offset_y,
                corner_radius)
    if (len(window) != 4 or any(not math.isfinite(value) for value in geometry) or
            bevel < 0 or thickness <= 0 or inset < 0 or width <= 0 or scatter < 0 or
            corner_radius < 0):
        raise ValueError('invalid reach geometry')
    bound = ring_bound(thickness, inset, width, scatter)
    reference = inset + 2 + 18 + 0.5 * inset
    max_delta = max_interior_delta = outside_bound = outside_reference = outside_slab = 0
    outside_bound_max_delta = outside_reference_max_delta = 0
    visible_reach = 0.0
    covered_slab = False
    for y, (on_row, off_row) in enumerate(zip(on, off)):
        for x, (on_pixel, off_pixel) in enumerate(zip(on_row, off_row)):
            distance = rounded_slab_distance(x, y, window, bevel, offset_x, offset_y,
                                              corner_radius)
            covered_slab |= distance <= 0
            delta = max(abs(a - b) for a, b in zip(on_pixel, off_pixel))
            max_delta = max(max_delta, delta)
            if distance <= 0:
                max_interior_delta = max(max_interior_delta, delta)
            if delta <= 1:
                continue
            if distance > 0:
                outside_slab += 1
                continue
            reach = -distance
            visible_reach = max(visible_reach, reach)
            if reach > bound:
                outside_bound += 1
                outside_bound_max_delta = max(outside_bound_max_delta, delta)
            if reach > reference:
                outside_reference += 1
                outside_reference_max_delta = max(outside_reference_max_delta, delta)
    if not covered_slab:
        raise ValueError('reach window has no covered slab pixel')
    report = dict(bound=bound, reference_contour=reference,
                max_channel_delta=max_delta, outside_bound_pixels=outside_bound,
                max_interior_channel_delta=max_interior_delta,
                outside_reference_pixels=outside_reference,
                outside_bound_max_channel_delta=outside_bound_max_delta,
                outside_reference_max_channel_delta=outside_reference_max_delta,
                outside_slab_pixels=outside_slab,
                visible_interior_reach=visible_reach)
    if profile is not None:
        report.update(profile_report(*profile))
    return report


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
    reach = commands.add_parser('reach')
    reach.add_argument('images', nargs=2, metavar=('ON', 'OFF'))
    reach.add_argument('--window', nargs=4, type=int, required=True, metavar=('X', 'Y', 'W', 'H'))
    reach.add_argument('--bevel', type=float, required=True)
    reach.add_argument('--thickness', type=float, required=True)
    reach.add_argument('--inset', type=float, required=True)
    reach.add_argument('--width', type=float, required=True)
    reach.add_argument('--scatter', type=float, default=0)
    reach.add_argument('--offset-x', type=float, default=0)
    reach.add_argument('--offset-y', type=float, default=0)
    reach.add_argument('--corner-radius', type=float, default=0)
    reach.add_argument('--profile', nargs=4, type=int, metavar=('X', 'Y', 'W', 'H'))
    reach.add_argument('--attenuation-images', nargs=4, metavar='IMAGE')
    reach.add_argument('--attenuation-rect', nargs=4, type=int, metavar=('X', 'Y', 'W', 'H'))
    reach.add_argument('--attenuation-ratio', nargs=3, type=float, metavar=('R', 'G', 'B'))
    reach.add_argument('--attenuation-only', action='store_true')
    args = parser.parse_args()
    try:
        if args.command == 'reach' and bool(args.attenuation_images) != bool(args.attenuation_rect):
            raise ValueError('attenuation images and rect must be supplied together')
        if args.command == 'reach' and args.attenuation_images and not args.attenuation_ratio:
            raise ValueError('attenuation ratio is required with attenuation images')
        if args.command == 'reach' and args.attenuation_only and not args.attenuation_images:
            raise ValueError('attenuation-only needs attenuation images')
        rect = args.rect if args.command != 'reach' else (0, 0, *subprocess.run(
            ['magick', 'identify', '-format', '%w %h', str(Path(args.images[0]).resolve(strict=True))],
            check=True, capture_output=True, text=True).stdout.split())
        rect = tuple(map(int, rect))
        decoded = [read_rgb(path, rect) for path in args.images]
        if len({size for size, _ in decoded}) != 1:
            raise ValueError('capture dimensions differ')
        frames = [pixels for _, pixels in decoded]
        if args.command == 'grain':
            report = dict(sd=grain_sd(*frames), pixels=len(frames[0]))
        elif args.command == 'additive':
            report = additive_report(frames)
        else:
            width, height = decoded[0][0]
            on = [frames[0][row * width:(row + 1) * width] for row in range(height)]
            off = [frames[1][row * width:(row + 1) * width] for row in range(height)]
            profile = None
            if args.profile:
                profile_size, profile_on = read_rgb(args.images[0], tuple(args.profile))
                _, profile_off = read_rgb(args.images[1], tuple(args.profile))
                if profile_size != decoded[0][0]:
                    raise ValueError('profile dimensions differ')
                profile = (profile_on, profile_off)
            report = ({} if args.attenuation_only else
                      reach_report(on, off, args.window, args.bevel, args.thickness,
                                   args.inset, args.width, args.scatter, args.offset_x,
                                   args.offset_y, args.corner_radius, profile))
            if args.attenuation_images:
                attenuation_decoded = [read_rgb(path, tuple(args.attenuation_rect))
                                       for path in args.attenuation_images]
                if any(size != decoded[0][0] for size, _ in attenuation_decoded):
                    raise ValueError('attenuation capture dimensions differ')
                attenuation = [pixels for _, pixels in attenuation_decoded]
                attenuation_report_data = attenuation_report(attenuation, args.attenuation_ratio)
                if args.attenuation_only:
                    report = attenuation_report_data
                else:
                    report.update(attenuation_report_data)
        if not all(math.isfinite(value) for value in report.values()):
            raise ValueError('non-finite metric')
        print(json.dumps(report, allow_nan=False, sort_keys=True))
        return int(report.get('failures', 0) > 0 or
                   (not getattr(args, 'attenuation_only', False) and
                    report.get('outside_bound_pixels', 0) > 0))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'metrics: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
