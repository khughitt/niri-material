---
id: material-3d48b0
title: Pilot nested measurements with the desktop idle against the pinned TTY reference
status: doing
priority: 1
size: s
complexity: mid
process: direct
needs: [quiet]
owner: materials-26.04
created: 2026-10-09T01:15:14Z
updated: 2026-10-16T11:49:37Z
started: 2026-10-11T12:04:57Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §8.2 pilot 2: reference runs, binary cb21ad03… and config hashes pinned there; I_c = [min-w-q, max+w+q], q = 0.001024 ms. Rehearse offline first to compare config hashes; on a mismatch take three fresh TTY reference runs first. Two desktop-idle runs (park --needs idle), the first is the pilot. Pass admits desktop-idle in §5.3 and maps nested measurements to idle in the queue.

## Notes

- 2026-10-09T01:32:32Z (capture-host-conditions): From the final review of material-18c2a1: await_load needs load1 < 1.0 (preflight allows 2.0) and runs before the hold, with the desktop up. The offline rehearsal and the pilot should show whether it passes on a desktop-idle host before preflight is reached; a timeout there is this check, not a refused preflight.
- 2026-10-11T12:04:57Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:94bf876f-10c5-43b5-9b79-bbaa31b7031d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-11T12:08:06Z (materials-26.04): Offline config rehearsal on 8a2d7e4f: write_cost_config regenerates none-b*.kdl aa09f751… and one-fine-1-b*.kdl bc488733…, equal to the pinned hashes and to all three reference runs; their niri-tracy-b is cb21ad03…. The pinned reference applies; no fresh TTY runs needed, so the pilot parks --needs idle.
- 2026-10-11T12:16:17Z (materials-26.04): Stubbed end-to-end rehearsal (CAPTURE_META=:, desktop in use, HEAD e91c92a0) of the exact pilot command: PASS in 7 min (builds 3, 8 sub-runs 4). Its b-side medians (none 0.0886/0.0983, one-fine-1 0.2473/0.1377 ms) are unsettled busy-host numbers far below the TTY reference, not evidence; they suggest GPU clock state dominates (cf. material-f386ae).
- 2026-10-11T12:16:17Z (materials-26.04): parked (waiting on user, quiet; idle, 25 min): Desktop-idle pilot run 1 (owner away, desktop up, nothing else working) from the main checkout: CAPTURE_TASK=material-3d48b0 OUT=$NIRI_MATERIAL_WORK_ROOT/noise-layers-ab-idle-pilot-$(date +%s) NOISE_LAYERS_COST_PILOT=0 NOISE_LAYERS_COST_AB=$NIRI_MATERIAL_WORK_ROOT/noise-layers-ab-hermite-rebuild-1791431057/niri-tracy-b docs/materials/scripts/noise-layers-cost.sh. Phases: build 3, await_load up to 5 (needs load1 < 1.0 with the desktop up; a timeout there is that check, not a refused preflight), preflight 1, 8 sub-runs 4 plus GPU cooldowns. Judge by spec §8.2: every settle settled, no disturbance, every renderer verified, provenance has b cb21ad03…, b config hashes aa09f751…/bc488733…, all four b medians in none [0.228352, 0.230400] and one-fine-1 [0.299008, 0.304128]. Write a run: note. If it passes, run 2 the same way (OUT …-idle-2-…) and judge it; on two passes admit desktop-idle in spec §5.3, map nested measurements to idle, record both run IDs in §8.2, done. On a failure record the reason and keep the route tty.
  provenance: {"harness_session":"claude-code:94bf876f-10c5-43b5-9b79-bbaa31b7031d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T11:49:37Z (materials-26.04): Not run in the 2026-10-16 quiet session: it needs the desktop up (idle) and the host was a bare TTY. Bearing from material-f386ae: noise-layers-cost medians are later-mode draws at the P8 idle clock (16 early draws at P0 1815 MHz, 0.041 ms; later at P8 210 MHz, 0.326 ms). A desktop that holds the GPU above P8 would move these medians for reasons unrelated to settles, so the pilot's interval check should record pstate/clocks (CAPTURE_GPU_CLOCKS=1) to tell the two apart.
