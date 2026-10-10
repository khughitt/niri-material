#!/usr/bin/env python3
"""The owner's still sheet for the rigid tilt (design 2026-10-09 §8.3).

Usage: view-tilt-sheet.py <VIEW_TILT_DUMP dir> <out dir>

Reads <dump>/sheet/*.png and geometry.json from `still_sheet_frames` and
writes sheet.png (the 3x3 grid, dark and light in each cell, the control,
the spill row), crops.png (1:1 crops of the advancing edge with a 4x zoom of
it, of whole text lines, and of a corner on a diagonal swing) and
filmstrip.png (one swing at the middle cell). Needs Pillow and the DejaVu fonts.
"""
import json
import os
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFont, ImageStat

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
    pad = 80
    W0, H0 = geo['size']

    def clamp(rect):
        x0, y0, x1, y1 = rect
        return (max(0, int(x0)), max(0, int(y0)), min(W0, int(x1)), min(H0, int(y1)))

    box = clamp((x - pad, y - pad, x + w + pad, y + h + pad))

    def cell(name):
        im = Image.open(os.path.join(d, f'{name}.png')).convert('RGB').crop(box)
        return im.resize((int(im.width * SCALE), int(im.height * SCALE)), Image.Resampling.LANCZOS)

    tw, th = cell('rest-dark').size
    pair = 2 * tw + GAP
    head, side = 170, 210
    rows = len(TS) + 1
    W = MARGIN * 2 + side + len(KS) * (pair + GAP)
    H = MARGIN * 2 + head + rows * (th + 46 + GAP) + 60
    sheet = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(sheet)
    draw.text((MARGIN, MARGIN), 'How far, and how much perspective, for the focus tilt?', TEXT, font(30, True))
    draw.text((MARGIN, MARGIN + 44), 'Each cell: the peak of one focus-gain swing (171 ms of 900 ms), the accepted dark look (left) and a lighter look (right).', DIM, font(17))
    lifted = 'left' if geo['advancing'][0] < 0 else 'right'
    draw.text((MARGIN, MARGIN + 70), f'Look for depth you can read at the peak: the rounded rim turning toward you, the wall on the advancing ({lifted}) edge, the keystone.', DIM, font(17))
    draw.text((MARGIN, MARGIN + 96), "Wall: the slab side below the rim. Rim: the rounded bevel. Keystone: the near edge drawn taller than the far one. Wall spill: the focus beam's light on the wall.", DIM, font(17))
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
    ry = MARGIN + head + len(TS) * (th + 46 + GAP)
    draw.text((MARGIN, ry + th // 2 - 24), 'B2, wall spill', TEXT, font(21, True))
    draw.text((MARGIN, ry + th // 2 + 4), 'off | on', DIM, font(17))
    draw.text((MARGIN, ry + th // 2 + 28), 'close up: crops I, J', DIM, font(17))
    for j, (name, label) in enumerate([('spill-off-dark', 'S1  wall spill off'), ('cell-t10-k6-dark', 'S2  wall spill on (as B2)')]):
        cx = MARGIN + side + j * (tw + GAP)
        sheet.paste(cell(name), (cx, ry))
        draw.text((cx, ry + th + 6), label, DIM, font(16))
    cx = MARGIN + side + 2 * (pair + GAP)
    sheet.paste(cell('control-dark'), (cx, ry))
    draw.rectangle((cx - 3, ry - 3, cx + tw + 2, ry + th + 2), outline=GOLD, width=3)
    draw.text((cx, ry + th + 6), 'K  control: the spike, 4° with the camera at 2000 px', GOLD, font(16, True))
    draw.text((MARGIN, H - MARGIN - 26), 'Reply with a cell id (A1–C3), the spill (S1 or S2), and anything that reads wrong.', TEXT, font(19, True))
    sheet.save(os.path.join(out, 'sheet.png'))

    # The positive lobe lifts the edge along the swing's direction: the advancing edge. The
    # crops stay on the canvas, so a pane near the output's edge is cut there, not padded.
    left = geo['advancing'][0] < 0
    if left:
        edge = (x - 100, y + h / 2 - 160, x + 220, y + h / 2 + 160)
        zoom = (x - 30, y + h / 2 - 40, x + 50, y + h / 2 + 40)
    else:
        edge = (x + w - 220, y + h / 2 - 160, x + w + 100, y + h / 2 + 160)
        zoom = (x + w - 50, y + h / 2 - 40, x + w + 30, y + h / 2 + 40)
    # The diagonal crop's pose lifts the corner along (0.6, 0.8): bottom right.
    corner = (x + w - 200, y + h - 200, x + w + 100, y + h + 100)

    def frame(name):
        return Image.open(os.path.join(d, f'{name}.png')).convert('RGB')

    def crop(im, rect):
        return im.crop(clamp(rect))

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

    side_name = 'left' if left else 'right'
    peak_d, rest_d = frame('cell-t10-k6-dark'), frame('rest-dark')
    # The spill changes only the wall under the beam: crop where S1 and S2 differ.
    spill_off = frame('spill-off-dark')
    spill_diff = ImageChops.difference(spill_off, peak_d)
    bbox = spill_diff.getbbox()
    if bbox is None:
        sys.exit('spill-off-dark and cell-t10-k6-dark are identical: the spill hook did nothing')
    spill = (bbox[0] - 30, bbox[1] - 30, bbox[2] + 30, bbox[3] + 30)
    spill_max = max(hi for _, hi in spill_diff.getextrema())
    mean = lambda im: sum(ImageStat.Stat(im.crop(bbox)).mean)
    spill_way = 'brighter' if mean(peak_d) > mean(spill_off) else 'darker'
    text = text_rect([peak_d, rest_d], y + 16 + 20 * 2, 6)
    big = lambda im: im.resize((im.width * 4, im.height * 4), Image.Resampling.NEAREST)
    rows = [
        [(f'A  advancing ({side_name}) edge, B2 dark', crop(peak_d, edge)),
         (f'B  advancing ({side_name}) edge, B2 light', crop(frame('cell-t10-k6-light'), edge))],
        [('C  A at 4x: the wall is the grey strip on the slab\'s outer side', big(crop(peak_d, zoom))),
         ('D  the same edge at 4x, at rest: no wall', big(crop(rest_d, zoom)))],
        [(f'E  whole text lines, B2 dark at the peak (the {side_name} end is lifted)', crop(peak_d, text))],
        [('F  the same lines at rest', crop(rest_d, text))],
        [('G  bottom-right corner, swing toward the bottom right, dark', crop(frame('diagonal-dark'), corner)),
         ('H  the same, light', crop(frame('diagonal-light'), corner))],
        [('I  wall spill off (S1) at 4x, where the beam crosses the wall', big(crop(spill_off, spill))),
         (f'J  wall spill on (S2) at 4x: {spill_way} on average, up to {spill_max} levels apart, in a {bbox[2] - bbox[0]} x {bbox[3] - bbox[1]} px patch', big(crop(peak_d, spill)))],
    ]
    label_font = font(15)

    def slot(label, im):
        return max(im.width, int(label_font.getlength(label)))

    CW = MARGIN * 2 + max(sum(slot(*t) for t in row) + GAP * (len(row) - 1) for row in rows)
    CH = MARGIN * 2 + 84 + sum(max(im.height for _, im in row) + 40 for row in rows)
    page = Image.new('RGB', (CW, CH), BG)
    draw = ImageDraw.Draw(page)
    draw.text((MARGIN, MARGIN), 'Native-resolution crops (1:1) at the peak, B2 (10°, κ 6), unless marked 4x', TEXT, font(24, True))
    draw.text((MARGIN, MARGIN + 38), "Look for: the wall on the lifted edge (A–D), text that stays crisp and straight under the tilt (E against F), the corner wall (G, H), the beam's light on the wall (I against J).", DIM, font(17))
    yy = MARGIN + 84
    for row in rows:
        xx = MARGIN
        for label, im in row:
            page.paste(im, (xx, yy))
            draw.text((xx, yy + im.height + 6), label, DIM, label_font)
            xx += slot(label, im) + GAP
        yy += max(im.height for _, im in row) + 40
    page.save(os.path.join(out, 'crops.png'))

    frames = [ms for ms in range(0, 901, 50)]
    fs = [cell(f'film-{ms:03d}') for ms in frames]
    FW = MARGIN * 2 + 5 * (tw + GAP)
    FH = MARGIN * 2 + 84 + ((len(fs) + 4) // 5) * (th + 34)
    film = Image.new('RGB', (FW, FH), BG)
    draw = ImageDraw.Draw(film)
    draw.text((MARGIN, MARGIN), 'One swing at B2 (10°, κ 6), every 50 ms after the focus gain', TEXT, font(24, True))
    draw.text((MARGIN, MARGIN + 38), "Look for: one swing out and back with a small overshoot, face-on again by 900 ms (the ring beam still runs to about 950 ms). The sheet's cells are the 171 ms peak.", DIM, font(17))
    for n, (ms, im) in enumerate(zip(frames, fs)):
        fx, fy = MARGIN + (n % 5) * (tw + GAP), MARGIN + 84 + (n // 5) * (th + 34)
        film.paste(im, (fx, fy))
        draw.text((fx, fy + th + 4), f'{ms} ms', DIM, font(15))
    film.save(os.path.join(out, 'filmstrip.png'))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
