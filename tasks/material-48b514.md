---
id: material-48b514
title: "Signal accent strength and edge tint, apart from the focus light"
status: todo
priority: 1
size: s
complexity: mid
process: direct
created: 2026-10-02T10:32:21Z
updated: 2026-10-02T10:33:26Z
depends: []
tags: [rendering, signals, prism]
agent: claude-code/claude-opus-5-5
---

Owner report (2026-10-02, live c93180be): with ring-rest 0, Claude Code windows still show two static outlines in their familiar hue. Screenshot-confirmed on the focused window (accent #5990cf): (1) the accent band at ring-gap, from accent "ring": main.frag glow = accentGlow * presence with accentGlow = (0.15 + 0.35 * level) * (1 + pulse * level), which no ring knob scales; (2) the bevel's outer glint tinted by the accent, from the default attention "rim-orbit" (specular mixed toward accent by light.z = level), which Prism never emits. Prism ties accent "ring" to colorSource familiar (integrations/niri/render.js), so the session hue is either a standing outline or absent. Want: (a) a native ring-accent level (0-3, default 1) that scales only the accent's own glow. At 0 the hue still colors the focus light (mix(ring_color, accent, presence) is unchanged), so the comet runs in the session's hue with no standing outline; Demand and Pulse still reach the glass through the rim glint. (b) Prism exposure: glass.ring.accent and an Edge tint switch emitting attention rim-orbit or none (prism task). Check: at ring-accent 0 and ring-rest 0, an unfocused window with a familiar signal renders like one without (attention none), and the focused comet keeps the accent color.

## Notes

- 2026-10-02T10:32:22Z (materials-26.04): Prism half: prism-f67834
- 2026-10-02T10:33:26Z (materials-26.04): Prism half: prism-f67834
