---
id: material-3fcba2
title: "Glass noise layers: N stacked grain generators with gain, type, and seed scale each"
status: todo
priority: 2
size: m
created: 2026-09-09T03:03:30Z
updated: 2026-09-09T03:03:30Z
depends: []
tags: [material, noise]
---

Prism goal prism-85f63a wants several noise devices stacked on one material. Extend the glass noise node to a small fixed number of layers (four, after the impulse fan-out precedent): packed vec4 uniforms for gain, type, and scale, an unrolled constant-bound loop in material.frag, and a per-layer seed scale or offset so identical types do not coincide. Single-node configs must render byte-identical. Record the cost of stacked fine layers (nine hashes per fragment per layer).
