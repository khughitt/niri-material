#!/usr/bin/env python3
"""Offline calibration of a local glass-warp estimator (material-bb8480).

No renderer runs. A seeded aperiodic backdrop and a replica of the sweep's
20 px grid are warped by known displacement fields and changed by known
photometric-only controls, quantized to 8-bit sRGB like a capture, and a
zero-mean normalized cross-correlation (ZNCC) block matcher estimates the
displacement back. Each case reports the estimator's error against the known
field, the share of pixels it flags as ambiguous, and the regional RMSE the
sweep used, so the two instruments can be compared on the same inputs.

The displacement convention is the shader's: out(p) = backdrop(p + u(p)),
the tap in prelude.frag. The chamfer case is the shape a 45-degree chamfer
gives on a straight left edge at distortion 0: u_x constant across a 12 px
strip, zero on the backdrop outside it and on the flat face inside it.

Commands:
  calibrate --out DIR   run every case; write results.json and results.md
  backdrop PATH         write the aperiodic backdrop as a 1280x720 PNG

Exit 0 on success, 2 on invalid input. Stdlib only.
"""
import argparse
import json
import math
import operator
import struct
import sys
import zlib
from array import array
from itertools import accumulate, product
from pathlib import Path

SEED = 20260929
GRID_PERIOD = 20
# The noise octaves: lattice spacing in px and weight. Non-integer spacings
# keep the lattice off the pixel grid, so no octave repeats on whole pixels.
OCTAVES = ((2.3, 1.0), (4.1, 0.8), (7.3, 0.6), (13.9, 0.45), (31.7, 0.35))
CONTRAST = 2.4

# ROI: chamfer-sized, at a fixed backdrop position. Columns [0, STRIP_X) are
# backdrop outside the window, [STRIP_X, STRIP_X + STRIP_W) the chamfer and
# the rest the flat face, as in the sweep's bevel band.
ROI_X, ROI_Y, ROI_W, ROI_H = 401, 287, 64, 96
STRIP_X, STRIP_W = 24, 12
SEARCH_X, SEARCH_Y = 24, 3
WINDOWS = (5, 7, 11)
# Ambiguity thresholds: a window flatter than this (linear std) has no
# texture; a second peak, away from the best by more than one pixel, within
# this ZNCC margin of it makes the match non-unique; a best score below the
# floor is too weak to trust.
FLAT_STD = 0.004
UNIQUE_MARGIN = 0.02
SCORE_FLOOR = 0.8
# Gauss-Newton fit residual, RMS over the window's own RMS spread.
MISFIT = 0.1


def decode(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def encode(x):
    x = min(max(x, 0.0), 1.0)
    return 12.92 * x if x <= 0.0031308 else 1.055 * x ** (1 / 2.4) - 0.055


DECODE8 = [decode(i / 255) for i in range(256)]


def quantize(x):
    """Linear value -> the linear value an 8-bit sRGB capture holds."""
    return DECODE8[round(encode(x) * 255)]


def mix64(x):
    x = (x + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
    x = ((x ^ (x >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
    x = ((x ^ (x >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
    return x ^ (x >> 31)


def lattice(seed, octave, i, j):
    h = mix64(mix64(mix64(seed * 8 + octave) ^ (i & 0xFFFFFFFF)) ^ (j & 0xFFFFFFFF))
    return (h >> 11) / float(1 << 53)


def aperiodic_code(seed, x, y):
    """sRGB code fraction of the aperiodic field at pixel (x, y).

    A function of absolute coordinates, so a crop equals the same region of
    the full backdrop.
    """
    total = weight = 0.0
    for k, (spacing, w) in enumerate(OCTAVES):
        ox = lattice(seed, k, -1, 0) * spacing
        oy = lattice(seed, k, 0, -1) * spacing
        fx, fy = (x + ox) / spacing, (y + oy) / spacing
        i, j = math.floor(fx), math.floor(fy)
        tx, ty = fx - i, fy - j
        tx = tx * tx * tx * (tx * (tx * 6 - 15) + 10)
        ty = ty * ty * ty * (ty * (ty * 6 - 15) + 10)
        a = lattice(seed, k, i, j) + (lattice(seed, k, i + 1, j) - lattice(seed, k, i, j)) * tx
        b = lattice(seed, k, i, j + 1) + (lattice(seed, k, i + 1, j + 1)
                                          - lattice(seed, k, i, j + 1)) * tx
        total += w * (a + (b - a) * ty)
        weight += w
    return min(max(0.5 + CONTRAST * (total / weight - 0.5), 0.02), 0.98)


# The sweep's make_backdrop, as luminance: four quadrant colours under 1 px
# grid lines every GRID_PERIOD px.
QUADRANTS = ((150, 60, 52), (120, 96, 40), (38, 44, 62), (48, 120, 80))
LINE = 226


def luminance_code(rgb):
    lin = sum(w * decode(c / 255) for w, c in zip((0.2126, 0.7152, 0.0722), rgb))
    return encode(lin)


def grid_code(x, y):
    if x % GRID_PERIOD == 0 or y % GRID_PERIOD == 0:
        return LINE / 255
    q = (0 if x < 640 else 1) if y < 360 else (2 if x < 640 else 3)
    return luminance_code(QUADRANTS[q])


class Field:
    """A linear-light crop of a backdrop: rows of floats, origin (x0, y0)."""

    def __init__(self, rows, x0, y0):
        self.x0, self.y0, self.w, self.h = x0, y0, len(rows[0]), len(rows)
        self.rows = rows

    @classmethod
    def of(cls, code, x0, y0, w, h):
        """The backdrop as its 8-bit PNG holds it, decoded to linear."""
        return cls([[DECODE8[round(code(x0 + i, y0 + j) * 255)] for i in range(w)]
                    for j in range(h)], x0, y0)

    def gradients(self):
        """Central-difference x and y gradients, one-sided at the border."""
        w, h, rows = self.w, self.h, self.rows
        gx = [[(row[min(i + 1, w - 1)] - row[max(i - 1, 0)]) / (2 if 0 < i < w - 1 else 1)
               for i in range(w)] for row in rows]
        gy = [[(rows[min(j + 1, h - 1)][i] - rows[max(j - 1, 0)][i])
               / (2 if 0 < j < h - 1 else 1) for i in range(w)] for j in range(h)]
        return Field(gx, self.x0, self.y0), Field(gy, self.x0, self.y0)

    def at(self, x, y):
        """Bilinear sample at absolute coordinates, clamped to the crop."""
        fx = min(max(x - self.x0, 0.0), self.w - 1.0)
        fy = min(max(y - self.y0, 0.0), self.h - 1.0)
        i, j = min(int(fx), self.w - 2), min(int(fy), self.h - 2)
        tx, ty = fx - i, fy - j
        r0, r1 = self.rows[j], self.rows[j + 1]
        a = r0[i] + (r0[i + 1] - r0[i]) * tx
        b = r1[i] + (r1[i + 1] - r1[i]) * tx
        return a + (b - a) * ty


def margin_field(code):
    m = max(SEARCH_X, SEARCH_Y) + max(WINDOWS) + 24
    return Field.of(code, ROI_X - m, ROI_Y - m, ROI_W + 2 * m, ROI_H + 2 * m)


# ------------------------------------------------------------- the cases

def in_strip(x):
    return STRIP_X <= x < STRIP_X + STRIP_W


def warp_none(x, y):
    return 0.0, 0.0


def warp_rigid(dx, dy):
    return lambda x, y: (dx, dy)


def warp_chamfer(s):
    return lambda x, y: (s, 0.0) if in_strip(x) else (0.0, 0.0)


def warp_smooth(ax, ay):
    """A smooth 2-D field over the whole ROI, zero at its left border."""
    def u(x, y):
        ramp = x / (ROI_W - 1)
        return (ax * ramp * math.sin(2 * math.pi * y / 48),
                ay * ramp * math.cos(2 * math.pi * y / 40))
    return u


def photo_none(x, v):
    return v


def photo_gain(g):
    return lambda x, v: g * v


def photo_ramp(x, v):
    return v * (0.75 + 0.4 * x / (ROI_W - 1))


# The shader's terms at a 45-degree chamfer, attenuation #dfe8ff over 60 px
# at thickness 20, and Schlick at ior 1.5: Beer-Lambert tints each channel
# along a path of thickness / cos, sqrt(2) times longer on the chamfer than on
# the face, and the chamfer adds a specular glint (at full facing, its
# largest). On a grey backdrop the transmitted luminance is the backdrop's
# times the luminance-weighted channel attenuations. Outside the window the
# backdrop is untouched.
ATTENUATION = (0xdf, 0xe8, 0xff)


def attenuation(path):
    return sum(w * decode(c / 255) ** (path / 60)
               for w, c in zip((0.2126, 0.7152, 0.0722), ATTENUATION))


ATT_FACE = attenuation(20)
ATT_CHAMFER = attenuation(20 * math.sqrt(2))
SPECULAR = 0.04 + 0.96 * (1 - math.sqrt(0.5)) ** 5


def photo_chamfer(x, v):
    if x < STRIP_X:
        return v
    if in_strip(x):
        return ATT_CHAMFER * v + SPECULAR
    return ATT_FACE * v


CASES = (
    # name, kind, warp, photometric
    ('repeat', 'identity', warp_none, photo_none),
    ('rigid 0.5,0', 'rigid', warp_rigid(0.5, 0.0), photo_none),
    ('rigid 3.4,-1.6', 'rigid', warp_rigid(3.4, -1.6), photo_none),
    ('rigid 20,0 (one grid period)', 'rigid', warp_rigid(20.0, 0.0), photo_none),
    ('chamfer 1', 'warp', warp_chamfer(1.0), photo_none),
    ('chamfer 3', 'warp', warp_chamfer(3.0), photo_none),
    ('chamfer 6', 'warp', warp_chamfer(6.0), photo_none),
    ('chamfer 12', 'warp', warp_chamfer(12.0), photo_none),
    ('chamfer 20', 'warp', warp_chamfer(20.0), photo_none),
    ('smooth 3,1.5', 'warp', warp_smooth(3.0, 1.5), photo_none),
    ('gain 1.35', 'photometric', warp_none, photo_gain(1.35)),
    ('ramp 0.75-1.15', 'photometric', warp_none, photo_ramp),
    ('chamfer tint+glint', 'photometric', warp_none, photo_chamfer),
    ('chamfer 3 + tint+glint', 'combined', warp_chamfer(3.0), photo_chamfer),
    ('chamfer 6 + tint+glint', 'combined', warp_chamfer(6.0), photo_chamfer),
)


def render(field, warp, photo):
    """The ROI as an 8-bit capture of the warped, photometrically changed field."""
    rows = []
    for j in range(ROI_H):
        row = []
        for i in range(ROI_W):
            ux, uy = warp(i, j)
            v = field.at(ROI_X + i + ux, ROI_Y + j + uy)
            row.append(quantize(photo(i, v)))
        rows.append(row)
    return rows


def flat_field(photo):
    """Recover the photometric layer from two uniform-backdrop renders.

    The shader's attenuation multiplies and its specular adds, both
    independent of backdrop content, so renders over uniform g1 and g2
    give att = (R2 - R1) / (g2 - g1) and spec = R1 - att * g1 per pixel.
    """
    g1, g2 = 0.2, 0.6
    layers = []
    for i in range(ROI_W):
        r1, r2 = quantize(photo(i, g1)), quantize(photo(i, g2))
        att = (r2 - r1) / (g2 - g1)
        layers.append((att, r1 - att * g1))
    return layers


def correct(rows, layers):
    return [[(v - layers[i][1]) / layers[i][0] for i, v in enumerate(row)] for row in rows]


# ------------------------------------------------------------- estimator

def box_rows(rows, r):
    """Sums over (2r+1)^2 windows, valid region only."""
    k = 2 * r + 1
    hs = []
    for row in rows:
        c = [0.0, *accumulate(row)]
        hs.append(list(map(operator.sub, c[k:], c[:-k])))
    cum = [[0.0] * len(hs[0])]
    for h in hs:
        cum.append(list(map(operator.add, cum[-1], h)))
    return [list(map(operator.sub, cum[j + k], cum[j])) for j in range(len(hs) - k + 1)]


def estimate(ref, out, win):
    """ZNCC block matching of each out window against ref, integer search
    over [-SEARCH_X, SEARCH_X] x [-SEARCH_Y, SEARCH_Y], then a parabola
    through the peak and its neighbours on each axis.

    Returns, per valid pixel (window inside the ROI) in row-major order,
    (x, y, ux, uy, score, flag) where flag is '' or the ambiguity reason.
    """
    r = win // 2
    n = win * win
    vw, vh = ROI_W - 2 * r, ROI_H - 2 * r
    sa = box_rows(out, r)
    saa = box_rows([[v * v for v in row] for row in out], r)
    ox, oy = ROI_X - ref.x0, ROI_Y - ref.y0
    bw, bh = ROI_W + 2 * SEARCH_X, ROI_H + 2 * SEARCH_Y
    bsub = [ref.rows[oy - SEARCH_Y + j][ox - SEARCH_X:ox - SEARCH_X + bw] for j in range(bh)]
    sb_all = box_rows(bsub, r)
    sbb_all = box_rows([[v * v for v in row] for row in bsub], r)

    var_a = [[(q - s * s / n) for s, q in zip(srow, qrow)] for srow, qrow in zip(sa, saa)]
    shifts = [(dx, dy) for dy in range(-SEARCH_Y, SEARCH_Y + 1)
              for dx in range(-SEARCH_X, SEARCH_X + 1)]
    volume = []
    for dx, dy in shifts:
        prod = [list(map(operator.mul, out[j],
                         bsub[j + SEARCH_Y + dy][SEARCH_X + dx:SEARCH_X + dx + ROI_W]))
                for j in range(ROI_H)]
        sab = box_rows(prod, r)
        scores = array('d')
        for j in range(vh):
            sb = sb_all[j + SEARCH_Y + dy][SEARCH_X + dx:SEARCH_X + dx + vw]
            sbb = sbb_all[j + SEARCH_Y + dy][SEARCH_X + dx:SEARCH_X + dx + vw]
            for a, b, bb, ab, va in zip(sa[j], sb, sbb, sab[j], var_a[j]):
                vb = bb - b * b / n
                den = va * vb
                scores.append((ab - a * b / n) / math.sqrt(den) if den > 1e-18 else 0.0)
        volume.append(scores)

    index = {s: k for k, s in enumerate(shifts)}
    flat_var = FLAT_STD * FLAT_STD * n
    result = []
    for p in range(vw * vh):
        j, i = divmod(p, vw)
        best_k = max(range(len(shifts)), key=lambda k: volume[k][p])
        best = volume[best_k][p]
        bx, by = shifts[best_k]
        second = max((volume[k][p] for k, (dx, dy) in enumerate(shifts)
                      if max(abs(dx - bx), abs(dy - by)) > 1), default=-1.0)

        def refine(d, lo, hi, key):
            if d in (lo, hi):
                return float(d), True
            m, c, pl = (volume[index[key(d - 1)]][p], volume[index[key(d)]][p],
                        volume[index[key(d + 1)]][p])
            den = m - 2 * c + pl
            return (d + 0.5 * (m - pl) / den if den < 0 else float(d)), False

        ux, edge_x = refine(bx, -SEARCH_X, SEARCH_X, lambda d: (d, by))
        uy, edge_y = refine(by, -SEARCH_Y, SEARCH_Y, lambda d: (bx, d))
        if var_a[j][i] < flat_var:
            flag = 'flat'
        elif edge_x or edge_y:
            flag = 'edge'
        elif best < SCORE_FLOOR:
            flag = 'weak'
        elif best - second < UNIQUE_MARGIN:
            flag = 'nonunique'
        else:
            flag = ''
        result.append((i + r, j + r, ux, uy, best, flag))
    return result


def solve4(m, v):
    """Gaussian elimination with partial pivoting; None when singular."""
    a = [row[:] + [b] for row, b in zip(m, v)]
    for c in range(4):
        piv = max(range(c, 4), key=lambda k: abs(a[k][c]))
        if abs(a[piv][c]) < 1e-12:
            return None
        a[c], a[piv] = a[piv], a[c]
        for k in range(c + 1, 4):
            f = a[k][c] / a[c][c]
            for t in range(c, 5):
                a[k][t] -= f * a[c][t]
    x = [0.0] * 4
    for c in range(3, -1, -1):
        x[c] = (a[c][4] - sum(a[c][t] * x[t] for t in range(c + 1, 4))) / a[c][c]
    return x


def refine(ref, grads, out, est, win, iters=6):
    """Gauss-Newton on out(q) = g * ref(q + u) + b over each window, seeded
    with the integer ZNCC peak. Gain and offset absorb a locally affine
    photometric change, as ZNCC does. A pixel whose refinement leaves the
    seed's pixel cell, or whose system is singular, keeps its ZNCC estimate
    and is flagged 'diverged'. One whose fitted window still leaves a
    residual above MISFIT of the window's own spread is flagged 'misfit': a
    window straddling a discontinuity in the warp fits no single shift.
    """
    gx, gy = grads
    r = win // 2
    refined = []
    for x, y, ux0, uy0, score, flag in est:
        if flag:
            refined.append((x, y, ux0, uy0, score, flag))
            continue
        sx, sy = round(ux0), round(uy0)
        ux, uy, g, b = float(sx), float(sy), 1.0, 0.0
        ok = True
        sse = 0.0
        n = win * win
        mean = sum(out[j][i] for j in range(y - r, y + r + 1)
                   for i in range(x - r, x + r + 1)) / n
        spread = sum((out[j][i] - mean) ** 2 for j in range(y - r, y + r + 1)
                     for i in range(x - r, x + r + 1))
        for _ in range(iters):
            h = [[0.0] * 4 for _ in range(4)]
            rhs = [0.0] * 4
            sse = 0.0
            for j in range(y - r, y + r + 1):
                orow = out[j]
                for i in range(x - r, x + r + 1):
                    ax, ay = ROI_X + i + ux, ROI_Y + j + uy
                    bv = ref.at(ax, ay)
                    jac = (g * gx.at(ax, ay), g * gy.at(ax, ay), bv, 1.0)
                    res = orow[i] - (g * bv + b)
                    sse += res * res
                    for c in range(4):
                        rhs[c] += jac[c] * res
                        hc = h[c]
                        for t in range(c, 4):
                            hc[t] += jac[c] * jac[t]
            for c in range(4):
                for t in range(c):
                    h[c][t] = h[t][c]
            step = solve4(h, rhs)
            if step is None:
                ok = False
                break
            ux, uy, g, b = ux + step[0], uy + step[1], g + step[2], b + step[3]
            if abs(ux - sx) > 1 or abs(uy - sy) > 1:
                ok = False
                break
            if abs(step[0]) < 1e-4 and abs(step[1]) < 1e-4:
                break
        if ok:
            misfit = sse > MISFIT * MISFIT * spread
            refined.append((x, y, ux, uy, score, 'misfit' if misfit else ''))
        else:
            refined.append((x, y, ux0, uy0, score, 'diverged'))
    return refined


# ------------------------------------------------------------- scoring

def region(x, r):
    """Where a pixel's window sits relative to the chamfer strip."""
    lo, hi = STRIP_X, STRIP_X + STRIP_W
    if x + r < lo or x - r >= hi:
        return 'outside' if x < lo else 'face'
    if x - r >= lo and x + r < hi:
        return 'strip'
    return 'boundary'


def pct(values, q):
    if not values:
        return None
    s = sorted(values)
    return s[min(len(s) - 1, int(q * len(s)))]


def summarize(est, warp, r):
    groups = {}
    for x, y, ux, uy, score, flag in est:
        tx, ty = warp(x, y)
        err = math.hypot(ux - tx, uy - ty)
        for name in ('all', region(x, r)):
            g = groups.setdefault(name, {'n': 0, 'flags': {}, 'errs': [], 'all_errs': []})
            g['n'] += 1
            g['all_errs'].append(err)
            if flag:
                g['flags'][flag] = g['flags'].get(flag, 0) + 1
            else:
                g['errs'].append(err)
    out = {}
    for name, g in groups.items():
        out[name] = {
            'pixels': g['n'],
            'confident': len(g['errs']) / g['n'],
            'flags': g['flags'],
            'median': pct(g['errs'], 0.5),
            'p95': pct(g['errs'], 0.95),
            'max': max(g['errs']) if g['errs'] else None,
            'p95_unflagged_and_flagged': pct(g['all_errs'], 0.95),
            'wrong_confident': sum(e > 1.0 for e in g['errs']),
        }
    return out


def rmse_code(ref, rows, x0, x1):
    """Luminance RMSE in 8-bit sRGB code values between ref and rows over
    columns [x0, x1), the comparison the sweep tabulates."""
    total = n = 0
    for j in range(ROI_H):
        for i in range(x0, x1):
            a = round(encode(ref.at(ROI_X + i, ROI_Y + j)) * 255)
            b = round(encode(rows[j][i]) * 255)
            total += (a - b) ** 2
            n += 1
    return math.sqrt(total / n)


def calibrate(seed, windows):
    backdrops = {
        f'aperiodic-s{seed}': margin_field(lambda x, y: aperiodic_code(seed, x, y)),
        f'grid{GRID_PERIOD}': margin_field(grid_code),
    }
    grads = {name: field.gradients() for name, field in backdrops.items()}
    results = []
    for bname, field in backdrops.items():
        for name, kind, warp, photo in CASES:
            rows = render(field, warp, photo)
            variants = [('raw', rows)]
            if kind in ('photometric', 'combined') and photo is photo_chamfer:
                variants.append(('flat-field', correct(rows, flat_field(photo))))
            for variant, img in variants:
                entry = {
                    'backdrop': bname, 'case': name, 'kind': kind, 'input': variant,
                    'rmse_strip': rmse_code(field, img, STRIP_X, STRIP_X + STRIP_W),
                    'rmse_band': rmse_code(field, img, 0, ROI_W),
                    'windows': {},
                }
                for win in windows:
                    est = estimate(field, img, win)
                    entry['windows'][str(win)] = {
                        'parabola': summarize(est, warp, win // 2),
                        'gauss-newton': summarize(refine(field, grads[bname], img, est, win),
                                                  warp, win // 2),
                    }
                results.append(entry)
                print(f'{bname:16} {name:30} {variant:10} done', file=sys.stderr)
    return results


def fmt(v, digits=2):
    return '—' if v is None else f'{v:.{digits}f}'


def markdown(results, seed, windows):
    lines = [
        '# Warp calibration results',
        '',
        f'Generated by `docs/materials/scripts/warp-calibration.py calibrate`, seed {seed}.',
        f'ROI {ROI_W}x{ROI_H}+{ROI_X}+{ROI_Y}; chamfer strip x {STRIP_X}-{STRIP_X + STRIP_W - 1};'
        f' search ±{SEARCH_X} x ±{SEARCH_Y} px; ambiguity: flat std < {FLAT_STD},'
        f' second peak within {UNIQUE_MARGIN}, score < {SCORE_FLOOR}, peak on the search edge.',
        '',
        'Errors are px, over confident (unflagged) pixels. `wrong` counts confident pixels'
        ' more than 1 px from the known field. RMSE is luminance, 8-bit code values,'
        ' against the unchanged backdrop.',
        '',
    ]
    for win, method in product(windows, ('parabola', 'gauss-newton')):
        lines += [f'## Window {win}x{win}, {method}', '',
                  '| backdrop | case | input | region | confident | median | p95 | max | wrong |'
                  ' flags |',
                  '|---|---|---|---|---|---|---|---|---|---|']
        for e in results:
            s = e['windows'][str(win)][method]
            for reg in ('all', 'outside', 'boundary', 'strip', 'face'):
                if reg not in s:
                    continue
                g = s[reg]
                flags = ', '.join(f'{k} {v}' for k, v in sorted(g['flags'].items())) or '—'
                lines.append(f"| {e['backdrop']} | {e['case']} | {e['input']} | {reg} |"
                             f" {g['confident']:.0%} | {fmt(g['median'])} | {fmt(g['p95'])} |"
                             f" {fmt(g['max'])} | {g['wrong_confident']} | {flags} |")
        lines.append('')
    lines += ['## Regional RMSE', '', '| backdrop | case | input | strip | band |',
              '|---|---|---|---|---|']
    for e in results:
        lines.append(f"| {e['backdrop']} | {e['case']} | {e['input']} |"
                     f" {e['rmse_strip']:.1f} | {e['rmse_band']:.1f} |")
    return '\n'.join(lines) + '\n'


# ------------------------------------------------------------- backdrop PNG

def png_bytes(w, h, rows, text):
    def chunk(tag, data):
        body = tag + data
        return struct.pack('>I', len(data)) + body + struct.pack('>I', zlib.crc32(body))
    raw = b''.join(b'\x00' + bytes(row) for row in rows)
    return (b'\x89PNG\r\n\x1a\n'
            + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
            + b''.join(chunk(b'tEXt', f'{k}\x00{v}'.encode('latin-1')) for k, v in text)
            + chunk(b'IDAT', zlib.compress(raw, 9))
            + chunk(b'IEND', b''))


def backdrop(path, seed, w=1280, h=720):
    """The calibrated field itself, grey (R = G = B = its code), so the PNG
    holds exactly what the calibration measured. Tinting it by the grid's
    quadrant colours clips the bright quadrants, which bends the affine
    photometric model the estimator relies on."""
    rows = []
    for y in range(h):
        row = bytearray()
        for x in range(w):
            row += bytes((round(aperiodic_code(seed, x, y) * 255),)) * 3
        rows.append(row)
    text = [('Software', 'niri-material warp-calibration.py'),
            ('Comment', f'backdrop aperiodic-s{seed}; octaves {OCTAVES}; contrast {CONTRAST}')]
    Path(path).write_bytes(png_bytes(w, h, rows, text))


def main(argv=None):
    ap = argparse.ArgumentParser(
        description='Offline calibration of a local glass-warp estimator.')
    sub = ap.add_subparsers(dest='cmd', required=True)
    c = sub.add_parser('calibrate')
    c.add_argument('--out', required=True, type=Path)
    c.add_argument('--seed', type=int, default=SEED)
    c.add_argument('--windows', default=','.join(map(str, WINDOWS)))
    b = sub.add_parser('backdrop')
    b.add_argument('path', type=Path)
    b.add_argument('--seed', type=int, default=SEED)
    args = ap.parse_args(argv)
    if args.cmd == 'backdrop':
        backdrop(args.path, args.seed)
        return 0
    try:
        windows = tuple(int(w) for w in args.windows.split(','))
    except ValueError:
        print(f'--windows: not a list of integers: {args.windows}', file=sys.stderr)
        return 2
    if any(w < 3 or w % 2 == 0 for w in windows):
        print('--windows: each window must be odd and at least 3', file=sys.stderr)
        return 2
    args.out.mkdir(parents=True, exist_ok=True)
    results = calibrate(args.seed, windows)
    (args.out / 'results.json').write_text(json.dumps(
        {'seed': args.seed, 'roi': [ROI_X, ROI_Y, ROI_W, ROI_H],
         'strip': [STRIP_X, STRIP_W], 'search': [SEARCH_X, SEARCH_Y],
         'results': results}, indent=1) + '\n')
    (args.out / 'results.md').write_text(markdown(results, args.seed, windows))
    return 0


if __name__ == '__main__':
    sys.exit(main())
