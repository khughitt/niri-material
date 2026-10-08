---
id: material-f386ae
title: Record GPU clocks in cost captures to explain the bimodal draw times
status: idea
priority: 2
created: 2026-10-08T09:13:39Z
updated: 2026-10-08T09:13:39Z
depends: []
tags: [performance, harness]
agent: claude-code/claude-opus-5-5
---

Every cost capture's material draws split into ~16 early draws at about 1/8 of the later ones (noise-layers evidence, Cost). A GPU clock change between the start-up burst and the 1 Hz cadence fits but was never measured. Sampling nvidia-smi pstate/clocks beside the Tracy capture would say which mode the reported medians are in.
