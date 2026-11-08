---
id: material-0df22c
title: "Design the material cost capture matrix and whole-preset estimator, and prepare the run"
status: todo
priority: 2
size: m
complexity: high
process: planned
created: 2026-10-15T18:40:54Z
updated: 2026-10-15T18:40:54Z
depends: []
parent: material-31074f
tags: [performance]
agent: claude-code/claude-opus-5-5
---

Why: material-31074f's measurement needs a written design before any quiet run. Nothing yet defines how a whole preset's cost is estimated. The only guidance is negative: costs are not summed per effect (docs/notes/2026-10-02-workstreams-brief.md:26), and the shared blur/prefilter caches and docs/materials/performance.md §2's invalidation rules apply.

Where to start: the resource-aware and adaptive-materials briefs (docs/notes/2026-09-29-*), performance.md, the capture protocol (docs/specs/2026-09-11-material-capture-protocol-design.md; its §8, :471-474, leaves whole-run drift bounding to this study), host conditions (docs/specs/2026-10-08-capture-host-conditions-design.md §5.3), and niri-config/src/material/pipeline.rs as the stage and parameter skeleton (its only cost field is the Cached/Fragment class on sites).

The design must settle:
- the estimator model: cached per-output work (chain, prefilter, grain), once per output and target, against per-fragment work scaled by glass area and redraw cadence; the focus split duplicating pyramids; measured interaction terms; and the estimate's inputs and uncertainty for the consumer (prism-d54be4, shelved until an estimate exists with hardware/workload provenance, uncertainty and focused/unfocused coverage);
- the capture matrix: stages, owned parameters and ranges, the interaction pairs (frosted backdrop with high distortion detail, aberration with fractional roughness) and focus-split scenes, plus the scene factors the schema omits (transparency, area, bevel band, output size, noise slots, render targets);
- the repeat and noise-floor protocol: same-session A/B (cross-night comparison hid a 15 % shift, noise layers), building on view-tilt-cost.sh's a/b repeat bound (unmerged branch material-abc08c; run by material-2283b8) rather than starting fresh; coordinate with material-4394f3's A/B guard.

Preparation, no host: a generic cost script over glass-optic-smoke-lib.sh, GPU spans for the sharp backdrop render (effect_buffer.rs ~:540) and the window offscreen (offscreen.rs ~:78), which have CPU spans only today, and the pilot definition. Also correct performance.md §6 and §8 (:217-219), which still call off-view culling unbuilt though it landed (material-7afc31).

Done: a reviewed spec and plan, the script and spans merged, and the pilot defined so the quiet run is mechanical.
