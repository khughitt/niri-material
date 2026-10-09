---
id: material-a8232f
title: "noise-layers smoke: re-pin the baseline to the noise merge's first parent"
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: noise-baseline-repin
created: 2026-10-09T04:34:05Z
updated: 2026-10-09T10:51:18Z
started: 2026-10-09T04:34:25Z
completed: 2026-10-09T10:51:18Z
depends: []
tags: [capture, noise]
source: material-52dc6e
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-06-noise-layers-design.md
---

glass-noise-layers-smoke.sh asserts whole-frame byte identity against a baseline built from 4a8b2072 (spec §7.2 item 2, 'the task-start commit'). Since merge 1ae20983 the branch also carries glass-edges (material-be611b), whose bevel change is intended and changes edge pixels by one code value: the material-52dc6e pilot (noise-layers-pixels-pilot-1791520292) failed 'fine glass vs baseline' with AE 8.3, face ROI identical, every differing pixel within 16 px of the window edge or outside it. The merged-branch equivalent of the task-start commit is the merge's first parent 37de154e (glass-edges included). Build that baseline (release, in a detached worktree), point the smoke's BASE_NIRI comment and --config baseline= at it, amend spec §7.2 and the evidence doc's reproduction, and rerun the pilot (material-52dc6e, owner go-ahead). If an identity still fails against 37de154e, the later merged commits (off-view cull 797ab6ae, overview view d9e43727) are the next suspects.

## Notes

- 2026-10-09T04:34:25Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T04:34:56Z (noise-baseline-repin): resumed
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T04:43:59Z (noise-baseline-repin): parked (waiting on user, approval): On the owner's go-ahead (host use): from .worktrees/noise-baseline-repin run CAPTURE_TASK=material-52dc6e NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material OUT=/mnt/ssd3/niri-material/noise-layers-pixels-pilot-$(date +%s) BASE_NIRI=/mnt/ssd3/niri-material/noise-baseline-37de154e/niri NOISE_LAYERS_PILOT=1 docs/materials/scripts/glass-noise-layers-smoke.sh (binaries prebuilt; ~1 min). Pass: commit the uncommitted re-pin (script + spec 7.2), evidence-doc note, done a8232f and 52dc6e, merge. Fail: suspect 797ab6ae / d9e43727.
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T10:47:59Z (noise-baseline-repin): resumed
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T10:51:18Z (noise-baseline-repin): done
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T10:51:18Z (noise-baseline-repin): Smoke and spec §7.2 pin the 37de154e baseline (noise merge's first parent, glass edges included); pixels pilot passes every identity against it; evidence doc records both pilots
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
