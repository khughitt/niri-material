---
id: material-cd0e1d
title: "Identity, visibility and cost smoke; review clips"
status: done
priority: 2
complexity: low
process: direct
owner: material-77db8a
created: 2026-09-23T00:39:48Z
updated: 2026-09-24T20:25:17Z
started: 2026-09-23T09:46:08Z
completed: 2026-09-24T20:25:17Z
depends: []
parent: material-77db8a
tags: [camera]
agent: "claude-code/claude-opus-5-5[1m]"
plan: docs/plans/2026-09-22-focus-view-tilt.md
step: "Task 6: Identity, visibility and cost smoke; review clips"
---

## Notes

- 2026-09-23T09:46:08Z (material-77db8a): started
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T10:15:20Z (material-77db8a): parked (waiting on user, quiet; idle, 40 min): Rerun glass-view-tilt-smoke.sh (OUT=view-tilt-smoke-fc326c1b, BASE_NIRI=view-tilt-base-niri) then ring-motion-clips.sh view sequences once capture-meta preflight passes, then commit per brief Step 6
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T10:17:22Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T11:18:06Z (material-77db8a): parked (waiting on user, quiet; idle, 40 min): Rerun glass-view-tilt-smoke.sh then ring-motion-clips.sh view sequences when the desktop GPU is idle (preflight refuses on gpu_util_pct > 5), then commit per brief Step 6
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T11:22:14Z (material-77db8a): parked (waiting on user, quiet; idle, 40 min): Live runs from .worktrees/material-77db8a on an idle desktop GPU: (1) OUT=$NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-$(git rev-parse --short HEAD) CAPTURE_TASK=material-77db8a BASE_NIRI=$NIRI_MATERIAL_WORK_ROOT/view-tilt-base-niri docs/materials/scripts/glass-view-tilt-smoke.sh, expect renderer_verified_launches=15 and base_vs_plain_ae=0; (2) CAPTURE_TASK=material-77db8a SEQUENCES='view-swing-t10-p1 view-swing-t10-p3 view-swing-t25-p1 view-swing-t25-p3 view-swing-beam view-persp-k25-p1 view-persp-k25-p3 view-persp-k50-p1 view-persp-k50-p3' docs/materials/scripts/ring-motion-clips.sh, expect clips: OK; (3) commit with tasks done material-cd0e1d per the plan's Step 6 protocol
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T07:59:25Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T09:20:14Z (material-77db8a): Run 2026-09-24 (monitors off via niri power-off-monitors): headless preflight passed on the second try (first refused 04:00 on GPU P5/P8 mix and power IQR 1.012 W from desktop redraws; retained as view-tilt-smoke-1a5190f3.refused-*). Timing: preflight 04:08:42-04:09:02, build to 04:12:44 (~3.7 min), launches 04:12:44-04:19:58 at ~48 s each, 9 of 15 done. Launch 9's tracy-csvexport --gpu on gpu-deep-3.tracy looped at 100% CPU until killed at 05:17 (~57 min lost). Projected smoke wall-clock: ~17 min; clips not reached.
- 2026-09-24T09:20:14Z (material-77db8a): tracy-csvexport hang reproduces: gpu-deep-3.tracy times out every time (60 s retry, empty output), gpu-deep-2.tracy exports in 0.6 s. Not the 0 ns timer resolution in its capture log (six earlier 0 ns traces exported fine). Trace kept at NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-1a5190f3/ for a Tracy 0.13.1 bug report. glass-optic-smoke-lib.sh now bounds every export at 120 s and fails, so a recurrence costs one launch, not the night; a retry of the affected launch is the next step if it recurs. A monitor wake at ~05:08 (lock screen) came after the hang and did not cause it.
- 2026-09-24T09:20:43Z (material-77db8a): parked (waiting on user, quiet; idle, 45 min): Tonight, from .worktrees/material-77db8a with monitors off (niri msg action power-off-monitors after a 15 s delay; the desktop's redraws alone fail the GPU P8/IQR gate): move the aborted NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-1a5190f3 aside (keep it: gpu-deep-3.tracy reproduces the csvexport hang), then (1) OUT=$NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-$(git rev-parse --short HEAD) CAPTURE_TASK=material-77db8a BASE_NIRI=$NIRI_MATERIAL_WORK_ROOT/view-tilt-base-niri docs/materials/scripts/glass-view-tilt-smoke.sh, expect renderer_verified_launches=15 and base_vs_plain_ae=0 (~17 min); (2) the ring-motion-clips.sh SEQUENCES run from the previous park, expect clips: OK; (3) record actual wall-clock per step in a note; commit with tasks done material-cd0e1d per the plan's Step 6 protocol. If an export times out again, add a one-retry of that launch before rerunning.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T19:54:26Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T20:09:28Z (material-77db8a): Rerun 2026-09-24 15:54: build 2x ~1.6 min; gpu-deep-3 exported fine this time; failed at 16:08:00 in gpu-persp-3 with 'expected exactly one probe window' (probe kitty never mapped in ~60 s). Only external event: wali-rotate.timer rotated the wallpaper at 16:06:58 (Prism apply, host niri reload, glass back on via profile glass4) as the probe spawned. Host changes for the quiet window: wali-rotate.timer stopped (restore: systemctl --user start wali-rotate.timer) and scratch glass.enabled=false (restore: prism unset glass.enabled before restarting the timer).
- 2026-09-24T20:25:17Z (material-77db8a): Actual wall-clock 2026-09-24 (rotation paused, monitors off): smoke 16:09:53-16:19:20 (9 min 27 s, incl. preflight and two cached release builds), clips 16:19:20-16:24:42 (5 min 22 s); total ~15 min against the 40-45 min park estimate. Attempts before it: ~70 min lost to a tracy-csvexport hang (05:17 kill) and ~13 min to the wallpaper-rotation probe failure.
- 2026-09-24T20:25:17Z (material-77db8a): done
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T20:25:17Z (material-77db8a): Smoke PASS: renderer_verified_launches=15, base_vs_plain_ae=0, neutral_vs_plain_ae=0; persp chamfer RMSE 27085, parallax face RMSE 40091.6; llvmpipe GPU plain 3.204 ms, persp +3.0%, deep +5.0%. Clips OK with view-swing and view-persp grids (ring-motion-clips-20461987/ring-clips-389037-1790281160). Evidence under NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-20461987.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
