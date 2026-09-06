---
id: material-ea6c37
title: glass-noise-saturation-smoke.sh's cross-binary identity check can no longer run
status: done
priority: 2
size: s
owner: materials-26.04
created: 2026-09-06T12:01:30Z
updated: 2026-09-06T23:24:34Z
depends: []
tags: [harness, bug]
---

assert_zero omitted_identity_ae in docs/materials/scripts/glass-noise-saturation-smoke.sh compares full-frame captures from BASE against IMPL, to show that omitted noise/saturation values render byte-identically to the pre-change binary. Its original BASE for material-1293e8 necessarily predates f8bcb34c, the ring-of-light merge, and a pre-f8bcb34c binary draws no ring while the current one draws one at the window edge inside those frames - the ring contributes AE 3675 in this fixture, so the assertion cannot hold. Pinning response default focus none does not rescue it: FocusResponse does not parse before f8bcb34c, so the shared config would make BASE fail validation instead. The check is therefore unrunnable for the purpose it was written for, while still passing when BASE and IMPL are both recent. Decide what it should be: retire the BASE leg now that the migration it guarded has landed, re-point it at a post-ring baseline, or replace it with a same-binary determinism check. material-0af212 measured this and documented the constraint in the script header; it did not change the script's contract.

## Notes

- 2026-09-06T23:21:29Z (materials-26.04): Premise confirmed: pre-change pin 56aed303/7185ee51 is an ancestor of the ring merge f8bcb34c, so no BASE can serve the migration guard. Decision: retire the BASE leg (the 2026-09-05 evidence record stands as the guard's evidence) and replace the slot with omitted_determinism_ae, two nested sessions of the omitted fixture on the same binary asserted byte-identical, which states the precondition of every other zero assertion. Evidence doc and plan doc corrected in the same change.
- 2026-09-06T23:24:34Z (materials-26.04): Retired the BASE leg; the slot is now omitted_determinism_ae (two nested sessions of the omitted fixture, same binary, AE 0). Re-run on 740d062f passes with the seven carried metrics unchanged; negative control with animations on fails first on the new check (AE 1780.32). Evidence doc, plan doc, and design spec carry the correction.
