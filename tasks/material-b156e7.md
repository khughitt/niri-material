---
id: material-b156e7
title: Match the Noctalia bar and panel surfaces to the terminal glass's hue and darkness
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-17T01:24:01Z
updated: 2026-10-17T01:24:01Z
depends: []
parent: material-55380c
tags: []
source: "owner, 2026-10-16: hue and darkness are the main difference between the terminal and the Noctalia surfaces on the e27dbc sheet"
agent: claude-code/claude-opus-5-5
---

Why: on the material-e27dbc sheet (tasks/files/material-e27dbc/noctalia-cohesion-sheet.png) the owner judged hue and darkness the main difference between the terminal glass and the Noctalia bar, launcher and settings panels. The glass is a near-opaque teal-black slab (attenuation #0D1D1E over 31.2 px at distance 12); the shell paints the warm wallpaper-derived surface at 0.55-0.70 over blurred backdrop.
Done: a matched case in docs/materials/scripts/noctalia-surfaces.sh sets Noctalia's surface family (custom palette transcoded from the wallpaper palette, accents kept) and opacity so the bar and panels match the glass's hue and darkness; a three-column sheet (baseline, treated, matched) labelled so it is plain that no niri material is on the shell, attached here for the owner; the exact settings recorded here and in docs/notes/2026-10-14-noctalia-material-brief.md.
Where: docs/materials/scripts/noctalia-surfaces.sh; Noctalia v5.2.1 custom palettes (palettes/<name>.json, mSurface-style keys) and transparency settings; the glass in ~/.config/niri/prism.kdl.
