---
id: material-ea6c37
title: glass-noise-saturation-smoke.sh's cross-binary identity check can no longer run
status: todo
priority: 2
size: s
created: 2026-09-06T12:01:30Z
updated: 2026-09-06T12:01:30Z
depends: []
tags: [harness, bug]
---

assert_zero omitted_identity_ae in docs/materials/scripts/glass-noise-saturation-smoke.sh compares full-frame captures from BASE against IMPL, to show that omitted noise/saturation values render byte-identically to the pre-change binary. Its original BASE for material-1293e8 necessarily predates f8bcb34c, the ring-of-light merge, and a pre-f8bcb34c binary draws no ring while the current one draws one at the window edge inside those frames - the ring contributes AE 3675 in this fixture, so the assertion cannot hold. Pinning response default focus none does not rescue it: FocusResponse does not parse before f8bcb34c, so the shared config would make BASE fail validation instead. The check is therefore unrunnable for the purpose it was written for, while still passing when BASE and IMPL are both recent. Decide what it should be: retire the BASE leg now that the migration it guarded has landed, re-point it at a post-ring baseline, or replace it with a same-binary determinism check. material-0af212 measured this and documented the constraint in the script header; it did not change the script's contract.
