#!/usr/bin/env python3
"""The owner's still sheet for the rigid tilt (design 2026-10-09 §8.3).

Usage: view-tilt-sheet.py <VIEW_TILT_DUMP dir> <out dir>

Reads <dump>/sheet/*.png and geometry.json from `still_sheet_frames` and
writes sheet.png (the 3x3 grid, dark and light in each cell, the control,
the spill row), crops.png (1:1 crops of the advancing edge with a 4x zoom of
it, of whole text lines, of a corner on a diagonal swing, the spill where the
head crosses a wall, and the advancing edge of every cell at 4x) and
filmstrip.png (one swing at the middle cell). Wall widths and the spill are
measured from the wall-probe frames, never assumed. Needs Pillow (with
Image.get_flattened_data) and the DejaVu fonts.
"""
import json
import os
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFont

FONTS = '/usr/share/fonts/TTF'
GOLD, TEXT, DIM, BG = '#d4a62a', '#ffffff', '#cccccc', '#1e1e1e'
SCALE, GAP, MARGIN = 0.28, 10, 16
TS, KS = [6, 10, 14], [2.5, 6, 10]


def font(size, bold=False):
    return ImageFont.truetype(os.path.join(FONTS, 'DejaVuSans-Bold.ttf' if bold else 'DejaVuSans.ttf'), size)


def num(v):
    return f'{v:g}'


def main(dump, out):
    d = os.path.join(dump, 'sheet')
    with open(os.path.join(d, 'geometry.json')) as g:
        geo = json.load(g)
    x, y, w, h = geo['pane']
    # The crops stop short of all the second window draws in any frame: a forced pose (K, G,
    # H, I, J) tilts it too. The render asserts the pane lies wholly left of the cut.
    right = geo['cut']
    W0, H0 = geo['size']
    peak_ms, spill_ms, kc = geo['peak_ms'], geo['spill_ms'], geo['control_kappa']
    left = geo['advancing'][0] < 0
    lifted = 'left' if left else 'right'
    mid = int(y + h / 2)

    def clamp(rect):
        x0, y0, x1, y1 = rect
        return (max(0, int(x0)), max(0, int(y0)), min(W0, right, int(x1)), min(H0, int(y1)))

    def frame(name):
        return Image.open(os.path.join(d, f'{name}.png')).convert('RGB')

    def crop(im, rect):
        return im.crop(clamp(rect))

    def walls(name, rect):
        """The wall probe's pixels (pure magenta) of `name` inside `rect`."""
        im = crop(frame(name), rect)
        return sum(1 for p in im.get_flattened_data() if p == (255, 0, 255))

    def wall_width(name):
        """The wall's width on the advancing side, along the pane's middle row."""
        if left:
            return walls(name, (0, mid, x + w / 2, mid + 1))
        return walls(name, (x + w / 2, mid, W0, mid + 1))

    box = clamp((x - 80, y - 80, x + w + 80, y + h + 80))

    def cell(name):
        im = frame(name).crop(box)
        return im.resize((int(im.width * SCALE), int(im.height * SCALE)), Image.Resampling.LANCZOS)

    def spill(prefix):
        """How the spill-on frame differs from spill-off: (changed px, max levels, bbox).
        Fails unless every change brightens a wall-probe pixel, as the render asserts."""
        off, on, probe = frame(f'{prefix}-off'), frame(f'{prefix}-on'), frame(f'{prefix}-walls')
        changed, top = 0, 0
        for a, b, p in zip(off.get_flattened_data(), on.get_flattened_data(), probe.get_flattened_data()):
            if a == b:
                continue
            changed += 1
            if p != (255, 0, 255) or any(bc < ac for ac, bc in zip(a, b)):
                sys.exit(f'{prefix}: the spill changed a pixel off the walls, or darkened one')
            top = max(top, max(bc - ac for ac, bc in zip(a, b)))
        return changed, top, ImageChops.difference(off, on).getbbox()

    widths = {(t, k): wall_width(f'walls-t{num(t)}-k{num(k)}') for t in TS for k in KS}
    b2_changed, b2_top, _ = spill('spill')

    tw, th = cell('rest-dark').size
    pair = 2 * tw + GAP
    head, side = 196, 210
    rows = len(TS) + 1
    W = MARGIN * 2 + side + len(KS) * (pair + GAP)
    H = MARGIN * 2 + head + rows * (th + 46 + GAP) + 40 + 60
    sheet = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(sheet)
    draw.text((MARGIN, MARGIN), 'How far, and how much perspective, for the focus tilt?', TEXT, font(30, True))
    draw.text((MARGIN, MARGIN + 44), f'Each cell: the peak of one focus-gain swing ({peak_ms} ms of 900 ms), the accepted dark look (left) and a lighter look (right).', DIM, font(17))
    draw.text((MARGIN, MARGIN + 70), f'Look for depth you can read at the peak: the rounded rim turning toward you, the wall on the advancing ({lifted}) edge, the keystone.', DIM, font(17))
    draw.text((MARGIN, MARGIN + 96), "Wall: the slab side below the rim. Rim: the rounded bevel. Keystone: the near edge drawn taller than the far one. Wall spill: the focus beam's light on the wall.", DIM, font(17))
    none = sum(1 for v in widths.values() if v == 0)
    draw.text((MARGIN, MARGIN + 122), f'The wall is at most {max(widths.values())} px wide in any cell, and absent in {none} of 9: too thin to see at this scale. crops.png shows it at 4x for every cell.', DIM, font(17))
    for j, k in enumerate(KS):
        cx = MARGIN + side + j * (pair + GAP)
        draw.text((cx, MARGIN + head - 34), f'camera distance κ = {num(k)} pane widths', TEXT, font(19, True))
    for i, t in enumerate(TS):
        ry = MARGIN + head + i * (th + 46 + GAP)
        draw.text((MARGIN, ry + th // 2 - 12), f'peak angle {t}°', TEXT, font(21, True))
        for j, k in enumerate(KS):
            cx = MARGIN + side + j * (pair + GAP)
            cid = f'{"ABC"[i]}{j + 1}'
            for n, look in enumerate(['dark', 'light']):
                sheet.paste(cell(f'cell-t{num(t)}-k{num(k)}-{look}'), (cx + n * (tw + GAP), ry))
            draw.text((cx, ry + th + 6), f'{cid}   (dark | light)', DIM, font(16))
    # The extra row sits below a rule: it is not part of the κ columns.
    ry = MARGIN + head + len(TS) * (th + 46 + GAP) + 40
    draw.line((MARGIN, ry - 24, W - MARGIN, ry - 24), fill=DIM, width=1)
    draw.text((MARGIN, ry - 20 - 2), 'Not part of the grid:', DIM, font(15))
    draw.text((MARGIN, ry + th // 2 - 48), 'B2, wall spill', TEXT, font(21, True))
    draw.text((MARGIN, ry + th // 2 - 20), f'at {spill_ms} ms: off | on', DIM, font(17))
    if b2_changed == 0:
        s1, s2 = f'S1  wall spill off, {spill_ms} ms', f'S2  wall spill on, {spill_ms} ms: identical to S1'
        draw.text((MARGIN, ry + th // 2 + 4), 'no visible difference', DIM, font(17))
        draw.text((MARGIN, ry + th // 2 + 26), 'at B2; elsewhere:', DIM, font(17))
        draw.text((MARGIN, ry + th // 2 + 48), 'crops I, J', DIM, font(17))
    else:
        s1 = f'S1  wall spill off, {spill_ms} ms'
        s2 = f'S2  spill on: {b2_changed} wall px brighter, by up to {b2_top} levels'
        draw.text((MARGIN, ry + th // 2 + 4), 'close up: crops I, J', DIM, font(17))
    for j, (name, label) in enumerate([('spill-off', s1), ('spill-on', s2)]):
        cx = MARGIN + side + j * (tw + GAP)
        sheet.paste(cell(name), (cx, ry))
        draw.text((cx, ry + th + 6), label, DIM, font(16))
    cx = MARGIN + side + 2 * (pair + GAP)
    sheet.paste(cell('control-dark'), (cx, ry))
    draw.rectangle((cx - 3, ry - 3, cx + tw + 2, ry + th + 2), outline=GOLD, width=3)
    draw.text((cx, ry + th + 6), "K  control: the spike's 4° with the camera at 2000 px", GOLD, font(16, True))
    draw.text((cx, ry + th + 26), f'(κ ≈ {kc:.2f}, not κ 10; the pose is forced, dark look)', GOLD, font(16))
    draw.text((MARGIN, H - MARGIN - 26), 'Reply with a cell id (A1–C3), the spill (off or on: S1/S2, and crops I, J), and anything that reads wrong.', TEXT, font(19, True))
    sheet.save(os.path.join(out, 'sheet.png'))

    # The positive lobe lifts the edge along the swing's direction: the advancing edge. The
    # crops stay on the canvas, so a pane near the output's edge is cut there, not padded.
    if left:
        edge = (x - 100, mid - 160, x + 220, mid + 160)
        zoom = (x - 30, mid - 40, x + 50, mid + 40)
    else:
        edge = (x + w - 220, mid - 160, x + w + 100, mid + 160)
        zoom = (x + w - 50, mid - 40, x + w + 30, mid + 40)
    # The diagonal crop's pose lifts the corner along (0.6, 0.8): bottom right.
    corner = (x + w - 200, y + h - 200, x + w + 100, y + h + 100)
    # The wall's width across each lifted edge, 100 px from the corner.
    bottom = walls('walls-diagonal', (x + w - 100, y + h - 100, x + w - 99, y + h + 100))
    side_w = walls('walls-diagonal', (x + w - 100, y + h - 100, x + w + 100, y + h - 99))
    if bottom or side_w:
        corner_note = f'corner wall {bottom} px wide on the bottom edge, {side_w} px on the right'
    else:
        corner_note = 'no corner wall at 10°, κ 6'

    def text_rect(images, top, lines):
        """Whole text lines across the pane: from the first blank row at or below `top` to
        the first blank row at or past `lines` 20 px lines further, blank in every image."""
        def blank(yy):
            return all(im.crop((int(x + 16), yy, int(x + w - 16), yy + 1)).convert('L').getextrema()[1] < 120 for im in images)
        a = int(top)
        while not blank(a):
            a += 1
        b = a + 20 * lines
        while not blank(b):
            b += 1
        return (x - 20, a, x + w + 20, b + 1)

    peak_d, rest_d = frame('cell-t10-k6-dark'), frame('rest-dark')
    text = text_rect([peak_d, rest_d], y + 16 + 20 * 2, 6)
    big = lambda im: im.resize((im.width * 4, im.height * 4), Image.Resampling.NEAREST)
    top_changed, top_top, tb = spill('spill-top')
    if top_changed == 0:
        sys.exit('spill-top: the spill changed nothing under the head')
    # The head leads the lit run: crop its end.
    head_crop = (tb[2] - 200, tb[1] - 30, tb[2] + 20, tb[3] + 30)
    rows = [
        'At the peak of B2 (10°, κ 6), 1:1',
        [(f'A  advancing ({lifted}) edge, B2 dark', crop(peak_d, edge)),
         (f'B  advancing ({lifted}) edge, B2 light', crop(frame('cell-t10-k6-light'), edge))],
        [(f'C  A at 4x: the wall ({widths[(10, 6)]} px) is the grey strip on the slab\'s outer side', big(crop(peak_d, zoom))),
         ('D  the same edge at 4x, at rest: no wall', big(crop(rest_d, zoom)))],
        [(f'E  whole text lines, B2 dark at the peak (the {lifted} end is lifted)', crop(peak_d, text))],
        [('F  the same lines at rest', crop(rest_d, text))],
        'A diagonal swing, pose forced: 10°, κ 6, toward the bottom right, 1:1',
        [(f'G  bottom-right corner, dark: {corner_note}', crop(frame('diagonal-dark'), corner)),
         ('H  the same, light', crop(frame('diagonal-light'), corner))],
        f'The wall spill where the head crosses a wall, 4x. Pose forced (the gain never makes it): the top edge lifted, 10°, κ 6, {peak_ms} ms',
        [('I  spill off', big(crop(frame('spill-top-off'), head_crop))),
         (f'J  spill on: {top_changed} wall px brighter, by up to {top_top} levels; nothing else changes', big(crop(frame('spill-top-on'), head_crop)))],
        f'The advancing ({lifted}) edge of every dark cell at its peak, 4x; the wall width is measured on the middle row',
    ]
    for i, t in enumerate(TS):
        rows.append([(f'{"ABC"[i]}{j + 1}  {t}°, κ {num(k)}: wall {widths[(t, k)]} px',
                      big(crop(frame(f'cell-t{num(t)}-k{num(k)}-dark'), zoom))) for j, k in enumerate(KS)])
    label_font, head_font = font(15), font(18, True)

    def slot(label, im):
        return max(im.width, int(label_font.getlength(label)))

    def height(row):
        return 34 if isinstance(row, str) else max(im.height for _, im in row) + 40

    CW = MARGIN * 2 + max(sum(slot(*t) for t in row) + GAP * (len(row) - 1) for row in rows if not isinstance(row, str))
    CH = MARGIN * 2 + 84 + sum(height(row) for row in rows)
    page = Image.new('RGB', (CW, CH), BG)
    draw = ImageDraw.Draw(page)
    draw.text((MARGIN, MARGIN), 'Crops of the sheet at native resolution (1:1), and at 4x where marked', TEXT, font(24, True))
    draw.text((MARGIN, MARGIN + 38), "Look for: the wall on the lifted edge (A–D and the cell rows), text that stays crisp and straight under the tilt (E against F), the corner (G, H), the beam's light on a wall (I against J).", DIM, font(17))
    yy = MARGIN + 84
    for row in rows:
        if isinstance(row, str):
            draw.text((MARGIN, yy + 6), row, TEXT, head_font)
        else:
            xx = MARGIN
            for label, im in row:
                page.paste(im, (xx, yy))
                draw.text((xx, yy + im.height + 6), label, DIM, label_font)
                xx += slot(label, im) + GAP
        yy += height(row)
    page.save(os.path.join(out, 'crops.png'))

    frames = [ms for ms in range(0, 901, 50)]
    fs = [cell(f'film-{ms:03d}') for ms in frames]
    FW = MARGIN * 2 + 5 * (tw + GAP)
    FH = MARGIN * 2 + 84 + ((len(fs) + 4) // 5) * (th + 34)
    film = Image.new('RGB', (FW, FH), BG)
    draw = ImageDraw.Draw(film)
    draw.text((MARGIN, MARGIN), 'One swing at B2 (10°, κ 6), every 50 ms after the focus gain', TEXT, font(24, True))
    draw.text((MARGIN, MARGIN + 38), f"Look for: one swing out and back with a small overshoot, face-on again by 900 ms (the ring beam still runs to about 950 ms). The sheet's cells are the {peak_ms} ms peak.", DIM, font(17))
    for n, (ms, im) in enumerate(zip(frames, fs)):
        fx, fy = MARGIN + (n % 5) * (tw + GAP), MARGIN + 84 + (n // 5) * (th + 34)
        film.paste(im, (fx, fy))
        draw.text((fx, fy + th + 4), f'{ms} ms', DIM, font(15))
    film.save(os.path.join(out, 'filmstrip.png'))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
