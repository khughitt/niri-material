---
id: material-98beac
title: "Draw border, focus ring and popups inside the material pass so they tilt with the pane"
status: idea
priority: 2
created: 2026-10-09T20:42:55Z
updated: 2026-10-09T20:42:59Z
depends: []
parent: material-53f873
tags: [rendering, camera]
source: docs/specs/2026-10-09-transient-rigid-tilt-design.md §9
agent: claude-code/claude-opus-5-5
---

From the transient rigid tilt design (material-abc08c, docs/specs/2026-10-09-transient-rigid-tilt-design.md §5): border, focus ring and popups are drawn outside the material pass, so a swing does not start while border or focus ring are on, and popups stay flat. Drawing them inside the material pass would let them tilt with the pane (the spike saw the border detach at 4°). Scope after the tilt lands and the owner has seen it.
