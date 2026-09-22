---
id: material-9704b0
title: "Ring beam: tunable noise on the comet head's brightness"
status: doing
priority: 2
size: m
complexity: mid
process: direct
owner: material-9704b0
created: 2026-09-22T13:52:16Z
updated: 2026-09-22T15:43:36Z
started: 2026-09-22T15:43:35Z
depends: []
tags: [rendering, noise]
agent: claude-code/claude-opus-5
---

The beam head is a smooth Gaussian of constant amplitude (main.frag BEAM_HEAD_SIGMA/BEAM_BASE, envelope in src/render_helpers/material/ring.rs). Give its brightness a noise term with two knobs: intensity (how far the head's brightness wanders) and rate (how fast it wanders), so the comet reads as a living light rather than a moving lamp. Native params on the response block first, then Prism exposure in the Ring group alongside beamSpeed/gap/width/glow.

Two constraints the design must respect. The head noise applies only while the beam runs: the ring settles to a constant resting glow and costs no redraws until the next focus gain (a constant fingerprint, no deadline), and that quiescence must survive — the noise must not put the resting ring back on a clock. And the reduced/off motion policies that already skip the beam (signal { motion "reduced" }, motion "off", animations { off }) must skip the noise with it.

Open: whether the noise rides the head alone or the tail and resting glow too; whether rate is Hz or a spatial frequency along the arc; whether it is shared with the existing glass noise generators (material-3fcba2) or its own. Source: owner request 2026-09-22, after the ring follow-through closed.

## Notes

- 2026-09-22T15:35:37Z (materials-26.04): Scoped 2026-09-22. process=direct: the three open questions are bounded and the code answers them. (a) env already multiplies the head term alone in both ring.rs comet() and main.frag (decay multiplies head+tail together), so a head-brightness gain rides env — no new uniform, no shader formula change, and BeamFrame::REST stays exactly zero. (b) rate in Hz, evaluated CPU-side per frame in beam_frame as smooth value noise over elapsed time: the head is one point, so its brightness needs no per-fragment noise, and a free-running temporal wobble is what 'living light' asks for (a spatial-along-arc term would repeat identically every lap and across windows). (c) its own knob, not shared with material-3fcba2: those are per-fragment spatial grain on the sampled backdrop, this is a per-frame scalar. Seeded per window from jelly_seed so two focused panes do not flicker in lockstep. Quiescence holds by construction: env is 0 at rest and after the lap, so the noise term vanishes with the run and the resting glow keeps its constant fingerprint.
- 2026-09-22T15:43:36Z (material-9704b0): started
  provenance: {"harness_session":"claude-code:365f5b3e-15f6-430e-aa21-082d2a27e409","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
