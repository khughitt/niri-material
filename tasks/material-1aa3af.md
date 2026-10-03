---
id: material-1aa3af
title: "Surface grain on the glass edge: noise that survives dark glass"
status: idea
priority: 2
created: 2026-10-03T00:52:50Z
updated: 2026-10-03T00:52:50Z
depends: []
parent: material-6062fd
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
---

Owner priority (2026-10-01, on material-be611b): edges read flat and static, with no glass noise on the chamfer. Today noise grains only the transmitted backdrop (render-pipeline stage 3b), so dark tinted glass attenuates the grain away on the bevel; material-be611b's height-field bevel lets more transmitted grain through where the glass thins, and its reflection and edge-highlight carry the light, but nothing grains the surface light itself. Candidate: a surface-microtexture term on the bevel that modulates the reflected and specular light (or a fine normal perturbation scaled by u), screen- or element-seeded, with its own neutral. Builds on be611b's Surface hook and u coordinate; scope after the be611b contact sheet shows how much grain the thinner rim already carries.
