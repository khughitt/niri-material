---
id: material-d21ff0
title: Prepare the covered sustained-optic capture in the hidden-window fixture
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-10-08T14:54:18Z
updated: 2026-10-08T14:54:18Z
depends: []
parent: material-5d6b2c
tags: [capture, performance]
agent: claude-code/claude-opus-5-5
---

Add an optic mode to docs/materials/scripts/hidden-window-attribution.sh per the audit's section 5 (docs/materials/2026-10-08-culling-damage-boundaries-audit.md): static kitty probe with aurora 0.5 { drift-hz 4; }, cases optic-visible, optic-covered-alpha, optic-covered-idle, optic-covered-opaque and optional glass-cover, verdict gates as tabulated, pilot = optic-visible + optic-covered-idle. Prove it with HWA_REHEARSAL=1 on the desktop; no evidence run here.
