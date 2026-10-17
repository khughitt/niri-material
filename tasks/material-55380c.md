---
id: material-55380c
title: Evaluate material cohesion on Noctalia surfaces
status: todo
priority: 2
size: m
complexity: high
process: planned
created: 2026-10-14T10:41:49Z
updated: 2026-10-17T02:03:08Z
depends: []
parent: material-3aa1f2
tags: []
source: docs/notes/2026-10-14-noctalia-material-brief.md
agent: codex
---

Make the bar and panels visually coherent with the window material. First milestone: identify one representative panel and the smallest comparison using existing layer effects before deciding whether a material render pass is needed.

Done: the owner judges that comparison, then the requested treatment is implemented through reviewed design and planning, or the idea is explicitly shelved with a wake condition. Where: docs/notes/2026-10-14-noctalia-material-brief.md, niri-config/src/layer_rule.rs and src/layer/mapped.rs.

## Notes

- 2026-10-15T22:55:08Z (docs/noctalia-brief-review): brief revised 2026-10-15: the panel comparison picks the optical treatment only; the owner's focused-hue requirement needs a compositor hue-delivery design whichever treatment wins (docs/notes/2026-10-14-noctalia-material-brief.md, Alternatives)
- 2026-10-16T00:01:54Z (research/material-e27dbc): next: owner judges the sheet attached to material-e27dbc (cells A1-B3: does B read as the same material as the glass?) and answers the brief's hue questions; then the planned design (optical treatment + hue delivery) starts from docs/notes/2026-10-14-noctalia-material-brief.md
- 2026-10-17T02:03:08Z (materials-26.04): owner verdict 2026-10-16: column C of the material-b156e7 sheet (matched hue and darkness: custom palette with glass-colored surfaces, bar and attached panels 0.95, floating panels solid) looks viable; the optical treatment is Noctalia styling, no layer material needed for cohesion. Consequence for hue delivery: at 0.95/solid a niri tint under the client shows through at most 5%, so Alternative A no longer carries the hue; the design must choose the hue route again.
