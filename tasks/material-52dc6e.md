---
id: material-52dc6e
title: Pilot the pixels lane on a desktop in use with glass-noise-layers-smoke.sh
status: done
priority: 2
size: xs
complexity: low
process: direct
needs: [nested, owner]
owner: materials-26.04
created: 2026-10-09T01:15:14Z
updated: 2026-10-09T10:51:19Z
started: 2026-10-09T04:25:14Z
completed: 2026-10-09T10:51:18Z
depends: [material-a8232f]
parent: material-2834d7
tags: [capture]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §8.2 pilot 1. Run glass-noise-layers-smoke.sh on --lane pixels on the desktop in use, with the owner's go-ahead for host use. Pass: every AE-0 assertion holds; the record has no baseline or GPU sampling, renderers recorded, timers held and restored; metrics.txt equals a retained run with identical provenance.binaries if one exists (else say so in the run note).

## Notes

- 2026-10-09T04:25:14Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T04:34:05Z (materials-26.04): run: 7 min (est 5, nested, owner); build 6, smoke 0.5; failed: fine glass vs baseline AE 8.3 at HEAD 8e0f06c2 (noise-layers-pixels-pilot-1791520292). Lane mechanics all held: preflight unsampled host desktop, no GPU section, 13 timers held and restored (scan clean), renderer recorded on 4/4 launches, determinism AE 0 x3. The diff is +-1 code value, face ROI identical, all pixels within 16 px of the window edge or outside it: glass-edges' bevel (material-be611b, intended per its spec 'bevel differences are reported, not asserted') landed beside noise layers and the smoke's 4a8b2072 whole-frame identity was never rerun after merge 1ae20983. No retained run has these binaries.
- 2026-10-09T04:34:16Z (materials-26.04): parked (waiting on agent, dependency): After material-a8232f re-pins the baseline: ask the owner, then rerun CAPTURE_TASK=material-52dc6e OUT=$NIRI_MATERIAL_WORK_ROOT/noise-layers-pixels-pilot-$(date +%s) BASE_NIRI=<37de154e release niri> NOISE_LAYERS_PILOT=1 docs/materials/scripts/glass-noise-layers-smoke.sh (build 6 if stale, smoke ~1); first attempt failed only on the stale 4a8b2072 identity
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T10:51:18Z (noise-baseline-repin): run: 9 min (est 7, nested, owner); build 6, smoke 2.6; passed: noise-layers-pixels-pilot-1791542879 against the 37de154e baseline (material-a8232f). Every AE-0 assertion holds; preflight unsampled host desktop, no GPU section; 13 timers held, restore complete, scan clean; renderer recorded 23/23 launches; metrics.txt equals noise-layers-bspline-pilot-1791425925 (e5edd631 code, headless lane) line for line, though binaries differ (merged branch), so no identical-provenance run exists.
- 2026-10-09T10:51:18Z (noise-baseline-repin): done
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T10:51:18Z (noise-baseline-repin): Pixels lane piloted on the desktop in use: passed at the second attempt after the stale baseline re-pin (material-a8232f); lane mechanics held on both attempts
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
