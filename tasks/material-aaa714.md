---
id: material-aaa714
title: Isolate clipping and rounding in dark backdrop grain controls
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-06T20:44:05Z
updated: 2026-10-06T20:44:05Z
depends: []
parent: material-7ff4bc
tags: [harness, rendering]
source: docs/notes/2026-10-06-render-anomalies-brief.md
agent: codex
---

Question: Does pre-blur clipping in the backdrop grain pass account for the observed positive blue lift, or does quantization in the blur/prefilter chain contribute independently?

Where to start: docs/notes/2026-10-06-render-anomalies-brief.md; docs/materials/2026-10-05-noise-placement-evidence.md (Nested smoke); docs/specs/2026-10-05-noise-placement-design.md section 4; src/render_helpers/shaders/material/noise.frag::noise_source; src/render_helpers/grain.rs; src/render_helpers/effect_buffer.rs; src/render_helpers/blur.rs; src/tests/noise_site.rs and its existing frozen-clock helpers; docs/materials/scripts/glass-noise-site-smoke.sh::signed_diff (site minus zero).

Bound: One deterministic flat-field study using the existing in-process GLES fixture, plus an offline signed-grain reference that isolates clipping versus quantization. Pin fine grain amount 0.3 and seed/geometry; compare matched zero/backdrop/glass controls on a dark flat blue channel, an interior midtone and a near-upper-bound channel with blur off/on and roughness 0/1. Report raw grained-source and final face deltas where the existing readback permits. Vary clipping and quantization independently in the reference; note unsupported raw readback or renderer differences rather than assume equivalence. Keep control positions, alpha, color encoding and neutral optics fixed. No production format change, extra noise layers, new capture framework, live desktop takeover or GPU-cost/power claim.

Expected result: Record signed per-channel means, spread, code-value distributions, repeats and resolution limits, reference assumptions and which cause is supported or remains unknown. Use raw code values as well as grey spread so a DC/channel bias is not called residual grain. Define a check that distinguishes a supported mechanism from blur rounding. Report a recommendation (retain documented 8-bit behavior or scope a format/clamp design) with unknown visual acceptance and memory/driver-support costs explicit; no wider-format implementation is authorized by the measurement. Reuse material-674d4e for owner judgement of the backdrop site's look. Run just test-one -p niri <focused-filter> then just test-fast for fixture changes, and update this task and the brief. This is a pixel/control study, not a quiet-host benchmark.

Ideas it wakes: On completion, run tasks note on material-d1171f with the findings in the same commit as this result.
