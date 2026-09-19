---
id: material-0e130e
title: Use one-shot focus ring effects; gate sustained attention motion on visibility and activity
status: done
priority: 1
size: m
complexity: high
process: planned
created: 2026-09-16T12:00:25Z
updated: 2026-09-19T22:33:56Z
completed: 2026-09-19T22:33:56Z
depends: [material-92edaf, material-82323e]
parent: material-5d6b2c
tags: [performance, dynamics, signals]
agent: codex
plan: docs/plans/2026-09-18-ring-focus-motion.md
---

Replace continuously pulsing or drifting ordinary focus-ring light with a finite one-shot effect, such as a single pulse on focus gain, then settle with no periodic redraw solely for focus. A static focus indication may remain. Alert/attention state may justify sustained dynamics only while BOTH the window is actually visible on an active workspace/output and the user is active rather than in extended keyboard/mouse inactivity. Preserve static alert state while hidden or idle; visibility/activity changes must not restart an endless focus pulse. Reuse existing impulse/envelope and motion-policy mechanisms where they fit. Design pulse duration/retrigger behavior, input-idle threshold, resume semantics and reduced-motion behavior before implementation. Verify ordinary focus settles, hidden/idle attention stops scheduling animation work, and visible active attention still signals appropriately. Coordinate with material-7afc31 (visibility), material-f86183 (inactivity settle), material-265eb0 (idle budget), and material-6d4de5 (dynamics review). Source: user request in render-order session, 2026-09-16.

Scope refinement (2026-09-18): material-82323e owns the next design pass; this implementation idea stays unstarted until its reviewed spec/plan. Evaluate the new ring from e79b226b, not the old face-masked shader. Existing slab_in_view/tick_deadline gating is evidence to reuse, not proof of complete visibility, occlusion or input-idle support. Bound this work to focus/attention ring scheduling; do not expand into every optic, hidden-client rendering or a generic animation framework. Require settled focus to stop ring-caused deadlines/uniform churn, not merely make amplitude zero. Prism wiring is part of the contract: preserve existing color sources and resolve what happens to driftHz; placement uses prism-d8ee06, light-ior prism-0ea68f. Pulse duration, retrigger/focus-loss, idle timeout and resume behavior remain design decisions. Handoff: docs/notes/2026-09-18-ring-next-steps-brief.md.

## Notes

- 2026-09-17T23:48:41Z (materials-26.04): Sequencing: scope against the within ring from material-92edaf (mask removed, roughness scatter, face placement), not the masked one; that child records motion frames under the existing drift and makes no motion judgment. Add a dependency on material-92edaf at scoping.
- 2026-09-18T23:28:05Z (materials-26.04): New within-ring motion evidence is retained for later scoping: $NIRI_MATERIAL_WORK_ROOT/render-order-old-ring.FH88OR/evidence/focus-ring/focus-ring-light-29e7df74 has paired pinned/drifting move+resize frames, adjacent request times and raw IPC target layouts. Animated slab geometry is unavailable, so whole-frame deltas are not interior reach or a motion-quality gate. Aurora motion frames are in render-order-within-resume.C2DA1f/run. Candidate rendering source is snapshotted under render-order-within-pixels.V12xVh; material-92edaf remains in acceptance, no motion behavior change/judgment in this child.
- 2026-09-18T23:57:25Z (materials-26.04): Within implementation is now committed as e79b226b on material-5b3107; its pixel, attenuation, selector, cadence and strict-cost acceptance passed. Final whole-branch review found no Critical/Important issue; documentation-only parent closeout follows. Scope motion against this new ring and retained FH88OR/C2DA1f frames, with unavailable animated geometry explicitly preserved. No motion judgment or behavior change was made by material-92edaf.
- 2026-09-19T00:26:26Z (materials-26.04): scope: briefed; separated visual placement, native motion design and existing/new Prism wiring; brief: docs/notes/2026-09-18-ring-next-steps-brief.md
- 2026-09-19T02:49:26Z (material-82323e): Design reviewed (docs/specs/2026-09-18-ring-focus-motion-design.md) and plan attached (docs/plans/2026-09-18-ring-focus-motion.md): one eased lap on focus gain over ring-sweep-ms on the animation loop, ending on the pinned pattern; ring-drift-hz retired with a parse error; sustained attention frozen after signal idle-after-ms (default 30 s) and resumed from the absolute clock. Four step children, ordered by deps; Task 4 runs on the headless host. Deviation to know: the smoke drives resume via a threshold reload, not injected input.
- 2026-09-19T03:00:21Z (material-82323e): Plan review applied: idle timer always drops and re-arms (no ToDuration) with a fixture test across an early fire and reloads; start eligibility (nonzero ring-sweep-ms) separated from the cut rule so a reload to 0 finishes the lap; phase test nondecreasing per ms with coarse strict checks; resume exercised by real input at both levels (virtual pointer in src/tests, wlrctl inside the nested smoke instance - wlrctl is now a host requirement).
- 2026-09-19T22:33:56Z (material-82323e): done
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T22:33:56Z (material-82323e): one eased lap on focus gain, settled focus costs nothing, attention frozen while idle; evidence 2026-09-18-ring-focus-motion-evidence.md
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
