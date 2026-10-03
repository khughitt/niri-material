---
id: material-fd40f3
title: Glass edge test and script cleanups
status: todo
priority: 4
size: xs
complexity: low
created: 2026-10-03T11:31:46Z
updated: 2026-10-03T11:31:46Z
depends: [material-be611b]
tags: [material]
agent: claude-code
---

Minor items from the material-be611b reviews: glass-edge-compare.py exits 1 on bad input though its docstring says 2, and pair without --region/--expect errors confusingly; glass_edge.rs writes the renderer string with {:?} instead of JSON escaping; edge_highlight's uniforms test uses 0.5 for both amount and alpha (a swap would pass; use roughness 0 or 0.5); rename ggxPeakRatio with the optic's prefix (and its pin); edge_highlight.frag could return early on the face (s.across <= 0) as spec §5 says; one_core's error may name a pixel one count above the true minimum; glass-edge-sheet.sh builds the unused Tracy binary.
