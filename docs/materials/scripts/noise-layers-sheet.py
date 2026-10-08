#!/usr/bin/env python3
"""Owner's contact sheets for the noise layers smoke (design §7.1).

Usage:
  noise-layers-sheet.py grid <run dir> <source rev> <out.png>
  noise-layers-sheet.py lattice <before run dir> <before rev> <after run dir> <after rev> <out.png>

`grid` lays out the smoke's sheet cells (scales, kinds, stacks, quadrature)
with today's single-layer grain bordered as the control. `lattice` sets two
runs' scaled cells side by side, the earlier run marked as the control. Each
tile is a 200x200 crop at 1:1 from the centre of a cell's `<name>-face.png`,
shown at 2x with nearest-neighbour resampling, so no smoothing hides a
lattice. Needs Pillow and the DejaVu fonts.
"""
import os
import sys

from PIL import Image, ImageDraw, ImageFont

CROP, ZOOM, GAP, MARGIN = 200, 2, 8, 12
TILE = CROP * ZOOM
GOLD, TEXT, DIM, BG = '#d4a62a', '#ffffff', '#cccccc', '#1e1e1e'
FONTS = '/usr/share/fonts/TTF'


def font(size, bold=False):
    return ImageFont.truetype(os.path.join(FONTS, 'DejaVuSans-Bold.ttf' if bold else 'DejaVuSans.ttf'), size)


def tile(run, name):
    face = Image.open(os.path.join(run, f'{name}-face.png')).convert('RGB')
    w, h = face.size
    if w < CROP or h < CROP:
        sys.exit(f'{name}-face.png is {w}x{h}, smaller than the {CROP} px crop')
    x, y = (w - CROP) // 2, (h - CROP) // 2
    return face.crop((x, y, x + CROP, y + CROP)).resize((TILE, TILE), Image.Resampling.NEAREST)


def render(title, notes, rows, out):
    """rows: [(heading, [(run, cell, caption, control)])]"""
    cols = max(len(cells) for _, cells in rows)
    width = MARGIN * 2 + cols * TILE + (cols - 1) * GAP
    head = 70 + 22 * len(notes)
    row_h = 34 + TILE + 52
    sheet = Image.new('RGB', (width, head + row_h * len(rows) + MARGIN), BG)
    draw = ImageDraw.Draw(sheet)
    draw.text((MARGIN + 8, 18), title, font=font(26, True), fill=TEXT)
    for i, line in enumerate(notes):
        draw.text((MARGIN + 8, 62 + 22 * i), line, font=font(16), fill=DIM)
    for r, (heading, cells) in enumerate(rows):
        y = head + r * row_h
        draw.text((MARGIN + 8, y), heading, font=font(18, True), fill=DIM)
        for c, (run, cell, caption, control) in enumerate(cells):
            x = MARGIN + c * (TILE + GAP)
            sheet.paste(tile(run, cell), (x, y + 30))
            if control:
                draw.rectangle((x - 2, y + 28, x + TILE + 1, y + 30 + TILE + 1), outline=GOLD, width=4)
            draw.multiline_text((x + TILE // 2, y + 36 + TILE), caption, font=font(16), fill=TEXT,
                                anchor='ma', align='center')
    sheet.save(out)


def grid(run, rev, out):
    def scales(kind):
        return [(run, f's{s}-{kind}', f'{kind}, scale {s}' + (f"\nCONTROL: today's {kind}" if s == 1 else ''), s == 1)
                for s in (1, 2, 4, 8)]
    render('Noise layers: does scaled and stacked glass grain look right?', [
        'Each tile: a 200x200 px crop at 1:1 from the centre of a glass window face, shown at 2x with no smoothing.',
        'The glass is optically neutral (ior 1, no tint, saturation 1) over a flat brown wallpaper, so only grain shows.',
        'Gold border = control: the grain niri renders today (one layer, scale 1). Scale = grain cell size in physical pixels.',
        f'Nested headless capture at {rev}, run {os.path.basename(os.path.normpath(run))}.',
    ], [
        ('Row 1: fine grain, amount 0.3, grain cell size (scale) 1 / 2 / 4 / 8 physical px', scales('fine')),
        ('Row 2: white grain, amount 0.3, scale 1 / 2 / 4 / 8', scales('white')),
        ('Row 3: lightness grain (amount 0.3) and two-layer stacks (each layer 0.3)', [
            (run, 'l1', "lightness, scale 1\nCONTROL: today's lightness", True),
            (run, 'l4', 'lightness, scale 4', False),
            (run, 'stack-white4-fine1', 'white scale 4\n+ fine scale 1 (both glass)', False),
            (run, 'stack-lightness-film', 'lightness on glass\n+ white on film', False),
        ]),
        ('Row 4: four fine layers at 0.15 against one at 0.3 (independent layers add in quadrature: 2 x 0.15 = 0.3)', [
            (run, 'four-fine-015', 'four fine layers\nat 0.15 each', False),
            (run, 'one-fine-03', 'one fine layer at 0.3\nCONTROL: today', True),
        ]),
    ], out)


def lattice(before, before_rev, after, after_rev, out):
    # Centre crops cover the same face region only when the faces match.
    for kind in ('white', 'fine'):
        for s in (2, 4, 8):
            sizes = {Image.open(os.path.join(run, f's{s}-{kind}-face.png')).size for run in (before, after)}
            if len(sizes) != 1:
                sys.exit(f's{s}-{kind}-face.png differs in size between the runs: {sorted(sizes)}')
    def row(run, kind, label, control):
        return [(run, f's{s}-{kind}', f'{label}{s}: {kind}, scale {s}', control) for s in (2, 4, 8)]
    render('Scaled grain: is the lattice still visible at scale 8?', [
        'Each tile: a 200x200 px crop at 1:1 from the centre of a glass window face, shown at 2x with no smoothing.',
        'Amount 0.3, neutral glass over a flat brown wallpaper. Scale = grain cell size in physical pixels.',
        f'Gold border = control: the Hermite lattice at {before_rev} ({os.path.basename(os.path.normpath(before))}),',
        f'judged blocky at scale 8. Unbordered: the cubic B-spline lattice at {after_rev} ({os.path.basename(os.path.normpath(after))}).',
        'Look for flat squares or a grid at the cell spacing in the scale-8 column. Name a tile by its id (e.g. B8).',
    ], [
        ('Row A: white grain, Hermite lattice (before)', row(before, 'white', 'A', True)),
        ('Row B: white grain, B-spline lattice (after)', row(after, 'white', 'B', False)),
        ('Row C: fine grain, Hermite lattice (before)', row(before, 'fine', 'C', True)),
        ('Row D: fine grain, B-spline lattice (after)', row(after, 'fine', 'D', False)),
    ], out)


if __name__ == '__main__':
    if len(sys.argv) == 5 and sys.argv[1] == 'grid':
        grid(*sys.argv[2:])
    elif len(sys.argv) == 7 and sys.argv[1] == 'lattice':
        lattice(*sys.argv[2:])
    else:
        sys.exit(__doc__)
