#!/usr/bin/env python3
"""Compare glass edge harness dumps (src/tests/glass_edge.rs) across commits.

    glass-edge-compare.py identical A B CASE... [--region R]
    glass-edge-compare.py pair DIR CASE_A CASE_B --region R --expect same|differ
    glass-edge-compare.py predict-face BEFORE AFTER CASE --ior N
    glass-edge-compare.py predict-within BEFORE AFTER OFF ON --ior N
    glass-edge-compare.py report A B CASE [--region R]

`identical` compares one case across two dump dirs; `pair` compares two cases
in one dir. Regions come from <case>.json: `all`; `window`, inside the window;
`face`, the face rectangle inset by 2 px (the ring band included); `outside`,
farther than 2 px outside the slab rectangle; `bevel`, inside the slab and
outside the face. Values are 8-bit sRGB; predictions run in linear light.

predict-face (spec §6, ring and aurora off): AFTER = (1 - f0) * (BEFORE - S) + S
with S = 0.15 * f0, the face's constant glint. predict-within (ring and aurora
on): AFTER_ON = (1 - f0) * (BEFORE_OFF - S) + S + (BEFORE_ON - BEFORE_OFF).
"Within one code value" is |after - round(prediction)| <= 1: the 8-bit inputs
alone put up to about 1.02 code values between the exact prediction and a
correct render. Exit 1 on a failed check, 2 on bad input. Standard library only.
"""
import json
import math
import pathlib
import sys


def load(directory, case):
    base = pathlib.Path(directory) / case
    meta = json.loads(base.with_suffix(".json").read_text())
    pixels = base.with_suffix(".rgba").read_bytes()
    width, height = meta["size"]
    if len(pixels) != width * height * 4:
        sys.exit(f"{base}.rgba: {len(pixels)} bytes for {width}x{height}")
    return meta, pixels


def inside(rect, px, py, pad=0.0):
    x, y, w, h = rect
    cx, cy = px + 0.5, py + 0.5
    return x - pad <= cx < x + w + pad and y - pad <= cy < y + h + pad


def region(meta, name):
    width, height = meta["size"]
    for py in range(height):
        for px in range(width):
            if name == "all":
                hit = True
            elif name == "window":
                hit = inside(meta["window"], px, py)
            elif name == "face":
                hit = inside(meta["face"], px, py, -2.0)
            elif name == "outside":
                hit = not inside(meta["slab"], px, py, 2.0)
            elif name == "bevel":
                hit = inside(meta["slab"], px, py) and not inside(meta["face"], px, py)
            else:
                sys.exit(f"unknown region {name}")
            if hit:
                yield px, py


def rgb(meta, pixels, px, py):
    i = (py * meta["size"][0] + px) * 4
    return pixels[i:i + 3]


def lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def enc(c):
    c = max(c, 0.0)
    c = c * 12.92 if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
    return c * 255


def f0_of(ior):
    return ((ior - 1) / (ior + 1)) ** 2


def option(args, flag, default=None):
    if flag in args:
        at = args.index(flag)
        return args[at + 1], args[:at] + args[at + 2:]
    return default, args


def check(failures, label, got, want):
    worst = max(abs(g - round(w)) for g, w in zip(got, want))
    if worst > 1:
        failures.append(f"{label}: got {tuple(got)}, predicted {tuple(round(v, 2) for v in want)}")
    return worst


def differing(meta, pa, pb, name):
    return sum(1 for px, py in region(meta, name) if rgb(meta, pa, px, py) != rgb(meta, pb, px, py))


def main(argv):
    if len(argv) < 2:
        sys.exit(__doc__)
    cmd, args = argv[1], argv[2:]
    failures = []
    if cmd == "identical":
        name, args = option(args, "--region", "all")
        a, b, cases = args[0], args[1], args[2:]
        for case in cases:
            meta, pa = load(a, case)
            _, pb = load(b, case)
            diff = differing(meta, pa, pb, name)
            print(f"{case} {name}: {diff} differing pixels")
            if diff:
                failures.append(f"{case} {name}: {diff} pixels differ")
    elif cmd == "pair":
        name, args = option(args, "--region")
        expect, args = option(args, "--expect")
        directory, case_a, case_b = args
        meta, pa = load(directory, case_a)
        _, pb = load(directory, case_b)
        diff = differing(meta, pa, pb, name)
        print(f"{case_a} vs {case_b} {name}: {diff} differing pixels (expect {expect})")
        if (diff == 0) != (expect == "same"):
            failures.append(f"{case_a} vs {case_b} {name}: {diff} differing pixels, expected {expect}")
    elif cmd == "predict-face":
        ior, args = option(args, "--ior")
        before, after, case = args
        f0 = f0_of(float(ior))
        s = 0.15 * f0
        meta, pb = load(before, case)
        _, pa = load(after, case)
        worst = 0
        for px, py in region(meta, "face"):
            want = [enc((1 - f0) * (lin(c) - s) + s) for c in rgb(meta, pb, px, py)]
            worst = max(worst, check(failures, f"{case} ({px},{py})", rgb(meta, pa, px, py), want))
        print(f"{case} face, f0 {f0:.5f}: worst {worst} code values from the rounded prediction")
    elif cmd == "predict-within":
        ior, args = option(args, "--ior")
        before, after, off, on = args
        f0 = f0_of(float(ior))
        s = 0.15 * f0
        meta, b_off = load(before, off)
        _, b_on = load(before, on)
        _, a_on = load(after, on)
        worst = 0
        for px, py in region(meta, "face"):
            t_s = [lin(c) for c in rgb(meta, b_off, px, py)]
            w = [lin(c1) - c0 for c1, c0 in zip(rgb(meta, b_on, px, py), t_s)]
            want = [enc((1 - f0) * (c - s) + s + d) for c, d in zip(t_s, w)]
            worst = max(worst, check(failures, f"{on} ({px},{py})", rgb(meta, a_on, px, py), want))
        print(f"{on} face with ring and aurora, f0 {f0:.5f}: worst {worst} code values from the rounded prediction")
    elif cmd == "report":
        name, args = option(args, "--region", "bevel")
        a, b, case = args
        meta, pa = load(a, case)
        _, pb = load(b, case)
        total, count, worst = 0.0, 0, 0
        for px, py in region(meta, name):
            for ca, cb in zip(rgb(meta, pa, px, py), rgb(meta, pb, px, py)):
                total += (ca - cb) ** 2
                count += 1
                worst = max(worst, abs(ca - cb))
        print(f"{case} {name}: rmse {math.sqrt(total / count):.3f}, max {worst} code values")
    else:
        sys.exit(__doc__)
    for line in failures[:20]:
        print("FAIL", line)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
