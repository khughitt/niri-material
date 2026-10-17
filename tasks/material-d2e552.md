---
id: material-d2e552
title: Select later-mode draws by clock state in cost reports; tidy the clock sampler
status: doing
priority: 2
size: s
complexity: low
process: direct
owner: material-d2e552
created: 2026-10-16T11:56:58Z
updated: 2026-10-17T13:01:24Z
started: 2026-10-17T12:32:29Z
depends: []
tags: [rendering]
agent: claude-code/claude-opus-5-5
---

From material-f386ae's verdict and its review (2026-10-16): cost medians should come from draws whose joined clock row is P8 at the idle clock, not from every in-window draw (P0 start-up burst included). Decide whether cost runs sample clocks by default (CAPTURE_GPU_CLOCKS) so the selection is always possible.

Review minors carried here: add an in_window column to <case>.draw-clocks.csv (noise-layers-cost.sh report); test clock sampling combined with NOISE_LAYERS_COST_AB (per-sub-run draw-clocks files) and the case-argument refusals (tools/test_gpu_clocks.py NoiseCostPilotTests); capture_wait's unguarded kill of the sampler under set -e, and report Tracy's failure before or with the sampler's; record the case selection in capture_identity (--config cases=...); derive the case-name validation list from the CASES table and correct the header ('picks one case for any run, refused with A/B'); capture_identity claims gpu-clock-sampling for fixtures that never reach capture_bg (widen material-93ef9e or record it from capture_bg); the report should require the clock files when CAPTURE_GPU_CLOCKS=1 instead of testing for their presence.

## Notes

- 2026-10-16T12:02:11Z (materials-26.04): From the f386ae re-review: nvidia-smi refreshes clocks/pstate/power on a 0.5 s grid though polled at 100 ms, so a joined row can lag a draw by up to 0.5 s; a strict P8/210 filter would drop draws that ran at the idle clock (the 225-tagged ones cost the 210 rate). Select with that lag allowed for.
- 2026-10-17T12:29:24Z (materials-26.04): Since material-2e4ea5 (merged 2026-10-17), every settle entry records gpu_power_step_w and its gpu_power_w, so cost medians can also be stratified by idle floor (10.4 vs 17 W), which the gates design §8 left out of scope. Consider it alongside the clock-state selection.
- 2026-10-17T12:32:29Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:05694925-34c9-4a16-ae62-2de8550ed7aa","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T12:32:49Z (material-d2e552): resumed
  provenance: {"harness_session":"claude-code:05694925-34c9-4a16-ae62-2de8550ed7aa","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T12:46:53Z (material-d2e552): Decision: noise-layers-cost samples clocks by default (f386ae's unsampled control matched the steady median); draw_median_ms keeps every in-window draw so 3d48b0's pinned interval stands, and its spec §8.2 recipe now names CAPTURE_GPU_CLOCKS=0. Idle-clock rule: P8 at the trace's lowest P8 clock from the joined reading through the first reading 0.5 s past the draw. Floor stratification: idle_clock_power_median_w per case. Minor 'identity claims sampling for fixtures that never reach capture_bg' is already material-93ef9e's scope (optic-settling, the only independent Tracy launcher besides material-signals, which has no capture_identity).
- 2026-10-17T12:59:09Z (material-d2e552): review: impl round 1 — verdict: revise; findings: Important 1, Minor 4; reviewer: claude-code/claude-opus-5-5
- 2026-10-17T13:01:24Z (material-d2e552): Round 1 ruling: Important (3d48b0 recipe runs sampled against §8.2) accepted — keep CAPTURE_GPU_CLOCKS=0 for the pilot to match its unsampled references; a held clock is diagnosed by a sampled rerun after a failed interval check, not by the pilot; rejected the alternative of counting an idle-draw refusal as the verdict (mixes conditions). Park text amended from the main checkout after merge. Minors 1-4 fixed in place (empty case name, discriminating delta test plus mixed-clock refusal test, mutation-checked; comment; doc refusals).
