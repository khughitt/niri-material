---
id: material-a85a18
title: Revisit the filament shift cap versus light-ior for dense glass
status: idea
priority: 2
created: 2026-09-05T17:14:14Z
updated: 2026-09-05T17:48:20Z
depends: []
tags: []
---

Task 5 of material-26dd8a capped the filament's shared refracted shift at half ring-inset so the core stays in the bevel for any ior; once saturated, light-ior only widens the chromatic split. Decide whether the cap, a bevel-relative depth, or an auto light-ior is the intended model, with captures at ior 1.02, 1.24, 1.5. Scope includes whether light-ior should be removed or replaced, since the cap saturates on stock default glass and Prism's ior 1.24 at every value; take one capture at ior 1.24 / bevel 9 once the deterministic flex probe (material-22d78f) exists so the owner's own glass has evidence.
