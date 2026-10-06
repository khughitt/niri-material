---
id: material-2592a2
title: "Ring beam: revisit ring-glow's default after burn-in"
status: idea
priority: 2
needs: [owner]
defer: 2026-10-05
created: 2026-09-21T09:39:49Z
updated: 2026-10-06T18:28:46Z
depends: []
parent: material-6062fd
tags: [rendering]
agent: claude-code/claude-opus-5
---

The head measured a 41/255 peak over a 0.6-opacity terminal and 10–14 under kitty's tab bar on the 2026-09-21 sheets (docs/materials/2026-09-19-ring-beam-evidence.md, 'What the frames show'). The owner accepted ring-glow 1 on the sheets; a fortnight of daily use is the real test. Revisit: keep 1, raise the default, or add a per-material override (Prism glass.ring.glow already carries it).

## Open questions

After daily use of the accepted ring-glow 1.2 profile and the shipped ring-rest control, should the native default remain 1 or become 1.2? Recommendation: retain default 1 and the profile override 1.2 unless the owner finds the stock setting consistently too dim. The 2026-10-01 profile choice does not establish a global default. ring-rest shipped in 7d240563 (material-1d70db); the accepted look is pinned in src/tests/ring_look.rs (material-519eeb). No new renderer knob is needed.

## Notes

- 2026-10-02T00:08:50Z (materials-26.04): Owner raised ring-glow to 1.2 (2026-10-01) alongside decay 4150 and gap 6, and asked for a glow on the moving beam that is separate from the resting ring: see the rest/beam split task
- 2026-10-06T18:28:46Z (materials-26.04): scope: question; collected the remaining owner-only default-gain decision; ring-rest already shipped; brief: docs/notes/2026-10-06-glass-optics-brief.md
