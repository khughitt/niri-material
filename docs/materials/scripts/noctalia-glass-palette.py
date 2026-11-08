#!/usr/bin/env python3
"""Build a Noctalia custom palette whose surfaces match the terminal glass.

Reads the dark token map `noctalia theme <image> --dark -o` writes and keeps
its accents, text and terminal colors. The surface family Noctalia's shell
paints (surface, surface_container for cards, outline_variant for outlines)
moves to the glass's hue and darkness in OKLCH: `surface` becomes the target
color and the other two keep their lightness and chroma steps above it.

Usage: noctalia-glass-palette.py <tokens.json> <#rrggbb target> <out.json>
"""

import json
import math
import sys

# Noctalia's mapping from generated tokens to the 16 fixed roles
# (mapGeneratedPaletteMode, Noctalia v5.2.1 src/theme).
ROLES = {
    "mPrimary": "primary",
    "mOnPrimary": "on_primary",
    "mSecondary": "secondary",
    "mOnSecondary": "on_secondary",
    "mTertiary": "tertiary",
    "mOnTertiary": "on_tertiary",
    "mError": "error",
    "mOnError": "on_error",
    "mSurface": "surface",
    "mOnSurface": "on_surface",
    "mSurfaceVariant": "surface_container",
    "mOnSurfaceVariant": "on_surface_variant",
    "mOutline": "outline_variant",
    "mShadow": "shadow",
    "mHover": "hover",
    "mOnHover": "on_hover",
}
SURFACE_FAMILY = ("mSurface", "mSurfaceVariant", "mOutline")
ANSI = ("black", "red", "green", "yellow", "blue", "magenta", "cyan", "white")


def hex_to_rgb(h):
    h = h.lstrip("#")
    return [int(h[i : i + 2], 16) / 255 for i in (0, 2, 4)]


def rgb_to_hex(rgb):
    return "#" + "".join(f"{round(min(max(c, 0.0), 1.0) * 255):02x}" for c in rgb)


def to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def from_linear(c):
    return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055


def to_oklch(rgb):
    r, g, b = (to_linear(c) for c in rgb)
    l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b
    m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b
    s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b
    l, m, s = (math.copysign(abs(x) ** (1 / 3), x) for x in (l, m, s))
    L = 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s
    a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s
    b2 = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s
    return L, math.hypot(a, b2), math.atan2(b2, a)


def from_oklch(L, C, h):
    a, b = C * math.cos(h), C * math.sin(h)
    l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (L - 0.0894841775 * a - 1.2914855480 * b) ** 3
    r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s
    g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s
    b3 = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    return [from_linear(c) for c in (r, g, b3)]


def main():
    tokens_path, target, out_path = sys.argv[1:]
    with open(tokens_path) as f:
        root = json.load(f)
    tokens = root.get("dark", root)

    palette = {role: tokens[token] for role, token in ROLES.items()}
    base_L, base_C, _ = to_oklch(hex_to_rgb(palette["mSurface"]))
    t_L, t_C, t_h = to_oklch(hex_to_rgb(target))
    for role in SURFACE_FAMILY:
        L, C, _ = to_oklch(hex_to_rgb(palette[role]))
        palette[role] = rgb_to_hex(from_oklch(t_L + (L - base_L), max(t_C + (C - base_C), 0.0), t_h))

    palette["terminal"] = {
        "background": tokens["terminal_background"],
        "foreground": tokens["terminal_foreground"],
        "cursor": tokens["terminal_cursor"],
        "cursorText": tokens["terminal_cursor_text"],
        "selectionBg": tokens["terminal_selection_bg"],
        "selectionFg": tokens["terminal_selection_fg"],
        "normal": {c: tokens[f"terminal_normal_{c}"] for c in ANSI},
        "bright": {c: tokens[f"terminal_bright_{c}"] for c in ANSI},
    }
    with open(out_path, "w") as f:
        json.dump({"dark": palette}, f, indent=2)
    for role in SURFACE_FAMILY:
        print(f"{role} {tokens[ROLES[role]]} -> {palette[role]}")


if __name__ == "__main__":
    main()
