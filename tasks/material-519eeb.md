---
id: material-519eeb
title: Pin the owner's accepted ring look as a reference before ring/edge changes land
status: done
priority: 2
size: s
complexity: low
process: direct
owner: material-519eeb
created: 2026-10-02T00:14:51Z
updated: 2026-10-02T00:37:50Z
started: 2026-10-02T00:16:32Z
completed: 2026-10-02T00:37:49Z
depends: []
tags: [rendering, harness]
agent: claude-code/claude-opus-5-5
---

Owner-accepted look (2026-10-01, live a18ca619): ring-beam-speed 4350, noise 0.55 @ 12 Hz, decay 4150, ring-gap 6, ring-width 1.1, ring-glow 1.2, light-ior 4.5, color #ccccff, on the owner's Prism glass. Add a Glass row with these values to src/tests/ring_pair.rs so RING_PAIR_DUMP gives a before/after sheet for any change to ring or edge rendering, starting with material-be611b (glow follows the bevel profile) and material-1d70db (resting level). Prism half: save the same values as a named profile so experiments can be reverted; see prism-c45c6a.

## Notes

- 2026-10-02T00:16:32Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:16:35Z (material-519eeb): resumed
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:37:49Z (material-519eeb): Prism half not done by the agent: saving a profile is prism commit profile <name>, which moves the owner's 16 pending scratch edits on top of the loaded glass6, so it is the owner's call. Command: prism commit profile ring-accepted-2026-10-01
- 2026-10-02T00:37:49Z (material-519eeb): done
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:37:49Z (material-519eeb): src/tests/ring_look.rs accepted_ring_look: the owner's accepted terminal-glass pair rendered through a real focus gain (inactive, comet at 0/150/400/800 ms, rest) on a 1600x1200 pane (dark before the lap) and a 480x300 pane (laps); renders are frozen and byte-identical across runs; RING_LOOK_DUMP writes PNGs; checks are rest is quiet, the comet lights, the swap shows. Documented in material-config.md. Prism profile left to the owner (see note).
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
