#!/usr/bin/env python3
"""The owner's measured sheet for the rigid tilt swing.

Usage: swing-measured-sheet.py <ring-motion-clips run dir>

Reads the tilt-swing* bursts that ring-motion-clips.sh wrote (frames.txt, the
gain offsets in clips.txt, the sequences' configs) and writes
swing-measured.json and swing-measured-sheet.png: a plot of the gaining
(left) pane's side-edge displacement per captured frame, and zoomed strips of
both side edges against their rest position, one column per frame through the
swing. The swing is a few pixels at small angles, which crops of a corner do
not show; the edges are measured, never assumed. The rest frame is each
burst's last. Assumes the script's scene: 1280x720, two panes, gaps 24.
Needs Pillow, matplotlib and the DejaVu fonts.
"""
import io
import json
import math
import os
import re
import sys

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from PIL import Image, ImageDraw, ImageFont

FONTS = '/usr/share/fonts/TTF'
BG, TEXT, DIM, TAG, REST = '#1e1e1e', '#ffffff', '#cccccc', '#ffeb3b', '#00e5ff'
ROWS = 200, 600            # scanlines the edges are measured on, clear of the title bar
LEFT_REST, RIGHT_REST = 24, 628   # the gaining pane's edges at rest in the script's scene
GAP_X = 640                # inside the gap between the panes
SEQUENCES = [   # id, label, config, colour, line style
    ('A', 'tilt-swing', 'tilt-swing.kdl', '#4caf50', '-'),
    ('B', 'tilt-swing-beam', 'tilt-swing-beam.kdl', '#ffb300', '--'),
    ('C', 'tilt-swing-slow', 'tilt-swing-slow.kdl', '#42a5f5', '-'),
    ('D', 'tilt-swing-flipped', 'tilt-swing.kdl', '#ef5350', '-'),
]


def font(size, bold=False):
    return ImageFont.truetype(os.path.join(FONTS, 'DejaVuSans-Bold.ttf' if bold else 'DejaVuSans.ttf'), size)


def fail(msg):
    sys.exit(f'swing-measured-sheet: {msg}')


def config(path):
    text = open(path).read()

    def get(key):
        m = re.search(rf'\b{key} ([0-9.]+);', text)
        if m is None:
            fail(f'{path} sets no {key}')
        return float(m.group(1))
    return {'tilt': get('view-tilt'), 'kappa': get('view-distance'), 'settle': get('view-tilt-settle'),
            'speed': get('ring-beam-speed'), 'glow': get('ring-glow')}


def crossing(profile, start, step, threshold):
    """First position from `start` along `step` where `profile` falls below `threshold`, sub-pixel."""
    x = start
    while 0 < x + step < len(profile) - 1:
        if profile[x + step] < threshold <= profile[x]:
            return x + step * (profile[x] - threshold) / (profile[x] - profile[x + step])
        x += step
    fail(f'no edge crossing from x={start}')


def luma(path):
    return np.asarray(Image.open(path).convert('L'), dtype=float)


def thresholds(rest):
    """Per scanline, the midpoints between the background beside each edge and the pane at rest."""
    out = {}
    for y in ROWS:
        row = rest[y - 1:y + 2].mean(axis=0)
        left, right = (row[4] + row[LEFT_REST + 36]) / 2, (row[GAP_X] + row[RIGHT_REST - 68]) / 2
        if row[4] - row[LEFT_REST + 36] < 30 or row[GAP_X] - row[RIGHT_REST - 68] < 30:
            fail(f'scanline {y} of the rest frame has no contrast at the pane edges: not the script\'s scene?')
        out[y] = left, right
    return out


def edges(img, th):
    left, right = [], []
    for y in ROWS:
        row = img[y - 1:y + 2].mean(axis=0)
        left.append(crossing(row, 4, 1, th[y][0]))
        right.append(crossing(row, GAP_X, -1, th[y][1]))
    return float(np.mean(left)), float(np.mean(right))


def measure(run, label):
    gains = [l.split() for l in open(f'{run}/clips.txt') if l.startswith(f'{label}: gain at ')]
    if len(gains) != 1:
        fail(f'{label}: expected one gain line in clips.txt, found {len(gains)}')
    gain = float(gains[0][3])
    frames = [l.split() for l in open(f'{run}/{label}/frames.txt')]
    rest = luma(f'{run}/{label}/{frames[-1][0]}')
    th = thresholds(rest)
    rl, rr = edges(rest, th)
    if abs(rl - LEFT_REST) > 6 or abs(rr - RIGHT_REST) > 6:
        fail(f'{label}: rest edges at x={rl:.1f}, {rr:.1f}, expected {LEFT_REST}, {RIGHT_REST}')
    rows = []
    for f, t in frames:
        l, r = edges(luma(f'{run}/{label}/{f}'), th)
        rows.append({'frame': f, 't': round(float(t) - gain, 3), 'left_in': round(l - rl, 2), 'right_in': round(rr - r, 2)})
    return {'gain': gain, 'rest': [round(rl, 2), round(rr, 2)], 'rows': rows}


def signed(r):
    """+ = the right (origin-side) edge pulled in, - = the left edge pulled in."""
    return r['right_in'] if abs(r['right_in']) >= abs(r['left_in']) else -r['left_in']


def describe(cid, cfg):
    beam = f'beam on (ring-glow {cfg["glow"]:g})' if cfg['speed'] > 0 else 'beam off'
    parts = [beam, f'settle {cfg["settle"]:g} ms']
    parts.append('sign FLIPPED (scratch build)' if cid == 'D' else 'shipped sign')
    return parts


def plot(present, data, cfgs):
    plt.rcParams.update({'font.size': 15, 'axes.edgecolor': '#aaa', 'axes.labelcolor': '#ddd',
                         'xtick.color': '#ddd', 'ytick.color': '#ddd', 'text.color': '#ddd'})
    fig, ax = plt.subplots(figsize=(15, 5.2), dpi=100)
    fig.patch.set_facecolor(BG)
    ax.set_facecolor('#262626')
    peak = max(abs(signed(r)) for s in present for r in data[s[1]]['rows'])
    t_end = max(cfgs[s[1]]['settle'] for s in present) / 1000 + 0.2
    for cid, label, _, colour, style in present:
        rs = [r for r in data[label]['rows'] if r['t'] <= t_end]
        ax.plot([r['t'] for r in rs], [signed(r) for r in rs], style, color=colour, lw=2.5, marker='o', ms=7,
                label=f'{cid}: {label} ({", ".join(describe(cid, cfgs[label]))})')
    for settle in sorted({cfgs[s[1]]['settle'] for s in present}):
        ax.axvline(settle / 1000, color='#888', ls=':', lw=1.5)
        ax.text(settle / 1000 + 0.01, peak * 0.9, f'{settle:g} ms settle ends', color='#aaa', fontsize=13)
    ax.axhline(0, color='#777', lw=1)
    ax.set_xlim(-0.05, t_end)
    ax.set_ylim(-peak * 1.2 - 1, peak * 1.2 + 1)
    ax.set_xlabel('time after the focus gain, s  (dots = captured frames)')
    ax.set_ylabel('edge pulled in, px\n+ right (origin) edge   − left edge')
    ax.grid(color='#333')
    ax.legend(loc='lower right', fontsize=12.5, facecolor='#262626', edgecolor='#555')
    fig.tight_layout()
    buf = io.BytesIO()
    fig.savefig(buf, format='png', facecolor=fig.get_facecolor())
    buf.seek(0)
    return Image.open(buf).convert('RGB'), peak


def main():
    if len(sys.argv) != 2:
        fail('usage: swing-measured-sheet.py <run dir>')
    run = sys.argv[1].rstrip('/')
    present = [s for s in SEQUENCES if os.path.isfile(f'{run}/{s[1]}/frames.txt')]
    if not present:
        fail(f'no tilt-swing bursts under {run}')
    cfgs = {label: config(f'{run}/{path}') for _, label, path, _, _ in present}
    data = {label: measure(run, label) for _, label, _, _, _ in present}
    json.dump({'run': os.path.basename(run), 'configs': cfgs, 'sequences': data},
              open(f'{run}/swing-measured.json', 'w'), indent=1)
    graph, peak = plot(present, data, cfgs)
    tilt, kappa = cfgs[present[0][1]]['tilt'], cfgs[present[0][1]]['kappa']

    # Strips: wide enough for the peak plus a margin either side, zoomed to about 200 px.
    reach = math.ceil(peak) + 20
    strips = [(max(0, LEFT_REST - 20), LEFT_REST + reach, LEFT_REST), (RIGHT_REST - reach, min(650, RIGHT_REST + 20), RIGHT_REST)]
    zoom = max(1, min(3, 200 // (strips[0][1] - strips[0][0])))
    y0, y1 = 310, 410
    sws = [(x1 - x0) * zoom for x0, x1, _ in strips]
    sh = (y1 - y0) * zoom
    cell_w, cell_h = sum(sws) + 12, sh + 34
    # Columns: one per 0.1 s from the gain until every sequence has rested for a frame.
    moving = [r['t'] for s in present for r in data[s[1]]['rows'] if abs(signed(r)) >= 0.5]
    targets = [round(0.1 * i, 1) for i in range(min(10, int(max(moving) / 0.1) + 2))]
    label_w, pad = 330, 14
    W = max(graph.width + 40, label_w + len(targets) * (cell_w + pad) + 20)
    head_h = 196
    grid_top = head_h + graph.height + 70
    H = grid_top + len(present) * (cell_h + pad) + 60
    img = Image.new('RGB', (W, H), BG)
    d = ImageDraw.Draw(img)
    pane_w = RIGHT_REST - LEFT_REST
    d.text((20, 14), f'Rigid tilt swing at view-tilt {tilt:g}, view-distance {kappa:g}: peak edge pull-in {peak:.0f} px '
           f'of a {pane_w} px pane. Do the motion, the sign and the settle read right?', font=font(30, True), fill=TEXT)
    lines = [
        f'Run {os.path.basename(run)}. The plot tracks the gaining (left) pane\'s side edges per captured frame, '
        'measured on two scanlines; the rest position is the burst\'s last frame.',
        f'Below, each cell is a {zoom}x zoom of that pane\'s left and right edges at mid-height; the CYAN line is where '
        'the edge sits at rest. A gap between edge and cyan line = the tilt.',
        'Look for: which edge pulls in (shipped sign: the right, origin-side edge; row D: the left), how far, and when '
        'each row is back on the line. The motion itself is in <sequence>.gif.',
        'Row B differs from A only by the beam, which runs under the title bar, outside these strips: judge it in '
        'tilt-swing-beam.gif. Green frame = control (row A).',
    ]
    for i, line in enumerate(lines):
        d.text((20, 62 + i * 30), line, font=font(19), fill=DIM)
    img.paste(graph, (20, head_h))
    d.text((20, grid_top - 46), 'Edge strips: rows = sequences, columns = captured frames after the focus gain '
           '(left half of each cell = left edge, right half = right edge)', font=font(21, True), fill=TEXT)
    for ri, (cid, label, _, colour, _) in enumerate(present):
        y = grid_top + ri * (cell_h + pad)
        if cid == 'A':
            d.rectangle((6, y - 7, W - 8, y + cell_h + 5), outline='#4caf50', width=3)
        d.text((20, y + 10), f'Row {cid}: {label}', font=font(21, True), fill=colour)
        for k, part in enumerate(describe(cid, cfgs[label])):
            d.text((20, y + 46 + k * 28), part, font=font(18), fill=DIM)
        rows = data[label]['rows']
        for ci, target in enumerate(targets):
            r = min(rows, key=lambda r: abs(r['t'] - target))
            x = label_w + ci * (cell_w + pad)
            frame = Image.open(f'{run}/{label}/{r["frame"]}').convert('RGB')
            sx = x
            for (x0, x1, rest), sw in zip(strips, sws):
                strip = frame.crop((x0, y0, x1, y1)).resize((sw, sh), Image.Resampling.NEAREST)
                lx = round((rest - x0) * zoom)
                ImageDraw.Draw(strip).line((lx, 0, lx, sh), fill=REST, width=2)
                img.paste(strip, (sx, y + 34))
                sx += sw + 12
            d.rectangle((x, y, x + cell_w, y + 32), fill='#111')
            d.text((x + 6, y + 4), f'{cid}{ci + 1}  {max(r["t"], 0.0):+.2f} s  {signed(r):+.0f} px', font=font(19, True), fill=TAG)
    d.text((20, H - 44), f'Edges: threshold crossings on scanlines {ROWS[0]} and {ROWS[1]} between the background beside '
           'the pane and the pane at rest, sub-pixel; values in swing-measured.json.', font=font(17), fill='#999')
    img.save(f'{run}/swing-measured-sheet.png')
    print(f'{run}/swing-measured-sheet.png: {len(present)} sequences, peak {peak:.1f} px')


if __name__ == '__main__':
    main()
