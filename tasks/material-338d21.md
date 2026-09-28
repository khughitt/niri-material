---
id: material-338d21
title: "Ring beam: tunable decay so the comet can die before the lap closes"
status: doing
priority: 2
size: m
complexity: mid
process: direct
owner: materials-26.04
created: 2026-09-22T15:33:56Z
updated: 2026-09-28T11:11:42Z
started: 2026-09-28T11:05:34Z
depends: []
tags: [rendering, prism]
agent: claude-code/claude-opus-5
---

Owner goal (2026-09-22): adjust the comet's decay so it fades away completely before it completes a full circuit, with the rate exposed in Prism.

Today there is no decay knob at all. In src/render_helpers/material/ring.rs `BeamFrame.decay` is the shared head+tail brightness; the shipped `Envelope::Plateau` holds it at 1.0 for the whole run, and the alternative `Envelope::Splash` (kept for the sheets, `BEAM_ENVELOPE` selects) computes `(1 - t/run)^2` over `run = (P + tail_length(P)) / speed`. Splash therefore reaches zero only at the end of the tail drain, after the lap, so neither shape can satisfy the goal. `decay` already reaches the shader as `mat_sig_focus.w`, so main.frag likely needs no new uniform.

Wanted: a decay rate whose meaning does not depend on the window, so the comet's reach is a property of the beam rather than of the perimeter. Native param on the response block beside `ring-beam-speed`/`ring-gap`/`ring-glow` (niri-config, docs/materials/material-config.md), then Prism exposure in the Ring group as `glass.ring.decay` alongside `beamSpeed`/`gap`/`glow`, with the usual defs/migration/render-fixture/apply coverage.

Two constraints.
- Bounded motion and quiescence. The run ends today at `(P + L) / speed` (`BEAM_MAX_RUN` backstops a tile that never renders). When decay reaches zero before the lap the run must end there instead, so a fast decay shortens the redraw window rather than leaving the tile on a clock drawing nothing. The resting glow (`BEAM_REST`) is unaffected and must stay constant and redraw-free.
- The reduced/off motion policies that already skip the beam (signal { motion \"reduced\" }, motion \"off\", animations { off }) skip this with it; no new path around them.

Interaction with material-19cca1: the envelope's `env` fade-out starts at `lap - BEAM_FADE`, and that task clamps the fade when the lap is shorter than it. A decay that kills the comet before the lap makes the seam fade moot in that case. Land the two consistently rather than in isolation.

Open: whether the knob is a distance (px the comet travels before it is dark), which matches the stated goal most directly and is perimeter-independent by construction, or a half-life in seconds; and whether it replaces `Envelope::Splash` or generalises it (Plateau = an infinite decay length). Recommendation: a decay distance in px, with 0/unset meaning no decay so the shipped plateau look is the default.

Shares its plumbing with material-9704b0 (noise on the head's brightness): the same per-response float on ResolvedResponse, the same uniform vector, the same Prism Ring group. Cheaper to land in one worktree than twice.

## Notes

- 2026-09-22T15:34:01Z (materials-26.04): Filed from owner request 2026-09-22 alongside material-9704b0; both add a per-response beam-envelope float and a Prism Ring knob.
- 2026-09-28T11:05:34Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:59949da0-e47e-4228-9027-05bf99914943","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T11:05:34Z (materials-26.04): Decay knob chosen 2026-09-28 (owner approved the recommendation): ring-beam-decay as a distance in px the comet travels before it is dark, 0/unset = no decay (the plateau). Lands with material-19cca1 in one worktree, .worktrees/material-338d21.
- 2026-09-28T11:11:42Z (material-338d21): niri half landed in .worktrees/material-338d21: ring-beam-decay (px, (1 - d/D)^2 on the shared decay, run ends at min(P + L, D)), config tests, tile test, spec addendum, docs, beam-decay clip sequence. Next: Prism glass.ring.decay on a prism branch, held off prism main until the niri carrying it is installed.
