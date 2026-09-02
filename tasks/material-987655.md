---
id: material-987655
title: Inactive desaturation post-process
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-02T12:09:35Z
depends: [material-a54d89]
tags: [signals, rendering]
---

Outcome: decide and, if accepted, implement a desaturation of the window texture, not the glass, for windows at Quiet or inactive, so focus reads even when the material is quiet. Must compose with the existing background-effect postprocess (see material-cad932). Acceptance: control surface, compositing order, and cost recorded before code. Source: docs/materials/2026-09-02-material-signals-design.md section 11.
