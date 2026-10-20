---
id: material-d1171f
title: Does backdrop grain clipping lift dark texels under blur?
status: idea
priority: 2
reviewed: true
created: 2026-10-06T08:56:17Z
updated: 2026-10-20T21:44:02Z
depends: [material-aaa714]
parent: material-7ff4bc
tags: [material, noise]
agent: claude-code/claude-opus-5-5
---

The 2026-10-05 roughness cell showed backdrop grain under a blurred roughness-1 pyramid raising blue by one code on 55% of face pixels (mean +0.55), on a backdrop whose plasma has 24% of blue texels within the grain's ±38-code span of 0. If the grained 8-bit source clips at 0, blur averages a one-signed bias into dark regions: a slight lift the glass site does not have. Check with a dark flat backdrop at backdrop vs glass, blurred, and decide whether the grain pass should run in a wider format or accept it.

## Current scope (2026-10-20)

Current noise_source still clamps before the Abgr8888 target; the lightness path has an additional gamut clamp. Noise-layer accumulation has since landed, so controls must use one layer and pin kind/seed. Reuse material-aaa714 to separate clipping from blur quantization with deterministic signed dark/midtone/bright controls. A wider target alone cannot remove the shader clamp; no format change is justified yet.

Read [ref:brief] for the existing alternatives, constraints and bounded handoff.

## References

- brief (input, required, kind=brief): docs/notes/2026-10-06-render-anomalies-brief.md — existing scope handoff and follow-up boundaries

## Notes

- 2026-10-06T20:44:03Z (materials-26.04): scope: briefed; measured blue lift is one-signed but clipping and 8-bit blur rounding are not isolated; use deterministic flat-field controls before changing format or accepting the bias; brief: docs/notes/2026-10-06-render-anomalies-brief.md
