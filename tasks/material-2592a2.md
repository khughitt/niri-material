---
id: material-2592a2
title: "Ring beam: revisit ring-glow's default after burn-in"
status: idea
priority: 2
needs: [owner]
defer: 2026-10-05
created: 2026-09-21T09:39:49Z
updated: 2026-10-03T16:48:43Z
depends: []
parent: material-6062fd
tags: [rendering]
agent: claude-code/claude-opus-5
---

The head measured a 41/255 peak over a 0.6-opacity terminal and 10–14 under kitty's tab bar on the 2026-09-21 sheets (docs/materials/2026-09-19-ring-beam-evidence.md, 'What the frames show'). The owner accepted ring-glow 1 on the sheets; a fortnight of daily use is the real test. Revisit: keep 1, raise the default, or add a per-material override (Prism glass.ring.glow already carries it).

## Notes

- 2026-10-02T00:08:50Z (materials-26.04): Owner raised ring-glow to 1.2 (2026-10-01) alongside decay 4150 and gap 6, and asked for a glow on the moving beam that is separate from the resting ring: see the rest/beam split task
