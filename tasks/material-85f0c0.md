---
id: material-85f0c0
title: "signals smoke: anchor idle-resume's resumed window to the input, not the trace end"
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-21T09:01:15Z
updated: 2026-09-27T12:16:53Z
started: 2026-09-27T12:15:19Z
completed: 2026-09-27T12:16:53Z
depends: []
tags: [tooling, signals]
agent: claude-code/claude-opus-5
---

idle_case in docs/materials/scripts/material-signals-smoke.sh counts Niri::redraw in [end-18 s, end-13 s) while the pointer move fires 12 s from the capture's start. Trace length varies by about a second between runs (35.44 s vs 36.43 s on 2026-09-21 and 2026-09-20), so the fixed-from-end window slides over the burst: identical 136-redraw bursts counted 99 (FAIL, bound 100-166) and 115. Anchor the window to the first redraw after the input (or to 12 s from the first zone), or count the burst itself: first-to-last redraw after input, with the quiet check unchanged. Evidence: material-signals-35e736b6/signals-3013733-1789980505/idle-resume.csv and the 0219599f run's, histogrammed per second.

## Notes

- 2026-09-21T09:01:22Z (material-2c3984): filed from the ring-beam evidence run (material-912dab); not a beam regression, so it stays outside the material-2c3984 goal
- 2026-09-27T12:15:19Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:5c58ac0f-7e10-4813-ae4c-48834db98357","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T12:16:53Z (material-85f0c0): done
  provenance: {"harness_session":"claude-code:5c58ac0f-7e10-4813-ae4c-48834db98357","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T12:16:53Z (material-85f0c0): idle_case counts the final 20 s as one burst (burst_in_steady): >=1 s quiet lead before the first redraw, 5 +/- 0.5 s first-to-last span, count vs pulse rate at 25%; frozen check unchanged. Replayed on the 35e736b6 (was 99, FAIL) and 0219599f (was 115) traces: both now 136 over 5.00 s; demand-pulse's ungated trace fails the lead check
  provenance: {"harness_session":"claude-code:5c58ac0f-7e10-4813-ae4c-48834db98357","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
