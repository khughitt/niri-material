---
id: material-48b514
title: "Signal accent strength and edge tint, apart from the focus light"
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: material-48b514
created: 2026-10-02T10:32:21Z
updated: 2026-10-02T10:51:39Z
started: 2026-10-02T10:38:39Z
completed: 2026-10-02T10:51:39Z
depends: []
tags: [rendering, signals, prism]
agent: claude-code/claude-opus-5-5
---

Owner report (2026-10-02, live c93180be): with ring-rest 0, Claude Code windows still show two static outlines in their familiar hue. Screenshot-confirmed on the focused window (accent #5990cf): (1) the accent band at ring-gap, from accent "ring": main.frag glow = accentGlow * presence with accentGlow = (0.15 + 0.35 * level) * (1 + pulse * level), which no ring knob scales; (2) the bevel's outer glint tinted by the accent, from the default attention "rim-orbit" (specular mixed toward accent by light.z = level), which Prism never emits. Prism ties accent "ring" to colorSource familiar (integrations/niri/render.js), so the session hue is either a standing outline or absent. Want: (a) a native ring-accent level (0-3, default 1) that scales only the accent's own glow. At 0 the hue still colors the focus light (mix(ring_color, accent, presence) is unchanged), so the comet runs in the session's hue with no standing outline; Demand and Pulse still reach the glass through the rim glint. (b) Prism exposure: glass.ring.accent and an Edge tint switch emitting attention rim-orbit or none (prism task). Check: at ring-accent 0 and ring-rest 0, an unfocused window with a familiar signal renders like one without (attention none), and the focused comet keeps the accent color.

## Notes

- 2026-10-02T10:32:22Z (materials-26.04): Prism half: prism-f67834
- 2026-10-02T10:33:26Z (materials-26.04): Prism half: prism-f67834
- 2026-10-02T10:38:39Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T10:38:43Z (material-48b514): resumed
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T10:51:39Z (material-48b514): done
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T10:51:39Z (material-48b514): ring-accent (0-3, default 1) scales only the signal accent's own glow on the band (mat_sig_ring_accent); the accent still tints the focus light. ring_accent_zero_leaves_no_signal_outline: with ring-accent 0, ring-rest 0 and attention none an unfocused signalled window renders byte-identical to an unsignalled one, at ring-accent 1 the band shows, and the focused comet still differs with the signal. Config/uniform/shader tests; material-config.md and render-pipeline.md. Prism half prism-f67834.
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
