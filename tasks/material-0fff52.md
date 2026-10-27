---
id: material-0fff52
title: Decide whether a second desktop-idle pilot is worth a pre-registered interval rule
status: idea
priority: 2
created: 2026-10-27T10:38:07Z
updated: 2026-10-27T11:34:15Z
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
- 2026-10-27T11:34:15Z (materials-26.04): Owner, 2026-10-27: asked whether gates should tolerate slight jitter, since background processes will run now and then even with every timer held. Agent recommendation: separate the two gate kinds. (1) Host-quiet gates (settle, await_load, disturbances) refuse a run but never produce a wrong number. Robustness there means a bounded automatic retry of a refused settle or load wait, not a looser threshold. Note that on 2026-09-24 the owner chose TTY runs over fixture-retried settles, so this reopens that choice. (2) Acceptance intervals compare medians with a reference. Widening them blindly would also absorb real changes of the same size. A principled pre-registered floor fits: GPU timestamps are quantized to q = 0.001024 ms, and even-count medians land on q/2, so an interval narrower than about ±2q tests quantization rather than the host. A floor of w_c ≥ 2q, or run-to-run spread measured under the candidate condition, would have kept none at about [0.2273, 0.2314]. The 1.5-tick miss is ~0.2% of the median and about 1% of the a/b difference these captures measure (0.287 vs 0.230 ms). Fix the rule in the spec before any re-pilot.
