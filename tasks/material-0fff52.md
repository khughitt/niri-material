---
id: material-0fff52
title: Decide whether a second desktop-idle pilot is worth a pre-registered interval rule
status: idea
priority: 2
created: 2026-10-27T10:38:07Z
updated: 2026-10-27T10:54:04Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code
origin: agent
---

The 2026-10-27 desktop-idle pilot (material-3d48b0, spec 2026-10-08-capture-host-conditions §8.2) failed by one half-tick: none-b1-1 median 0.230912 ms against I_none's upper bound 0.230400. I_none is ±1 GPU timestamp tick because the six TTY reference medians for 'none' had zero spread. A clock-sampled attribution rerun had all b medians inside I_c, and every draw after the start-up burst ran at the 210 MHz idle clock. It is not pass evidence, being sampled. No cause is named; the pilot's hold also left the monitor unwatched. Question for the owner: is admitting desktop-idle for nested measurements worth a new pilot under an interval rule fixed in advance (for example a floor on w_c of a few ticks, or fresh TTY references taken the same night), with the monitor confirmed lit at the hold so it is watched? Or does tty stay the route? Desktop-idle would let nested work run on nights the desktop is left up.

## Notes

- 2026-10-27T10:38:07Z (add): created
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"agent"}
