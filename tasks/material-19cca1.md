---
id: material-19cca1
title: "Ring beam: fade overlap when a lap is shorter than BEAM_FADE"
status: doing
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-21T09:39:49Z
updated: 2026-09-28T11:05:34Z
started: 2026-09-28T11:05:34Z
depends: []
tags: [rendering]
agent: claude-code/claude-opus-5
---

In src/render_helpers/material/ring.rs the envelope's fade (BEAM_FADE 300 ms) starts at the end of the lap and the head/tail terms assume the lap outlasts it. A face whose perimeter at ring-beam-speed takes under 300 ms (a small window at a high speed, e.g. P 240 px at 1200 px/s) fades before the head has cleared, so the comet dims mid-run. Harmless at the shipped 300 px/s on ordinary windows. Fix: clamp the fade to the lap (or start it at min(lap, lap − fade)), mirror in main.frag, add a ring.rs case at P < speed × BEAM_FADE. Deferred minor from the 2026-09-19 ring-beam final review.

## Notes

- 2026-09-22T15:34:01Z (materials-26.04): Related: material-338d21 exposes a comet decay rate; a decay that ends the run before the lap makes this seam fade moot in that case. Land the two consistently.
- 2026-09-28T11:05:34Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:59949da0-e47e-4228-9027-05bf99914943","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T11:05:34Z (materials-26.04): Landing with material-338d21 in .worktrees/material-338d21.
