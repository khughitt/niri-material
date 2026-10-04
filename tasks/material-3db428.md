---
id: material-3db428
title: Re-derive the ring sample row and reach bound in focus-ring-light.sh and glass-render-order-smoke.sh for ring-gap (measured from the face edge)
status: doing
priority: 3
size: s
complexity: mid
process: direct
needs: [quiet]
owner: materials-26.04
created: 2026-09-20T10:34:21Z
updated: 2026-10-04T03:38:13Z
started: 2026-10-04T01:56:01Z
depends: [material-22d78f]
parent: material-2834d7
tags: [rendering]
agent: claude-code/claude-opus-5
---

The ring beam (98013739) moved the filament band from the slab's outer edge to the face edge: `ring-gap` is measured from the face (the slab minus its chamfer), where the retired inset key was measured from the slab's outer edge. The key sweep in the measured old-ring scripts kept the slab-edge derivations: in docs/materials/scripts/focus-ring-light.sh the sample row `FIL_Y = SLAB_TOP + RING_GAP` and the `--inset $RING_GAP` passed to `glass-render-order-metrics.py reach` sit PIN_BEVEL (11) px above the band; in docs/materials/scripts/glass-render-order-smoke.sh `within_ring`, `profile_reach` and `attenuation_reach` pass the gap as the reach model's slab-edge `--inset`, so their bounds and the `profile_reach` row are the bevel (12) px short. The rest-confinement, selectors, resize-flex and within cases will fail or sample the wrong row until the row and `ring_bound` in glass-render-order-metrics.py are re-derived for a face-edge gap (band core at slab edge + bevel + gap; the shader caps the refracted shift at half the gap, not half the slab-edge distance). Neither script is in the ring beam plan's run list (Task 6 runs material-signals-smoke.sh cases and ring-motion-clips.sh), so this is not on that plan's path.

## Notes

- 2026-10-01T09:55:04Z (material-0e80c1): From material-0e80c1: focus-ring-light.sh pins no light-ior, jelly or noise, so it inherits Prism (light-ior 6, jelly-flex 0.0066), and case_tiny's comment cites the retired slabChamfer gate (the shader now gates on hasLine, main.frag). Band core measured at ring-gap inside the face edge; the face sits 2*offset in from the window's left/top and flush right/bottom at offset 6 (brief: docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring-findings).
- 2026-10-01T10:00:23Z (materials-26.04): Depends on material-22d78f, which retires case_resize_flex; skip re-deriving that case's row.
- 2026-10-04T01:56:01Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:d752873e-df16-40ec-ae7a-2e0b6a4ee888","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T02:55:16Z (materials-26.04): host pointer: stopped wali-rotate.timer and dropbox.service (both active; dropbox crash-loops in a TTY) for this task's captures, 22:56 EDT; restore with 'systemctl --user start wali-rotate.timer dropbox.service' before parking or closing.
- 2026-10-04T03:07:16Z (materials-26.04): run: focus-ring-light.sh pilot (rest-confinement) PASS, then rest-confinement/accent-midfade/selectors PASS (bin cc5480f3; sample row y=61, band peak y=60-61; reach 17.5 px from the face, bound 46; the old row y=50 sees ~1 code). glass-render-order-smoke within/pixels run 1: every reach case passed its face-edge bound; within-face-strip failed (its strip PX..PX+7 sat at the old slab-edge core; gap-20 core measured at PX+20) -> strip and within_opaque's ring ROI moved to PX+16. Run 2 refused at settle before within-pinned-on: gpu_pstate [P5, P8], the previous capture's GPU tail -> gpu_cooldown in the lib's settle_before_launch (same finding as material-124f1f).
- 2026-10-04T03:22:52Z (materials-26.04): host pointer restored: wali-rotate.timer and dropbox.service started again 23:23 EDT.
- 2026-10-04T03:22:53Z (materials-26.04): run: glass-render-order-smoke within/pixels run 3 ($NIRI_MATERIAL_WORK_ROOT/material-3db428/within-3, 23:07-23:22 EDT): PASS, 41/41 settles, cool-down 6-9 polls. Reach bounds from the face: pinned 37 (reach 16.5), dense 61, wide 61 (reach 30.5), rough 56, face 59 (reach 31.5); within-face-strip 3200 changed; opaque identity 0; pinned/rough FWHM 4/9. neutral_identity compared this commit's binary with itself (BASE = candidate), so it checks only capture determinism here. resize-flex not run: material-22d78f.
- 2026-10-04T03:28:24Z (ring-gap-reach): parked (waiting on agent, dependency): Agent: after material-22d78f lands, run focus-ring-light.sh CASES=resize-flex against branch ring-gap-reach (FIL_Y already re-derived) and close; the other cases passed 2026-10-03 (notes)
  provenance: {"harness_session":"claude-code:d752873e-df16-40ec-ae7a-2e0b6a4ee888","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T03:38:13Z (ring-gap-reach): review: impl round 1 — verdict: revise; findings: Important 1, Minor 2; reviewer: claude-code/claude-opus-5-5. Important: gpu_cooldown ran under a CAPTURE_META stub (HWA_REHEARSAL on a busy or non-NVIDIA host would fail after 60 s) -> skipped under a stub. Minor, not changed: callers with their own await_gpu_rest wait twice on real runs (harmless); a historical plan still shows --inset.
