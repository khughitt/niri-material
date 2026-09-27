---
id: material-5f9dee
title: Measure isolated idle power and publish the material budget
status: done
priority: 2
size: m
owner: material-265eb0
created: 2026-09-11T10:28:52Z
updated: 2026-09-27T08:53:57Z
started: 2026-09-26T03:15:03Z
completed: 2026-09-27T08:53:57Z
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
- 2026-09-27T06:46:33Z (material-265eb0): 2026-09-27 TTY window (VT3, desktop stopped, P8 10.2 W, no GPU users): wali-rotate.timer stopped for the runs (restore: systemctl --user start wali-rotate.timer). Starting the sham-block pilot.
- 2026-09-27T07:14:05Z (material-265eb0): 2026-09-27 pilots on DRM (TTY VT3). Pilot 1 (02:46) failed window 1 in 46 s: 'GPU client contamination' - with a DRM session, root systemd (pid 1) and systemd-logind hold /dev/dri/card1 (logind opens the KMS node for the compositor), which the inventory counted as clients; fix experiments 73fbbe0 exempts only root systemd/systemd-logind rows on a card node that nvidia-smi does not list. Pilot 2 (02:50) failed at once: captured fuser -v puts pids on stdout and the rest of each row on stderr; fix 7953684 pairs them in order. Pilot 3 (02:52-03:00): sham-1-1..4 PASS, medians 11.17/11.06/11.035/11.08 W, P8 210 MHz throughout, 27-28 C. Full run 1 (03:00) passed sham-1-1/1-2 (11.14/11.15 W), then the settle before sham-1-3 refused on P5/P8: a DRM niri's exit can hold the GPU in P5 ~2 s (probe: 1 of 3 exits), and the settle started 1.6 s after exit; fix 127cf6b waits for 3 s of consecutive P8 (bounded 30 s) before each power settle, gate unchanged. Full run 2 started 03:13 (power-full-20260927T031335).
- 2026-09-27T08:53:57Z (material-265eb0): done
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T08:53:57Z (material-265eb0): Task 3 complete: isolated DRM power run 2026-09-27 03:13-04:47 (93 min; power-full-20260927T031335), 48/48 windows valid (settle 48/48, full GPU-client inventory every sample, P8 210 MHz throughout). Sham floor 0.105 W; A->B +0.015 W, floor 0.1175 W, upper 0.13 W, not resolved (budget_passed); B->C +0.845 W (upper 1.11), B->D +0.68 W (upper 0.98). Independent raw recomputation matches analysis.json to 1e-9. Pilots found and fixed three power-lane fixture defects (experiments 73fbbe0, 7953684, 127cf6b). Evidence: docs/materials/2026-09-11-idle-budget-evidence.md; experiments results/idle-budget 647f3c8 (pushed); wali-rotate.timer restored.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
