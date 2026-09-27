---
id: material-5f9dee
title: Measure isolated idle power and publish the material budget
status: doing
priority: 2
size: m
owner: material-265eb0
created: 2026-09-11T10:28:52Z
updated: 2026-09-27T06:30:48Z
started: 2026-09-26T03:15:03Z
depends: [material-4241c3]
parent: material-265eb0
tags: [performance]
spec: docs/specs/2026-09-11-material-idle-budget-design.md
plan: docs/plans/2026-09-11-material-idle-budget.md
step: "Task 3: Measure isolated power and publish the budget verdict"
---

After user resume and trace validation, use an operator-provided dedicated DRM session with complete GPU-client visibility and no other clients. Run uninstrumented shamA/A plus A/B,B/C,B/D repeated comparisons:48 windows,60s warmup+30s observation,1Hz,about72min. Apply proposed1W resolution target and conservative repeat floor, report idle increment and active cost bounds, archive evidence, and close parent only with valid conclusions. Do not log out user, switch VT, stop services, change clocks, or substitute shared-desktop power measurements.

## Notes

- 2026-09-11T10:30:13Z (material-265eb0): parked (waiting on user): Prepared only; await user resume, trace validation, and operator-arranged isolated DRM session before power collection.
- 2026-09-11T10:57:07Z (material-265eb0): Preparation review: report sham_floor_w/sham_precision_ok, abba_floor_w, raw_repeat_floor_w, and combined floor_w/precision_ok. The conservative gate intentionally rejects raw repeat drift even when ABBA cancels it; require positive compositor/kitty PID visibility. No power run authorized yet.
- 2026-09-26T03:15:03Z (material-265eb0): started
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-26T03:19:19Z (material-265eb0): Prepared 2026-09-25 23:15-23:25 in the TTY session: power binary built by build-power from 191ad747 (NIRI_MATERIAL_WORK_ROOT/material-265eb0/power-target/release/niri, sha256 282fdb10..., no features, 2 min 46 s). Output DP-1 only connected: 3440x1440, preferred 59.999 Hz, scale 1. Blocker found before any run: the collector's GPU-client inventory runs 'sudo -n fuser -v <devices>' and sudo needs a password here, so every window (and the power preflight) would fail 'device-user visibility unavailable'. Exact device list: /dev/dri/card1 /dev/dri/renderD128 /dev/nvidia-modeset /dev/nvidia-uvm /dev/nvidia-uvm-tools /dev/nvidia0 /dev/nvidiactl. Pilot runner written (scratch power-pilot.sh: sham block 1 only, 4 windows, ~7 min, per-window power_observation) before the ~80 min full run.
- 2026-09-26T03:19:19Z (material-265eb0): parked (waiting on user, approval): User: add a sudoers NOPASSWD rule for the exact fuser inventory command and approve the DRM run (niri takes over DP-1 at 3440x1440@59.999 from this TTY for ~7 min pilot then ~80 min; the fixture's cleanup returns the VT on exit). Then agent: run the sham-block pilot, read its 4 windows, and only if all pass run the full power lane and analyze.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T06:30:24Z (material-265eb0): 2026-09-27: user added the sudoers rule; verified 'sudo -n /usr/bin/fuser -v <the 7 devices>' runs without a password (rc 0, full device-user listing; same device list). Remaining gate: a TTY session with the desktop stopped (this check ran from the Wayland desktop: niri, noctalia, firefox, kitty on the GPU, P3 33 W).
- 2026-09-27T06:30:48Z (material-265eb0): parked (waiting on user, quiet; headless, 90 min): User: start a TTY session with the desktop stopped and resume this session there. Then agent: run the sham-block power pilot (scratch power-pilot.sh, ~7 min, DRM takeover of DP-1 3440x1440@59.999), read its 4 windows, and only if all pass run the full power lane (~80 min) and analyze.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
