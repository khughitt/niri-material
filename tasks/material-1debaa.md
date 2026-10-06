---
id: material-1debaa
title: Correct stale cost and ownership claims in older material designs
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: material-1debaa
created: 2026-10-06T15:07:38Z
updated: 2026-10-06T16:57:28Z
started: 2026-10-06T16:52:20Z
completed: 2026-10-06T16:57:28Z
depends: []
parent: material-5d6b2c
tags: [docs]
agent: claude-code/claude-opus-5-5
---

Found while writing docs/materials/performance.md (material-233295), verified against code at 958c2a10: the backdrop-blur design's line references (material.rs:576, tile.rs:1275/1460, effect_buffer.rs:148, niri.rs:4115-4123) and its render(frame, false) call are now material/mod.rs render_prefiltered and moved lines; the roughness design's Renderer ownership names blur.rs as owner of pyramid storage (it is effect_buffer.rs: Offscreen, prepare_prefilter) and its Cache and damage contract omits grain invalidation; 2026-09-02-material-signals-design.md §4 says the beam runs through are_transitions_ongoing (code: are_animations_ongoing since 06b71bb1); material-config.md 'Signal motion and animation' still calls focus-gain motion a sweep; the backdrop-grain config error still says 'one texture per output' (one per backdrop buffer; the test in niri-config/src/lib.rs pins the text). Designs are historical: correct with a dated note where rewriting would falsify the record.

## Notes

- 2026-10-06T16:52:20Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:52:24Z (material-1debaa): resumed
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:57:28Z (material-1debaa): done
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:57:28Z (material-1debaa): Corrected stale claims: dated code-location note in the backdrop-blur design; as-built note in the roughness design (effect_buffer.rs owns pyramid state and textures, grain invalidation, no partial rebuilds); signals design's beam paragraph names are_animations_ongoing; material-config.md says focus-gain beam and shared backdrop buffers; backdrop-grain agreement error now says 'backdrop grain is one setting per output' (test updated first). Noise-placement spec and plan keep the original quoted message as record.
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
