---
id: material-ab6281
title: "GPU pixel-parity test: surface_remap.frag against RemapUniforms::apply"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-17T14:42:59Z
updated: 2026-10-17T14:42:59Z
depends: []
parent: material-55380c
tags: [rendering]
agent: claude-code/claude-opus-5-5
---

Final review of material-62347d (follow-up): the shader is checked to compile, and its CPU mirror is unit-tested, but nothing compares them. Render a known texture through SurfaceRemapRenderElement on the headless renderer, read it back, and compare with apply() within one level, for neutral and hued blends.
