---
id: material-36e968
title: Motion-stimulus sweep for jelly-flex and jelly-ripple
status: done
priority: 1
size: m
owner: material-36e968
created: 2026-09-06T09:48:24Z
updated: 2026-09-11T10:06:15Z
depends: [material-37cec9]
parent: material-53f873
tags: [tooling, harness, dynamics]
---

material-37cec9's sweep captures a settled window with animations off, where jelly activity is 0 (material.rs:216 derives activity from motion residuals) and the shader gates ripple behind mat_jelly_activity > 0 (material.frag:398). Flex and ripple therefore render identically at every value, so the static sweep excludes them. Measuring them needs a reproducible motion stimulus (scripted resize or move via niri msg), a defined settle delay, and a capture phase pinned relative to the impulse - the timing determinism is its own verification problem. Note prism-5758d3 records that jelly-flex's range is blocked on material-6d4de5 deciding the renderer cap anyway.

## Notes

- 2026-09-11T00:52:15Z (materials-26.04): Promoted to the dynamics sprint's first child (material-53f873): the sweep is the only way to see whether jostle, ease-to-rest, or idle micro-movement changed anything. Do this before the rendering changes so each is judged by measurement, not eye. From mindful:thought:3f94e656b70f4e5585c1cb60c166e4da.
- 2026-09-11T09:41:21Z (material-36e968): Context checked at native2afd0fb2 and experiments results/slice3: v1-parity-replay.sh already captures timed column-move bursts; v1-parity-analyze.mjs exports motionMetrics/summarizeMotion/pairedMotionEvidence and aligns by measured pane progress. Ripple uses absolute adjusted clock, so equal requested delays do not imply equal pixels. Proposed bounded sweep: reuse existing motion metrics, one parameter at a time, paired repeats, coverage/monotonic-progress/settled-return integrity checks, and retain fixtures/results in niri-experiments. No renderer changes proposed; awaiting design approval per brainstorming skill.
- 2026-09-11T09:41:21Z (material-36e968): parked (waiting on user): Approve bounded motion sweep: scripted column moves with fixed spring and settle delay, paired bursts aligned by measured pane progress, repeat-noise and exact settled-return checks; reuse experiments motion analyzer and native host helper, preserve fixture/results in experiments. Then implement and run sweep.
- 2026-09-11T09:57:00Z (material-36e968): Approved bounded design implemented in experiments worktree .worktrees/material-36e968: native host helper plus archived v1 motionMetrics/summarizeMotion. Installed7526af1d binary pinned by SHA. Two-value flex preflight passes: 13 captures per burst, monotonic measured travel about489px to0, exact settled return, flex0.02 signal0.222707 vs repeat floor0.086335. Review tightened sparse-progress rejection and includes each setting repeat range in noise floor. Full flex/ripple sweep collecting.
- 2026-09-11T10:06:15Z (material-36e968): Added motion sweep in niri-experiments results/jelly-motion f1b0741, reusing native host helper and archived motion analyzer. Installed7526af1d RTX3070: eight cases,208 burst frames, every sampled flex/ripple neighbor step resolves above repeat variation; all16 bursts return to identical settled pixels. Coverage, progress, repeat-noise and planned-case tests prevent false passes. Evidence docs/materials/2026-09-11-jelly-motion-sweep.md; native dynamics follow-up material-6d4de5 updated; no renderer or Prism range changes.
