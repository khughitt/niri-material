---
id: material-6901e0
title: "View tilt: a geometric tilt that moves window content and glass together"
status: idea
priority: 1
created: 2026-09-22T23:46:32Z
updated: 2026-10-06T22:10:40Z
depends: [material-77db8a]
parent: material-53f873
tags: [rendering, camera]
agent: "claude-code/claude-opus-5-5[1m]"
---

The optical-only view tilt (material-77db8a, spec docs/specs/2026-09-22-focus-view-tilt-design.md) was rejected in review on 2026-10-06: the terminal text stays static while the glass optics swing, which breaks the goal of one cohesive material. Any tilt or perspective must move the window's content and the glass as one body: the slab, its optics and the window pixels share one transform.

Costs to weigh when scoping: text softening while the pane moves (resampling under a projective transform), element geometry, damage and hit-testing during the swing, and whether the static by-position perspective survives at all once content must follow it.

Reusable from the unmerged branch material-77db8a: the swing curve and settle (render_helpers/material/view.rs), the Layout's focus origin (layout/focus_origin.rs), the swing run on the tile, the config keys, and the smoke and clip scripts. The shader's view-ray routing may still serve as the optical half of a geometric tilt.

## Notes

- 2026-10-06T21:03:54Z (materials-26.04): scope: briefed; P1 under lane material-53f873; rigid in-shader tilt is the lean (window and glass as one body, backdrop at true screen position); spike material-89fb6b (owner stills) then design material-abc08c; brief: docs/notes/2026-10-06-transient-depth-brief.md
- 2026-10-06T22:10:40Z (materials-26.04): finding (material-89fb6b, 2026-10-06): a planar in-shader tilt reads to the owner as one object and its text softening is acceptable; side walls are missing first, interior detail second. Next: material-abc08c, building side walls on material-be611b's height-field edge.
