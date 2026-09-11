---
id: material-9be53d
title: Animation system with named animation profiles
status: idea
priority: 2
created: 2026-09-11T00:47:47Z
updated: 2026-09-11T00:52:15Z
depends: []
tags: [quick-add, dynamics]
source: "mindful:thought:3f94e656b70f4e5585c1cb60c166e4da"
---

Should the dynamics (drag, move, idle micro-movement, ease-to-rest) be driven by an animation system with named profiles, and should prism own or resolve those profiles the way it resolves glass profiles (prism-2f0b4b, prism-9298b9)? Unscoped; if it becomes cross-project, file an ops goal with material and prism pieces.

Source: mindful:thought:3f94e656b70f4e5585c1cb60c166e4da

## Notes

- 2026-09-11T00:52:15Z (materials-26.04): Scoping lead: material-a54d89 shipped a signals model (`signal { motion }`, glass responses). Idle micro-movement and jostle-on-drag may be responses to existing or new signals rather than a new subsystem; then an animation profile is the set of responses a material subscribes to, which partly answers whether prism should own profiles.
