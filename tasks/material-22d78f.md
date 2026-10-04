---
id: material-22d78f
title: Deterministic mid-flex probe for filament face confinement
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: mid-flex-check
created: 2026-09-05T16:12:46Z
updated: 2026-10-04T14:37:30Z
started: 2026-10-04T14:23:20Z
completed: 2026-10-04T14:37:29Z
depends: []
parent: material-49871a
tags: [harness]
---

resize-flex in focus-ring-light.sh compares two nested hosts mid-resize; independent clocks skew 2-5 px so the 2-level face bound cannot be measured. Needs a way to capture both filament-on and filament-off at identical animation progress (clock stepping or a single-host toggle).

## Notes

- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; independent-host resize skew remains; old zero-face-light requirement was superseded by the within ring; investigate matched-state capture using existing frozen-clock fixtures; brief: docs/notes/2026-09-29-glass-measurement-brief.md
- 2026-10-01T09:55:04Z (material-0e80c1): material-0e80c1 result: a matched pair needs no production change. src/tests/ring_pair.rs renders ring on/off at a frozen 500 ms instant of a linear resize, byte-reproducibly (repeat 0 px; a 10 ms shifted control changes 22-24k px). Under flex the band centroid moves inward on the resizing axis only, +0.44 px at jelly-flex 0.0066 and +0.96 px at 0.02. Opaque client: 0 px lit inside the window. The face-bound premise is obsolete (light over a translucent face scales with 1 - client alpha). Remaining: an owner-set tolerance for the proposed centroid motion check; case_resize_flex's two-host comparison can give way to this fixture. Brief: docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring-findings
- 2026-10-01T10:00:23Z (materials-26.04): scope: todo, direct. The probe exists (src/tests/ring_pair.rs, material-0e80c1). Remaining: (1) turn its flex report into a check: per edge, the band centroid's mid-flex shift should equal the face edge displacement predicted from jelly_state's resize term and slabSurface's inner_half scaling; the bound comes from that computed prediction plus the flex-0 residual (0.01 px), not from an invented tolerance. (2) Retire focus-ring-light.sh::case_resize_flex's two-host comparison in favour of the test. Mid-resize ring glow is brighter by design (main.frag:166, glow * (1 + 2 * jelly activity)), so pairs must share an instant.
- 2026-10-04T14:23:20Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a1074a-246e-72e0-8c01-2d366891620c","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T14:23:49Z (mid-flex-check): resumed
  provenance: {"harness_session":"codex:01a1074a-246e-72e0-8c01-2d366891620c","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T14:28:35Z (mid-flex-check): measurement: the discrete half-maximum centroid jumps when a sample enters the run (stock flex 0.0066: measured 0.44 px vs independently predicted 0.3144 px). Use a same-instant flex-zero ring-gap control at the predicted face displacement to calibrate pixel sampling, with the existing 0.01 px residual; pin gap 8 for this motion check so the core clears the refracted chamfer. Keep cap-binding coverage in ring_cap_keeps_one_core. No production clock or shader change.
- 2026-10-04T14:34:14Z (mid-flex-check): verification: focused ring_pair module 9/9 passed; Rust test-fast 468 passed, 1 skipped. Removing mat_jelly_resize from slabSurface made ring_tracks_face_during_resize fail at stock flex 0.0066 left (20.02297 vs control20.46285); restored shader and confirmed no production diff. Python resize-flex delegation failed before routing change and passes afterward, including failure propagation and absent capture/runtime setup.
- 2026-10-04T14:35:25Z (mid-flex-check): review: impl round 1 — verdict: accept; findings: none; reviewer: codex/gpt-6
- 2026-10-04T14:35:25Z (mid-flex-check): verification: real CASES=resize-flex script entry passed with nonexistent capture binary and Prism state; no nested host needed. Independent review accepted the assertion, control and routing with no findings.
- 2026-10-04T14:37:29Z (mid-flex-check): verification: Python tooling test-fast completed: 287 tests, 2 skipped, exit0. Rust test-fast completed: 468 passed, 1 skipped. Real resize-flex wrapper passed without capture binary or Prism state. Feedback ops-7933db records temporary-backup cleanup approval friction; Python restoration was used for the mutation check.
- 2026-10-04T14:37:29Z (mid-flex-check): done
  provenance: {"harness_session":"codex:01a1074a-246e-72e0-8c01-2d366891620c","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T14:37:29Z (mid-flex-check): Added deterministic per-edge ring/face tracking assertions at frozen mid-resize instants, calibrated sampled centroids against independently predicted ring-gap controls, replaced the two-host resize-flex comparison with the focused test, and documented coverage and limits.
  provenance: {"harness_session":"codex:01a1074a-246e-72e0-8c01-2d366891620c","harness_session_source":"CODEX_SESSION_ID"}
