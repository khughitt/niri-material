---
id: material-55380c
title: Evaluate material cohesion on Noctalia surfaces
status: todo
priority: 2
size: m
complexity: high
process: planned
created: 2026-10-14T10:41:49Z
updated: 2026-10-15T22:55:08Z
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
