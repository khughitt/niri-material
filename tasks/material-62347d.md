---
id: material-62347d
title: Design and build Noctalia glass cohesion with workspace project hue
status: doing
priority: 2
size: l
complexity: high
process: planned
owner: material-62347d
created: 2026-10-12T10:50:31Z
updated: 2026-10-17T13:47:07Z
started: 2026-10-17T02:10:41Z
depends: [material-e27dbc, material-6b1c0b]
parent: material-55380c
tags: []
source: "mindful:thought:d6cae16e5bd867ec390c14ab866e7619"
agent: codex
spec: docs/specs/2026-10-16-noctalia-glass-cohesion-design.md
plan: docs/plans/2026-10-17-noctalia-glass-cohesion.md
---

Explore applying the existing material treatment to other Noctalia surfaces, including the bar and panels, to improve visual cohesion. Identify suitable surfaces and integration constraints during scoping; the capture asks for cohesion without settling the visual treatment or implementation.

Owner addition (2026-10-15): beyond a static variant of the material, the shell surfaces should follow the hue of the workspace's active coding-agent terminal, the familiar identity hue its glass already carries, so the whole workspace, not only the terminal, reinforces the project being worked on. The brief's "focused hue" material frames the routes and the semantics left for the owner.

## Notes

- 2026-10-14T10:42:33Z (materials-26.04): scope: briefed; layer rules and MappedLayer have background effects but no material pass at d9914514; representative panel geometry and alpha remain unknown; first compare existing effects before renderer design; waits on material-e27dbc under material-55380c; brief: docs/notes/2026-10-14-noctalia-material-brief.md
- 2026-10-15T22:20:43Z (materials-26.04): scope: briefed; owner added focused-hue linkage (shell surfaces follow the active agent terminal's familiar accent); evidence: accent already on niri IPC (Window.signal, Workspace.active_window_id), Noctalia 5.2.1 plugins can color their own UI from niri's event stream but cannot write the palette; filed material-6b1c0b for the compositor binding and now wait on it and material-e27dbc; brief: docs/notes/2026-10-14-noctalia-material-brief.md
- 2026-10-15T22:26:11Z (research/material-6b1c0b): material-6b1c0b finding: niri can bind a layer surface's accent to its output's active window with existing layout state and redraw triggers (Monitor::active_window holds under panel keyboard focus); a layer needs a per-output crossfade, animation coverage and fingerprinted damage; the binding belongs inside a layer-material design. The drawing still waits on material-e27dbc's comparison.
- 2026-10-15T22:53:06Z (materials-26.04): review: brief round 1 — verdict: revise; findings: should-fix 2; reviewer: unverified (pasted by the owner)
- 2026-10-15T22:55:01Z (docs/noctalia-brief-review): brief revised after review round 1: optical treatment (comparison) and hue delivery are separate choices; a compositor tint under the client's translucent background is the only route to built-in chrome at Noctalia 5.2.1/5.3.0; the hold uses a per-workspace record of settled active windows, not focus timestamps
- 2026-10-16T00:01:42Z (research/material-e27dbc): material-e27dbc finding: control center (noctalia-attached-panel) is the first target, launcher (noctalia-panel) the second; Noctalia's ext-background-effect region gives the exact visible body for a footprint; existing effects reach translucency, grain, saturation and corner radius only (nested sheet attached to material-e27dbc, owner to judge A1-B3); hue delivery needs a compositor change regardless.
- 2026-10-17T02:03:13Z (materials-26.04): owner verdict 2026-10-16: column C of the material-b156e7 sheet (matched hue and darkness: custom palette with glass-colored surfaces, bar and attached panels 0.95, floating panels solid) looks viable; the optical treatment is Noctalia styling, no layer material needed for cohesion
- 2026-10-17T02:10:41Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T02:22:40Z (material-62347d): spec drafted: docs/specs/2026-10-16-noctalia-glass-cohesion-design.md — niri remaps Noctalia's surface family to the modelled glass face plus a per-workspace held hue (layer-rule surface-remap, Prism emits the colors); palette routes rejected: a Noctalia palette change runs ~17 templates and colors_changed (prism apply niri), restyles GTK/Qt, and loops under tintSource=noctalia. Next: owner reviews the spec, then writing-plans.
- 2026-10-17T02:23:04Z (material-62347d): parked (waiting on user, review): owner reviews docs/specs/2026-10-16-noctalia-glass-cohesion-design.md (branch material-62347d, .worktrees/material-62347d); on approval the agent runs writing-plans in that worktree
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T08:47:38Z (material-62347d): resumed
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T08:47:44Z (material-62347d): review: spec round 1 — verdict: revise; findings: must-fix 1, should-fix 2; reviewer: unverified (pasted by the owner)
- 2026-10-17T08:50:37Z (material-62347d): spec revised after round 1: crossfade and switch blend interpolate OKLab target families, never accents (no gray-midpoint snap); face model is a reference swatch over B_ref 0.2 with a supported range Y(T)(1-f0) <= 0.01, rejected at load outside it; surface-remap nodes are complete and replace whole; Prism omits the rule in light mode and owns hue-chroma
- 2026-10-17T08:50:58Z (material-62347d): parked (waiting on user, review): owner re-reviews docs/specs/2026-10-16-noctalia-glass-cohesion-design.md after round 1 fixes (commit 42bec1d0, .worktrees/material-62347d); on approval the agent runs writing-plans there
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T08:58:29Z (material-62347d): resumed
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T08:58:34Z (material-62347d): review: spec round 2 — verdict: revise; findings: should-fix 2; reviewer: unverified (pasted by the owner)
- 2026-10-17T09:00:14Z (material-62347d): spec revised after round 2: range check is max(T)(1-f0) <= 0.01 (bounds every channel for any backdrop); lightness bound derived across IOR 1-3, guaranteed -0.047/+0.045 (worst case IOR 1), with both counterexamples and a sampled sweep in verification; Prism emits the remap only when glass.enabled and shell.remap are both on
- 2026-10-17T09:00:25Z (material-62347d): parked (waiting on user, review): owner re-reviews docs/specs/2026-10-16-noctalia-glass-cohesion-design.md after round 2 fixes (.worktrees/material-62347d); on approval the agent runs writing-plans there
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T09:16:56Z (material-62347d): resumed
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T09:17:01Z (material-62347d): review: spec round 3 — verdict: accept; findings: minor 1; reviewer: unverified (pasted by the owner)
- 2026-10-17T09:17:07Z (material-62347d): owner (Keith) 2026-10-17: spec accepted for planning; section 8 defaults kept (hold last hue, 1 s dwell, hue only, bar + attached + floating panels)
- 2026-10-17T09:40:09Z (material-62347d): review: spec round 4 — verdict: revise; findings: must-fix 1, should-fix 2; reviewer: unverified (pasted by the owner, forwarded from session niri-material-97)
- 2026-10-17T09:44:05Z (material-62347d): spec revised after round 4: aurora and iridescence rejected for surface-remap (they light the flat face); achromatic accents (Oklab chroma < 0.02) resolve neutral; remap reaches xdg popups only, input-method popups exempt in the same per-kind match. Spec section 3 overlay claim corrected while planning (vivid primaries leave the family by 8% alpha). Plan written: docs/plans/2026-10-17-noctalia-glass-cohesion.md (9 tasks; Task 8 is the owner's sheet gate).
- 2026-10-17T09:44:26Z (material-62347d): parked (waiting on user, review): owner reviews the round-4 spec revision and the plan docs/plans/2026-10-17-noctalia-glass-cohesion.md (.worktrees/material-62347d, commit dfd38b25) and picks subagent-driven or native execution; the agent then runs it there
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:16:22Z (material-62347d): resumed
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:16:28Z (material-62347d): review: plan round 1 — verdict: revise; findings: must-fix 2, should-fix 1, minor 1; reviewer: unverified (pasted by the owner)
- 2026-10-17T10:21:35Z (material-62347d): plan revised after round 1: reload recomputes each mapped layer's rules before update_config (Glass-off panic) with removal/rename tests; family() resolves to neutral when hue-chroma is 0 or omitted, tested under accents and fades; remap damage pushed before layer and popup pixels whenever a remap exists, with test-client popup + opaque-region support and a render_to_vec ordering test; Oklab round-trip tolerance 1e-6
- 2026-10-17T10:22:03Z (material-62347d): parked (waiting on user, review): owner re-reviews the plan after round-1 fixes (docs/plans/2026-10-17-noctalia-glass-cohesion.md, commit a4ce44e3, .worktrees/material-62347d); on accept the agent runs it subagent-driven there
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:27:18Z (material-62347d): resumed
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:27:23Z (material-62347d): review: plan round 2 — verdict: accept; findings: minor 1; reviewer: unverified (pasted by the owner)
- 2026-10-17T10:27:41Z (material-62347d): owner (Keith) 2026-10-17: plan approved for execution after the round-2 fix (headless renderer setup in remap_damage_precedes_the_pixels_it_redraws); execution approach: subagent-driven; execution starts in a fresh session
- 2026-10-17T10:28:01Z (material-62347d): parked (waiting on agent, session): a fresh session resumes in .worktrees/material-62347d (branch material-62347d, commit 03ce5559): tasks start material-62347d, then run docs/plans/2026-10-17-noctalia-glass-cohesion.md with superpowers:subagent-driven-development, starting with its Setup (worktree skill baseline tests) and Task 1; Task 8 stops for the owner's sheet verdict
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:35:16Z (material-62347d): resumed
  provenance: {"harness_session":"claude-code:4e29fe84-ae01-4701-a4ac-da51f5fe6c77","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T10:57:38Z (material-62347d): execution: preflight scan of the plan found 5 blocking defects (T6 compile/test-time/count, T2 count, T8 $OUT) and undeclared spec deviations; rulings in the SDD ledger, carried into dispatches; baseline just test green (680 passed)
- 2026-10-17T13:47:02Z (material-62347d): pilot: matched+remap (pilot-1792244160) — both cases finished, no shader compile failure; control-center p{640,360}: remap 19,21,21 vs matched 20,21,21 (B matches A; both ~20, not the brief's guessed ~10). Full run sheet-1792244203: hue-quiet 27,18,16, hue-strong 32,15,12; strip bar pixel F0 12,22,29 (cyan) → F1 11,9,9 → F2 24,10,10 → F3..F11 31,16,16 (red): continuous blend through neutral over ~0.6 s, F3-F11 settled
- 2026-10-17T13:47:07Z (material-62347d): attached: noctalia-cohesion-sheet.png (1087405 bytes): Task 8 sheet: A approved look (control), B niri remap no hue, C hue-chroma 0.015, D hue-chroma 0.03; rows control center, launcher, bar
