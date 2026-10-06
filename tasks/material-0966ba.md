---
id: material-0966ba
title: "Capture script hygiene: signed_diff's sign and Tracy-client IPC in trace_run"
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: capture-script-hygiene
created: 2026-10-06T08:56:17Z
updated: 2026-10-06T12:34:55Z
started: 2026-10-06T12:33:36Z
completed: 2026-10-06T12:34:55Z
depends: []
tags: [material]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

signed_diff in glass-noise-site-smoke.sh and glass-noise-type-smoke.sh computes B - A + 0.5 (ImageMagick's Mathematics args apply to source minus destination), so grain-*.png is zero minus cell; sd is unaffected but any mean read from it has the wrong sign (it misled the 2026-10-05 residue analysis). Swap the operands or rename. glass-optic-smoke-lib.sh trace_run calls spawn_probe and steal_focus through niri-tracy, whose msg client spends ~0.5 s per call on Tracy calibration; use the plain binary as the IPC client as noise-placement-cost.sh now does.

## Notes

- 2026-10-06T12:33:36Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1ef0a4f0-97b2-444a-897f-191c9ef06a82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T12:33:45Z (capture-script-hygiene): resumed
  provenance: {"harness_session":"claude-code:1ef0a4f0-97b2-444a-897f-191c9ef06a82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T12:34:55Z (capture-script-hygiene): done
  provenance: {"harness_session":"claude-code:1ef0a4f0-97b2-444a-897f-191c9ef06a82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T12:34:55Z (capture-script-hygiene): signed_diff in the noise site and type smokes now computes a - b + 0.5 (compose args 0,-1,1,0.5, checked on 0.2/0.3 gray: 0.4), so grain-*.png is site minus zero; every consumer reads sd only, so no metric changes; the noise placement evidence note records that the 2026-10-05 images were zero minus site. trace_run drives spawn_probe and steal_focus through the plain binary, as noise-placement-cost.sh does.
  provenance: {"harness_session":"claude-code:1ef0a4f0-97b2-444a-897f-191c9ef06a82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
