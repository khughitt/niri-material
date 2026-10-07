---
id: material-2f2af4
title: Noise layers cost captures with Tracy
status: done
priority: 2
complexity: mid
process: direct
needs: [quiet]
owner: material-3fcba2
created: 2026-10-06T10:17:17Z
updated: 2026-10-07T03:14:27Z
started: 2026-10-06T11:21:34Z
completed: 2026-10-07T03:14:27Z
depends: []
parent: material-3fcba2
tags: []
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-06-noise-layers.md
step: "Task 5: Cost captures with Tracy"
---

## Notes

- 2026-10-06T11:21:34Z (material-3fcba2): started
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:23:32Z (material-3fcba2): Cost script and shared wallpaper cleanup prepared; focused cleanup/report tests 4 passed, full tooling 409 passed with 2 optional retained-binary skips. Captures not attempted: the smoke preflight measured GPU 34% at P5 plus CPU/load over limits; no environment change or retry.
- 2026-10-06T11:23:32Z (material-3fcba2): parked (waiting on user, quiet; headless, 18 min): Agent: run noise-layers-cost.sh pilot (~8 min: build 4, six cases 4), then full (~10 min: cases 10), then add the Cost section to the smoke evidence document; smoke preflight refused CPU/load/GPU P5, no cost cases attempted
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:24:53Z (material-3fcba2): Validation detail: full tooling ran 409 cases, 407 passed and 2 optional retained-binary checks skipped
- 2026-10-07T02:24:47Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T02:31:05Z (material-3fcba2): run: 5.9 min (est 8, headless); build 0.1, cases 5.8; passed: six cases at 3 steps, material draw medians none 0.0302 / one-fine-1 0.0410 / four-fine-1 0.0614 / four-fine-8 0.0788 ms; run dir noise-layers-cost-pilot-1791339892
- 2026-10-07T02:40:28Z (material-3fcba2): run: 8.6 min (est 10, headless); build 2.7, cases 5.9; passed, but provenance mixed: noise.frag was edited (lightness identity fast path, uncommitted) during the run's preflight, so this run measured that shader; medians none 0.230 / one-fine-1 0.301 / four-fine-1 0.505 / four-fine-8 0.652 ms; draw times bimodal (~30 us first frames, ~230 us after), the pilot's 3 steps saw only the fast mode; rerun once the shader settles; run dir noise-layers-cost-full-1791340265
- 2026-10-07T03:12:02Z (material-3fcba2): run: 5.9 min (est 10, headless); build 0.1, cases 5.8; passed at 558bfa02 (dirty only in task records): material draw medians none 0.229 / one-fine-1 0.301 / four-fine-1 0.504 / four-fine-8 0.653 ms, backdrop Grain::render 0.088 (fine-1) / 0.091 (fine-8); within 1% of the previous full run; clean release; run dir noise-layers-cost-full2-1791342346
- 2026-10-07T03:14:27Z (material-3fcba2): done
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:14:27Z (material-3fcba2): Tracy cost captures at 558bfa02: material draw 0.229 (none) / 0.301 (one fine) / 0.504 (four fine) / 0.653 ms (four fine at scale 8), backdrop grain 0.088-0.091 ms; Cost section in the evidence document, including the bimodal draw-time finding
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
