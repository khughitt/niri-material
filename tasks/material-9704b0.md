---
id: material-9704b0
title: "Ring beam: tunable noise on the comet head's brightness"
status: idea
priority: 2
created: 2026-09-22T13:52:16Z
updated: 2026-09-22T13:52:16Z
depends: []
tags: [rendering, noise]
agent: claude-code/claude-opus-5
---

The beam head is a smooth Gaussian of constant amplitude (main.frag BEAM_HEAD_SIGMA/BEAM_BASE, envelope in src/render_helpers/material/ring.rs). Give its brightness a noise term with two knobs: intensity (how far the head's brightness wanders) and rate (how fast it wanders), so the comet reads as a living light rather than a moving lamp. Native params on the response block first, then Prism exposure in the Ring group alongside beamSpeed/gap/width/glow.

Two constraints the design must respect. The head noise applies only while the beam runs: the ring settles to a constant resting glow and costs no redraws until the next focus gain (a constant fingerprint, no deadline), and that quiescence must survive — the noise must not put the resting ring back on a clock. And the reduced/off motion policies that already skip the beam (signal { motion "reduced" }, motion "off", animations { off }) must skip the noise with it.

Open: whether the noise rides the head alone or the tail and resting glow too; whether rate is Hz or a spatial frequency along the arc; whether it is shared with the existing glass noise generators (material-3fcba2) or its own. Source: owner request 2026-09-22, after the ring follow-through closed.
