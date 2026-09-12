---
id: material-31074f
title: Explicit performance estimation and evaluation per material element
status: todo
priority: 2
size: m
created: 2026-09-11T23:34:15Z
updated: 2026-09-11T23:34:15Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Using the capture protocol, measure the cost of each render pass and each exposed parameter across its range, including the main interactions (e.g. frosted backdrop with high distortion detail) and the focused/unfocused split. Produce an estimate that can evaluate a whole preset's cost. This is the measurement side; prism-d54be4 is the consumer that wants these estimates to shape exposed ranges and warn on demanding settings in the Noctalia panel.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821
